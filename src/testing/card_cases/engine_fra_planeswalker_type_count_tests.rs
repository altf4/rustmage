//! Focused printed outcomes; shared engine invariants live in core tests.
use super::*;

#[test]
fn planeswalker_type_inventory_is_explicit_and_does_not_accept_unknown_names() {
    assert!(subtypes::PLANESWALKER_TYPES.contains(&"Ajani"));
    assert!(subtypes::PLANESWALKER_TYPES.contains(&"Jace"));
    assert!(subtypes::PLANESWALKER_TYPES.contains(&"Kaito"));
    assert!(!subtypes::PLANESWALKER_TYPES.contains(&"Bogus"));
    assert!(!subtypes::PLANESWALKER_TYPES.contains(&"Construct"));
}
