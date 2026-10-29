use super::*;

#[test]
fn planeswalker_type_count_is_distinct_controller_sensitive_and_snapshot_owned() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let source = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        let first = put(
            &mut g,
            player,
            "Ajani, Caller of the Pride",
            Zone::Battlefield,
        );
        let second = put(
            &mut g,
            player,
            "Ajani, Caller of the Pride",
            Zone::Battlefield,
        );
        let enemy = put(
            &mut g,
            player.other(),
            "Kaito, Cunning Infiltrator",
            Zone::Battlefield,
        );
        let value = |g: &Game| {
            g.stat_value(
                StatValue::DistinctControlledPlaneswalkerTypes,
                g.object_ref(source),
                player,
            )
        };
        assert_eq!(value(&g), 1);
        g.create_tokens(player, Card::test_index(5435), 1, false);
        assert_eq!(value(&g), 2);
        let saved = g.snapshot();
        g.change_controller(g.object_ref(enemy), player);
        assert_eq!(value(&g), 3);
        g.move_object(first, Zone::Graveyard);
        assert_eq!(value(&g), 3);
        g.move_object(second, Zone::Graveyard);
        assert_eq!(value(&g), 2);
        g.restore(&saved).unwrap();
        assert_eq!(value(&g), 2);
    }
}

#[test]
fn planeswalker_type_inventory_is_explicit_and_does_not_accept_unknown_names() {
    assert!(subtypes::PLANESWALKER_TYPES.contains(&"Ajani"));
    assert!(subtypes::PLANESWALKER_TYPES.contains(&"Jace"));
    assert!(subtypes::PLANESWALKER_TYPES.contains(&"Kaito"));
    assert!(!subtypes::PLANESWALKER_TYPES.contains(&"Bogus"));
    assert!(!subtypes::PLANESWALKER_TYPES.contains(&"Construct"));
}
