//! Resolution-created continuous effects with exact, irreversible source durations.
use super::ability_world::CharacteristicWorld;
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
pub enum SourceLinkedEffectKind {
    RemoveAbilities,
    PreventUntap,
    Control,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct SourceLinkedEffectView {
    pub source: ObjectRef,
    pub recipient: ObjectRef,
    pub controller: Player,
    pub kind: SourceLinkedEffectKind,
    pub timestamp: u64,
}
pub(super) fn active<W: CharacteristicWorld + ?Sized>(
    world: &W,
    row: &SourceLinkedEffectView,
) -> bool {
    world.object(row.source.card).zone == Zone::Battlefield
        && world.reference(row.source.card) == row.source
        && world.object(row.recipient.card).zone == Zone::Battlefield
        && world.reference(row.recipient.card) == row.recipient
        && (row.kind != SourceLinkedEffectKind::PreventUntap
            || world.object(row.source.card).controller == row.controller)
}
impl Game {
    pub(super) fn resolve_source_linked_effect(
        &mut self,
        item: &SpellView,
        kind: SourceLinkedEffectKind,
    ) {
        // CR611.2b: an already-ended duration never starts. The preceding tap
        // remains independent and has already happened even when this fails.
        if !self.on_battlefield(item.object.card)
            || self.object_ref(item.object.card) != item.object
            || (kind == SourceLinkedEffectKind::PreventUntap
                && self.state.objects[item.object.card as usize].controller != item.controller)
        {
            return;
        }
        let recipients: Vec<_> = self
            .keyword_subjects(item, Subject::Targets)
            .into_iter()
            .filter(|&id| self.on_battlefield(id))
            .map(|id| self.object_ref(id))
            .collect();
        if recipients.is_empty() {
            return;
        }
        if kind == SourceLinkedEffectKind::Control {
            for &recipient in &recipients {
                self.ensure_control_base(recipient);
            }
        }
        let timestamp = self.next_role_timestamp();
        let rows = &mut self.state.roles.as_mut().unwrap().source_linked_effects;
        for recipient in recipients {
            rows.push(SourceLinkedEffectView {
                source: item.object,
                recipient,
                controller: item.controller,
                kind,
                timestamp,
            });
        }
        if kind == SourceLinkedEffectKind::Control {
            self.refresh_controllers();
        } else {
            self.refresh_copy_grant_availability();
        }
    }
    pub(super) fn forget_source_linked_effects(&mut self, reference: ObjectRef) {
        if let Some(state) = &mut self.state.roles {
            state
                .source_linked_effects
                .retain(|row| row.source != reference && row.recipient != reference);
        }
    }
    pub(super) fn end_control_linked_effects(&mut self, reference: ObjectRef) {
        if let Some(state) = &mut self.state.roles {
            state.source_linked_effects.retain(|row| {
                row.source != reference || row.kind != SourceLinkedEffectKind::PreventUntap
            });
        }
    }
    pub(super) fn source_linked_effect_views(&self) -> Option<Vec<SourceLinkedEffectView>> {
        let rows: Vec<_> = self
            .state
            .roles
            .as_ref()?
            .source_linked_effects
            .iter()
            .filter(|row| active(self, row))
            .copied()
            .collect();
        (!rows.is_empty()).then_some(rows)
    }
}

impl std::hash::Hash for SourceLinkedEffectView {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.source.hash(state);
        self.recipient.hash(state);
        self.controller.index().hash(state);
        self.kind.hash(state);
        self.timestamp.hash(state);
    }
}
