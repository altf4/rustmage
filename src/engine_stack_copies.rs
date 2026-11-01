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
    victim: Option<(u64, bool)>,
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
    #[cfg(test)]
    pub(super) fn test_retarget(item: SpellView, template: SpellView) -> Self {
        Self {
            delayed: Vec::new(),
            retarget: Some(Retarget {
                item,
                template,
                remaining: 0,
                victim: None,
            }),
        }
    }
    #[cfg(test)]
    pub(super) fn packets(&self) -> impl Iterator<Item = &SpellView> {
        self.delayed
            .iter()
            .chain(self.retarget.iter().flat_map(|r| [&r.item, &r.template]))
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
fn delayed_copy_reads_source_power(item: &SpellView) -> bool {
    matches!(
        item.kind,
        StackKind::TriggeredAbility { ability }
            if item.trigger_definition().triggers()[usize::from(ability)]
                .condition.event() == TriggerCondition::DelayedSpellCastSourcePower
    )
}
fn delayed_copy_all_spells(item: &SpellView) -> bool {
    matches!(
        item.kind,
        StackKind::TriggeredAbility { ability }
            if matches!(
                item.trigger_definition().triggers()[usize::from(ability)].condition,
                TriggerCondition::Granted(&TriggerCondition::SpellCast {
                    relation: CastRelation::You,
                    filter: crate::cards::CastFilter::Any,
                    ordinal: None,
                })
            )
    )
}
impl Game {
    /// Retain the final characteristics of an activated ability for triggers
    /// that refer to it after it leaves the stack. These packets are owned, so
    /// rollback and copied triggers keep the same last-known targets.
    pub(super) fn retain_departing_activation_for_triggers(&mut self, departed: &SpellView) {
        if !matches!(departed.kind, StackKind::ActivatedAbility { .. }) {
            return;
        }
        let retain = |packet: &mut SpellView| {
            if let Some(event) = packet.copy_event.as_mut()
                && event.spell.id == departed.id
            {
                event.spell = departed.clone();
            }
        };
        for packet in self
            .state
            .stack
            .iter_mut()
            .chain(&mut self.state.pending_triggers)
            .chain(&mut self.state.ordering_triggers)
        {
            retain(packet);
        }
        if let Some(copies) = self.state.stack_copy_data.as_mut() {
            for packet in &mut copies.delayed {
                retain(packet);
            }
            if let Some(retarget) = &mut copies.retarget {
                retain(&mut retarget.item);
                retain(&mut retarget.template);
            }
        }
    }
    pub(super) fn stack_target_rule_matches(
        &self,
        rule: TargetRule,
        item: &SpellView,
        controller: Player,
    ) -> bool {
        match rule {
            TargetRule::YourAbilityFromSourceKinds { kinds } => {
                item.kind != StackKind::Spell
                    && item.controller == controller
                    && self.ability_source_kinds(item) & kinds != 0
            }
            TargetRule::SingleTargetStackObject => item.targets.len() == 1,
            TargetRule::Spell => item.kind == StackKind::Spell,
            TargetRule::YourSpellKinds { kinds } => {
                item.kind == StackKind::Spell
                    && item.controller == controller
                    && item.source.expect("spell source exists").card.kind_mask() & u16::from(kinds)
                        != 0
            }
            TargetRule::CreatureOrLegendarySpell => {
                item.kind == StackKind::Spell
                    && (item.source.expect("spell source exists").card.kind_mask() & 1 != 0
                        || item
                            .source
                            .expect("spell source exists")
                            .card
                            .is_legendary())
            }
            TargetRule::SpellKinds { kinds, excluded } => {
                item.kind == StackKind::Spell
                    && (kinds == 0
                        || item.source.expect("spell source exists").card.kind_mask()
                            & u16::from(kinds)
                            != 0)
                    && item.source.expect("spell source exists").card.kind_mask()
                        & u16::from(excluded)
                        == 0
            }
            TargetRule::SpellCharacteristics {
                colors,
                min_mana,
                max_mana,
            } => {
                item.kind == StackKind::Spell
                    && (colors == 0
                        || item.source.expect("spell source exists").card.colors() & colors != 0)
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
            .find(|s| {
                s.kind == StackKind::Spell && s.source.is_some_and(|source| source.id == card)
            })
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
                    && item.source.is_some_and(|source| {
                        source.id == card && Some(source.object) == bound.object
                    })
            }
            _ => false,
        })
    }
    pub(super) fn remember_next_spell_copy(&mut self, item: &SpellView, ability: u16) {
        let mut delayed = item.clone();
        delayed.kind = StackKind::TriggeredAbility { ability };
        delayed.cast = None;
        delayed.is_copy = false;
        delayed.targets.clear();
        delayed.crime_qualifying_target = false;
        delayed.target_type_slots = None;
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
            && let Some(source) = r.item.source
            && self.object_ref(source.id) == source.object
            && self.state.objects[source.id as usize].zone == Zone::Stack
            && !self.state.stack.iter().any(|s| s.id == r.item.id)
        {
            self.move_object(source.id, Zone::Exile);
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
                trigger.source = Some(StackSourceView {
                    id: source.object.card,
                    object: source.object,
                    card: source.definition,
                    facts: source.facts,
                });
                trigger.source_keywords = source.keywords;
                trigger.source_tapped = source.tapped;
                trigger.source_attachment = None;
                trigger.cast = None;
                trigger.mode = None;
                trigger.x = None;
                trigger.loyalty_x = None;
                trigger.gift = None;
                trigger.cast_event = None;
                trigger.source_counter_lki = None;
                trigger.source_quantity_lki = None;
                trigger.event_object = None;
                trigger.target_controller_lki = None;
                trigger.target_kinds_lki = None;
                trigger.is_copy = false;
                trigger.targets.clear();
                trigger.crime_qualifying_target = false;
                trigger.target_type_slots = None;
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
        let instant_or_sorcery = crate::cards::CastFilter::InstantSorcery
            .matches(spell.source.expect("object-backed spell or trigger").card);
        self.capture_copy_cast_classified(spell, first_trigger, instant_or_sorcery);
    }

    fn capture_copy_cast_classified(
        &mut self,
        spell: SpellView,
        first_trigger: usize,
        instant_or_sorcery: bool,
    ) {
        let repeated = self
            .state
            .stack_copy_data
            .as_ref()
            .map_or_else(Vec::new, |state| {
                state
                    .delayed
                    .iter()
                    .filter(|trigger| {
                        trigger.controller == spell.controller && delayed_copy_all_spells(trigger)
                    })
                    .cloned()
                    .collect::<Vec<_>>()
            });
        for mut trigger in repeated {
            trigger.id = self.next_stack_id();
            trigger.copy_event = Some(Box::new(CopyEvent {
                spell: spell.clone(),
                amount: 1,
            }));
            self.state.pending_triggers.push(trigger);
        }
        if !instant_or_sorcery
            && !spell
                .source
                .expect("object-backed spell or trigger")
                .card
                .is_prepared_face()
        {
            return;
        }
        let prior = self.state.players[spell.controller.index()]
            .instant_sorcery_cast_this_turn
            .saturating_sub(1);
        for trigger in &mut self.state.pending_triggers[first_trigger..] {
            if let Some((_, program)) = trigger.program()
                && let Some(amount) = program.iter().find_map(|i| match i {
                    Instruction::CopyCastSpell { prior_spells } if instant_or_sorcery => {
                        Some(if *prior_spells { prior } else { 1 })
                    }
                    Instruction::CopyCastSpellChosenX if instant_or_sorcery => {
                        Some(u64::from(spell.x.unwrap_or(0)))
                    }
                    Instruction::CopyPreparedCastSpell => Some(1),
                    _ => None,
                })
            {
                trigger.copy_event = Some(Box::new(CopyEvent {
                    spell: spell.clone(),
                    amount,
                }));
            }
        }
        // The remaining delayed copy abilities only observe instant/sorcery
        // casts, even when a different prepared spell needed capture above.
        if !instant_or_sorcery {
            return;
        }
        let Some(state) = &mut self.state.stack_copy_data else {
            return;
        };
        let mut delayed = std::mem::take(&mut state.delayed);
        let mut retained = Vec::new();
        for mut trigger in delayed.drain(..) {
            if delayed_copy_all_spells(&trigger) {
                retained.push(trigger);
                continue;
            }
            if trigger.controller == spell.controller
                && (!delayed_copy_reads_source_power(&trigger) || {
                    let power = if self.on_battlefield(
                        trigger
                            .source
                            .expect("object-backed spell or trigger")
                            .object
                            .card,
                    ) && self.object_ref(
                        trigger
                            .source
                            .expect("object-backed spell or trigger")
                            .object
                            .card,
                    ) == trigger
                        .source
                        .expect("object-backed spell or trigger")
                        .object
                    {
                        self.stats(
                            trigger
                                .source
                                .expect("object-backed spell or trigger")
                                .object
                                .card,
                        )
                        .power
                    } else {
                        trigger
                            .source
                            .expect("object-backed spell or trigger")
                            .facts
                            .power
                    };
                    i128::from(spell.mana_value()) <= power
                })
            {
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
    #[cfg(test)]
    pub(super) fn test_capture_noninstant_prepared_cast(
        &mut self,
        index: usize,
        first_trigger: usize,
    ) {
        let mut spell = self.state.stack[index].clone();
        // Preserve an actual spell object and stack id, while substituting the
        // existing prepared-face marker. Classification is explicitly non-instant.
        spell.source.as_mut().unwrap().card = Card::test_index(5458);
        self.capture_copy_cast_classified(spell, first_trigger, false);
    }
    pub(super) fn allocate_spell_copy(
        &mut self,
        template: &SpellView,
        controller: Player,
    ) -> SpellView {
        assert_eq!(template.kind, StackKind::Spell);
        let (card, generation) = if let Some(slot) = self.replay_frozen_mana_copy_slot() {
            slot
        } else if let Some(id) = self.state.free_tokens.pop() {
            (id, self.state.objects[id as usize].generation)
        } else {
            (
                CardId::try_from(self.state.objects.len()).capacity("object slots exhausted"),
                0,
            )
        };
        self.record_cast_mana_allocation(card, generation);
        let object = Object {
            card: template
                .source
                .expect("object-backed spell or trigger")
                .card,
            copy_kind: CopyKind::Spell,
            owner: controller,
            controller,
            zone: Zone::Stack,
            generation,
            tapped: false,
            summoning_sick: false,
            flashback: false,
            cast_for_warp: false,
            damage: 0,
            damaged_this_turn: false,
            dealt_damage_this_turn: false,
            dealt_combat_damage_this_incarnation: false,
            deathtouch_since_sba: false,
            growth: Stats::default(),
            gained_keywords: Keywords::NONE,
            counters: InitialCounters::default(),
            used_abilities: 0,
            loyalty_activated_this_turn: false,
            class_level: 1,
            attached_to: None,
        };
        if card as usize == self.state.objects.len() {
            self.state.objects.push(object);
        } else {
            self.state.objects[card as usize] = object;
        }
        let mut copy = template.clone();
        copy.id = self.next_stack_id();
        let source = copy
            .source
            .as_mut()
            .expect("a copied spell has an object source");
        source.id = card;
        source.object = self.object_ref(card);
        source.facts.controller = controller;
        copy.controller = controller;
        copy.is_copy = true;
        if let Some(cast) = &mut copy.cast {
            cast.payment = [0; 6];
            // The permission-created replacement is external to copiable values.
            cast.exile_on_graveyard = false;
            cast.spend_any_mana = false;
        }
        copy.copy_event = None;
        copy.mana_copy_sources = None;
        copy.cast_event = None;
        if template
            .source
            .expect("object-backed spell or trigger")
            .card
            .kind_mask()
            & (1 | 16 | 32 | 64)
            != 0
        {
            let recipe = template
                .source_copy_recipe
                .as_deref()
                .copied()
                .unwrap_or_else(|| {
                    CopyRecipe::new(
                        template
                            .source
                            .expect("object-backed spell or trigger")
                            .card,
                    )
                });
            self.set_copy_recipe(
                copy.source.expect("object-backed spell or trigger").object,
                recipe,
            );
            copy.source_copy_recipe = Some(Box::new(recipe));
        }
        // Source keywords are an LKI snapshot, not copied characteristics.
        // Evaluate after installing the recipe so the new spell has its own
        // copiable values and the continuous effects of its own controller.
        copy.source_keywords = self.keywords(card);
        copy
    }
    pub(super) fn copy_target_spell(
        &mut self,
        item: &SpellView,
        group: u16,
        retarget: bool,
        exceptions: CopyExceptions,
    ) -> bool {
        let Some(template) = item
            .targets
            .iter()
            .find(|b| b.group == group)
            .and_then(|b| self.bound_stack_item(b))
            .cloned()
        else {
            return false;
        };
        if template
            .source
            .expect("object-backed spell or trigger")
            .card
            .spell_cannot_be_copied()
        {
            return false;
        }
        let mut copy = self.allocate_spell_copy(&template, item.controller);
        if exceptions != CopyExceptions::NONE {
            let mut recipe = copy
                .source_copy_recipe
                .as_deref()
                .copied()
                .unwrap_or_else(|| {
                    CopyRecipe::new(copy.source.expect("object-backed spell or trigger").card)
                });
            recipe = recipe.with_exceptions(exceptions);
            self.set_copy_recipe(
                copy.source.expect("object-backed spell or trigger").object,
                recipe,
            );
            copy.source_copy_recipe = Some(Box::new(recipe));
            if exceptions.sacrifice_abilities != 0
                || exceptions.conditional_sacrifice_abilities != 0
            {
                copy.source_keywords = copy.source_keywords.union(Keywords::HASTE);
            }
        }
        self.state.stack_copy_data.get_or_insert_default().retarget = Some(Retarget {
            item: copy,
            template,
            remaining: 0,
            victim: None,
        });
        if retarget {
            self.advance_copy_targets()
        } else {
            self.commit_stack_targets();
            self.release_stack_copy_state();
            false
        }
    }
    /// Copy an ability packet, never its physical source. All declared choices,
    /// paid-cost facts and immutable program/origin contexts remain owned.
    pub(super) fn copy_target_ability(&mut self, item: &SpellView, group: u16) -> bool {
        let Some(template) = item
            .targets
            .iter()
            .find(|bound| bound.group == group)
            .and_then(|bound| self.bound_stack_item(bound))
            .filter(|candidate| candidate.kind != StackKind::Spell)
            .cloned()
        else {
            return false;
        };
        let mut copy = template.clone();
        copy.id = self.next_stack_id();
        copy.controller = item.controller;
        copy.is_copy = true;
        self.state.stack_copy_data.get_or_insert_default().retarget = Some(Retarget {
            item: copy,
            template,
            remaining: 0,
            victim: None,
        });
        self.advance_copy_targets()
    }
    pub(super) fn copy_captured_activated_ability(&mut self, item: &SpellView) -> bool {
        let Some(event) = item.copy_event.as_ref() else {
            return false;
        };
        let template = self
            .state
            .stack
            .iter()
            .find(|candidate| {
                candidate.id == event.spell.id
                    && matches!(candidate.kind, StackKind::ActivatedAbility { .. })
            })
            .cloned()
            .unwrap_or_else(|| event.spell.clone());
        if !matches!(template.kind, StackKind::ActivatedAbility { .. }) {
            return false;
        }
        let mut copy = template.clone();
        copy.id = self.next_stack_id();
        copy.controller = item.controller;
        copy.is_copy = true;
        self.state.stack_copy_data.get_or_insert_default().retarget = Some(Retarget {
            item: copy,
            template,
            remaining: 0,
            victim: None,
        });
        self.advance_copy_targets()
    }
    fn ability_source_kinds(&self, item: &SpellView) -> u16 {
        let Some(source) = item.source() else {
            return 0;
        };
        self.state
            .objects
            .get(source.object.card as usize)
            .filter(|object| object.generation == source.object.generation)
            .map_or(source.facts.kinds, |_| {
                self.current_kinds(source.object.card)
            })
    }
    /// Codie's trigger refers to the actual spell, not its last known copy.
    pub(super) fn copy_captured_prepared_spell(&mut self, item: &SpellView) -> bool {
        let Some(event) = &item.copy_event else {
            return false;
        };
        let Some(expected_source) = event.spell.source else {
            return false;
        };
        if !self.state.stack.iter().any(|spell| {
            spell.id == event.spell.id
                && spell
                    .source
                    .is_some_and(|source| source.object == expected_source.object)
                && spell.kind == StackKind::Spell
        }) {
            return false;
        }
        self.copy_captured_spell(item)
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
        if template
            .source
            .expect("object-backed spell or trigger")
            .card
            .spell_cannot_be_copied()
        {
            return false;
        }
        let copy = self.allocate_spell_copy(&template, item.controller);
        self.state
            .stack_copy_data
            .get_or_insert_with(Default::default)
            .retarget = Some(Retarget {
            item: copy,
            template,
            remaining: event.amount - 1,
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
            victim: Some((id, false)),
        });
        self.advance_copy_targets()
    }
    /// Only callbacks that read source power opt into its latest-departure facts.
    /// Other delayed programs keep their original packet observations unchanged.
    pub(super) fn capture_delayed_copy_source_departure(&mut self, before: &ObjectView) {
        if let Some(state) = &mut self.state.stack_copy_data {
            for item in &mut state.delayed {
                if item
                    .source
                    .is_some_and(|source| source.object == before.reference)
                    && delayed_copy_reads_source_power(item)
                {
                    item.source.as_mut().expect("delayed source exists").facts =
                        DamageFacts::from_view(before);
                    item.source_keywords = before.keywords;
                    item.source_tapped = before.tapped;
                }
            }
        }
    }
    pub(super) fn change_cast_spell_targets(&mut self, item: &SpellView) -> bool {
        let Some(victim) = item
            .cast_event
            .as_ref()
            .and_then(|event| {
                self.state
                    .stack
                    .iter()
                    .find(|spell| spell.kind == StackKind::Spell && spell.id == event.stack_id)
            })
            .cloned()
        else {
            return false;
        };
        let id = victim.id;
        self.state.stack_copy_data.get_or_insert_default().retarget = Some(Retarget {
            item: victim.clone(),
            template: victim,
            remaining: 0,
            victim: Some((id, true)),
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
            victim: r.victim.map(|(id, _)| id),
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
        if matches!(
            spec.rule,
            TargetRule::Grouped {
                constraint: TargetGroupConstraint::DistinctCardTypeSlots,
                ..
            }
        ) {
            let Some(slot) = r
                .item
                .target_type_slots
                .as_ref()
                .and_then(|slots| slots.get(index))
            else {
                return false;
            };
            if !matches!(target, Target::Card(id) if self.current_kinds(id) & *slot != 0) {
                return false;
            }
        }
        self.item_target_matches(&r.item, spec.rule, target)
    }
    fn stack_target_relations_hold(&self, item: &SpellView, index: usize) -> bool {
        let bound = item.targets[index];
        let plan = self.target_spec(item);
        let spec = plan.group(bound.group).unwrap();
        if matches!(
            spec.rule,
            TargetRule::Grouped {
                constraint: TargetGroupConstraint::DistinctCardTypeSlots,
                ..
            }
        ) {
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
        if matches!(
            spec.rule,
            TargetRule::Grouped {
                constraint: TargetGroupConstraint::TotalManaValueAtMost(_),
                ..
            }
        ) {
            // A retained target can be illegal (for example, it gained
            // hexproof) while still being part of the final target set. Count
            // its current mana value when checking a newly selected target.
            // CR 115.7d–e and XMage's total-value helper support this reading;
            // neither is a card-specific ruling for this interaction. Targets
            // that left the zone or changed incarnation remain excluded.
            let other_present: Vec<_> = item
                .targets
                .iter()
                .enumerate()
                .filter(|(i, other)| *i != index && self.same_target_object(**other))
                .map(|(_, bound)| *bound)
                .collect();
            return self.target_group_compatible(
                spec.rule,
                bound.group,
                &other_present,
                bound.target,
            );
        }
        if matches!(
            spec.rule,
            TargetRule::Grouped {
                constraint: TargetGroupConstraint::SameGraveyard
                    | TargetGroupConstraint::SharedCardType
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
        if r.victim.is_some_and(|(_, optional)| !optional) && !changed {
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
        if let Some((id, _)) = r.victim {
            if let Some(index) = self.state.stack.iter().position(|s| s.id == id) {
                let previous = self.state.stack[index].clone();
                self.acquire_stack_target_set(&r.item, &previous.targets);
                self.state.stack[index] = r.item.clone();
            }
        } else {
            self.acquire_stack_target_set(&r.item, &[]);
            self.state.stack.push(r.item.clone());
        }
        let state = self.state.stack_copy_data.as_mut().unwrap();
        if r.remaining == 0 {
            state.retarget = None;
        } else {
            let copy = self.allocate_spell_copy(&r.template, r.item.controller);
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
