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
    pub forage: Option<Box<forage::ForageSelection>>,
    pub evidence: Option<Box<evidence::EvidenceSelection>>,
    pub convoke: Option<Box<convoke::ConvokeSelection>>,
    pub additional_paid: bool,
}

impl Announcement {
    pub(super) fn additional_cost(&self) -> Option<AdditionalCost> {
        match self.item.kind {
            StackKind::Spell if self.item.cast.is_some_and(|c| c.evidence == Some(true)) => Some(
                evidence::cost(self.item.source_card.collect_evidence_cost().unwrap()),
            ),
            StackKind::Spell if self.item.cast.is_some_and(|c| c.bargain == Some(true)) => {
                Some(bargain::COST)
            }
            StackKind::Spell => self.item.source_card.additional_cost(),
            StackKind::ActivatedAbility { ability } => {
                self.item.source_card.activations()[ability as usize]
                    .cost
                    .additional
            }
            StackKind::TriggeredAbility { .. }
            | StackKind::DelayedSacrifice
            | StackKind::DelayedWarpExile => None,
        }
    }

    pub(super) fn cost_options(&self) -> crate::cards::ManaCostOptions {
        match self.item.kind {
            StackKind::Spell if self.item.cast.is_some_and(|c| c.method == CastMethod::Warp) => {
                std::iter::once(self.item.source_card.warp_cost().expect("fixed Warp cost"))
                    .chain([].iter().copied())
            }
            StackKind::Spell
                if self
                    .item
                    .cast
                    .is_some_and(|cast| cast.method == CastMethod::Flashback) =>
            {
                std::iter::once(self.item.source_card.flashback_cost().unwrap())
                    .chain([].iter().copied())
            }
            StackKind::Spell if self.item.cast.is_some_and(|c| c.method == CastMethod::Plot) => {
                std::iter::once(ManaCost {
                    colored: [0; 6],
                    generic: 0,
                })
                .chain([].iter().copied())
            }
            StackKind::Spell => self.item.source_card.cost_options(),
            StackKind::ActivatedAbility { ability } => self.item.source_card.activations()
                [ability as usize]
                .cost
                .mana_options(),
            StackKind::TriggeredAbility { .. }
            | StackKind::DelayedSacrifice
            | StackKind::DelayedWarpExile => {
                unreachable!("triggered ability has no activation cost")
            }
        }
    }
    pub(super) fn needs_evidence(&self) -> bool {
        self.pay_window
            && self.item.kind == StackKind::Spell
            && self.item.source_card.collect_evidence_cost().is_some()
            && self.item.cast.is_some_and(|c| c.evidence.is_none())
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
                            CastMethod::Flashback | CastMethod::Plot | CastMethod::Warp
                        )
                    }) && self.item.source_card.has_hybrid_cost()
                }
                StackKind::ActivatedAbility { ability } => !self.item.source_card.activations()
                    [ability as usize]
                    .cost
                    .alternatives
                    .is_empty(),
                StackKind::TriggeredAbility { .. }
                | StackKind::DelayedSacrifice
                | StackKind::DelayedWarpExile => false,
            }
    }
}

impl Game {
    pub(super) fn target_spec(&self, item: &SpellView) -> TargetPlan {
        item.program()
            .map(|(plan, _)| plan)
            .unwrap_or(TargetPlan::NONE)
    }

