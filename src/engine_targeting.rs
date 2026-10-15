//! Shared target predicates, binding, and owned incremental announcements.
use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Announcement {
    pub item: SpellView,
    pub spec: TargetSpec,
    pub trigger: bool,
    pub payment: Option<Mana>,
    pub counter_payment: Counters,
    pub pay_window: bool,
    pub production: Option<Mana>,
}

impl Game {
    pub(super) fn target_spec(&self, item: &SpellView) -> TargetSpec {
        match item.kind {
            StackKind::TriggeredAbility { ability } => {
                item.source_card.triggers()[ability as usize].targets
            }
            StackKind::ActivatedAbility { ability } => {
                item.source_card.activations()[ability as usize].targets
            }
            StackKind::Spell => item
                .source_card
                .spell_program(item.mode)
                .map(|(spec, _)| spec)
                .unwrap_or(TargetSpec::NONE),
        }
    }

    pub(super) fn targets_available(
        &self,
        spec: TargetSpec,
        actor: Player,
        source: ObjectRef,
    ) -> bool {
        if spec.min == 0 {
            return true;
        }
        let mut count = 0;
        self.visit_targets(spec.rule, actor, source, |_| {
            count += 1;
            count >= usize::from(spec.min)
        })
    }

    pub(super) fn choose_mode(&mut self, mode: u16) {
        let announcement = self.state.announcing.as_mut().unwrap();
        announcement.item.mode = Some(mode);
        announcement.spec = announcement
            .item
            .source_card
            .spell_program(Some(mode))
            .unwrap()
            .0;
        let item = announcement.item.clone();
        let spec = announcement.spec;
        self.state.pending = if spec.max == 0
            || !self.visit_targets(spec.rule, item.controller, item.object, |_| true)
        {
            Pending::PayCosts {
                player: item.controller,
            }
        } else {
            Pending::Targets {
                player: item.controller,
            }
        };
    }

    pub(super) fn bind_target(&self, target: Target) -> BoundTarget {
        BoundTarget {
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
        match target {
            Target::Player(player) => match rule {
                TargetRule::Any => true,
                TargetRule::Player { opponent_only } => !opponent_only || player != controller,
                _ => false,
            },
            Target::Spell(id) => {
                rule == TargetRule::Spell
                    && self.state.stack.iter().any(|item| {
                        item.kind == StackKind::Spell && item.card == id && item.object != source
                    })
            }
            Target::Card(id) => {
                let Some(obj) = self.state.objects.get(id as usize) else {
                    return false;
                };
                if obj.zone == Zone::Battlefield
                    && obj.owner != controller
                    && obj.keywords().contains(Keywords::HEXPROOF)
                {
                    return false;
                }
                match rule {
                    TargetRule::Creature | TargetRule::Any => {
                        obj.zone == Zone::Battlefield && obj.card.is_creature()
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
                            && (kinds == 0 || obj.card.kind_mask() & kinds != 0)
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
        mut emit: impl FnMut(Target) -> bool,
    ) -> bool {
        if rule == TargetRule::None {
            return false;
        }
        for player in [Player::P0, Player::P1] {
            let target = Target::Player(player);
            if self.target_matches(rule, controller, source, target) && emit(target) {
                return true;
            }
        }
        for id in 0..self.state.objects.len() {
            let target = Target::Card(id as CardId);
            if self.target_matches(rule, controller, source, target) && emit(target) {
                return true;
            }
        }
        for item in &self.state.stack {
            if item.kind == StackKind::Spell {
                let target = Target::Spell(item.card);
                if self.target_matches(rule, controller, source, target) && emit(target) {
                    return true;
                }
            }
        }
        false
    }

    pub(super) fn cast_item(&mut self, actor: Player, card: CardId, payment: Mana) -> SpellView {
        for (pool, cost) in self.state.players[actor.index()]
            .mana
            .iter_mut()
            .zip(payment)
        {
            *pool -= cost;
        }
        self.state.passes = 0;
        let object = self.move_object(card, Zone::Stack).after;
        let id = self.next_stack_id();
        SpellView {
            id,
            kind: StackKind::Spell,
            source_card: self.state.objects[card as usize].card,
            source_keywords: self.state.objects[card as usize].keywords(),
            card,
            object,
            controller: actor,
            targets: Vec::new(),
            mode: None,
        }
    }

    /// Returns true only when a decision suspends the enclosing ordering/cast work.
    pub(super) fn start_announcement(&mut self, item: SpellView, trigger: bool) -> bool {
        let spec = self.target_spec(&item);
        if spec.max == 0 {
            self.state.stack.push(item);
            return false;
        }
        let mut count = 0;
        self.visit_targets(spec.rule, item.controller, item.object, |_| {
            count += 1;
            false
        });
        if count < usize::from(spec.min) {
            assert!(trigger, "validated spell announcement");
            return false;
        } // Impossible trigger announcement: remove it.
        if spec.max == 0 || count == 0 {
            self.state.stack.push(item);
            return false;
        }
        let player = item.controller;
        assert!(self.state.announcing.is_none());
        self.state.announcing = Some(Announcement {
            item,
            spec,
            trigger,
            payment: None,
            counter_payment: Counters::default(),
            pay_window: false,
            production: None,
        });
        self.state.pending = Pending::Targets { player };
        true
    }

    pub(super) fn open_payment_announcement(
        &mut self,
        item: SpellView,
        counters: Counters,
        production: Option<Mana>,
    ) {
        let spec = self.target_spec(&item);
        let player = item.controller;
        let mut count = 0;
        if spec.max > 0 {
            self.visit_targets(spec.rule, player, item.object, |_| {
                count += 1;
                false
            });
        }
        assert!(count >= usize::from(spec.min));
        let needs_mode = item.kind == StackKind::Spell
            && matches!(item.source_card.effect(), Effect::Modal(_))
            && item.mode.is_none();
        self.state.announcing = Some(Announcement {
            item,
            spec,
            trigger: false,
            payment: None,
            counter_payment: counters,
            pay_window: true,
            production,
        });
        self.state.pending = if needs_mode {
            Pending::Modes { player }
        } else if spec.max == 0 || count == 0 {
            Pending::PayCosts { player }
        } else {
            Pending::Targets { player }
        };
    }

    pub(super) fn finish_announcement(&mut self) {
        if let Some(announcement) = &self.state.announcing
            && announcement.pay_window
            && matches!(self.state.pending, Pending::Targets { .. })
        {
            self.state.pending = Pending::PayCosts {
                player: announcement.item.controller,
            };
            return;
        }
        let mut announcement = self.state.announcing.take().unwrap();
        announcement.item.targets.sort_by_key(|bound| bound.target);
        if let Some(payment) = announcement.payment {
            if matches!(announcement.item.kind, StackKind::ActivatedAbility { .. }) {
                self.pay_activation(&announcement.item, payment, announcement.counter_payment);
            } else {
                for (pool, cost) in self.state.players[announcement.item.controller.index()]
                    .mana
                    .iter_mut()
                    .zip(payment)
                {
                    *pool -= cost;
                }
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
            self.state.stack.push(announcement.item);
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
        let events = self.move_objects_raw(&ids, destination);
        self.finish_zone_events(&events, &sources);
    }
}
