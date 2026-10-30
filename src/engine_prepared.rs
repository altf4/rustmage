//! CR722 preparation designation and its independently visible associated card copy.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct PreparationView {
    pub permanent: ObjectRef,
    pub spell_copy: ObjectRef,
}
pub(super) type Rows = Option<SharedVec<PreparationView>>;
pub(super) fn owned_bytes(rows: &Rows) -> usize {
    rows.as_ref().map_or(0, |rows| {
        std::mem::size_of::<Vec<PreparationView>>()
            + rows.len() * std::mem::size_of::<PreparationView>()
    })
}
impl Game {
    pub(super) fn keeps_card_copy(&self, copy: ObjectRef) -> bool {
        self.state
            .preparations
            .as_ref()
            .is_some_and(|rows| rows.iter().any(|r| r.spell_copy == copy))
            || self
                .state
                .resolution_free_casts
                .as_ref()
                .is_some_and(|r| r.remaining.contains(&copy))
    }

    pub(super) fn is_prepared(&self, permanent: ObjectRef) -> bool {
        self.state
            .preparations
            .as_ref()
            .is_some_and(|rows| rows.iter().any(|r| r.permanent == permanent))
    }
    pub(super) fn prepare_permanent(&mut self, permanent: ObjectRef) {
        if !self.on_battlefield(permanent.card)
            || self.object_ref(permanent.card) != permanent
            || self.is_prepared(permanent)
        {
            return;
        }
        // Preparation uses the current copiable definition, ignoring copy exceptions.
        // It is a designation and does not require the permanent to remain a creature
        // or to retain its abilities after preparation.
        let Some(spell) = self.copiable_recipe(permanent).definition.prepared_spell() else {
            return;
        };
        let controller = self.state.objects[permanent.card as usize].controller;
        let spell_copy = self.allocate_card_copy(spell, controller);
        let rows = self.state.preparations.get_or_insert_with(Default::default);
        rows.try_reserve(1)
            .capacity("preparation allocation exhausted");
        rows.push(PreparationView {
            permanent,
            spell_copy,
        });
    }
    pub(super) fn can_cast_prepared(&self, player: Player, copy: CardId) -> bool {
        let object = &self.state.objects[copy as usize];
        object.zone == Zone::Exile
            && self.state.preparations.as_ref().is_some_and(|rows| {
                rows.iter().any(|r| {
                    r.spell_copy == self.object_ref(copy)
                        && self.on_battlefield(r.permanent.card)
                        && self.object_ref(r.permanent.card) == r.permanent
                        && self.state.objects[r.permanent.card as usize].controller == player
                })
            })
    }
    pub(super) fn advance_prepared_copy(&mut self, before: ObjectRef, after: ObjectRef) {
        if let Some(rows) = &mut self.state.preparations {
            for row in rows.iter_mut() {
                if row.spell_copy == before {
                    row.spell_copy = after;
                }
            }
        }
    }
    pub(super) fn complete_prepared_cast(&mut self, copy: ObjectRef) {
        let source = self.state.preparations.as_ref().and_then(|rows| {
            rows.iter()
                .find(|r| r.spell_copy == copy)
                .map(|r| r.permanent)
        });
        if let Some(source) = source {
            self.forget_prepared(source);
        }
    }
    pub(super) fn forget_prepared(&mut self, permanent: ObjectRef) {
        let Some(rows) = &mut self.state.preparations else {
            return;
        };
        let Some(index) = rows.iter().position(|r| r.permanent == permanent) else {
            return;
        };
        rows.remove(index);
        if rows.is_empty() {
            self.state.preparations = None;
        }
        // An unassociated card copy in exile ceases at the next state-based
        // action, not in the middle of the effect that removed preparation.
        // A copy already cast remains on the stack as usual.
    }
}
