//! A scheduled or skipped extra turn is not an extra turn begun.
use crate::support::*;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};

const OBSERVER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::LifeGained {
            first_only: false,
            own_turn: false,
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::If {
                guard: StepGuard::NoExtraTurnBegunThisGame,
                skip: 1,
            },
            Instruction::ExtraTurn {
                power_up_prohibited: false,
            },
        ],
    }],
    ..CardRules::EMPTY
};
const GAIN: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::GainLife(PlayerSet::Controller, 1)],
    ),
    ..CardRules::EMPTY
};

fn settle_all(game: &mut Game) {
    for _ in 0..48 {
        if game.state.stack.is_empty()
            && game.state.resolving.is_none()
            && game.state.pending_triggers.is_empty()
            && game.state.ordering_triggers.is_empty()
            && game.state.pending == Pending::Priority
        {
            return;
        }
        choose(game, |action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. })
        });
    }
    panic!("extra-turn trigger did not finish");
}

fn finish_turn(game: &mut Game) {
    game.state.step = Step::End;
    game.give_priority();
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn observed(game: &Game, player: Player) -> bool {
    for viewer in [Player::P0, Player::P1] {
        assert_eq!(
            game.observe(viewer).players[player.index()].has_begun_extra_turn_this_game,
            game.observe(player).players[player.index()].has_begun_extra_turn_this_game,
        );
    }
    game.observe(player).players[player.index()].has_begun_extra_turn_this_game
}

#[test]
fn trigger_resolution_allows_more_scheduled_turns_until_one_actually_begins() {
    let _rules = fixture_rules::activate(&[
        (Card::from_name("Bear Cub").unwrap(), &OBSERVER),
        (Card::from_name("Opt").unwrap(), &GAIN),
    ]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        put(&mut game, player, "Bear Cub", Zone::Battlefield);
        assert!(!observed(&game, player));
        for expected in [1, 2] {
            let spell = put(&mut game, player, "Opt", Zone::Hand);
            cast(&mut game, spell, None);
            settle_all(&mut game);
            assert_eq!(
                game.observe(player).turn_schedule.unwrap().pending.len(),
                expected
            );
            assert!(!observed(&game, player));
        }

        finish_turn(&mut game);
        assert_eq!(game.state.active, player);
        assert!(observed(&game, player));
        assert!(!observed(&game, player.other()));
        let pending = game.observe(player).turn_schedule.unwrap().pending.len();
        game.state.players[player.index()].mana = [8; 6];
        let later = put(&mut game, player, "Opt", Zone::Hand);
        cast(&mut game, later, None);
        settle_all(&mut game);
        assert_eq!(
            game.observe(player).turn_schedule.unwrap().pending.len(),
            pending
        );
        assert!(observed(&game, player));
    }
}

#[test]
fn skipped_extra_turn_does_not_set_game_long_history() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        game.add_extra_turn(player, false);
        game.state.turn_skips.get_or_insert_default()[player.index()] = 1;
        assert!(!observed(&game, player));
        finish_turn(&mut game);
        assert_eq!(game.state.active, player.other());
        assert!(!observed(&game, player));
        game.add_extra_turn(player, false);
        finish_turn(&mut game);
        assert_eq!(game.state.active, player);
        assert!(observed(&game, player));
    }
}

#[test]
fn extra_turn_beginning_has_identical_raw_index_and_restored_history() {
    for player in [Player::P0, Player::P1] {
        let mut parent = scenario(player);
        parent.add_extra_turn(player, false);
        parent.state.step = Step::End;
        parent.give_priority();
        let checkpoint = parent.snapshot();
        let revision = parent.revision();
        let action = Action::Pass;
        let index = parent
            .legal_actions()
            .iter()
            .position(|row| *row == action)
            .unwrap();
        let mut raw = parent.clone();
        let mut indexed = parent.clone();
        drop(parent);
        act(&mut raw, action.clone());
        indexed.apply_index(player, revision, index).unwrap();
        assert_eq!(raw.snapshot(), indexed.snapshot());
        assert!(!observed(&raw, player));
        act(&mut raw, Action::Pass);
        act(&mut indexed, Action::Pass);
        assert_eq!(raw.snapshot(), indexed.snapshot());
        assert!(observed(&raw, player));

        indexed.restore(&checkpoint).unwrap();
        assert_eq!(indexed.snapshot(), checkpoint);
        assert!(!observed(&indexed, player));
        act(&mut indexed, Action::Pass);
        act(&mut indexed, Action::Pass);
        assert!(observed(&indexed, player));
    }
}
