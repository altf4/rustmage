//! Shared live predicates, beginning-of-step events and intervening-if checks.
use super::*;
/// The relation belongs to the beginning-of-step event, not its later resolution.
pub(super) fn attached_controller_turn<W: ability_world::CharacteristicWorld + ?Sized>(
    world: &W,
    source: ObjectRef,
    active: Player,
) -> bool {
    let attachment = world.object(source.card);
    if attachment.zone != Zone::Battlefield || world.reference(source.card) != source {
        return false;
    }
    let Some(recipient) = attachment.attached_to else {
        return false;
    };
    let object = world.object(recipient.card);
    object.zone == Zone::Battlefield
        && world.reference(recipient.card) == recipient
        && object.controller == active
}
impl Game {
    pub(super) fn begin_step(&mut self, step: Step) {
        self.state.step = step;
        if step == Step::Upkeep
            && let Some(rows) = self.state.delayed_upkeep_tokens.take()
        {
            self.state.pending_triggers.extend(*rows);
        }
        if step == Step::Draw {
            self.state.players[self.state.active.index()].drew_in_own_draw_step = false;
        }
        if matches!(step, Step::Main1 | Step::Main2)
            && let Some(rows) = self.state.delayed_main_mana.take()
        {
            let (ready, remaining): (Vec<_>, Vec<_>) = rows
                .into_iter()
                .partition(|row| row.controller == self.state.active);
            self.state.pending_triggers.extend(ready);
            if !remaining.is_empty() {
                self.state.delayed_main_mana = Some(Box::new(remaining));
            }
        }
        if step == Step::Main1 {
            self.queue_radiation_trigger();
            self.queue_paradigm_triggers();
            self.precombat_lore_counters();
        }
        if step == Step::End {
            self.queue_copy_end_step_abilities();
            self.queue_delayed_sacrifices();
            if let Some(items) = self.state.delayed_artifact_copies.take() {
                self.state.pending_triggers.extend(*items);
            }
            self.queue_delayed_returns();
            self.queue_warp_exiles();
        }
        let sources = self.indexed_trigger_sources(&self.state.step_sources);
        for (source, grant) in sources {
            let definition = grant
                .as_ref()
                .map_or(source.card, |event| event.granting_card);
            for (index, ability) in definition.triggers().iter().enumerate() {
                if let TriggerCondition::AtStep {
                    step: wanted,
                    relation,
                    ..
                } = ability.condition.event()
                {
                    let matches_step = match wanted {
                        TriggerStep::Upkeep => step == Step::Upkeep,
                        TriggerStep::Draw => step == Step::Draw,
                        TriggerStep::Main1 => step == Step::Main1,
                        TriggerStep::Main2 => step == Step::Main2,
                        TriggerStep::MainPhases => matches!(step, Step::Main1 | Step::Main2),
                        TriggerStep::BeginCombat => step == Step::BeginCombat,
                        TriggerStep::EndCombat => step == Step::EndCombat,
                        TriggerStep::End => step == Step::End,
                    };
                    let related = match relation {
                        CastRelation::You => source.controller == self.state.active,
                        CastRelation::Opponent => source.controller != self.state.active,
                        CastRelation::Any => true,
                    };
                    if matches_step && related {
                        self.queue_trigger_from(&source, index, grant.map(Box::new));
                    }
                }
            }
        }
    }

