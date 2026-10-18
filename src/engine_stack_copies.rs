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
    index: usize,
    victim: Option<u64>,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct CopyState {
    delayed: Vec<SpellView>,
    retarget: Option<Retarget>,
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
    pub(super) fn stack_target_rule_matches(&self, rule: TargetRule, item: &SpellView) -> bool {
        match rule {
            TargetRule::SingleTargetStackObject => item.targets.len() == 1,
            TargetRule::Spell => item.kind == StackKind::Spell,
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
        let mut delayed = item.clone();
        delayed.kind = StackKind::TriggeredAbility { ability: 0 };
        delayed.cast = None;
        delayed.targets.clear();
        self.state
            .stack_copy_data
            .get_or_insert_with(Default::default)
            .delayed
            .push(delayed);
    }
    pub(super) fn expire_spell_copies(&mut self) {
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
    fn allocate_spell_copy(&mut self, template: &SpellView, controller: Player) -> SpellView {
        // These consumers copy only instant/sorcery spells, which never enter as permanents.
        assert!(crate::cards::CastFilter::InstantSorcery.matches(template.source_card));
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
            copy_token: false,
            spell_copy: true,
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
        copy.copy_event = None;
        copy.cast_event = None;

        copy
    }
    pub(super) fn copy_captured_spell(&mut self, item: &SpellView) -> bool {
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
        self.state
            .stack_copy_data
            .get_or_insert_with(Default::default)
            .retarget = Some(Retarget {
            item: copy,
            template,
            remaining: event.amount - 1,
            index: 0,
            victim: None,
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
            index: 0,
            victim: Some(id),
        });
        self.advance_copy_targets()
    }
    fn new_copy_target_legal(&self, r: &Retarget, target: Target) -> bool {
        let old = r.item.targets[r.index];
        if target == old.target || matches!(target, Target::Stack(id) if id == r.item.id) {
            return false;
        }
        let Some(spec) = self.target_spec(&r.item).group(old.group) else {
            return false;
        };
        self.item_target_matches(&r.item, spec.rule, target)
            && !r
                .item
                .targets
                .iter()
                .enumerate()
                .any(|(i, b)| i != r.index && b.group == old.group && b.target == target)
            && self.target_group_compatible(
                spec.rule,
                old.group,
                &r.item
                    .targets
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| *i != r.index)
                    .map(|(_, b)| *b)
                    .collect::<Vec<_>>(),
                target,
            )
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
        if r.victim.is_none() && emit(Action::KeepCopyTarget) {
            return true;
        }
        let old = r.item.targets[r.index];
        let spec = self.target_spec(&r.item).group(old.group).unwrap();
        self.visit_item_targets(&r.item, spec.rule, |target| {
            self.new_copy_target_legal(r, target) && emit(Action::ChangeCopyTarget { target })
        })
    }
    pub(super) fn finish_copy_target(&mut self, target: Option<Target>) {
        let mut r = self
            .state
            .stack_copy_data
            .as_mut()
            .unwrap()
            .retarget
            .take()
            .unwrap();
        if let Some(target) = target {
            let mut bound = self.bind_target(target);
            bound.group = r.item.targets[r.index].group;
            r.item.targets[r.index] = bound;
        }
        r.index += 1;
        self.state.stack_copy_data.as_mut().unwrap().retarget = Some(r);
        if !self.advance_copy_targets() {
            self.run_program();
        }
    }
    fn advance_copy_targets(&mut self) -> bool {
        loop {
            let r = self
                .state
                .stack_copy_data
                .as_ref()
                .unwrap()
                .retarget
                .as_ref()
                .unwrap();
            if r.index < r.item.targets.len() {
                // Mandatory target changes do nothing if no legal replacement exists.
                let available = r.victim.is_none() || self.visit_retarget_actions(|_| true);
                if available {
                    self.state.pending = Pending::Retarget {
                        player: self.state.resolving.as_ref().unwrap().item.controller,
                    };
                    return true;
                }
            }
            let r = self
                .state
                .stack_copy_data
                .as_mut()
                .unwrap()
                .retarget
                .take()
                .unwrap();
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
                    self.state.stack[index] = r.item;
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
            if r.remaining == 0 {
                self.release_stack_copy_state();
                return false;
            }
            let copy = self.allocate_spell_copy(&r.template, r.item.controller);
            self.state.stack_copy_data.as_mut().unwrap().retarget = Some(Retarget {
                item: copy,
                index: 0,
                remaining: r.remaining - 1,
                ..r
            });
        }
    }
}
