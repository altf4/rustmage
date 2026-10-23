//! Sneak adds a return cost to printed additional costs (CR118.9d).
//! Rare composed payments use owned cold branches; ordinary Sneak stays borrowed.
use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct SneakExtraCost {
    pub after_return: Option<bool>,
    pub selected: Option<ObjectRef>,
    pub paid: bool,
    pub projecting: bool,
}
impl Announcement {
    pub(super) fn sneak_extra(&self) -> Option<&SneakExtraCost> {
        self.item.sneak.as_ref()?.extra.as_deref()
    }
    pub(super) fn sneak_extra_mut(&mut self) -> Option<&mut SneakExtraCost> {
        self.item.sneak.as_mut()?.extra.as_deref_mut()
    }
    pub(super) fn sneak_extra_reservation(&self) -> Option<(ObjectRef, AdditionalCost)> {
        let c = self.sneak_extra()?;
        (c.after_return == Some(false) && !c.paid).then_some(())?;
        Some((c.selected?, self.sneak_extra_spec()?))
    }
    pub(super) fn sneak_extra_spec(&self) -> Option<AdditionalCost> {
        self.sneak_extra()?;
        if self
            .cost
            .is_some_and(|cost| self.mana_route_replaces_additional(cost))
        {
            return None;
        }
        self.item.source_card.additional_cost()
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SneakAdditionalCostView {
    pub kind: EffectChoiceKind,
    pub after_return: Option<bool>,
    pub selected: Option<ObjectRef>,
    pub candidates: Vec<ObjectRef>,
    pub order_options: Vec<bool>,
    pub paid: bool,
}
impl Game {
    pub(super) fn sneak_cost_projecting(&self) -> bool {
        self.state
            .announcing
            .as_ref()
            .and_then(|a| a.sneak_extra())
            .is_some_and(|c| c.projecting)
    }
    pub(super) fn sneak_additional_view(&self, viewer: Player) -> Option<SneakAdditionalCostView> {
        let a = self.state.announcing.as_ref()?;
        if viewer != a.item.controller {
            return None;
        }
        let cost = a.sneak_extra()?;
        let spec = a.sneak_extra_spec()?;
        Some(SneakAdditionalCostView {
            kind: spec.kind,
            after_return: cost.after_return,
            selected: cost.selected,
            paid: cost.paid,
            candidates: if matches!(self.state.pending, Pending::SneakAdditionalCost { .. }) {
                (0..self.state.objects.len())
                    .map(|id| self.object_ref(id as CardId))
                    .filter(|&r| self.sneak_extra_choice_available(r))
                    .collect()
            } else {
                Vec::new()
            },
            order_options: if matches!(self.state.pending, Pending::SneakCostOrder { .. }) {
                [false, true]
                    .into_iter()
                    .filter(|&after| self.sneak_extra_order_available(after))
                    .collect()
            } else {
                Vec::new()
            },
        })
    }
    pub(super) fn advance_sneak_extra_choices(&mut self) {
        if !matches!(self.state.pending, Pending::PayCosts { .. }) {
            return;
        }
        let Some(a) = self.state.announcing.as_ref() else {
            return;
        };
        if a.sneak_extra_spec().is_none() {
            return;
        }
        let extra = a.sneak_extra().unwrap();
        if extra.projecting || extra.paid {
            return;
        }
        let player = a.item.controller;
        if extra.after_return.is_none() {
            self.state.pending = Pending::SneakCostOrder { player };
        } else if extra.selected.is_none()
            && (extra.after_return == Some(false) || a.additional_paid)
        {
            self.state.pending = Pending::SneakAdditionalCost { player };
        }
    }
    pub(super) fn resume_sneak_extra_payment(&mut self) -> bool {
        if !matches!(self.state.pending, Pending::PayCosts { .. })
            || self.state.entry_batch.is_some()
            || self.state.replacements.is_some()
        {
            return false;
        }
        let resume = self.state.announcing.as_ref().is_some_and(|a| {
            a.sneak_extra_spec().is_some()
                && a.sneak_extra().is_some_and(|c| {
                    !c.projecting
                        && ((c.paid && !a.additional_paid)
                            || (a.additional_paid && c.selected.is_some()))
                })
        });
        if resume {
            self.finish_announcement();
        }
        resume
    }
    pub(super) fn sneak_extra_order_available(&self, after: bool) -> bool {
        let a = self.state.announcing.as_ref().unwrap();
        if a.sneak_extra().is_some_and(|c| c.projecting) {
            return true;
        }
        a.cost
            .into_iter()
            .chain(a.cost_options().filter(|_| a.cost.is_none()))
            .any(|mana| self.sneak_extra_fundable(a, mana, Some(after), None))
    }
    pub(super) fn sneak_extra_choice_available(&self, object: ObjectRef) -> bool {
        let a = self.state.announcing.as_ref().unwrap();
        let Some(spec) = a.sneak_extra_spec() else {
            return false;
        };
        if !self.additional_candidate(spec, a.item.controller, a.item.object, object) {
            return false;
        }
        let c = a.sneak_extra().unwrap();
        if a.additional_paid || c.projecting {
            return true;
        }
        a.cost
            .into_iter()
            .chain(a.cost_options().filter(|_| a.cost.is_none()))
            .any(|mana| self.sneak_extra_fundable(a, mana, c.after_return, Some(object)))
    }
    pub(super) fn choose_sneak_extra(&mut self, object: ObjectRef) {
        let a = self.state.announcing.as_mut().unwrap();
        a.sneak_extra_mut().unwrap().selected = Some(object);
        let player = a.item.controller;
        self.state.pending = Pending::PayCosts { player };
        if a.additional_paid {
            self.finish_announcement();
        }
    }
    pub(super) fn pay_sneak_extra(&mut self, a: &mut Announcement, after: bool) {
        let Some(spec) = a.sneak_extra_spec() else {
            return;
        };
        let c = a.sneak_extra().unwrap();
        if c.paid || c.after_return != Some(after) || (c.projecting && after) {
            return;
        }
        let selected = c.selected.expect("printed Sneak cost selected");
        let mut payment = a.clone();
        payment.additional = Some(selected);
        payment.additional_paid = false;
        self.pay_additional_cost_spec(&mut payment, spec);
        a.item = payment.item;
        a.sneak_extra_mut().unwrap().paid = true;
    }
    /// Stop after the first nonmana cost during a cold feasibility projection.
    pub(super) fn defer_sneak_extra_before_return(&mut self, a: &mut Announcement) -> bool {
        if a.sneak_extra_spec().is_none() {
            return false;
        }
        a.sneak_extra().is_some_and(|c| {
            c.projecting && c.after_return == Some(false) && c.paid && !a.additional_paid
        })
    }
    pub(super) fn defer_sneak_extra_after_return(&mut self, a: &mut Announcement) -> bool {
        if a.sneak_extra_spec().is_none() {
            return false;
        }
        let c = a.sneak_extra().unwrap();
        if c.projecting {
            return true;
        }
        if c.after_return == Some(true) && !c.paid && c.selected.is_none() {
            self.state.pending = Pending::SneakAdditionalCost {
                player: a.item.controller,
            };
            return true;
        }
        false
    }
    pub(super) fn sneak_extra_fundable(
        &self,
        announcement: &Announcement,
        mana: ManaCost,
        after: Option<bool>,
        selected: Option<ObjectRef>,
    ) -> bool {
        [false, true].into_iter().any(|order| {
            if after.is_some_and(|chosen| chosen != order) {
                return false;
            }
            let mut child = self.clone();
            let mut a = announcement.clone();
            a.cost = Some(mana);
            // No explicit selection is stored for a sole cost equivalent;
            // its total is nevertheless fixed before mana abilities.
            if announcement.cost.is_some_and(|prior| prior != mana) {
                a.total_cost = None;
            }
            let c = a.sneak_extra_mut().unwrap();
            c.projecting = true;
            c.after_return = Some(order);
            c.selected = selected;
            child.state.announcing = Some(a);
            child.state.pending = Pending::Targets {
                player: announcement.item.controller,
            };
            child.continue_announcement_choices(false);
            child.sneak_extra_completion_exists()
        })
    }
    pub(super) fn sneak_extra_action_viable(&self, action: &Action) -> bool {
        if *action == Action::Concede || self.linked_funding_projecting() {
            return true;
        }
        let Some(a) = self.state.announcing.as_ref() else {
            return true;
        };
        let Some(c) = a.sneak_extra() else {
            return true;
        };
        if c.projecting || a.sneak_extra_spec().is_none() {
            return true;
        }
        if !matches!(
            self.state.pending,
            Pending::PayCosts { .. }
                | Pending::Convoke { .. }
                | Pending::SneakAdditionalCost { .. }
        ) && self.state.entry_batch.is_none()
            && self.state.replacements.is_none()
        {
            return true;
        }
        if matches!(self.state.pending, Pending::SneakAdditionalCost { .. }) && a.additional_paid {
            return true;
        }
        let mut child = self.clone();
        child
            .state
            .announcing
            .as_mut()
            .unwrap()
            .sneak_extra_mut()
            .unwrap()
            .projecting = true;
        let Some(actor) = child.actor() else {
            return false;
        };
        if matches!(action, Action::PayCosts { .. }) && !child.sneak_first_cost_valid() {
            return false;
        }
        child.apply(actor, action.clone()).is_ok() && child.sneak_extra_completion_exists()
    }
    fn sneak_first_cost_valid(&self) -> bool {
        let a = self.state.announcing.as_ref().unwrap();
        let c = a.sneak_extra().unwrap();
        if c.after_return == Some(false) && !c.paid {
            return c.selected.is_some_and(|r| {
                self.additional_candidate(
                    a.sneak_extra_spec().unwrap(),
                    a.item.controller,
                    a.item.object,
                    r,
                )
            });
        }
        self.sneak_reservation_valid(a)
    }
    // Pure mana production and tapping cannot introduce a missing nonmana
    // witness for these keyword-free filters. Linked returns, counter changes
    // and other object-changing costs require the full cold search.
    fn sneak_extra_upper_allows(&self) -> bool {
        if !self.state.exile_links.is_empty()
            || self.state.objects.iter().any(|o| {
                o.zone == Zone::Battlefield
                    && o.card.activations().iter().enumerate().any(|(index, a)| {
                        a.mana_production().is_some()
                            && (a.cost.sacrifice
                                || a.cost.exile_self
                                || a.cost.discard
                                || a.cost.additional.is_some()
                                || a.cost.remove.is_some()
                                || a.cost.life > 0
                                || o.card.activation_put_counter(index as u16).is_some())
                    })
            })
        {
            return true;
        }
        let a = self.state.announcing.as_ref().unwrap();
        let c = a.sneak_extra().unwrap();
        let Some(spec) = a.sneak_extra_spec() else {
            return true;
        };
        if spec.filter.keywords != Keywords::NONE {
            return true;
        }
        let choices = if c.after_return == Some(false) {
            c.selected
                .into_iter()
                .chain(
                    (0..self.state.objects.len())
                        .filter(|_| c.selected.is_none())
                        .map(|id| self.object_ref(id as CardId)),
                )
                .filter(|&r| self.additional_candidate(spec, a.item.controller, a.item.object, r))
                .map(Some)
                .collect::<Vec<_>>()
        } else {
            vec![None]
        };
        choices.into_iter().any(|choice| {
            let mut child = self.clone();
            let proposal = child.state.announcing.as_mut().unwrap();
            proposal.sneak_extra_mut().unwrap().selected = choice;
            proposal.total_cost = Some(ManaCost {
                colored: [0; 6],
                generic: 0,
            });
            proposal.convoke = None;
            child.state.pending = Pending::PayCosts {
                player: a.item.controller,
            };
            if !child.sneak_first_cost_valid()
                || child
                    .apply(a.item.controller, Action::PayCosts { payment: [0; 6] })
                    .is_err()
            {
                return false;
            }
            if child.state.entry_batch.is_some() || child.state.replacements.is_some() {
                return true;
            }
            let Some(p) = child.state.announcing.as_ref() else {
                return true;
            };
            if c.after_return == Some(false) {
                child.sneak_reservation_valid(p)
            } else {
                (0..child.state.objects.len()).any(|id| {
                    child.additional_candidate(
                        spec,
                        p.item.controller,
                        p.item.object,
                        child.object_ref(id as CardId),
                    )
                })
            }
        })
    }
    fn sneak_extra_completion_exists(self) -> bool {
        fn visit(mut game: Game, seen: &mut Vec<Snapshot>) -> bool {
            let Some(a) = game.state.announcing.as_ref() else {
                return true;
            };
            if game.state.entry_batch.is_none() && game.state.replacements.is_none() {
                let c = a.sneak_extra().unwrap();
                if c.after_return == Some(false) && c.paid {
                    return game.sneak_reservation_valid(a);
                }
                if a.additional_paid {
                    return a.sneak_extra_spec().is_none_or(|spec| {
                        (0..game.state.objects.len()).any(|id| {
                            game.additional_candidate(
                                spec,
                                a.item.controller,
                                a.item.object,
                                game.object_ref(id as CardId),
                            )
                        })
                    });
                }
                if c.after_return == Some(false)
                    && c.selected.is_none()
                    && matches!(game.state.pending, Pending::PayCosts { .. })
                {
                    game.state.pending = Pending::SneakAdditionalCost {
                        player: a.item.controller,
                    };
                }
            }
            if game.state.entry_batch.is_none()
                && game.state.replacements.is_none()
                && matches!(
                    game.state.pending,
                    Pending::PayCosts { .. } | Pending::SneakAdditionalCost { .. }
                )
                && !game.sneak_extra_upper_allows()
            {
                return false;
            }
            let snapshot = game.snapshot();
            if seen.contains(&snapshot) {
                return false;
            }
            seen.push(snapshot);
            let Some(actor) = game.actor() else {
                return false;
            };
            // Pay mana first when possible. Impossible witnesses must not force
            // exploration of every permutation of irrelevant basic-land taps.
            if let Some(domain) = game.mana_lot_spend_range() {
                let mut try_amount = |amount| {
                    let mut child = game.clone();
                    child
                        .apply(actor, Action::ChooseManaLotSpend { amount })
                        .is_ok()
                        && visit(child, seen)
                };
                return if let Some(values) = domain.values {
                    values.into_iter().any(try_amount)
                } else {
                    (domain.minimum..=domain.maximum).any(&mut try_amount)
                };
            }
            let mut actions = game.legal_actions();
            actions.sort_by_key(|a| !matches!(a, Action::PayCosts { .. }));
            actions
                .into_iter()
                .filter(|a| *a != Action::Concede)
                .any(|action| {
                    if matches!(action, Action::PayCosts { .. }) && !game.sneak_first_cost_valid() {
                        return false;
                    }
                    let mut child = game.clone();
                    child.apply(actor, action).is_ok() && visit(child, seen)
                })
        }
        visit(self, &mut Vec::new())
    }
}
