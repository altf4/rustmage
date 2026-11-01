//! Focused printed outcomes; shared engine invariants live in core tests.
use super::*;

#[test]
fn tomik_static_grant_has_distinct_public_identity_from_activated_grants() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let source = put(&mut g, p, "Tomik, Orzhov Lawmage", Zone::Battlefield);
        let walker = put(&mut g, p, "Ajani, Caller of the Pride", Zone::Battlefield);
        g.state.objects[walker as usize].counters.loyalty = 5;
        let view = g.object_view(walker);
        let grants = &view.abilities.as_ref().unwrap().grants;
        let grant = grants
            .iter()
            .find(|grant| grant.grantor == g.object_ref(source))
            .unwrap();
        assert_eq!(grant.ability, None);
        assert_eq!(
            grant.static_rule,
            Some(GrantedStaticRule::PlaneswalkerAttackLimit)
        );
        let encoded = serde_json::to_value(grant).unwrap();
        assert!(encoded.get("ability").is_none());
        assert_eq!(encoded["static_rule"], "PlaneswalkerAttackLimit");
        assert!(!view.grant_available(
            g.object_ref(source),
            g.state.objects[source as usize].card,
            0,
            None
        ));
    }
}

#[test]
fn tomik_matching_reassigns_earlier_attacker_for_asymmetric_choices() {
    // First attacker can use either walker; second can use only the first.
    // A greedy first-slot assignment gets one, while reassignment gets two.
    assert_eq!(
        combat_restrictions::maximum_attack_matching(&[vec![0, 1], vec![0]], 2),
        2
    );
    assert_eq!(
        combat_restrictions::maximum_attack_matching(&[vec![0], vec![0]], 2),
        1
    );
    assert_eq!(
        combat_restrictions::maximum_attack_matching(&[vec![], vec![1]], 2),
        1
    );
}
