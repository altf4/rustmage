//! Non-targeting creature anchors chosen by one controller, followed by one sacrifice batch.
use super::*;
impl Game {
    pub(super) fn keep_shared_type_player(
        &self,
        choice: &effect_choices::EffectChoice,
    ) -> Option<Player> {
        [self.state.active, self.state.active.other()]
            .into_iter()
            .find(|player| {
                choice.selected[player.index()].len() < usize::from(choice.required[player.index()])
            })
    }
    pub(super) fn keep_shared_type_candidate(
        &self,
        choice: &effect_choices::EffectChoice,
        object: ObjectRef,
    ) -> bool {
        self.keep_shared_type_player(choice).is_some_and(|player| {
            self.on_battlefield(object.card)
                && self.object_ref(object.card) == object
                && self.state.objects[object.card as usize].controller == player
                && self.is_creature(object.card)
        })
    }
    pub(super) fn start_keep_shared_types(&mut self, item: &SpellView) -> bool {
        let required = std::array::from_fn(|index| {
            u16::from(self.state.objects.iter().enumerate().any(|(id, object)| {
                object.zone == Zone::Battlefield
                    && object.controller.index() == index
                    && self.is_creature(id as CardId)
            }))
        });
        self.state.resolving.as_mut().unwrap().choice =
            Some(Box::new(effect_choices::EffectChoice {
                kind: EffectChoiceKind::KeepSharedCreatureTypes,
                filter: evidence::cost(0).filter,
                amount: 1,
                source: item.object,
                required,
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
                hand_reveal: None,
            }));
        self.advance_keep_shared_types()
    }
    pub(super) fn advance_keep_shared_types(&mut self) -> bool {
        let resolution = self.state.resolving.as_ref().unwrap();
        let choice = resolution.choice.as_ref().unwrap();
        if self.keep_shared_type_player(choice).is_some() {
            self.state.pending = Pending::EffectChoice {
                player: resolution.item.controller,
            };
            return true;
        }
        let choice = self
            .state
            .resolving
            .as_mut()
            .unwrap()
            .choice
            .take()
            .unwrap();
        // Freeze characteristics before any sacrifice can remove a subtype provider.
        let anchors: [Option<ObjectView>; 2] = std::array::from_fn(|index| {
            choice.selected[index]
                .first()
                .map(|r| self.object_view(r.card))
        });
        let sacrifices: Vec<_> = self
            .state
            .objects
            .iter()
            .enumerate()
            .filter_map(|(id, object)| {
                let id = id as CardId;
                if object.zone != Zone::Battlefield || !self.is_creature(id) {
                    return None;
                }
                let anchor = anchors[object.controller.index()].as_ref()?;
                if anchor.reference == self.object_ref(id) {
                    return None;
                }
                let recipient = self.object_view(id);
                let shares = crate::cards::CREATURE_TYPES
                    .iter()
                    .any(|kind| anchor.has_subtype(kind) && recipient.has_subtype(kind));
                (!shares).then_some((object.controller, id))
            })
            .collect();
        self.sacrifice_batch(&sacrifices);
        false
    }
}
