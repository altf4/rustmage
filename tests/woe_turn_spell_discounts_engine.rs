//! Resolution captures a public generic discount for the remainder of the turn.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const WHITE_BLUE: u8 = 0b00011;
const BLACK_RED: u8 = 0b01100;

const LOSE_ONE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Player {
            opponent_only: false,
        })),
        &[Instruction::LoseLife(PlayerSet::Targeted, 1)],
    ),
    ..CardRules::EMPTY
};
const GAIN_ONE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::GainLife(PlayerSet::Controller, 1)],
    ),
    ..CardRules::EMPTY
};
const VANILLA: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    ..CardRules::EMPTY
};
const fn discount_ability(program: &'static [Instruction]) -> ActivatedAbility {
    ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            colored: [0; 6],
            generic: 0,
            alternatives: &[],
            additional: None,
            mana_component: false,
            tap: true,
            sacrifice: false,
            exile_self: false,
            discard: false,
            life: 0,
            remove: None,
            loyalty: None,
        },
        targets: TargetPlan::NONE,
        program,
        sorcery: true,
        once_per_turn: false,
    }
}
const LOSS_DISCOUNT: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    activated: &[discount_ability(&[
        Instruction::ReduceColoredSpellCostsThisTurn {
            colors: BLACK_RED,
            amount: AmountValue::LifeLostThisTurn,
        },
    ])],
    ..CardRules::EMPTY
};
const GAIN_DISCOUNT: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    activated: &[discount_ability(&[
        Instruction::ReduceColoredSpellCostsThisTurn {
            colors: WHITE_BLUE,
            amount: AmountValue::LifeGainedThisTurn,
        },
    ])],
    ..CardRules::EMPTY
};

