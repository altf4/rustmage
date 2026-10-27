//! Resolution-created abilities belong to exact recipients, not printed definitions.
use super::ability_world::CharacteristicWorld;
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum SelfGrantedAbilityKind {
    /// A self-granted base-power setter is layer7b, not a CDA (CR604.3a).
    ControllerHandPower,
    /// Self-granted layer7b setter for both axes, using current controlled lands.
    ControllerLandStats,
    /// Source-owned layer7c bonus, counting current controlled artifacts.
    ControllerArtifactBonus,
    /// A real immutable activated template, independently acquired per grant.
    Activated { ability: u16 },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Row {
    pub recipient: ObjectRef,
    pub granting_card: Card,
    pub kind: SelfGrantedAbilityKind,
    pub timestamp: u64,
    /// Generated static-effect chronology, independent of ability acquisition.
    pub effect_timestamp: Option<u64>,
    pub expires_turn: Option<u32>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct SelfGrantedAbilityView {
    pub recipient: ObjectRef,
    pub granting_card: Card,
    pub kind: SelfGrantedAbilityKind,
    pub timestamp: u64,
    /// Generated static-effect chronology, independent of ability acquisition.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effect_timestamp: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_turn: Option<u32>,
    pub available: bool,
}
// Preserve the optional cold-state allocation used by State.
#[allow(clippy::box_collection)]
pub(super) fn owned_bytes(rows: &Option<Box<Vec<Row>>>) -> usize {
    rows.as_ref().map_or(0, |rows| {
        std::mem::size_of::<Vec<Row>>() + rows.len() * std::mem::size_of::<Row>()
    })
}
impl Game {
    pub(super) fn grant_self_ability(&mut self, item: &SpellView, kind: SelfGrantedAbilityKind) {
        if !self.on_battlefield(item.object.card)
            || self.object_ref(item.object.card) != item.object
        {
            return;
        }
        let timestamp = self.next_role_timestamp();
        self.grant_self_ability_at(item, kind, timestamp);
    }
    pub(super) fn grant_permanent_self_ability(
        &mut self,
        item: &SpellView,
        kind: SelfGrantedAbilityKind,
    ) {
        if !self.on_battlefield(item.object.card)
            || self.object_ref(item.object.card) != item.object
        {
            return;
        }
        let timestamp = self.next_role_timestamp();
        self.grant_self_ability_with_expiry(item, kind, timestamp, None);
    }
    /// One resolving continuous effect may bestow animation and an ability at
    /// the same timestamp; callers supply that effect's already allocated time.
    pub(super) fn grant_self_ability_at(
        &mut self,
        item: &SpellView,
        kind: SelfGrantedAbilityKind,
        timestamp: u64,
    ) {
        self.grant_self_ability_with_expiry(item, kind, timestamp, Some(self.state.turn));
    }
    fn grant_self_ability_with_expiry(
        &mut self,
        item: &SpellView,
        kind: SelfGrantedAbilityKind,
        timestamp: u64,
        expires_turn: Option<u32>,
    ) {
        self.grant_recipient_ability(
            item.object,
            item.trigger_definition(),
            kind,
            timestamp,
            expires_turn,
        );
    }
    pub(super) fn grant_recipient_ability(
        &mut self,
        recipient: ObjectRef,
        definition: Card,
        kind: SelfGrantedAbilityKind,
        timestamp: u64,
        expires_turn: Option<u32>,
    ) {
        if !self.on_battlefield(recipient.card) || self.object_ref(recipient.card) != recipient {
            return;
        }
        if let SelfGrantedAbilityKind::Activated { ability } = kind {
            assert!(definition.self_granted_activation_template(ability));
            let rules = definition.activation(ability);
            assert!(
                rules.origin == ActivationOrigin::Battlefield
                    && !rules.once_per_turn
                    && !rules.exhaust
                    && !definition.power_up_abilities().contains(&ability)
                    && rules.mana_production().is_none(),
                "unsupported granted activation lifecycle"
            );
        }
        self.state
            .self_granted_abilities
            .get_or_insert_default()
            .push(Row {
                recipient,
                granting_card: definition,
                kind,
                timestamp,
                effect_timestamp: if matches!(kind, SelfGrantedAbilityKind::Activated { .. }) {
                    None
                } else {
                    Some(timestamp)
                },
                expires_turn,
            });
    }
    pub(super) fn retimestamp_self_granted_statics(&mut self, source: ObjectRef) {
        let Some(rows) = &self.state.self_granted_abilities else {
            return;
        };
        let mut indices: Vec<_> = rows
            .iter()
            .enumerate()
            .filter(|(_, row)| row.recipient == source && row.effect_timestamp.is_some())
            .map(|(index, row)| (row.effect_timestamp.unwrap(), index))
            .collect();
        indices.sort_unstable();
        // CR613.7a: retain relative static-effect order without reacquiring any ability.
        for (_, index) in indices {
            let timestamp = self.next_role_timestamp();
            self.state.self_granted_abilities.as_mut().unwrap()[index].effect_timestamp =
                Some(timestamp);
        }
    }
    pub(super) fn forget_self_grants(&mut self, event: &ZoneChange) {
        if !event.moved {
            return;
        }
        if let Some(rows) = &mut self.state.self_granted_abilities {
            rows.retain(|row| row.recipient != event.before.reference);
            if rows.is_empty() {
                self.state.self_granted_abilities = None;
            }
        }
    }
    pub(super) fn self_granted_ability_views(&self) -> Option<Vec<SelfGrantedAbilityView>> {
        let rows = self.state.self_granted_abilities.as_ref()?;
        let query = self.characteristic_query();
        Some(
            rows.iter()
                .map(|row| SelfGrantedAbilityView {
                    recipient: row.recipient,
                    granting_card: row.granting_card,
                    kind: row.kind,
                    timestamp: row.timestamp,
                    effect_timestamp: row.effect_timestamp,
                    expires_turn: row.expires_turn,
                    available: row.expires_turn.is_none_or(|turn| turn == self.state.turn)
                        && query.self_grant_available(row.recipient, row.timestamp),
                })
                .collect(),
        )
    }
}
impl<W: CharacteristicWorld + ?Sized> characteristic_query::CharacteristicQuery<'_, W> {
    pub(super) fn self_granted_artifact_bonus(&self, id: CardId) -> Stats {
        let Some(rows) = &self.game().state.self_granted_abilities else {
            return Stats::default();
        };
        let recipient = self.reference(id);
        let count = rows
            .iter()
            .filter(|row| {
                row.recipient == recipient
                    && row.kind == SelfGrantedAbilityKind::ControllerArtifactBonus
                    && row
                        .expires_turn
                        .is_none_or(|turn| turn == self.game().state.turn)
                    && self.self_grant_available(recipient, row.timestamp)
            })
            .count();
        if count == 0 {
            return Stats::default();
        }
        let controller = self.object(id).controller;
        let artifacts = (0..self.game().state.objects.len())
            .filter(|&id| {
                let id = id as CardId;
                let object = self.object(id);
                object.zone == Zone::Battlefield
                    && object.controller == controller
                    && self.kinds(id) & 16 != 0
            })
            .count() as i128;
        let bonus = artifacts
            .checked_mul(count as i128)
            .capacity("self-granted artifact bonus exhausted");
        Stats {
            power: bonus,
            toughness: bonus,
        }
    }
}
