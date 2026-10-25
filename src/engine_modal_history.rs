//! Lifetime and per-turn choose-once histories are cold owned state, keyed by incarnation.
use super::*;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ModalHistoryView {
    pub object: ObjectRef,
    pub definition: Card,
    pub ability: u16,
    pub modes: Vec<u16>,
    /// Absent for lifetime histories; otherwise the exact actual turn of selection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub turn: Option<u32>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Key {
    object: ObjectRef,
    definition: Card,
    ability: u16,
    per_turn: bool,
}
pub(super) type Rows = Option<SharedVec<ModalHistoryView>>;

// Only triggered headers are admitted: an activated ability needs history-aware
// activation admission as well as the shared modal announcement path.
pub(super) fn key(item: &SpellView) -> Option<Key> {
    let StackKind::TriggeredAbility { ability } = item.kind else {
        return None;
    };
    let definition = item.trigger_definition();
    let per_turn = match definition.triggers()[usize::from(ability)].program.first() {
        Some(Instruction::ModalOnce(_)) => false,
        Some(Instruction::ModalOncePerTurn(_)) => true,
        _ => return None,
    };
    Some(Key {
        object: item.object,
        definition,
        ability,
        per_turn,
    })
}
impl ModalHistoryView {
    fn key(&self) -> Key {
        Key {
            object: self.object,
            definition: self.definition,
            ability: self.ability,
            per_turn: self.turn.is_some(),
        }
    }
}
pub(super) fn owned_bytes(rows: &Rows) -> usize {
    rows.as_ref().map_or(0, |rows| {
        std::mem::size_of::<Vec<ModalHistoryView>>()
            + rows.len() * std::mem::size_of::<ModalHistoryView>()
            + rows
                .iter()
                .map(|r| r.modes.len() * std::mem::size_of::<u16>())
                .sum::<usize>()
    })
}
impl Game {
    pub(super) fn modal_choice_unused(&self, item: &SpellView, index: usize) -> bool {
        let Some(key) = key(item) else { return true };
        !self.state.modal_history.as_ref().is_some_and(|rows| {
            rows.iter().any(|row| {
                row.key() == key
                    && row.turn.is_none_or(|turn| turn == self.state.turn)
                    && row.modes.contains(&(index as u16))
            })
        })
    }
    pub(super) fn record_modal_choice(&mut self, key: Key, mode: u16) {
        let turn = key.per_turn.then_some(self.state.turn);
        let rows = self
            .state
            .modal_history
            .get_or_insert_with(Default::default);
        if let Some(row) = rows.iter_mut().find(|row| row.key() == key) {
            if row.turn != turn {
                row.modes.clear();
                row.turn = turn;
            }
            match row.modes.binary_search(&mode) {
                Ok(_) => unreachable!("validated unused modal choice"),
                Err(index) => {
                    row.modes
                        .try_reserve(1)
                        .capacity("modal choice history allocation exhausted");
                    row.modes.insert(index, mode);
                }
            }
        } else {
            rows.try_reserve(1)
                .capacity("modal history allocation exhausted");
            rows.push(ModalHistoryView {
                object: key.object,
                definition: key.definition,
                ability: key.ability,
                modes: vec![mode],
                turn,
            });
        }
    }
    pub(super) fn prune_modal_history(&mut self) {
        let Some(rows) = &self.state.modal_history else {
            return;
        };
        let needed = |row: &ModalHistoryView| {
            if row.turn.is_some_and(|turn| turn != self.state.turn) {
                return false;
            }
            let object = &self.state.objects[row.object.card as usize];
            (object.zone == Zone::Battlefield && self.object_ref(row.object.card) == row.object)
                || self
                    .state
                    .stack
                    .iter()
                    .chain(&self.state.pending_triggers)
                    .chain(&self.state.ordering_triggers)
                    .chain(self.state.announcing.iter().map(|a| &a.item))
                    .chain(self.state.resolving.iter().map(|r| &r.item))
                    .any(|item| key(item) == Some(row.key()))
        };
        if rows.iter().all(&needed) {
            return;
        }
        let retained: Vec<_> = rows.iter().filter(|row| needed(row)).cloned().collect();
        self.state.modal_history = (!retained.is_empty()).then(|| SharedVec::from(retained));
    }
}
