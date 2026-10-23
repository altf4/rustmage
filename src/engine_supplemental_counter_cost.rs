//! A linked casting permission supplements, rather than replaces, printed costs.
//! Cold completion branches reuse actual mana, entry and replacement actions:
//! printed costs may return fresh permanents, and mana abilities can consume or
//! create the same counters. No completion branch grants players priority.
use super::*;

impl Game {
    pub(super) fn advance_supplemental_counter_choices(&mut self) {
        if !matches!(self.state.pending, Pending::PayCosts { .. }) {
            return;
        }
        let Some(a) = self.state.announcing.as_mut() else {
            return;
        };
        if !a
            .item
            .cast
            .is_some_and(|cast| cast.method == CastMethod::LinkedExileCounterCost)
        {
            return;
        }
        let player = a.item.controller;
        let printed = a.additional_cost().is_some();
        let cost = a
            .creature_counter_cost
            .as_mut()
            .expect("captured supplemental cost");
        if cost.projecting || cost.paid {
            return;
        }
        if cost.after_printed.is_none() {
            if printed {
                self.state.pending = Pending::CounterCostOrder { player };
                return;
            }
            cost.after_printed = Some(false);
        }
        if (cost.after_printed == Some(false) || a.additional_paid)
            && cost.len() < usize::from(cost.amount.unwrap())
        {
            self.state.pending = Pending::CreatureCounterCost { player };
        }
    }
    pub(super) fn resume_supplemental_counter_payment(&mut self) -> bool {
        if !matches!(self.state.pending, Pending::PayCosts { .. })
            || self.state.entry_batch.is_some()
            || self.state.replacements.is_some()
        {
            return false;
        }
        let resume = self.state.announcing.as_ref().is_some_and(|a| {
            a.additional_paid
                && a.item
                    .cast
                    .is_some_and(|cast| cast.method == CastMethod::LinkedExileCounterCost)
                && a.creature_counter_cost.as_ref().is_some_and(|cost| {
                    !cost.projecting && (cost.paid || cost.after_printed == Some(true))
                })
        });
        if resume {
            self.finish_announcement();
        }
        resume
    }
    pub(super) fn supplemental_counter_order_available(&self, after_printed: bool) -> bool {
        let a = self.state.announcing.as_ref().unwrap();
        a.cost
            .into_iter()
            .chain(a.cost_options().filter(|_| a.cost.is_none()))
            .any(|cost| self.supplemental_counter_fundable(a, cost, &[], Some(after_printed)))
    }
    pub(super) fn supplemental_counter_choice_fundable(
        &self,
        a: &Announcement,
        selected: &[(ObjectRef, CounterKind)],
    ) -> bool {
        let cost = a.creature_counter_cost.as_ref().unwrap();
        if a.additional_paid && cost.after_printed == Some(true) {
            return self.live_supplemental_counters_available(
                a.item.controller,
                cost.amount.unwrap(),
                selected,
            );
        }
        a.cost
            .into_iter()
            .chain(a.cost_options().filter(|_| a.cost.is_none()))
            .any(|mana| self.supplemental_counter_fundable(a, mana, selected, cost.after_printed))
    }
    pub(super) fn live_supplemental_counters_available(
        &self,
        player: Player,
        amount: u16,
        selected: &[(ObjectRef, CounterKind)],
    ) -> bool {
        if selected.len() > usize::from(amount) {
            return false;
        }
        let mut requested = std::collections::BTreeMap::<(ObjectRef, CounterKind), u64>::new();
        for &(object, kind) in selected {
            if !self.on_battlefield(object.card)
                || self.object_ref(object.card) != object
                || self.state.objects[object.card as usize].controller != player
                || !self.is_creature(object.card)
            {
                return false;
            }
            *requested.entry((object, kind)).or_default() += 1;
        }
        if requested
            .iter()
            .any(|((object, kind), count)| self.counter_count(object.card, *kind) < *count)
        {
            return false;
        }
        let total = self
            .state
            .objects
            .iter()
            .enumerate()
            .filter(|(id, o)| {
                o.zone == Zone::Battlefield
                    && o.controller == player
                    && self.is_creature(*id as CardId)
            })
            .map(|(id, _)| self.counter_amounts(id as CardId).total())
            .sum::<u128>();
        total >= u128::from(amount)
    }
    pub(super) fn pay_supplemental_counters(&mut self, a: &mut Announcement, after_printed: bool) {
        if !a
            .item
            .cast
            .is_some_and(|cast| cast.method == CastMethod::LinkedExileCounterCost)
        {
            return;
        }
        let cost = a.creature_counter_cost.as_mut().unwrap();
        if cost.paid
            || cost.after_printed != Some(after_printed)
            || cost.projecting && after_printed
        {
            return;
        }
        assert_eq!(
            cost.len(),
            usize::from(cost.amount.unwrap()),
            "complete supplemental declaration"
        );
        assert!(self.live_supplemental_counters_available(
            a.item.controller,
            cost.amount.unwrap(),
            &cost.selected
        ));
        for &(object, kind) in &cost.selected {
            self.pay_counters(object.card, &Counters::of(kind, 1));
        }
        cost.paid = true;
    }
    pub(super) fn defer_supplemental_counters_after(&mut self, a: &mut Announcement) -> bool {
        if !a
            .item
            .cast
            .is_some_and(|cast| cast.method == CastMethod::LinkedExileCounterCost)
        {
            return false;
        }
        let cost = a.creature_counter_cost.as_ref().unwrap();
        if cost.projecting {
            return true;
        }
        if cost.after_printed != Some(true)
            || cost.paid
            || cost.len() == usize::from(cost.amount.unwrap())
        {
            return false;
        }
        self.state.pending = Pending::CreatureCounterCost {
            player: a.item.controller,
        };
        true
    }
    fn supplemental_cost_projection(&self) -> bool {
        self.state.announcing.as_ref().is_some_and(|a| {
            a.creature_counter_cost
                .as_ref()
                .is_some_and(|cost| cost.projecting)
        })
    }
    pub(super) fn supplemental_counter_fundable(
        &self,
        announcement: &Announcement,
        mana: ManaCost,
        selected: &[(ObjectRef, CounterKind)],
        after_printed: Option<bool>,
    ) -> bool {
        let amount = announcement.creature_counter_cost_amount().unwrap();
        [false, true].into_iter().any(|after| {
            if after_printed.is_some_and(|chosen| chosen != after) {
                return false;
            }
            let mut child = self.clone();
            let mut a = announcement.clone();
            a.cost = Some(mana);
            // Existing total costs include target-dependent changes and must stay
            // frozen once announced. A new cost alternative has no locked total.
            if announcement.cost != Some(mana) {
                a.total_cost = None;
            }
            let c = a.creature_counter_cost.as_mut().unwrap();
            c.projecting = true;
            c.after_printed = Some(after);
            c.selected = selected.to_vec();
            child.state.announcing = Some(a);
            child.state.pending = Pending::Targets {
                player: announcement.item.controller,
            };
            child.continue_announcement_choices(false);
            child.supplemental_counter_completion_exists(amount, selected)
        })
    }
    fn supplemental_counter_declaration_valid(&self, a: &Announcement) -> bool {
        let cost = a.creature_counter_cost.as_ref().unwrap();
        let amount = cost.amount.unwrap();
        if cost.len() != usize::from(amount)
            || !self.live_supplemental_counters_available(a.item.controller, amount, &cost.selected)
        {
            return false;
        }
        // Paying counters can change a printed witness's characteristics.
        let mut preview = self.clone();
        for &(r, kind) in &cost.selected {
            preview.pay_counters(r.card, &Counters::of(kind, 1));
        }
        a.additional_cost().is_none_or(|spec| {
            a.additional_group
                .as_deref()
                .map(Vec::as_slice)
                .unwrap_or_else(|| a.additional.as_slice())
                .iter()
                .all(|&r| preview.additional_candidate(spec, a.item.controller, a.item.object, r))
        })
    }
    pub(super) fn supplemental_counter_action_viable(&self, action: &Action) -> bool {
        if *action == Action::Concede || self.supplemental_cost_projection() {
            return true;
        }
        let Some(a) = self.state.announcing.as_ref() else {
            return true;
        };
        if !a
            .item
            .cast
            .is_some_and(|c| c.method == CastMethod::LinkedExileCounterCost)
        {
            return true;
        }
        let Some(cost) = &a.creature_counter_cost else {
            return true;
        };
        // Declaration phases perform their own combined funding checks. Once
        // payment begins, every mana/entry/replacement prefix needs a completion.
        if !matches!(self.state.pending, Pending::PayCosts { .. })
            && self.state.entry_batch.is_none()
            && self.state.replacements.is_none()
        {
            return true;
        }
        let amount = cost.amount.unwrap();
        let selected = cost.selected.clone();
        if matches!(action, Action::PayCosts { .. })
            && cost.after_printed == Some(false)
            && !cost.paid
            && !self.supplemental_counter_declaration_valid(a)
        {
            return false;
        }
        let mut child = self.clone();
        child
            .state
            .announcing
            .as_mut()
            .unwrap()
            .creature_counter_cost
            .as_mut()
            .unwrap()
            .projecting = true;
        let Some(actor) = child.actor() else {
            return false;
        };
        child.apply(actor, action.clone()).is_ok()
            && child.supplemental_counter_completion_exists(amount, &selected)
    }
    // A necessary bound only: count every player's permanents, because a mana
    // source departure can change control or characteristics. Linked returns and counter-producing
    // mana costs disable the bound; those paths need the actual cold completion.
    fn supplemental_counter_upper_allows(&self, a: &Announcement, amount: u16) -> bool {
        if !self.state.exile_links.is_empty()
            || self.state.objects.iter().any(|o| {
                o.zone == Zone::Battlefield
                    && o.card
                        .activations()
                        .iter()
                        .enumerate()
                        .any(|(index, ability)| {
                            ability.mana_production().is_some()
                                && o.card.activation_put_counter(index as u16).is_some()
                        })
            })
            || a.additional_cost()
                .is_some_and(|spec| matches!(spec.kind, EffectChoiceKind::BlightCost(_)))
        {
            return true;
        }
        let leaving = (a
            .creature_counter_cost
            .as_ref()
            .is_some_and(|c| c.after_printed == Some(true))
            && a.additional_cost().is_some_and(|spec| {
                matches!(
                    spec.kind,
                    EffectChoiceKind::Sacrifice
                        | EffectChoiceKind::SacrificeCreatureOrToken
                        | EffectChoiceKind::ReturnToHand
                        | EffectChoiceKind::BeholdAndExile
                )
            }))
        .then_some(a.additional)
        .flatten();
        let upper = self
            .state
            .objects
            .iter()
            .enumerate()
            .filter(|(id, o)| {
                o.zone == Zone::Battlefield && leaving != Some(self.object_ref(*id as CardId))
            })
            .map(|(id, _)| self.counter_amounts(id as CardId).total())
            .sum::<u128>();
        upper >= u128::from(amount)
    }
    /// This rare permission can return fresh permanents while paying printed
    /// costs. Own the cold branch and reuse actual mana, entry and replacement
    /// actions instead of approximating those effects in the borrowed mana graph.
    /// Snapshot equality omits external revisions and prevents repeated prefixes.
    pub(super) fn supplemental_counter_completion_exists(
        self,
        amount: u16,
        selected: &[(ObjectRef, CounterKind)],
    ) -> bool {
        fn counter_declarations(
            game: &Game,
            amount: u16,
            selected: &mut Vec<(ObjectRef, CounterKind)>,
            start: usize,
            action: &Action,
            seen: &mut Vec<Snapshot>,
        ) -> bool {
            let player = game.state.announcing.as_ref().unwrap().item.controller;
            if !game.live_supplemental_counters_available(player, amount, selected) {
                return false;
            }
            if selected.len() == usize::from(amount) {
                let mut child = game.clone();
                let a = child.state.announcing.as_mut().unwrap();
                a.creature_counter_cost.as_mut().unwrap().selected = selected.clone();
                if !child.supplemental_counter_declaration_valid(
                    child.state.announcing.as_ref().unwrap(),
                ) {
                    return false;
                }
                return child.apply(player, action.clone()).is_ok() && visit(child, amount, seen);
            }
            let choices: Vec<_> = game
                .state
                .objects
                .iter()
                .enumerate()
                .filter(|(id, o)| {
                    o.zone == Zone::Battlefield
                        && o.controller == player
                        && game.is_creature(*id as CardId)
                })
                .flat_map(|(id, _)| {
                    CounterKind::ALL
                        .into_iter()
                        .map(move |kind| (game.object_ref(id as CardId), kind))
                })
                .filter(|(r, k)| game.counter_count(r.card, *k) > 0)
                .collect();
            for (index, &choice) in choices.iter().enumerate().skip(start) {
                selected.push(choice);
                if counter_declarations(game, amount, selected, index, action, seen) {
                    return true;
                }
                selected.pop();
            }
            false
        }
        fn visit(mut game: Game, amount: u16, seen: &mut Vec<Snapshot>) -> bool {
            if game.state.entry_batch.is_none()
                && game.state.replacements.is_none()
                && let Some(a) = &game.state.announcing
                && a.additional_paid
            {
                let c = a.creature_counter_cost.as_ref().unwrap();
                return c.paid
                    || game.live_supplemental_counters_available(
                        a.item.controller,
                        amount,
                        &c.selected,
                    );
            }
            if matches!(
                game.state.pending,
                Pending::CounterCostOrder { .. } | Pending::CreatureCounterCost { .. }
            ) {
                let player = game.state.announcing.as_ref().unwrap().item.controller;
                game.state.pending = Pending::PayCosts { player };
            }
            if game.state.entry_batch.is_none()
                && game.state.replacements.is_none()
                && let Some(a) = &game.state.announcing
                && !game.supplemental_counter_upper_allows(a, amount)
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
            game.legal_actions()
                .into_iter()
                .filter(|a| *a != Action::Concede)
                .any(|action| {
                    let a = game.state.announcing.as_ref().unwrap();
                    let c = a.creature_counter_cost.as_ref().unwrap();
                    if matches!(action, Action::PayCosts { .. })
                        && c.after_printed == Some(false)
                        && !c.paid
                    {
                        return counter_declarations(
                            &game,
                            amount,
                            &mut c.selected.clone(),
                            0,
                            &action,
                            seen,
                        );
                    }
                    let mut child = game.clone();
                    child.apply(actor, action).is_ok() && visit(child, amount, seen)
                })
        }
        let mut game = self;
        if let Some(c) = game
            .state
            .announcing
            .as_mut()
            .and_then(|a| a.creature_counter_cost.as_mut())
        {
            c.selected = selected.to_vec();
        }
        visit(game, amount, &mut Vec::new())
    }
}
