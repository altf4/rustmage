//! Conservative CR104.4b proof for closed, pure exile/return cycles.
//!
//! This is not a decision-count heuristic. Unsupported programs, optional
//! choices and any unexplained state change leave the game running normally.
use super::*;
use crate::cards::StaticAbilityFragment;

fn pure_exiler(card: Card) -> bool {
    let triggers = card.triggers();
    !card.is_creature()
        && triggers.len() == 1
        && triggers[0].condition == TriggerCondition::SelfEnters
        && matches!(
            triggers[0].program,
            [Instruction::ExileUntilSourceLeaves]
                | [Instruction::ExileBattlefieldUntilSourceLeaves { .. }]
        )
        && matches!(
            triggers[0].targets,
            TargetPlan::Single(TargetSpec {
                rule: TargetRule::Object {
                    zone: TargetZone::Battlefield,
                    ..
                },
                min: 1,
                max: 1
            }) | TargetPlan::Single(TargetSpec {
                rule: TargetRule::None,
                min: 0,
                max: 0
            })
        )
        && card.entry_tapped() == crate::cards::EntryTapped::Never
        && card.entry_counter_rules().is_empty()
        && card.devour().is_none()
        && card.entry_choice().is_none()
        && card.chosen_entry_counters().is_none()
        && card.activations().iter().all(|ability| {
            use crate::cards::{ActivationGuard, ActivationOrigin, EffectChoiceKind};
            ability.guard == ActivationGuard::Always
                && ability.origin == ActivationOrigin::Battlefield
                && !ability.cycling
                && !ability.exhaust
                && ability.cost.remove.is_none()
                && ability.cost.loyalty.is_none()
                && ability
                    .cost
                    .additional
                    .is_none_or(|cost| cost.kind == EffectChoiceKind::DiscardLegendary)
                && matches!(
                    ability.program,
                    [Instruction::Draw(PlayerSet::Controller, 1)]
                        | [Instruction::PutCounters {
                            subject: Subject::Targets,
                            kind: CounterKind::PlusOne,
                            amount: 1
                        }]
                        | [
                            Instruction::LinkedExileToGraveyardTokens { .. },
                            Instruction::SacrificeSource
                        ]
                )
        })
        && card.static_ability_groups().iter().all(|group| {
            group.fragments.iter().all(|fragment| {
                matches!(
                    fragment,
                    StaticAbilityFragment::StartYourEngines
                        | StaticAbilityFragment::SourceMaxSpeedActivation(_)
                )
            })
        })
}

// An entry damage trigger with a mandatory creature target can commute with
// the exile cycle only when it has no legal target. Actual availability is
// checked on each probe state; no counter/optional/entry replacement programs.
fn inert_entry_damage(card: Card) -> bool {
    let triggers = card.triggers();
    !card.is_creature()
        && triggers.len() == 1
        && triggers[0].condition == TriggerCondition::SelfEnters
        && matches!(triggers[0].program, [Instruction::Damage(_)])
        && matches!(
            triggers[0].targets,
            TargetPlan::Single(TargetSpec {
                rule: TargetRule::Object {
                    zone: TargetZone::Battlefield,
                    kinds: 1,
                    ..
                },
                min: 1,
                max: 1
            })
        )
        && card.entry_tapped() == crate::cards::EntryTapped::Never
        && card.entry_counter_rules().is_empty()
        && card.entry_choice().is_none()
        && card.chosen_entry_counters().is_none()
        && card.static_ability_groups().is_empty()
        && card.activations().iter().all(|ability| {
            use crate::cards::{ActivationGuard, ActivationOrigin, EffectChoiceKind};
            ability.guard == ActivationGuard::Always
                && ability.origin == ActivationOrigin::Battlefield
                && !ability.cycling
                && !ability.exhaust
                && ability.sorcery
                && !ability.once_per_turn
                && ability.targets == TargetPlan::NONE
                && matches!(
                    ability.program,
                    [Instruction::ReturnCraftedSourceTransformed]
                )
                && ability.cost.colored == [1, 0, 0, 0, 0, 0]
                && ability.cost.generic == 3
                && ability.cost.alternatives.is_empty()
                && ability.cost.mana_component
                && !ability.cost.tap
                && !ability.cost.sacrifice
                && ability.cost.exile_self
                && !ability.cost.discard
                && ability.cost.life == 0
                && ability.cost.remove.is_none()
                && ability.cost.loyalty.is_none()
                && ability.cost.additional.is_some_and(|cost| {
                    cost.kind == EffectChoiceKind::ExileControlledOrGraveyard
                        && cost.filter.kinds == 16
                        && cost.filter.other
                        && !cost.filter.source_only
                        && !cost.filter.nonland
                        && cost.filter.tokens.is_none()
                        && cost.filter.keywords == Keywords::NONE
                        && !cost.filter.artifact_enchantment_or_token
                        && cost.filter.subtype.is_none()
                })
        })
}

