//! Snapshot-owned sequential connive operations and private nonland discards.
use super::*;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct ConniverView {
    pub object: ObjectRef,
    pub controller: Player,
    #[serde(skip)]
    creature: bool,
}
#[derive(Clone, Debug, Serialize)]
pub struct ConniveChoiceView {
    pub amount: u16,
    pub remaining: Vec<ConniverView>,
    pub current: Option<ConniverView>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ConniveSequence {
    pub amount: u16,
    pub remaining: Vec<ConniverView>,
    pub current: Option<ConniverView>,
    used: Vec<(ObjectRef, u16)>,
    draw_started: bool,
}
impl ConniveSequence {
    pub fn bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.remaining.len() * std::mem::size_of::<ConniverView>()
            + self.used.len() * std::mem::size_of::<(ObjectRef, u16)>()
    }
}
impl Game {
    pub(super) fn start_connive(&mut self, item: &SpellView, subject: Subject, amount: u16) {
        if amount == 0 {
            return;
        }
        let remaining = match subject {
            Subject::Entering => item
                .entering_object
                .map(|object| {
                    let saved = item.event_object.as_ref().expect("event conniver facts");
                    assert_eq!(saved.object, object);
                    let EventObjectFacts::Controller(controller) = saved.facts else {
                        unreachable!("event conniver controller");
                    };
                    ConniverView {
                        object,
                        creature: if self.on_battlefield(object.card)
                            && self.object_ref(object.card) == object
                        {
                            self.is_creature(object.card)
                        } else {
                            saved.conniver_creature.expect("departed conniver kind")
                        },
                        controller: if self.on_battlefield(object.card)
                            && self.object_ref(object.card) == object
                        {
                            self.state.objects[object.card as usize].controller
                        } else {
                            controller
                        },
                    }
                })
                .into_iter()
                .collect(),
            Subject::Source => {
                if !item.source_facts.battlefield {
                    return;
                }
                vec![ConniverView {
                    object: item.object,
                    creature: if self.on_battlefield(item.card)
                        && self.object_ref(item.card) == item.object
                    {
                        self.is_creature(item.card)
                    } else {
                        item.source_facts.is_creature()
                    },
                    controller: if self.on_battlefield(item.card)
                        && self.object_ref(item.card) == item.object
                    {
                        self.state.objects[item.card as usize].controller
                    } else {
                        item.source_facts.controller
                    },
                }]
            }
            Subject::Targets => item
                .targets
                .iter()
                .filter(|bound| bound.group == self.state.resolving.as_ref().unwrap().target_group)
                .filter_map(|bound| {
                    let Target::Card(id) = bound.target else {
                        return None;
                    };
                    let object = bound.object?;
                    if !self.on_battlefield(id) || self.object_ref(id) != object {
                        return None;
                    }
                    Some(ConniverView {
                        object,
                        creature: self.is_creature(id),
                        controller: self.state.objects[id as usize].controller,
                    })
                })
                .collect(),
            Subject::Attached => self
                .attached_subject(item)
                .map(|id| {
                    vec![ConniverView {
                        object: self.object_ref(id),
                        creature: self.is_creature(id),
                        controller: self.state.objects[id as usize].controller,
                    }]
                })
                .unwrap_or_default(),
        };
        if remaining.is_empty() {
            return;
        }
        assert!(self.state.resolving.as_ref().unwrap().connive.is_none());
        self.state.resolving.as_mut().unwrap().connive = Some(Box::new(ConniveSequence {
            amount,
            remaining,
            current: None,
            used: Vec::new(),
            draw_started: false,
        }));
    }
    pub(super) fn connive_replacements(&self) -> Vec<replacements::ReplacementEffectView> {
        let Some(sequence) = self
            .state
            .resolving
            .as_ref()
            .and_then(|r| r.connive.as_ref())
        else {
            return Vec::new();
        };
        let Some(current) = sequence
            .current
            .filter(|c| c.creature && !sequence.draw_started)
        else {
            return Vec::new();
        };
        let mut out = Vec::new();
        for &id in &self.state.static_sources {
            if !self.on_battlefield(id)
                || self.state.objects[id as usize].controller != current.controller
            {
                continue;
            }
            let card = self.state.objects[id as usize].card;
            let source = self.object_ref(id);
            for (index, spec) in card.replacement_effects().iter().enumerate() {
                let ability = u16::try_from(index).expect("replacement slot");
                if *spec == crate::cards::ReplacementSpec::ConniveDraw
                    && !sequence.used.contains(&(source, ability))
                    && self.continuous_fragment_available(id, |f| matches!(f, crate::cards::StaticAbilityFragment::Replacement(slot, _) if *slot == ability)) {
                    out.push(replacements::ReplacementEffectView { source, source_card: card, ability });
                }
            }
        }
        out
    }
    pub(super) fn apply_connive_replacement(&mut self, source: ObjectRef, ability: u16) {
        let sequence = self
            .state
            .resolving
            .as_mut()
            .unwrap()
            .connive
            .as_mut()
            .unwrap();
        assert!(!sequence.draw_started && !sequence.used.contains(&(source, ability)));
        sequence.used.push((source, ability));
        let player = sequence.current.unwrap().controller;
        self.draw_many(player, 1);
    }
    pub(super) fn connive_replacement_choice(&self) -> Option<replacements::ReplacementChoiceView> {
        let sequence = self.state.resolving.as_ref()?.connive.as_ref()?;
        let current = sequence.current?;
        let effects = self.connive_replacements();
        if effects.len() < 2 {
            return None;
        }
        Some(replacements::ReplacementChoiceView {
            kind: "Connive",
            object: Some(current.object),
            effects,
            counters: None,
            tokens: Vec::new(),
            copy_recipes: Vec::new(),
            role_attachments: Vec::new(),
            damage: None,
            damage_preventions: Vec::new(),
            heals_before_damage: Vec::new(),
            damage_event_order: Vec::new(),
        })
    }
    pub(super) fn choose_conniver(&mut self, object: ObjectRef) {
        let sequence = self
            .state
            .resolving
            .as_mut()
            .unwrap()
            .connive
            .as_mut()
            .unwrap();
        assert!(sequence.current.is_none());
        let index = sequence
            .remaining
            .iter()
            .position(|c| c.object == object)
            .unwrap();
        let current = sequence.remaining.remove(index);
        sequence.current = Some(current);
        sequence.used.clear();
        sequence.draw_started = false;
    }
    pub(super) fn advance_connive(&mut self) -> bool {
        loop {
            if self.miracle_choice_pending() {
                return true;
            }
            if let Some(current) = self
                .state
                .resolving
                .as_ref()
                .unwrap()
                .connive
                .as_ref()
                .unwrap()
                .current
            {
                if !self
                    .state
                    .resolving
                    .as_ref()
                    .unwrap()
                    .connive
                    .as_ref()
                    .unwrap()
                    .draw_started
                {
                    let choices = self.connive_replacements();
                    match choices.as_slice() {
                        [] => {
                            let sequence = self
                                .state
                                .resolving
                                .as_mut()
                                .unwrap()
                                .connive
                                .as_mut()
                                .unwrap();
                            sequence.draw_started = true;
                            let amount = sequence.amount;
                            self.state.resolving.as_mut().unwrap().discards
                                [current.controller.index()] = usize::from(amount);
                            self.draw_many(current.controller, u128::from(amount));
                        }
                        [only] => self.apply_connive_replacement(only.source, only.ability),
                        _ => {
                            self.state.pending = Pending::Replacement {
                                player: current.controller,
                            };
                            return true;
                        }
                    }
                    continue;
                }
                let r = self.state.resolving.as_ref().unwrap();
                let player = current.controller;
                let remaining = r.discards[player.index()];
                if remaining > 0
                    && self.hand_count(player) > r.selected_discards[player.index()].len()
                {
                    self.state.pending = Pending::EffectDiscard { player, remaining };
                    return true;
                }
                let r = self.state.resolving.as_mut().unwrap();
                r.discards[player.index()] = 0;
                let discarded = std::mem::take(&mut r.selected_discards[player.index()]);
                let nonlands = u32::try_from(
                    discarded
                        .iter()
                        .filter(|&&id| !self.state.objects[id as usize].card.is_land())
                        .count(),
                )
                .capacity("connive count exhausted");
                let cause = self.state.resolving.as_ref().unwrap().item.controller;
                self.discard_batch_caused(&discarded, None, Some(cause));
                if let Some(context) = self
                    .state
                    .entry_batch
                    .as_mut()
                    .and_then(|b| b.discard_context.as_mut())
                {
                    context.after = Some(discard_events::DiscardAfter::ConniveCounters(
                        current.object,
                        u64::from(nonlands),
                        current.controller,
                    ));
                    self.state
                        .resolving
                        .as_mut()
                        .unwrap()
                        .connive
                        .as_mut()
                        .unwrap()
                        .current = None;
                    return true;
                }
                if self.on_battlefield(current.object.card)
                    && self.object_ref(current.object.card) == current.object
                {
                    self.effect_counter_by(
                        current.object.card,
                        CounterKind::PlusOne,
                        u64::from(nonlands),
                        current.controller,
                        replacements::ResumeMode::Program,
                    );
                }
                self.state
                    .resolving
                    .as_mut()
                    .unwrap()
                    .connive
                    .as_mut()
                    .unwrap()
                    .current = None;
                if self.state.replacements.is_some() {
                    return true;
                }
            }
            let sequence = self
                .state
                .resolving
                .as_ref()
                .unwrap()
                .connive
                .as_ref()
                .unwrap();
            if sequence.remaining.is_empty() {
                self.state.resolving.as_mut().unwrap().connive = None;
                return false;
            }
            let player = if sequence
                .remaining
                .iter()
                .any(|c| c.controller == self.state.active)
            {
                self.state.active
            } else {
                self.state.active.other()
            };
            let mut candidates = sequence.remaining.iter().filter(|c| c.controller == player);
            let object = candidates.next().unwrap().object;
            if candidates.next().is_some() {
                self.state.pending = Pending::ConniveOrder { player };
                return true;
            }
            self.choose_conniver(object);
        }
    }
}
