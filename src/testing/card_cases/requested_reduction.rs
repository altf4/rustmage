//! Focused printed outcomes; shared engine invariants live in core tests.
//! Paid origin, plot, and Adventure cost reductions.
use super::big_helpers::{cast, checked, settle, setup};
use super::*;

#[test]
fn doc_reduces_exile_casts_and_plot_costs_but_not_hand_spells() {
    {
        let player = Player::P0;
        let mut game = setup(player);
        put(
            &mut game,
            player,
            "Doc Aurlock, Grizzled Genius",
            Zone::Battlefield,
        );
        let hand = put(&mut game, player, "Stagecoach Security", Zone::Hand);
        let creature = put(&mut game, player, "Llanowar Elves", Zone::Battlefield);
        game.airbend_objects(&[creature]);
        game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 0];
        let action = Action::AnnounceAirbend {
            card: creature,
            face: CastFace::Normal,
            flash_surcharge: false,
        };
        assert!(game.legal_actions().contains(&action));
        checked(&mut game, action);
        checked(&mut game, Action::PayCosts { payment: [0; 6] });
        settle(&mut game);
        game.state.players[player.index()].mana = [1, 0, 0, 0, 0, 1];
        assert!(
            !game
                .legal_actions()
                .contains(&Action::AnnounceCast { card: hand })
        );
        checked(&mut game, Action::AnnouncePlot { card: hand });
        assert_eq!(
            game.plot_payment().unwrap().cost,
            ManaCost {
                colored: [1, 0, 0, 0, 0, 0],
                generic: 1
            }
        );
        checked(
            &mut game,
            Action::PayPlot {
                payment: [1, 0, 0, 0, 0, 1],
            },
        );
        assert_eq!(game.object_view(hand).zone, Zone::Exile);
        let flashback = put(&mut game, player, "Think Twice", Zone::Graveyard);
        game.state.players[player.index()].mana = [0, 1, 0, 0, 0, 0];
        checked(&mut game, Action::AnnounceFlashback { card: flashback });
        checked(
            &mut game,
            Action::PayCosts {
                payment: [0, 1, 0, 0, 0, 0],
            },
        );
        settle(&mut game);
        assert_eq!(game.object_view(flashback).zone, Zone::Exile);
    }
}

#[test]
fn hearth_counts_qualifying_graveyard_cards_once_and_stoke_discards_then_draws() {
    {
        let player = Player::P0;
        let mut game = setup(player);
        let elemental = put(&mut game, player, "Hearth Elemental", Zone::Hand);
        put(&mut game, player, "Opt", Zone::Graveyard);
        put(&mut game, player, "Callous Sell-Sword", Zone::Graveyard);
        put(&mut game, player.other(), "Opt", Zone::Graveyard);
        game.state.players[player.index()].mana = [0, 0, 0, 1, 0, 3];
        cast(&mut game, elemental, None, &[]);
        assert_eq!(game.state.players[player.index()].mana, [0; 6]);
        settle(&mut game);
        assert_eq!(
            (game.stats(elemental).power, game.stats(elemental).toughness),
            (4, 5)
        );

        let mut game = setup(player);
        let elemental = put(&mut game, player, "Hearth Elemental", Zone::Hand);
        let discarded = put(&mut game, player, "Bear Cub", Zone::Hand);
        game.state.players[player.index()].mana = [0, 0, 0, 1, 0, 1];
        checked(&mut game, Action::AnnounceAdventure { card: elemental });
        let payment = game
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut game, payment);
        settle(&mut game);
        assert_eq!(game.object_view(discarded).zone, Zone::Graveyard);
        assert_eq!(game.hand_count(player), 2);
        assert_eq!(game.object_view(elemental).zone, Zone::Exile);
        assert!(game.can_play_adventure(player, elemental));
    }
}
