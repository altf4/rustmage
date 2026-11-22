//! Resolve an optional caller-named play from information available to its payer.
use super::*;
use auto_tap_solver::Solution;

/// A normal cast from the payer's hand or a printed activation on their battlefield.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum AutoTapFollowUp {
    Cast { object: ObjectRef },
    Activate { object: ObjectRef, ability: u16 },
}

#[derive(Clone, Copy)]
pub(super) struct FollowUp {
    cost: Option<ManaCost>,
    // Sacrificing the named permanent always loses its ability. Tapping also loses a {T} ability.
    permanent: Option<(CardId, bool)>,
}

impl Game {
    pub(super) fn auto_tap_followup(
        &self,
        payer: Player,
        current_source: Option<ObjectRef>,
        options: &AutoTapOptions,
    ) -> Result<FollowUp, Error> {
        let Some(action) = options.follow_up_action else {
            return Ok(FollowUp {
                cost: options.follow_up,
                permanent: None,
            });
        };
        if options.follow_up.is_some() {
            return Err(Error::IllegalAction);
        }
        let reference = match action {
            AutoTapFollowUp::Cast { object } | AutoTapFollowUp::Activate { object, .. } => object,
        };
        let id = reference.card;
        let object = self
            .state
            .objects
            .get(id as usize)
            .ok_or(Error::IllegalAction)?;
        // Check visibility, zone, control and incarnation before inspecting the definition or slot.
        // All invalid selections use the same response, including guesses at a concealed card.
        match action {
            AutoTapFollowUp::Cast { .. } => {
                if object.zone != Zone::Hand
                    || object.owner != payer
                    || self.object_ref(id) != reference
                    || Some(reference) == current_source
                    || self.attempt_draw_is_face_down(reference)
                {
                    return Err(Error::IllegalAction);
                }
                let card = object.card;
                if card.is_land()
                    || card.info().mana_cost.is_empty()
                    || card.x_symbols() != 0
                    || card.has_hybrid_cost()
                    || card.has_target_cost_modifier()
                {
                    return Err(Error::IllegalAction);
                }
                let (colored, generic) = card.cost();
                Ok(FollowUp {
                    cost: Some(ManaCost { colored, generic }),
                    permanent: None,
                })
            }
            AutoTapFollowUp::Activate { ability, .. } => {
                if object.zone != Zone::Battlefield
                    || object.controller != payer
                    || self.object_ref(id) != reference
                    || object.phased_out_under.is_some()
                    || self.is_face_down(reference)
                {
                    return Err(Error::IllegalAction);
                }
                let card = object.card;
                let definition = card
                    .activations()
                    .get(usize::from(ability))
                    .ok_or(Error::IllegalAction)?;
                if definition.origin != ActivationOrigin::Battlefield
                    || card.self_granted_activation_template(ability)
                    || card.grants_activation(ability)
                    || definition.guard.reveals_secret_creature_type()
                    || definition.has_x_cost()
                    || !definition.cost.alternatives.is_empty()
                    || definition.cost.additional.is_some()
                    || card.activation_target_reduction(ability).is_some()
                    || card.activation_target_color_discount(ability) != 0
                    || card.activation_target_counter_discount(ability).is_some()
                    || !self.activation_cost_available(
                        id,
                        definition,
                        usize::from(ability),
                        payer,
                        true,
                    )
                {
                    return Err(Error::IllegalAction);
                }
                let cost = definition.cost;
                Ok(FollowUp {
                    cost: Some(ManaCost {
                        colored: cost.colored,
                        generic: cost.generic,
                    }),
                    permanent: Some((id, cost.tap)),
                })
            }
        }
    }
}

impl TapInputs {
    pub(super) fn solve_with_followup(
        &mut self,
        cost: ManaCost,
        pools: Pools,
        followup: FollowUp,
        bonus: SpendBonus,
    ) -> Option<Solution> {
        let solve = |sources: &[Source], preferred| {
            auto_tap_solver::solve(cost, pools, sources, preferred, &self.followups, bonus)
        };
        let held = followup.permanent.and_then(|(card, needs_tap)| {
            self.actions
                .iter()
                .position(|choices| {
                    matches!(choices.first(),
                Some(Action::TapMana { card: id } | Action::TapGrantedMana { card: id, .. }
                    | Action::ActivateMana { card: id, .. }) if *id == card)
                })
                .filter(|&i| needs_tap || self.sources[i].modes.iter().any(|m| m.sacrifice))
                .map(|i| (i, needs_tap))
        });
        let Some((index, needs_tap)) = held else {
            return solve(&self.sources, followup.cost);
        };

        // Keep action and mode indices stable while excluding taps that lose the follow-up.
        let modes = std::mem::take(&mut self.sources[index].modes);
        if !needs_tap {
            self.sources[index].modes = modes
                .iter()
                .map(|&mode| {
                    if mode.sacrifice {
                        Mode {
                            mana: [0; 6],
                            ..mode
                        }
                    } else {
                        mode
                    }
                })
                .collect();
        }
        let held_plan = solve(&self.sources, followup.cost);
        self.sources[index].modes = modes;
        let payoff = |plan: &Solution| {
            let colors = plan.payment[..5].iter().filter(|&&n| n != 0).count() as u8;
            (colors.min(bonus.distinct_colors), plan.prefer_treasure)
        };
        if held_plan
            .as_ref()
            .is_some_and(|plan| payoff(plan) == (bonus.distinct_colors, bonus.treasure))
        {
            return held_plan;
        }
        // The named play is a preference. An otherwise impossible current payment, or a
        // current printed spending benefit, can still use the named source.
        let ordinary = solve(&self.sources, None);
        if ordinary.as_ref().map(payoff) > held_plan.as_ref().map(payoff) {
            ordinary
        } else {
            held_plan
        }
    }
}
