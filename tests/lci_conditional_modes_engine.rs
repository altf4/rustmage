//! Conditional modal options are checked when announcing the spell.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const DAMAGE: SpellMode = SpellMode {
    text: "Deal 3 damage to a creature",
    targets: TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
    program: &[Instruction::Damage(3)],
};
const DRAIN: SpellMode = SpellMode {
    text: "Opponent loses 1 life",
    targets: TargetPlan::NONE,
    program: &[Instruction::LoseLife(PlayerSet::Opponent, 1)],
};
const BOTH: SpellMode = SpellMode {
    text: "Choose both",
    targets: DAMAGE.targets,
    program: &[
        Instruction::Damage(3),
        Instruction::LoseLife(PlayerSet::Opponent, 1),
    ],
};

const DESCEND_MODE: CardRules = CardRules {
    effect: Effect::Modal(&[DAMAGE, DRAIN, BOTH]),
    extras: &CardExtras {
        cast_mode_requirements: &[(2, StatValue::DescendedThisTurn)],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const GRAVEYARD_MODE: CardRules = CardRules {
    effect: Effect::Modal(&[DAMAGE, DRAIN, BOTH]),
    extras: &CardExtras {
        cast_mode_requirements: &[(2, StatValue::GraveyardPermanentTypes)],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const GUARDED_ONLY: CardRules = CardRules {
    effect: Effect::Modal(&[DAMAGE, DRAIN]),
    extras: &CardExtras {
        cast_mode_requirements: &[(1, StatValue::GraveyardPermanentTypes)],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

fn named(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn modes(game: &Game) -> Vec<u16> {
    game.legal_actions()
        .into_iter()
        .filter_map(|action| match action {
            Action::ChooseMode { mode } => Some(mode),
            _ => None,
        })
        .collect()
}

#[test]
fn a_historical_descend_unlocks_the_combined_mode_in_the_same_turn() {
    let _rules = fixture_rules::activate(&[(named("Negate"), &DESCEND_MODE)]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let spell = put(&mut game, player, "Negate", Zone::Hand);
        let first = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let second = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
        let murder = put(&mut game, player, "Murder", Zone::Hand);
        let mut early = game.clone();
        act(&mut early, Action::AnnounceCast { card: spell });
        assert_eq!(modes(&early), [0, 1]);
        reject(&mut early, Action::ChooseMode { mode: 2 });

        cast(&mut game, murder, Some(Target::Card(first)));
        settle(&mut game);
        game.move_object(first, Zone::Exile);
        act(&mut game, Action::AnnounceCast { card: spell });
        assert_eq!(modes(&game), [0, 1, 2]);
        act(&mut game, Action::ChooseMode { mode: 2 });
        act(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(second),
            },
        );
        pay(&mut game);
        settle(&mut game);
        assert_eq!(game.object_view(second).zone, Zone::Graveyard);
        assert_eq!(
            game.state.players[player.other().index()].life,
            STARTING_LIFE - 1
        );
    }
}

#[test]
fn selecting_a_mode_locks_it_even_when_the_current_graveyard_changes() {
    let _rules = fixture_rules::activate(&[(named("Negate"), &GRAVEYARD_MODE)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Negate", Zone::Hand);
    let grave = put(&mut game, player, "Forest", Zone::Graveyard);
    let target = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    act(&mut game, Action::AnnounceCast { card: spell });
    assert_eq!(modes(&game), [0, 1, 2]);
    act(&mut game, Action::ChooseMode { mode: 2 });
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    pay(&mut game);

    game.move_object(grave, Zone::Exile);
    settle(&mut game);
    assert_eq!(game.object_view(target).zone, Zone::Graveyard);
    assert_eq!(
        game.state.players[player.other().index()].life,
        STARTING_LIFE - 1
    );
}

#[test]
fn announcement_requires_an_eligible_mode_with_legal_targets() {
    let _rules = fixture_rules::activate(&[(named("Negate"), &GUARDED_ONLY)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Negate", Zone::Hand);
    let announce = Action::AnnounceCast { card: spell };
    reject(&mut game, announce.clone());

    put(&mut game, player, "Forest", Zone::Graveyard);
    act(&mut game, announce);
    assert_eq!(modes(&game), [1]);
    act(&mut game, Action::ChooseMode { mode: 1 });
    pay(&mut game);
    settle(&mut game);
    assert_eq!(
        game.state.players[player.other().index()].life,
        STARTING_LIFE - 1
    );
}
