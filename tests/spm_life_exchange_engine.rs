//! Paid life exchange scenarios using ordinary life replacement effects.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const EXCHANGE: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Player {
            opponent_only: true,
        })),
        program: &[
            Instruction::May { skip: 1 },
            Instruction::ExchangeLifeTotalsWithTarget {
                draw_controller_loss: true,
            },
        ],
    }],
    ..CardRules::EMPTY
};

fn life_totals(game: &Game) -> [i128; 2] {
    [Player::P0, Player::P1].map(|player| game.state.players[player.index()].life)
}

fn offer_exchange(game: &mut Game, player: Player, life: [i128; 2]) {
    for (state, total) in game.state.players.iter_mut().zip(life) {
        state.life = total;
    }
    let source = put(game, player, "Grizzly Bears", Zone::Hand);
    cast(game, source, None);
    act(game, Action::Pass);
    act(game, Action::Pass);
    reject(
        game,
        Action::ChooseTarget {
            target: Target::Player(player),
        },
    );
    act(
        game,
        Action::ChooseTarget {
            target: Target::Player(player.other()),
        },
    );
    act(game, Action::Pass);
    act(game, Action::Pass);
    assert!(
        matches!(game.state.pending, Pending::EffectOptional { player: owner, .. } if owner == player)
    );
}

#[test]
fn exchange_is_optional_draws_only_actual_controller_loss_and_needs_both_halves() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Grizzly Bears").unwrap(), &EXCHANGE)]);
    let player = Player::P0;
    for (initial, accept, prohibited, expected, draw) in [
        ([20, 16], false, false, [20, 16], 0),
        ([20, 16], true, false, [16, 20], 4),
        ([16, 20], true, false, [20, 16], 0),
        ([20, 20], true, false, [20, 20], 0),
        ([20, 16], true, true, [20, 16], 0),
        ([16, 20], true, true, [16, 20], 0),
    ] {
        let mut game = scenario(player);
        if prohibited {
            put(
                &mut game,
                player.other(),
                "Giant Cindermaw",
                Zone::Battlefield,
            );
        }
        offer_exchange(&mut game, player, initial);
        act(&mut game, Action::ChooseOptional { accept });
        settle(&mut game);
        assert_eq!(life_totals(&game), expected);
        assert_eq!(game.hand_count(player), draw);
        assert_eq!(game.hand_count(player.other()), 0);
        for (index, state) in game.state.players.iter().enumerate() {
            assert_eq!(
                state.life_lost_this_turn,
                (initial[index] - expected[index]).max(0) as u128
            );
            assert_eq!(
                state.life_gain_events_this_turn,
                u64::from(expected[index] > initial[index])
            );
        }
    }
}

#[test]
fn ordered_replacements_are_atomic_and_branch_without_changing_actual_loss() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Grizzly Bears").unwrap(), &EXCHANGE)]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let angel = put(&mut game, opponent, "Angel of Vitality", Zone::Battlefield);
    let crystal = put(&mut game, opponent, "The Wind Crystal", Zone::Battlefield);
    offer_exchange(&mut game, player, [20, 16]);
    act(&mut game, Action::ChooseOptional { accept: true });
    assert_eq!(
        game.state.pending,
        Pending::LifeGainReplacement { player: opponent }
    );
    assert_eq!(life_totals(&game), [20, 16]);
    assert_eq!(game.hand_count(player), 0);
    assert!(game.observe(player).life_gain_replacement_choice.is_none());
    let choice = game.observe(opponent).life_gain_replacement_choice.unwrap();
    assert_eq!(choice.amount, 4);
    assert_eq!(choice.effects.len(), 2);
    let action_for = |card| {
        let effect = choice
            .effects
            .iter()
            .find(|effect| effect.source.card == card)
            .unwrap();
        Action::ChooseReplacement {
            source: effect.source,
            ability: effect.ability,
        }
    };
    let add = action_for(angel);
    let double = action_for(crystal);
    let saved = game.snapshot();
    let revision = game.revision();
    assert_eq!(game.apply(player, add.clone()), Err(Error::WrongActor));
    assert_eq!(game.snapshot(), saved);
    assert_eq!(game.revision(), revision);
    let index = game
        .legal_actions()
        .iter()
        .position(|action| *action == add)
        .unwrap();
    let mut raw = game.clone();
    let mut indexed = game;
    act(&mut raw, add);
    indexed.apply_index(opponent, revision, index).unwrap();
    assert_eq!(indexed.snapshot(), raw.snapshot());
    assert_eq!(life_totals(&raw), [16, 26]);
    assert_eq!(raw.hand_count(player), 4);
    assert_eq!(raw.state.players[player.index()].life_lost_this_turn, 4);
    assert_eq!(
        raw.state.players[opponent.index()].life_gain_events_this_turn,
        1
    );

    indexed.restore(&saved).unwrap();
    assert_eq!(
        indexed.apply_index(opponent, revision, index),
        Err(Error::StaleRevision)
    );
    act(&mut indexed, double);
    assert_eq!(life_totals(&indexed), [16, 25]);
    assert_eq!(indexed.hand_count(player), 4);
}
