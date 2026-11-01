//! Focused printed outcomes; shared engine invariants live in core tests.
//! Secret Identity changes only the selected controlled creature until cleanup.
use super::*;

#[test]
fn secret_identity_preserves_artifact_type_and_original_abilities() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let target = put(&mut g, p, "Darksteel Colossus", Zone::Battlefield);
        let spell = put(&mut g, p, "Secret Identity", Zone::Hand);
        big_helpers::cast(&mut g, spell, Some(0), &[Target::Card(target)]);
        big_helpers::settle(&mut g);
        assert_eq!(g.current_kinds(target) & (1 | 16), 1 | 16);
        assert!(
            g.keywords(target).contains(
                Keywords::INDESTRUCTIBLE
                    .union(Keywords::TRAMPLE)
                    .union(Keywords::HEXPROOF)
            )
        );
        assert!(g.has_subtype(target, "Citizen"));
        assert!(!g.has_subtype(target, "Golem"));
        g.finish_cleanup();
        assert!(g.has_subtype(target, "Golem"));
        assert!(
            g.keywords(target)
                .contains(Keywords::INDESTRUCTIBLE.union(Keywords::TRAMPLE))
        );
    }
}

#[test]
fn secret_identity_type_replacement_applies_after_changeling() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let target = put(&mut g, p, "Changeling Wayfinder", Zone::Battlefield);
        assert!(g.has_subtype(target, "Bear"));
        let spell = put(&mut g, p, "Secret Identity", Zone::Hand);
        big_helpers::cast(&mut g, spell, Some(1), &[Target::Card(target)]);
        big_helpers::settle(&mut g);
        assert!(g.has_subtype(target, "Hero"));
        assert!(!g.has_subtype(target, "Bear"));

        g.finish_cleanup();
        assert!(g.has_subtype(target, "Bear"));
    }
}
