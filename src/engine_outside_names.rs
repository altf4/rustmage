//! Exact outside-game rules names derived from pinned structured Oracle faces.
use super::*;
use std::collections::HashMap;
use std::sync::OnceLock;

fn table() -> &'static HashMap<&'static str, Vec<&'static str>> {
    static TABLE: OnceLock<HashMap<&'static str, Vec<&'static str>>> = OnceLock::new();
    TABLE.get_or_init(|| include_str!("../data/outside-card-names.tsv").lines().map(|line| {
        let mut fields = line.split('\t');
        let id = fields.next().expect("outside name row has identity");
        let names: Vec<_> = fields.collect();
        assert!(!names.is_empty(), "outside name row has names");
        (id, names)
    }).collect())
}

pub(super) fn names(card: Card) -> Option<&'static [&'static str]> {
    card.info().oracle_id.and_then(|id| table().get(id).map(Vec::as_slice))
}

#[allow(dead_code, reason = "outside-game choice consumer remains gated")]
pub(super) fn different(left: Card, right: Card) -> Option<bool> {
    let left = names(left)?;
    let right = names(right)?;
    Some(!left.iter().any(|a| right.iter().any(|b| crate::card_names::same_name(a, b))))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn oracle_faces_supply_split_and_front_face_names() {
        let split = Card::from_name("Bottomless Pool // Locker Room").unwrap();
        assert_eq!(names(split), Some(&["Bottomless Pool", "Locker Room"][..]));
        assert_eq!(different(split, split), Some(false));
        let dfc = Card::from_name("Fatehold Chronologist // Peer Review").unwrap();
        assert_eq!(names(dfc), Some(&["Fatehold Chronologist"][..]));
        assert_eq!(different(split, dfc), Some(true));
    }
}
