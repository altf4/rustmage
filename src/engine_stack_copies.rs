//! Owned spell-copy packets and incremental changes to existing targets.
use super::*;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CopyEvent {
    pub spell: SpellView,
    pub amount: u64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct Retarget {
    item: SpellView,
    template: SpellView,
    remaining: u64,
    victim: Option<u64>,
    keywords: Keywords,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RetargetView {
    pub item: SpellView,
    pub original: Vec<BoundTarget>,
    pub victim: Option<u64>,
    pub remaining_copies: u64,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct CopyState {
    delayed: Vec<SpellView>,
    retarget: Option<Retarget>,
}
impl CopyState {
    pub(super) fn visit_owned_packets(&mut self, visit: &mut impl FnMut(&mut SpellView)) {
        self.delayed.iter_mut().for_each(&mut *visit);
        if let Some(retarget) = &mut self.retarget {
            visit(&mut retarget.item);
            visit(&mut retarget.template);
        }
    }
}
pub(super) fn owned_bytes(state: &Option<Box<CopyState>>) -> usize {
    state.as_ref().map_or(0, |state| {
        std::mem::size_of::<CopyState>()
            + state
                .delayed
                .iter()
                .map(|item| std::mem::size_of::<SpellView>() + item.owned_bytes())
                .sum::<usize>()
            + state.retarget.as_ref().map_or(0, |r| {
                std::mem::size_of::<Retarget>() + r.item.owned_bytes() + r.template.owned_bytes()
            })
    })
}
impl Game {
    pub(super) fn stack_target_rule_matches(
        &self,
        rule: TargetRule,
        controller: Player,
        item: &SpellView,
    ) -> bool {
        match rule {
            TargetRule::SingleTargetStackObject => item.targets.len() == 1,
            TargetRule::Spell => item.kind == StackKind::Spell,
            TargetRule::TriggeredAbility { relation } => {
                item.kind.is_triggered()
                    && (relation == -1 || ((item.controller == controller) == (relation == 0)))
            }
            TargetRule::SpellKinds { kinds, excluded } => {
                item.kind == StackKind::Spell
                    && (kinds == 0 || item.source_card.kind_mask() & u16::from(kinds) != 0)
                    && item.source_card.kind_mask() & u16::from(excluded) == 0
            }
            TargetRule::SpellCharacteristics {
                colors,
                min_mana,
                max_mana,
            } => {
                item.kind == StackKind::Spell
                    && (colors == 0 || item.source_card.colors() & colors != 0)
                    && (min_mana..=max_mana).contains(&item.mana_value())
            }
            _ => false,
        }
    }
    pub(super) fn capture_stack_copy_departure(&mut self, card: CardId) {
        let Some(spell) = self
            .state
            .stack
            .iter()
            .find(|s| s.kind == StackKind::Spell && s.card == card)
            .cloned()
        else {
            return;
        };
        for item in self
            .state
            .stack
            .iter_mut()
            .chain(self.state.pending_triggers.iter_mut())
            .chain(self.state.ordering_triggers.iter_mut())
        {
            if let Some(event) = &mut item.copy_event
                && event.spell.id == spell.id
            {
                event.spell = spell.clone();
            }
        }
    }
    pub(super) fn bound_stack_item(&self, bound: &BoundTarget) -> Option<&SpellView> {
        self.state.stack.iter().find(|item| match bound.target {
            Target::Stack(id) => item.id == id,
            Target::Spell(card) => {
                item.kind == StackKind::Spell
                    && item.card == card
                    && Some(item.object) == bound.object
            }
            _ => false,
        })
    }
    pub(super) fn remember_next_spell_copy(&mut self, item: &SpellView) {
        self.remember_next_spell_copy_ability(item, 0);
    }
    pub(super) fn remember_next_spell_copy_ability(&mut self, item: &SpellView, ability: u16) {
        let mut delayed = item.clone();
        delayed.kind = StackKind::TriggeredAbility { ability };
        delayed.cast = None;
        delayed.is_copy = false;
        delayed.targets.clear();
        self.state
            .stack_copy_data
            .get_or_insert_with(Default::default)
            .delayed
            .push(delayed);
    }
    /// Abandon only active choices; future delayed copy abilities keep their duration.
    pub(super) fn abandon_copy_continuations(&mut self) {
        let choice = self
            .state
            .stack_copy_data
            .as_mut()
            .and_then(|s| s.retarget.take());
        self.state.nightmare_cull = None;
        if let Some(r) = choice
            && r.victim.is_none()
            && r.item.is_copy
            && r.item.kind == StackKind::Spell
            && self.object_ref(r.item.card) == r.item.object
            && self.state.objects[r.item.card as usize].zone == Zone::Stack
            && !self.state.stack.iter().any(|s| s.id == r.item.id)
        {
            self.move_object(r.item.card, Zone::Exile);
        }
        self.release_stack_copy_state();
    }
    pub(super) fn expire_spell_copies(&mut self) {
        self.abandon_copy_continuations();
        if let Some(state) = &mut self.state.stack_copy_data {
            state.delayed.clear();
        }
        self.release_stack_copy_state();
    }
    fn release_stack_copy_state(&mut self) {
        if self
            .state
            .stack_copy_data
            .as_ref()
            .is_some_and(|s| s.delayed.is_empty() && s.retarget.is_none())
        {
            self.state.stack_copy_data = None;
        }
    }
    pub(super) fn capture_mana_copies(&mut self, index: usize) {
        let Some(sources) = self.state.stack[index].mana_copy_sources.take() else {
            return;
        };
        let spell = self.state.stack[index].clone();
        for (mut source, amount) in *sources {
            if self
                .state
                .objects
                .get(source.object.card as usize)
                .is_some_and(|o| {
                    o.zone == Zone::Battlefield
                        && self.object_ref(source.object.card) == source.object
                })
            {
                let view = self.object_view(source.object.card);
                source.facts = DamageFacts::from_view(&view);
                source.keywords = view.keywords;
                source.tapped = view.tapped;
            }
            for _ in 0..amount {
                let mut trigger = spell.clone();
                trigger.id = self.next_stack_id();
                trigger.kind = StackKind::TriggeredAbility { ability: 0 };
                trigger.card = source.object.card;
                trigger.object = source.object;
                trigger.source_card = source.card;
                trigger.source_facts = source.facts;
                trigger.source_keywords = source.keywords;
                trigger.source_tapped = source.tapped;
                trigger.source_attachment = None;
                trigger.cast = None;
                trigger.mode = None;
                trigger.x = None;
                trigger.gift = None;
                trigger.cast_event = None;
                trigger.source_counter_lki = None;
                trigger.target_controller_lki = None;
                trigger.is_copy = false;
                trigger.targets.clear();
                trigger.controller = source.controller;
                trigger.copy_event = Some(Box::new(CopyEvent {
                    spell: spell.clone(),
                    amount: 1,
                }));
                self.state.pending_triggers.push(trigger);
            }
        }
    }
    pub(super) fn capture_copy_cast(&mut self, stack: Option<usize>, first_trigger: usize) {
        let Some(index) = stack else { return };
        let spell = self.state.stack[index].clone();
        if !crate::cards::CastFilter::InstantSorcery.matches(spell.source_card) {
            return;
        }
        let prior = self.state.players[spell.controller.index()].instant_sorcery_cast_this_turn - 1;
        for trigger in &mut self.state.pending_triggers[first_trigger..] {
            if let Some((_, program)) = trigger.program()
                && let Some(prior_spells) = program.iter().find_map(|i| match i {
                    Instruction::CopyCastSpell { prior_spells } => Some(*prior_spells),
                    Instruction::MayCopyCastSpellWithKeywords(_) => Some(false),
                    _ => None,
                })
            {
                trigger.copy_event = Some(Box::new(CopyEvent {
                    spell: spell.clone(),
                    amount: if prior_spells { prior } else { 1 },
                }));
            }
        }
        let Some(state) = &mut self.state.stack_copy_data else {
            return;
        };
        let mut delayed = std::mem::take(&mut state.delayed);
        let mut retained = Vec::new();
        for mut trigger in delayed.drain(..) {
            if trigger.controller == spell.controller {
                trigger.id = self.next_stack_id();
                trigger.copy_event = Some(Box::new(CopyEvent {
                    spell: spell.clone(),
                    amount: 1,
                }));
                self.state.pending_triggers.push(trigger);
            } else {
                retained.push(trigger);
            }
        }
        self.state.stack_copy_data.as_mut().unwrap().delayed = retained;
        self.release_stack_copy_state();
    }
    fn allocate_stack_copy(&mut self, template: &SpellView, controller: Player) -> SpellView {
        if template.kind == StackKind::Spell {
            return self.allocate_spell_copy(template, controller);
        }
        assert!(template.kind.is_triggered());
        // An ability copy has the same source and frozen event parameters. It
        // never allocates or moves the physical source object (CR707.10b).
        let mut copy = template.clone();
        copy.id = self.next_stack_id();
        copy.controller = controller;
        copy.is_copy = true;
        copy
    }
    pub(super) fn copy_targeted_trigger(&mut self, item: &SpellView) -> bool {
        let Some(template) = item
            .targets
            .first()
            .and_then(|bound| self.bound_stack_item(bound))
            .filter(|target| target.kind.is_triggered())
            .cloned()
        else {
            return false;
        };
        let copy = self.allocate_stack_copy(&template, item.controller);
        self.state
            .stack_copy_data
            .get_or_insert_with(Default::default)
            .retarget = Some(Retarget {
            item: copy,
            template,
            remaining: 0,
            victim: None,
            keywords: Keywords::NONE,
        });
        self.advance_copy_targets()
    }
    fn allocate_spell_copy(&mut self, template: &SpellView, controller: Player) -> SpellView {
        let (card, generation) = if let Some(id) = self.state.free_tokens.pop() {
            (id, self.state.objects[id as usize].generation)
        } else {
            (
                CardId::try_from(self.state.objects.len()).capacity("object slots exhausted"),
                0,
            )
        };
        let object = Object {
            card: template.source_card,
            copy_kind: CopyKind::Spell,
            owner: controller,
            controller,
            zone: Zone::Stack,
            generation,
            tapped: false,
            summoning_sick: false,
            flashback: false,
            damage: 0,
            damaged_this_turn: false,
            dealt_damage_this_turn: false,
            deathtouch_since_sba: false,
            growth: Stats::default(),
            gained_keywords: Keywords::NONE,
            counters: InitialCounters::default(),
            used_abilities: 0,
            attached_to: None,
        };
        if card as usize == self.state.objects.len() {
            self.state.objects.push(object);
        } else {
            self.state.objects[card as usize] = object;
        }
        let mut copy = template.clone();
        copy.id = self.next_stack_id();
        copy.card = card;
        copy.object = self.object_ref(card);
        copy.controller = controller;
        copy.source_facts.controller = controller;
        copy.is_copy = true;
        if let Some(cast) = &mut copy.cast {
            cast.payment = [0; 6];
        }
        copy.copy_event = None;
        copy.mana_copy_sources = None;
        copy.cast_event = None;

        copy
    }
    pub(super) fn copy_captured_spell(&mut self, item: &SpellView) -> bool {
        self.copy_captured_spell_with_keywords(item, Keywords::NONE)
    }
    pub(super) fn copy_captured_spell_with_keywords(
        &mut self,
        item: &SpellView,
        keywords: Keywords,
    ) -> bool {
        let Some(event) = &item.copy_event else {
            return false;
        };
        if event.amount == 0 {
            return false;
        }
        let template = self
            .state
            .stack
            .iter()
            .find(|s| s.id == event.spell.id)
            .cloned()
            .unwrap_or_else(|| event.spell.clone());
        let copy = self.allocate_spell_copy(&template, item.controller);
        // Ability grants are continuous effects, not copiable characteristics.
        // A different effect copying the original must not inherit this grant.
        if keywords != Keywords::NONE {
            if self
                .state
                .stack
                .iter()
                .any(|spell| spell.id == template.id && spell.object == template.object)
            {
                self.grant_stack_keywords_expiring(template.object, item.object, keywords, None);
            }
            self.grant_stack_keywords_expiring(copy.object, item.object, keywords, None);
        }
        self.state
            .stack_copy_data
            .get_or_insert_with(Default::default)
            .retarget = Some(Retarget {
            item: copy,
            template,
            remaining: event.amount - 1,
            victim: None,
            keywords,
        });
        self.advance_copy_targets()
    }
    pub(super) fn change_single_stack_target(&mut self, item: &SpellView) -> bool {
        let Some(victim) = item
            .targets
            .first()
            .and_then(|b| self.bound_stack_item(b))
            .filter(|v| v.targets.len() == 1)
            .cloned()
        else {
            return false;
        };
        let id = victim.id;
        self.state
            .stack_copy_data
            .get_or_insert_with(Default::default)
            .retarget = Some(Retarget {
            item: victim.clone(),
            template: victim,
            remaining: 0,
            victim: Some(id),
            keywords: Keywords::NONE,
        });
        self.advance_copy_targets()
    }
    pub(super) fn delayed_spell_copy_view(&self) -> Vec<SpellView> {
        self.state
            .stack_copy_data
            .as_ref()
            .map_or_else(Vec::new, |s| s.delayed.clone())
    }
    pub(super) fn retarget_view(&self) -> Option<RetargetView> {
        let r = self.state.stack_copy_data.as_ref()?.retarget.as_ref()?;
        Some(RetargetView {
            item: r.item.clone(),
            original: r.template.targets.clone(),
            victim: r.victim,
            remaining_copies: r.remaining,
        })
    }
    fn new_stack_target_legal(&self, r: &Retarget, index: usize, target: Target) -> bool {
        let old = r.item.targets[index];
        if self.bind_target(target) == (BoundTarget { group: 0, ..old })
            || matches!(target, Target::Stack(id) if id == r.item.id)
        {
            return false;
        }
        let Some(spec) = self.target_spec(&r.item).group(old.group) else {
            return false;
        };
        self.item_target_matches(&r.item, spec.rule, target)
    }
    fn stack_target_relations_hold(&self, item: &SpellView, index: usize) -> bool {
        let bound = item.targets[index];
        let plan = self.target_spec(item);
        let spec = plan.group(bound.group).unwrap();
        if matches!(
            spec.rule,
            TargetRule::Grouped {
                constraint: TargetGroupConstraint::SameGraveyard
                    | TargetGroupConstraint::SharedCreatureType
                    | TargetGroupConstraint::OtherThanGroup(_),
                ..
            }
        ) {
            self.target_group_compatible(
                spec.rule,
                bound.group,
                &item
                    .targets
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| *i != index)
                    .map(|(_, b)| *b)
                    .collect::<Vec<_>>(),
                bound.target,
            )
        } else {
            self.resolution_target_relations_hold(item, bound, plan)
        }
    }
    fn stack_targets_finished(&self, r: &Retarget) -> bool {
        let changed = r
            .item
            .targets
            .iter()
            .zip(&r.template.targets)
            .any(|(a, b)| a != b);
        if r.victim.is_some() && !changed {
            return false;
        }
        let plan = self.target_spec(&r.item);
        r.item.targets.iter().enumerate().all(|(index, bound)| {
            let spec = plan.group(bound.group).unwrap();
            if *bound == r.template.targets[index] {
                // An inherited illegal target may remain. A new choice cannot
                // invalidate an unchanged target that was legal before the edits.
                let was_legal = self.same_target_object(*bound)
                    && self.item_target_matches(&r.template, spec.rule, bound.target)
                    && self.stack_target_relations_hold(&r.template, index);
                return !was_legal || self.stack_target_relations_hold(&r.item, index);
            }
            self.item_target_matches(&r.item, spec.rule, bound.target)
                && !r.item.targets.iter().enumerate().any(|(i, other)| {
                    i != index
                        && other.group == bound.group
                        && other.target == bound.target
                        && other.object == bound.object
                })
                && self.stack_target_relations_hold(&r.item, index)
        })
    }
    pub(super) fn visit_retarget_actions(&self, mut emit: impl FnMut(Action) -> bool) -> bool {
        let r = self
            .state
            .stack_copy_data
            .as_ref()
            .unwrap()
            .retarget
            .as_ref()
            .unwrap();
        if self.stack_targets_finished(r) && emit(Action::FinishStackTargets) {
            return true;
        }
        for (index, old) in r.item.targets.iter().enumerate() {
            // Restoring an inherited target is allowed even if it has become illegal.
            if r.template.targets[index] != *old
                && emit(Action::RestoreStackTarget {
                    index: index as u32,
                })
            {
                return true;
            }
            let spec = self.target_spec(&r.item).group(old.group).unwrap();
            if self.visit_item_targets(&r.item, spec.rule, |target| {
                self.new_stack_target_legal(r, index, target)
                    && emit(Action::SetStackTarget {
                        index: index as u32,
                        target,
                    })
            }) {
                return true;
            }
        }
        false
    }
    pub(super) fn set_stack_target(&mut self, index: u32, target: Target) {
        let index = index as usize;
        let mut bound = self.bind_target(target);
        let r = self
            .state
            .stack_copy_data
            .as_mut()
            .unwrap()
            .retarget
            .as_mut()
            .unwrap();
        bound.group = r.item.targets[index].group;
        r.item.retarget_allocation(index, bound);
        r.item.targets[index] = bound;
    }
    pub(super) fn restore_stack_target(&mut self, index: u32) {
        let r = self
            .state
            .stack_copy_data
            .as_mut()
            .unwrap()
            .retarget
            .as_mut()
            .unwrap();
        let index = index as usize;
        let restored = r.template.targets[index];
        r.item.retarget_allocation(index, restored);
        r.item.targets[index] = restored;
    }
    pub(super) fn finish_stack_targets(&mut self) {
        self.commit_stack_targets();
        if !self.advance_copy_targets() {
            self.run_program();
        }
    }
    fn commit_stack_targets(&mut self) {
        let r = self
            .state
            .stack_copy_data
            .as_ref()
            .unwrap()
            .retarget
            .as_ref()
            .unwrap()
            .clone();
        if let Some(id) = r.victim {
            if let Some(index) = self.state.stack.iter().position(|s| s.id == id) {
                let previous = self.state.stack[index].clone();
                for bound in &r.item.targets {
                    if !previous
                        .targets
                        .iter()
                        .any(|old| old.target == bound.target)
                    {
                        self.target_becomes_bound(&previous, *bound);
                    }
                }
                self.state.stack[index] = r.item.clone();
            }
        } else {
            let mut empty = r.item.clone();
            empty.targets.clear();
            for bound in &r.item.targets {
                self.target_becomes_bound(&empty, *bound);
                empty.targets.push(*bound);
            }
            self.state.stack.push(r.item.clone());
        }
        let state = self.state.stack_copy_data.as_mut().unwrap();
        if r.remaining == 0 {
            state.retarget = None;
        } else {
            let copy = self.allocate_stack_copy(&r.template, r.item.controller);
            if r.keywords != Keywords::NONE {
                self.grant_stack_keywords_expiring(
                    copy.object,
                    r.template.object,
                    r.keywords,
                    None,
                );
            }
            self.state.stack_copy_data.as_mut().unwrap().retarget = Some(Retarget {
                item: copy,
                remaining: r.remaining - 1,
                ..r
            });
        }
    }
    fn advance_copy_targets(&mut self) -> bool {
        loop {
            let Some(r) = self
                .state
                .stack_copy_data
                .as_ref()
                .and_then(|s| s.retarget.as_ref())
            else {
                self.release_stack_copy_state();
                return false;
            };
            if !r.item.targets.is_empty()
                && (r.victim.is_none() || self.visit_retarget_actions(|_| true))
            {
                self.state.pending = Pending::Retarget {
                    player: self.state.resolving.as_ref().unwrap().item.controller,
                };
                return true;
            }
            // A mandatory retarget with no alternatives leaves its target unchanged.
            self.commit_stack_targets();
        }
    }
}

#[cfg(test)]
mod allocation_occurrence_tests {
    use super::super::ability_timeline_tests::{cast, checked, named, settle, setup};
    use super::*;

    #[test]
    fn allocation_occurrences_survive_duplicate_swap_and_restore_with_owned_edits() {
        for p in [Player::P0, Player::P1] {
            let mut g = setup(p);
            let chandra = named(&mut g, p, "Chandra, Flameshaper", Zone::Hand);
            cast(&mut g, chandra, None, &[]);
            settle(&mut g);
            let a = named(&mut g, p.other(), "Serra Angel", Zone::Battlefield);
            let b = named(&mut g, p.other(), "Zetalpa, Primal Dawn", Zone::Battlefield);
            checked(
                &mut g,
                Action::AnnounceActivate {
                    card: chandra,
                    ability: 2,
                    counter_payment: Counters::default(),
                },
            );
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(a),
                },
            );
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(b),
                },
            );
            checked(&mut g, Action::FinishTargets);
            checked(&mut g, Action::AllocateTargetAmount { amount: 3 });
            checked(&mut g, Action::AllocateTargetAmount { amount: 5 });
            // Allocation precedes the ordinary activation-cost commit, even
            // when the loyalty ability has no mana component.
            while g.state.announcing.is_some() {
                let action = g
                    .legal_actions()
                    .into_iter()
                    .find(|a| matches!(a, Action::PayCosts { .. }))
                    .unwrap();
                checked(&mut g, action);
            }
            // Seed a generic retarget continuation around the actual paid packet.
            // Multi-target editing is generic; Bolt Bend's one-target restriction
            // cannot supply this multi-target fixture.
            let item = g.state.stack.last().unwrap().clone();
            g.state
                .stack_copy_data
                .get_or_insert_with(Default::default)
                .retarget = Some(Retarget {
                template: item.clone(),
                victim: Some(item.id),
                item,
                remaining: 0,
                keywords: Keywords::NONE,
            });
            g.state.pending = Pending::Retarget { player: p };
            let expect = |g: &Game, targets: [CardId; 2]| {
                let r = g.retarget_view().unwrap();
                let rows = r.item.target_allocations.unwrap();
                assert_eq!(rows.iter().map(|v| v.amount).collect::<Vec<_>>(), [3, 5]);
                assert_eq!(
                    rows.iter().map(|v| v.target.target).collect::<Vec<_>>(),
                    targets.map(Target::Card)
                );
                assert_eq!(
                    r.item.targets.iter().map(|v| v.target).collect::<Vec<_>>(),
                    targets.map(Target::Card)
                );
            };
            checked(
                &mut g,
                Action::SetStackTarget {
                    index: 0,
                    target: Target::Card(b),
                },
            );
            expect(&g, [b, b]);
            checked(
                &mut g,
                Action::SetStackTarget {
                    index: 1,
                    target: Target::Card(a),
                },
            );
            expect(&g, [b, a]);
            checked(&mut g, Action::RestoreStackTarget { index: 0 });
            expect(&g, [a, a]);
            checked(&mut g, Action::RestoreStackTarget { index: 1 });
            expect(&g, [a, b]);
            checked(
                &mut g,
                Action::SetStackTarget {
                    index: 0,
                    target: Target::Card(b),
                },
            );
            checked(
                &mut g,
                Action::SetStackTarget {
                    index: 1,
                    target: Target::Card(a),
                },
            );
            expect(&g, [b, a]);
            g.commit_stack_targets();
            g.state.pending = Pending::Priority;
            settle(&mut g);
            assert_eq!(g.object_view(a).zone, Zone::Graveyard);
            assert_eq!(g.object_view(b).damage, 3);
        }
    }
}
