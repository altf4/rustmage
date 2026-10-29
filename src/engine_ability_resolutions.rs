//! Successful resolutions, keyed by exact source incarnation and printed ability.
//! Rows survive source departure for queued packets, but are never inherited by
//! a new incarnation. They are discarded together at the next turn boundary.
use super::*;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct AbilityResolutionView {
    pub source: ObjectRef,
    pub definition: Card,
    pub kind: StackKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin: Option<u64>,
    pub count: u64,
}
pub(super) type Rows = Option<SharedVec<AbilityResolutionView>>;
pub(super) fn owned_bytes(rows: &Rows) -> usize {
    rows.as_ref().map_or(0, |r| {
        std::mem::size_of::<Vec<AbilityResolutionView>>()
            + r.len() * std::mem::size_of::<AbilityResolutionView>()
    })
}
impl Game {
    pub(super) fn record_ability_resolution(&mut self, item: &SpellView) {
        let Some(source) = item.source else { return; };
        assert!(matches!(
            item.kind,
            StackKind::TriggeredAbility { .. } | StackKind::ActivatedAbility { .. }
        ));
        let rows = self
            .state
            .ability_resolutions
            .get_or_insert_with(SharedVec::default);
        if let Some(row) = rows.iter_mut().find(|r| {
            r.source == source.object
                && r.kind == item.kind
                && r.definition == item.trigger_definition()
                && r.origin == item.source_ability_origin
        }) {
            row.count = row
                .count
                .checked_add(1)
                .capacity("ability resolution count exhausted");
        } else {
            rows.push(AbilityResolutionView {
                source: source.object,
                definition: item.trigger_definition(),
                kind: item.kind,
                origin: item.source_ability_origin,
                count: 1,
            });
        }
    }
    pub(super) fn ability_resolution_number(&self, item: &SpellView) -> u64 {
        let Some(source) = item.source else { return 0; };
        self.state
            .ability_resolutions
            .as_ref()
            .and_then(|rows| {
                rows.iter().find(|r| {
                    r.source == source.object
                        && r.kind == item.kind
                        && r.definition == item.trigger_definition()
                        && r.origin == item.source_ability_origin
                })
            })
            .map_or(0, |r| r.count)
    }
}
