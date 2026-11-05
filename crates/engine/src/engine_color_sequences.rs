//! Owned WUBRG recipient choices and actual simultaneous placement receipts.
use super::*;
use crate::cards::ChoiceColor;
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ColorCounterSequence {
    pub player: Player,
    pub subtype: &'static str,
    pub next: usize,
    pub selected: Vec<(ChoiceColor, ObjectRef)>,
    pub successful: Vec<ObjectRef>,
    pub placed: bool,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl ColorCounterSequence {
    pub(super) fn owned_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.selected.len() * std::mem::size_of::<(ChoiceColor, ObjectRef)>()
            + self.successful.len() * std::mem::size_of::<ObjectRef>()
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn color_counter_candidate(
        &self,
        player: Player,
        color: ChoiceColor,
        subtype: &str,
        object: ObjectRef,
    ) -> bool {
        self.on_battlefield(object.card)
            && self.object_ref(object.card) == object
            && self.state.objects[object.card as usize].controller == player
            && ability_world::CharacteristicWorld::colors(self, object.card) & (1 << color.index())
                != 0
            && self.has_subtype(object.card, subtype)
    }
    pub(super) fn choose_color_counter_recipient(&mut self, object: ObjectRef) {
        let r = self.state.resolving.as_mut().unwrap();
        let sequence = r.color_sequence.as_mut().unwrap();
        let color = ChoiceColor::ALL[sequence.next];
        sequence.selected.push((color, object));
        sequence.next += 1;
        r.choice = None;
    }
    pub(super) fn record_color_counter_recipients(
        &mut self,
        packet: u64,
        recipients: &[ObjectRef],
    ) {
        let r = self.state.resolving.as_mut().expect("owned color result");
        assert_eq!(r.item.id, packet, "color receipt belongs to another packet");
        let sequence = r.color_sequence.as_mut().expect("owned color sequence");
        for &object in recipients {
            if !sequence.successful.contains(&object) {
                sequence.successful.push(object);
            }
        }
    }
    pub(super) fn advance_color_counter_sequence(&mut self) -> bool {
        loop {
            let Some(sequence) = self
                .state
                .resolving
                .as_ref()
                .and_then(|r| r.color_sequence.as_ref())
            else {
                return false;
            };
            if sequence.placed {
                let sequence = self
                    .state
                    .resolving
                    .as_mut()
                    .unwrap()
                    .color_sequence
                    .take()
                    .unwrap();
                if sequence.successful.len() >= 5 {
                    self.try_win_game(sequence.player);
                }
                return self.state.outcome.is_some();
            }
            if let Some(&color) = ChoiceColor::ALL.get(sequence.next) {
                let player = sequence.player;
                let subtype = sequence.subtype;
                if (0..self.state.objects.len()).any(|id| {
                    self.color_counter_candidate(
                        player,
                        color,
                        subtype,
                        self.object_ref(id as CardId),
                    )
                }) {
                    self.start_color_counter_choice(color, subtype);
                    return true;
                }
                self.state
                    .resolving
                    .as_mut()
                    .unwrap()
                    .color_sequence
                    .as_mut()
                    .unwrap()
                    .next += 1;
                continue;
            }
            let r = self.state.resolving.as_mut().unwrap();
            let sequence = r.color_sequence.as_mut().unwrap();
            sequence.placed = true;
            let mut changes: Vec<(ObjectRef, Counters)> = Vec::new();
            for &(_, object) in &sequence.selected {
                if let Some((_, counters)) =
                    changes.iter_mut().find(|(previous, _)| *previous == object)
                {
                    counters.add(CounterKind::PlusOne, 1);
                } else {
                    changes.push((object, Counters::of(CounterKind::PlusOne, 1)));
                }
            }
            let player = sequence.player;
            let packet = r.item.id;
            self.place_color_counter_batch(player, packet, changes);
            if self.advance_replacements(false) {
                return true;
            }
        }
    }
}
