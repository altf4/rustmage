//! Rare exact completion of mana windows whose costs can release CR610.3 returns.
//! The ordinary path retains its closed funding graph. Here actual owned
//! engine transitions preserve entry choices, copied abilities and mana provenance.
//!
//! Finiteness: admitted pure AddMana activations consume tap/once/exhaust,
//! sacrifice/exile/discard, positive life, or positive removal counters (the
//! intake regression checks the entire supported catalogue). They do not untap,
//! create permanents, gain life or resolve queued triggers/SBAs. Returns consume
//! an existing finite ExileLink row; pure mana never creates another such row.
//! Fresh entry choices are therefore finite except prospective copy cycles,
//! which use the shared canonical entry key and remaining-clock dominance.
//! Numeric mana-allocation domains are finite and streamed without materializing
//! their potentially u32-wide action spaces; no arbitrary search cap is used.
use super::*;
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct LinkedFundingGoal {
    pub goal: FundingGoal,
    pub purpose: mana_lots::Purpose,
    pub convoke: bool,
    pub evidence: Option<u16>,
    pub group: Vec<(ObjectRef, AdditionalCost)>,
    pub counters: Option<(u16, Vec<(ObjectRef, CounterKind)>)>,
    pub deferred: Option<Box<DeferredFundingCost>>,
}
impl LinkedFundingGoal {
    pub fn new(goal: FundingGoal, purpose: mana_lots::Purpose) -> Self {
        Self {
            goal,
            purpose,
            convoke: matches!(purpose, mana_lots::Purpose::Spell(_)),
            evidence: None,
            group: Vec::new(),
            counters: None,
            deferred: None,
        }
    }
}
/// Reuse the existing typed cost checker after mana abilities have returned new
/// resources. Probe guards prevent it recursively starting another rare search.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum DeferredFundingCost {
    Spell {
        card: CardId,
        cost: ManaCost,
        route: Option<ForageRoute>,
        definition: Card,
    },
    Activation(AdditionalCost),
    Additional {
        source: ObjectRef,
        spec: AdditionalCost,
    },
    BeholdGroup {
        announcement: Box<Announcement>,
        cost: Option<ManaCost>,
        selected: Vec<ObjectRef>,
    },
}
/// Ephemeral analysis context, owned by a counterfactual Game, deliberately
/// outside State so replacement/allocation checkpoint replay retains the guard.
#[derive(Clone, Debug, Default)]
pub(super) struct LinkedFundingProjection {
    pub probe: Option<LinkedFundingGoal>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub(super) enum LinkedFundingWindow {
    Announcement(u64),
    Plot(ObjectRef),
    Resolution(u64),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum CostCompletionView {
    Announcement(u64),
    Plot,
    Resolution(u64),
}
impl From<LinkedFundingWindow> for CostCompletionView {
    fn from(window: LinkedFundingWindow) -> Self {
        match window {
            LinkedFundingWindow::Announcement(id) => Self::Announcement(id),
            LinkedFundingWindow::Plot(_) => Self::Plot,
            LinkedFundingWindow::Resolution(id) => Self::Resolution(id),
        }
    }
}
impl Game {
    pub(super) fn linked_funding_can_probe(&self, actor: Player) -> bool {
        self.linked_funding_probe().is_none() && self.linked_funding_needed(actor)
    }
    pub(super) fn linked_deferred_funding(
        &self,
        goal: FundingGoal,
        purpose: mana_lots::Purpose,
        deferred: DeferredFundingCost,
    ) -> Option<bool> {
        let mut request = LinkedFundingGoal::new(goal, purpose);
        request.deferred = Some(Box::new(deferred));
        self.linked_funding_search(request)
    }
    pub(super) fn linked_deferred_cost_ready(&self) -> bool {
        let request = self.linked_funding_probe().unwrap();
        match request.deferred.as_deref() {
            None => true,
            Some(DeferredFundingCost::Spell {
                card,
                cost,
                route,
                definition,
            }) => self.spell_can_fund_total_with_purpose(
                request.goal.actor,
                *card,
                *cost,
                *route,
                *definition,
                request.purpose,
            ),
            Some(DeferredFundingCost::Additional { source, spec }) => (0..self.state.objects.len())
                .any(|id| {
                    let object = self.object_ref(id as CardId);
                    self.additional_candidate(*spec, request.goal.actor, *source, object)
                        && self.can_fund_for(
                            FundingGoal {
                                reserve: Some((object, *spec)),
                                ..request.goal.clone()
                            },
                            request.purpose,
                        )
                }),
            Some(DeferredFundingCost::Activation(spec)) => {
                self.activation_can_fund(request.goal.clone(), Some(*spec))
            }
            Some(DeferredFundingCost::BeholdGroup {
                announcement,
                cost,
                selected,
            }) => self.behold_group_fundable(announcement, *cost, selected),
        }
    }
    pub(super) fn visit_linked_pre_cost_mana(&self, emit: &mut impl FnMut(Action) -> bool) -> bool {
        self.linked_funding_active()
            && self
                .state
                .announcing
                .as_ref()
                .is_some_and(|a| a.total_cost.is_some() && !a.additional_paid)
            && self.visit_linked_funding_mana(emit)
    }
    pub(super) fn linked_funding_probe(&self) -> Option<&LinkedFundingGoal> {
        self.funding_projection.as_ref()?.probe.as_ref()
    }
    pub(super) fn linked_funding_projecting(&self) -> bool {
        self.funding_projection.is_some()
    }
    fn linked_funding_window(&self) -> Option<LinkedFundingWindow> {
        if let Some(a) = &self.state.announcing {
            return Some(LinkedFundingWindow::Announcement(a.item.id));
        }
        if let Some(p) = self.plot_payment() {
            return Some(LinkedFundingWindow::Plot(p.object));
        }
        self.state
            .resolving
            .as_ref()
            .filter(|r| r.mana_payment.is_some())
            .map(|r| LinkedFundingWindow::Resolution(r.item.id))
    }
    pub(super) fn linked_funding_needed(&self, actor: Player) -> bool {
        self.state.exile_links.iter().any(|link| {
            if link.return_zone != Zone::Battlefield
                || self.object_ref(link.exiled.card) != link.exiled
                || self.state.objects[link.exiled.card as usize].zone != Zone::Exile
                || !self.on_battlefield(link.source.card)
                || self.object_ref(link.source.card) != link.source
            {
                return false;
            }
            let source = &self.state.objects[link.source.card as usize];
            if source.controller == actor
                && source.card.activations().iter().any(|a| {
                    a.mana_production().is_some() && (a.cost.sacrifice || a.cost.exile_self)
                })
            {
                return true;
            }
            self.state.objects.iter().enumerate().any(|(id, o)| {
                o.zone == Zone::Battlefield
                    && o.controller == actor
                    && o.card.activations().iter().any(|a| {
                        a.mana_production().is_some()
                            && a.cost.additional.is_some_and(|cost| {
                                matches!(
                                    cost.kind,
                                    EffectChoiceKind::Sacrifice | EffectChoiceKind::BeholdAndExile
                                ) && self.additional_candidate(
                                    cost,
                                    actor,
                                    self.object_ref(id as CardId),
                                    link.source,
                                )
                            })
                    })
            })
        })
    }
    pub(super) fn capture_linked_funding_window(&mut self) {
        if self.linked_funding_probe().is_some()
            || (self.state.exile_links.is_empty() && !self.linked_funding_active())
        {
            return;
        }
        let current = self.linked_funding_window();
        if let Some(roles) = &mut self.state.roles
            && roles
                .linked_funding
                .as_deref()
                .is_some_and(|window| Some(*window) != current)
        {
            roles.linked_funding = None;
        }
        if let Some(window) = current
            && self
                .actor()
                .is_some_and(|actor| self.linked_funding_needed(actor))
        {
            self.state.roles.get_or_insert_default().linked_funding = Some(Box::new(window));
        }
    }
    pub(super) fn linked_funding_active(&self) -> bool {
        self.linked_funding_projecting()
            || self
                .state
                .roles
                .as_ref()
                .is_some_and(|r| r.linked_funding.is_some())
    }
    /// Preserve suspended entry/replacement/bonus choices; otherwise resume only
    /// this mana window, never priority, triggered abilities or state-based actions.
    pub(super) fn resume_linked_funding_probe(&mut self) -> bool {
        let Some(probe) = self.linked_funding_probe() else {
            return false;
        };
        let player = probe.goal.actor;
        self.state.pending = Pending::FundingProbe { player };
        true
    }
    pub(super) fn linked_funding_search(&self, request: LinkedFundingGoal) -> Option<bool> {
        // A probe's leaf/move helpers never call reachability, so nested entry
        // feasibility retains its own request rather than recursing on the goal.
        if self.linked_funding_probe().is_some() {
            return None;
        }
        if !self.linked_funding_needed(request.goal.actor) {
            return None;
        }
        let mut child = self.clone();
        child.funding_projection = Some(Box::new(LinkedFundingProjection {
            probe: Some(request),
        }));
        if child.state.entry_batch.is_none()
            && child.state.replacements.is_none()
            && !matches!(
                child.state.pending,
                Pending::ZoneReplacement { .. } | Pending::ManaLotSpend { .. }
            )
            && !child.mana_tap_bonus_pending()
        {
            child.resume_linked_funding_probe();
        }
        Some(child.linked_funding_completion(&mut Vec::new()))
    }
    fn linked_funding_completion(self, seen: &mut Vec<(Snapshot, u64)>) -> bool {
        if self.linked_funding_probe().is_some() {
            if matches!(self.state.pending, Pending::FundingProbe { .. })
                && self.linked_funding_probe_paid()
            {
                return true;
            }
        } else if self
            .state
            .roles
            .as_ref()
            .and_then(|r| r.linked_funding.as_deref())
            .copied()
            .is_none_or(|window| Some(window) != self.linked_funding_window())
        {
            return true;
        }
        // Completing a legal payment can lose at the following SBA checkpoint.
        // That does not make the preceding cost action illegal.
        if self.state.outcome.is_some() {
            return false;
        }
        let (snapshot, clock) = self.entry_analysis_snapshot();
        if seen
            .iter()
            .any(|(prior, prior_clock)| *prior == snapshot && *prior_clock <= clock)
        {
            return false;
        }
        seen.push((snapshot, clock));
        let Some(actor) = self.actor() else {
            return false;
        };
        // A provenance scalar domain can have holes and values beyond the
        // concrete action-vector representatives. Enumerate its exact domain.
        let mut try_action = |action| {
            if action == Action::Concede {
                return false;
            }
            let mut child = self.clone();
            child.apply(actor, action).is_ok() && child.linked_funding_completion(seen)
        };
        if let Some(domain) = self.mana_lot_spend_range() {
            if let Some(values) = domain.values {
                values
                    .into_iter()
                    .any(|amount| try_action(Action::ChooseManaLotSpend { amount }))
            } else {
                (domain.minimum..=domain.maximum)
                    .any(|amount| try_action(Action::ChooseManaLotSpend { amount }))
            }
        } else {
            self.legal_actions().into_iter().any(try_action)
        }
    }

    pub(super) fn linked_funding_action_viable(&self, action: &Action) -> bool {
        if *action == Action::Concede || self.linked_funding_projecting() {
            return true;
        }
        let watched = self
            .state
            .roles
            .as_ref()
            .and_then(|r| r.linked_funding.as_deref())
            .copied();
        if watched.is_none() && !self.actor().is_some_and(|p| self.linked_funding_needed(p)) {
            return true;
        }
        // Priority-only mana actions need not complete a future declaration.
        if watched.is_none()
            && self.linked_funding_window().is_none()
            && !matches!(
                action,
                Action::AnnounceCast { .. }
                    | Action::AnnounceActivate { .. }
                    | Action::BeginCast { .. }
                    | Action::BeginActivate { .. }
                    | Action::Cast { .. }
                    | Action::Activate { .. }
                    | Action::AnnouncePlot { .. }
            )
        {
            return true;
        }
        let mut child = self.clone();
        child.funding_projection = Some(Box::default());
        child.capture_linked_funding_window();
        let Some(actor) = child.actor() else {
            return false;
        };
        if child.apply(actor, action.clone()).is_err() {
            return false;
        }
        child.capture_linked_funding_window();
        child.linked_funding_completion(&mut Vec::new())
    }
    pub(super) fn prepare_linked_mana_allocations(&mut self, action: &Action) -> bool {
        let (Action::ActivateMana { payment, .. }
        | Action::ActivateManaWithTapCost { payment, .. }) = action
        else {
            return false;
        };
        // A zero payment never consumes prepared allocations in spend_mana_for.
        // Do not leave a zero vector for the later enclosing spell payment.
        if *payment == [0; 6] {
            return false;
        }
        if !self.linked_funding_active() {
            return false;
        }
        if self
            .state
            .mana_lots
            .as_ref()
            .is_some_and(|s| s.replay.is_some())
        {
            return true;
        }
        let Some(actor) = self.actor() else {
            return true;
        };
        if self.mana_lots(actor).is_empty() {
            return true;
        }
        let plans = self.linked_funding_allocations(action);
        let mut viable = Vec::new();
        for plan in plans {
            if self.linked_funding_projecting() {
                viable.push(plan);
                continue;
            }
            let mut child = self.clone();
            child.funding_projection = Some(Box::default());
            child.state.mana_lots.get_or_insert_default().replay = Some(plan.clone());
            if child.apply(actor, action.clone()).is_ok()
                && child.linked_funding_completion(&mut Vec::new())
            {
                viable.push(plan);
            }
        }
        if !viable.is_empty() {
            self.state.mana_lots.get_or_insert_default().prepared = Some(viable);
        }
        true
    }
}
