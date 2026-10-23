//! A private exact hand subset becomes public before another player chooses exile.
use super::*;
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct HandRevealChoice {
    pub owner: Player,
    pub required: usize,
    pub revealed: bool,
    pub cards: Vec<ObjectRef>,
}
impl HandRevealChoice {
    pub(super) fn bytes(&self) -> usize {
        std::mem::size_of::<Self>() + self.cards.len() * std::mem::size_of::<ObjectRef>()
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct HandRevealView {
    pub owner: Player,
    pub required: usize,
    pub revealed: bool,
    pub cards: Vec<ObjectRef>,
}
impl Game {
    pub(super) fn hand_reveal_candidate(
        &self,
        choice: &effect_choices::EffectChoice,
        object: ObjectRef,
    ) -> bool {
        let reveal = choice.hand_reveal.as_ref().unwrap();
        self.state
            .objects
            .get(object.card as usize)
            .is_some_and(|card| {
                card.zone == Zone::Hand
                    && card.owner == reveal.owner
                    && self.object_ref(object.card) == object
                    && if reveal.revealed {
                        reveal.cards.contains(&object)
                    } else {
                        !reveal.cards.contains(&object)
                    }
            })
    }
    pub(super) fn hand_reveal_view(
        &self,
        viewer: Player,
        choice: &effect_choices::EffectChoice,
        reveal: &HandRevealChoice,
    ) -> Option<EffectChoiceView> {
        if !reveal.revealed && viewer != reveal.owner {
            return None;
        }
        let candidates = self
            .state
            .objects
            .iter()
            .enumerate()
            .map(|(id, _)| self.object_ref(id as CardId))
            .filter(|&object| self.hand_reveal_candidate(choice, object))
            .collect();
        Some(EffectChoiceView {
            kind: choice.kind,
            hand_reveal: Some(HandRevealView {
                owner: reveal.owner,
                required: reveal.required,
                revealed: reveal.revealed,
                cards: reveal.cards.clone(),
            }),
            amount: 1,
            optional: false,
            required: [0; 2],
            selected: std::array::from_fn(|index| {
                if index == reveal.owner.index() {
                    reveal.cards.clone()
                } else {
                    Vec::new()
                }
            }),
            candidates,
            counter_removal: None,
            proliferate: None,
            defender_choice: false,
        })
    }
    pub(super) fn start_hand_reveal_for_cast(
        &mut self,
        item: &SpellView,
        amount: AmountValue,
    ) -> bool {
        let Some(owner) = item.targets.iter().find_map(|bound| {
            if let Target::Player(player) = bound.target {
                Some(player)
            } else {
                None
            }
        }) else {
            return false;
        };
        let required = usize::try_from(
            self.amount_value(amount, item.object, item.controller)
                .min(self.hand_count(owner) as u128),
        )
        .expect("bounded by hand size");
        if required == 0 {
            return false;
        }
        self.state.resolving.as_mut().unwrap().choice =
            Some(Box::new(effect_choices::EffectChoice {
                kind: EffectChoiceKind::RevealHandThenExile,
                filter: evidence::cost(0).filter,
                amount: 1,
                source: item.object,
                required: [0; 2],
                selected: std::array::from_fn(|_| Vec::new()),
                skip_to: None,
                greatest_power: [None; 2],
                discard_mana_value_life: None,
                hand_defender: None,
                hand_creature_entry: None,
                blight_copy_attacking: false,
                counter_removal: None,
                proliferate: None,
                entry_life_bonus: None,
                hand_reveal: Some(Box::new(HandRevealChoice {
                    owner,
                    required,
                    revealed: false,
                    cards: Vec::new(),
                })),
            }));
        self.advance_hand_reveal()
    }
    pub(super) fn advance_hand_reveal(&mut self) -> bool {
        let resolution = self.state.resolving.as_mut().unwrap();
        let reveal = resolution
            .choice
            .as_mut()
            .unwrap()
            .hand_reveal
            .as_mut()
            .unwrap();
        if reveal.cards.len() < reveal.required {
            self.state.pending = Pending::EffectChoice {
                player: reveal.owner,
            };
            return true;
        }
        if !reveal.revealed {
            reveal.revealed = true;
            for &card in &reveal.cards {
                if !self.state.known_hand.contains(&card) {
                    self.state.known_hand.push(card);
                }
            }
        }
        if reveal.cards.len() == 1 {
            let object = reveal.cards[0];
            self.complete_hand_reveal_exile(object);
            false
        } else {
            self.state.pending = Pending::EffectChoice {
                player: resolution.item.controller,
            };
            true
        }
    }
    fn complete_hand_reveal_exile(&mut self, object: ObjectRef) {
        let resolution = self.state.resolving.as_mut().unwrap();
        let item = resolution.item.clone();
        let choice = resolution.choice.take().unwrap();
        debug_assert!(choice.hand_reveal.as_ref().unwrap().cards.contains(&object));
        let event = self.move_object(object.card, Zone::Exile);
        if event.moved && event.destination == Zone::Exile {
            self.grant_source_bound_exile_cast(&item, event.after);
        }
    }
    pub(super) fn choose_hand_reveal_object(&mut self, object: ObjectRef) {
        let reveal = self
            .state
            .resolving
            .as_mut()
            .unwrap()
            .choice
            .as_mut()
            .unwrap()
            .hand_reveal
            .as_mut()
            .unwrap();
        if reveal.revealed {
            self.complete_hand_reveal_exile(object);
            self.run_program();
        } else {
            reveal.cards.push(object);
            if !self.advance_hand_reveal() {
                self.run_program();
            }
        }
    }
}