    // A spell will leave its origin before targets are chosen (CR601.2a/c).
    // In particular, it cannot target its own soon-to-depart graveyard card.
    pub(super) fn cast_targets_available(
        &self,
        plan: TargetPlan,
        actor: Player,
        source: ObjectRef,
    ) -> bool {
        let available = |spec: TargetSpec| {
            if spec.min == 0 {
                return true;
            }
            let mut count = 0;
            self.visit_cast_targets(spec.rule, actor, source, |target| {
                if target == Target::Card(source.card) {
                    return false;
                }
                count += 1;
                count >= usize::from(spec.min)
            })
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
        let available = |spec: TargetSpec| {
            if spec.min == 0 {
                return true;
            }
            let mut count = 0;
            self.visit_targets(spec.rule, actor, source, |_| {
                count += 1;
                count >= usize::from(spec.min)
            })
        };
        match plan {
            TargetPlan::Single(spec) => available(spec),
            TargetPlan::Groups(groups) => groups.iter().copied().all(available),
        }
    }

    pub(super) fn choose_mana_cost(&mut self, index: u16) {
        let a = self.state.announcing.as_mut().unwrap();
        a.cost = Some(a.cost_options().nth(index as usize).unwrap());
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
        if needs_mode {
            self.state.pending = Pending::Modes { player };
        } else if a.pay_window
            && a.item.kind == StackKind::Spell
            && a.item.source_card.x_symbols() > 0
            && a.item.x.is_none()
        {
            self.state.pending = Pending::X { player };
        } else if a.needs_offspring() {
            self.state.pending = Pending::Offspring { player };
        } else if a.needs_evidence() {
            self.state.pending = Pending::EvidenceIntent { player };
        } else if a.needs_bargain() {
            self.state.pending = Pending::Bargain { player };
        } else if a.needs_kicker() {
            self.state.pending = Pending::Kicker { player };
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
            if spec.max > 0 && self.visit_targets(spec.rule, item.controller, item.object, |_| true)
            {
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
                Target::Player(_) => None,
                Target::Card(id) | Target::Spell(id) => Some(self.object_ref(id)),
            },
        }
    }

    pub(super) fn target_matches(
        &self,
        rule: TargetRule,
        controller: Player,
        source: ObjectRef,
        target: Target,
    ) -> bool {
        self.target_matches_without_hand(rule, controller, source, target, None)
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
        self.target_matches_without_hand(rule, controller, source, target, departing)
    }
    fn target_matches_without_hand(
        &self,
        rule: TargetRule,
        controller: Player,
        source: ObjectRef,
        target: Target,
        departing: Option<CardId>,
    ) -> bool {
        match rule {
            TargetRule::Union(rules) => {
                return rules.iter().any(|&rule| {
                    self.target_matches_without_hand(rule, controller, source, target, departing)
                });
            }
            TargetRule::Filtered { base, predicate } => {
                if !self.target_matches_without_hand(*base, controller, source, target, departing) {
                    return false;
                }
                let Target::Card(id) = target else {
                    return false;
                };
                // The base rule validates the object index and zone before this lookup.
                let object = &self.state.objects[id as usize];
                return match *predicate {
                    TargetPredicate::Blocking => self.state.blocking.contains(&id),
                    TargetPredicate::Nontoken => !object.card.is_token(),
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
                    TargetPredicate::ToughnessAtLeast(n) => {
                        self.stats_without_hand(id, departing).toughness >= i128::from(n)
                    }
                    TargetPredicate::PowerOrToughnessAtLeast(n) => {
                        let stats = self.stats_without_hand(id, departing);
                        stats.power >= i128::from(n) || stats.toughness >= i128::from(n)
                    }
                    TargetPredicate::Colors(mask) => object.card.colors() & mask != 0,
                    TargetPredicate::Keyword(keyword) => self.keywords(id).contains(keyword),
                    TargetPredicate::Token => object.card.is_token(),
                    TargetPredicate::Subtype(subtype) => self.has_subtype(id, subtype),
                    TargetPredicate::Permanent => {
                        object.card.kind_mask() & (1 | 2 | 16 | 32 | 64 | 256) != 0
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
            Target::Player(player) => match rule {
                TargetRule::Any => true,
                TargetRule::Player { opponent_only } => !opponent_only || player != controller,
                _ => false,
            },
            Target::Spell(id) => self.state.stack.iter().any(|item| {
                item.kind == StackKind::Spell
                    && item.card == id
                    && item.object != source
                    && match rule {
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
                    && obj.owner != controller
                    && self.keywords(id).contains(Keywords::HEXPROOF)
                {
                    return false;
                }
                match rule {
                    TargetRule::CreatureOrVehicle { relation } => {
                        obj.zone == Zone::Battlefield
                            && (obj.card.is_creature() || obj.card.has_subtype("Vehicle"))
                            && (relation == -1 || (obj.owner == controller) == (relation == 0))
                    }
                    TargetRule::Creature => obj.zone == Zone::Battlefield && obj.card.is_creature(),
                    TargetRule::Any => {
                        obj.zone == Zone::Battlefield
                            && (obj.card.is_creature() || obj.card.is_planeswalker())
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
                            && (expected != Zone::Graveyard || !obj.card.is_token())
                            && (kinds == 0 || obj.card.kind_mask() & u16::from(kinds) != 0)
                            && (relation == -1 || (obj.owner == controller) == (relation == 0))
                            && (!other || self.object_ref(id) != source)
                            && (!nonland || !obj.card.is_land())
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
            Target::Player(_) => target.object.is_none(),
            Target::Card(id) | Target::Spell(id) => self
                .state
                .objects
                .get(id as usize)
                .is_some_and(|_| target.object == Some(self.object_ref(id))),
        }
    }

    pub(super) fn visit_targets(
        &self,
        rule: TargetRule,
        controller: Player,
        source: ObjectRef,
        emit: impl FnMut(Target) -> bool,
    ) -> bool {
        self.visit_targets_without_hand(rule, controller, source, None, emit)
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
        self.visit_targets_without_hand(rule, controller, source, departing, emit)
    }
    fn visit_targets_without_hand(
        &self,
        rule: TargetRule,
        controller: Player,
        source: ObjectRef,
        departing: Option<CardId>,
        mut emit: impl FnMut(Target) -> bool,
    ) -> bool {
        if rule == TargetRule::None {
            return false;
        }
        for player in [Player::P0, Player::P1] {
            let target = Target::Player(player);
            if self.target_matches_without_hand(rule, controller, source, target, departing)
                && emit(target)
            {
                return true;
            }
        }
        for id in 0..self.state.objects.len() {
            let target = Target::Card(id as CardId);
            if self.target_matches_without_hand(rule, controller, source, target, departing)
                && emit(target)
            {
                return true;
            }
        }
        for item in &self.state.stack {
            if item.kind == StackKind::Spell {
                let target = Target::Spell(item.card);
                if self.target_matches_without_hand(rule, controller, source, target, departing)
                    && emit(target)
                {
                    return true;
                }
            }
        }
        false
    }

    pub(super) fn cast_item(&mut self, actor: Player, card: CardId, payment: Mana) -> SpellView {
        self.spend_mana(actor, payment);
        self.state.passes = 0;
        let origin = self.state.objects[card as usize].zone;
        let object = self.move_object(card, Zone::Stack).after;
        let id = self.next_stack_id();
        SpellView {
            delayed_sacrifice: None,
            delayed_warp_exile: None,
            x: None,
            attachment_trigger: None,
            cast: Some(CastInfo {
                origin,
                method: CastMethod::Normal,
                offspring: None,
                kicker: None,
                bargain: None,
                evidence: None,
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
            discard_event: None,
            draw_event: None,
            life_event: None,
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
                .any(|mode| self.targets_available(mode.targets, item.controller, item.object))
        {
            assert!(trigger, "validated modal spell");
            return false;
        }
        let plan = self.target_spec(&item);
        if !self.targets_available(plan, item.controller, item.object) {
            assert!(trigger, "validated spell announcement");
            return false;
        }
        let has_choice = needs_mode
            || match plan {
                TargetPlan::Single(spec) => {
                    spec.max > 0
                        && self.visit_targets(spec.rule, item.controller, item.object, |_| true)
                }
                TargetPlan::Groups(groups) => groups.iter().any(|spec| {
                    spec.max > 0
                        && self.visit_targets(spec.rule, item.controller, item.object, |_| true)
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
        let plan = self.target_spec(&item);
        let player = item.controller;
        assert!(self.targets_available(plan, player, item.object));
        let needs_mode = item.kind == StackKind::Spell
            && matches!(item.source_card.effect(), Effect::Modal(_))
            && item.mode.is_none();
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
                spec.kind == EffectChoiceKind::Sacrifice
                    || forage::is_forage_cost(spec)
                    || evidence::threshold(spec).is_some()
            }) && let Some(payment) = announcement.payment.take()
            {
                self.pay_activation(&announcement.item, payment, announcement.counter_payment);
            }
            self.pay_additional_cost(&mut announcement);
            announcement.additional_paid = true;
        }
        if let Some(payment) = announcement.payment {
            if matches!(announcement.item.kind, StackKind::ActivatedAbility { .. }) {
                self.pay_activation(&announcement.item, payment, announcement.counter_payment);
            } else {
                self.spend_mana(announcement.item.controller, payment);
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
            self.add_mana(announcement.item.controller, production);
        } else {
            self.push_stack_item(announcement.item);
        }
        if announcement.trigger {
            self.continue_trigger_order();
        } else {
            self.state.pending = Pending::Priority;
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
                .map(|&id| self.entry_move(id, destination, self.enters_tapped(id)))
                .collect();
            self.stage_entries(moves, None);
        } else {
            let events = self.move_objects_raw(&ids, destination);
            self.finish_zone_events(&events, &sources);
        }
    }
}
