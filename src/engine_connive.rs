//! Snapshot-owned sequential connive operations and private nonland discards.
use super::*;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct ConniverView {
    pub object: ObjectRef,
    pub controller: Player,
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
}
impl ConniveSequence {
    pub fn bytes(&self) -> usize {
        std::mem::size_of::<Self>() + self.remaining.len() * std::mem::size_of::<ConniverView>()
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
                .map(|object| ConniverView {
                    object,
                    controller: item.controller,
                })
                .into_iter()
                .collect(),
            Subject::Source => {
                if !item.source_facts.battlefield {
                    return;
                }
                vec![ConniverView {
                    object: item.object,
                    controller: if self.on_battlefield(item.card)
                        && self.object_ref(item.card) == item.object
                    {
                        self.state.objects[item.card as usize].owner
                    } else {
                        item.controller
                    },
                }]
            }
            Subject::Targets => item
                .targets
                .iter()
                .filter_map(|bound| {
                    let Target::Card(id) = bound.target else {
                        return None;
                    };
                    Some(ConniverView {
                        object: bound.object?,
                        controller: self.state.objects[id as usize].owner,
                    })
                })
                .collect(),
            Subject::Attached => self
                .attached_subject(item)
                .map(|id| {
                    vec![ConniverView {
                        object: self.object_ref(id),
                        controller: self.state.objects[id as usize].owner,
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
        }));
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
        let amount = sequence.amount;
        self.draw_many(current.controller, u128::from(amount));
        self.state.resolving.as_mut().unwrap().discards[current.controller.index()] =
            usize::from(amount);
    }
    pub(super) fn advance_connive(&mut self) -> bool {
        loop {
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
                self.discard_batch(&discarded, None);
                if self.on_battlefield(current.object.card)
                    && self.object_ref(current.object.card) == current.object
                {
                    self.state.objects[current.object.card as usize]
                        .counters
                        .add(CounterKind::PlusOne, u64::from(nonlands));
                }
                self.state
                    .resolving
                    .as_mut()
                    .unwrap()
                    .connive
                    .as_mut()
                    .unwrap()
                    .current = None;
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
