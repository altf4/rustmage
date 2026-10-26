//! Explicit scalar declarations and complete interval domains. Work is bounded by
//! immutable threshold breakpoints and at most 33 funding queries per segment, never X.
use super::*;
use crate::cards::CastFilter;
impl Game {
    pub(super) fn scalar_x_total(
        &self,
        actor: Player,
        id: CardId,
        base: ManaCost,
        x: u32,
    ) -> Option<ManaCost> {
        self.scalar_x_total_for_definition(actor, id, base, x, self.effective_spell_card(id))
    }
    pub(super) fn scalar_x_total_for_definition(
        &self,
        actor: Player,
        id: CardId,
        base: ManaCost,
        x: u32,
        card: Card,
    ) -> Option<ManaCost> {
        let mana_value = card.fixed_mana_value() + u64::from(card.x_symbols()) * u64::from(x);
        let raw = u64::from(base.generic) + u64::from(card.x_symbols()) * u64::from(x);
        let discount = self.spell_reduction_for_definition(actor, id, mana_value, card);
        Some(ManaCost {
            colored: base.colored,
            generic: u32::try_from(raw.saturating_sub(discount)).ok()?,
        })
    }
    pub(super) fn scalar_x_intervals(&self, actor: Player, id: CardId) -> Vec<XInterval> {
        self.scalar_x_intervals_for_definition(actor, id, self.effective_spell_card(id))
    }
    pub(super) fn scalar_x_intervals_for_definition(
        &self,
        actor: Player,
        id: CardId,
        card: Card,
    ) -> Vec<XInterval> {
        let symbols = u64::from(card.x_symbols());
        if symbols == 0 {
            return Vec::new();
        }
        let end = u64::from(u32::MAX) + 1;
        let mut cuts = vec![0, end];
        fn thresholds(filter: CastFilter, base: u64, symbols: u64, end: u64, cuts: &mut Vec<u64>) {
            match filter {
                CastFilter::ManaValueMinimum { filter, minimum } => {
                    cuts.push(minimum.saturating_sub(base).div_ceil(symbols).min(end));
                    thresholds(*filter, base, symbols, end, cuts);
                }
                CastFilter::ManaValueMaximum { filter, maximum } => {
                    cuts.push(
                        maximum
                            .saturating_add(1)
                            .saturating_sub(base)
                            .div_ceil(symbols)
                            .min(end),
                    );
                    thresholds(*filter, base, symbols, end, cuts);
                }
                CastFilter::AnyOf(filters) => {
                    for f in filters {
                        thresholds(*f, base, symbols, end, cuts);
                    }
                }
                CastFilter::ManaSymbols { filter, .. } | CastFilter::Targeted { filter, .. } => {
                    thresholds(*filter, base, symbols, end, cuts)
                }
                _ => {}
            }
        }
        for rule in card.cost_reductions() {
            thresholds(
                rule.filter,
                card.fixed_mana_value(),
                symbols,
                end,
                &mut cuts,
            );
        }
        for &source in &self.state.static_sources {
            let o = &self.state.objects[source as usize];
            if o.controller == actor {
                for rule in o.card.cost_reductions() {
                    thresholds(
                        rule.filter,
                        card.fixed_mana_value(),
                        symbols,
                        end,
                        &mut cuts,
                    );
                }
            }
        }
        cuts.sort_unstable();
        cuts.dedup();
        let (colored, generic) = card.cost();
        let base = ManaCost { colored, generic };
        let fundable = |x| {
            self.scalar_x_total_for_definition(actor, id, base, x, card)
                .is_some_and(|cost| {
                    self.spell_can_fund_total_for_definition(actor, id, cost, None, card)
                })
        };
        let mut intervals: Vec<XInterval> = Vec::new();
        for ends in cuts.windows(2) {
            let lo = ends[0] as u32;
            let hi = (ends[1] - 1) as u32;
            if !fundable(lo) {
                continue;
            }
            let (mut left, mut right) = (u64::from(lo), u64::from(hi));
            while left < right {
                let mid = left + (right - left).div_ceil(2);
                if fundable(mid as u32) {
                    left = mid;
                } else {
                    right = mid - 1;
                }
            }
            let range = XInterval {
                minimum: lo,
                maximum: left as u32,
            };
            if let Some(last) = intervals.last_mut()
                && u64::from(last.maximum) + 1 == u64::from(range.minimum)
            {
                last.maximum = range.maximum;
            } else {
                intervals.push(range);
            }
        }
        intervals
    }
    fn activation_x_intervals(&self, a: &Announcement) -> Vec<XInterval> {
        let StackKind::ActivatedAbility { ability } = a.item.kind else {
            return Vec::new();
        };
        let definition = a.item.source_card.activation(ability);
        if a.power_up_costs.is_some() {
            let fundable = |value: u32| {
                let mut probe = a.clone();
                probe.item.x = Some(value);
                probe
                    .cost_options()
                    .any(|cost| self.announcement_cost_fundable(&probe, cost))
            };
            if !fundable(0) {
                return Vec::new();
            }
            let (mut low, mut high) = (0u32, u32::MAX);
            while low < high {
                let mid = low + (high - low).div_ceil(2);
                if fundable(mid) {
                    low = mid;
                } else {
                    high = mid - 1;
                }
            }
            return vec![XInterval {
                minimum: 0,
                maximum: low,
            }];
        }
        let base = a.cost.unwrap_or(ManaCost {
            colored: definition.cost.colored,
            generic: definition
                .cost
                .generic
                .saturating_sub(a.activation_discount),
        });
        let fundable = |value: u32| {
            value
                .checked_mul(definition.x_symbols())
                .and_then(|x| base.generic.checked_add(x))
                .is_some_and(|generic| {
                    self.activation_can_fund(
                        FundingGoal {
                            reserve: None,
                            actor: a.item.controller,
                            required: base.colored,
                            generic,
                            source: Some((
                                a.item.object,
                                ability,
                                a.counter_payment.clone(),
                                a.item.source_card,
                                a.item.borrowed_slot,
                            )),
                        },
                        a.additional_cost(),
                    )
                })
        };
        if !fundable(0) {
            return Vec::new();
        }
        let (mut low, mut high) = (0u32, (u32::MAX - base.generic) / definition.x_symbols());
        while low < high {
            let middle = low + (high - low).div_ceil(2);
            if fundable(middle) {
                low = middle;
            } else {
                high = middle - 1;
            }
        }
        vec![XInterval {
            minimum: 0,
            maximum: low,
        }]
    }
    fn life_x_intervals(&self, a: &Announcement) -> Vec<XInterval> {
        let goal = self.funding_goal(a);
        let purpose = self.purpose_for_item(&a.item);
        let fundable = |x| self.can_fund_life_cost(goal.clone(), purpose, x);
        if !fundable(0) {
            return Vec::new();
        }
        let (mut low, mut high) = (0u64, u64::from(u32::MAX));
        while low < high {
            let mid = low + (high - low).div_ceil(2);
            if fundable(mid as u32) {
                low = mid;
            } else {
                high = mid - 1;
            }
        }
        vec![XInterval {
            minimum: 0,
            maximum: low as u32,
        }]
    }
    pub fn x_choice(&self) -> Option<XChoiceView> {
        if let Pending::ResolutionX { player } = self.state.pending {
            let mut lo = 0u64;
            let resolving = self.state.resolving.as_ref().unwrap();
            let mut hi = match resolving.program.get(resolving.pc.saturating_sub(1)) {
                Some(Instruction::PayXReflexiveLimited { maximum, .. }) => u64::from(*maximum) + 1,
                _ => u64::from(u32::MAX) + 1,
            };
            while lo + 1 < hi {
                let mid = lo + (hi - lo) / 2;
                if self.can_fund(FundingGoal {
                    actor: player,
                    required: [0; 6],
                    generic: mid as u32,
                    reserve: None,
                    source: None,
                }) {
                    lo = mid;
                } else {
                    hi = mid;
                }
            }
            return Some(XChoiceView {
                card: self.state.resolving.as_ref().unwrap().item.object,
                intervals: vec![XInterval {
                    minimum: 0,
                    maximum: lo as u32,
                }],
            });
        }
        if !matches!(self.state.pending, Pending::X { .. }) {
            return None;
        }
        let a = self.state.announcing.as_ref()?;
        Some(XChoiceView {
            card: a.item.object,
            intervals: if a.item.kind == StackKind::Spell && a.item.source_card.life_x_cost() {
                self.life_x_intervals(a)
            } else if a.item.kind == StackKind::Spell {
                self.scalar_x_intervals(a.item.controller, a.item.card)
            } else {
                self.activation_x_intervals(a)
            },
        })
    }
    pub(super) fn item_amount(&self, value: AmountValue, item: &SpellView) -> u128 {
        match value {
            AmountValue::TargetPlayerHandSize => item
                .targets
                .iter()
                .find_map(|bound| {
                    if let Target::Player(player) = bound.target {
                        Some(self.hand_count(player) as u128)
                    } else {
                        None
                    }
                })
                .unwrap_or(0),
            AmountValue::TargetPlayerCreatureCount => item
                .targets
                .iter()
                .find_map(|bound| {
                    if let Target::Player(player) = bound.target {
                        Some(
                            self.state
                                .objects
                                .iter()
                                .enumerate()
                                .filter(|(id, object)| {
                                    object.zone == Zone::Battlefield
                                        && object.controller == player
                                        && self.is_creature(*id as CardId)
                                })
                                .count() as u128,
                        )
                    } else {
                        None
                    }
                })
                .unwrap_or(0),
            AmountValue::ManaSpent => u128::from(
                self.item_cast_payment(item)
                    .into_iter()
                    .map(u64::from)
                    .sum::<u64>(),
            ),
            AmountValue::CastSpellManaValue => item
                .cast_event
                .as_ref()
                .map_or(0, |e| u128::from(e.mana_value)),
            AmountValue::CastSpellManaSymbols(color) => {
                item.cast_event.as_ref().map_or(0, |event| {
                    u128::from(event.card.mana_symbols_of_color(color))
                })
            }
            AmountValue::SpellColors => item
                .cast_event
                .as_ref()
                .map_or(0, |event| event.card.colors().count_ones() as u128),
            AmountValue::ColorsSpent => (0..5)
                .filter(|&i| self.item_cast_payment(item)[i] > 0)
                .count() as u128,
            AmountValue::SacrificedPower => self
                .state
                .resolving
                .as_ref()
                .unwrap()
                .sacrificed_power
                .as_deref()
                .copied()
                .unwrap_or(0)
                .max(0) as u128,
            AmountValue::TargetCreaturePower => {
                let r = self.state.resolving.as_ref().unwrap();
                r.target_card_facts
                    .as_ref()
                    .and_then(|facts| facts.iter().find(|f| f.group == r.target_group))
                    .map_or(0, |f| f.power.max(0) as u128)
            }
            AmountValue::SourceLastKnownPower => {
                if item.source_facts.is_creature() {
                    item.source_facts.power.max(0) as u128
                } else {
                    0
                }
            }
            AmountValue::SacrificedToughness => item
                .sacrificed_toughness
                .as_deref()
                .copied()
                .unwrap_or(0)
                .max(0) as u128,
            AmountValue::DamageReceivedAmount => {
                item.damage_received
                    .as_ref()
                    .expect("damage-received amount requires its frozen packet")
                    .amount
            }
            AmountValue::DamageEventAmount => item.damage_event.expect("saved damage event").amount,
            AmountValue::PaidSourceCounterQuantity(kind) => item
                .source_counter_payment
                .filter(|payment| payment.paid && payment.kind == kind)
                .map_or(0, |payment| u128::from(payment.amount)),
            AmountValue::SourceCounterQuantity(kind) => self
                .source_counter_quantity_live(item.object, kind)
                .map(u128::from)
                .unwrap_or_else(|| {
                    item.source_counter_lki
                        .as_ref()
                        .and_then(|facts| facts.iter().find(|fact| fact.kind == kind))
                        .map_or(0, |fact| u128::from(fact.count))
                }),
            AmountValue::SourceToughness => self.source_toughness(item),
            AmountValue::SourceEquipmentCount => self.source_equipment_quantity(item),
            AmountValue::SourcePower => {
                if self.on_battlefield(item.card) && self.object_ref(item.card) == item.object {
                    self.stats(item.card).power.max(0) as u128
                } else {
                    item.source_facts.power.max(0) as u128
                }
            }
            AmountValue::LifeEventAmount => {
                item.life_event.expect("saved life event").amount.max(0) as u128
            }
            AmountValue::Sum(values) => values.iter().fold(0u128, |total, value| {
                total
                    .checked_add(self.item_amount(*value, item))
                    .capacity("effect sum exhausted")
            }),
            AmountValue::ChosenX => u128::from(item.x.unwrap_or(0)),
            AmountValue::MilledCreatures => self
                .state
                .resolving
                .as_ref()
                .unwrap()
                .milled
                .iter()
                .filter(|reference| {
                    let object = &self.state.objects[reference.card as usize];
                    object.generation == reference.generation
                        && object.zone == Zone::Graveyard
                        && object.card.is_creature()
                })
                .count() as u128,
            AmountValue::Multiple { value, factor } => self
                .item_amount(*value, item)
                .checked_mul(u128::from(factor))
                .capacity("effect amount representation exhausted"),
            other => self.amount_value(other, item.object, item.controller),
        }
    }
}
