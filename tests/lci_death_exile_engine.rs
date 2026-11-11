//! A resolved spell can replace actual later creature deaths using the
//! creatures' current characteristics and last battlefield controllers.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::engine::ability_world::CharacteristicWorld;
use rustmage_core::engine::funding::{FundingGoal, FundingState};
use rustmage_core::{cards::*, engine::*};
use support::*;

const SWEEP: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::GrowPlayerCreatures {
                players: PlayerSet::Each,
                power: -2,
                toughness: -2,
            },
            Instruction::ExileOpposingCreaturesThatWouldDieThisTurn,
        ],
    ),
    ..CardRules::EMPTY
};

#[test]
fn replacement_applies_to_the_sweep_and_to_later_opposing_creatures_until_cleanup() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Opt").unwrap(), &SWEEP)]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let opponent = player.other();
        let own = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
        let foreign = put(&mut game, opponent, "Grizzly Bears", Zone::Battlefield);
        let survivor = put(&mut game, opponent, "Hill Giant", Zone::Battlefield);
        let spell = put(&mut game, player, "Opt", Zone::Hand);
        cast(&mut game, spell, None);
        let snapshot = game.snapshot();
        settle(&mut game);
        assert_eq!(game.object_view(spell).zone, Zone::Graveyard);
        assert_eq!(game.object_view(own).zone, Zone::Graveyard);
        assert_eq!(game.object_view(foreign).zone, Zone::Exile);
        assert_eq!(stats(&game, survivor), (1, 1));
        assert_eq!(game.state.players[player.index()].descended_this_turn, 1);
        assert_eq!(game.state.players[opponent.index()].descended_this_turn, 0);

        let mut expected_flags = [false; 2];
        expected_flags[player.index()] = true;
        assert_eq!(
            game.observe(opponent).exile_opposing_creature_deaths,
            Some(expected_flags)
        );

        let later = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
        game.change_controller(game.object_ref(later), opponent);
        let plain = put(&mut game, opponent, "Swiftfoot Boots", Zone::Battlefield);
        assert_eq!(stats(&game, later), (2, 2));
        game.move_object(later, Zone::Graveyard);
        game.move_object(plain, Zone::Graveyard);
        assert_eq!(game.object_view(later).zone, Zone::Exile);
        assert_eq!(game.object_view(plain).zone, Zone::Graveyard);

        let turn = game.state.turn;
        for _ in 0..40 {
            if game.state.turn != turn {
                break;
            }
            choose(&mut game, |action| {
                matches!(
                    action,
                    Action::Pass | Action::FinishAttackers | Action::FinishBlockers
                )
            });
        }
        assert_ne!(game.state.turn, turn);
        assert_eq!(stats(&game, survivor), (3, 3));
        assert_eq!(game.observe(player).exile_opposing_creature_deaths, None);
        game.move_object(survivor, Zone::Graveyard);
        assert_eq!(game.object_view(survivor).zone, Zone::Graveyard);

        game.restore(&snapshot).unwrap();
        settle(&mut game);
        assert_eq!(game.object_view(own).zone, Zone::Graveyard);
        assert_eq!(game.object_view(foreign).zone, Zone::Exile);
    }
}

#[test]
fn projected_cost_departures_use_the_same_replacement_as_committed_moves() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Opt").unwrap(), &SWEEP)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Opt", Zone::Hand);
    cast(&mut game, spell, None);
    settle(&mut game);

    let own = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let foreign = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    let finality = put(&mut game, player, "Savannah Lions", Zone::Battlefield);
    game.add_counter(finality, CounterKind::Finality, 1);
    let cards = [own, foreign, finality];
    let objects = cards.map(|id| game.object_ref(id));
    let funding = FundingState::from_game(
        &game,
        &FundingGoal {
            actor: player,
            required: [0; 6],
            generic: 0,
            reserve: None,
            source: None,
        },
    );
    let mut projected = Vec::new();
    let snapshot = game.snapshot();
    funding.cost_departures(&objects, Zone::Graveyard, &mut |world| {
        projected.push(cards.map(|id| world.object(id).zone));
        false
    });
    assert_eq!(game.snapshot(), snapshot);
    assert_eq!(projected, [[Zone::Graveyard, Zone::Exile, Zone::Exile]]);
    for id in cards {
        game.move_object(id, Zone::Graveyard);
    }
    assert_eq!(projected[0], cards.map(|id| game.object_view(id).zone));
}
