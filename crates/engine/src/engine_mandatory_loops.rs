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
                if !pure_exiler(object.card)
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
                if !pure_exiler(object.card)
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
        if ids.len() != 3 {
            return false;
        }
        let mut probe = self.clone();
        for _ in 0..32 {
            let mut actions = probe
                .legal_actions()
                .into_iter()
                .filter(|a| *a != Action::Concede);
            let Some(action) = actions.next() else {
                return false;
            };
            if actions.next().is_some()
                || !matches!(action, Action::Pass | Action::ChooseTarget { .. })
            {
                return false;
            }
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
                && normalized(probe.state.clone(), &self.state, &ids)
                    .is_some_and(|s| s == self.state)
            {
                return true;
            }
        }
        false
    }
}
