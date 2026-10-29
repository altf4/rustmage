//! Exact returned recipients retain a source-independent layer-six effect.
use super::ability_world::CharacteristicWorld;
use super::*;
/// Lose all abilities, then gain these keywords, without a stated duration.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct ResolvedAbilityModificationView {
    pub recipient: ObjectRef,
    pub source: ObjectRef,
    pub keywords: Keywords,
    pub timestamp: u64,
}
pub(super) fn active<W: CharacteristicWorld + ?Sized>(
    world: &W,
    row: &ResolvedAbilityModificationView,
) -> bool {
    world.object(row.recipient.card).zone == Zone::Battlefield
        && world.reference(row.recipient.card) == row.recipient
}
impl std::hash::Hash for ResolvedAbilityModificationView {
    fn hash<H: std::hash::Hasher>(&self, h: &mut H) {
        self.recipient.hash(h);
        self.source.hash(h);
        self.keywords.hash(h);
        self.timestamp.hash(h);
    }
}
impl Game {
    pub(super) fn return_source_blank_with_haste(&mut self, item: &SpellView) {
        let Some(source) = item.source else { return; };
        let Some(movement) = item.source_move.as_ref().filter(|m| {
            m.before == source.object
                && m.origin == Zone::Battlefield
                && m.destination == Zone::Graveyard
        }) else {
            return;
        };
        let r = movement.after;
        let o = &self.state.objects[r.card as usize];
        if o.is_token() || o.zone != Zone::Graveyard || self.object_ref(r.card) != r {
            return;
        }
        let owner = o.owner;
        let mut entry = self.entry_move(r.card, Zone::Battlefield, false);
        entry.controller = owner;
        entry.effect_counter = Some((CounterKind::PlusOne, 1));
        entry.blank_with_haste_after = Some(source.object);
        self.stage_entries(vec![entry], None);
    }
    pub(super) fn blank_and_grant_haste(&mut self, recipient: ObjectRef, source: ObjectRef) {
        if !self.on_battlefield(recipient.card) || self.object_ref(recipient.card) != recipient {
            return;
        }
        let timestamp = self.next_role_timestamp();
        self.state
            .roles
            .as_mut()
            .unwrap()
            .resolved_ability_modifications
            .push(ResolvedAbilityModificationView {
                recipient,
                source,
                keywords: Keywords::HASTE,
                timestamp,
            });
        self.refresh_copy_grant_availability();
    }
    pub(super) fn resolved_ability_modification_views(
        &self,
    ) -> Option<Vec<ResolvedAbilityModificationView>> {
        let rows: Vec<_> = self
            .state
            .roles
            .as_ref()?
            .resolved_ability_modifications
            .iter()
            .filter(|row| active(self, row))
            .copied()
            .collect();
        (!rows.is_empty()).then_some(rows)
    }
}