    fn queue_radiation_trigger(&mut self) {
        let player = self.state.active;
        if self.state.players[player.index()].player_counter(crate::cards::PlayerCounterKind::Rad)
            == 0
        {
            return;
        }
        let id = self.next_stack_id();
        self.state.pending_triggers.push(SpellView {
            source: None,
            source_less_rule: Some(SourceLessRule::Radiation),
            target_type_slots: None,
            additional_triggers: 0,
            loyalty_counters_paid: 0,
            loyalty_x: None,
            reflexive_target_limit: None,
            mana_tap_bonus: None,
            event_player: None,
            source_move: None,
            borrowed_slot: None,
            self_granted_activation: None,
            target_controller_lki: None,
            crime_qualifying_target: false,
            target_kinds_lki: None,
            gift: None,
            reflexive_subject: None,
            delayed_warp_exile: None,
            return_link: None,
            crew: None,
            cast_event: None,
            cast_creature_subjects: None,
            copy_event: None,
            captured_artifact_copy: None,
            source_copy_recipe: None,
            source_ability_origin: None,
            target_allocations: None,
            mana_copy_sources: None,
            is_copy: false,
            ninjutsu_defender: None,
            delayed_sacrifice: None,
            x: None,
            attachment_trigger: None,
            cast: None,
            entering_object: None,
            evidence_collected: false,
            id,
            kind: StackKind::InherentRule {
                rule: SourceLessRule::Radiation,
                affected_player: player,
            },

            split_second: false,
            source_keywords: Keywords::NONE,
            source_tapped: false,
            source_attachment: None,
            source_transform: None,

            related_stack: None,
            damage_event: None,
            damage_received: None,
            creature_combat_event: None,
            discard_event: None,
            draw_event: None,
            life_event: None,
            source_counter_payment: None,
            exiled_symbol_cards: None,
            source_counter_lki: None,
            source_quantity_lki: None,
            event_object: None,
            manifest_event: None,
            sacrificed_toughness: None,
            sacrificed_power: None,

            targets: Vec::new(),
            mode: None,
            controller: player,
        });
    }

    pub(super) fn trigger_time_guards_hold(
        &self,
        condition: TriggerCondition,
        player: Player,
        source: ObjectRef,
        tapped: bool,
    ) -> bool {
        match condition {
            TriggerCondition::When { guard, event } => {
                self.step_guard(guard, player, source, tapped)
                    && self.trigger_time_guards_hold(*event, player, source, tapped)
            }
            TriggerCondition::InZone { event, .. }
            | TriggerCondition::Granted(event)
            | TriggerCondition::OncePerTurn(event)
            | TriggerCondition::DuringOpponentsTurn(event)
            | TriggerCondition::TreasureManaSpent(event)
            | TriggerCondition::ManaSpent { event, .. }
            | TriggerCondition::InterveningIf { event, .. } => {
                self.trigger_time_guards_hold(*event, player, source, tapped)
            }
            _ => true,
        }
    }
    pub(super) fn trigger_guards_hold(
        &self,
        condition: TriggerCondition,
        player: Player,
        source: ObjectRef,
        tapped: bool,
    ) -> bool {
        match condition {
            TriggerCondition::InZone { event, .. }
            | TriggerCondition::When { event, .. }
            | TriggerCondition::Granted(event)
            | TriggerCondition::OncePerTurn(event)
            | TriggerCondition::DuringOpponentsTurn(event)
            | TriggerCondition::TreasureManaSpent(event)
            | TriggerCondition::ManaSpent { event, .. } => {
                self.trigger_guards_hold(*event, player, source, tapped)
            }
            TriggerCondition::InterveningIf { guard, event } => {
                self.step_guard(guard, player, source, tapped)
                    && self.trigger_guards_hold(*event, player, source, tapped)
            }
            TriggerCondition::AtStep { guard, .. } => {
                self.step_guard(guard, player, source, tapped)
            }
            _ => true,
        }
    }

    /// The historic entrant is excluded by exact incarnation, even after it leaves or reenters.
    pub(super) fn other_controlled_subtypes_at_least(
        &self,
        player: Player,
        entering: ObjectRef,
        subtype: &str,
        minimum: u16,
    ) -> bool {
        self.state
            .objects
            .iter()
            .enumerate()
            .filter(|(id, o)| {
                o.zone == Zone::Battlefield
                    && o.controller == player
                    && self.object_ref(*id as CardId) != entering
                    && self.has_subtype(*id as CardId, subtype)
            })
            .take(usize::from(minimum))
            .count()
            >= usize::from(minimum)
    }
    pub(super) fn trigger_item_guards_hold(
        &self,
        item: &SpellView,
        condition: TriggerCondition,
    ) -> bool {
        let Some(source) = item.source else {
            return false;
        };
        if let TriggerCondition::SubtypeEntersYouWithOthers { subtype, minimum } = condition.event()
        {
            let Some(entering) = item.entering_object else {
                return false;
            };
            return self.other_controlled_subtypes_at_least(
                item.controller,
                entering,
                subtype,
                minimum,
            ) && self.trigger_guards_hold(
                condition,
                item.controller,
                source.object,
                item.source_tapped,
            );
        }
        if let TriggerCondition::InterveningIf {
            guard:
                StepGuard::Expression {
                    value: StatValue::SourceCounters(kind),
                    minimum,
                    maximum,
                },
            event,
        } = condition
            && !(self.on_battlefield(source.object.card)
                && self.object_ref(source.object.card) == source.object)
        {
            let value = i128::from(
                item.source_counter_lki
                    .as_ref()
                    .and_then(|rows| rows.iter().find(|row| row.kind == kind))
                    .map_or(0, |row| row.count),
            );
            return minimum.is_none_or(|min| value >= min)
                && maximum.is_none_or(|max| value <= max)
                && self.trigger_guards_hold(
                    *event,
                    item.controller,
                    source.object,
                    item.source_tapped,
                );
        }
        self.trigger_guards_hold(
            condition,
            item.controller,
            source.object,
            item.source_tapped,
        )
    }

