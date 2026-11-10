//! Simultaneous return-to-hand receipts count each actual opposing controller once.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const TOKEN: Card = Card::test_index(5286);
const SWEEPER: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::ReturnBattlefieldCountOpposingControllers(&[BattlefieldFilter {
                kinds: 1,
                relation: -1,
                nonland: false,
                tapped: None,
                without: Keywords::NONE,
                subtype: None,
                max_mana: u32::MAX,
            }]),
            Instruction::CreateTokensScaled {
                token: TOKEN,
                amount: AmountValue::Multiple {
                    value: &AmountValue::MovedCount,
                    factor: 2,
                },
                tapped: false,
            },
        ],
    ),
    ..CardRules::EMPTY
};

fn sweep(game: &mut Game, player: Player) {
    let spell = put(game, player, "Divination", Zone::Hand);
    cast(game, spell, None);
}

fn tokens(game: &Game, player: Player) -> usize {
    game.state
        .objects
        .iter()
        .filter(|object| {
            object.zone == Zone::Battlefield && object.controller == player && object.card == TOKEN
        })
        .count()
}

#[test]
fn all_creatures_return_to_their_owners_and_one_opponent_counts_once() {
    let _rules = fixture_rules::activate(&[(Card::Divination, &SWEEPER)]);
    for player in [Player::P0, Player::P1] {
        let opponent = player.other();
        let mut game = scenario(player);
        let stolen = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
        let stolen_object = game.object_ref(stolen);
        game.change_controller(stolen_object, opponent);
        let opposing = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
        let friendly = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let equipment = put(&mut game, opponent, "Short Bow", Zone::Battlefield);
        sweep(&mut game, player);
        act(&mut game, Action::Pass);
        let revision = game.revision();
        let index = game
            .legal_actions()
            .iter()
            .position(|a| *a == Action::Pass)
            .unwrap();
        let mut indexed = game.clone();
        act(&mut game, Action::Pass);
        indexed.apply_index(opponent, revision, index).unwrap();
        assert_eq!(game.snapshot(), indexed.snapshot());
        assert_eq!(game.object_view(stolen).controller, player);
        for id in [stolen, opposing, friendly] {
            assert_eq!(game.object_view(id).zone, Zone::Hand);
        }
        assert_eq!(game.object_view(equipment).zone, Zone::Battlefield);
        assert_eq!(tokens(&game, player), 2);
    }
}

#[test]
fn a_token_return_counts_even_if_the_token_ceases_to_exist() {
    let _rules = fixture_rules::activate(&[(Card::Divination, &SWEEPER)]);
    for player in [Player::P0, Player::P1] {
        let opponent = player.other();
        let mut game = scenario(player);
        game.create_tokens(opponent, TOKEN, 1, false);
        assert_eq!(tokens(&game, opponent), 1);
        sweep(&mut game, player);
        settle(&mut game);
        assert_eq!(tokens(&game, opponent), 0);
        assert_eq!(tokens(&game, player), 2);
        assert_eq!(game.hand_count(opponent), 0);
    }
}

#[test]
fn prevented_and_redirected_moves_and_friendly_control_do_not_count() {
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|slot| slot.set(None));
            zone_replacements::TEST_EXILE_INSTEAD_OF_HAND.with(|slot| slot.set(None));
        }
    }
    let _reset = Reset;
    let _rules = fixture_rules::activate(&[(Card::Divination, &SWEEPER)]);
    for player in [Player::P0, Player::P1] {
        let opponent = player.other();
        let mut game = scenario(player);
        let prevented = put(&mut game, opponent, "Grizzly Bears", Zone::Battlefield);
        let diverted = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
        let friendly = put(&mut game, opponent, "Hill Giant", Zone::Battlefield);
        let friendly_object = game.object_ref(friendly);
        game.change_controller(friendly_object, player);
        zone_replacements::TEST_PREVENT_ZONE_MOVE
            .with(|slot| slot.set(Some(game.object_ref(prevented))));
        zone_replacements::TEST_EXILE_INSTEAD_OF_HAND
            .with(|slot| slot.set(Some(game.object_ref(diverted))));

        sweep(&mut game, player);
        settle(&mut game);
        assert_eq!(game.object_view(prevented).zone, Zone::Battlefield);
        assert_eq!(game.object_view(diverted).zone, Zone::Exile);
        assert_eq!(game.object_view(friendly).zone, Zone::Hand);
        assert_eq!(game.object_view(friendly).controller, opponent);
        assert_eq!(tokens(&game, player), 0);
    }
}
