//! Caller-driven payment attempts. These controls inspect only the current
//! execution; they never ask whether a different future payment can succeed.
use super::*;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AttemptStage {
    Declare,
    GenerateMana,
    PayCosts,
    Recover,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AttemptFailureReason {
    UnsatisfiedMana,
    UnsatisfiedMaterial,
    UnsatisfiedDeclaration,
    IncompletePayment,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AttemptOutcome {
    Completed,
    Incomplete,
    Recovering,
    Failed,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct AttemptOutcomeView {
    pub attempt_id: u64,
    pub player: Player,
    pub source: Option<ObjectRef>,
    pub ability: Option<u16>,
    pub turn: u64,
    pub outcome: AttemptOutcome,
    pub reason: Option<AttemptFailureReason>,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct AttemptView {
    pub attempt_id: u64,
    pub parent_attempt_id: Option<u64>,
    pub player: Player,
    pub context: Option<u64>,
    pub source: Option<ObjectRef>,
    pub ability: Option<u16>,
    pub stage: AttemptStage,
    pub locked_cost: Option<ManaCost>,
    pub mana_window_closed: bool,
    pub contributions_sealed: bool,
    pub can_end_mana_window: bool,
    pub can_submit: bool,
    pub can_reset_selection: bool,
    pub failure: Option<AttemptFailureReason>,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    /// This is scoped to the active announcement. Suspended parent flags do not
    /// suppress a child's independent CR602 mana window.
    pub(super) fn attempt_mana_window_closed(&self) -> bool {
        let Some(a) = self.state.announcing.as_ref() else {
            return self
                .state
                .cast_rollback
                .as_ref()
                .is_some_and(|attempt| attempt.mana_closed);
        };
        a.cast_costs.as_ref().is_some_and(|costs| costs.mana_closed)
            || self
                .state
                .cast_rollback
                .as_ref()
                .is_some_and(|attempt| attempt.mana_closed && attempt.context == Some(a.item.id))
    }

    pub(super) fn attempt_contributions_sealed(&self) -> bool {
        self.state.cast_rollback.as_ref().is_some_and(|attempt| {
            attempt.contributions_sealed
                && self
                    .state
                    .announcing
                    .as_ref()
                    .is_none_or(|a| attempt.context == Some(a.item.id))
        })
    }

    fn attempt_payment_boundary(&self) -> bool {
        self.state.cast_rollback.is_some()
            && self.state.entry_batch.is_none()
            && self.state.replacements.is_none()
            && !self.mana_tap_bonus_pending()
            && matches!(
                self.state.pending,
                Pending::PayCosts { .. }
                    | Pending::CastCosts { .. }
                    | Pending::AdditionalCost { .. }
                    | Pending::Evidence { .. }
                    | Pending::Forage { .. }
                    | Pending::Convoke { .. }
                    | Pending::Improvise { .. }
                    | Pending::Delve { .. }
                    | Pending::Waterbend { .. }
                    | Pending::CreatureTapPayment { .. }
                    | Pending::Crew { .. }
                    | Pending::Saddle { .. }
                    | Pending::BlightPayment { .. }
                    | Pending::TeamworkPayment { .. }
                    | Pending::ConspirePayment { .. }
                    | Pending::CasualtyPayment { .. }
                    | Pending::SneakReturnCost { .. }
                    | Pending::WebSlingingReturnCost { .. }
                    | Pending::BeamMeUpReturnCost { .. }
                    | Pending::BeholdCost { .. }
                    | Pending::AttackPayment { .. }
                    | Pending::BlockPayment { .. }
                    | Pending::PlotPayment { .. }
                    | Pending::FaceUpPayment { .. }
                    | Pending::RoomUnlockPayment { .. }
            )
    }

    fn current_attempt_cost(&self) -> Option<(Player, ManaCost)> {
        if let Some(a) = self.state.announcing.as_ref() {
            return a.total_cost.map(|cost| (a.item.controller, cost));
        }
        if let Some(request) = self.state.combat_payment.as_ref() {
            return Some((
                request.player,
                ManaCost {
                    colored: [0; 6],
                    generic: request.generic,
                },
            ));
        }
        if let Some(request) = self.face_up_payment() {
            return Some((request.controller, request.cost));
        }
        if let Some(request) = self.plot_payment() {
            return Some((request.controller, request.cost));
        }
        self.state
            .room_unlock_payment()
            .map(|request| (request.controller, request.cost))
    }

    fn declaration_failure(&self) -> bool {
        let Some(a) = self.state.announcing.as_ref().filter(|a| !a.trigger) else {
            return false;
        };
        if matches!(self.state.pending, Pending::Modes { .. }) {
            return a.item.modes().is_some_and(|modes| {
                !(0..modes.len()).any(|index| self.mode_choice_fundable(a, index))
            });
        }
        if !matches!(self.state.pending, Pending::Targets { .. }) {
            return false;
        }
        let selected = a
            .item
            .targets
            .iter()
            .filter(|bound| bound.group == a.group)
            .count();
        let minimum = self
            .exact_x_targets(&a.item)
            .unwrap_or(usize::from(a.spec.min));
        if selected >= minimum {
            return false;
        }
        let mut available = 0usize;
        self.visit_item_targets(&a.item, a.spec.rule, |target| {
            let slot_available = if matches!(
                a.spec.rule,
                TargetRule::Grouped {
                    constraint: TargetGroupConstraint::DistinctCardTypeSlots,
                    ..
                }
            ) {
                let used = a.item.target_type_slots.as_ref().map_or(0, |slots| {
                    slots.iter().fold(0u16, |mask, kind| mask | *kind)
                });
                matches!(target, Target::Card(id) if self.current_kinds(id) & !used != 0)
            } else {
                true
            };
            if slot_available
                && !a
                    .item
                    .targets
                    .iter()
                    .any(|bound| bound.group == a.group && bound.target == target)
                && self.target_group_compatible(a.spec.rule, a.group, &a.item.targets, target)
            {
                available += 1;
            }
            false
        });
        // Count a superset of valid joint completions. This certifies failure
        // without trying combinations or assuming compatibility among candidates.
        available < minimum - selected
    }

    fn can_end_attempt_mana_window(&self) -> bool {
        self.attempt_payment_boundary()
            && !self.attempt_mana_window_closed()
            && self.current_attempt_cost().is_some()
    }

    fn can_submit_attempt(&self) -> bool {
        (self.attempt_payment_boundary() && self.attempt_mana_window_closed())
            || (self.state.cast_rollback.is_some() && self.declaration_failure())
    }

    pub(super) fn visit_attempt_controls(&self, emit: &mut impl FnMut(Action) -> bool) -> bool {
        (self.can_end_attempt_mana_window() && emit(Action::EndManaWindow))
            || (self.can_submit_attempt() && emit(Action::SubmitAttempt))
            || (self.resettable_attempt_selection().is_some() && emit(Action::ResetCostSelection))
    }

    pub(super) fn end_attempt_mana_window(&mut self) {
        let context = self.state.announcing.as_ref().map(|a| a.item.id);
        let attempt = self.state.cast_rollback.as_mut().unwrap();
        attempt.context = context;
        attempt.mana_closed = true;
    }

    /// Submit the current concrete execution. Missing choices alone are not a
    /// failed action: they preserve the owned continuation. An immediately
    /// impossible, selected obligation fails this attempt, including only its
    /// children; the suspended parent resumes after lawful mana reversal.
    pub(super) fn submit_attempt(&mut self) {
        // Submission is an explicit end of the supplied contribution selection;
        // it never automatically chooses an additional convoker or allocation.
        if matches!(
            self.state.pending,
            Pending::PayCosts { .. }
                | Pending::Convoke { .. }
                | Pending::Improvise { .. }
                | Pending::Delve { .. }
                | Pending::Waterbend { .. }
                | Pending::CreatureTapPayment { .. }
        ) {
            self.state
                .cast_rollback
                .as_mut()
                .unwrap()
                .contributions_sealed = true;
        }
        let reason = self.current_attempt_failure();
        let attempt = self.state.cast_rollback.as_mut().unwrap();
        let id = attempt.attempt_id;
        let player = attempt.player;
        let source = attempt.source;
        let ability = attempt.ability;
        let turn = attempt.turn;
        let Some(reason) = reason else {
            self.state.last_attempt_outcome[player.index()] = Some(AttemptOutcomeView {
                attempt_id: id,
                player,
                source,
                ability,
                turn,
                outcome: AttemptOutcome::Incomplete,
                reason: Some(AttemptFailureReason::IncompletePayment),
            });
            return;
        };
        attempt.failure = Some(reason);
        self.finish_cast_rollback(false);
        if matches!(self.state.pending, Pending::ManaRollback { .. }) {
            self.state.last_attempt_outcome[player.index()] = Some(AttemptOutcomeView {
                attempt_id: id,
                player,
                source,
                ability,
                turn,
                outcome: AttemptOutcome::Recovering,
                reason: Some(reason),
            });
        }
    }

    fn current_attempt_failure(&self) -> Option<AttemptFailureReason> {
        if self.declaration_failure() {
            return Some(AttemptFailureReason::UnsatisfiedDeclaration);
        }
        if self.current_material_cost_impossible() {
            return Some(AttemptFailureReason::UnsatisfiedMaterial);
        }
        if self.state.announcing.is_none()
            && (self
                .plot_payment()
                .is_some_and(|r| !r.source_is_current(self))
                || self
                    .state
                    .room_unlock_payment()
                    .is_some_and(|r| !r.source_is_current(self)))
        {
            return Some(AttemptFailureReason::UnsatisfiedMaterial);
        }
        let (player, mut total) = self.current_attempt_cost()?;
        let purpose = if let Some(a) = self.state.announcing.as_ref() {
            if !matches!(
                self.state.pending,
                Pending::PayCosts { .. }
                    | Pending::Convoke { .. }
                    | Pending::Improvise { .. }
                    | Pending::Delve { .. }
                    | Pending::Waterbend { .. }
                    | Pending::CreatureTapPayment { .. }
            ) {
                return None;
            }
            if a.evidence.as_ref().is_some_and(|e| !e.complete)
                || a.forage.as_ref().is_some_and(|f| !f.complete())
            {
                return None;
            }
            if let Some(costs) = &a.cast_costs {
                if costs.active_clause() != Some(CastCostClause::Mana) {
                    return None;
                }
            } else if a.additional_cost().is_some()
                && !a.additional_paid
                && a.additional.is_none()
                && !a.additional_group_complete
                && !a.additional_life
                // Specialized selections keep their own completion state. The
                // incomplete cases above already preserve their continuation.
                && a.forage.is_none()
                && a.evidence.is_none()
            {
                return None;
            }
            if let Some(contributions) = a.convoke.as_ref() {
                total = contributions.remaining;
            } else {
                let goal = self.funding_goal(a);
                total = ManaCost {
                    colored: goal.required,
                    generic: goal.generic,
                };
            }
            // All required selections are supplied. A source counter/tap or
            // other resource shared by two components cannot pay both. This
            // checks this exact packet only, with no alternative resource search.
            if !self.supplied_mana_costs_valid(a) {
                return Some(AttemptFailureReason::UnsatisfiedMaterial);
            }
            self.purpose_for_item(&a.item)
        } else {
            mana_lots::Purpose::Other
        };
        let (required, generic) =
            cast_origins::payment_requirements(self, player, purpose, total.colored, total.generic);
        let pool = self.available_mana(player, purpose);
        let short_color = pool.iter().zip(required).any(|(&have, need)| have < need);
        let spare: u64 = pool
            .iter()
            .zip(required)
            .map(|(&have, need)| u64::from(have.saturating_sub(need)))
            .sum();
        (short_color || spare < u64::from(generic)).then_some(AttemptFailureReason::UnsatisfiedMana)
    }

    pub fn attempt_view(&self, viewer: Player) -> Option<AttemptView> {
        let attempt = self.state.cast_rollback.as_ref()?;
        if attempt.attempt_id == 0 || !self.shares_information(viewer, attempt.player) {
            return None;
        }
        let recovering = matches!(self.state.pending, Pending::ManaRollback { .. });
        let closed = self.attempt_mana_window_closed();
        Some(AttemptView {
            attempt_id: attempt.attempt_id,
            parent_attempt_id: attempt.parent.as_ref().map(|parent| parent.attempt_id),
            player: attempt.player,
            context: attempt.context,
            source: attempt.source,
            ability: attempt.ability,
            stage: if recovering {
                AttemptStage::Recover
            } else if !self.attempt_payment_boundary() {
                AttemptStage::Declare
            } else if closed {
                AttemptStage::PayCosts
            } else {
                AttemptStage::GenerateMana
            },
            locked_cost: (!recovering)
                .then(|| self.current_attempt_cost().map(|(_, cost)| cost))
                .flatten(),
            mana_window_closed: closed,
            contributions_sealed: self.attempt_contributions_sealed(),
            can_end_mana_window: self.can_end_attempt_mana_window(),
            can_submit: self.can_submit_attempt(),
            can_reset_selection: self.resettable_attempt_selection().is_some(),
            failure: attempt.failure,
        })
    }

    pub fn last_attempt_outcome_view(&self, viewer: Player) -> Option<AttemptOutcomeView> {
        self.state
            .last_attempt_outcome
            .iter()
            .flatten()
            .copied()
            .filter(|outcome| self.shares_information(viewer, outcome.player))
            .max_by_key(|outcome| outcome.attempt_id)
    }
}

/// An opaque drawn card remains usable for an unrestricted discard without
/// exposing its identity or pretending that it has printed characteristics.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct AttemptDrawView {
    pub object: ObjectRef,
    pub player: Player,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct DeferredAttemptDraw {
    pub object: ObjectRef,
    pub player: Player,
    pub reveal: bool,
    pub miracle: Vec<miracle::RevealChoice>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl DeferredAttemptDraw {
    pub(super) fn bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.miracle.len() * std::mem::size_of::<miracle::RevealChoice>()
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    /// CR121.8: track the actual drawn incarnation in every enclosing action
    /// that is still being announced. A mana ability's own resolution is after
    /// activation, but any suspended parent's announcement remains unfinished.
    pub(super) fn record_attempt_draw(&mut self, object: ObjectRef) {
        let player = self.state.objects[object.card as usize].owner;
        let mut cursor = self.state.cast_rollback.as_deref_mut();
        while let Some(attempt) = cursor {
            if !attempt.special
                && !attempt.costs_complete
                && !attempt
                    .deferred_draws
                    .iter()
                    .any(|draw| draw.object == object)
            {
                attempt.deferred_draws.push(DeferredAttemptDraw {
                    object,
                    player,
                    reveal: false,
                    miracle: Vec::new(),
                });
            }
            cursor = attempt.parent.as_deref_mut();
        }
    }

    /// Called at CR601.2i/602.2b, before a mana ability starts its immediate
    /// resolution. Existing output continuations keep their checkpoint owner.
    pub(super) fn mark_attempt_costs_complete(&mut self, context: u64) {
        let Some(attempt) = self.state.cast_rollback.as_mut() else {
            return;
        };
        // A direct activation can finish in its first action, before the normal
        // post-step ownership binding. Never mistake a suspended parent's item
        // (or an unrelated trigger announcement) for this child's boundary.
        let belongs = attempt.context == Some(context)
            || (attempt.context.is_none()
                && attempt
                    .before
                    .announcing
                    .as_ref()
                    .is_none_or(|a| a.item.id != context)
                && attempt
                    .before
                    .mana_announcement_stack
                    .as_ref()
                    .is_none_or(|frames| {
                        frames
                            .iter()
                            .all(|frame| frame.announcement.item.id != context)
                    }));
        if belongs {
            attempt.context = Some(context);
            attempt.costs_complete = true;
        }
    }

    pub(super) fn attempt_draw_is_face_down(&self, object: ObjectRef) -> bool {
        if self
            .state
            .objects
            .get(object.card as usize)
            .is_none_or(|card| card.zone != Zone::Hand || card.generation != object.generation)
        {
            return false;
        }
        let mut cursor = self.state.cast_rollback.as_deref();
        while let Some(attempt) = cursor {
            if !attempt.costs_complete
                && attempt
                    .deferred_draws
                    .iter()
                    .any(|draw| draw.object == object)
            {
                return true;
            }
            cursor = attempt.parent.as_deref();
        }
        false
    }

    pub fn attempt_draws_view(&self, viewer: Player) -> Vec<AttemptDrawView> {
        let mut draws = Vec::new();
        let mut cursor = self.state.cast_rollback.as_deref();
        while let Some(attempt) = cursor {
            for draw in &attempt.deferred_draws {
                if self.attempt_draw_is_face_down(draw.object)
                    && self.shares_information(viewer, draw.player)
                    && !draws
                        .iter()
                        .any(|row: &AttemptDrawView| row.object == draw.object)
                {
                    draws.push(AttemptDrawView {
                        object: draw.object,
                        player: draw.player,
                    });
                }
            }
            cursor = attempt.parent.as_deref();
        }
        draws
    }

    pub(super) fn defer_attempt_draw_reveal(&mut self, object: ObjectRef) -> bool {
        if !self.attempt_draw_is_face_down(object) {
            return false;
        }
        let mut cursor = self.state.cast_rollback.as_deref_mut();
        while let Some(attempt) = cursor {
            for draw in &mut attempt.deferred_draws {
                if draw.object == object {
                    draw.reveal = true;
                }
            }
            cursor = attempt.parent.as_deref_mut();
        }
        true
    }

    /// Store optional draw-reveal decisions at the outermost enclosing action,
    /// retaining the original granting source even if costs later remove it.
    pub(super) fn defer_attempt_miracle_reveals(
        &mut self,
        object: ObjectRef,
        choices: &mut std::collections::VecDeque<miracle::RevealChoice>,
    ) -> bool {
        if !self.attempt_draw_is_face_down(object) {
            return false;
        }
        fn store(
            attempt: &mut cast_rollback::CastRollback,
            object: ObjectRef,
            choices: &mut std::collections::VecDeque<miracle::RevealChoice>,
        ) -> bool {
            if let Some(parent) = attempt.parent.as_mut()
                && store(parent, object, choices)
            {
                return true;
            }
            if let Some(draw) = attempt
                .deferred_draws
                .iter_mut()
                .find(|draw| draw.object == object)
            {
                draw.miracle.extend(choices.drain(..));
                return true;
            }
            false
        }
        store(self.state.cast_rollback.as_mut().unwrap(), object, choices)
    }

    pub(super) fn release_attempt_draws(&mut self, draws: Vec<DeferredAttemptDraw>) {
        let mut reveals = Vec::new();
        for draw in draws {
            if self
                .state
                .objects
                .get(draw.object.card as usize)
                .is_none_or(|card| {
                    card.zone != Zone::Hand || card.generation != draw.object.generation
                })
            {
                continue;
            }
            if draw.reveal {
                self.remember_public_hand(draw.object);
            }
            if !draw.miracle.is_empty() {
                let mut choices = draw.miracle.into_iter().collect();
                if !self.defer_attempt_miracle_reveals(draw.object, &mut choices) {
                    reveals.extend(choices);
                }
            }
        }
        self.resume_deferred_miracle_reveals(reveals);
    }
}

#[derive(Clone, Copy)]
enum AttemptSelection {
    Additional,
    Evidence,
    Forage,
    Teamwork,
    Conspire(u16),
    Crew,
    Saddle,
    Mana,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    fn resettable_attempt_selection(&self) -> Option<AttemptSelection> {
        if !self.attempt_payment_boundary() {
            return None;
        }
        let a = self.state.announcing.as_ref()?;
        // These frames retain only provisional contributors until the actual
        // activation payment. A completed selection with unpaid mana is editable.
        if matches!(
            self.state.pending,
            Pending::Crew { .. } | Pending::PayCosts { .. }
        ) && self
            .state
            .crew
            .as_ref()
            .and_then(|crew| crew.payment.as_ref())
            .is_some_and(|payment| {
                !payment.selected.is_empty()
                    && payment.context == a.item.id
                    && payment.actor == a.item.controller
                    && matches!(a.item.kind, StackKind::ActivatedAbility { .. })
                    && a.item.program().is_some_and(|(_, program)| {
                        matches!(program, [Instruction::Crew { minimum }] if *minimum == payment.minimum)
                    })
                    && a.item
                        .source
                        .is_some_and(|source| source.object == payment.source)
            })
        {
            return Some(AttemptSelection::Crew);
        }
        if matches!(
            self.state.pending,
            Pending::Saddle { .. } | Pending::PayCosts { .. }
        ) && self.state.saddle_payment.as_ref().is_some_and(|payment| {
            !payment.selected.is_empty()
                && payment.context == a.item.id
                && payment.actor == a.item.controller
                && matches!(a.item.kind, StackKind::ActivatedAbility { .. })
                && a.item.program().is_some_and(|(_, program)| {
                    matches!(program, [Instruction::BecomeSaddled { minimum }] if *minimum == payment.minimum)
                })
                && a.item
                    .source
                    .is_some_and(|source| source.object == payment.source)
        }) {
            return Some(AttemptSelection::Saddle);
        }
        let additional_unpaid = !a.additional_paid
            || a.cast_costs.as_ref().is_some_and(|costs| {
                matches!(
                    costs.active_clause(),
                    Some(CastCostClause::OriginCounters(_))
                )
            });
        match self.state.pending {
            Pending::AdditionalCost { .. } | Pending::PayCosts { .. } if additional_unpaid => {
                if a.evidence
                    .as_ref()
                    .is_some_and(|e| !e.paid && !e.selected.is_empty())
                {
                    return Some(AttemptSelection::Evidence);
                }
                if a.forage.as_ref().is_some_and(|f| !f.selected.is_empty()) {
                    return Some(AttemptSelection::Forage);
                }
                if a.additional.is_some()
                    || a.additional_life
                    || a.additional_group
                        .as_ref()
                        .is_some_and(|group| !group.is_empty())
                    || a.creature_counter_cost
                        .as_ref()
                        .is_some_and(|group| !group.is_empty())
                {
                    return Some(AttemptSelection::Additional);
                }
            }
            Pending::Evidence { .. }
                if a.evidence
                    .as_ref()
                    .is_some_and(|e| !e.paid && !e.selected.is_empty()) =>
            {
                return Some(AttemptSelection::Evidence);
            }
            Pending::Forage { .. }
                if a.forage.as_ref().is_some_and(|f| !f.selected.is_empty())
                    || a.cast_costs
                        .as_ref()
                        .and_then(|c| c.origin_forage.as_ref())
                        .is_some_and(|f| !f.selected.is_empty()) =>
            {
                return Some(AttemptSelection::Forage);
            }
            Pending::TeamworkPayment { .. }
                if a.cast_costs
                    .as_ref()
                    .is_some_and(|c| !c.teammates.is_empty()) =>
            {
                return Some(AttemptSelection::Teamwork);
            }
            Pending::ConspirePayment { instance, .. }
                if a.conspire
                    .as_ref()
                    .is_some_and(|rows| rows[usize::from(instance)].first_object.is_some()) =>
            {
                return Some(AttemptSelection::Conspire(instance));
            }
            _ => {}
        }
        if matches!(
            self.state.pending,
            Pending::PayCosts { .. }
                | Pending::Convoke { .. }
                | Pending::Improvise { .. }
                | Pending::Delve { .. }
                | Pending::Waterbend { .. }
                | Pending::CreatureTapPayment { .. }
        ) && (self.attempt_contributions_sealed()
            || a.convoke.as_ref().is_some_and(|frame| {
                !frame.selected.is_empty()
                    || !frame.improvised.is_empty()
                    || !frame.substituted.is_empty()
                    || !frame.delved.is_empty()
            }))
            && a.cast_costs.as_ref().is_none_or(|costs| !costs.mana_paid)
        {
            return Some(AttemptSelection::Mana);
        }
        None
    }

    /// Edit only the current unpaid selection. Actual cost effects, earlier
    /// paid clauses, declarations and the closed mana window are never undone.
    pub(super) fn reset_attempt_cost_selection(&mut self) {
        let selection = self
            .resettable_attempt_selection()
            .expect("editable unpaid selection");
        let a = self.state.announcing.as_mut().unwrap();
        let player = a.item.controller;
        match selection {
            AttemptSelection::Additional => {
                a.additional = None;
                a.additional_life = false;
                a.additional_group_complete = false;
                if let Some(group) = &mut a.additional_group {
                    group.clear();
                }
                if let Some(group) = &mut a.creature_counter_cost {
                    group.clear();
                }
                self.state.pending = Pending::AdditionalCost { player };
            }
            AttemptSelection::Evidence => {
                let e = a.evidence.as_mut().unwrap();
                e.selected.clear();
                e.total = 0;
                e.complete = false;
                a.additional = None;
                self.state.pending = Pending::Evidence { player };
            }
            AttemptSelection::Forage => {
                if let Some(f) = a
                    .cast_costs
                    .as_mut()
                    .and_then(|costs| costs.origin_forage.as_mut())
                {
                    f.selected.clear();
                } else {
                    a.forage.as_mut().unwrap().selected.clear();
                }
                a.additional = None;
                self.state.pending = Pending::Forage { player };
            }
            AttemptSelection::Teamwork => a.cast_costs.as_mut().unwrap().teammates.clear(),
            AttemptSelection::Conspire(instance) => {
                a.conspire.as_mut().unwrap()[usize::from(instance)].first_object = None;
            }
            AttemptSelection::Crew => {
                let payment = self.state.crew.as_mut().unwrap().payment.as_mut().unwrap();
                payment.selected.clear();
                payment.complete = false;
                self.state.pending = Pending::Crew { player };
            }
            AttemptSelection::Saddle => {
                let payment = self.state.saddle_payment.as_mut().unwrap();
                payment.selected.clear();
                payment.complete = false;
                self.state.pending = Pending::Saddle { player };
            }
            AttemptSelection::Mana => {
                if a.convoke.is_some() {
                    a.convoke = Some(Box::new(convoke::ConvokeSelection::new(
                        a.total_cost.expect("locked cost"),
                    )));
                }
                self.state
                    .cast_rollback
                    .as_mut()
                    .unwrap()
                    .contributions_sealed = false;
            }
        }
    }
}