fn map_reference(r: ObjectRef, before: &State, after: &State, ids: &[CardId]) -> Option<ObjectRef> {
    if !ids.contains(&r.card) {
        return Some(r);
    }
    let generation = i128::from(r.generation)
        - i128::from(after.objects[r.card as usize].generation)
        + i128::from(before.objects[r.card as usize].generation);
    Some(ObjectRef {
        card: r.card,
        generation: u64::try_from(generation).ok()?,
    })
}

fn same_announcement(
    a: &Announcement,
    b: &Announcement,
    before: &State,
    after: &State,
    ids: &[CardId],
) -> bool {
    let mut candidate = b.clone();
    let offset =
        i128::from(b.item.id) - i128::from(after.next_stack_id) + i128::from(before.next_stack_id);
    let Ok(id) = u64::try_from(offset) else {
        return false;
    };
    candidate.item.id = id;
    if let Some(source) = &mut candidate.item.source {
        let Some(mapped) = map_reference(source.object, before, after, ids) else {
            return false;
        };
        source.object = mapped;
    }
    if let Some(object) = &mut candidate.item.entering_object {
        let Some(mapped) = map_reference(*object, before, after, ids) else {
            return false;
        };
        *object = mapped;
    }
    if let Some(event) = &mut candidate.item.event_object {
        if event.facts != EventObjectFacts::Identity {
            return false;
        }
        let Some(mapped) = map_reference(event.object, before, after, ids) else {
            return false;
        };
        event.object = mapped;
        if let Some(objects) = &mut event.semantic_objects {
            for reference in objects.iter_mut() {
                let DamageSourceRef::Object(object) = reference else {
                    return false;
                };
                let Some(mapped) = map_reference(*object, before, after, ids) else {
                    return false;
                };
                *object = mapped;
            }
        }
    }
    // Typed equality includes all private/nonserialized packet witnesses.
    *a == candidate
}

fn normalized(mut after: State, before: &State, ids: &[CardId]) -> Option<State> {
    if !same_announcement(
        before.announcing.as_ref()?,
        after.announcing.as_ref()?,
        before,
        &after,
        ids,
    ) {
        return None;
    }
    let map = |r| map_reference(r, before, &after, ids);
    let mut links = after.exile_links.clone();
    for row in &mut links {
        row.source = map(row.source)?;
        row.exiled = map(row.exiled)?;
    }
    let mut entered = after.entered_this_turn.clone();
    if let Some(rows) = &mut entered {
        // The admitted cycle contains only noncreature pure exilers and inert
        // lands, with no observer that can read entry history. Their exact old
        // incarnations are retained for LKI damage generally, but add no choice
        // or observable difference in this closed cycle. Preserve every other
        // object's history and restore only these proven-inert IDs.
        rows.retain(|row| !ids.contains(&row.card));
        let unchanged = before
            .entered_this_turn
            .as_ref()
            .map_or_else(Vec::new, |original| {
                original
                    .iter()
                    .filter(|row| !ids.contains(&row.card))
                    .copied()
                    .collect()
            });
        if rows.as_slice() != unchanged.as_slice() {
            return None;
        }
        entered = before.entered_this_turn.clone();
    }
    // Pure entry programs do not read this counter. All other battlefield
    // observers and generated/layered abilities are excluded by admission.
    for p in 0..2 {
        after.players[p].nonland_permanents_entered_this_turn =
            before.players[p].nonland_permanents_entered_this_turn;
        after.players[p].lands_entered_this_turn = before.players[p].lands_entered_this_turn;
    }
    if !roles::same_inert_loop_timestamps(before, &after, ids) {
        return None;
    }
    after.roles = before.roles.clone();
    for &id in ids {
        after.objects[id as usize].generation = before.objects[id as usize].generation;
    }
    after.exile_links = links;
    after.entered_this_turn = entered;
    after.announcing = before.announcing.clone();
    after.next_stack_id = before.next_stack_id;
    Some(after)
}

impl Game {
    fn printed_entry_packet(item: &SpellView) -> bool {
        !item.is_copy
            && item.attachment_trigger.is_none()
            && item.borrowed_slot.is_none()
            && item.copy_event.is_none()
            && item.mode.is_none()
            && matches!(item.kind, StackKind::TriggeredAbility { ability: 0 })
            && item.source.is_some_and(|source| {
                item.entering_object == Some(source.object)
                    && item.trigger_definition() == source.card
            })
    }

    fn unplaceable_entry_damage(&self, item: &SpellView) -> bool {
        Self::printed_entry_packet(item)
            && item
                .source
                .is_some_and(|source| inert_entry_damage(source.card))
            && !self.item_targets_available(item, self.target_spec(item))
    }

