//! Shared target predicates, binding, and owned incremental announcements.
use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Announcement {
    pub item: SpellView,
    pub spec: TargetSpec,
    pub plan: TargetPlan,
    pub group: u16,
    pub trigger: bool,
    pub payment: Option<Mana>,
    pub cost: Option<ManaCost>,
    pub total_cost: Option<ManaCost>,
    pub counter_payment: Counters,
    pub pay_window: bool,
    pub production: Option<Mana>,
    pub additional: Option<ObjectRef>,
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
    pub evidence: Option<Box<evidence::EvidenceSelection>>,
    pub convoke: Option<Box<convoke::ConvokeSelection>>,
    pub additional_paid: bool,
}

impl Announcement {
    pub(super) fn additional_cost(&self) -> Option<AdditionalCost> {
        match self.item.kind {
            StackKind::Spell
                if self
                    .item
                    .cast
                    .is_some_and(|c| c.method == CastMethod::GraveyardCounterCost) =>
            {
                Some(Game::creature_counter_spec(
                    self.item.source_card.graveyard_counter_cost().unwrap(),
                ))
            }
            StackKind::Spell if self.item.cast.is_some_and(|c| c.evidence == Some(true)) => Some(
                evidence::cost(self.item.source_card.collect_evidence_cost().unwrap()),
            ),
            StackKind::Spell if self.item.cast.is_some_and(|c| c.bargain == Some(true)) => {
                Some(bargain::COST)
            }
            StackKind::Spell
                if self
                    .cost
                    .is_some_and(|cost| self.mana_route_replaces_additional(cost)) =>
            {
                None
            }
            StackKind::Spell => self.item.source_card.additional_cost().and_then(|spec| {
                if let EffectChoiceKind::OptionalBlightCost(amount) = spec.kind {
                    self.item
                        .cast
                        .filter(|cast| cast.optional_additional == Some(true))
                        .map(|_| AdditionalCost {
                            kind: EffectChoiceKind::BlightCost(amount),
                            ..spec
                        })
                } else {
                    Some(spec)
                }
            }),
            StackKind::ActivatedAbility { ability } => {
                self.item.source_card.activations()[ability as usize]
                    .cost
                    .additional
            }
            StackKind::TriggeredAbility { .. }
            | StackKind::CopyEndStepSacrifice
            | StackKind::DelayedSacrifice
            | StackKind::DelayedReturn
            | StackKind::DelayedWarpExile => None,
        }
    }

