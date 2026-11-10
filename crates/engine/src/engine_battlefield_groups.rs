//! Non-targeted battlefield sets, frozen before simultaneous movement or damage.
use super::*;
use crate::cards::{AnimationSpec, ControlledPermanentFilter, PermanentGroupCounters};
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn remove_abilities_exact(
        &mut self,
        recipients: &[ObjectRef],
        source: ObjectRef,
        controller: Player,
        duration: crate::cards::AnimationDuration,
    ) {
        let objects = &self.state.objects;
        let mut live = recipients
            .iter()
            .copied()
            .filter(|object| {
                objects.get(object.card as usize).is_some_and(|recipient| {
                    recipient.zone == Zone::Battlefield && recipient.generation == object.generation
                })
            })
            .peekable();
        if live.peek().is_none() {
            return;
        }
        let roles = self.state.roles.get_or_insert_with(Default::default);
        let timestamp = roles.next_timestamp();
        for recipient in live {
            match duration {
                crate::cards::AnimationDuration::Permanent => roles
                    .resolved_ability_modifications
                    .push(ResolvedAbilityModificationView {
                        recipient,
                        source,
                        keywords: Keywords::NONE,
                        timestamp,
                    }),
                crate::cards::AnimationDuration::EndOfTurn => {
                    roles.source_linked_effects.push(SourceLinkedEffectView {
                        recipient,
                        source,
                        controller,
                        kind: SourceLinkedEffectKind::RemoveAbilitiesUntilEndOfTurn,
                        timestamp,
                    })
                }
            }
        }
        self.refresh_copy_grant_availability();
    }
    pub(super) fn grant_chosen_creature_type_keywords(
        &mut self,
        item: &SpellView,
        other: bool,
        keywords: Keywords,
    ) {
        let Some(source) = item.source else { return };
        let typ = self
            .state
            .resolving
            .as_ref()
            .unwrap()
            .chosen_creature_type
            .expect("preceding creature-type choice");
        let recipients: Vec<_> = self
            .state
            .objects
            .iter()
            .enumerate()
            .filter(|(id, object)| {
                object.zone == Zone::Battlefield
                    && object.controller == item.controller
                    && (!other || self.object_ref(*id as CardId) != source.object)
                    && self.has_subtype(*id as CardId, typ.name())
            })
            .map(|(id, _)| id as CardId)
            .collect();
        self.grant_keyword_effects(&recipients, source.object, keywords);
    }
    pub(super) fn set_player_creature_characteristics(
        &mut self,
        item: &SpellView,
        players: PlayerSet,
        types: &'static [&'static str],
        power: i32,
        toughness: i32,
        remove: bool,
    ) {
        let Some(source) = item.source else { return };
        let recipients: Vec<_> = self
            .player_creature_recipients(players, item)
            .into_iter()
            .map(|id| self.object_ref(id))
            .collect();
        if remove {
            self.remove_abilities_exact(
                &recipients,
                source.object,
                item.controller,
                crate::cards::AnimationDuration::Permanent,
            );
        }
        for recipient in recipients {
            self.change_object_form(
                recipient,
                AnimationSpec {
                    artifact: false,
                    creature_types: crate::cards::AnimationCreatureTypes::Additional(types),
                    base: Some((i128::from(power), i128::from(toughness))),
                    keywords: Keywords::NONE,
                    duration: crate::cards::AnimationDuration::Permanent,
                },
                characteristic_layers::AnimationTypeMode::PreserveCardTypes,
            );
        }
    }
    pub(super) fn grant_permanent_group_keywords(
        &mut self,
        item: &SpellView,
        filters: &[BattlefieldFilter],
        keywords: Keywords,
        exclude_source: bool,
    ) {
        let Some(source) = item.source else {
            return;
        };
        let mut recipients = self.battlefield_group(item, filters);
        if exclude_source {
            recipients.retain(|&id| self.object_ref(id) != source.object);
        }
        self.grant_keyword_effects(&recipients, source.object, keywords);
    }
    pub(super) fn remove_permanent_group_keywords(
        &mut self,
        item: &SpellView,
        filters: &[BattlefieldFilter],
        keywords: Keywords,
    ) {
        let Some(source) = item.source else {
            return;
        };
        let keywords = keywords.without(keywords.resolution_rules());
        let recipients = self.battlefield_group(item, filters);
        if recipients.is_empty() || keywords == Keywords::NONE {
            return;
        }
        let timestamp = self.next_role_timestamp();
        for id in recipients {
            let recipient = self.object_ref(id);
            self.state
                .roles
                .as_mut()
                .unwrap()
                .keyword_effects
                .push(KeywordEffectView {
                    recipient,
                    origin: KeywordEffectOrigin::ResolutionRemoval {
                        source: source.object,
                    },
                    keywords,
                    rules: Keywords::NONE,
                    timestamp,
                    expires_turn: Some(self.state.turn),
                });
        }
    }
    pub(super) fn grow_permanent_group(
        &mut self,
        item: &SpellView,
        filters: &[BattlefieldFilter],
        power: i32,
        toughness: i32,
    ) {
        for id in self.battlefield_group(item, filters) {
            self.state.objects[id as usize].growth += Stats {
                power: i128::from(power),
                toughness: i128::from(toughness),
            };
        }
    }
    pub(super) fn grow_permanent_group_scaled(
        &mut self,
        item: &SpellView,
        filters: &[BattlefieldFilter],
        exclude_source: bool,
        amount: AmountValue,
    ) {
        let amount = i128::try_from(self.item_amount(amount, item))
            .capacity("permanent growth quantity exhausted");
        let affected: Vec<_> = self
            .battlefield_group(item, filters)
            .into_iter()
            .filter(|&id| {
                !exclude_source
                    || item
                        .source
                        .is_none_or(|source| self.object_ref(id) != source.object)
            })
            .collect();
        for id in affected {
            self.state.objects[id as usize].growth += Stats {
                power: amount,
                toughness: amount,
            };
        }
    }
    pub(super) fn grow_filtered_scaled(
        &mut self,
        item: &SpellView,
        filter: CountFilter,
        amount: AmountValue,
        power: i32,
        toughness: i32,
    ) {
        assert!(
            filter.zone == CountZone::Battlefield,
            "growth recipients are battlefield permanents"
        );
        let Some(source) = item.source else { return };
        let amount = i128::try_from(self.item_amount(amount, item))
            .capacity("filtered growth quantity exhausted");
        let delta = Stats {
            power: i128::from(power)
                .checked_mul(amount)
                .capacity("filtered power growth exhausted"),
            toughness: i128::from(toughness)
                .checked_mul(amount)
                .capacity("filtered toughness growth exhausted"),
        };
        let affected: Vec<_> = self
            .state
            .objects
            .iter()
            .enumerate()
            .filter(|(id, object)| {
                Self::count_object_matches(
                    filter,
                    source.object,
                    item.controller,
                    *id as CardId,
                    object,
                    &|id| self.granted_subtypes(id),
                    &|id, object| self.object_kinds(id, object),
                )
            })
            .map(|(id, _)| self.object_ref(id as CardId))
            .collect();
        for recipient in affected {
            self.state.objects[recipient.card as usize].growth += delta;
        }
    }
    pub(super) fn put_permanent_counters_scaled(
        &mut self,
        item: &SpellView,
        filters: &[BattlefieldFilter],
        kind: CounterKind,
        amount: AmountValue,
        legendary_only: bool,
    ) {
        let count = u64::try_from(self.item_amount(amount, item))
            .capacity("group counter quantity exhausted");
        let affected = self
            .battlefield_group(item, filters)
            .into_iter()
            .filter(|&id| {
                !legendary_only || ability_world::CharacteristicWorld::legendary(self, id)
            })
            .map(|id| {
                let mut counters = Counters::default();
                counters.add(kind, count);
                (self.object_ref(id), counters)
            })
            .collect();
        self.effect_counters(affected, replacements::ResumeMode::Program);
    }

    pub(super) fn change_controlled_permanent_group_counters(
        &mut self,
        controller: Player,
        filters: &[ControlledPermanentFilter],
        effect: PermanentGroupCounters,
    ) {
        use ability_world::CharacteristicWorld;
        let changes = self
            .state
            .objects
            .iter()
            .enumerate()
            .filter_map(|(id, object)| {
                let id = id as CardId;
                if object.zone != Zone::Battlefield
                    || object.controller != controller
                    || !filters.iter().any(|filter| {
                        (filter.all_kinds == 0
                            || self.current_kinds(id) & filter.all_kinds == filter.all_kinds)
                            && (!filter.legendary || CharacteristicWorld::legendary(self, id))
                            && filter
                                .subtype
                                .is_none_or(|subtype| self.has_subtype(id, subtype))
                    })
                {
                    return None;
                }
                let counters = match effect {
                    PermanentGroupCounters::Add { kind, amount } => {
                        Counters::of(kind, u64::from(amount))
                    }
                    PermanentGroupCounters::DoubleEachKind => self.counter_amounts(id).to_owned(),
                };
                Some((self.object_ref(id), counters))
            })
            .collect();
        self.effect_counters(changes, replacements::ResumeMode::Program);
    }

    pub(super) fn battlefield_filter_matches(
        &self,
        filter: BattlefieldFilter,
        actor: Player,
        id: CardId,
    ) -> bool {
        filter_matches_in_world(self, filter, actor, id)
    }
    pub(super) fn battlefield_group(
        &self,
        item: &SpellView,
        filters: &[BattlefieldFilter],
    ) -> Vec<CardId> {
        (0..self.state.objects.len())
            .filter_map(|id| {
                let id = id as CardId;
                filters
                    .iter()
                    .any(|&filter| self.battlefield_filter_matches(filter, item.controller, id))
                    .then_some(id)
            })
            .collect()
    }
    pub(super) fn exile_battlefield_until_source_leaves(
        &mut self,
        item: &SpellView,
        filters: &[BattlefieldFilter],
    ) {
        let Some(source) = item.source else {
            return;
        };
        let objects = self
            .battlefield_group(item, filters)
            .into_iter()
            .map(|id| self.object_ref(id))
            .collect::<Vec<_>>();
        self.exile_objects_until_source_leaves_origin(
            source.object,
            &objects,
            item.source_ability_origin,
        );
    }

    pub(super) fn move_battlefield_group(
        &mut self,
        item: &SpellView,
        filters: &[BattlefieldFilter],
        exile: bool,
    ) {
        let mut ids = self.battlefield_group(item, filters);
        if !exile {
            ids.retain(|&id| !self.effect_destruction_prevented(id));
        }
        let sources = if exile || ids.is_empty() {
            Vec::new()
        } else {
            self.death_trigger_sources()
        };
        let events = self.move_objects_raw(&ids, if exile { Zone::Exile } else { Zone::Graveyard });
        self.state.resolving.as_mut().unwrap().moved_count = Some(
            u64::try_from(events.iter().filter(|e| e.moved).count())
                .capacity("group result count exhausted"),
        );
        // Returns caused by the frozen departure event may suspend for entry
        // choices. The result count survives that pause before later instructions.
        self.finish_zone_events(&events, &sources);
    }
    pub(super) fn return_battlefield_group(
        &mut self,
        item: &SpellView,
        filters: &[BattlefieldFilter],
        attacking: bool,
        players: Option<PlayerSet>,
    ) {
        // Resolve the player target first, but do not target any group member:
        // hexproof, ward and shroud on those permanents do not apply.
        let controllers = players.map(|set| {
            let mut selected = [false; 2];
            for player in self.program_players(set, item) {
                selected[player.index()] = true;
            }
            selected
        });
        let ids: Vec<_> = self
            .battlefield_group(item, filters)
            .into_iter()
            .filter(|&id| {
                (!attacking || self.state.attackers.iter().any(|a| a.card == id))
                    && controllers.is_none_or(|selected| {
                        selected[self.state.objects[id as usize].controller.index()]
                    })
            })
            .collect();
        // One frozen group preserves simultaneous departure/LKI. The existing
        // zone mover returns stolen objects to their owners and handles tokens.
        let events = self.move_objects_raw(&ids, Zone::Hand);
        self.finish_zone_events(&events, &[]);
    }
    pub(super) fn program_damage_source(&self, item: &SpellView) -> DamageSource {
        let source = item.source.expect("damage instruction requires a source");
        if self.object_ref(source.id) == source.object {
            self.damage_source(source.id)
        } else {
            DamageSource {
                facts: source.facts,
                object: source.id,
                reference: source.object,
                controller: source.facts.controller,
                keywords: item.source_keywords,
            }
        }
    }
    pub(super) fn damage_with_target_controller(
        &mut self,
        item: &SpellView,
        damage: i128,
        controller_damage: i128,
    ) {
        let source = self.program_damage_source(item);
        let hit = |target, amount| DamageEvent {
            preventable: true,
            source,
            target,
            amount,
            combat: false,
        };
        let mut events = Vec::new();
        for &bound in &item.targets {
            if self.same_target_object(bound)
                && let Target::Card(id) = bound.target
                && self.on_battlefield(id)
            {
                let controller = self.state.objects[id as usize].controller;
                events.extend([
                    hit(bound.target, damage),
                    hit(Target::Player(controller), controller_damage),
                ]);
            }
        }
        self.apply_damage_batch(&events);
    }
    pub(super) fn damage_battlefield_group(
        &mut self,
        item: &SpellView,
        filters: &[BattlefieldFilter],
        players: Option<PlayerSet>,
        amount: i128,
        excluded_subtype: Option<&str>,
        target_amount: Option<i32>,
    ) {
        let source = self.program_damage_source(item);
        let hit = |target, amount| DamageEvent {
            preventable: true,
            source,
            target,
            amount,
            combat: false,
        };
        let mut events: Vec<_> = self
            .battlefield_group(item, filters)
            .into_iter()
            .filter(|&id| excluded_subtype.is_none_or(|subtype| !self.has_subtype(id, subtype)))
            .map(Target::Card)
            .chain(
                players
                    .into_iter()
                    .flat_map(|set| self.program_players(set, item).map(Target::Player)),
            )
            .map(|target| hit(target, amount))
            .collect();
        if let Some(amount) = target_amount.filter(|&amount| amount > 0) {
            for target in item
                .targets
                .iter()
                .filter(|&&bound| self.same_target_object(bound))
                .map(|bound| bound.target)
                .filter(|target| matches!(target, Target::Card(_) | Target::Player(_)))
            {
                // One source deals the combined amount to a shared recipient.
                if let Some(event) = events.iter_mut().find(|event| event.target == target) {
                    event.amount = event
                        .amount
                        .checked_add(i128::from(amount))
                        .capacity("simultaneous damage quantity exhausted");
                } else {
                    events.push(hit(target, i128::from(amount)));
                }
            }
        }
        self.apply_damage_batch(&events);
    }

    pub(super) fn damage_each_creature_by_departure_mana_value(&mut self, item: &SpellView) {
        let Some(EventObjectFacts::DepartureManaValue(amount)) =
            item.event_object.as_deref().map(|event| &event.facts)
        else {
            unreachable!("departure damage requires the saved battlefield mana value");
        };
        if *amount == 0 {
            return;
        }
        self.damage_battlefield_group(
            item,
            &[BattlefieldFilter {
                kinds: 1,
                relation: -1,
                nonland: false,
                tapped: None,
                without: Keywords::NONE,
                subtype: None,
                max_mana: u32::MAX,
            }],
            None,
            i128::from(*amount),
            None,
            None,
        );
    }
    /// Freeze the complete recipients before the simultaneous damage event.
    pub(super) fn damage_creatures_except_controlled_tokens(
        &mut self,
        item: &SpellView,
        amount: i32,
    ) {
        let source = self.program_damage_source(item);
        let events: Vec<_> = self
            .state
            .objects
            .iter()
            .enumerate()
            .filter_map(|(id, o)| {
                (o.zone == Zone::Battlefield
                    && self.is_creature(id as CardId)
                    && !(o.is_token() && o.controller == item.controller))
                    .then_some(DamageEvent {
                        source,
                        target: Target::Card(id as CardId),
                        amount: i128::from(amount),
                        combat: false,
                        preventable: true,
                    })
            })
            .collect();
        self.apply_damage_batch(&events);
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn animate_permanent_group(
        &mut self,
        item: &SpellView,
        filters: &[BattlefieldFilter],
        spec: AnimationSpec,
    ) {
        let recipients: Vec<_> = self
            .battlefield_group(item, filters)
            .into_iter()
            .map(|id| self.object_ref(id))
            .collect();
        for recipient in recipients {
            self.change_object_form(
                recipient,
                spec,
                characteristic_layers::AnimationTypeMode::AddCreature,
            );
        }
    }
    pub(super) fn return_graveyard_with_delayed_sacrifice(
        &mut self,
        item: &SpellView,
        filter: crate::SearchFilter,
    ) {
        let moves = self
            .state
            .objects
            .iter()
            .enumerate()
            .filter(|(id, object)| {
                object.zone == Zone::Graveyard
                    && object.owner == item.controller
                    && object.is_card()
                    && self.object_search_filter_matches(filter, *id as CardId)
            })
            .map(|(id, _)| {
                let mut movement = self.entry_move(id as CardId, Zone::Battlefield, false);
                movement.controller = item.controller;
                movement
            })
            .collect();
        self.stage_entries_with_delayed_sacrifice(moves, item);
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn filter_matches_in_world<W: ability_world::CharacteristicWorld + ?Sized>(
    world: &W,
    filter: BattlefieldFilter,
    actor: Player,
    id: CardId,
) -> bool {
    let object = world.object(id);
    object.zone == Zone::Battlefield
        && (filter.kinds == 0 || world.kinds(id) & filter.kinds != 0)
        && (filter.relation == -1 || (object.controller == actor) == (filter.relation == 0))
        && (!filter.nonland || world.kinds(id) & 2 == 0)
        && filter.tapped.is_none_or(|tapped| object.tapped == tapped)
        && (filter.without == Keywords::NONE
            || !characteristic_query::CharacteristicQuery::in_world(world)
                .keywords(id)
                .intersects(filter.without))
        && filter
            .subtype
            .is_none_or(|subtype| world.subtypes(id).has(world.definition(id), subtype))
        && (filter.max_mana == u32::MAX || world.mana_value(id) <= u64::from(filter.max_mana))
}
