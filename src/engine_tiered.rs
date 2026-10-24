//! One immutable modal additional cost; declaration and total lock precede payment.
use super::*;
impl Game {
    pub(super) fn add_tiered_mana(mut base: ManaCost, extra: ManaCost) -> Option<ManaCost> {
        base.generic = base.generic.checked_add(extra.generic)?;
        for (amount, extra) in base.colored.iter_mut().zip(extra.colored) {
            *amount = amount.checked_add(extra)?;
        }
        Some(base)
    }
    pub(super) fn selected_tiered_cost(item: &SpellView) -> Option<ManaCost> {
        if item.kind != StackKind::Spell {
            return None;
        }
        item.mode.and_then(|mode| {
            item.source_card
                .tiered_costs()?
                .get(usize::from(mode))
                .copied()
        })
    }
    pub(super) fn with_tiered_mana(base: ManaCost, item: &SpellView) -> Option<ManaCost> {
        Self::selected_tiered_cost(item)
            .map_or(Some(base), |extra| Self::add_tiered_mana(base, extra))
    }
    pub(super) fn tiered_initial_fundable(
        &self,
        actor: Player,
        id: CardId,
        base: ManaCost,
    ) -> bool {
        let card = self.state.objects[id as usize].card;
        let costs = card.tiered_costs().expect("Tiered recipe");
        let Effect::Modal(modes) = card.effect() else {
            unreachable!("complete Tiered modal body")
        };
        modes.iter().zip(costs).any(|(mode, extra)| {
            self.cast_targets_available(mode.targets, actor, self.object_ref(id))
                && Self::add_tiered_mana(base, *extra)
                    .is_some_and(|cost| self.spell_can_fund(actor, id, cost))
        })
    }
    pub(super) fn mode_choice_fundable(&self, a: &Announcement, index: usize) -> bool {
        let mode = &a.item.modes().expect("modal announcement")[index];
        if !self.teamwork_mode_allowed(&a.item, index)
            || !self.modal_choice_unused(&a.item, index)
            || !self.item_targets_available(&a.item, mode.targets)
        {
            return false;
        }
        if a.item.source_card.tiered_costs().is_none() {
            return self.announcement_target_fundable(
                a,
                Some(mode.targets),
                None,
                None,
                None,
                false,
            );
        }
        // A prepaid compact cast cannot waive an unannounced mandatory tier.
        if a.item.kind != StackKind::Spell || !a.pay_window {
            return false;
        }
        let mut proposal = a.clone();
        proposal.item.mode = Some(u16::try_from(index).expect("mode index"));
        proposal.plan = mode.targets;
        proposal
            .cost_options()
            .any(|cost| self.announcement_cost_fundable(&proposal, cost))
    }
}

impl Game {
    pub(super) fn shuffle_target_into_library(&mut self, item: &SpellView) {
        for bound in &item.targets {
            if !self.valid_library_target(item, *bound) {
                continue;
            }
            let object = bound.object.expect("public targeted object");
            let before = self.object_view(object.card);
            let event = self.move_object(object.card, Zone::Library);
            // CR701.24c: replacement of the move does not cancel the shuffle.
            self.forget_library_top(before.owner);
            self.state
                .rng
                .shuffle(&mut self.state.players[before.owner.index()].library);
            if event.moved && event.destination == Zone::Library {
                self.state.last_target_library_move = Some(Box::new(TargetLibraryMovement {
                    source: item.object,
                    card: before.card,
                    target_before: object,
                    target_after: event.after,
                    owner: before.owner,
                    destination: LibraryDestination::Shuffle,
                }));
            }
        }
    }
    pub(super) fn multiply_target_stats(&mut self, item: &SpellView, factor: u8) {
        let extra = i128::from(factor)
            .checked_sub(1)
            .expect("positive multiplier");
        let affected = item
            .targets
            .iter()
            .filter(|bound| self.valid_library_target(item, **bound))
            .filter_map(|bound| match bound.target {
                Target::Card(id) if self.on_battlefield(id) => Some((id, self.stats(id))),
                _ => None,
            })
            .collect::<Vec<_>>();
        // A resolving multiplier adds a signed snapshot delta in layer7c.
        // Later changes are not multiplied again; base setters retain their layer.
        for (id, stats) in affected {
            self.state.objects[id as usize].growth += Stats {
                power: stats
                    .power
                    .checked_mul(extra)
                    .capacity("power multiplication exhausted"),
                toughness: stats
                    .toughness
                    .checked_mul(extra)
                    .capacity("toughness multiplication exhausted"),
            };
        }
    }
}
