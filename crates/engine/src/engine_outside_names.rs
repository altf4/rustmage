//! Exact outside-game rules names derived from pinned structured Oracle faces.
use super::*;
use std::collections::HashMap;
use std::sync::OnceLock;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
fn table() -> &'static HashMap<&'static str, Vec<&'static str>> {
    static TABLE: OnceLock<HashMap<&'static str, Vec<&'static str>>> = OnceLock::new();
    TABLE.get_or_init(|| {
        crate::cards::generated::catalogue()
            .outside_names
            .lines()
            .map(|line| {
                let mut fields = line.split('\t');
                let id = fields.next().expect("outside name row has identity");
                let names: Vec<_> = fields.collect();
                assert!(!names.is_empty(), "outside name row has names");
                (id, names)
            })
            .collect()
    })
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn names(card: Card) -> Option<&'static [&'static str]> {
    card.info()
        .oracle_id
        .and_then(|id| table().get(id).map(Vec::as_slice))
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[allow(dead_code, reason = "outside-game choice consumer remains gated")]
pub(super) fn different(left: Card, right: Card) -> Option<bool> {
    let left = names(left)?;
    let right = names(right)?;
    Some(
        !left
            .iter()
            .any(|a| right.iter().any(|b| crate::card_names::same_name(a, b))),
    )
}
