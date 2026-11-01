//! Focused printed outcomes; shared engine invariants live in core tests.
use super::*;

fn make_copy(g: &mut Game, player: Player, target: Player) -> CardId {
    put(g, target, "Bear Cub", Zone::Battlefield);
    let spell = put(g, player, "Face Yourself", Zone::Hand);
    cast(g, spell, Some(Target::Player(target)));
    settle(g);
    g.state
        .objects
        .iter()
        .enumerate()
        .find_map(|(id, object)| {
            (object.zone == Zone::Battlefield
                && object.controller == player
                && object.is_copy_token()
                && object.card.name() == "Bear Cub")
                .then_some(id as CardId)
        })
        .unwrap()
}

#[test]
fn face_yourself_copies_target_players_creatures_with_copiable_haste() {
    {
        let player = Player::P0;
        for target in [player, player.other()] {
            let mut g = scenario(player);
            let copy = make_copy(&mut g, player, target);
            assert!(g.keywords(copy).contains(Keywords::HASTE));
            assert_eq!(
                g.copiable_recipe(g.object_ref(copy))
                    .exceptions
                    .conditional_sacrifice_abilities,
                1
            );
            assert_eq!(stats(&g, copy), (2, 2));
        }
    }
}
