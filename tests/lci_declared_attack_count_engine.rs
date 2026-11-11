//! Activation gates can read actual declarations across multiple combats.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const COUNT: StatValue = StatValue::DeclaredAttackersThisTurn;
const TEMPLE: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::StatAtLeast {
            value: COUNT,
            minimum: 3,
        },
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            tap: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn declare(game: &mut Game, attackers: &[CardId]) {
    game.begin_step(Step::BeginCombat);
    game.give_priority();
    pass_pair(game);
    for &card in attackers {
        act(game, Action::Attack { card });
    }
    act(game, Action::FinishAttackers);
}

fn finish_combat(game: &mut Game) {
    for _ in 0..16 {
        if game.state.step == Step::Main2 && game.state.pending == Pending::Priority {
            return;
        }
        choose(game, |action| {
            matches!(action, Action::Pass | Action::FinishBlockers)
        });
    }
    panic!("combat did not finish");
}

#[test]
fn a_late_source_reads_both_combats_and_skips_a_token_that_entered_attacking() {
    let name = "Short Bow";
    let _rules = fixture_rules::activate(&[(Card::from_name(name).unwrap(), &TEMPLE)]);
    for player in [Player::P0, Player::P1] {
        let opponent = player.other();
        let mut game = scenario(player);
        let first = put(&mut game, player, "Miner's Guidewing", Zone::Battlefield);
        let second = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        declare(&mut game, &[first, second]);

        let attacking = game.create_token_batch_capture(
            player,
            Card::from_catalogue_id(5326),
            1,
            true,
            Some(&[CombatDefender::Player(opponent)]),
            true,
        );
        assert_eq!(attacking.len(), 1);
        assert!(game.object_view(attacking[0].card).attacking);
        assert_eq!(game.stat_value(COUNT, game.object_ref(first), player), 2);
        assert_eq!(game.stat_value(COUNT, game.object_ref(first), opponent), 0);
        finish_combat(&mut game);

        let temple = put(&mut game, player, name, Zone::Battlefield);
        let reference = game.object_ref(temple);
        let activate = Action::AnnounceActivate {
            card: temple,
            ability: 0,
            counter_payment: Counters::default(),
        };
        reject(&mut game, activate.clone());
        assert!(!game.object_view(first).tapped);
        declare(&mut game, &[first]);
        assert_eq!(game.stat_value(COUNT, reference, player), 3);
        finish_combat(&mut game);
        assert!(game.legal_actions().contains(&activate));

        game.state.step = Step::End;
        pass_pair(&mut game);
        assert_eq!(game.state.active, opponent);
        game.state.priority = player;
        assert_eq!(game.stat_value(COUNT, reference, player), 0);
        reject(&mut game, activate);
    }
}
