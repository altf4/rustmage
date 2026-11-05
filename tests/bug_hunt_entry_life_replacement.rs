mod support;
use rustmage as _;
use rustmage_core::engine::{
    ability_world::CharacteristicWorld,
    funding::{FundingGoal, FundingState},
    *,
};
use support::*;

fn shock_land_from_library(other_cards: usize) -> (Game, CardId, Vec<CardId>) {
    let p = Player::P0;
    let mut game = scenario(p);
    let ashiok = put(&mut game, p, "Ashiok, Wicked Manipulator", Zone::Hand);
    cast(&mut game, ashiok, None);
    settle(&mut game);
    assert_eq!(game.object_view(ashiok).zone, Zone::Battlefield);
    put(&mut game, p, "Ka-Zar of the Savage Land", Zone::Battlefield);
    let original: Vec<_> = game.state.players[p.index()]
        .library
        .iter()
        .copied()
        .collect();
    for id in original {
        game.move_object(id, Zone::Exile);
    }
    let forests = (0..other_cards)
        .map(|_| put(&mut game, p, "Forest", Zone::Library))
        .collect();
    let pool = put(&mut game, p, "Breeding Pool", Zone::Library);
    act(&mut game, Action::PlayLand { card: pool });
    (game, pool, forests)
}

#[test]
fn life_replacement_counts_an_entering_library_card_without_exiling_it() {
    for other_cards in 0..=2 {
        let (mut game, pool, forests) = shock_land_from_library(other_cards);
        let p = game.state.active;
        let mut projected = FundingState::from_game(
            &game,
            &FundingGoal {
                actor: p,
                required: [0; 6],
                generic: 0,
                reserve: None,
                source: None,
            },
        );
        projected.pay_projected_life(p, 2, &game.state.entry_batch.as_ref().unwrap().moves);
        assert_eq!(projected.object(pool).zone, Zone::Library);
        let expected_life = if other_cards == 0 { 18 } else { 20 };
        assert_eq!(projected.life(p), expected_life);
        for &forest in &forests {
            assert_eq!(projected.object(forest).zone, Zone::Exile);
        }

        let action = Action::ChooseEntryLife { pay: true };
        let index = game
            .legal_actions()
            .iter()
            .position(|a| *a == action)
            .unwrap();
        let checkpoint = game.snapshot();
        let mut indexed = game.clone();
        indexed.apply_index(p, game.revision(), index).unwrap();
        act(&mut game, action.clone());
        assert_eq!(indexed.snapshot(), game.snapshot());
        assert_eq!(game.object_view(pool).zone, Zone::Battlefield);
        assert!(!game.object_view(pool).tapped);
        assert_eq!(game.state.players[p.index()].life, expected_life);
        for forest in forests {
            assert_eq!(game.object_view(forest).zone, Zone::Exile);
        }
        indexed.restore(&checkpoint).unwrap();
        assert!(indexed.legal_actions().contains(&action));
    }
}
