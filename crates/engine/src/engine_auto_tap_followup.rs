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
                    || self
                        .state
                        .announcing
                        .as_ref()
                        .is_some_and(|a| a.commits_followup_object(reference, Zone::Hand, false))
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
                    || self.state.announcing.as_ref().is_some_and(|a| {
                        a.commits_followup_object(reference, Zone::Battlefield, definition.cost.tap)
                    })
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

impl Announcement {
    /// Whether the known payment consumes the exact object, or taps a source that must stay untapped.
    /// References such as a Behold witness or the provider of a casualty grant remain available.
    fn commits_followup_object(&self, object: ObjectRef, zone: Zone, needs_untapped: bool) -> bool {
        let same_source = self.item.source.filter(|s| s.object == object);
        if let Some(source) = same_source {
            match self.item.kind {
                StackKind::Spell => return true,
                StackKind::ActivatedAbility { ability }
                    if self.activation_cost_route == ActivationCostRoute::Printed =>
                {
                    let cost = source.card.activation(ability).cost;
                    if cost.sacrifice
                        || cost.exile_self
                        || cost.discard
                        || needs_untapped && cost.tap
                    {
                        return true;
                    }
                }
                _ => {}
            }
        }
        if self.item.attached_activation_exile_cost() == Some(object)
            || self.item.attached_activation_sacrifice_cost() == Some(object)
            || needs_untapped && self.item.attached_activation_tap_cost() == Some(object)
        {
            return true;
        }

        if !self.additional_paid
            && let Some(spec) = self.additional_cost()
        {
            if zone == Zone::Hand && spec.kind == EffectChoiceKind::DiscardHand
                || same_source.is_some()
                    && matches!(spec.kind, EffectChoiceKind::ExileSourceAndNamed { .. })
            {
                return true;
            }
            let selected = self.additional == Some(object)
                || self
                    .additional_group
                    .as_ref()
                    .is_some_and(|group| group.contains(&object));
            if selected && !self.additional_life {
                let kind = spec.in_zone(zone).kind.payment_kind();
                if matches!(kind, EffectChoiceKind::TapPermanents(_)) {
                    if needs_untapped {
                        return true;
                    }
                } else if !kind.chooses_cost_creature()
                    && kind.counter_requirement().is_none()
                    && !matches!(
                        kind,
                        EffectChoiceKind::Behold
                            | EffectChoiceKind::BeholdMany(_)
                            | EffectChoiceKind::BlightCost(_)
                    )
                {
                    return true;
                }
            }
        }
        if self
            .forage
            .as_ref()
            .is_some_and(|f| f.selected.contains(&object))
            || self
                .casualty
                .iter()
                .flat_map(|rows| rows.iter())
                .any(|row| row.pay && !row.paid && row.sacrifice == Some(object))
            || self
                .item
                .craft_materials
                .as_ref()
                .is_some_and(|rows| rows.contains(&object))
        {
            return true;
        }
        if needs_untapped
            && (self.harmonizer.flatten() == Some(object)
                || self
                    .conspire
                    .iter()
                    .flat_map(|rows| rows.iter())
                    .any(|row| {
                        row.pay
                            && !row.paid
                            && (row.first_object == Some(object)
                                || row.objects.is_some_and(|pair| pair.contains(&object)))
                    })
                || self.convoke.as_ref().is_some_and(|f| {
                    f.selected.iter().any(|row| row.object == object)
                        || f.improvised.contains(&object)
                        || f.substituted.contains(&object)
                }))
        {
            return true;
        }
        self.cast_costs.as_ref().is_some_and(|costs| {
            needs_untapped && costs.teammates.contains(&object)
                || costs
                    .origin_forage
                    .as_ref()
                    .is_some_and(|f| f.selected.contains(&object))
                || costs
                    .clauses
                    .iter()
                    .filter(|row| !row.paid)
                    .any(|row| match row.clause {
                        CastCostClause::OriginDiscard(Some(r))
                        | CastCostClause::SneakReturn(Some(r))
                        | CastCostClause::WebSlingingReturn(Some(r))
                        | CastCostClause::BeamMeUpReturn(Some(r), _) => r == object,
                        CastCostClause::CraftSource => same_source.is_some(),
                        CastCostClause::Harmonize(r) => needs_untapped && r == object,
                        _ => false,
                    })
        })
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
