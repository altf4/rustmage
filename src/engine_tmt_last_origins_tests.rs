//! Explicit physical outside-game pools preserve legality, ownership and private observations.
use super::*;
fn config(rules: DeckRules) -> Config {
    Config {
        decks: [vec![Card::Forest; 60], vec![Card::Forest; 60]],
        seed: 913,
        starting_player: Player::P0,
        deck_rules: rules,
    }
}
#[test]
fn tmt_last_outside_private_supplied_cards_are_distinct_from_ceased_tokens() {
    let plains = Card::from_name("Plains").unwrap();
    let island = Card::from_name("Island").unwrap();
    let g = Game::new_with_sideboards(config(DeckRules::Standard), [vec![plains], vec![island]])
        .unwrap();
    let own = g.outside_game_cards(Player::P0).next().unwrap();
    let other = g.outside_game_cards(Player::P1).next().unwrap();
    assert_ne!(own, other);
    for p in [Player::P0, Player::P1] {
        let observation = g.observe(p);
        assert_eq!(observation.outside_game_cards.len(), 1);
        assert_eq!(observation.outside_game_cards[0].owner, p);
        assert!(!observation.objects.iter().any(|o| o.zone == Zone::Outside));
    }
    assert!(!g.outside_game_card_matches(Player::P0, other));
    let empty = Game::new(config(DeckRules::Standard)).unwrap();
    assert!(empty.outside_game_cards(Player::P0).next().is_none());
}
#[test]
fn tmt_last_outside_combined_copy_limits_sideboard_size_and_physical_faces_are_checked() {
    let bear = Card::from_name("Bear Cub").unwrap();
    let token = Card::from_name("@token/283fd83a9aec83a51b736926").unwrap();
    let mut c = config(DeckRules::Constructed);
    c.decks[0].splice(0..4, [bear; 4]);
    assert!(Game::new_with_sideboards(c, [vec![bear], vec![]]).is_err());
    assert!(
        Game::new_with_sideboards(
            config(DeckRules::Standard),
            [vec![Card::Forest; 16], vec![]]
        )
        .is_err()
    );
    assert!(Game::new_with_sideboards(config(DeckRules::Freeform), [vec![token], vec![]]).is_err());
    assert!(
        Game::new_with_sideboards(
            config(DeckRules::Freeform),
            [vec![Card::Forest; 16], vec![]]
        )
        .is_ok()
    );
}
#[test]
fn tmt_last_outside_exact_membership_movement_branch_and_restore() {
    let p = Player::P0;
    let mut g =
        Game::new_with_sideboards(config(DeckRules::Standard), [vec![Card::Forest], vec![]])
            .unwrap();
    let object = g.outside_game_cards(p).next().unwrap();
    let snapshot = g.snapshot();
    let mut branch = {
        let parent = g.clone();
        parent.clone()
    };
    let event = g.move_object(object.card, Zone::Hand);
    branch.move_object(object.card, Zone::Hand);
    assert!(event.moved);
    assert_eq!(g.snapshot(), branch.snapshot());
    assert!(!g.outside_game_card_matches(p, object));
    assert!(g.outside_game_cards(p).next().is_none());
    assert!(
        !g.observe(p.other())
            .objects
            .iter()
            .any(|o| o.id == object.card)
    );
    g.restore(&snapshot).unwrap();
    assert!(g.outside_game_card_matches(p, object));
    assert_eq!(g.observe(p).outside_game_cards[0].reference, object);
}
#[test]
fn tmt_last_outside_ceased_token_or_unregistered_outside_card_is_never_selectable() {
    let p = Player::P0;
    let mut g =
        Game::new_with_sideboards(config(DeckRules::Standard), [vec![Card::Forest], vec![]])
            .unwrap();
    let id = super::tests::spawn(&mut g, p, Card::Forest, Zone::Outside);
    let object = g.object_ref(id);
    assert!(!g.outside_game_card_matches(p, object));
    assert_eq!(g.outside_game_cards(p).count(), 1);
    let token = Card::from_name("@token/283fd83a9aec83a51b736926").unwrap();
    let id = super::tests::spawn(&mut g, p, token, Zone::Outside);
    let object = g.object_ref(id);
    assert!(!g.outside_game_card_matches(p, object));
    assert_eq!(g.outside_game_cards(p).count(), 1);
}
