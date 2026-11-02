//! Shared target predicates, binding, and owned incremental announcements.
use super::*;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Announcement {
    pub miracle_cost: Option<ManaCost>,
    pub item: SpellView,
    pub spec: TargetSpec,
    pub plan: TargetPlan,
    pub group: u16,
    pub trigger: bool,
    pub payment: Option<Mana>,
    pub cost: Option<ManaCost>,
    pub equip_zero_available: bool,
    pub activation_cost_route: ActivationCostRoute,
    pub total_cost: Option<ManaCost>,
    /// Locked before mana abilities or any nonmana payment changes the source.
    pub activation_discount: u32,
    pub power_up_costs: Option<Box<power_up::PowerUpCosts>>,
    pub free_hand_permission: Option<Box<ObjectRef>>,
    /// Mandatory nonmana cost belonging to the selected origin permission.
    pub origin_counter_cost: Option<u16>,
    pub counter_payment: Counters,
    pub pay_window: bool,
    pub production: Option<Mana>,
    pub additional: Option<ObjectRef>,
    /// Explicit chosen life route for a discard-or-life additional cost.
    pub additional_life: bool,
    #[allow(
        clippy::box_collection,
        reason = "Rare growable tap selections use one pointer in ordinary announcements"
    )]
    pub additional_group: Option<Box<Vec<ObjectRef>>>,
    #[allow(
        clippy::box_collection,
        reason = "Rare growable counter selections use one pointer in ordinary announcements"
    )]
    pub creature_counter_cost: Option<Box<Vec<(ObjectRef, CounterKind)>>>,
    pub forage: Option<Box<forage::ForageSelection>>,
    #[allow(
        clippy::box_collection,
        reason = "Rare casualty declarations use one pointer in ordinary announcements"
    )]
    pub casualty: Option<Box<Vec<CasualtyDeclaration>>>,
    pub cast_costs: Option<Box<CastCostTransaction>>,
    pub evidence: Option<Box<evidence::EvidenceSelection>>,
    pub convoke: Option<Box<convoke::ConvokeSelection>>,
    /// None before choice; Some(None) declines the optional tap.
    pub harmonizer: Option<Option<ObjectRef>>,
    pub harmonize_discount: u32,
    pub additional_paid: bool,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Announcement {
    pub(super) fn graveyard_origin_counter_cost(&self) -> Option<u16> {
        self.origin_counter_cost.or_else(|| {
            self.item
                .cast
                .filter(|c| c.method == CastMethod::GraveyardCounterCost)
                .and_then(|_| {
                    self.item
                        .source
                        .expect("announcement has object source")
                        .card
                        .graveyard_counter_cost()
                })
        })
    }
    pub(super) fn fixed_alternative_cost(&self) -> Option<ManaCost> {
        self.item.cast.and_then(|c| match c.method {
            CastMethod::Miracle => self.miracle_cost,
            CastMethod::Mayhem => self
                .item
                .source
                .and_then(|source| source.card.mayhem_cost()),
            CastMethod::Airbend => Some(ManaCost {
                colored: [0; 6],
                generic: 2,
            }),
            CastMethod::Disguise => Some(ManaCost {
                colored: [0; 6],
                generic: 3,
            }),
            CastMethod::ExileLife | CastMethod::Free | CastMethod::Plot => Some(ManaCost {
                colored: [0; 6],
                generic: 0,
            }),
            _ => None,
        })
    }
    pub(super) fn creature_counter_cost_amount(&self) -> Option<u16> {
        self.additional_cost().and_then(|cost| match cost.kind {
            EffectChoiceKind::RemoveCreatureCounters(amount) => Some(amount),
            _ => None,
        })
    }
    pub(super) fn additional_cost(&self) -> Option<AdditionalCost> {
        if self.activation_cost_route == ActivationCostRoute::FirstEquipZero {
            return None;
        }
        if self
            .item
            .cast
            .is_some_and(|c| c.method == CastMethod::Flashback)
            && let Some(spec) = self
                .item
                .source
                .expect("announcement has object source")
                .card
                .flashback_additional()
        {
            return Some(spec);
        }
        if self
            .item
            .cast
            .is_some_and(|c| c.method == CastMethod::Sneak)
        {
            return Some(sneak::COST);
        }
        match self.item.kind {
            StackKind::Spell if self.graveyard_origin_counter_cost().is_some() => Some(
                Game::creature_counter_spec(self.graveyard_origin_counter_cost().unwrap()),
            ),
            StackKind::Spell if self.item.cast.is_some_and(|c| c.evidence == Some(true)) => {
                Some(evidence::cost(
                    self.item
                        .source
                        .expect("announcement has object source")
                        .card
                        .collect_evidence_cost()
                        .unwrap(),
                ))
            }
            StackKind::Spell if self.item.cast.is_some_and(|c| c.bargain == Some(true)) => {
                Some(bargain::COST)
            }
            StackKind::Spell
                if self.item.cast.is_some_and(|c| match c.method {
                    CastMethod::Normal => self.cost.is_some_and(|cost| {
                        self.item
                            .source
                            .expect("object-backed announcement")
                            .card
                            .mana_alternative_replaces_additional(cost)
                    }),
                    CastMethod::ExileLife
                    | CastMethod::Airbend
                    | CastMethod::Free
                    | CastMethod::Plot
                    | CastMethod::Miracle
                    | CastMethod::Mayhem => {
                        self.fixed_alternative_cost()
                            .is_some_and(|base| self.cost.is_some_and(|cost| cost != base))
                            && self
                                .item
                                .source
                                .expect("object-backed announcement")
                                .card
                                .mana_additional_alternative()
                                .is_some()
                    }
                    _ => false,
                }) =>
            {
                None
            }
            StackKind::Spell => self
                .item
                .source
                .expect("announcement has object source")
                .card
                .additional_cost(),
            StackKind::ActivatedAbility { ability } => {
                self.item
                    .source
                    .expect("announcement has object source")
                    .card
                    .activation(ability)
                    .cost
                    .additional
            }
            StackKind::TriggeredAbility { .. }
            | StackKind::CopyEndStepSacrifice
            | StackKind::CopyEndStepConditionalSacrifice
            | StackKind::CasualtyCopy
            | StackKind::DelayedSacrifice
            | StackKind::DelayedReturn
            | StackKind::DelayedTokenCreation { .. }
            | StackKind::DelayedManifestDread
            | StackKind::DelayedWarpExile
            | StackKind::InherentRule { .. }
            | StackKind::SiegeDefeat => None,
        }
    }

    fn cost_route_options(&self) -> impl Iterator<Item = (ManaCost, ActivationCostRoute)> + '_ {
        let options = match self.item.kind {
            StackKind::Spell
                if self
                    .item
                    .cast
                    .is_some_and(|c| c.method == CastMethod::Miracle) =>
            {
                std::iter::once(self.miracle_cost.expect("miracle cost")).chain([].iter().copied())
            }
            StackKind::Spell
                if self
                    .item
                    .cast
                    .is_some_and(|c| c.method == CastMethod::Sneak) =>
            {
                std::iter::once(
                    self.item
                        .source
                        .expect("announcement has object source")
                        .card
                        .sneak_cost()
                        .expect("Sneak cost"),
                )
                .chain([].iter().copied())
            }
            StackKind::Spell
                if self
                    .item
                    .cast
                    .is_some_and(|c| c.method == CastMethod::Disguise) =>
            {
                std::iter::once(ManaCost {
                    colored: [0; 6],
                    generic: 3,
                })
                .chain([].iter().copied())
            }
            StackKind::Spell if self.item.cast.is_some_and(|c| c.method == CastMethod::Warp) => {
                std::iter::once(
                    self.item
                        .source
                        .expect("announcement has object source")
                        .card
                        .warp_cost()
                        .expect("fixed Warp cost"),
                )
                .chain([].iter().copied())
            }
            StackKind::Spell
                if self
                    .item
                    .cast
                    .is_some_and(|cast| cast.method == CastMethod::Mayhem) =>
            {
                std::iter::once(self.fixed_alternative_cost().expect("mayhem cost"))
                    .chain([].iter().copied())
            }
            StackKind::Spell
                if self
                    .item
                    .cast
                    .is_some_and(|cast| cast.method == CastMethod::Harmonize) =>
            {
                std::iter::once(
                    self.item
                        .source
                        .expect("object-backed announcement")
                        .card
                        .harmonize_cost()
                        .expect("harmonize cost"),
                )
                .chain([].iter().copied())
            }
            StackKind::Spell
                if self
                    .item
                    .cast
                    .is_some_and(|cast| cast.method == CastMethod::Flashback) =>
            {
                self.item
                    .source
                    .expect("announcement has object source")
                    .card
                    .flashback_cost_options()
            }
            StackKind::Spell
                if self
                    .item
                    .cast
                    .is_some_and(|c| c.method == CastMethod::Airbend) =>
            {
                std::iter::once(ManaCost {
                    colored: [0; 6],
                    generic: 2,
                })
                .chain([].iter().copied())
            }
            StackKind::Spell
                if self.item.cast.is_some_and(|c| {
                    matches!(
                        c.method,
                        CastMethod::ExileLife | CastMethod::Plot | CastMethod::Free
                    )
                }) =>
            {
                std::iter::once(ManaCost {
                    colored: [0; 6],
                    generic: 0,
                })
                .chain([].iter().copied())
            }
            StackKind::Spell => self
                .item
                .source
                .expect("announcement has object source")
                .card
                .cost_options(),
            StackKind::ActivatedAbility { ability } => self
                .item
                .source
                .expect("object-backed announcement")
                .card
                .activation(ability)
                .cost
                .mana_options(),
            StackKind::TriggeredAbility { .. }
            | StackKind::CopyEndStepSacrifice
            | StackKind::CopyEndStepConditionalSacrifice
            | StackKind::CasualtyCopy
            | StackKind::DelayedSacrifice
            | StackKind::DelayedReturn
            | StackKind::DelayedTokenCreation { .. }
            | StackKind::DelayedManifestDread
            | StackKind::DelayedWarpExile
            | StackKind::InherentRule { .. }
            | StackKind::SiegeDefeat => {
                unreachable!("triggered ability has no activation cost")
            }
        };
        let extra = self.fixed_alternative_cost().and_then(|base| {
            self.item
                .source
                .expect("object-backed announcement")
                .card
                .mana_additional_alternative()
                .map(|extra| ManaCost {
                    colored: std::array::from_fn(|i| {
                        base.colored[i]
                            .checked_add(extra.colored[i])
                            .capacity("additional mana cost exhausted")
                    }),
                    generic: base
                        .generic
                        .checked_add(extra.generic)
                        .capacity("additional mana cost exhausted"),
                })
        });
        // Choose the printed base or its replacement before applying the
        // shared cost modifiers. A route identity survives equal mana totals.
        options
            .map(|cost| (cost, ActivationCostRoute::Printed))
            .chain(self.equip_zero_available.then_some((
                ManaCost {
                    colored: [0; 6],
                    generic: 0,
                },
                ActivationCostRoute::FirstEquipZero,
            )))
            .flat_map(move |(cost, route)| {
                std::iter::once((cost, route)).chain(
                    (self.item.kind == StackKind::Spell && self.fixed_alternative_cost().is_none())
                        .then(|| {
                            self.item
                                .source
                                .expect("announcement has object source")
                                .card
                                .explicit_additional_mana()
                        })
                        .flatten()
                        .map(move |extra| (Card::add_mana_cost(cost, extra), route)),
                )
            })
            .flat_map(move |(cost, route)| {
                const ZERO: [ManaCost; 1] = [ManaCost {
                    colored: [0; 6],
                    generic: 0,
                }];
                self.power_up_costs
                    .as_deref()
                    .map_or(&ZERO[..], |r| r.reductions.as_slice())
                    .iter()
                    .filter_map(move |&reduction| {
                        power_up::reduce(
                            cost,
                            reduction,
                            self.activation_discount,
                            if self.power_up_costs.is_some()
                                && route == ActivationCostRoute::Printed
                            {
                                u64::from(self.item.x.unwrap_or(0))
                                    * u64::from(match self.item.kind {
                                        StackKind::ActivatedAbility { ability } => self
                                            .item
                                            .source
                                            .expect("announcement has object source")
                                            .card
                                            .activation(ability)
                                            .x_symbols(),
                                        _ => 0,
                                    })
                            } else {
                                0
                            },
                        )
                        .map(|cost| (cost, route))
                    })
            })
            .filter(|(cost, _)| {
                self.item
                    .cast
                    .is_none_or(|cast| cast.method != CastMethod::GrantedFlashback)
                    || (self
                        .item
                        .source
                        .expect("announcement has object source")
                        .card
                        .cost_alternative_guard()
                        .is_none()
                        && !self
                            .item
                            .source
                            .expect("object-backed announcement")
                            .card
                            .mana_alternative_replaces_additional(*cost))
                    || *cost
                        == ManaCost {
                            colored: self
                                .item
                                .source
                                .expect("announcement has object source")
                                .card
                                .cost()
                                .0,
                            generic: self
                                .item
                                .source
                                .expect("announcement has object source")
                                .card
                                .cost()
                                .1,
                        }
            })
            .filter_map(|(cost, route)| {
                Game::with_tiered_mana(
                    Game::with_flash_surcharge(
                        cost,
                        self.item
                            .source
                            .expect("announcement has object source")
                            .card,
                        self.item.cast.is_some_and(|c| c.flash_surcharge),
                    ),
                    &self.item,
                )
                .map(|cost| (cost, route))
            })
            .chain(
                extra
                    .and_then(|cost| {
                        Game::with_tiered_mana(
                            Game::with_flash_surcharge(
                                cost,
                                self.item
                                    .source
                                    .expect("announcement has object source")
                                    .card,
                                self.item.cast.is_some_and(|c| c.flash_surcharge),
                            ),
                            &self.item,
                        )
                    })
                    .map(|cost| (cost, ActivationCostRoute::Printed)),
            )
    }
    pub(super) fn cost_options(&self) -> impl Iterator<Item = ManaCost> + '_ {
        self.cost_route_options().map(|(cost, _)| cost)
    }
    pub(super) fn activation_route_for_index(&self, index: usize) -> ActivationCostRoute {
        self.cost_route_options().nth(index).expect("cost option").1
    }
    pub(super) fn needs_evidence(&self) -> bool {
        self.pay_window
            && self.item.kind == StackKind::Spell
            && self
                .item
                .source
                .expect("announcement has object source")
                .card
                .collect_evidence_cost()
                .is_some()
            && self.item.cast.is_some_and(|c| c.evidence.is_none())
    }
    pub(super) fn needs_bargain(&self) -> bool {
        self.pay_window
            && self.item.kind == StackKind::Spell
            && self
                .item
                .source
                .expect("announcement has object source")
                .card
                .has_bargain()
            && self.item.cast.is_some_and(|c| c.bargain.is_none())
    }
    pub(super) fn needs_blight(&self) -> bool {
        self.pay_window
            && self.item.kind == StackKind::Spell
            && self
                .item
                .source
                .expect("announcement has object source")
                .card
                .blight()
                .is_some()
            && self.item.cast.is_some_and(|c| c.blight.is_none())
    }
    pub(super) fn needs_teamwork(&self) -> bool {
        self.pay_window
            && self.item.kind == StackKind::Spell
            && self
                .item
                .source
                .expect("announcement has object source")
                .card
                .teamwork()
                .is_some()
            && self.item.cast.is_some_and(|c| c.teamwork.is_none())
    }
    pub(super) fn needs_kicker(&self) -> bool {
        self.pay_window
            && self.item.kind == StackKind::Spell
            && self
                .item
                .source
                .expect("announcement has object source")
                .card
                .kicker_cost()
                .is_some()
            && self.item.cast.is_some_and(|c| c.kicker.is_none())
    }
    pub(super) fn needs_gift(&self) -> bool {
        self.pay_window
            && self.item.kind == StackKind::Spell
            && self
                .item
                .source
                .expect("announcement has object source")
                .card
                .gift()
                .is_some()
            && self.item.cast.is_some_and(|c| c.gift.is_none())
    }
    pub(super) fn needs_offspring(&self) -> bool {
        self.pay_window
            && self.item.kind == StackKind::Spell
            && self
                .item
                .source
                .expect("announcement has object source")
                .card
                .offspring_cost()
                .is_some()
            && self.item.cast.is_some_and(|c| c.offspring.is_none())
    }
    pub(super) fn needs_mana_cost(&self) -> bool {
        self.pay_window
            && self.cost.is_none()
            && match self.item.kind {
                StackKind::Spell => {
                    self.item
                        .cast
                        .is_none_or(|cast| cast.method != CastMethod::Warp)
                        && self.cost_options().count() > 1
                }
                StackKind::ActivatedAbility { .. } => self.cost_options().count() > 1,
                StackKind::TriggeredAbility { .. }
                | StackKind::CopyEndStepSacrifice
                | StackKind::CopyEndStepConditionalSacrifice
                | StackKind::CasualtyCopy
                | StackKind::DelayedSacrifice
                | StackKind::DelayedReturn
                | StackKind::DelayedTokenCreation { .. }
                | StackKind::DelayedManifestDread
                | StackKind::DelayedWarpExile
                | StackKind::InherentRule { .. }
                | StackKind::SiegeDefeat => false,
            }
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy)]
struct TargetSourceCharacteristics<'a> {
    reference: ObjectRef,
    snapshot: Option<&'a DamageFacts>,
    attachment: Option<ObjectRef>,
    spell: Option<Card>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl<'a> TargetSourceCharacteristics<'a> {
    fn from_item(item: &'a SpellView) -> Self {
        let source = item
            .source
            .as_ref()
            .expect("target source characteristics require an object source");
        Self {
            reference: source.object,
            snapshot: Some(&source.facts),
            attachment: item.source_attachment,
            spell: (item.kind == StackKind::Spell).then_some(source.card),
        }
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn target_spec(&self, item: &SpellView) -> TargetPlan {
        let mut plan = item
            .program()
            .map(|(plan, _)| plan)
            .unwrap_or(TargetPlan::NONE);
        if let Some((TargetPlan::Single(mut spec), program)) = item.program()
            && let Some(Instruction::UpToTargets(value)) = program.first()
        {
            assert_eq!(spec.min, 0, "optional bound requires zero minimum");
            spec.max = usize::try_from(self.item_amount(*value, item)).unwrap_or(usize::MAX);
            plan = TargetPlan::Single(spec);
        }
        if item.kind == StackKind::Spell
            && item
                .source
                .expect("spell has source")
                .card
                .targets_up_to_x()
        {
            let TargetPlan::Single(mut spec) = plan else {
                panic!("X target maximum requires one homogeneous target group")
            };
            assert_eq!(spec.min, 0);
            spec.max = usize::try_from(item.x.unwrap_or(0)).unwrap_or(usize::MAX);
            if item.division_total().is_some_and(|total| total > 0) {
                spec.min = 1;
            }
            return TargetPlan::Single(spec);
        }
        if let TargetPlan::Single(mut spec) = plan
            && let TargetRule::ManaValueExactly { base, amount, .. } = spec.rule
        {
            spec.rule = TargetRule::ManaValueExactly {
                base,
                amount,
                value: Some(u64::try_from(self.item_amount(amount, item)).unwrap_or(u64::MAX)),
            };
            return TargetPlan::Single(spec);
        }
        if let TargetPlan::Single(mut spec) = plan
            && let TargetRule::ManaValueAtMost { base, amount, .. } = spec.rule
        {
            spec.rule = TargetRule::ManaValueAtMost {
                base,
                amount,
                maximum: Some(u64::try_from(self.item_amount(amount, item)).unwrap_or(u64::MAX)),
            };
            return TargetPlan::Single(spec);
        }
        if let TargetPlan::Single(mut spec) = plan
            && matches!(spec.rule, TargetRule::GraveCreatureSourcePower { .. })
        {
            let Some(source) = item.source else {
                return plan;
            };
            let power =
                if self.on_battlefield(source.id) && self.object_ref(source.id) == source.object {
                    self.stats(source.id).power
                } else {
                    source.facts.power
                };
            spec.rule = TargetRule::GraveCreatureSourcePower {
                maximum: Some(power),
            };
            TargetPlan::Single(spec)
        } else {
            plan
        }
    }

    // A spell will leave its origin before targets are chosen (CR601.2a/c).
    // In particular, it cannot target its own soon-to-depart graveyard card.
    pub(super) fn cast_targets_available(
        &self,
        plan: TargetPlan,
        actor: Player,
        source: ObjectRef,
    ) -> bool {
        self.cast_definition_targets_available(
            plan,
            actor,
            source,
            self.state.objects[source.card as usize].card,
        )
    }
    pub(super) fn cast_definition_targets_available(
        &self,
        plan: TargetPlan,
        actor: Player,
        source: ObjectRef,
        definition: Card,
    ) -> bool {
        let qualification = Some(TargetSourceCharacteristics {
            reference: source,
            snapshot: None,
            attachment: None,
            spell: Some(definition),
        });
        let departing =
            (self.state.objects[source.card as usize].zone == Zone::Hand).then_some(source.card);
        let available = |spec: TargetSpec| {
            if spec.min == 0 {
                return true;
            }
            let mut count = [0usize; 2];
            let mut seen_kinds = 0u16;
            self.visit_targets_without_hand(
                spec.rule,
                actor,
                source,
                departing,
                qualification,
                |target| {
                    if target == Target::Card(source.card) {
                        return false;
                    }
                    if matches!(
                        spec.rule,
                        TargetRule::Grouped {
                            constraint: TargetGroupConstraint::SharedCardType,
                            ..
                        }
                    ) {
                        let Target::Card(id) = target else {
                            return false;
                        };
                        let kinds = self.current_kinds(id);
                        let available = seen_kinds & kinds != 0;
                        seen_kinds |= kinds;
                        return available;
                    }
                    let distinct = matches!(
                        spec.rule,
                        TargetRule::Grouped {
                            constraint: TargetGroupConstraint::DifferentControllers,
                            ..
                        }
                    );
                    let bucket = if matches!(spec.rule, TargetRule::Grouped { .. }) {
                        let Target::Card(id) = target else {
                            return false;
                        };
                        let object = &self.state.objects[id as usize];
                        if distinct {
                            object.controller.index()
                        } else {
                            object.owner.index()
                        }
                    } else {
                        0
                    };
                    count[bucket] += 1;
                    if distinct {
                        spec.min <= 1 || spec.min <= 2 && count[0] > 0 && count[1] > 0
                    } else {
                        count[bucket] >= usize::from(spec.min)
                    }
                },
            )
        };
        match plan {
            TargetPlan::Single(spec) => available(spec),
            TargetPlan::Groups(groups) => groups.iter().copied().all(available),
        }
    }

    pub(super) fn targets_available(
        &self,
        plan: TargetPlan,
        actor: Player,
        source: ObjectRef,
    ) -> bool {
        self.targets_available_with(plan, actor, source, None)
    }
    pub(super) fn item_targets_available(&self, item: &SpellView, plan: TargetPlan) -> bool {
        if item.source.is_none() {
            // The only supported source-less rule has no targets. Future rule
            // abilities with targets need a source-optional target query.
            return plan == TargetPlan::NONE;
        }
        self.targets_available_with(
            plan,
            item.controller,
            item.target_source(),
            Some(TargetSourceCharacteristics::from_item(item)),
        )
    }
    fn targets_available_with(
        &self,
        plan: TargetPlan,
        actor: Player,
        source: ObjectRef,
        qualification: Option<TargetSourceCharacteristics<'_>>,
    ) -> bool {
        let available = |spec: TargetSpec| {
            if spec.min == 0 {
                return true;
            }
            let mut count = [0usize; 2];
            let mut seen_kinds = 0u16;
            self.visit_targets_without_hand(
                spec.rule,
                actor,
                source,
                None,
                qualification,
                |target| {
                    if matches!(
                        spec.rule,
                        TargetRule::Grouped {
                            constraint: TargetGroupConstraint::SharedCardType,
                            ..
                        }
                    ) {
                        let Target::Card(id) = target else {
                            return false;
                        };
                        let kinds = self.current_kinds(id);
                        let available = seen_kinds & kinds != 0;
                        seen_kinds |= kinds;
                        return available;
                    }
                    let distinct = matches!(
                        spec.rule,
                        TargetRule::Grouped {
                            constraint: TargetGroupConstraint::DifferentControllers,
                            ..
                        }
                    );
                    let bucket = if matches!(spec.rule, TargetRule::Grouped { .. }) {
                        let Target::Card(id) = target else {
                            return false;
                        };
                        let object = &self.state.objects[id as usize];
                        if distinct {
                            object.controller.index()
                        } else {
                            object.owner.index()
                        }
                    } else {
                        0
                    };
                    count[bucket] += 1;
                    if distinct {
                        spec.min <= 1 || spec.min <= 2 && count[0] > 0 && count[1] > 0
                    } else {
                        count[bucket] >= usize::from(spec.min)
                    }
                },
            )
        };
        match plan {
            TargetPlan::Single(spec) => available(spec),
            TargetPlan::Groups(groups) => groups.iter().copied().all(available),
        }
    }

    /// Graveyard identity is its owner's player, never the battlefield controller.
    /// Bindings already own exact incarnations; this relationship adds no state.
    pub(super) fn target_group_compatible(
        &self,
        rule: TargetRule,
        group: u16,
        chosen: &[BoundTarget],
        target: Target,
    ) -> bool {
        let TargetRule::Grouped { constraint, .. } = rule else {
            return true;
        };
        if let TargetGroupConstraint::TotalManaValueAtMost(maximum) = constraint {
            let mut total = 0u128;
            for bound in chosen.iter().filter(|b| b.group == group) {
                let Target::Card(id) = bound.target else {
                    return false;
                };
                total += u128::from(self.object_mana_value(id));
            }
            if !chosen
                .iter()
                .any(|b| b.group == group && b.target == target)
            {
                let Target::Card(id) = target else {
                    return false;
                };
                total += u128::from(self.object_mana_value(id));
            }
            return total <= u128::from(maximum);
        }
        if let TargetGroupConstraint::DistinctCardTypeSlots = constraint {
            let Target::Card(id) = target else {
                return false;
            };
            if self
                .state
                .objects
                .get(id as usize)
                .is_none_or(|o| o.zone != Zone::Graveyard)
            {
                return false;
            }
            let mut masks = Vec::new();
            for bound in chosen.iter().filter(|b| b.group == group) {
                let Target::Card(other) = bound.target else {
                    return false;
                };
                if other == id
                    || !self.same_target_object(*bound)
                    || self.state.objects[other as usize].zone != Zone::Graveyard
                {
                    return false;
                }
                masks.push(self.current_kinds(other));
            }
            masks.push(self.current_kinds(id));
            return card_type_assignment::distinct_type_assignment(&masks);
        }
        if let TargetGroupConstraint::OtherThanGroup(first) = constraint {
            return chosen
                .iter()
                .all(|bound| bound.group != first || bound.target != target);
        }
        let Target::Card(id) = target else {
            return false;
        };
        let Some(object) = self.state.objects.get(id as usize) else {
            return false;
        };
        match constraint {
            TargetGroupConstraint::OtherThanGroup(_) | TargetGroupConstraint::TotalManaValueAtMost(_)
                | TargetGroupConstraint::DistinctCardTypeSlots => unreachable!(),
            TargetGroupConstraint::ControllerOfPlayerGroup(first_group) => object.zone == Zone::Battlefield
                && chosen.iter().find(|b| b.group == first_group).is_some_and(|b| matches!(b.target, Target::Player(player) if object.controller == player)),
            TargetGroupConstraint::AttachedToGroup(first_group) => object.zone == Zone::Battlefield
                && chosen.iter().find(|b| b.group == first_group).is_some_and(|b| {
                    self.same_target_object(*b) && object.attached_to == b.object
                }),
            TargetGroupConstraint::SameGraveyard => object.zone == Zone::Graveyard && chosen.iter().filter(|b| b.group == group).all(|b| {
                matches!(b.target, Target::Card(other) if self.state.objects.get(other as usize).is_some_and(|o| o.owner == object.owner))
            }),
            TargetGroupConstraint::SharedCardType => object.zone == Zone::Battlefield && chosen.iter().filter(|b| b.group == group).all(|b| {
                matches!(b.target, Target::Card(other) if self.same_target_object(*b) && self.on_battlefield(other) && self.current_kinds(other) & self.current_kinds(id) != 0)
            }),
            TargetGroupConstraint::DifferentControllers => object.zone == Zone::Battlefield && chosen.iter().filter(|b| b.group == group).all(|b| {
                matches!(b.target, Target::Card(other) if self.state.objects.get(other as usize).is_some_and(|o| o.controller != object.controller))
            }),
        }
    }

    pub(super) fn shared_type_target_completion(&self, a: &Announcement, target: Target) -> bool {
        if !matches!(
            a.spec.rule,
            TargetRule::Grouped {
                constraint: TargetGroupConstraint::SharedCardType,
                ..
            }
        ) || a.item.targets.iter().any(|b| b.group == a.group)
        {
            return true;
        }
        let Target::Card(id) = target else {
            return false;
        };
        self.visit_item_targets(&a.item, a.spec.rule, |other| {
            matches!(other, Target::Card(other_id) if other_id != id && self.current_kinds(other_id) & self.current_kinds(id) != 0)
        })
    }

    pub(super) fn resolution_target_relations_hold(
        &self,
        item: &SpellView,
        bound: BoundTarget,
        plan: TargetPlan,
    ) -> bool {
        if plan.group(bound.group).is_some_and(|g| {
            matches!(
                g.rule,
                TargetRule::Grouped {
                    constraint: TargetGroupConstraint::DistinctCardTypeSlots,
                    ..
                }
            )
        }) {
            let Some(index) = item.targets.iter().position(|target| *target == bound) else {
                return false;
            };
            let Some(slot) = item
                .target_type_slots
                .as_ref()
                .and_then(|slots| slots.get(index))
            else {
                return false;
            };
            return matches!(bound.target, Target::Card(id)
                if self.same_target_object(bound) && self.current_kinds(id) & *slot != 0);
        }
        if plan.group(bound.group).is_some_and(|g| {
            matches!(
                g.rule,
                TargetRule::Grouped {
                    constraint: TargetGroupConstraint::SharedCardType,
                    ..
                }
            )
        }) {
            let Target::Card(id) = bound.target else {
                return false;
            };
            let kinds = self.current_kinds(id);
            return item
                .targets
                .iter()
                .filter(|other| other.group == bound.group && other.target != bound.target)
                .all(|other| {
                    let Target::Card(other_id) = other.target else {
                        return false;
                    };
                    let other_kinds =
                        if self.same_target_object(*other) && self.on_battlefield(other_id) {
                            Some(self.current_kinds(other_id))
                        } else {
                            item.target_kinds_lki.as_ref().and_then(|rows| {
                                rows.iter().find_map(|(reference, kinds)| {
                                    (Some(*reference) == other.object).then_some(*kinds)
                                })
                            })
                        };
                    other_kinds.is_some_and(|other| kinds & other != 0)
                });
        }
        if let Some(TargetSpec {
            rule:
                TargetRule::Grouped {
                    constraint: TargetGroupConstraint::ControllerOfPlayerGroup(_),
                    ..
                },
            ..
        }) = plan.group(bound.group)
        {
            return self.target_group_compatible(
                plan.group(bound.group).unwrap().rule,
                bound.group,
                &item.targets,
                bound.target,
            );
        }
        if let Some(TargetSpec {
            rule:
                TargetRule::Grouped {
                    constraint: TargetGroupConstraint::AttachedToGroup(first_group),
                    ..
                },
            ..
        }) = plan.group(bound.group)
        {
            let Target::Card(id) = bound.target else {
                return false;
            };
            return self.same_target_object(bound)
                && self.on_battlefield(id)
                && item
                    .targets
                    .iter()
                    .find(|b| b.group == first_group)
                    .is_some_and(|b| {
                        self.same_target_object(*b)
                            && self.state.objects[id as usize].attached_to == b.object
                    });
        }
        if !plan.group(bound.group).is_some_and(|g| {
            matches!(
                g.rule,
                TargetRule::Grouped {
                    constraint: TargetGroupConstraint::DifferentControllers,
                    ..
                }
            )
        }) {
            return true;
        }
        let Target::Card(id) = bound.target else {
            return false;
        };
        let controller = self.state.objects[id as usize].controller;
        item.targets
            .iter()
            .filter(|other| other.group == bound.group && other.target != bound.target)
            .all(|other| {
                let Target::Card(other_id) = other.target else {
                    return false;
                };
                let other_controller =
                    if self.same_target_object(*other) && self.on_battlefield(other_id) {
                        Some(self.state.objects[other_id as usize].controller)
                    } else {
                        item.target_controller_lki.as_ref().and_then(|history| {
                            history.iter().find_map(|(reference, player)| {
                                (Some(*reference) == other.object).then_some(*player)
                            })
                        })
                    };
                other_controller.is_some_and(|other| other != controller)
            })
    }

    pub(super) fn retain_resolution_target_budgets(&self, item: &mut SpellView, plan: TargetPlan) {
        let failed: Vec<_> = item
            .targets
            .iter()
            .filter_map(|bound| {
                let TargetRule::Grouped {
                    constraint: TargetGroupConstraint::TotalManaValueAtMost(maximum),
                    ..
                } = plan.group(bound.group)?.rule
                else {
                    return None;
                };
                let total: u128 = item
                    .targets
                    .iter()
                    .filter(|b| b.group == bound.group)
                    .map(|b| match b.target {
                        Target::Card(id) => u128::from(self.object_mana_value(id)),
                        _ => u128::MAX,
                    })
                    .fold(0u128, u128::saturating_add);
                (total > u128::from(maximum)).then_some(bound.group)
            })
            .collect();
        if !failed.is_empty() {
            item.targets.retain(|b| !failed.contains(&b.group));
        }
    }
    pub(super) fn choose_mana_cost(&mut self, index: u16) {
        let a = self.state.announcing.as_mut().unwrap();
        let cost = a.cost_options().nth(index as usize).unwrap();
        a.activation_cost_route = a.activation_route_for_index(index as usize);
        a.cost = Some(cost);
        self.continue_announcement_choices(false);
    }

    pub(super) fn choose_mode(&mut self, mode: u16) {
        if self
            .state
            .announcing
            .as_ref()
            .unwrap()
            .item
            .bounded_mode_count(usize::from(mode))
            == Some(0)
        {
            // CR700.2b/603.3c: choosing no modes removes the triggered ability.
            assert!(self.state.announcing.take().unwrap().trigger);
            self.continue_trigger_order();
            return;
        }
        let plan = self
            .state
            .announcing
            .as_ref()
            .unwrap()
            .item
            .modes()
            .unwrap()[usize::from(mode)]
        .targets;
        self.declare_mode(mode, plan);
    }

    pub(super) fn declare_mode(&mut self, mode: u16, plan: TargetPlan) {
        let item = &self.state.announcing.as_ref().unwrap().item;
        if let Some(key) = modal_history::key(item) {
            self.record_modal_choice(key, mode);
        }
        let announcement = self.state.announcing.as_mut().unwrap();
        announcement.item.mode = Some(mode);
        announcement.plan = plan;
        announcement.group = 0;
        self.continue_announcement_choices(false);
    }

    pub(super) fn continue_announcement_choices(&mut self, needs_mode: bool) {
        let a = self.state.announcing.as_ref().unwrap();
        let player = a.item.controller;
        // CR601.2b announces the mode before a hybrid equivalent. Both
        // declarations precede targets, mana abilities and payment.
        if a.needs_blight() {
            self.state.pending = Pending::BlightIntent { player };
        } else if a.needs_teamwork() {
            self.state.pending = Pending::TeamworkIntent { player };
        } else if needs_mode || a.item.modes().is_some() && a.item.mode.is_none() {
            self.state.pending = Pending::Modes { player };
        } else if a.pay_window
            && a.item.source_counter_cost().is_some()
            && a.item.source_counter_payment.is_none()
        {
            self.state.pending = Pending::SourceCounterAmount { player };
        } else if a.pay_window && a.has_x_cost() && a.item.x.is_none() {
            self.state.pending = Pending::X { player };
        } else if a.needs_offspring() {
            self.state.pending = Pending::Offspring { player };
        } else if a.needs_evidence() {
            self.state.pending = Pending::EvidenceIntent { player };
        } else if a.needs_bargain() {
            self.state.pending = Pending::Bargain { player };
        } else if a.needs_kicker() {
            self.state.pending = Pending::Kicker { player };
        } else if a.needs_gift() {
            self.state.pending = Pending::Gift { player };
        } else if a.needs_mana_cost() {
            self.state.pending = if a.item.kind == StackKind::Spell {
                Pending::SpellCost { player }
            } else {
                Pending::ActivationCost { player }
            };
        } else if a.additional_cost().is_some_and(forage::is_forage_cost) && a.forage.is_none() {
            self.state.pending = Pending::ForageMethod { player };
        } else if a
            .casualty
            .as_ref()
            .is_some_and(|rows| rows.iter().any(|row| !row.declared))
        {
            self.state.pending = Pending::Casualty { player };
        } else if a.pay_window
            && a.item
                .cast
                .is_some_and(|c| c.method == CastMethod::Harmonize)
            && a.harmonizer.is_none()
        {
            self.state.pending = Pending::Harmonize { player };
        } else {
            let improvise = improvise::potential(
                self,
                self.state
                    .announcing
                    .as_ref()
                    .unwrap()
                    .item
                    .source
                    .expect("announcement has object source")
                    .card,
            );
            self.state
                .announcing
                .as_mut()
                .unwrap()
                .initialize_cast_costs(improvise);
            self.state.pending = Pending::Targets { player };
            self.advance_target_groups();
        }
    }

    pub(super) fn finish_target_group(&mut self) {
        self.state.announcing.as_mut().unwrap().group += 1;
        self.advance_target_groups();
    }

    fn advance_target_groups(&mut self) {
        loop {
            let announcement = self.state.announcing.as_ref().unwrap();
            let Some(spec) = announcement.plan.group(announcement.group) else {
                self.finish_announcement();
                return;
            };
            let item = &announcement.item;
            if spec.max > 0 && self.visit_item_targets(item, spec.rule, |_| true) {
                let player = item.controller;
                self.state.announcing.as_mut().unwrap().spec = spec;
                self.state.pending = Pending::Targets { player };
                return;
            }
            assert_eq!(spec.min, 0, "validated group completion");
            self.state.announcing.as_mut().unwrap().group += 1;
        }
    }

    pub(super) fn bind_target(&self, target: Target) -> BoundTarget {
        BoundTarget {
            group: 0,
            target,
            object: match target {
                Target::Stack(_) | Target::Player(_) => None,
                Target::Card(id) | Target::Spell(id) => Some(self.object_ref(id)),
            },
        }
    }

    /// Only qualified recipients inspect source facts; ordinary targets keep their fast path.
    fn qualified_hexproof_blocks(
        &self,
        id: CardId,
        controller: Player,
        source: ObjectRef,
        qualification: Option<TargetSourceCharacteristics<'_>>,
    ) -> bool {
        let object = &self.state.objects[id as usize];
        if object.zone != Zone::Battlefield || object.controller == controller {
            return false;
        }
        let filters = || {
            object
                .card
                .static_ability_groups()
                .iter()
                .flat_map(|g| g.fragments)
                .filter_map(|f| {
                    if let crate::cards::StaticAbilityFragment::QualifiedHexproof(filter) = f {
                        Some(*filter)
                    } else {
                        None
                    }
                })
        };
        if filters().next().is_none() {
            return false;
        }
        let packets = || {
            self.state
                .stack
                .iter()
                .chain(&self.state.pending_triggers)
                .chain(&self.state.ordering_triggers)
                .chain(self.state.announcing.iter().map(|a| &a.item))
                .chain(self.state.resolving.iter().map(|r| &r.item))
        };
        let qualification = qualification.or_else(|| {
            packets()
                .find(|item| {
                    item.source
                        .is_some_and(|identity| identity.object == source)
                })
                .map(TargetSourceCharacteristics::from_item)
        });
        let source = qualification.map_or(source, |q| q.reference);
        let current = self
            .state
            .objects
            .get(source.card as usize)
            .filter(|o| o.generation == source.generation);
        let facts = if let Some(definition) = qualification.and_then(|q| q.spell) {
            // A spell uses its chosen face even in preflight, before it reaches the stack.
            Some((definition.colors(), definition.kind_mask()))
        } else if let Some(current) = current {
            Some((
                ability_world::CharacteristicWorld::colors(self, source.card),
                self.object_kinds(source.card, current),
            ))
        } else {
            qualification
                .and_then(|q| q.snapshot)
                .map(|snapshot| (snapshot.colors(), snapshot.kinds))
        };
        let Some((colors, kinds)) = facts else {
            return false;
        };
        let query = self.characteristic_query();
        filters().any(|filter| {
            let matches = match filter {
                crate::cards::SourceCharacteristicFilter::Color(color) => {
                    colors & (1u8 << color.index()) != 0
                }
                crate::cards::SourceCharacteristicFilter::Instant => Card::kinds_are_instant(kinds),
            };
            matches
                && query.continuous_fragment_available(id, |fragment| {
                    matches!(fragment,
                crate::cards::StaticAbilityFragment::QualifiedHexproof(found) if *found == filter)
                })
        })
    }

    pub(super) fn item_target_matches(
        &self,
        item: &SpellView,
        rule: TargetRule,
        target: Target,
    ) -> bool {
        self.target_matches_without_hand(
            rule,
            item.controller,
            item.target_source(),
            target,
            None,
            Some(TargetSourceCharacteristics::from_item(item)),
        )
    }

    pub(super) fn cast_target_matches(
        &self,
        rule: TargetRule,
        controller: Player,
        source: ObjectRef,
        target: Target,
    ) -> bool {
        let departing = self
            .state
            .objects
            .get(source.card as usize)
            .filter(|o| o.zone == Zone::Hand && o.generation == source.generation)
            .map(|_| source.card);
        self.target_matches_without_hand(rule, controller, source, target, departing, None)
    }
    fn instant_sorcery_source(
        &self,
        source: ObjectRef,
        qualification: Option<TargetSourceCharacteristics<'_>>,
    ) -> bool {
        let qualification = qualification.or_else(|| {
            self.state
                .stack
                .iter()
                .chain(self.state.pending_triggers.iter())
                .chain(self.state.ordering_triggers.iter())
                .chain(self.state.announcing.iter().map(|a| &a.item))
                .chain(self.state.resolving.iter().map(|r| &r.item))
                .find(|item| {
                    item.source
                        .is_some_and(|identity| identity.object == source)
                })
                .map(TargetSourceCharacteristics::from_item)
        });
        let kinds = if let Some(card) = qualification.and_then(|q| q.spell) {
            card.kind_mask()
        } else if let Some(object) = self
            .state
            .objects
            .get(source.card as usize)
            .filter(|o| o.generation == source.generation)
        {
            self.object_kinds(source.card, object)
        } else {
            qualification
                .and_then(|q| q.snapshot)
                .map_or(0, |f| f.kinds)
        };
        kinds & (4 | 8) != 0
    }
    fn target_matches_without_hand(
        &self,
        rule: TargetRule,
        controller: Player,
        source: ObjectRef,
        target: Target,
        departing: Option<CardId>,
        qualification: Option<TargetSourceCharacteristics<'_>>,
    ) -> bool {
        match rule {
            TargetRule::ManaValueExactly { base, value, .. } => {
                return value.is_some_and(|value| matches!(target, Target::Card(id) if self.state.objects.get(id as usize).is_some_and(|_o| self.object_mana_value(id) == value)))
                    && self.target_matches_without_hand(*base, controller, source, target, departing, qualification);
            }
            TargetRule::ManaValueAtMost { base, maximum, .. } => {
                return maximum.is_some_and(|maximum| matches!(target, Target::Card(id) if self.state.objects.get(id as usize).is_some_and(|_o| self.object_mana_value(id) <= maximum)))
                    && self.target_matches_without_hand(*base, controller, source, target, departing, qualification);
            }
            TargetRule::GraveCreatureEnteredThisTurn => {
                return matches!(target, Target::Card(id) if self.state.objects.get(id as usize).is_some_and(|o|
                    o.zone == Zone::Graveyard && o.is_card() && o.card.is_creature()
                    && self.graveyard_entry_this_turn(self.object_ref(id))));
            }
            TargetRule::GraveCreatureSourcePower { maximum } => {
                let Target::Card(id) = target else {
                    return false;
                };
                return self.state.objects.get(id as usize).is_some_and(|o| {
                    o.zone == Zone::Graveyard
                        && o.is_card()
                        && o.owner == controller
                        && o.card.is_creature()
                        && maximum.is_some_and(|max| i128::from(o.card.fixed_mana_value()) <= max)
                });
            }
            TargetRule::GraveCreaturePower {
                excluded_subtype,
                maximum,
            } => {
                let Target::Card(id) = target else {
                    return false;
                };
                return self.state.objects.get(id as usize).is_some_and(|object| {
                    object.zone == Zone::Graveyard
                        && object.is_card()
                        && object.owner == controller
                        && object.card.is_creature()
                        && !object.card.has_subtype(excluded_subtype)
                        && self.object_ref(id) != source
                        && maximum
                            .is_some_and(|max| i128::from(object.card.fixed_mana_value()) <= max)
                });
            }
            TargetRule::Grouped { base, constraint } => {
                if constraint == TargetGroupConstraint::DistinctCardTypeSlots
                    && !matches!(target, Target::Card(id) if self.state.objects.get(id as usize).is_some_and(|o| !o.is_token() && !o.is_spell_copy()))
                {
                    return false;
                }
                let zone = match constraint {
                    TargetGroupConstraint::SameGraveyard
                    | TargetGroupConstraint::DistinctCardTypeSlots => Some(Zone::Graveyard),
                    TargetGroupConstraint::DifferentControllers
                    | TargetGroupConstraint::SharedCardType
                    | TargetGroupConstraint::ControllerOfPlayerGroup(_)
                    | TargetGroupConstraint::AttachedToGroup(_) => Some(Zone::Battlefield),
                    TargetGroupConstraint::OtherThanGroup(_)
                    | TargetGroupConstraint::TotalManaValueAtMost(_) => None,
                };
                return zone.is_none_or(|zone| matches!(target, Target::Card(id) if self.state.objects.get(id as usize).is_some_and(|o| o.zone == zone)))
                    && self.target_matches_without_hand(
                        *base,
                        controller,
                        source,
                        target,
                        departing,
                        qualification,
                    );
            }
            TargetRule::Union(rules) => {
                return rules.iter().any(|&rule| {
                    self.target_matches_without_hand(
                        rule,
                        controller,
                        source,
                        target,
                        departing,
                        qualification,
                    )
                });
            }
            TargetRule::Filtered { base, predicate } => {
                if !self.target_matches_without_hand(
                    *base,
                    controller,
                    source,
                    target,
                    departing,
                    qualification,
                ) {
                    return false;
                }
                let Target::Card(id) = target else {
                    return false;
                };
                // The base rule validates the object index and zone before this lookup.
                let object = &self.state.objects[id as usize];
                return match *predicate {
                    TargetPredicate::OwnedByController => object.owner == controller,
                    TargetPredicate::EnteredGraveyardThisTurn => {
                        object.zone == Zone::Graveyard
                            && object.is_card()
                            && self.graveyard_entry_this_turn(self.object_ref(id))
                    }
                    TargetPredicate::EnteredBattlefieldThisTurn => {
                        object.zone == Zone::Battlefield
                            && self.entered_this_turn(self.object_ref(id))
                    }
                    TargetPredicate::ManaValueAtMostLifeGained => {
                        u128::from(self.object_mana_value(id))
                            <= self.state.players[controller.index()].life_gained_this_turn
                    }
                    TargetPredicate::OtherThanSource => self.object_ref(id) != source,
                    TargetPredicate::OtherThanSourceAttachment => {
                        let attached = if self.on_battlefield(source.card)
                            && self.object_ref(source.card) == source
                        {
                            self.state.objects[source.card as usize].attached_to
                        } else {
                            qualification.and_then(|q| q.attachment)
                        };
                        attached.is_some_and(|r| self.object_ref(id) != r)
                    }
                    TargetPredicate::Blocking => self.state.blocking.contains(&id),
                    TargetPredicate::Nontoken => !object.is_token(),
                    TargetPredicate::NoAbilities => !self.object_has_rules_abilities(id),
                    TargetPredicate::Attacking => self.state.attackers.iter().any(|a| a.card == id),
                    TargetPredicate::AttackedThisTurn => {
                        self.state.attacked_this_turn.contains(&self.object_ref(id))
                    }
                    TargetPredicate::Nonattacking => {
                        !self.state.attackers.iter().any(|a| a.card == id)
                    }
                    TargetPredicate::AttackingAlone => {
                        self.state.attackers.len() == 1 && self.state.attackers[0].card == id
                    }
                    TargetPredicate::AttackingOrBlocking => {
                        self.state.attackers.iter().any(|a| a.card == id)
                            || self.state.blocking.contains(&id)
                    }
                    TargetPredicate::Damaged => object.damaged_this_turn,
                    TargetPredicate::DealtDamage => object.dealt_damage_this_turn,
                    TargetPredicate::HasCounter(kind) => self.counter_count(id, kind) > 0,
                    TargetPredicate::PowerAtLeast(n) => {
                        self.stats_without_hand(id, departing).power >= i128::from(n)
                    }
                    TargetPredicate::PowerAtMost(n) => {
                        self.stats_without_hand(id, departing).power <= i128::from(n)
                    }
                    TargetPredicate::ToughnessAtLeast(n) => {
                        self.stats_without_hand(id, departing).toughness >= i128::from(n)
                    }
                    TargetPredicate::PowerOrToughnessAtLeast(n) => {
                        let stats = self.stats_without_hand(id, departing);
                        stats.power >= i128::from(n) || stats.toughness >= i128::from(n)
                    }
                    TargetPredicate::Colors(mask) => {
                        ability_world::CharacteristicWorld::colors(self, id) & mask != 0
                    }
                    TargetPredicate::Keyword(keyword) => self.keywords(id).contains(keyword),
                    TargetPredicate::WithoutKeyword(keyword) => {
                        !self.keywords(id).contains(keyword)
                    }
                    TargetPredicate::WithoutKinds(kinds) => self.current_kinds(id) & kinds == 0,
                    TargetPredicate::WithoutSubtype(subtype) => !self.has_subtype(id, subtype),
                    TargetPredicate::Token => object.is_token(),
                    TargetPredicate::Subtype(subtype) => self.has_subtype(id, subtype),
                    TargetPredicate::Permanent => {
                        self.current_kinds(id) & (1 | 2 | 16 | 32 | 64 | 256) != 0
                    }
                    TargetPredicate::Legendary => {
                        ability_world::CharacteristicWorld::legendary(self, id)
                    }
                    TargetPredicate::Nonlegendary => {
                        !ability_world::CharacteristicWorld::legendary(self, id)
                    }
                    TargetPredicate::Nonbasic => !object.card.is_basic_land(),
                    TargetPredicate::NonOutlaw => {
                        !["Assassin", "Mercenary", "Pirate", "Rogue", "Warlock"]
                            .iter()
                            .any(|subtype| self.has_subtype(id, subtype))
                    }
                };
            }
            _ => {}
        }
        match target {
            Target::Player(player) => {
                (player == controller || !self.player_has_hexproof(player))
                    && match rule {
                        TargetRule::Any => true,
                        TargetRule::Player { opponent_only } => {
                            !opponent_only || player != controller
                        }
                        _ => false,
                    }
            }
            Target::Stack(id) => self.state.stack.iter().any(|item| {
                item.id == id && self.stack_target_rule_matches(rule, item, controller)
            }),
            Target::Spell(id) => self.state.stack.iter().any(|item| {
                item.kind == StackKind::Spell
                    && item
                        .source
                        .is_some_and(|identity| identity.id == id && identity.object != source)
                    && self.stack_target_rule_matches(rule, item, controller)
            }),
            Target::Card(id) => {
                let Some(obj) = self.state.objects.get(id as usize) else {
                    return false;
                };
                if obj.zone == Zone::Battlefield
                    && (self.keywords(id).contains(Keywords::PROTECTION_EVERYTHING)
                        || (self
                            .keywords(id)
                            .contains(Keywords::PROTECTION_INSTANT_SORCERY)
                            && self.instant_sorcery_source(source, qualification)))
                {
                    return false;
                }
                if obj.zone == Zone::Battlefield
                    && obj.controller != controller
                    && (self.keywords(id).contains(Keywords::HEXPROOF)
                        || self.qualified_hexproof_blocks(id, controller, source, qualification))
                {
                    return false;
                }
                match rule {
                    TargetRule::CreatureOrVehicle { relation } => {
                        obj.zone == Zone::Battlefield
                            && (self.is_creature(id) || self.has_subtype(id, "Vehicle"))
                            && (relation == -1 || (obj.controller == controller) == (relation == 0))
                    }
                    TargetRule::Creature => obj.zone == Zone::Battlefield && self.is_creature(id),
                    TargetRule::Any => {
                        obj.zone == Zone::Battlefield
                            && (self.is_creature(id)
                                || self.current_kinds(id) & (64 | crate::cards::BATTLE_KIND) != 0)
                    }
                    TargetRule::Object {
                        zone,
                        kinds,
                        relation,
                        other,
                        nonland,
                        tapped,
                        min_mana,
                        max_mana,
                    } => {
                        let expected = match zone {
                            TargetZone::Battlefield => Zone::Battlefield,
                            TargetZone::Graveyard => Zone::Graveyard,
                        };
                        obj.zone == expected
                            && (expected != Zone::Graveyard || obj.is_card())
                            && (kinds == 0 || self.current_kinds(id) & u16::from(kinds) != 0)
                            && (relation == -1
                                || (if expected == Zone::Battlefield {
                                    obj.controller
                                } else {
                                    obj.owner
                                } == controller)
                                    == (relation == 0))
                            && (!other || self.object_ref(id) != source)
                            && (!nonland || self.current_kinds(id) & 2 == 0)
                            && (!tapped || obj.tapped)
                            && ((min_mana == 0 && max_mana == u32::MAX) || {
                                let value = self.object_mana_value(id);
                                value >= u64::from(min_mana) && value <= u64::from(max_mana)
                            })
                    }
                    _ => false,
                }
            }
        }
    }

    pub(super) fn same_target_object(&self, target: BoundTarget) -> bool {
        match target.target {
            Target::Stack(id) => self.state.stack.iter().any(|item| item.id == id),
            Target::Player(_) => target.object.is_none(),
            Target::Card(id) | Target::Spell(id) => self
                .state
                .objects
                .get(id as usize)
                .is_some_and(|_| target.object == Some(self.object_ref(id))),
        }
    }

    pub(super) fn visit_item_targets(
        &self,
        item: &SpellView,
        rule: TargetRule,
        emit: impl FnMut(Target) -> bool,
    ) -> bool {
        self.visit_targets_without_hand(
            rule,
            item.controller,
            item.target_source(),
            None,
            Some(TargetSourceCharacteristics::from_item(item)),
            emit,
        )
    }
    pub(super) fn visit_targets(
        &self,
        rule: TargetRule,
        controller: Player,
        source: ObjectRef,
        emit: impl FnMut(Target) -> bool,
    ) -> bool {
        self.visit_targets_without_hand(rule, controller, source, None, None, emit)
    }
    pub(super) fn visit_cast_targets(
        &self,
        rule: TargetRule,
        controller: Player,
        source: ObjectRef,
        emit: impl FnMut(Target) -> bool,
    ) -> bool {
        let departing = self
            .state
            .objects
            .get(source.card as usize)
            .filter(|o| o.zone == Zone::Hand && o.generation == source.generation)
            .map(|_| source.card);
        self.visit_targets_without_hand(rule, controller, source, departing, None, emit)
    }
    fn visit_targets_without_hand(
        &self,
        rule: TargetRule,
        controller: Player,
        source: ObjectRef,
        departing: Option<CardId>,
        qualification: Option<TargetSourceCharacteristics<'_>>,
        mut emit: impl FnMut(Target) -> bool,
    ) -> bool {
        if rule == TargetRule::None {
            return false;
        }
        for player in [Player::P0, Player::P1] {
            let target = Target::Player(player);
            if self.target_matches_without_hand(
                rule,
                controller,
                source,
                target,
                departing,
                qualification,
            ) && emit(target)
            {
                return true;
            }
        }
        for id in 0..self.state.objects.len() {
            let target = Target::Card(id as CardId);
            if self.target_matches_without_hand(
                rule,
                controller,
                source,
                target,
                departing,
                qualification,
            ) && emit(target)
            {
                return true;
            }
        }
        for item in &self.state.stack {
            {
                let target = if item.kind == StackKind::Spell && !item.is_copy {
                    let Some(source) = item.source else {
                        continue;
                    };
                    Target::Spell(source.id)
                } else {
                    Target::Stack(item.id)
                };
                if self.target_matches_without_hand(
                    rule,
                    controller,
                    source,
                    target,
                    departing,
                    qualification,
                ) && emit(target)
                {
                    return true;
                }
            }
        }
        false
    }

    pub(super) fn cast_item(&mut self, actor: Player, card: CardId, payment: Mana) -> SpellView {
        let outcome = self.spend_mana_for(
            actor,
            payment,
            self.spell_mana_purpose_for_card(actor, card, self.effective_spell_card(card)),
        );
        self.state.passes = 0;
        let stash_origin = self.foreign_stash(actor, card);
        let modifiers = self.exile_cast_modifiers(actor, card);
        let origin = self.state.objects[card as usize].zone;
        let before = self.object_view(card);
        let event = self.transition_object_under_control(
            before,
            Zone::Stack,
            false,
            Counters::default(),
            Some(actor),
        );
        if origin == Zone::Exile
            && event.moved
            && event.destination == Zone::Stack
            && let Some(data) = &mut self.state.linked_choices
        {
            for link in &mut data.null_cast_links {
                if link.exiled == event.before.reference {
                    link.exiled = event.after;
                }
            }
        }
        self.finish_zone_events(std::slice::from_ref(&event), &[]);
        let object = event.after;
        self.advance_prepared_copy(event.before.reference, event.after);
        if outcome.haste {
            self.grant_stack_keywords(object, Keywords::HASTE);
        }
        let id = self.next_stack_id();
        let mut item = self.cast_preview(actor, card, origin, object, id, payment);
        if outcome.uncounterable {
            self.mark_spell_uncounterable(id);
        }
        item.cast.as_mut().unwrap().treasure_mana_spent |= outcome.treasure_spent;
        item.mana_copy_sources = outcome.copies;
        item.cast.as_mut().unwrap().stash_origin = stash_origin;
        item.cast.as_mut().unwrap().spend_any_mana = modifiers.0;
        item.cast.as_mut().unwrap().exile_on_graveyard = modifiers.1;
        item
    }
    pub(super) fn cast_preview(
        &self,
        actor: Player,
        card: CardId,
        origin: Zone,
        object: ObjectRef,
        id: u64,
        payment: Mana,
    ) -> SpellView {
        SpellView {
            source: Some(StackSourceView {
                id: card,
                object,
                card: self.state.objects[card as usize].card,
                facts: self.damage_facts(card),
            }),
            source_less_rule: None,
            additional_triggers: 0,
            gift: None,
            source_move: None,
            borrowed_slot: None,
            self_granted_activation: None,
            target_controller_lki: None,
            crime_qualifying_target: false,
            target_kinds_lki: None,
            target_type_slots: None,
            reflexive_subject: None,
            crew: None,
            cast_event: None,
            cast_creature_subjects: None,
            copy_event: None,
            captured_artifact_copy: None,
            source_copy_recipe: None,
            source_ability_origin: None,
            target_allocations: None,
            mana_copy_sources: None,
            is_copy: false,
            ninjutsu_defender: None,
            delayed_sacrifice: None,
            delayed_warp_exile: None,
            return_link: None,
            x: None,
            loyalty_x: None,
            reflexive_target_limit: None,
            attachment_trigger: None,
            cast: Some(CastInfo {
                spend_any_mana: self.exile_cast_modifiers(actor, card).0,
                exile_on_graveyard: self.exile_cast_modifiers(actor, card).1,
                caster: actor,
                gift: None,
                physical_card: None,
                face: CastFace::Normal,
                payment,
                treasure_mana_spent: false,
                origin,
                stash_origin: self.foreign_stash(actor, card),
                method: CastMethod::Normal,
                sneak_defender: None,
                flash_surcharge: false,
                offspring: None,
                kicker: None,
                teamwork: None,
                blight: None,
                bargain: None,
                evidence: None,
            }),
            id,
            kind: StackKind::Spell,
            source_transform: None,

            split_second: false,
            source_keywords: self.keywords(card),
            source_tapped: self.state.objects[card as usize].tapped,
            mana_tap_bonus: None,
            related_stack: None,
            entering_object: None,
            evidence_collected: false,
            damage_event: None,
            damage_received: None,
            creature_combat_event: None,
            discard_event: None,
            draw_event: None,
            life_event: None,
            source_counter_payment: None,
            loyalty_counters_paid: 0,
            exiled_symbol_cards: None,
            craft_materials: None,
            exiled_trigger_cards: None,
            source_counter_lki: None,
            source_quantity_lki: None,
            event_player: None,
            event_object: None,
            manifest_event: None,
            sacrificed_toughness: None,
            sacrificed_mana_value: None,
            sacrificed_power: None,
            source_attachment: self.state.objects[card as usize].attached_to,

            controller: actor,
            targets: Vec::new(),
            mode: None,
        }
    }

    /// Returns true only when a decision suspends the enclosing ordering/cast work.
    pub(super) fn start_announcement(&mut self, item: SpellView, trigger: bool) -> bool {
        let needs_mode = item.modes().is_some() && item.mode.is_none();
        if needs_mode
            && !item
                .modes()
                .unwrap()
                .iter()
                .enumerate()
                .any(|(index, mode)| {
                    self.modal_choice_unused(&item, index)
                        && self.item_targets_available(&item, mode.targets)
                })
        {
            assert!(trigger, "validated modal spell");
            return false;
        }
        let plan = self.target_spec(&item);
        if !self.item_targets_available(&item, plan) {
            assert!(
                trigger,
                "validated spell announcement: {} ({plan:?})",
                item.source
                    .map_or("inherent rule", |source| source.card.name())
            );
            return false;
        }
        let has_choice = needs_mode
            || match plan {
                TargetPlan::Single(spec) => {
                    spec.max > 0 && self.visit_item_targets(&item, spec.rule, |_| true)
                }
                TargetPlan::Groups(groups) => groups.iter().any(|spec| {
                    spec.max > 0 && self.visit_item_targets(&item, spec.rule, |_| true)
                }),
            };
        if !has_choice {
            // Prepaid declarations settle their deferred costs in the caller.
            // A spell is still only a proposal here, so do not fire cast triggers.
            let mut item = item;
            self.capture_transform_stack_serial(&mut item);
            self.state.stack.push(item);
            return false;
        }
        assert!(self.state.announcing.is_none());
        self.state.announcing = Some(Announcement {
            equip_zero_available: false,
            activation_cost_route: ActivationCostRoute::Printed,
            activation_discount: self.item_activation_discount(&item),
            power_up_costs: self.item_power_up_costs(&item),
            free_hand_permission: None,
            origin_counter_cost: None,
            miracle_cost: self.miracle_cost_for(&item),
            item,
            spec: TargetSpec::NONE,
            plan,
            group: 0,
            trigger,
            payment: None,
            cost: None,
            total_cost: None,
            counter_payment: Counters::default(),
            additional: None,
            additional_life: false,
            additional_group: None,
            creature_counter_cost: None,
            forage: None,
            casualty: None,
            cast_costs: None,
            evidence: None,
            convoke: None,
            harmonizer: None,
            harmonize_discount: 0,
            additional_paid: false,
            pay_window: false,
            production: None,
        });
        self.continue_announcement_choices(needs_mode);
        true
    }

    pub(super) fn open_payment_announcement(
        &mut self,
        item: SpellView,
        counters: Counters,
        production: Option<Mana>,
    ) {
        self.open_payment_announcement_with_origin_cost(item, counters, production, None);
    }

    pub(super) fn open_payment_announcement_with_origin_cost(
        &mut self,
        mut item: SpellView,
        counters: Counters,
        production: Option<Mana>,
        origin_counter_cost: Option<u16>,
    ) {
        let source = item.source.expect("payment announcement has object source");
        if matches!(item.kind, StackKind::ActivatedAbility { .. })
            && self.state.objects[source.id as usize].zone == Zone::Hand
            && item
                .program()
                .is_some_and(|(_, program)| program.contains(&Instruction::PutHandSourceAttacking))
        {
            // Ninjutsu reveals its exact hand object while the ability is announced.
            // Public knowledge persists until this incarnation leaves the hand.
            self.remember_public_hand(source.object);
        }
        if item.cast.is_some_and(|cast| cast.face == CastFace::Back) {
            self.state.objects[source.id as usize].card = source.card;
        }
        // A copied card becomes a genuine token when its permanent spell resolves.
        // Freeze the selected face now, after face choice and before any zone move.
        if self.state.objects[source.id as usize].copy_kind == CopyKind::Card
            && source.card.kind_mask() & (1 | 16 | 32 | 64 | 256) != 0
        {
            let recipe = CopyRecipe::new(source.card);
            self.set_copy_recipe(source.object, recipe);
            item.source_copy_recipe = Some(Box::new(recipe));
        }
        if production.is_some() {
            self.suspend_mana_announcement();
            self.capture_announced_mana_quality(&item);
        }
        let plan = self.target_spec(&item);
        assert!(
            self.item_targets_available(&item, plan)
                || source.card.gift().is_some_and(|gift| {
                    self.item_targets_available(&item, gift.promised_targets)
                })
                || source
                    .card
                    .teamwork()
                    .and_then(|s| s.paid_targets)
                    .is_some_and(|plan| self.item_targets_available(&item, plan))
                || source
                    .card
                    .kicked_targets()
                    .is_some_and(|plan| { self.item_targets_available(&item, plan) })
        );
        let needs_mode = item.modes().is_some() && item.mode.is_none();
        let casualty = self.casualty_declarations(&item);
        self.state.announcing = Some(Announcement {
            equip_zero_available: self.item_first_equip_zero_available(&item)
                && counters == Counters::default(),
            activation_cost_route: ActivationCostRoute::Printed,
            activation_discount: self.item_activation_discount(&item),
            power_up_costs: self.item_power_up_costs(&item),
            free_hand_permission: None,
            origin_counter_cost,
            miracle_cost: self.miracle_cost_for(&item),
            item,
            spec: TargetSpec::NONE,
            plan,
            group: 0,
            trigger: false,
            payment: None,
            cost: None,
            total_cost: None,
            counter_payment: counters,
            additional: None,
            additional_life: false,
            additional_group: None,
            creature_counter_cost: None,
            forage: None,
            casualty,
            cast_costs: None,
            evidence: None,
            convoke: None,
            harmonizer: None,
            harmonize_discount: 0,
            additional_paid: false,
            pay_window: true,
            production,
        });
        self.state
            .announcing
            .as_mut()
            .unwrap()
            .initialize_alternative_life();
        self.continue_announcement_choices(needs_mode);
    }

    pub(super) fn finish_announcement(&mut self) {
        if self.start_target_amounts() {
            return;
        }
        if self.start_crew_payment() {
            return;
        }
        if self.start_saddle_payment() {
            return;
        }
        if let Some(announcement) = &self.state.announcing
            && announcement.pay_window
            && matches!(self.state.pending, Pending::Targets { .. })
        {
            if let StackKind::ActivatedAbility { ability } = announcement.item.kind
                && Self::activation_target_sensitive(
                    announcement
                        .item
                        .source
                        .expect("announcement has object source")
                        .card,
                    ability,
                )
                && announcement.total_cost.is_none()
            {
                let base = announcement.cost.unwrap_or_else(|| {
                    announcement.cost_options().next().expect("activation cost")
                });
                let total = self.selected_activation_target_cost(&announcement.item, base);
                self.state.announcing.as_mut().unwrap().total_cost = Some(total);
            }
            let announcement = self.state.announcing.as_ref().unwrap();
            if announcement.item.kind == StackKind::Spell && announcement.total_cost.is_none() {
                let base = Self::announcement_base_cost(announcement);
                let discount = announcement.harmonizer.flatten().map_or(0, |reference| {
                    harmonize::contribution(self, announcement.item.controller, reference)
                        .expect("chosen Harmonizer remains available before cost determination")
                });
                let extra =
                    Self::announcement_optional_reduction(announcement) + u64::from(discount);
                let total = if announcement
                    .item
                    .source
                    .expect("announcement has object source")
                    .card
                    .x_symbols()
                    > 0
                {
                    self.scalar_x_total_with_extra(
                        announcement.item.controller,
                        announcement
                            .item
                            .source
                            .expect("announcement has object source")
                            .id,
                        base,
                        announcement.item.x.expect("X chosen"),
                        announcement
                            .item
                            .source
                            .expect("announcement has object source")
                            .card,
                        extra,
                    )
                    .expect("validated scalar cost")
                } else {
                    self.target_spell_cost_with_extra(
                        announcement.item.controller,
                        announcement
                            .item
                            .source
                            .expect("announcement has object source")
                            .id,
                        base,
                        &announcement.item.targets,
                        extra,
                    )
                    .expect("validated target spell cost")
                };
                let announcement = self.state.announcing.as_mut().unwrap();
                announcement.harmonize_discount = discount;
                announcement.total_cost = Some(total);
            }
            if self.state.announcing.as_ref().unwrap().cast_costs.is_none()
                && self
                    .state
                    .announcing
                    .as_ref()
                    .unwrap()
                    .additional_cost()
                    .and_then(evidence::threshold)
                    .is_some()
            {
                self.start_announcement_evidence();
                return;
            }
            let announcement = self.state.announcing.as_ref().unwrap();
            self.state.pending = if announcement.cast_costs.is_some() {
                Pending::PayCosts {
                    player: announcement.item.controller,
                }
            } else if announcement
                .additional_cost()
                .is_some_and(forage::is_forage_cost)
                && announcement.forage.as_ref().is_some_and(|f| !f.complete())
            {
                Pending::Forage {
                    player: announcement.item.controller,
                }
            } else if announcement
                .additional_cost()
                .is_some_and(|spec| !forage::is_forage_cost(spec))
                && announcement.additional.is_none()
                && !announcement.additional_life
            {
                Pending::AdditionalCost {
                    player: announcement.item.controller,
                }
            } else {
                Pending::PayCosts {
                    player: announcement.item.controller,
                }
            };
            return;
        }
        let mut announcement = self.state.announcing.take().unwrap();
        self.capture_announced_cast_mana(&announcement);
        if announcement.cast_costs.is_none()
            && let Some(frame) = announcement.convoke.take()
        {
            self.tap_objects(&frame.objects().iter().map(|s| s.card).collect::<Vec<_>>());
        }
        if let Some(slots) = &mut announcement.item.target_type_slots {
            let mut rows: Vec<_> = announcement
                .item
                .targets
                .iter()
                .copied()
                .zip(slots.iter().copied())
                .collect();
            rows.sort_by_key(|(bound, _)| (bound.group, bound.target));
            announcement.item.targets = rows.iter().map(|(bound, _)| *bound).collect();
            **slots = rows.into_iter().map(|(_, slot)| slot).collect();
        } else {
            announcement
                .item
                .targets
                .sort_by_key(|bound| (bound.group, bound.target));
        }
        // External discard precedes source sacrifice so it captures battlefield
        // observers. External sacrifice follows source tap/mana costs: the chosen
        // object may itself be this ability's source. No priority is granted.
        if !announcement.ordered_craft()
            && matches!(announcement.item.kind, StackKind::ActivatedAbility { .. })
            && !announcement.additional_paid
        {
            if announcement.additional_cost().is_some_and(|spec| {
                let spec = announcement.additional.map_or(spec, |object| {
                    spec.in_zone(self.state.objects[object.card as usize].zone)
                });
                matches!(
                    spec.kind,
                    EffectChoiceKind::Sacrifice
                        | EffectChoiceKind::SacrificeDifferentNames(_)
                        | EffectChoiceKind::ReturnToHand
                ) || forage::is_forage_cost(spec)
                    || evidence::threshold(spec).is_some()
            }) && let Some(payment) = announcement.payment.take()
                && let Some(phase) = self.pay_activation_route(
                    &mut announcement.item,
                    payment,
                    announcement.counter_payment.clone(),
                    announcement.activation_cost_route,
                )
            {
                let item = announcement.item.clone();
                let route = announcement.activation_cost_route;
                self.state.announcing = Some(announcement);
                self.suspend_activation_payment(
                    item,
                    phase,
                    route,
                    activations::ActivationPaymentCompletion::Announcement,
                );
                return;
            }
            self.pay_additional_cost(&mut announcement);
            announcement.additional_paid = true;
        }
        if let Some(payment) = announcement.payment {
            if announcement.ordered_craft() {
                self.spend_mana_for(
                    announcement.item.controller,
                    payment,
                    self.purpose_for_item(&announcement.item),
                );
            } else if matches!(announcement.item.kind, StackKind::ActivatedAbility { .. }) {
                if let Some(phase) = self.pay_activation_route(
                    &mut announcement.item,
                    payment,
                    announcement.counter_payment.clone(),
                    announcement.activation_cost_route,
                ) {
                    announcement.payment = None;
                    let item = announcement.item.clone();
                    let route = announcement.activation_cost_route;
                    self.state.announcing = Some(announcement);
                    self.suspend_activation_payment(
                        item,
                        phase,
                        route,
                        activations::ActivationPaymentCompletion::Announcement,
                    );
                    return;
                }
            } else {
                let outcome = self.spend_mana_for(
                    announcement.item.controller,
                    payment,
                    self.purpose_for_item(&announcement.item),
                );
                if outcome.uncounterable {
                    self.mark_spell_uncounterable(announcement.item.id);
                }
                if outcome.haste {
                    self.grant_stack_keywords(
                        announcement
                            .item
                            .source
                            .expect("announcement has object source")
                            .object,
                        Keywords::HASTE,
                    );
                }
                announcement.item.cast.as_mut().unwrap().treasure_mana_spent =
                    outcome.treasure_spent;
                announcement.item.mana_copy_sources = outcome.copies;
                announcement.item.cast.as_mut().unwrap().payment = payment;
            }
        }
        self.finish_paid_announcement(announcement);
    }

    /// Finish the remaining external cost and announcement after an owned
    /// activation payment resumes from a counter-replacement choice.
    pub(super) fn resume_activation_announcement(&mut self, item: SpellView) {
        let mut announcement = self
            .state
            .announcing
            .take()
            .expect("suspended activation announcement");
        announcement.item = item;
        if !announcement.additional_paid {
            self.pay_additional_cost(&mut announcement);
            announcement.additional_paid = true;
        }
        self.finish_paid_announcement(announcement);
    }

    fn finish_paid_announcement(&mut self, mut announcement: Announcement) {
        // Costs may synchronously return exiled Auras/lands. Finish their entry
        // choices before the spell becomes cast and cast observers are collected.
        announcement.payment = None;
        if self.state.entry_batch.is_some() {
            self.state.announcing = Some(announcement);
            return;
        }
        if self.continue_cast_cost_payment(&mut announcement) {
            self.state.announcing = Some(announcement);
            return;
        }
        if announcement.cast_costs.is_none()
            && announcement.item.kind == StackKind::Spell
            && !announcement.additional_paid
        {
            self.pay_additional_cost(&mut announcement);
            if self.state.entry_batch.is_some() {
                self.state.announcing = Some(announcement);
                return;
            }
        }
        self.state.priority = if announcement.trigger {
            self.state.priority
        } else {
            announcement.item.controller
        };
        if let Some(production) = announcement.production {
            self.add_announced_mana(&announcement.item, production);
            self.committed_mana_reflexive(&announcement.item);
            if self.restore_mana_announcement() {
                return;
            }
            if let Some(payment) = &self.state.attack_payment {
                self.state.pending = Pending::AttackPayment {
                    player: payment.player,
                };
                return;
            }
        } else {
            if let Some(permission) = announcement.free_hand_permission.take() {
                self.use_free_hand_permission(*permission);
            }
            self.queue_casualty_copies(&announcement);
            self.push_stack_item(announcement.item);
        }
        if announcement.trigger {
            self.continue_trigger_order();
        } else {
            self.state.pending =
                self.state
                    .resolution_free_casts
                    .as_ref()
                    .map_or(Pending::Priority, |free| Pending::ResolutionCasts {
                        player: free.player,
                    });
            self.state.passes = 0;
        }
    }

    pub(super) fn move_target_batch(&mut self, targets: &[BoundTarget], destination: Zone) {
        let sources = if destination == Zone::Graveyard {
            self.death_trigger_sources()
        } else {
            Vec::new()
        };
        let ids: Vec<_> = targets
            .iter()
            .copied()
            .filter(|&bound| self.same_target_object(bound))
            .filter_map(|bound| match bound.target {
                Target::Card(id) | Target::Spell(id) => Some(id),
                _ => None,
            })
            .collect();
        if destination == Zone::Battlefield {
            let moves = ids
                .iter()
                .map(|&id| self.entry_move(id, destination, false))
                .collect();
            self.stage_entries(moves, None);
        } else {
            let events = self.move_objects_raw(&ids, destination);
            self.finish_zone_events(&events, &sources);
        }
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    /// Current immutable recipe plus any rules abilities acquired in the current zone.
    pub(super) fn object_has_rules_abilities(&self, id: CardId) -> bool {
        let object = &self.state.objects[id as usize];
        if object.card.has_rules_abilities() {
            return true;
        }
        // Existing graveyard cast rows grant flashback-like abilities, rather than
        // merely allowing an out-of-zone cast. They count even while unusable.
        object.zone == Zone::Graveyard
            && self
                .state
                .graveyard_cast_permissions
                .as_ref()
                .is_some_and(|rows| {
                    rows.iter().any(|row| {
                        row.object == self.object_ref(id)
                            && row.kind == crate::cards::GraveyardCastKind::Flashback
                    })
                })
    }
    pub(super) fn object_search_filter_matches(
        &self,
        filter: crate::SearchFilter,
        id: CardId,
    ) -> bool {
        match filter {
            crate::SearchFilter::NoAbilities => !self.object_has_rules_abilities(id),
            crate::SearchFilter::AllOf(filters) => filters
                .iter()
                .all(|filter| self.object_search_filter_matches(*filter, id)),
            crate::SearchFilter::AnyOf(filters) => filters
                .iter()
                .any(|filter| self.object_search_filter_matches(*filter, id)),
            filter => filter.matches(self.state.objects[id as usize].card),
        }
    }
}