    fn method_base_cost(&self) -> ManaCost {
        let card = self.item.source_card;
        match self.item.cast.map(|cast| cast.method) {
            Some(CastMethod::Warp) => card.warp_cost().expect("fixed Warp cost"),
            Some(CastMethod::Flashback) => card.flashback_cost().unwrap(),
            Some(CastMethod::Plot | CastMethod::Free) => ManaCost {
                colored: [0; 6],
                generic: 0,
            },
            _ => ManaCost {
                colored: card.cost().0,
                generic: card.cost().1,
            },
        }
    }
    fn additional_mana_route(&self, alternative: ManaCost) -> Option<ManaCost> {
        let printed = self.item.source_card.cost();
        let mut total = self.method_base_cost();
        for (i, component) in total.colored.iter_mut().enumerate() {
            *component =
                component.checked_add(alternative.colored[i].checked_sub(printed.0[i])?)?;
        }
        total.generic = total
            .generic
            .checked_add(alternative.generic.checked_sub(printed.1)?)?;
        Some(total)
    }
    fn modified_cost(&self, cost: ManaCost) -> Option<ManaCost> {
        Game::with_tiered_mana(
            Game::with_flash_surcharge(
                cost,
                self.item.source_card,
                self.item.cast.is_some_and(|cast| cast.flash_surcharge),
            ),
            &self.item,
        )
    }
    pub(super) fn mana_route_replaces_additional(&self, cost: ManaCost) -> bool {
        self.item.kind == StackKind::Spell
            && self
                .item
                .source_card
                .additional_mana_alternatives()
                .iter()
                .any(|alternative| {
                    self.additional_mana_route(*alternative)
                        .and_then(|route| self.modified_cost(route))
                        == Some(cost)
                })
    }
    pub(super) fn cost_options(&self) -> impl Iterator<Item = ManaCost> + '_ {
        let card = self.item.source_card;
        let fixed_method = self.item.cast.is_some_and(|cast| {
            matches!(
                cast.method,
                CastMethod::Warp | CastMethod::Flashback | CastMethod::Plot | CastMethod::Free
            )
        });
        let options = match self.item.kind {
            StackKind::Spell if fixed_method => std::iter::once(self.method_base_cost())
                .chain(card.additional_mana_alternatives().iter().copied()),
            StackKind::Spell => card.cost_options(),
            StackKind::ActivatedAbility { ability } => {
                card.activations()[ability as usize].cost.mana_options()
            }
            _ => unreachable!("triggered ability has no activation cost"),
        };
        options.enumerate().filter_map(move |(index, cost)| {
            if self.item.kind == StackKind::Spell
                && self
                    .item
                    .cast
                    .is_some_and(|cast| cast.method == CastMethod::GrantedFlashback)
                && card.cost_alternative_guard().is_some()
                && cost != self.method_base_cost()
            {
                return None;
            }
            let cost = if fixed_method && index > 0 {
                self.additional_mana_route(cost)?
            } else {
                cost
            };
            self.modified_cost(cost)
        })
    }
    pub(super) fn needs_evidence(&self) -> bool {
        self.pay_window
            && self.item.kind == StackKind::Spell
            && self.item.source_card.collect_evidence_cost().is_some()
            && self.item.cast.is_some_and(|c| c.evidence.is_none())
    }
    pub(super) fn needs_optional_additional(&self) -> bool {
        self.pay_window
            && self.item.kind == StackKind::Spell
            && self
                .item
                .source_card
                .additional_cost()
                .is_some_and(|spec| matches!(spec.kind, EffectChoiceKind::OptionalBlightCost(_)))
            && self
                .item
                .cast
                .is_some_and(|cast| cast.optional_additional.is_none())
    }
    pub(super) fn needs_bargain(&self) -> bool {
        self.pay_window
            && self.item.kind == StackKind::Spell
            && self.item.source_card.has_bargain()
            && self.item.cast.is_some_and(|c| c.bargain.is_none())
    }
    pub(super) fn needs_kicker(&self) -> bool {
        self.pay_window
            && self.item.kind == StackKind::Spell
            && self.item.source_card.kicker_cost().is_some()
            && self.item.cast.is_some_and(|c| c.kicker.is_none())
    }
    pub(super) fn needs_gift(&self) -> bool {
        self.pay_window
            && self.item.kind == StackKind::Spell
            && self.item.source_card.gift().is_some()
            && self.item.cast.is_some_and(|c| c.gift.is_none())
    }
    pub(super) fn needs_offspring(&self) -> bool {
        self.pay_window
            && self.item.kind == StackKind::Spell
            && self.item.source_card.offspring_cost().is_some()
            && self.item.cast.is_some_and(|c| c.offspring.is_none())
    }
    pub(super) fn needs_mana_cost(&self) -> bool {
        self.pay_window
            && self.cost.is_none()
            && match self.item.kind {
                StackKind::Spell => {
                    self.item.cast.is_none_or(|cast| {
                        !matches!(
                            cast.method,
                            CastMethod::Flashback
                                | CastMethod::Plot
                                | CastMethod::Warp
                                | CastMethod::Free
                        )
                    }) && self.item.source_card.has_hybrid_cost()
                        || !self
                            .item
                            .source_card
                            .additional_mana_alternatives()
                            .is_empty()
                }
                StackKind::ActivatedAbility { ability } => !self.item.source_card.activations()
                    [ability as usize]
                    .cost
                    .alternatives
                    .is_empty(),
                StackKind::TriggeredAbility { .. }
                | StackKind::CopyEndStepSacrifice
                | StackKind::DelayedSacrifice
                | StackKind::DelayedReturn
                | StackKind::DelayedWarpExile => false,
            }
    }
}