fn fixture(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn activate_discount(game: &mut Game, source: CardId) {
    act(
        game,
        Action::Activate {
            card: source,
            ability: 0,
            target: None,
            payment: [0; 6],
            counter_payment: Counters::default(),
        },
    );
}

fn resolve_top(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn finish_turn(game: &mut Game) {
    let turn = game.state.turn;
    for _ in 0..48 {
        if game.state.turn != turn {
            return;
        }
        choose(game, |action| {
            matches!(
                action,
                Action::Pass | Action::FinishAttackers | Action::FinishBlockers
            )
        });
    }
    panic!("turn did not end");
}

fn assert_discount(game: &Game, player: Player, expected: &[ColoredSpellReductionView]) {
    for viewer in [player, player.other()] {
        let observation = game.observe(viewer);
        assert_eq!(
            observation.players[player.index()].colored_spell_reductions,
            expected,
        );
        assert!(
            observation.players[player.other().index()]
                .colored_spell_reductions
                .is_empty()
        );
    }
}

#[test]
fn the_discount_freezes_life_loss_at_resolution_and_survives_source_departure() {
    let _rules = fixture_rules::activate(&[
        (fixture("Bear Cub"), &LOSS_DISCOUNT),
        (fixture("Shock"), &LOSE_ONE),
        (fixture("Opt"), &GAIN_ONE),
        (fixture("Bolg of the North"), &VANILLA),
    ]);
    for player in [Player::P0, Player::P1] {
        let mut root = scenario(player);
        let source = put(&mut root, player, "Bear Cub", Zone::Battlefield);
        let before = put(&mut root, player, "Shock", Zone::Hand);
        let gain = put(&mut root, player, "Opt", Zone::Hand);
        let during = put(&mut root, player.other(), "Shock", Zone::Hand);
        let removal = put(&mut root, player.other(), "Murder", Zone::Hand);
        let after = put(&mut root, player, "Shock", Zone::Hand);
        let spell = put(&mut root, player, "Bolg of the North", Zone::Hand);
        let opponent_adventure = put(&mut root, player.other(), "Gingerbread Hunter", Zone::Hand);
        put(&mut root, player.other(), "Hill Giant", Zone::Battlefield);
        cast(&mut root, before, Some(Target::Player(player)));
        settle(&mut root);
        cast(&mut root, gain, None);
        settle(&mut root);
        assert_eq!(root.state.players[player.index()].life, STARTING_LIFE);

        activate_discount(&mut root, source);
        assert!(root.object_view(source).tapped);
        act(&mut root, Action::Pass);
        let snapshot = root.snapshot();
        let revision = root.revision();
        let index = root
            .legal_actions()
            .iter()
            .position(|action| *action == Action::Pass)
            .unwrap();
        let mut raw = root.clone();
        let mut indexed = root.clone();
        let mut responded = root;
        act(&mut raw, Action::Pass);
        indexed
            .apply_index(player.other(), revision, index)
            .unwrap();
        assert_eq!(raw.snapshot(), indexed.snapshot());
        assert_discount(
            &raw,
            player,
            &[ColoredSpellReductionView {
                colors: BLACK_RED,
                generic: 1,
            }],
        );
        indexed.restore(&snapshot).unwrap();
        assert_discount(&indexed, player, &[]);
        act(&mut indexed, Action::Pass);
        assert_eq!(indexed.snapshot(), raw.snapshot());

        cast(&mut responded, during, Some(Target::Player(player)));
        resolve_top(&mut responded);
        act(&mut responded, Action::Pass);
        cast(&mut responded, removal, Some(Target::Card(source)));
        settle(&mut responded);
        assert_eq!(responded.object_view(source).zone, Zone::Graveyard);
        cast(&mut responded, after, Some(Target::Player(player)));
        settle(&mut responded);
        assert_eq!(
            responded.state.players[player.index()].life,
            STARTING_LIFE - 2
        );
        assert_discount(
            &responded,
            player,
            &[ColoredSpellReductionView {
                colors: BLACK_RED,
                generic: 2,
            }],
        );

        let mut other = responded.clone();
        other.state.players[player.other().index()].mana = [0, 0, 1, 0, 0, 0];
        act(&mut other, Action::Pass);
        reject(
            &mut other,
            Action::AnnounceAdventure {
                card: opponent_adventure,
            },
        );

        // Bolg costs {3}{B}{R}. His two colors both match the same discount.
        act(&mut responded, Action::AnnounceCast { card: spell });
        reject(
            &mut responded,
            Action::PayCosts {
                payment: [0, 0, 1, 1, 0, 0],
            },
        );
        reject(
            &mut responded,
            Action::PayCosts {
                payment: [0, 0, 0, 1, 2, 0],
            },
        );
        act(
            &mut responded,
            Action::PayCosts {
                payment: [0, 0, 1, 1, 1, 0],
            },
        );
        settle(&mut responded);
        assert_eq!(responded.object_view(spell).zone, Zone::Battlefield);
        finish_turn(&mut responded);
        assert_discount(&responded, player, &[]);
    }
}

#[test]
fn discounts_stack_and_use_the_selected_adventure_face() {
    let _rules = fixture_rules::activate(&[
        (fixture("Bear Cub"), &LOSS_DISCOUNT),
        (fixture("Grizzly Bears"), &GAIN_DISCOUNT),
        (fixture("Shock"), &LOSE_ONE),
        (fixture("Opt"), &GAIN_ONE),
        (fixture("Armament Dragon"), &VANILLA),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);
    let first = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let second = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let white_blue = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let loss = put(&mut game, player, "Shock", Zone::Hand);
    let gain = put(&mut game, player, "Opt", Zone::Hand);
    let gain_after = put(&mut game, player, "Opt", Zone::Hand);
    let multi = put(&mut game, player, "Armament Dragon", Zone::Hand);
    let adventure = put(&mut game, player, "Glamdring, Foe-hammer", Zone::Hand);

    cast(&mut game, loss, Some(Target::Player(player)));
    settle(&mut game);
    cast(&mut game, gain, None);
    settle(&mut game);
    for source in [first, second, white_blue] {
        activate_discount(&mut game, source);
        settle(&mut game);
    }
    cast(&mut game, gain_after, None);
    settle(&mut game);
    assert_discount(
        &game,
        player,
        &[
            ColoredSpellReductionView {
                colors: WHITE_BLUE,
                generic: 1,
            },
            ColoredSpellReductionView {
                colors: BLACK_RED,
                generic: 2,
            },
        ],
    );

    // The white-black-green Dragon combines both discounts even without static sources.
    act(&mut game, Action::AnnounceCast { card: multi });
    act(
        &mut game,
        Action::PayCosts {
            payment: [1, 0, 1, 0, 1, 0],
        },
    );
    settle(&mut game);

    // The blue Adventure combines the remaining turn discount with a live cost
    // reducer. Its colorless physical face gets neither.
    put(&mut game, player, "Mocking Sprite", Zone::Battlefield);
    let mut underfunded = game.clone();
    underfunded.state.players[player.index()].mana = [0, 1, 0, 0, 0, 0];
    reject(&mut underfunded, Action::AnnounceCast { card: adventure });
    act(&mut game, Action::AnnounceAdventure { card: adventure });
    reject(
        &mut game,
        Action::PayCosts {
            payment: [0, 1, 0, 0, 0, 0],
        },
    );
    act(
        &mut game,
        Action::PayCosts {
            payment: [0, 1, 0, 0, 1, 0],
        },
    );
    let spell = game.spell_characteristics().pop().unwrap();
    assert_eq!(spell.face, CastFace::Adventure);
    assert_eq!(spell.mana_value, 4);
    settle(&mut game);
    assert_eq!(game.object_view(adventure).zone, Zone::Exile);
}