    pub(super) fn step_guard(
        &self,
        guard: StepGuard,
        player: Player,
        source: ObjectRef,
        source_tapped: bool,
    ) -> bool {
        self.step_guard_without_hand(guard, player, source, source_tapped, None)
    }
    pub(super) fn step_guard_without_hand(
        &self,
        guard: StepGuard,
        player: Player,
        source: ObjectRef,
        source_tapped: bool,
        departing: Option<CardId>,
    ) -> bool {
        let p = &self.state.players[player.index()];
        let (value, minimum, maximum) = match guard {
            StepGuard::SourceUnprepared => {
                return self.on_battlefield(source.card)
                    && self.object_ref(source.card) == source
                    && !self.is_prepared(source);
            }
            StepGuard::ClassLevelAtLeast(level) => {
                return self.on_battlefield(source.card)
                    && self.object_ref(source.card) == source
                    && self.state.objects[source.card as usize].class_level >= level;
            }
            StepGuard::UnsolvedLifeGainedAtLeast(n) => {
                return !self.case_solved(source) && p.life_gained_this_turn >= n;
            }
            StepGuard::Always => return true,
            StepGuard::ArtifactEnteredThisTurn => return p.turn_facts.artifact_entered,
            StepGuard::HeroEnteredOrAttackedThisTurn => {
                return p.turn_facts.hero_entered || p.turn_facts.hero_attacked;
            }
            StepGuard::AttachedControllerTurn => {
                return attached_controller_turn(self, source, self.state.active);
            }
            StepGuard::CardExiledThisTurn => return self.state.cards_exiled_this_turn,
            StepGuard::OpponentControlsMoreCreatures => {
                let count = |controller| {
                    self.state
                        .objects
                        .iter()
                        .enumerate()
                        .filter(|(id, o)| {
                            o.zone == Zone::Battlefield
                                && o.controller == controller
                                && self.is_creature(*id as CardId)
                        })
                        .count()
                };
                return count(player.other()) > count(player);
            }
            StepGuard::OpponentHasMoreCardsInHand => {
                let count = |owner| {
                    self.hand_count(owner)
                        - usize::from(departing.is_some_and(|id| {
                            self.state
                                .objects
                                .get(id as usize)
                                .is_some_and(|o| o.zone == Zone::Hand && o.owner == owner)
                        }))
                };
                return count(player.other()) > count(player);
            }
            StepGuard::SourceReceivedCounterThisTurn => {
                return self.put_counter_on_source_this_turn(source, player);
            }
            StepGuard::YourTurn => return self.state.active == player,
            StepGuard::SourcePowerAtLeast(minimum) => {
                return self.on_battlefield(source.card)
                    && self.object_ref(source.card) == source
                    && self.stats_without_hand(source.card, departing).power
                        >= i128::from(minimum);
            }
            StepGuard::SourceToughnessAtMost(maximum) => {
                return self.on_battlefield(source.card)
                    && self.object_ref(source.card) == source
                    && self.stats_without_hand(source.card, departing).toughness
                        <= i128::from(maximum);
            }
            StepGuard::Outlaw => {
                return self.state.objects.iter().enumerate().any(|(id, o)| {
                    o.controller == player
                        && o.zone == Zone::Battlefield
                        && ["Assassin", "Mercenary", "Pirate", "Rogue", "Warlock"]
                            .iter()
                            .any(|s| self.has_subtype(id as CardId, s))
                });
            }
            StepGuard::Expression {
                value,
                minimum,
                maximum,
            } => {
                let value = self.stat_value_without_hand(value, source, player, departing);
                return minimum.is_none_or(|min| value >= min)
                    && maximum.is_none_or(|max| value <= max);
            }
            StepGuard::GainedLife => return p.life_gained_this_turn > 0,
            StepGuard::ScriedOrSurveilledThisTurn => return p.scried_or_surveilled_this_turn,
            StepGuard::LoyaltyAbilityActivatedThisTurn => {
                return p.loyalty_ability_activated_this_turn;
            }
            StepGuard::OpponentNoncombatDamageLastTurn => {
                return self.state.players[player.other().index()].noncombat_damage_last_turn;
            }
            StepGuard::GainedAndLostLife => {
                return p.life_gained_this_turn > 0 && p.life_lost_this_turn > 0;
            }
            StepGuard::Attacked => return p.attacked_this_turn,
            StepGuard::CreatureDied => return self.state.creatures_died_this_turn > 0,
            StepGuard::OpponentLostLife => {
                return self.state.players[player.other().index()].life_lost_this_turn > 0;
            }
            StepGuard::CommittedCrimeThisTurn => return p.committed_crime_this_turn,
            StepGuard::SourceTapped => {
                return if self.on_battlefield(source.card) && self.object_ref(source.card) == source
                {
                    self.state.objects[source.card as usize].tapped
                } else {
                    source_tapped
                };
            }
            StepGuard::Threshold {
                value,
                minimum,
                maximum,
            } => (value, minimum, maximum),
        };
        let owned = |id: usize| {
            let o = &self.state.objects[id];
            o.zone == Zone::Battlefield && o.controller == player
        };
        let creatures = || {
            self.state
                .objects
                .iter()
                .enumerate()
                .filter(|(id, _)| owned(*id) && self.is_creature(*id as CardId))
        };
        let value = match value {
            StepValue::Life => Some(p.life),
            StepValue::Hand => Some(
                self.hand_count(player) as i128
                    - i128::from(departing.is_some_and(|id| {
                        self.state
                            .objects
                            .get(id as usize)
                            .is_some_and(|o| o.zone == Zone::Hand && o.owner == player)
                    })),
            ),
            StepValue::SpellsCast => Some(i128::from(p.spells_cast_this_turn)),
            StepValue::Creatures => Some(creatures().count() as i128),
            StepValue::EnchantedCreatures => Some(
                creatures()
                    .filter(|(id, _)| self.is_enchanted(*id as CardId))
                    .count() as i128,
            ),
            StepValue::TappedCreatures => {
                Some(creatures().filter(|(_, o)| o.tapped).count() as i128)
            }
            StepValue::Lands => Some(
                self.state
                    .objects
                    .iter()
                    .enumerate()
                    .filter(|(id, _)| owned(*id) && self.current_kinds(*id as CardId) & 2 != 0)
                    .count() as i128,
            ),
            StepValue::GreatestCreaturePower => creatures()
                .map(|(id, _)| self.stats_without_hand(id as CardId, departing).power)
                .max(),
            StepValue::GreatestOtherCreaturePower => creatures()
                .filter(|(id, _)| self.object_ref(*id as CardId) != source)
                .map(|(id, _)| self.stats_without_hand(id as CardId, departing).power)
                .max(),
            StepValue::TotalCreaturePower => Some(
                creatures()
                    .map(|(id, _)| self.stats_without_hand(id as CardId, departing).power)
                    .try_fold(0i128, i128::checked_add)
                    .capacity("step aggregate exhausted"),
            ),
            StepValue::TotalCreatureToughness => Some(
                creatures()
                    .map(|(id, _)| self.stats_without_hand(id as CardId, departing).toughness)
                    .try_fold(0i128, i128::checked_add)
                    .capacity("step aggregate exhausted"),
            ),
        };
        value.is_some_and(|value| {
            minimum.is_none_or(|min| value >= min) && maximum.is_none_or(|max| value <= max)
        })
    }
}
