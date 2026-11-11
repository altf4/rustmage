//! Actual creature departures belong to their last battlefield controller.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const DEPARTURES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::GainLifeScaled(
            PlayerSet::Controller,
            AmountValue::Stat(StatValue::ControlledCreatureDeparturesThisTurn),
        )],
    ),
    ..CardRules::EMPTY
};

#[test]
fn tokens_replaced_deaths_and_bounced_creatures_count_for_the_previous_controller() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Opt").unwrap(), &DEPARTURES)]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let opponent = player.other();
        let stolen = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
        game.record_control(game.object_ref(stolen), player, None);
        let finality = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        game.add_counter(finality, CounterKind::Finality, 1);
        let token = game.state.objects.len() as CardId;
        game.create_tokens(player, Card::test_index(5323), 1, false);
        let enemy = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);

        let bounce = put(&mut game, player, "Unsummon", Zone::Hand);
        cast(&mut game, bounce, Some(Target::Card(stolen)));
        settle(&mut game);
        assert_eq!(game.object_view(stolen).zone, Zone::Hand);
        assert_eq!(game.object_view(stolen).owner, opponent);

        let sweeper = put(&mut game, player, "Day of Judgment", Zone::Hand);
        cast(&mut game, sweeper, None);
        settle(&mut game);
        assert_eq!(game.object_view(finality).zone, Zone::Exile);
        assert!(!game.on_battlefield(token));
        assert_eq!(game.object_view(enemy).zone, Zone::Graveyard);

        let count = put(&mut game, player, "Opt", Zone::Hand);
        cast(&mut game, count, None);
        settle(&mut game);
        assert_eq!(game.state.players[player.index()].life, STARTING_LIFE + 3);
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
        assert_eq!(
            game.stat_value(
                StatValue::ControlledCreatureDeparturesThisTurn,
                game.object_ref(count),
                player,
            ),
            0,
        );
    }
}