    pub(super) fn prove_mandatory_exile_loop(&self) -> bool {
        if !matches!(self.state.pending, Pending::Targets { .. })
            || self.state.outcome.is_some()
            || self.state.exile_links.is_empty()
        {
            return false;
        }
        let Some(announcement) = &self.state.announcing else {
            return false;
        };
        if !announcement.trigger
            || !matches!(
                announcement.item.program(),
                Some((_, [Instruction::ExileUntilSourceLeaves]))
            )
        {
            return false;
        }
        // Only printed pure exilers and inert noncreature lands may observe the
        // cycle. No generated abilities, replacement/zone rules or copies.
        if self.state.mana_lots.is_some()
            || self.state.library_top.is_some()
            || self.state.players.iter().any(|p| p.library_top_life_play)
            || self.state.zone_rules.is_some()
            || self.state.replacements.is_some()
            || self.state.temporary_abilities.is_some()
            || self.state.self_granted_abilities.is_some()
            || self.state.copy_recipes.is_some()
            || self.state.temporary_copies.is_some()
            || self.state.trigger_multiplicity.is_some()
            || self.state.player_control.is_some()
            || self.state.controls.is_some()
            || self.state.temporary_controls.is_some()
        {
            return false;
        }
        // Counter-dependent cast timing/alternative costs in hidden zones
        // could introduce an escape after a later iteration. Decline them.
        if self.state.objects.iter().any(|o| {
            o.zone != Zone::Library
                && (o.card.cast_timing_guard() != StepGuard::Always
                    || o.card
                        .cost_alternative_guard()
                        .is_some_and(|g| g != StepGuard::Always)
                    || (o.zone != Zone::Battlefield
                        && o.card
                            .triggers()
                            .iter()
                            .any(|t| t.condition.origin().is_some())))
        }) {
            return false;
        }
        let mut ids = Vec::new();
        for (id, object) in self.state.objects.iter().enumerate() {
            if object.zone == Zone::Battlefield {
                if object.card.is_land()
                    && !object.card.is_creature()
                    && object.card.static_ability_groups().is_empty()
                    && object
                        .card
                        .triggers()
                        .iter()
                        .all(|t| t.condition == TriggerCondition::SelfEnters)
                {
                    continue;
                }
                if !(pure_exiler(object.card) || inert_entry_damage(object.card))
                    || object.is_token()
                    || object.is_spell_copy()
                    || object.attached_to.is_some()
                {
                    return false;
                }
                ids.push(id as CardId);
            }
        }
        for link in &self.state.exile_links {
            for id in [link.source.card, link.exiled.card] {
                let object = &self.state.objects[id as usize];
                if !(pure_exiler(object.card) || inert_entry_damage(object.card))
                    || !matches!(object.zone, Zone::Battlefield | Zone::Exile)
                    || object.is_token()
                    || object.is_spell_copy()
                {
                    return false;
                }
                if !ids.contains(&id) {
                    ids.push(id);
                }
            }
        }
        let observers = ids
            .iter()
            .filter(|&&id| inert_entry_damage(self.state.objects[id as usize].card))
            .count();
        if ids.len() != 3 + observers || observers > 1 {
            return false;
        }
        let mut baseline = self.clone();
        // CR603.3d: an unplaceable trigger is removed without going on the
        // stack. At this target-choice checkpoint it may still be waiting only
        // because the player ordered the live exile trigger first. Removing
        // this provably unplaceable packet cannot change any target choice.
        baseline
            .state
            .ordering_triggers
            .retain(|item| !self.unplaceable_entry_damage(item));
        let mut probe = baseline.clone();
        for _ in 0..32 {
            let actions: Vec<_> = probe
                .legal_actions()
                .into_iter()
                .filter(|a| *a != Action::Concede)
                .collect();
            let action = if actions.len() == 1
                && matches!(actions[0], Action::Pass | Action::ChooseTarget { .. })
            {
                actions[0].clone()
            } else if actions.len() == 2
                && actions
                    .iter()
                    .all(|a| matches!(a, Action::PutTrigger { .. }))
            {
                // Exactly one unplaceable damage trigger and one live exile
                // trigger: either ordering has the same stack/targets. Choose
                // the unplaceable packet first on the disposable proof branch.
                let items: Vec<_> = actions
                    .iter()
                    .filter_map(|action| {
                        let Action::PutTrigger { id } = action else {
                            return None;
                        };
                        probe
                            .state
                            .ordering_triggers
                            .iter()
                            .find(|item| item.id == *id)
                    })
                    .collect();
                if items.len() != 2 {
                    return false;
                }
                let Some(index) = items
                    .iter()
                    .position(|item| probe.unplaceable_entry_damage(item))
                else {
                    return false;
                };
                let other = items[1 - index];
                if !Self::printed_entry_packet(other)
                    || !other.source.is_some_and(|source| pure_exiler(source.card))
                    || !probe.item_targets_available(other, probe.target_spec(other))
                {
                    return false;
                }
                actions[index].clone()
            } else {
                return false;
            };
            let Some(actor) = probe.semantic_actor() else {
                return false;
            };
            // Generated legal actions, on a disposable owned clone. The raw
            // transaction avoids recursively entering this proof.
            let applied = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                probe.apply_transaction(actor, action)
            }));
            if !matches!(applied, Ok(Ok(()))) || probe.state.outcome.is_some() {
                return false;
            }
            if matches!(probe.state.pending, Pending::Targets { .. })
                && normalized(probe.state.clone(), &baseline.state, &ids)
                    .is_some_and(|s| s == baseline.state)
            {
                return true;
            }
        }
        false
    }
}
