//! Resolution-created continuous effects with exact, irreversible source durations.
use super::ability_world::CharacteristicWorld;
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
pub enum SourceLinkedEffectKind {
    RemoveAbilities,
    /// Source-independent removal lasting until cleanup.
    RemoveAbilitiesUntilEndOfTurn,
    GrantHexproof,
    PreventDamageBy,
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
    (row.kind == SourceLinkedEffectKind::RemoveAbilitiesUntilEndOfTurn
        || (world.object(row.source.card).zone == Zone::Battlefield
            && world.reference(row.source.card) == row.source))
        && (row.kind == SourceLinkedEffectKind::PreventDamageBy
            || (world.object(row.recipient.card).zone == Zone::Battlefield
                && world.reference(row.recipient.card) == row.recipient))
        && (row.kind != SourceLinkedEffectKind::PreventUntap
            || world.object(row.source.card).controller == row.controller)
}
impl Game {
    pub(super) fn resolve_source_linked_effect(
        &mut self,
        item: &SpellView,
        kind: SourceLinkedEffectKind,
    ) {
        let Some(source) = item.source else { return; };
        // CR611.2b: an already-ended duration never starts. The preceding tap
        // remains independent and has already happened even when this fails.
        if !self.on_battlefield(source.object.card)
            || self.object_ref(source.object.card) != source.object
            || (kind == SourceLinkedEffectKind::PreventUntap
                && self.state.objects[source.object.card as usize].controller != item.controller)
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
                source: source.object,
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
    pub(super) fn remove_target_abilities_until_end_of_turn(&mut self, item: &SpellView) {
        let Some(source) = item.source else { return; };
        let recipients: Vec<_> = self.keyword_subjects(item, Subject::Targets).into_iter()
            .filter(|&id| self.on_battlefield(id)).map(|id| self.object_ref(id)).collect();
        if recipients.is_empty() { return; }
        let timestamp = self.next_role_timestamp();
        let rows = &mut self.state.roles.as_mut().unwrap().source_linked_effects;
        for recipient in recipients {
            rows.push(SourceLinkedEffectView { source: source.object, recipient,
                controller: item.controller, kind: SourceLinkedEffectKind::RemoveAbilitiesUntilEndOfTurn,
                timestamp });
        }
        self.refresh_copy_grant_availability();
    }
    pub(super) fn expire_temporary_ability_removals(&mut self) {
        let changed = if let Some(roles) = &mut self.state.roles {
            let before = roles.source_linked_effects.len();
            roles.source_linked_effects.retain(|row| row.kind != SourceLinkedEffectKind::RemoveAbilitiesUntilEndOfTurn);
            before != roles.source_linked_effects.len()
        } else { false };
        if changed { self.refresh_copy_grant_availability(); }
    }
    pub(super) fn forget_source_linked_effects(&mut self, reference: ObjectRef) {
        if let Some(state) = &mut self.state.roles {
            state.source_linked_effects.retain(|row| {
                (row.source != reference || row.kind == SourceLinkedEffectKind::RemoveAbilitiesUntilEndOfTurn)
                    && (row.recipient != reference
                        || row.kind == SourceLinkedEffectKind::PreventDamageBy)
            });
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
