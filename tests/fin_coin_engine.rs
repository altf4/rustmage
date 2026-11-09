mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const FIRST_FLIPS: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::FirstCoinFlipsHeads],
    }],
    ..CardRules::EMPTY
};

const fn skip_turns(program: &'static [Instruction]) -> CardRules {
    CardRules {
        effect: Effect::Spell(
            TargetPlan::Single(TargetSpec::one(TargetRule::Player {
                opponent_only: true,
            })),
            program,
        ),
        ..CardRules::EMPTY
    }
}

const FIVE_FLIPS: CardRules = skip_turns(&[Instruction::FlipCoinsSkipTurns { coins: 5 }]);
const ZERO_FLIPS: CardRules = skip_turns(&[Instruction::FlipCoinsSkipTurns { coins: 0 }]);

const WIN_LIFE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::IfCoinFlipWon { skip: 1 },
            Instruction::GainLife(PlayerSet::Controller, 3),
        ],
    ),
    ..CardRules::EMPTY
};

fn cast_named(game: &mut Game, player: Player, name: &str, target: Option<Target>) {
    let id = put(game, player, name, Zone::Hand);
    cast(game, id, target);
}

#[test]
fn first_positive_batch_is_forced_and_next_flip_replays_from_owned_rng() {
    let _rules = fixture_rules::activate(&[
        (Card::from_name("Swiftfoot Boots").unwrap(), &FIRST_FLIPS),
        (Card::from_name("Opt").unwrap(), &FIVE_FLIPS),
        (Card::from_name("Think Twice").unwrap(), &WIN_LIFE),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);
    put(&mut game, player, "Swiftfoot Boots", Zone::Battlefield);
    let original_rng = game.state.rng;

    cast_named(
        &mut game,
        player,
        "Opt",
        Some(Target::Player(player.other())),
    );
    settle(&mut game);
    assert_eq!(game.state.turn_skips.as_deref(), Some(&[0, 5]));
    assert_eq!(game.state.rng, original_rng);
    let facts = game.observe(player.other()).turn_facts.unwrap();
    assert!(facts[player.index()].coin_flipped_this_turn);
    assert!(!facts[player.other().index()].coin_flipped_this_turn);

    let life = game.state.players[player.index()].life;
    let mut expected_rng = original_rng;
    let won = expected_rng.below(2) == 1;
    cast_named(&mut game, player, "Think Twice", None);
    act(&mut game, Action::Pass);
    let before = game.snapshot();
    let index = game
        .legal_actions()
        .iter()
        .position(|action| *action == Action::Pass)
        .unwrap();
    let mut indexed = game.clone();
    indexed
        .apply_index(player.other(), indexed.revision(), index)
        .unwrap();
    assert_eq!(game.snapshot(), before);
    act(&mut game, Action::Pass);
    assert_eq!(game.snapshot(), indexed.snapshot());
    indexed.restore(&before).unwrap();
    act(&mut indexed, Action::Pass);
    assert_eq!(game.snapshot(), indexed.snapshot());
    assert_eq!(game.state.rng, expected_rng);
    assert_eq!(
        game.state.players[player.index()].life,
        life + 3 * i128::from(won)
    );
}

#[test]
fn zero_flips_do_not_consume_the_first_event_and_a_late_provider_cannot_rewind_it() {
    let _rules = fixture_rules::activate(&[
        (Card::from_name("Swiftfoot Boots").unwrap(), &FIRST_FLIPS),
        (Card::from_name("Opt").unwrap(), &ZERO_FLIPS),
        (Card::from_name("Think Twice").unwrap(), &WIN_LIFE),
    ]);
    let player = Player::P1;
    let mut game = scenario(player);
    let initial_rng = game.state.rng;
    cast_named(
        &mut game,
        player,
        "Opt",
        Some(Target::Player(player.other())),
    );
    settle(&mut game);
    assert!(
        !game.state.players[player.index()]
            .turn_facts
            .coin_flipped_this_turn
    );
    assert_eq!(game.state.rng, initial_rng);

    cast_named(&mut game, player, "Think Twice", None);
    settle(&mut game);
    assert!(
        game.state.players[player.index()]
            .turn_facts
            .coin_flipped_this_turn
    );
    assert_ne!(game.state.rng, initial_rng);
    put(&mut game, player, "Swiftfoot Boots", Zone::Battlefield);
    let rng = game.state.rng;
    cast_named(&mut game, player, "Think Twice", None);
    settle(&mut game);
    assert_ne!(game.state.rng, rng);

    let turn = game.state.turn;
    for _ in 0..28 {
        if game.state.turn != turn {
            break;
        }
        choose(&mut game, |action| {
            matches!(action, Action::Pass | Action::FinishAttackers)
        });
    }
    assert_eq!(game.state.turn, turn + 1);
    assert!(
        !game.state.players[player.index()]
            .turn_facts
            .coin_flipped_this_turn
    );
}
