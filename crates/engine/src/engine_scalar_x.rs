//! Explicit scalar declarations and complete interval domains. Work is bounded by
//! immutable threshold breakpoints and at most 33 funding queries per segment, never X.
use super::*;
use crate::cards::CastFilter;

const SCALAR_X_END: u64 = u32::MAX as u64 + 1;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl CastFilter {
    pub(super) fn add_scalar_x_cuts(self, card: Card, cuts: &mut Vec<u64>) {
        let symbols = u64::from(card.x_symbols());
        if symbols == 0 {
            return;
        }
        let cut = |mana_value: u64| {
            mana_value
                .saturating_sub(card.fixed_mana_value())
                .div_ceil(symbols)
                .min(SCALAR_X_END)
        };
        match self {
            Self::ManaValueMinimum { filter, minimum } => {
                cuts.push(cut(minimum));
                filter.add_scalar_x_cuts(card, cuts);
            }
            Self::ManaValueMaximum { filter, maximum } => {
                if let Some(first_excluded) = maximum.checked_add(1) {
                    cuts.push(cut(first_excluded));
                }
                filter.add_scalar_x_cuts(card, cuts);
            }
            Self::AnyOf(filters) => {
                for filter in filters {
                    filter.add_scalar_x_cuts(card, cuts);
                }
            }
            Self::Colors { filter, .. }
            | Self::ManaSymbols { filter, .. }
            | Self::TargetCount { filter, .. }
            | Self::Targeted { filter, .. }
            | Self::CasterOffTurn { filter } => filter.add_scalar_x_cuts(card, cuts),
            _ => {}
        }
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn scalar_x_total(
        &self,
        actor: Player,
        id: CardId,
        base: ManaCost,
        x: u32,
    ) -> Option<ManaCost> {
        let extra = self
            .state
            .announcing
            .as_ref()
            .filter(|a| {
                a.item
                    .source
                    .expect("cost announcement has object source")
                    .id
                    == id
                    && a.item.controller == actor
            })
            .map_or(0, Self::announcement_optional_reduction);
        self.scalar_x_total_with_extra(actor, id, base, x, self.effective_spell_card(id), extra)
    }
    pub(super) fn scalar_x_total_for_definition(
        &self,
        actor: Player,
        id: CardId,
        base: ManaCost,
        x: u32,
        card: Card,
    ) -> Option<ManaCost> {
        self.scalar_x_total_with_extra(actor, id, base, x, card, 0)
    }
    pub(super) fn scalar_x_total_with_extra(
        &self,
        actor: Player,
        id: CardId,
        base: ManaCost,
        x: u32,
        card: Card,
        reduction: u64,
    ) -> Option<ManaCost> {
        let a = self.state.announcing.as_ref().filter(|a| {
            a.item.kind == StackKind::Spell
                && a.item.controller == actor
                && a.item.source.is_some_and(|s| s.id == id && s.card == card)
        });
        self.scalar_x_total_with_symbols(
            actor,
            id,
            base,
            x,
            card,
            a.map_or(u32::from(card.x_symbols()), Announcement::payment_x_symbols),
            a.map_or(&[], |a| a.item.targets.as_slice()),
            reduction,
        )
    }
    #[allow(clippy::too_many_arguments)] // Keep printed value, payment symbols and target discounts independent.
    pub(super) fn scalar_x_total_with_symbols(
        &self,
        actor: Player,
        id: CardId,
        base: ManaCost,
        x: u32,
        card: Card,
        payment_symbols: u32,
        targets: &[BoundTarget],
        reduction: u64,
    ) -> Option<ManaCost> {
        let base = Self::with_additional_target_mana(base, card, targets.len())?;
        let extra = (u64::from(payment_symbols) * u64::from(x))
            .checked_add(self.target_spell_increase(actor, id, card, targets))?;
        let mana_value = card.fixed_mana_value() + u64::from(card.x_symbols()) * u64::from(x);
        let discount = card
            .target_cost_reduction()
            .filter(|r| {
                targets.iter().any(|b| {
                    self.same_target_object(*b)
                        && self.cast_target_matches(
                            r.rule,
                            actor,
                            self.object_ref(id),
                            card,
                            b.target,
                        )
                })
            })
            .map_or(
                ManaCost {
                    colored: [0; 6],
                    generic: 0,
                },
                |r| r.discount,
            );
        self.spell_total_with_reductions(
            actor, id, base, card, mana_value, extra, discount, reduction,
        )
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
        let current = self.state.announcing.as_ref().filter(|a| {
            a.item.kind == StackKind::Spell
                && a.item.controller == actor
                && a.item.source.is_some_and(|s| s.id == id && s.card == card)
        });
        self.scalar_x_intervals_with_announcement(actor, id, card, current)
    }
    pub(super) fn scalar_x_intervals_with_announcement(
        &self,
        actor: Player,
        id: CardId,
        card: Card,
        current: Option<&Announcement>,
    ) -> Vec<XInterval> {
        let symbols = u64::from(card.x_symbols());
        if current.map_or(symbols, |a| u64::from(a.payment_x_symbols())) == 0 {
            return Vec::new();
        }
        let ordinary_library = current.map_or_else(
            || self.state.objects[id as usize].zone == Zone::Library,
            |a| {
                a.item.cast.is_some_and(|cast| {
                    cast.origin == Zone::Library && cast.method != CastMethod::LibraryLife
                }) && a.library_top_cast_permission().is_none()
            },
        ) && self.state.resolving.is_none();
        let mut cuts = vec![0, SCALAR_X_END];
        if ordinary_library {
            cast_origins::add_library_spell_value_cuts(self, actor, card, &mut cuts);
        }
        for rule in card.cost_reductions() {
            rule.filter.add_scalar_x_cuts(card, &mut cuts);
        }
        for &source in &self.state.static_sources {
            let o = &self.state.objects[source as usize];
            if o.controller == actor {
                for rule in o.card.cost_reductions() {
                    rule.filter.add_scalar_x_cuts(card, &mut cuts);
                }
            }
        }
        if symbols > 0 {
            let mut add_minimum = |minimum: u64| {
                cuts.push(
                    minimum
                        .saturating_sub(card.fixed_mana_value())
                        .div_ceil(symbols)
                        .min(SCALAR_X_END),
                )
            };
            if let Some(state) = &self.state.mana_lots {
                for lot in &state.pools[actor.index()] {
                    if let mana_lots::ManaRestrictionView::SpellManaValueAtLeast(minimum) =
                        lot.quality.restriction
                    {
                        add_minimum(minimum);
                    }
                }
            }
            // A threshold also matters before the restricted producer has been tapped.
            for object in &self.state.objects {
                for ability in object.card.activations() {
                    if let Some(ManaProduction::Tagged { quality, .. }) = ability.mana_production()
                        && let crate::cards::ManaRestriction::SpellManaValueAtLeast(minimum) =
                            quality.restriction
                    {
                        add_minimum(minimum);
                    }
                }
            }
        }
        cuts.sort_unstable();
        cuts.dedup();
        let (colored, generic) = card.cost();
        let base = ManaCost { colored, generic };
        let fundable = |x| {
            if let Some(a) = current {
                let mut candidate = a.clone();
                candidate.item.x = Some(x);
                return candidate
                    .cost_options()
                    .any(|cost| self.announcement_cost_fundable(&candidate, cost));
            }
            self.scalar_x_total_for_definition(actor, id, base, x, card)
                .is_some_and(|cost| {
                    self.spell_can_fund_total_at_value(
                        actor,
                        id,
                        cost,
                        None,
                        card,
                        Some(card.fixed_mana_value() + symbols * u64::from(x)),
                    )
                })
        };
        let mut intervals: Vec<XInterval> = Vec::new();
        for ends in cuts.windows(2) {
            let lo = ends[0] as u32;
            let hi = (ends[1] - 1) as u32;
            if ordinary_library
                && !cast_origins::library_spell_value_permission(
                    self,
                    actor,
                    card,
                    card.fixed_mana_value() + symbols * u64::from(lo),
                )
            {
                continue;
            }
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
    pub(super) fn activation_x_intervals(&self, a: &Announcement) -> Vec<XInterval> {
        let StackKind::ActivatedAbility { ability } = a.item.kind else {
            unreachable!("activation X announcement")
        };
        let source = a.item.source.expect("activation source");
        let definition = source.card.activation(ability);
        let minimum = definition.minimum_x();
        let tap_x = definition.tap_x_cost();
        let fundable = |value: u32| {
            let mut probe = a.clone();
            probe.item.x = Some(value);
            if tap_x && !self.item_targets_available(&probe.item, self.target_spec(&probe.item)) {
                return false;
            }
            probe
                .cost_options()
                .any(|cost| self.announcement_cost_fundable(&probe, cost))
        };
        if !fundable(minimum) {
            return Vec::new();
        }
        // Every possible tap contributor corresponds to a distinct object,
        // including cards that could enter during an intervening mana ability.
        let upper = if tap_x {
            u32::try_from(self.state.objects.len()).unwrap_or(u32::MAX)
        } else {
            u32::MAX
        };
        let (mut low, mut high) = (minimum, upper);
        while low < high {
            let mid = low + (high - low).div_ceil(2);
            if fundable(mid) {
                low = mid;
            } else {
                high = mid - 1;
            }
        }
        vec![XInterval {
            minimum,
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
    pub(super) fn visit_x_actions(&self, emit: &mut impl FnMut(Action) -> bool) -> bool {
        self.x_choice().is_some_and(|domain| {
            domain.intervals.into_iter().any(|range| {
                if range.maximum - range.minimum <= 64 {
                    (range.minimum..=range.maximum).any(|value| emit(Action::ChooseX { value }))
                } else {
                    [
                        range.minimum,
                        range.minimum + (range.maximum - range.minimum) / 2,
                        range.maximum,
                    ]
                    .into_iter()
                    .any(|value| emit(Action::ChooseX { value }))
                }
            })
        })
    }

    pub fn x_choice(&self) -> Option<XChoiceView> {
        if matches!(self.state.pending, Pending::FaceUpPayment { .. }) {
            return self.face_up_x_choice();
        }
        if matches!(self.state.pending, Pending::PlotX { .. }) {
            return self.plot_x_choice();
        }
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
                card: self.state.resolving.as_ref()?.item.source?.object,
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
        let intervals = if a.needs_sacrifice_discount_count() {
            self.sacrifice_discount_intervals(a)
        } else if a.blight_x_cost() {
            self.blight_x_intervals(a)
        } else if a.declares_target_value_x() {
            self.target_value_x_intervals(a)
        } else if a.item.kind == StackKind::Spell
            && a.item
                .source
                .expect("cost announcement has object source")
                .card
                .life_x_cost()
        {
            self.life_x_intervals(a)
        } else if a.item.kind == StackKind::Spell {
            self.scalar_x_intervals(
                a.item.controller,
                a.item
                    .source
                    .expect("cost announcement has object source")
                    .id,
            )
        } else {
            self.activation_x_intervals(a)
        };
        Some(XChoiceView {
            card: a
                .item
                .source
                .expect("cost announcement has object source")
                .object,
            intervals,
        })
    }
    pub(super) fn item_amount(&self, value: AmountValue, item: &SpellView) -> u128 {
        match value {
            AmountValue::PaidAdditionalCostSelectionCount => item
                .paid_additional_cost
                .as_ref()
                .map_or(0, |paid| u128::from(paid.selected)),
            AmountValue::PaidAdditionalCostCount => item
                .paid_additional_cost
                .as_ref()
                .map_or(0, |paid| u128::from(paid.count)),
            AmountValue::DiscardEventAmount => item
                .discard_event
                .map_or(0, |event| u128::from(event.amount)),
            AmountValue::ReflexiveSubjectCount => item
                .reflexive_subject
                .as_ref()
                .map_or(0, |row| 1 + row.additional_objects.len() as u128),
            AmountValue::MovedCount | AmountValue::RememberedCount => u128::from(
                self.state
                    .resolving
                    .as_ref()
                    .unwrap()
                    .moved_count
                    .expect("quantity requires a saved resolution count"),
            ),
            AmountValue::TargetPlayerLibraryToGraveyardThisTurn => item
                .targets
                .iter()
                .find_map(|bound| {
                    if let Target::Player(player) = bound.target {
                        Some(u128::from(
                            self.state.players[player.index()].cards_library_to_graveyard_this_turn,
                        ))
                    } else {
                        None
                    }
                })
                .unwrap_or(0),
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
            AmountValue::DelayedExiledCardCount => self.count_delayed_exiled_cards(item) as u128,
            AmountValue::TargetPlayerCreatureCount | AmountValue::TargetPlayersCreatureCount => {
                let mut chosen = 0u8;
                for player in item.targets.iter().filter_map(|bound| match bound.target {
                    Target::Player(player) => Some(player),
                    _ => None,
                }) {
                    chosen |= 1 << player.index();
                    if value == AmountValue::TargetPlayerCreatureCount {
                        break;
                    }
                }
                self.state
                    .objects
                    .iter()
                    .enumerate()
                    .filter(|(id, object)| {
                        object.zone == Zone::Battlefield
                            && chosen & (1 << object.controller.index()) != 0
                            && self.is_creature(*id as CardId)
                    })
                    .count() as u128
            }
            AmountValue::UntapStepCount => item.event_object.as_ref().map_or(0, |event| {
                if let EventObjectFacts::UntapStepCount(amount) = event.facts {
                    u128::from(amount)
                } else {
                    0
                }
            }),
            AmountValue::ManaSpent => u128::from(
                self.item_cast_payment(item)
                    .into_iter()
                    .map(u64::from)
                    .sum::<u64>(),
            ),
            AmountValue::DiscoverEventAmount => {
                item.event_object
                    .as_ref()
                    .map_or(0, |event| match event.facts {
                        EventObjectFacts::DiscoverAmount(amount) => u128::from(amount),
                        _ => 0,
                    })
            }
            AmountValue::DiscoverManaValueShortfall => self.discover_mana_value_shortfall(),
            AmountValue::CaveManaSpent => item
                .cast_event
                .as_ref()
                .map_or(0, |event| u128::from(event.cave_mana_spent)),
            AmountValue::CastSpellManaValue => item
                .cast_event
                .as_ref()
                .map_or(0, |e| u128::from(e.mana_value)),
            AmountValue::OtherSpellsCastThisTurn => {
                let own_cast =
                    !item.is_copy && item.cast.is_some_and(|cast| cast.caster == item.controller);
                u128::from(
                    self.state.players[item.controller.index()]
                        .spells_cast_this_turn
                        .saturating_sub(u64::from(own_cast)),
                )
            }
            AmountValue::NoncreatureSpellsCastByCastingPlayer => {
                item.cast_event.as_ref().map_or(0, |event| {
                    let player = &self.state.players[event.controller.index()];
                    u128::from(
                        player
                            .spells_cast_this_turn
                            .saturating_sub(player.creature_spells_cast_this_turn),
                    )
                })
            }
            AmountValue::CastCardTypeCount => u128::from(
                self.state.players[item.controller.index()]
                    .turn_facts
                    .cast_card_kinds
                    .count_ones(),
            ),
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
            AmountValue::ReflexiveSacrificedPower | AmountValue::SacrificeCostPower => {
                item.sacrificed_power
                    .as_deref()
                    .copied()
                    .unwrap_or(0)
                    .max(0) as u128
            }
            AmountValue::ResolutionSacrificedManaValue => u128::from(
                self.state
                    .resolving
                    .as_ref()
                    .unwrap()
                    .sacrificed_mana_value
                    .expect("paid sacrifice cost during this resolution"),
            ),
            AmountValue::SacrificeCostManaValueWithBonus(bonus) => item
                .sacrificed_mana_value
                .as_deref()
                .map_or(0, |value| u128::from(*value) + u128::from(bonus)),
            AmountValue::OtherZombieEntriesThisTurn => self.other_zombie_entries_this_turn(item),
            AmountValue::EventObjectTotalCounters => {
                item.event_object.as_ref().map_or(0, |event| {
                    if self.on_battlefield(event.object.card)
                        && self.object_ref(event.object.card) == event.object
                    {
                        self.counter_amounts(event.object.card).total()
                    } else if let EventObjectFacts::CounterTotal(total) = event.facts {
                        total
                    } else {
                        0
                    }
                })
            }
            AmountValue::ChosenCostCreaturePower => {
                self.chosen_cost_creature_power(item).max(0) as u128
            }
            AmountValue::TappedCostCreaturePower => {
                let event = item.event_object.as_ref().expect("tapped cost subject");
                let EventObjectFacts::CreatureComparison(saved) = event.facts else {
                    unreachable!()
                };
                let facts = self.comparison_facts(event.object, saved);
                if !facts.creature {
                    return 0;
                }
                let station_with_toughness = |fragment: &crate::cards::StaticAbilityFragment| {
                    matches!(
                        fragment,
                        crate::cards::StaticAbilityFragment::ControlledStationWithGreaterToughness
                    )
                };
                let contribution = if facts.controller == item.controller
                    && facts.toughness > facts.power
                    && self.state.static_sources.iter().copied().any(|source| {
                        let provider = &self.state.objects[source as usize];
                        self.on_battlefield(source)
                            && provider.controller == item.controller
                            && provider
                                .card
                                .static_ability_groups()
                                .iter()
                                .any(|group| group.fragments.iter().any(station_with_toughness))
                            && self.continuous_fragment_available(source, station_with_toughness)
                    }) {
                    facts.toughness
                } else {
                    facts.power
                };
                contribution.max(0) as u128
            }
            AmountValue::TargetCreaturePower => {
                let r = self.state.resolving.as_ref().unwrap();
                r.target_card_facts
                    .as_ref()
                    .and_then(|facts| facts.iter().find(|f| f.group == r.target_group))
                    .map_or(0, |f| f.power.max(0) as u128)
            }
            AmountValue::EnteringCreaturePower => self
                .entering_subject(item)
                .filter(|&id| self.is_creature(id))
                .map_or(0, |id| self.stats(id).power.max(0) as u128),
            AmountValue::SourceLastKnownPower => item
                .source
                .filter(|source| source.facts.is_creature())
                .map_or(0, |source| source.facts.power.max(0) as u128),
            AmountValue::SacrificedToughness => item
                .sacrificed_toughness
                .as_deref()
                .copied()
                .unwrap_or(0)
                .max(0) as u128,
            AmountValue::DiscardedCardManaValue => {
                match item.event_object.as_deref().map(|event| &event.facts) {
                    Some(EventObjectFacts::DiscardedManaValue(amount)) => u128::from(*amount),
                    _ => unreachable!("discard amount requires its frozen event"),
                }
            }
            AmountValue::CounterPlacementAmount => {
                match item.event_object.as_deref().map(|event| &event.facts) {
                    Some(EventObjectFacts::CountersPlaced { amount }) => u128::from(*amount),
                    _ => unreachable!("counter-placement amount requires its frozen event"),
                }
            }
            AmountValue::DepartureCounterCount(kind) => {
                item.event_object.as_ref().map_or(0, |event| {
                    if let EventObjectFacts::DeathCounters(counters) = &event.facts {
                        u128::from(counters.get(kind))
                    } else {
                        0
                    }
                })
            }
            AmountValue::DeathEventPower => item.event_object.as_ref().map_or(0, |event| {
                if let EventObjectFacts::CreatureComparison(facts) = event.facts {
                    if facts.creature {
                        facts.power.max(0) as u128
                    } else {
                        0
                    }
                } else {
                    0
                }
            }),
            AmountValue::SacrificeEventManaValue => {
                match item.event_object.as_deref().map(|event| &event.facts) {
                    Some(EventObjectFacts::DepartureManaValue(value)) => u128::from(*value),
                    _ => 0,
                }
            }
            AmountValue::DeathEventCounterTotal => item.event_object.as_ref().map_or(0, |event| {
                if let EventObjectFacts::DeathCounters(counters) = &event.facts {
                    counters.total()
                } else {
                    0
                }
            }),
            AmountValue::DeathEventToughness => item.event_object.as_ref().map_or(0, |event| {
                if let EventObjectFacts::CreatureComparison(facts) = event.facts {
                    facts.toughness.max(0) as u128
                } else {
                    0
                }
            }),
            AmountValue::DamageReceivedAmount => {
                item.damage_received
                    .as_ref()
                    .expect("damage-received amount requires its frozen packet")
                    .amount
            }
            AmountValue::DamageEventAmount => {
                match item.event_object.as_deref().map(|event| &event.facts) {
                    Some(EventObjectFacts::DamageDealt { amount }) => *amount,
                    _ => item.damage_event.expect("saved damage event").amount,
                }
            }
            AmountValue::CreatureCombatDamageAmount => {
                item.creature_combat_event
                    .as_ref()
                    .expect("equipped combat damage event")
                    .amount
            }
            AmountValue::PaidSourceCounterQuantity(kind) => item
                .source_counter_payment
                .filter(|payment| payment.paid && payment.kind == kind)
                .map_or(0, |payment| u128::from(payment.amount)),
            AmountValue::SourceTotalCounters => CounterKind::ALL
                .into_iter()
                .map(|kind| self.item_amount(AmountValue::SourceCounterQuantity(kind), item))
                .sum(),
            AmountValue::SourceCounterQuantity(kind) => item
                .source
                .and_then(|source| self.source_counter_quantity_live(source.object, kind))
                .map(u128::from)
                .unwrap_or_else(|| {
                    item.source_counter_lki
                        .as_ref()
                        .and_then(|facts| facts.iter().find(|fact| fact.kind == kind))
                        .map_or(0, |fact| u128::from(fact.count))
                }),
            AmountValue::SourceToughness => self.source_toughness(item),
            AmountValue::EquippedCreaturesYouControl => {
                self.equipped_creature_count(item.controller)
            }
            AmountValue::DeclaredAttackerCount => item
                .reflexive_subject
                .as_deref()
                .filter(|subjects| !subjects.action)
                .map_or(0, |subjects| 1 + subjects.additional_objects.len() as u128),
            AmountValue::SourceEquipmentCount => self.source_equipment_quantity(item),
            AmountValue::SourcePower => item.source.map_or(0, |source| {
                if self.on_battlefield(source.id) && self.object_ref(source.id) == source.object {
                    self.stats(source.id).power.max(0) as u128
                } else {
                    item.source_quantity_lki
                        .as_ref()
                        .and_then(|facts| facts.power)
                        .unwrap_or(source.facts.power)
                        .max(0) as u128
                }
            }),
            AmountValue::LifeEventAmount => {
                item.life_event.expect("saved life event").amount.max(0) as u128
            }
            AmountValue::Sum(values) => values.iter().fold(0u128, |total, value| {
                total
                    .checked_add(self.item_amount(*value, item))
                    .capacity("effect sum exhausted")
            }),
            AmountValue::ReflexiveTargetLimit => {
                u128::from(item.reflexive_target_limit.unwrap_or(0))
            }
            AmountValue::ChosenX => u128::from(
                item.paid_additional_cost
                    .as_ref()
                    .and_then(|cost| cost.total_mana_value)
                    .or(item.loyalty_x)
                    .unwrap_or_else(|| u64::from(item.x.unwrap_or(0))),
            ),
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
            AmountValue::Divided { value, divisor } => {
                self.item_amount(*value, item) / u128::from(divisor.get())
            }
            AmountValue::Multiple { value, factor } => self
                .item_amount(*value, item)
                .checked_mul(u128::from(factor))
                .capacity("effect amount representation exhausted"),
            other => item.source.map_or(0, |source| {
                self.amount_value(other, source.object, item.controller)
            }),
        }
    }
}
