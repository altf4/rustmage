//! Sparse exact-incarnation grants reuse static programs and owned trigger packets.
use super::*;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct TemporaryAbilityView {
    pub recipient: ObjectRef,
    /// Historical spell/ability source; departure never revokes the grant.
    pub grantor: ObjectRef,
    pub granting_card: Card,
    pub ability: u16,
    pub grantor_controller: Player,
    pub expires_turn: u32,
    pub permanent: bool,
    pub timestamp: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin: Option<u64>,
}

impl TemporaryAbilityView {
    fn context(self) -> AttachmentTriggerView {
        AttachmentTriggerView {
            grantor: self.grantor,
            granting_card: self.granting_card,
            recipient: self.recipient,
            defender: None,
            granted: true,
            origin: self.origin,
        }
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn grant_source_permanent_trigger(
        &mut self,
        item: &SpellView,
        ability: u16,
        timestamp: u64,
    ) {
        let Some(source) = item.source else {
            return;
        };
        self.grant_permanent_triggers(item, source.object, &[ability], timestamp);
    }

    pub(super) fn grant_permanent_triggers(
        &mut self,
        item: &SpellView,
        recipient: ObjectRef,
        abilities: &[u16],
        timestamp: u64,
    ) {
        let Some(source) = item.source else {
            return;
        };
        let granting_card = item.trigger_definition();
        let turn = self.state.turn;
        self.state
            .temporary_abilities
            .get_or_insert_with(|| Box::new(Vec::new()))
            .extend(abilities.iter().map(|&ability| {
                assert!(
                    granting_card.triggers()[usize::from(ability)]
                        .condition
                        .is_granted()
                );
                TemporaryAbilityView {
                    recipient,
                    grantor: source.object,
                    granting_card,
                    ability,
                    grantor_controller: item.controller,
                    expires_turn: turn,
                    permanent: true,
                    timestamp,
                    origin: Some(timestamp),
                }
            }));
    }
    /// Each stored grant identifies one ability. Scanning its definition's whole
    /// trigger array would duplicate independent grants from the same card.
    pub(super) fn temporary_cast_triggers(
        &mut self,
        caster: Player,
        card: Card,
        stack: Option<usize>,
    ) {
        let Some(rows) = self.state.temporary_abilities.as_ref() else {
            return;
        };
        let rows = rows.to_vec();
        let event = stack.and_then(|index| {
            let item = &self.state.stack[index];
            item.cast
                .zip(item.source)
                .map(|(cast, source)| CastPaymentEvent {
                    mana_value: item.mana_value(),
                    stack_id: item.id,
                    object: source.object,
                    card: source.card,
                    controller: item.controller,
                    payment: cast.payment,
                    treasure_mana_spent: cast.treasure_mana_spent,
                    web_slinging_return_mana_value: cast.web_slinging_return_mana_value,
                })
        });
        let owner = event
            .as_ref()
            .map(|e| self.state.objects[e.object.card as usize].owner);
        let origin = stack
            .and_then(|index| self.state.stack[index].cast)
            .map(|cast| (cast.origin, owner == Some(caster)));
        for grant in rows {
            if !(grant.permanent || grant.expires_turn == self.state.turn)
                || !self.on_battlefield(grant.recipient.card)
                || self.object_ref(grant.recipient.card) != grant.recipient
            {
                continue;
            }
            let ability = grant.granting_card.triggers()[usize::from(grant.ability)];
            let TriggerCondition::SpellCast {
                relation,
                filter,
                ordinal,
            } = ability.condition.event()
            else {
                continue;
            };
            let source = self.object_view(grant.recipient.card);
            let related = match relation {
                CastRelation::You => source.controller == caster,
                CastRelation::Opponent => source.controller != caster,
                CastRelation::Any => true,
            };
            if !related
                || !ability.condition.matches_play_origin(origin)
                || !source.grant_available(
                    grant.grantor,
                    grant.granting_card,
                    grant.ability,
                    Some(grant.timestamp),
                )
                || ordinal
                    .is_some_and(|n| n != self.state.players[caster.index()].spells_cast_this_turn)
                || !self.cast_payment_condition(ability.condition, source.reference, event.as_ref())
                || (ability.condition.requires_opponents_turn()
                    && self.state.active == source.controller)
                || !self.cast_filter_matches_value(
                    owner,
                    filter,
                    card,
                    event.map_or_else(|| card.fixed_mana_value(), |e| e.mana_value),
                    caster,
                    &source,
                    stack.map_or(&[][..], |i| self.state.stack[i].targets.as_slice()),
                )
            {
                continue;
            }
            let context = Box::new(AttachmentTriggerView {
                grantor: grant.grantor,
                granting_card: grant.granting_card,
                recipient: grant.recipient,
                defender: None,
                granted: true,
                origin: grant.origin,
            });
            if self.queue_trigger_from(&source, usize::from(grant.ability), Some(context)) {
                let chosen_x = stack.and_then(|i| self.state.stack[i].x);
                let objects = stack.map_or_else(Vec::new, |i| {
                    self.cast_semantic_targets(
                        self.state.stack[i]
                            .source
                            .map(|s| self.state.objects[s.id as usize].owner),
                        filter,
                        card,
                        event.map_or_else(|| card.fixed_mana_value(), |e| e.mana_value),
                        caster,
                        &source,
                        &self.state.stack[i].targets,
                    )
                });
                let item = self.state.pending_triggers.last_mut().unwrap();
                item.refer_objects(objects);
                item.x = chosen_x;
                item.cast_event = event.map(Box::new);
            }
        }
    }
    pub(super) fn grant_temporary_ability(&mut self, item: &SpellView, ability: u16) {
        let ids = item
            .targets
            .iter()
            .filter(|&&bound| self.same_target_object(bound))
            .filter_map(|bound| {
                if let Target::Card(id) = bound.target {
                    self.on_battlefield(id).then_some(id)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        self.grant_temporary_ability_to(item, ability, &ids);
    }
    pub(super) fn grant_temporary_ability_to(
        &mut self,
        item: &SpellView,
        ability: u16,
        ids: &[CardId],
    ) {
        let Some(source) = item.source else {
            return;
        };
        let granting_card = item.trigger_definition();
        assert!(
            granting_card.triggers()[usize::from(ability)]
                .condition
                .is_granted()
        );
        let timestamp = self.next_role_timestamp();
        for &id in ids {
            let grant = TemporaryAbilityView {
                recipient: self.object_ref(id),
                grantor: source.object,
                granting_card,
                ability,
                grantor_controller: item.controller,
                expires_turn: self.state.turn,
                permanent: false,
                timestamp,
                origin: Some(timestamp),
            };
            self.state
                .temporary_abilities
                .get_or_insert_with(|| Box::new(Vec::new()))
                .push(grant);
        }
    }
    pub(super) fn temporary_land_entry_triggers(
        &mut self,
        event: &ZoneChange,
        entered: &ObjectView,
    ) {
        if entered.kinds & 2 == 0 {
            return;
        }
        let grants: Vec<_> = self
            .state
            .temporary_abilities
            .as_deref()
            .into_iter()
            .flatten()
            .copied()
            .filter(|grant| {
                (grant.permanent || grant.expires_turn == self.state.turn)
                    && self.on_battlefield(grant.recipient.card)
                    && self.object_ref(grant.recipient.card) == grant.recipient
                    && self.state.objects[grant.recipient.card as usize].controller
                        == entered.controller
                    && grant.granting_card.triggers()[usize::from(grant.ability)]
                        .condition
                        .event()
                        == TriggerCondition::LandEntersYou
            })
            .collect();
        for grant in grants {
            let source = self.object_view(grant.recipient.card);
            if !source.grant_available(
                grant.grantor,
                grant.granting_card,
                grant.ability,
                Some(grant.timestamp),
            ) {
                continue;
            }
            if self.queue_zone_trigger_from(
                &source,
                usize::from(grant.ability),
                event,
                Some(Box::new(AttachmentTriggerView {
                    grantor: grant.grantor,
                    granting_card: grant.granting_card,
                    recipient: grant.recipient,
                    defender: None,
                    granted: true,
                    origin: grant.origin,
                })),
            ) {
                self.add_entry_trigger_multiplicity(
                    &source,
                    std::slice::from_ref(&entered.reference),
                );
            }
        }
    }
    pub(super) fn temporary_attack_triggers(&mut self, actor: Player) {
        let grants = self
            .state
            .temporary_abilities
            .as_deref()
            .into_iter()
            .flatten()
            .copied()
            .filter(|grant| {
                (grant.permanent || grant.expires_turn == self.state.turn)
                    && self.on_battlefield(grant.recipient.card)
                    && self.object_ref(grant.recipient.card) == grant.recipient
                    && self.state.objects[grant.recipient.card as usize].controller == actor
                    && self
                        .state
                        .attackers
                        .iter()
                        .any(|a| a.card == grant.recipient.card)
                    && grant.granting_card.triggers()[usize::from(grant.ability)]
                        .condition
                        .event()
                        == TriggerCondition::SelfAttacks
            })
            .collect::<Vec<_>>();
        for grant in grants {
            let source = self.object_view(grant.recipient.card);
            if !source.grant_available(
                grant.grantor,
                grant.granting_card,
                grant.ability,
                Some(grant.timestamp),
            ) {
                continue;
            }
            self.queue_attack_trigger_from(
                &source,
                usize::from(grant.ability),
                Some(Box::new(AttachmentTriggerView {
                    grantor: grant.grantor,
                    granting_card: grant.granting_card,
                    recipient: grant.recipient,
                    defender: Some(actor.other()),
                    granted: true,
                    origin: grant.origin,
                })),
            );
        }
    }
    pub(super) fn temporary_step_triggers(&mut self, step: Step) {
        let Some(rows) = &self.state.temporary_abilities else {
            return;
        };
        let matching: Vec<_> = rows
            .iter()
            .copied()
            .filter(|grant| {
                let object = &self.state.objects[grant.recipient.card as usize];
                (grant.permanent || grant.expires_turn == self.state.turn)
                    && object.zone == Zone::Battlefield
                    && self.object_ref(grant.recipient.card) == grant.recipient
                    && self.step_event_matches(
                        grant.granting_card.triggers()[usize::from(grant.ability)].condition,
                        step,
                        object.controller,
                    )
            })
            .collect();
        for grant in matching {
            let source = self.object_view(grant.recipient.card);
            if source.grant_available(
                grant.grantor,
                grant.granting_card,
                grant.ability,
                Some(grant.timestamp),
            ) {
                self.queue_trigger_from(
                    &source,
                    usize::from(grant.ability),
                    Some(Box::new(grant.context())),
                );
            }
        }
    }

    pub(super) fn granted_departure_mana_value(&self, before: &ObjectView) -> u64 {
        let needs_mana_value = self.state.temporary_abilities.as_ref().is_some_and(|rows| {
            rows.iter().any(|grant| {
                grant.recipient == before.reference
                    && (grant.permanent || grant.expires_turn == self.state.turn)
                    && grant.granting_card.triggers()[usize::from(grant.ability)]
                        .program
                        .contains(&Instruction::DamageEachCreatureByDepartureManaValue)
                    && before.grant_available(
                        grant.grantor,
                        grant.granting_card,
                        grant.ability,
                        Some(grant.timestamp),
                    )
            })
        });
        if needs_mana_value {
            self.object_mana_value(before.id)
        } else {
            0
        }
    }

    pub(super) fn temporary_departure_triggers(&mut self, event: &ZoneChange) {
        if event.before.zone != Zone::Battlefield || !event.moved {
            return;
        }
        let Some(rows) = &self.state.temporary_abilities else {
            return;
        };
        let matching: Vec<_> = rows
            .iter()
            .copied()
            .filter(|grant| {
                grant.recipient == event.before.reference
                    && (grant.permanent || grant.expires_turn == self.state.turn)
                    && event.before.grant_available(
                        grant.grantor,
                        grant.granting_card,
                        grant.ability,
                        Some(grant.timestamp),
                    )
            })
            .collect();
        for grant in matching {
            let ability = grant.granting_card.triggers()[usize::from(grant.ability)];
            let return_death = match ability.condition.event() {
                TriggerCondition::SelfLeaves => false,
                TriggerCondition::SelfDies if event.destination == Zone::Graveyard => false,
                TriggerCondition::ReturnDeath(filter)
                    if event.destination == Zone::Graveyard
                        && self.return_death_matches(filter, &event.before, event) =>
                {
                    true
                }
                _ => continue,
            };
            if !self.queue_zone_trigger_from(
                &event.before,
                usize::from(grant.ability),
                event,
                Some(Box::new(grant.context())),
            ) {
                continue;
            }
            if return_death {
                self.capture_return_death(event);
            } else if ability
                .program
                .contains(&Instruction::DamageEachCreatureByDepartureManaValue)
            {
                self.state
                    .pending_triggers
                    .last_mut()
                    .unwrap()
                    .set_event_object(EventObjectView {
                        object: event.before.reference,
                        facts: EventObjectFacts::DepartureManaValue(
                            event.granted_departure_mana_value,
                        ),
                        semantic_objects: None,
                        conniver_creature: None,
                        subtype_lki: None,
                    });
            }
        }
        // Dispatch before erasing the exact old incarnation, including each
        // member of a simultaneous departure batch. Other recipients survive.
        if let Some(grants) = &mut self.state.temporary_abilities {
            grants.retain(|grant| grant.recipient != event.before.reference);
        }
        if self
            .state
            .temporary_abilities
            .as_ref()
            .is_some_and(|grants| grants.is_empty())
        {
            self.state.temporary_abilities = None;
        }
    }
    pub(super) fn return_granted_source(
        &mut self,
        item: &SpellView,
        counter: Option<(CounterKind, u64)>,
        role: Option<Card>,
    ) {
        let moves: Vec<_> = self
            .departed_source_in_graveyard(item)
            .filter_map(|id| {
                let kinds = self.state.objects[id as usize].card.kind_mask();
                // An instant or sorcery that was face down stays in the graveyard.
                if kinds & (4 | 8) != 0
                    || !PermanentType::ALL
                        .into_iter()
                        .any(|kind| kind.mask() & kinds != 0)
                {
                    return None;
                }
                let mut movement = self.granted_return_move(id);
                movement.effect_counter = counter;
                movement.role_after = role.map(|token| (token, item.controller));
                Some(movement)
            })
            .collect();
        if !moves.is_empty() {
            self.stage_entries(moves, None);
        }
    }
    pub(super) fn granted_source_entry(&self, item: &SpellView) -> Option<entries::EntryMove> {
        let expected = item.target_source();
        let object = self.state.objects.get(expected.card as usize)?;
        if object.zone != Zone::Graveyard || self.object_ref(expected.card) != expected {
            return None;
        }
        Some(self.granted_return_move(expected.card))
    }
    fn granted_return_move(&self, id: CardId) -> entries::EntryMove {
        let mut movement = self.entry_move(id, Zone::Battlefield, true);
        movement.controller = movement.before.owner;
        movement
    }
    pub(super) fn temporary_ability_bytes(&self) -> usize {
        self.state.temporary_abilities.as_ref().map_or(0, |grants| {
            std::mem::size_of::<Vec<TemporaryAbilityView>>()
                + grants.len() * std::mem::size_of::<TemporaryAbilityView>()
        })
    }
}
