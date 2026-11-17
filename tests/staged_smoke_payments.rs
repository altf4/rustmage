//! Smoke payment choices use the current declaration and actual contributors.
mod support;
use rustmage::{Card, policy};
use rustmage_core::engine::*;
use support::*;

fn smoke_step(game: &mut Game) -> Action {
    let view = game.observe(game.actor().unwrap());
    let index = policy::smoke_index_from_observation(&view, &view.actions);
    let action = view.actions[index].clone();
    let mut indexed = game.clone();
    indexed
        .apply_index(view.actor.unwrap(), view.revision, index)
        .unwrap();
    act(game, action.clone());
    assert_eq!(game.snapshot(), indexed.snapshot());
    action
}

#[test]
fn declared_back_face_colors_take_precedence_over_other_hand_cards() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let spell = put(
            &mut game,
            player,
            "Norman Osborn // Green Goblin",
            Zone::Hand,
        );
        put(&mut game, player, "Cancel", Zone::Hand);
        put(&mut game, player, "Burglar Rat", Zone::Hand);
        let island = put(&mut game, player, "Island", Zone::Battlefield);
        let swamp = put(&mut game, player, "Swamp", Zone::Battlefield);
        let forest = put(&mut game, player, "Forest", Zone::Battlefield);
        let flexible = put(&mut game, player, "Oscorp Industries", Zone::Battlefield);
        game.state.players[player.index()].mana = [0; 6];
        act(&mut game, Action::AnnounceBack { card: spell });
        assert_eq!(
            game.observe(player).payment_request.unwrap().required,
            [0, 1, 1, 1, 0, 0]
        );
        let first = smoke_step(&mut game);
        assert!(matches!(first, Action::TapMana { card } if card == island || card == swamp));
        let second = smoke_step(&mut game);
        assert!(matches!(second, Action::TapMana { card } if card == island || card == swamp));
        assert_ne!(first, second);
        let third = smoke_step(&mut game);
        assert!(
            matches!(third, Action::ActivateMana { card, production, .. }
            if card == flexible && production == [0, 0, 0, 1, 0, 0])
        );
        assert_eq!(smoke_step(&mut game), Action::TapMana { card: forest });
        assert_eq!(
            smoke_step(&mut game),
            Action::PayCosts {
                payment: [0, 1, 1, 1, 1, 0]
            }
        );
        let spell = game
            .state
            .stack
            .iter()
            .find(|item| item.source.is_some_and(|source| source.id == spell))
            .unwrap();
        assert_eq!(spell.cast.unwrap().face, CastFace::Back);
        assert_eq!(
            spell.source.unwrap().card,
            Card::from_name("Green Goblin").unwrap()
        );
        assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    }
}

#[test]
fn an_offered_contribution_is_committed_before_final_mana_payment() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let spell = put(&mut game, player, "Web of Life and Destiny", Zone::Hand);
        let creature = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 8];
        act(&mut game, Action::AnnounceCast { card: spell });
        act(&mut game, Action::BeginConvoke);
        assert!(
            matches!(smoke_step(&mut game), Action::ChooseConvoker { object, .. }
            if object.card == creature)
        );
        assert_eq!(
            smoke_step(&mut game),
            Action::PayCosts {
                payment: [0, 0, 0, 0, 0, 7]
            }
        );
        assert!(game.object_view(creature).tapped);
        assert_eq!(game.object_view(spell).zone, Zone::Stack);
        assert_eq!(game.state.players[player.index()].mana, [0, 0, 0, 0, 0, 1]);
    }
}
