//! Warp is an alternative cast, a counterable next-end-step exile, then a paid
//! permission for the exact card exiled by that ability (CR702.185/608.3g).
use super::*;
use crate::cards::{CastFilter, StaticAbilityFragment, WarpSpec};

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct WarpedCard {
    pub object: ObjectRef,
    pub owner: Player,
    /// The turn of actual exile, which must end before this permission is usable.
    pub exiled_turn: u32,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct DelayedWarpExileView {
    pub id: u64,
    pub source: ObjectRef,
    pub source_card: Card,
    pub controller: Player,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct DelayedWarp {
    pub view: DelayedWarpExileView,
    keywords: Keywords,
    tapped: bool,
    facts: DamageFacts,
    attachment: Option<ObjectRef>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct WarpState {
    pub delayed: Vec<DelayedWarp>,
    pub cards: Vec<WarpedCard>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl WarpState {
    pub fn bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.delayed.len() * std::mem::size_of::<DelayedWarp>()
            + self.cards.len() * std::mem::size_of::<WarpedCard>()
    }
}
fn selected_spec(card: Card, granted_cost: Option<ManaCost>) -> WarpSpec {
    granted_cost
        .map(WarpSpec::fixed)
        .or_else(|| card.warp_spec())
        .expect("a warp announcement has an alternative cost")
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn configure_cast(item: &mut SpellView, granted_cost: Option<ManaCost>) -> WarpSpec {
    let card = item.source.expect("warp spell source").card;
    let spec = selected_spec(card, granted_cost);
    let cast = item.cast.as_mut().expect("warp cast");
    cast.method = CastMethod::Warp;
    cast.granted_warp = granted_cost.is_some();
    if spec.x_symbols == 0 && card.x_symbols() > 0 {
        item.x = Some(0);
    }
    spec
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl SpellView {
    pub(super) fn delayed_warp_controller(&self) -> Option<Player> {
        self.cast
            .filter(|cast| cast.method == CastMethod::Warp && !(cast.granted_warp && self.is_copy))
            .map(|_| self.controller)
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Announcement {
    pub(super) fn warp_spec(&self) -> WarpSpec {
        selected_spec(
            self.item.source.expect("warp spell source").card,
            self.origin_alternative_cost,
        )
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn hand_warp_grants(&self, actor: Player) -> Vec<(CastFilter, ManaCost)> {
        let mut grants = Vec::new();
        for &id in &self.state.static_sources {
            let source = &self.state.objects[id as usize];
            if source.zone != Zone::Battlefield || source.controller != actor {
                continue;
            }
            for fragment in source
                .card
                .static_ability_groups()
                .iter()
                .flat_map(|group| group.fragments)
            {
                let StaticAbilityFragment::GrantHandWarp { filter, cost } = *fragment else {
                    continue;
                };
                if !grants.contains(&(filter, cost))
                    && self.continuous_fragment_available(id, |found| std::ptr::eq(found, fragment))
                {
                    grants.push((filter, cost));
                }
            }
        }
        grants
    }

    pub(super) fn visit_warp_casts(
        &self,
        actor: Player,
        card: CardId,
        sorcery: bool,
        grants: &[(CastFilter, ManaCost)],
        emit: &mut impl FnMut(Action) -> bool,
    ) -> bool {
        let object = &self.state.objects[card as usize];
        if !matches!(object.zone, Zone::Hand | Zone::Graveyard)
            || object.owner != actor
            || !object.is_card()
            || !object.card.is_supported()
        {
            return false;
        }
        let fundable = |face, flash_surcharge, grant| {
            self.permitted_cast_with_alternative(
                actor,
                card,
                CastMethod::Warp,
                face,
                sorcery,
                flash_surcharge,
                None,
                None,
                None,
                false,
                cast_permissions::OriginAlternative {
                    granted_warp: grant,
                    ..Default::default()
                },
            )
        };
        if !object.card.is_land()
            && object
                .card
                .warp_spec()
                .is_some_and(|spec| object.zone == Zone::Hand || spec.from_graveyard)
            && fundable(CastFace::Normal, false, None)
            && emit(Action::AnnounceWarp { card })
        {
            return true;
        }
        if object.zone != Zone::Hand || grants.is_empty() {
            return false;
        }

        let back = if object.card.transforming_double_face() {
            None
        } else {
            object.card.back_face()
        };
        for (face, definition) in [
            (CastFace::Normal, Some(object.card)),
            (CastFace::Back, back),
        ] {
            let Some(definition) = definition.filter(|d| !d.is_land()) else {
                continue;
            };
            let mut offered = Vec::new();
            for &(filter, cost) in grants {
                if !filter.matches(object.card)
                    || !filter.matches(definition)
                    || offered.contains(&cost)
                {
                    continue;
                }
                offered.push(cost);
                for flash_surcharge in [false, true] {
                    if (!flash_surcharge || definition.flash_surcharge().is_some())
                        && fundable(face, flash_surcharge, Some(cost))
                        && emit(Action::AnnounceGrantedWarp {
                            card,
                            face,
                            flash_surcharge,
                            cost,
                        })
                    {
                        return true;
                    }
                }
            }
        }
        false
    }

    pub(super) fn start_warp_cast(
        &mut self,
        actor: Player,
        card: CardId,
        face: CastFace,
        flash_surcharge: bool,
        granted_cost: Option<ManaCost>,
    ) {
        let mut item = self.cast_item(actor, card, [0; 6]);
        self.set_cast_face(&mut item, face);
        let spec = configure_cast(&mut item, granted_cost);
        item.cast.as_mut().unwrap().flash_surcharge = flash_surcharge;
        self.open_payment_announcement_configured(
            item,
            Counters::default(),
            None,
            None,
            |announcement| {
                announcement.origin_alternative_cost = granted_cost;
                announcement.add_origin_life_cost(spec.life);
            },
        );
    }

    /// Called by the actual atomic entry commit, before its ordinary ETB observers.
    /// A countered spell never reaches this path. The exact entering incarnation
    /// remains the delayed ability's subject even when its source later leaves.
    pub(super) fn register_warp_exile(&mut self, event: &ZoneChange, controller: Player) {
        let source = self.object_view(event.after.card);
        let registration = DelayedWarp {
            view: DelayedWarpExileView {
                id: self.next_stack_id(),
                source: event.after,
                source_card: source.card,
                controller,
            },
            keywords: source.keywords,
            tapped: source.tapped,
            facts: self.damage_facts(event.after.card),
            attachment: source.attached_to,
        };
        let state = self.state.warp.get_or_insert_with(Default::default);
        state
            .delayed
            .try_reserve(1)
            .capacity("Warp delayed allocation exhausted");
        state.delayed.push(registration);
    }
    pub(super) fn queue_warp_exiles(&mut self) {
        let Some(state) = self.state.warp.as_mut() else {
            return;
        };
        let delayed = std::mem::take(&mut state.delayed);
        if state.cards.is_empty() {
            self.state.warp = None;
        }
        self.state
            .pending_triggers
            .try_reserve(delayed.len())
            .capacity("Warp trigger allocation exhausted");
        for d in delayed {
            let view = d.view;
            let id = self.next_stack_id();
            self.state.pending_triggers.push(SpellView {
                source: Some(StackSourceView {
                    id: view.source.card,
                    object: view.source,
                    card: view.source_card,
                    facts: d.facts,
                }),
                source_less_rule: None,
                additional_triggers: 0,
                source_move: None,
                borrowed_slot: None,
                self_granted_activation: None,
                target_controller_lki: None,
                target_types_lki: None,
                target_type_slots: None,
                gift: None,
                reflexive_subject: None,
                delayed_warp_exile: Some(Box::new(view)),
                return_link: None,
                crew: None,
                cast_event: None,
                cast_creature_subjects: None,
                copy_event: None,
                captured_artifact_copy: None,
                source_copy_recipe: None,
                source_ability_origin: None,
                target_allocations: None,
                announced_division: None,
                mana_copy_sources: None,
                is_copy: false,
                ninjutsu_defender: None,
                delayed_sacrifice: None,
                x: None,
                loyalty_x: None,
                reflexive_target_limit: None,
                attachment_trigger: None,
                cast: None,
                entering_object: None,
                evidence_collected: false,
                id,
                kind: StackKind::DelayedWarpExile,

                split_second: false,
                source_keywords: d.keywords,
                source_tapped: d.tapped,
                mana_tap_bonus: None,
                source_attachment: d.attachment,
                source_transform: None,

                related_stack: None,
                damage_event: None,
                damage_received: None,
                creature_combat_event: None,
                discard_event: None,
                draw_event: None,
                life_event: None,
                source_counter_payment: None,
                loyalty_counters_paid: 0,
                exiled_symbol_cards: None,
                craft_materials: None,
                paid_additional_cost: None,
                exiled_trigger_cards: None,
                source_counter_lki: None,
                source_quantity_lki: None,
                event_player: None,
                event_object: None,
                manifest_event: None,
                sacrificed_toughness: None,
                sacrificed_mana_value: None,
                sacrificed_power: None,

                targets: Vec::new(),
                mode: None,
                controller: view.controller,
            });
        }
    }
    pub(super) fn exile_warped_permanent(&mut self, item: &SpellView) {
        let subject = item
            .delayed_warp_exile
            .as_ref()
            .expect("Warp delayed context")
            .source;
        let Some(object) = self
            .state
            .objects
            .get(subject.card as usize)
            .filter(|o| o.zone == Zone::Battlefield && o.generation == subject.generation)
        else {
            return;
        };
        let owner = object.owner;
        let is_card = !object.is_token();
        let event = self.move_object(subject.card, Zone::Exile);
        if is_card && event.moved && event.destination == Zone::Exile {
            let state = self.state.warp.get_or_insert_with(Default::default);
            state
                .cards
                .try_reserve(1)
                .capacity("Warp permission allocation exhausted");
            let at = state
                .cards
                .partition_point(|p| p.object.card < event.after.card);
            state.cards.insert(
                at,
                WarpedCard {
                    object: event.after,
                    owner,
                    exiled_turn: self.state.turn,
                },
            );
        }
    }
    /// A card is warped from the moment its delayed exile succeeds. Only the
    /// separate recast permission waits for the next turn.
    pub(super) fn warped_card(&self, object: ObjectRef) -> Option<WarpedCard> {
        let state = self.state.warp.as_ref()?;
        let i = state
            .cards
            .binary_search_by_key(&object.card, |row| row.object.card)
            .ok()?;
        let permission = state.cards[i];
        let card = self.state.objects.get(object.card as usize)?;
        (permission.object == object
            && card.generation == object.generation
            && card.zone == Zone::Exile
            && card.is_card())
        .then_some(permission)
    }

    pub(super) fn can_cast_warped(&self, player: Player, card: CardId) -> bool {
        self.warped_card(self.object_ref(card))
            .is_some_and(|permission| {
                let object = &self.state.objects[card as usize];
                permission.owner == player
                    && object.owner == player
                    && self.state.turn > permission.exiled_turn
                    && !object.card.is_land()
                    && object.card.is_supported()
            })
    }
    pub(super) fn remove_warped(&mut self, object: ObjectRef) {
        if let Some(state) = self.state.warp.as_mut() {
            state.cards.retain(|p| p.object != object);
            if state.cards.is_empty() && state.delayed.is_empty() {
                self.state.warp = None;
            }
        }
    }
}
