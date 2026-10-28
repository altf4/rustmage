//! Copy acquisition and external grant identity are distinct from static-effect
//! timestamps. Only a permanent that has participated in a copy allocates a row.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct GrantOrigin {
    source: ObjectRef,
    origin: Option<u64>,
}
pub(super) type Rows = Option<SharedVec<GrantOrigin>>;
pub(super) fn owned_bytes(rows: &Rows) -> usize {
    rows.as_ref().map_or(0, |rows| {
        std::mem::size_of::<Vec<GrantOrigin>>() + rows.len() * std::mem::size_of::<GrantOrigin>()
    })
}
impl Game {
    fn has_live_program_grant(&self, source: ObjectRef) -> bool {
        use crate::cards::StaticAbilityFragment as F;
        self.on_battlefield(source.card)
            && self.object_ref(source.card) == source
            && self.printed_abilities_available(source.card)
            && self.state.objects[source.card as usize]
                .card
                .static_ability_groups()
                .iter()
                .flat_map(|group| group.fragments)
                .any(|fragment| {
                    matches!(
                        fragment,
                        F::AttachmentTrigger(_)
                            | F::AttachmentActivation(_)
                            | F::ControlledPlaneswalkerActivation(_)
                            | F::GrantInstantSorceryStorm { .. }
                            | F::CascadeFromHand { .. }
                            | F::HandMiracle { .. }
                    )
                })
    }
    pub(super) fn copied_grant_origin(&self, source: ObjectRef) -> Option<u64> {
        self.state
            .copy_grant_origins
            .as_ref()?
            .iter()
            .find(|row| row.source == source)?
            .origin
    }
    pub(super) fn retains_provided_origin(&self, origin: u64) -> bool {
        self.state
            .copy_grant_origins
            .as_ref()
            .is_some_and(|rows| rows.iter().any(|row| row.origin == Some(origin)))
    }
    /// New copy, restored printed text, or an actual new attachment grants a
    /// new recipient instance. A pure object timestamp change does not.
    pub(super) fn renew_copy_grant_origin(&mut self, source: ObjectRef, track: bool) {
        if !track
            && !self
                .state
                .copy_grant_origins
                .as_ref()
                .is_some_and(|rows| rows.iter().any(|row| row.source == source))
        {
            return;
        }
        let origin = self
            .has_live_program_grant(source)
            .then(|| self.next_role_timestamp());
        let rows = self.state.copy_grant_origins.get_or_insert_default();
        if let Some(row) = rows.iter_mut().find(|row| row.source == source) {
            row.origin = origin;
        } else {
            rows.push(GrantOrigin { source, origin });
        }
    }
    pub(super) fn refresh_copy_grant_availability(&mut self) {
        self.refresh_graveyard_borrowing();
        let Some(rows) = &self.state.copy_grant_origins else {
            return;
        };
        let changed: Vec<_> = rows
            .iter()
            .filter(|row| row.origin.is_some() != self.has_live_program_grant(row.source))
            .map(|row| row.source)
            .collect();
        for source in changed {
            self.renew_copy_grant_origin(source, false);
        }
    }
    pub(super) fn forget_copy_grant_origin(&mut self, source: ObjectRef) {
        if let Some(rows) = &mut self.state.copy_grant_origins {
            rows.retain(|row| row.source != source);
            if rows.is_empty() {
                self.state.copy_grant_origins = None;
            }
        }
    }
}