#[derive(Clone, Copy)]
struct TargetSourceCharacteristics<'a> {
    chosen_type: Option<crate::cards::CreatureType>,
    reference: ObjectRef,
    snapshot: Option<&'a DamageFacts>,
    spell: Option<Card>,
}
impl<'a> TargetSourceCharacteristics<'a> {
    fn from_item(item: &'a SpellView) -> Self {
        Self {
            chosen_type: item.source_chosen_type,
            reference: item.object,
            snapshot: Some(&item.source_facts),
            spell: (item.kind == StackKind::Spell).then_some(item.source_card),
        }
    }
}

impl Game {
    pub(super) fn target_spec(&self, item: &SpellView) -> TargetPlan {
        let plan = item
            .program()
            .map(|(plan, _)| plan)
            .unwrap_or(TargetPlan::NONE);
        // Freeze variable limits into the owned announcement plan before targets are chosen.
        let plan = if let TargetPlan::ScaledSingle { mut spec, maximum } = plan {
            spec.max = spec
                .max
                .min(usize::try_from(self.item_amount(maximum, item)).unwrap_or(usize::MAX));
            TargetPlan::Single(spec)
        } else {
            plan
        };
        if let TargetPlan::Single(mut spec) = plan
            && matches!(spec.rule, TargetRule::GraveCreatureSourcePower { .. })
        {
            let power =
                if self.on_battlefield(item.card) && self.object_ref(item.card) == item.object {
                    self.stats(item.card).power
                } else {
                    item.source_facts.power
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
            chosen_type: None,
            reference: source,
            snapshot: None,
            spell: Some(definition),
        });
        let departing =
            (self.state.objects[source.card as usize].zone == Zone::Hand).then_some(source.card);
        let available = |spec: TargetSpec| {
            if spec.min == 0 {
                return true;
            }
            let mut count = [0usize; 2];
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
            TargetPlan::Single(spec) | TargetPlan::ScaledSingle { spec, .. } => available(spec),
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
            self.visit_targets_without_hand(
                spec.rule,
                actor,
                source,
                None,
                qualification,
                |target| {
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
            TargetPlan::Single(spec) | TargetPlan::ScaledSingle { spec, .. } => available(spec),
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
            TargetGroupConstraint::OtherThanGroup(_) => unreachable!(),
            TargetGroupConstraint::AttachedToGroup(first_group) => object.zone == Zone::Battlefield
                && chosen.iter().find(|b| b.group == first_group).is_some_and(|b| {
                    self.same_target_object(*b) && object.attached_to == b.object
                }),
            TargetGroupConstraint::SharedCreatureType => chosen.iter().filter(|b| b.group == group).all(|b| {
                matches!(b.target, Target::Card(other) if crate::cards::CREATURE_TYPES.iter().any(|ty| self.has_subtype(id, ty) && self.has_subtype(other, ty)))
            }),
            TargetGroupConstraint::SameGraveyard => object.zone == Zone::Graveyard && chosen.iter().filter(|b| b.group == group).all(|b| {
                matches!(b.target, Target::Card(other) if self.state.objects.get(other as usize).is_some_and(|o| o.owner == object.owner))
            }),
            TargetGroupConstraint::DifferentControllers => object.zone == Zone::Battlefield && chosen.iter().filter(|b| b.group == group).all(|b| {
                matches!(b.target, Target::Card(other) if self.state.objects.get(other as usize).is_some_and(|o| o.controller != object.controller))
            }),
        }
    }

    pub(super) fn resolution_target_relations_hold(
        &self,
        item: &SpellView,
        bound: BoundTarget,
        plan: TargetPlan,
    ) -> bool {
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

    pub(super) fn choose_mana_cost(&mut self, index: u16) {
        let a = self.state.announcing.as_mut().unwrap();
        let cost = a.cost_options().nth(index as usize).unwrap();
        if let Some(cast) = a.item.cast.as_mut() {
            cast.evoked = cast.method == CastMethod::Normal
                && a.item.source_card.evoke_costs().contains(&cost);
        }
        a.cost = Some(cost);
        self.continue_announcement_choices(false);
    }

    pub(super) fn choose_mode(&mut self, mode: u16) {
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
        if needs_mode && !a.needs_optional_additional() {
            self.state.pending = Pending::Modes { player };
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
        } else if a.needs_optional_additional() {
            self.state.pending = Pending::OptionalAdditionalCost { player };
        } else if a.needs_mana_cost() {
            self.state.pending = if a.item.kind == StackKind::Spell {
                Pending::SpellCost { player }
            } else {
                Pending::ActivationCost { player }
            };
        } else if a.additional_cost().is_some_and(forage::is_forage_cost) && a.forage.is_none() {
            self.state.pending = Pending::ForageMethod { player };
        } else {
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
                .find(|item| item.object == source)
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

    #[cfg(test)]
    pub(super) fn target_matches(
        &self,
        rule: TargetRule,
        controller: Player,
        source: ObjectRef,
        target: Target,
    ) -> bool {
        self.target_matches_without_hand(rule, controller, source, target, None, None)
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
            TargetRule::GraveCreatureEnteredThisTurn => {
                return matches!(target, Target::Card(id) if self.state.objects.get(id as usize).is_some_and(|o|
                    o.zone == Zone::Graveyard && !o.is_token() && o.card.is_creature()
                    && self.graveyard_entry_this_turn(self.object_ref(id))));
            }
            TargetRule::GraveCreatureSourcePower { maximum } => {
                let Target::Card(id) = target else {
                    return false;
                };
                return self.state.objects.get(id as usize).is_some_and(|o| {
                    o.zone == Zone::Graveyard
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
                        && object.owner == controller
                        && object.card.is_creature()
                        && !object.card.has_subtype(excluded_subtype)
                        && self.object_ref(id) != source
                        && maximum
                            .is_some_and(|max| i128::from(object.card.fixed_mana_value()) <= max)
                });
            }
            TargetRule::Grouped { base, constraint } => {
                let zone = match constraint {
                    TargetGroupConstraint::SameGraveyard => Some(Zone::Graveyard),
                    TargetGroupConstraint::DifferentControllers
                    | TargetGroupConstraint::AttachedToGroup(_) => Some(Zone::Battlefield),
                    TargetGroupConstraint::OtherThanGroup(_)
                    | TargetGroupConstraint::SharedCreatureType => None,
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
                    TargetPredicate::OtherThanSource => self.object_ref(id) != source,
                    TargetPredicate::Blocking => self.state.blocking.contains(&id),
                    TargetPredicate::Nontoken => !object.is_token(),
                    TargetPredicate::Attacking => self.state.attackers.iter().any(|a| a.card == id),
                    TargetPredicate::AttackingOrBlocking => {
                        self.state.attackers.iter().any(|a| a.card == id)
                            || self.state.blocking.contains(&id)
                    }
                    TargetPredicate::Damaged => object.damaged_this_turn,
                    TargetPredicate::DealtDamage => object.dealt_damage_this_turn,
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
                    TargetPredicate::SourceChosenCreatureType => qualification
                        .map(|facts| facts.chosen_type)
                        .unwrap_or_else(|| self.source_chosen_creature_type(source))
                        .is_some_and(|kind| self.has_subtype(id, kind.name())),
                    TargetPredicate::Permanent => {
                        self.current_kinds(id) & (1 | 2 | 16 | 32 | 64 | 256) != 0
                    }
                    TargetPredicate::Legendary => object.card.info().legendary,
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
            Target::Stack(id) => self
                .state
                .stack
                .iter()
                .any(|item| item.id == id && self.stack_target_rule_matches(rule, item)),
            Target::Spell(id) => self.state.stack.iter().any(|item| {
                item.kind == StackKind::Spell
                    && item.card == id
                    && item.object != source
                    && match rule {
                        TargetRule::SingleTargetStackObject => item.targets.len() == 1,
                        TargetRule::Spell => true,
                        TargetRule::SpellKinds { kinds, excluded } => {
                            (kinds == 0 || item.source_card.kind_mask() & u16::from(kinds) != 0)
                                && item.source_card.kind_mask() & u16::from(excluded) == 0
                        }
                        TargetRule::SpellCharacteristics {
                            colors,
                            min_mana,
                            max_mana,
                        } => {
                            (colors == 0 || item.source_card.colors() & colors != 0)
                                && (min_mana..=max_mana).contains(&item.mana_value())
                        }
                        _ => false,
                    }
            }),
            Target::Card(id) => {
                let Some(obj) = self.state.objects.get(id as usize) else {
                    return false;
                };
                if obj.zone == Zone::Battlefield
                    && self.keywords(id).contains(Keywords::PROTECTION_EVERYTHING)
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
                            && (self.is_creature(id) || self.current_kinds(id) & 64 != 0)
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
                            && (expected != Zone::Graveyard || !obj.is_token())
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
                                let value = obj.card.fixed_mana_value();
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
                    Target::Spell(item.card)
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
        let origin = self.state.objects[card as usize].zone;
        let before = self.object_view(card);
        let event = self.transition_object_under_control(
            before,
            Zone::Stack,
            false,
            Counters::default(),
            Some(actor),
        );
        self.finish_zone_events(std::slice::from_ref(&event), &[]);
        let object = event.after;
        if outcome.haste {
            self.grant_stack_keywords(object, Keywords::HASTE);
        }
        let id = self.next_stack_id();
        let mut item = self.cast_preview(actor, card, origin, object, id, payment);
        item.mana_copy_sources = outcome.copies;
        item.cast.as_mut().unwrap().stash_origin = stash_origin;
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
            gift: None,
            source_move: None,
            source_chosen_type: None,
            target_controller_lki: None,
            reflexive_subject: None,
            crew: None,
            cast_event: None,
            copy_event: None,
            source_copy_recipe: None,
            target_allocations: None,
            mana_copy_sources: None,
            is_copy: false,
            delayed_sacrifice: None,
            delayed_warp_exile: None,
            return_link: None,
            x: None,
            attachment_trigger: None,
            cast: Some(CastInfo {
                gift: None,
                physical_card: None,
                face: CastFace::Normal,
                payment,
                origin,
                stash_origin: self.foreign_stash(actor, card),
                method: CastMethod::Normal,
                flash_surcharge: false,
                offspring: None,
                kicker: None,
                bargain: None,
                evidence: None,
                optional_additional: None,
                evoked: false,
            }),
            id,
            kind: StackKind::Spell,
            source_card: self.state.objects[card as usize].card,
            source_keywords: self.keywords(card),
            source_tapped: self.state.objects[card as usize].tapped,
            source_facts: self.damage_facts(card),
            related_stack: None,
            entering_object: None,
            evidence_collected: false,
            damage_event: None,
            creature_combat_event: None,
            discard_event: None,
            draw_event: None,
            life_event: None,
            source_counter_lki: None,
            sacrificed_toughness: None,
            source_attachment: self.state.objects[card as usize].attached_to,
            card,
            object,
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
                item.source_card.name()
            );
            return false;
        }
        let has_choice = needs_mode
            || match plan {
                TargetPlan::Single(spec) | TargetPlan::ScaledSingle { spec, .. } => {
                    spec.max > 0 && self.visit_item_targets(&item, spec.rule, |_| true)
                }
                TargetPlan::Groups(groups) => groups.iter().any(|spec| {
                    spec.max > 0 && self.visit_item_targets(&item, spec.rule, |_| true)
                }),
            };
        if !has_choice {
            // Prepaid declarations settle their deferred costs in the caller.
            // A spell is still only a proposal here, so do not fire cast triggers.
            self.state.stack.push(item);
            return false;
        }
        assert!(self.state.announcing.is_none());
        self.state.announcing = Some(Announcement {
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
            additional_group: None,
            creature_counter_cost: None,
            forage: None,
            evidence: None,
            convoke: None,
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
        if production.is_some() {
            self.capture_announced_mana_quality(&item);
        }
        let plan = self.target_spec(&item);
        assert!(
            self.item_targets_available(&item, plan)
                || item.source_card.gift().is_some_and(|gift| {
                    self.item_targets_available(&item, gift.promised_targets)
                })
                || item
                    .source_card
                    .kicked_targets()
                    .is_some_and(|plan| { self.item_targets_available(&item, plan) })
        );
        let needs_mode = item.modes().is_some() && item.mode.is_none();
        self.state.announcing = Some(Announcement {
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
            additional_group: None,
            creature_counter_cost: None,
            forage: None,
            evidence: None,
            convoke: None,
            additional_paid: false,
            pay_window: true,
            production,
        });
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
            if announcement.item.kind == StackKind::Spell && announcement.total_cost.is_none() {
                let base = Self::announcement_base_cost(announcement);
                let total = if announcement.item.source_card.x_symbols() > 0 {
                    self.scalar_x_total(
                        announcement.item.controller,
                        announcement.item.card,
                        base,
                        announcement.item.x.expect("X chosen"),
                    )
                    .expect("validated scalar cost")
                } else {
                    self.target_spell_cost(
                        announcement.item.controller,
                        announcement.item.card,
                        base,
                        &announcement.item.targets,
                    )
                };
                self.state.announcing.as_mut().unwrap().total_cost = Some(total);
            }
            if self
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
            self.state.pending = if announcement
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
        if let Some(frame) = announcement.convoke.take() {
            self.tap_objects(
                &frame
                    .selected
                    .iter()
                    .map(|s| s.object.card)
                    .collect::<Vec<_>>(),
            );
        }
        announcement
            .item
            .targets
            .sort_by_key(|bound| (bound.group, bound.target));
        // External discard precedes source sacrifice so it captures battlefield
        // observers. External sacrifice follows source tap/mana costs: the chosen
        // object may itself be this ability's source. No priority is granted.
        if matches!(announcement.item.kind, StackKind::ActivatedAbility { .. })
            && !announcement.additional_paid
        {
            if announcement.additional_cost().is_some_and(|spec| {
                matches!(
                    spec.kind,
                    EffectChoiceKind::BlightCost(_)
                        | EffectChoiceKind::Sacrifice
                        | EffectChoiceKind::ReturnToHand
                ) || forage::is_forage_cost(spec)
                    || evidence::threshold(spec).is_some()
            }) && let Some(payment) = announcement.payment.take()
            {
                self.pay_activation(
                    &mut announcement.item,
                    payment,
                    announcement.counter_payment.clone(),
                );
            }
            self.pay_additional_cost(&mut announcement);
            announcement.additional_paid = true;
        }
        if let Some(payment) = announcement.payment {
            if matches!(announcement.item.kind, StackKind::ActivatedAbility { .. }) {
                self.pay_activation(
                    &mut announcement.item,
                    payment,
                    announcement.counter_payment.clone(),
                );
            } else {
                let outcome = self.spend_mana_for(
                    announcement.item.controller,
                    payment,
                    self.purpose_for_item(&announcement.item),
                );
                if outcome.haste {
                    self.grant_stack_keywords(announcement.item.object, Keywords::HASTE);
                }
                announcement.item.mana_copy_sources = outcome.copies;
                announcement.item.cast.as_mut().unwrap().payment = payment;
            }
        }
        // Costs may synchronously return exiled Auras/lands. Finish their entry
        // choices before the spell becomes cast and cast observers are collected.
        announcement.payment = None;
        if self.state.entry_batch.is_some() {
            self.state.announcing = Some(announcement);
            return;
        }
        if announcement.item.kind == StackKind::Spell && !announcement.additional_paid {
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
        } else {
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
