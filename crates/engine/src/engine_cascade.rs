//! Cascade uses a frozen cast event and exact public exiled incarnations.
use super::*;
use crate::cards::{CastFilter, StaticAbilityFragment};
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CascadeView {
    pub controller: Player,
    pub maximum_mana_value: u64,
    pub exiled: Vec<ObjectRef>,
    /// Discover puts this card into its owner's hand if it was not cast.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discovered: Option<ObjectRef>,
    /// A Breaching-style instruction leaves the rest of its exiles in exile.
    #[serde(skip_serializing_if = "is_false")]
    pub keep_exiled: bool,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct DiscoverResult {
    amount: u64,
    hit_mana_value: Option<u64>,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum CascadeState {
    Active {
        view: CascadeView,
        discover: Option<DiscoverResult>,
    },
    Discovered(DiscoverResult),
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl CascadeState {
    pub(super) fn view(&self) -> Option<&CascadeView> {
        match self {
            Self::Active { view, .. } => Some(view),
            Self::Discovered(_) => None,
        }
    }

    pub(super) fn owned_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.view().map_or(0, |view| {
                view.exiled.len() * std::mem::size_of::<ObjectRef>()
            })
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    /// A granted spell ability belongs to the spell, with immutable provenance
    /// from the granting permanent. Later loss of that permanent cannot erase it.
    pub(super) fn queue_granted_spell_trigger(
        &mut self,
        spell: &SpellView,
        grant: AttachmentTriggerView,
        ability: u16,
    ) {
        let mut trigger = spell.clone();
        trigger.id = self.next_stack_id();
        trigger.kind = StackKind::TriggeredAbility { ability };
        trigger.targets.clear();
        trigger.target_type_slots = None;
        trigger.mode = None;
        trigger.cast_event = spell.actual_cast_event().map(Box::new);
        trigger.cast = None;
        trigger.is_copy = false;
        trigger.related_stack = None;
        trigger.source_ability_origin = grant.origin;
        trigger.attachment_trigger = Some(Box::new(grant));
        self.state.pending_triggers.push(trigger);
    }

    pub(super) fn queue_hand_cascade(&mut self, index: usize) {
        let spell = &self.state.stack[index];
        if spell.is_copy
            || !spell.cast.is_some_and(|c| c.origin == Zone::Hand)
            || !spell
                .source
                .is_some_and(|source| CastFilter::InstantSorcery.matches(source.card))
        {
            return;
        }
        let spell = spell.clone();
        let mut grants = Vec::new();
        for &id in &self.state.static_sources {
            let source = self.object_view(id);
            if source.zone != Zone::Battlefield
                || source.controller != spell.controller
                || !source.printed_abilities_available()
            {
                continue;
            }
            for group in source.card.static_ability_groups() {
                for fragment in group.fragments {
                    if let StaticAbilityFragment::CascadeFromHand { ability } = *fragment {
                        grants.push((
                            AttachmentTriggerView {
                                grantor: source.reference,
                                granting_card: source.card,
                                recipient: spell.source.expect("cast spell has source").object,
                                defender: None,
                                granted: true,
                                origin: source.copied_grant_origin(),
                            },
                            ability,
                        ));
                    }
                }
            }
        }
        for (grant, ability) in grants {
            self.queue_granted_spell_trigger(&spell, grant, ability);
        }
    }

    pub(super) fn start_cascade(&mut self, item: &SpellView) -> bool {
        let maximum = item
            .cast_event
            .as_ref()
            .map_or_else(|| item.mana_value(), |event| event.mana_value);
        self.start_cascade_or_discover(item.controller, Some(maximum), None)
    }

    pub(super) fn start_discover(&mut self, player: Player, amount: u64) -> bool {
        self.start_cascade_or_discover(player, amount.checked_add(1), Some(amount))
    }

    pub(super) fn counter_target_then_discover(&mut self, item: &SpellView) -> bool {
        let Some((id, amount)) = item.targets.iter().find_map(|bound| {
            let target = self.bound_stack_item(bound)?;
            (target.kind == StackKind::Spell).then(|| (target.id, target.mana_value()))
        }) else {
            return false;
        };
        self.counter_stack_item(id, false);
        self.start_discover(item.controller, amount)
    }

    pub(super) fn shuffle_target_then_discover(&mut self, item: &SpellView) -> bool {
        let Some(target) = item
            .targets
            .iter()
            .find(|&&bound| self.valid_library_target(item, bound))
            .and_then(|bound| bound.object)
        else {
            return false;
        };
        let owner = self.state.objects[target.card as usize].owner;
        let amount = self.object_mana_value(target.card);
        self.shuffle_target_into_library(item);
        self.start_discover(owner, amount)
    }

    fn start_cascade_or_discover(
        &mut self,
        player: Player,
        maximum: Option<u64>,
        discover: Option<u64>,
    ) -> bool {
        let mut exiled = Vec::new();
        let mut eligible = None;
        let mut hit_mana_value = None;
        while let Some(card) = self.top_library_card(player) {
            let definition = self.state.objects[card as usize].card;
            let events = self.move_objects_raw(&[card], Zone::Exile);
            let moved = events
                .iter()
                .find(|event| event.moved && event.destination == Zone::Exile)
                .map(|event| event.after);
            self.finish_zone_events(&events, &[]);
            if let Some(object) = moved {
                exiled.push(object);
                let mana_value = definition.fixed_mana_value();
                if !definition.is_land() && maximum.is_none_or(|limit| mana_value < limit) {
                    eligible = Some(object);
                    hit_mana_value = Some(mana_value);
                    break;
                }
            } else if self.top_library_card(player) == Some(card) {
                // A prevented movement cannot make progress through this library.
                break;
            }
        }
        self.state.resolving.as_mut().unwrap().cascade = Some(Box::new(CascadeState::Active {
            view: CascadeView {
                controller: player,
                maximum_mana_value: maximum.unwrap_or(u64::MAX),
                exiled,
                discovered: discover.and(eligible),
                keep_exiled: false,
            },
            discover: discover.map(|amount| DiscoverResult {
                amount,
                hit_mana_value,
            }),
        }));
        if let Some(object) = eligible {
            self.state.resolution_free_casts = Some(Box::new(ResolutionFreeCasts {
                discard_instead_of_mana: false,
                permanent_type: None,
                permanent_spell_only: false,
                uncast_to_hand: false,
                siege_transformed: false,
                spell_kinds: 0,
                paid: None,
                casts_remaining: None,
                pending_cast: None,
                remaining_mana_value: None,
                alternative_cost: None,
                maximum_mana_value: maximum,
                origin: Zone::Exile,
                exile_on_graveyard: false,
                allow_lands: false,
                player,
                remaining: vec![object],
            }));
            self.state.pending = Pending::ResolutionCasts { player };
            return true;
        }
        self.finish_cascade();
        false
    }

    pub(super) fn start_exile_until_nonland_cast_or_hand(
        &mut self,
        item: &SpellView,
        maximum_mana_value: u64,
    ) -> bool {
        let mut exiled = Vec::new();
        let mut found = None;
        while let Some(card) = self.top_library_card(item.controller) {
            let nonland = !self.state.objects[card as usize].card.is_land();
            let events = self.move_objects_raw(&[card], Zone::Exile);
            let moved = events
                .iter()
                .find(|e| e.moved && e.destination == Zone::Exile)
                .map(|event| event.after);
            self.finish_zone_events(&events, &[]);
            if let Some(object) = moved {
                exiled.push(object);
                if nonland {
                    found = Some(object);
                    break;
                }
            } else if self.top_library_card(item.controller) == Some(card) {
                break;
            }
        }
        let Some(object) = found else {
            return false;
        };
        self.state.resolving.as_mut().unwrap().cascade = Some(Box::new(CascadeState::Active {
            view: CascadeView {
                controller: item.controller,
                maximum_mana_value,
                exiled,
                discovered: Some(object),
                keep_exiled: true,
            },
            discover: None,
        }));
        self.state.resolution_free_casts = Some(Box::new(ResolutionFreeCasts {
            discard_instead_of_mana: false,
            permanent_type: None,
            permanent_spell_only: false,
            uncast_to_hand: false,
            siege_transformed: false,
            spell_kinds: 0,
            paid: None,
            casts_remaining: Some(1),
            pending_cast: None,
            remaining_mana_value: None,
            alternative_cost: None,
            maximum_mana_value: maximum_mana_value.checked_add(1),
            origin: Zone::Exile,
            exile_on_graveyard: false,
            allow_lands: false,
            player: item.controller,
            remaining: vec![object],
        }));
        self.state.pending = Pending::ResolutionCasts {
            player: item.controller,
        };
        true
    }
    pub(super) fn finish_cascade(&mut self) {
        if self.state.resolution_free_casts.is_some() {
            return;
        }
        let Some(state) = self.state.resolving.as_mut().and_then(|r| {
            r.cascade
                .take_if(|state| matches!(state.as_ref(), CascadeState::Active { .. }))
        }) else {
            return;
        };
        let CascadeState::Active { view, discover } = *state else {
            unreachable!("only active cascades finish")
        };
        if let Some(object) = view.discovered
            && self.object_ref(object.card) == object
            && self.state.objects[object.card as usize].zone == Zone::Exile
        {
            let event = self.move_object(object.card, Zone::Hand);
            if event.moved && event.destination == Zone::Hand {
                self.remember_public_hand(event.after);
            }
        }
        if !view.keep_exiled {
            self.bottom_cascade_exiles(&view);
        }
        if let Some(result) = discover {
            self.state.resolving.as_mut().unwrap().cascade =
                Some(Box::new(CascadeState::Discovered(result)));
            self.queue_discover_triggers(view.controller, result.amount);
        }
    }

    fn bottom_cascade_exiles(&mut self, view: &CascadeView) {
        let mut cards: Vec<_> = view
            .exiled
            .iter()
            .filter(|&&r| {
                self.object_ref(r.card) == r
                    && self.state.objects[r.card as usize].zone == Zone::Exile
            })
            .map(|r| r.card)
            .collect();
        if cards.is_empty() {
            return;
        }
        self.state.rng.shuffle(&mut cards);
        let player = view.controller;
        self.forget_library_top(player);
        let events = self.move_objects_raw(&cards, Zone::Library);
        let returned: Vec<_> = events
            .iter()
            .filter(|e| e.moved && e.destination == Zone::Library)
            .map(|e| e.after.card)
            .collect();
        let library = &mut self.state.players[player.index()].library;
        library.retain(|id| !returned.contains(id));
        library.splice(0..0, returned);
        self.finish_zone_events(&events, &[]);
    }

    pub(super) fn discover_mana_value_shortfall(&self) -> u128 {
        let Some(CascadeState::Discovered(result)) = self
            .state
            .resolving
            .as_ref()
            .and_then(|r| r.cascade.as_deref())
        else {
            return 0;
        };
        result
            .hit_mana_value
            .map_or(0, |mana_value| u128::from(result.amount - mana_value))
    }

    fn queue_discover_triggers(&mut self, player: Player, amount: u64) {
        let sources = self.indexed_trigger_sources(&self.state.draw_sources);
        for (source, grant) in sources {
            if source.zone != Zone::Battlefield || source.controller != player {
                continue;
            }
            let definition = grant.as_ref().map_or(source.card, |row| row.granting_card);
            for (index, ability) in definition.triggers().iter().enumerate() {
                if ability.condition.event() == TriggerCondition::DiscoveredYou
                    && self.queue_trigger_from(&source, index, grant.map(Box::new))
                {
                    self.state
                        .pending_triggers
                        .last_mut()
                        .unwrap()
                        .set_event_object(EventObjectView {
                            object: source.reference,
                            facts: EventObjectFacts::DiscoverAmount(amount),
                            semantic_objects: None,
                            conniver_creature: None,
                            subtype_lki: None,
                        });
                }
            }
        }
    }
}
