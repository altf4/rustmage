//! Owned recipe and choices for mana whose costs may change its quantity.
use super::*;
use crate::cards::{ChoiceColor, ManaSplit};
use ability_world::CharacteristicWorld;

/// Exact capacity bounds for one color. The other receives the remaining amount.
/// A guided payment additionally requires checked membership in the enclosing
/// payment. The legal subset of a guided range may be noncontiguous.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct ManaSplitChoiceView {
    pub first: ChoiceColor,
    pub second: ChoiceColor,
    pub total: u64,
    pub minimum_first: u32,
    pub maximum_first: u32,
    pub guided: bool,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl ManaSplitChoiceView {
    fn split(self) -> ManaSplit {
        ManaSplit {
            first: self.first,
            second: self.second,
            total: self.total,
        }
    }
    /// Checks the allocation and storage bounds. Use the game membership query
    /// to also check any guided payment.
    pub fn contains_allocation(self, production: Mana) -> bool {
        let amount = production[self.first.index()];
        (self.minimum_first..=self.maximum_first).contains(&amount)
            && self.split().contains(production)
    }
}

/// Try a closed range without materializing its individual choices. A range
/// whose componentwise upper supplies cannot fund the goal can be discarded.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn visit_split_range(
    split: ManaSplit,
    minimum: u32,
    maximum: u32,
    possible: &mut impl FnMut(Mana) -> bool,
    emit: &mut impl FnMut(Mana) -> bool,
) -> bool {
    let mut upper = split.production(minimum);
    upper[split.first.index()] = maximum;
    if !possible(upper) {
        return false;
    }
    let middle = minimum + (maximum - minimum) / 2;
    emit(split.production(middle))
        || minimum < middle && visit_split_range(split, minimum, middle - 1, possible, emit)
        || middle < maximum && visit_split_range(split, middle + 1, maximum, possible, emit)
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ManaOutputChoice {
    Color(u8),
    Allocation(Mana),
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ManaOutput {
    pub recipe: ManaProduction,
    pub choice: Option<ManaOutputChoice>,
    pub instance: mana_announcements::ManaActivationKey,
    pub controller: Player,
    pub quality: Option<mana_lots::Quality>,
    pub source_power: i128,
    /// Only speculative not-yet-payable starts carry this cost.
    pub deferred_cost: Option<ManaCost>,
    pub resume: Option<Box<Pending>>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl ManaOutput {
    pub(super) fn new<W: CharacteristicWorld + ?Sized>(
        world: &W,
        recipe: ManaProduction,
        selected: Mana,
        instance: mana_announcements::ManaActivationKey,
        controller: Player,
        quality: Option<mana_lots::Quality>,
    ) -> Self {
        let recipe = if recipe.base() == ManaProduction::ChosenColor {
            recipe.in_world(world, instance.source.card)
        } else {
            recipe.base()
        };
        let choice = match recipe {
            ManaProduction::Choice { .. }
            | ManaProduction::AnyColorChangingSource(_)
            | ManaProduction::ChoiceStat { .. }
            | ManaProduction::TapSourcePowerChoice { .. }
            | ManaProduction::DistinctCreaturePowers => {
                let mut colors = selected.iter().enumerate().filter(|(_, n)| **n != 0);
                colors.next().and_then(|(color, _)| {
                    colors
                        .next()
                        .is_none()
                        .then_some(ManaOutputChoice::Color(color as u8))
                })
            }
            ManaProduction::Combination { .. }
            | ManaProduction::SourcePowerSplit { .. }
            | ManaProduction::Split(_)
                if selected != [0; 6] =>
            {
                Some(ManaOutputChoice::Allocation(selected))
            }
            _ => None,
        };
        Self {
            recipe,
            choice,
            instance,
            controller,
            quality,
            source_power: world.power(instance.source.card),
            deferred_cost: None,
            resume: None,
        }
    }
    pub(super) fn domain<W: CharacteristicWorld + ?Sized>(
        &self,
        world: &W,
        fallback: i128,
    ) -> ManaProduction {
        let source = self.instance.source;
        let live = world.reference(source.card) == source
            && world.object(source.card).zone
                == self
                    .instance
                    .definition
                    .activation(self.instance.ability)
                    .source_zone();
        match self.recipe {
            ManaProduction::SourcePower { color }
            | ManaProduction::TapSourcePowerChoice { mask: color }
                if !live || world.kinds(source.card) & 1 == 0 =>
            {
                let power = if live {
                    0
                } else {
                    world.departed_source_power(source).unwrap_or(fallback)
                };
                let amount = u32::try_from(power.max(0)).capacity("mana representation exhausted");
                if let ManaProduction::SourcePower { color } = self.recipe {
                    let mut mana = [0; 6];
                    mana[usize::from(color)] = amount;
                    ManaProduction::Fixed(mana)
                } else if amount == 0 {
                    ManaProduction::Fixed([0; 6])
                } else {
                    ManaProduction::Choice {
                        mask: color,
                        amount,
                    }
                }
            }
            ManaProduction::SourcePowerSplit { first, second } if !live => {
                ManaProduction::split_for_power(
                    first,
                    second,
                    world.departed_source_power(source).unwrap_or(fallback),
                )
            }
            recipe => recipe.in_world_for_controller(world, source.card, self.controller),
        }
    }
    pub(super) fn unselected_split<W: CharacteristicWorld + ?Sized>(
        &self,
        world: &W,
        fallback: i128,
    ) -> Option<ManaSplit> {
        if self.choice.is_none()
            && matches!(
                self.recipe,
                ManaProduction::SourcePowerSplit { .. } | ManaProduction::Split(_)
            )
            && let ManaProduction::Split(split) = self.domain(world, fallback)
        {
            Some(split)
        } else {
            None
        }
    }

    pub(super) fn visit<W: CharacteristicWorld + ?Sized>(
        &self,
        world: &W,
        fallback: i128,
        mut emit: impl FnMut(Mana) -> bool,
    ) -> bool {
        let domain = self.domain(world, fallback);
        match self.choice {
            Some(ManaOutputChoice::Color(color)) => match domain {
                ManaProduction::Choice { mask, amount } if mask & (1 << color) != 0 => {
                    let mut mana = [0; 6];
                    mana[usize::from(color)] = amount;
                    emit(mana)
                }
                ManaProduction::Fixed(mana) if mana == [0; 6] => emit(mana),
                _ => false,
            },
            Some(ManaOutputChoice::Allocation(mana)) => {
                if let ManaProduction::Split(split) = domain {
                    split.contains(mana) && emit(mana)
                } else {
                    domain.visit(|m| m == mana && emit(m))
                }
            }
            None => domain.visit(emit),
        }
    }
    pub(super) fn bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self
                .resume
                .as_ref()
                .map_or(0, |p| std::mem::size_of::<Pending>() + p.owned_bytes())
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl ManaProduction {
    pub(super) fn has_late_choice(self) -> bool {
        matches!(
            self.base(),
            Self::Choice { .. }
                | Self::AnyColorChangingSource(_)
                | Self::ChoiceStat { .. }
                | Self::Combination { .. }
                | Self::SourcePowerSplit { .. }
                | Self::Split(_)
                | Self::TapSourcePowerChoice { .. }
                | Self::DistinctCreaturePowers
        )
    }
    /// Planning fixed domains uses concrete outputs once; real costly actions
    /// may additionally retain an unselected choice until those costs finish.
    pub(super) fn visit_at_start_with_fixed_choice<W: CharacteristicWorld + ?Sized>(
        self,
        world: &W,
        source: CardId,
        late_fixed: bool,
        mut emit: impl FnMut(Mana) -> bool,
    ) -> bool {
        if matches!(self.base(), Self::SourcePowerSplit { .. } | Self::Split(_)) {
            return emit([0; 6]);
        }
        let mut zero = false;
        if self.in_world(world, source).visit(|mana| {
            zero |= mana == [0; 6];
            emit(mana)
        }) {
            return true;
        }
        let late_dynamic = matches!(
            self.base(),
            Self::TapSourcePowerChoice { .. }
                | Self::DistinctCreaturePowers
                | Self::ChoiceStat { .. }
        );
        (late_dynamic || (late_fixed && self.has_late_choice())) && !zero && emit([0; 6])
    }
    pub(super) fn accepts_at_start<W: CharacteristicWorld + ?Sized>(
        self,
        world: &W,
        source: CardId,
        selected: Mana,
    ) -> bool {
        self.visit_at_start_with_fixed_choice(world, source, true, |mana| mana == selected)
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    fn mana_output_escaping(&self) -> bool {
        self.foreign_cast_choice_pending()
            && self
                .state
                .cast_rollback
                .as_ref()
                .is_some_and(|r| r.capture.is_some())
    }

    fn mana_split_capacity(&self, player: Player) -> Mana {
        self.state.players[player.index()]
            .mana
            .map(|pool| u32::MAX - pool)
    }

    pub(super) fn require_mana_split_capacity(&self, player: Player, production: ManaProduction) {
        if let ManaProduction::Split(split) = production {
            split
                .bounds(self.mana_split_capacity(player))
                .capacity("mana representation exhausted");
        }
    }

    pub fn mana_split_choice(&self) -> Option<ManaSplitChoiceView> {
        let Pending::EffectMana {
            player,
            production: ManaProduction::Split(split),
            ..
        } = self.state.pending
        else {
            return None;
        };
        let (minimum_first, maximum_first) = split.bounds(self.mana_split_capacity(player))?;
        Some(ManaSplitChoiceView {
            first: split.first,
            second: split.second,
            total: split.total,
            minimum_first,
            maximum_first,
            guided: !self.mana_output_escaping() && self.mana_output_parent_goal().is_some(),
        })
    }

    /// Exact membership for the current split, including any enclosing payment.
    pub fn mana_split_production_legal(&self, production: Mana) -> bool {
        self.mana_split_choice().is_some_and(|choice| {
            choice.contains_allocation(production)
                && (!choice.guided || self.mana_output_choice_fundable(production))
        })
    }

    fn visit_mana_split_choices(
        &self,
        choice: ManaSplitChoiceView,
        mut emit: impl FnMut(Mana) -> bool,
    ) -> bool {
        let split = choice.split();
        let min = choice.minimum_first;
        let max = choice.maximum_first;
        let legal = |mana| !choice.guided || self.mana_output_choice_fundable(mana);
        if max - min < 64 {
            return (min..=max).any(|amount| {
                let mana = split.production(amount);
                legal(mana) && emit(mana)
            });
        }
        let preferred = self.mana_output_parent_goal().map_or(min, |goal| {
            goal.required[split.first.index()]
                .saturating_sub(self.state.players[goal.actor.index()].mana[split.first.index()])
                .clamp(min, max)
        });
        let points = [min, min + (max - min) / 2, max, preferred];
        let mut offered = false;
        for (index, &amount) in points.iter().enumerate() {
            if points[..index].contains(&amount) {
                continue;
            }
            let mana = split.production(amount);
            if legal(mana) {
                offered = true;
                if emit(mana) {
                    return true;
                }
            }
        }
        if offered || !choice.guided {
            return false;
        }
        // Filter-dependent payment routes can make the valid subset disjoint.
        // Find an exact witness when representative points miss all of it.
        let mut accepted = false;
        visit_split_range(
            split,
            min,
            max,
            &mut |upper| self.mana_split_output_may_fund(upper),
            &mut |mana| {
                if !legal(mana) {
                    return false;
                }
                accepted = emit(mana);
                true
            },
        );
        accepted
    }

    /// One domain for the observation and checked raw/index actions. Only an
    /// owned paid activation has an enclosing payment promise to preserve.
    pub(super) fn visit_effect_mana_choices(&self, mut emit: impl FnMut(Mana) -> bool) -> bool {
        let Pending::EffectMana { production, .. } = self.state.pending else {
            return false;
        };
        if matches!(production, ManaProduction::Split(_)) {
            return self
                .mana_split_choice()
                .is_some_and(|choice| self.visit_mana_split_choices(choice, emit));
        }
        let owned = self
            .state
            .announcing
            .as_ref()
            .is_some_and(|a| a.mana_output.is_some());
        // A foreign choice can already have stranded the parent. CR605 requires
        // finishing this captured child before the existing rollback/replay.
        let escaping = self.mana_output_escaping();
        production.visit(|mana| {
            if self.mana_tap_bonus_active() {
                return (escaping || self.tap_bonus_choice_fundable(mana)) && emit(mana);
            }
            (!owned || escaping || self.mana_output_choice_fundable(mana)) && emit(mana)
        })
    }
    pub(super) fn visit_activation_mana_outputs(
        &self,
        production: ManaProduction,
        source: CardId,
        definition: Card,
        index: u16,
        emit: impl FnMut(Mana) -> bool,
    ) -> bool {
        let late_fixed = matches!(
            production.base(),
            ManaProduction::Choice { .. }
                | ManaProduction::Combination { .. }
                | ManaProduction::AnyColorChangingSource(_)
        ) && self.fixed_mana_choice_can_be_late(source, definition, index);
        // Pure tap-only fixed mana has no intervening information choice and
        // retains its original concrete action set. Real costs retain a full
        // late color/allocation route after those costs have been paid.
        production.visit_at_start_with_fixed_choice(self, source, late_fixed, emit)
    }
    pub(super) fn mana_output(&self, item: &SpellView, selected: Mana) -> Option<Box<ManaOutput>> {
        let recipe = self.announced_mana_recipe(item, Some(selected))?;
        let instance = item.mana_activation_key()?;
        if matches!(recipe.base(), ManaProduction::Fixed(_))
            && !matches!(
                instance.ability,
                Card::GRANTED_TAP_MANA | Card::GRANTED_SACRIFICE_MANA
            )
        {
            return None;
        }
        Some(Box::new(ManaOutput::new(
            self,
            recipe,
            selected,
            instance,
            item.controller,
            self.announced_mana_quality(item, Some(selected))
                .map(|(_, quality)| quality),
        )))
    }
    pub(super) fn start_direct_mana_output(
        &mut self,
        item: SpellView,
        counters: Counters,
        payment: Mana,
        production: Mana,
    ) -> bool {
        let Some(mut output) = self.mana_output(&item, production) else {
            return false;
        };
        if self.state.announcing.is_none() {
            output.resume = Some(Box::new(self.state.pending.clone()));
        }
        self.suspend_mana_announcement();
        let mut a = self.payment_announcement(item, counters, Some(production), None);
        let source = a.item.source.expect("activation source").object;
        let total = (0..a.cost_options().count())
            .find_map(|slot| {
                a.select_cost(slot);
                a.untargeted_activation_totals()
                    .into_iter()
                    .flatten()
                    .find(|cost| {
                        let cost = cast_origins::activation_payment_cost(
                            self,
                            a.item.controller,
                            source,
                            *cost,
                        );
                        payment.iter().zip(cost.colored).all(|(p, c)| *p >= c)
                            && payment.into_iter().map(u64::from).sum::<u64>()
                                == cost.colored.into_iter().map(u64::from).sum::<u64>()
                                    + u64::from(cost.generic)
                    })
            })
            .expect("legal direct mana payment has a cost option");
        a.total_cost = Some(total);
        a.payment = Some(payment);
        a.mana_output = Some(output);
        self.state.pending = Pending::PayCosts {
            player: a.item.controller,
        };
        self.state.announcing = Some(a);
        self.finish_announcement();
        true
    }
    pub(super) fn finish_mana_output(&mut self, mut a: Announcement) {
        let output = a.mana_output.as_ref().expect("owned mana output");
        let source = a.item.source.expect("mana source");
        let domain = output.domain(self, source.facts.power);
        if output.choice.is_none()
            && matches!(
                domain,
                ManaProduction::Choice { .. }
                    | ManaProduction::Combination { .. }
                    | ManaProduction::Split(_)
            )
        {
            self.require_mana_split_capacity(a.item.controller, domain);
            self.state.pending = Pending::EffectMana {
                player: a.item.controller,
                production: domain,
                recipients: std::array::from_fn(|i| i == a.item.controller.index()),
            };
            self.state.announcing = Some(a);
            return;
        }
        let mut production = None;
        output.visit(self, source.facts.power, |mana| {
            production = Some(mana);
            true
        });
        let production = production.expect("frozen mana choice remains legal after costs");
        self.complete_mana_output(&mut a, production);
    }
    pub(super) fn complete_mana_output(&mut self, a: &mut Announcement, production: Mana) {
        let output = a.mana_output.take().expect("owned paid mana output");
        self.add_quality_mana(a.item.controller, production, output.quality);
        self.apply_mana_color_change(output.recipe, output.instance.source, production);
        self.finish_mana_tap_bonus(a.item.controller, a.item.tap_mana_bonus(self), production);
        self.committed_mana_followups(&a.item);
        if self.restore_mana_announcement() {
            self.reopen_evidence_after_mana_sacrifice(output.instance.source.card);
            return;
        }
        if let Some(pending) = output.resume {
            self.state.pending = *pending;
        } else if let Some(payment) = &self.state.attack_payment {
            self.state.pending = Pending::AttackPayment {
                player: payment.player,
            };
        } else {
            self.state.pending = Pending::Priority;
        }
    }
}
