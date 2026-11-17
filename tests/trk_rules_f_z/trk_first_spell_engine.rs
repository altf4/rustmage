//! The first successful cast is a game-long fact of that exact spell packet.
use crate::support::*;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};

const FIRST_CAST_BONUS: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::IfFirstSpellCastThisGame { skip: 1 },
            Instruction::GainLife(PlayerSet::Controller, 2),
        ],
    ),
    ..CardRules::EMPTY
};
const COPY: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::YourSpell)),
        &[Instruction::CopyTargetSpell {
            group: 0,
            retarget: false,
            exceptions: CopyExceptions::NONE,
        }],
    ),
    ..CardRules::EMPTY
};

fn rules() -> fixture_rules::Guard {
    fixture_rules::activate(&[
        (Card::from_name("Opt").unwrap(), &FIRST_CAST_BONUS),
        (Card::from_name("Unsummon").unwrap(), &COPY),
    ])
}

#[test]
fn first_cast_fact_stays_with_original_through_later_casts_and_copies_for_both_seats() {
    let _rules = rules();
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let first = put(&mut game, player, "Opt", Zone::Hand);
        let later = put(&mut game, player, "Opt", Zone::Hand);
        cast(&mut game, first, None);
        assert!(game.state.stack[0].cast.unwrap().first_spell_cast_this_game);
        cast(&mut game, later, None);
        assert!(!game.state.stack[1].cast.unwrap().first_spell_cast_this_game);
        settle(&mut game);
        assert_eq!(game.state.players[player.index()].life, 22);

        let original = put(&mut game, player, "Opt", Zone::Hand);
        let copier = put(&mut game, player, "Unsummon", Zone::Hand);
        cast(&mut game, original, None);
        cast(&mut game, copier, Some(Target::Spell(original)));
        settle(&mut game);
        assert_eq!(game.state.players[player.index()].life, 22);
        assert_eq!(game.state.players[player.index()].spells_cast_this_turn, 4);
        assert_eq!(game.state.players[player.other().index()].life, 20);

        // A turn-boundary reset must not make the next spell first again.
        for _ in 0..2 {
            game.state.step = Step::End;
            game.give_priority();
            act(&mut game, Action::Pass);
            act(&mut game, Action::Pass);
        }
        assert_eq!(game.state.active, player);
        assert_eq!(game.state.players[player.index()].spells_cast_this_turn, 0);
        for seat in &mut game.state.players {
            seat.mana = [8; 6];
        }
        let next_turn = put(&mut game, player, "Opt", Zone::Hand);
        cast(&mut game, next_turn, None);
        settle(&mut game);
        assert_eq!(game.state.players[player.index()].life, 22);
    }
}

#[test]
fn changing_control_of_the_spell_does_not_make_it_the_new_controllers_first_cast() {
    let _rules = rules();
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let first = put(&mut game, player, "Opt", Zone::Hand);
        cast(&mut game, first, None);
        game.state.stack[0].controller = player.other();
        settle(&mut game);
        assert_eq!(game.state.players[player.index()].life, 20);
        assert_eq!(game.state.players[player.other().index()].life, 20);
    }
}

#[test]
fn countered_first_spell_still_consumes_first_cast_and_recast_has_no_bonus() {
    let _rules = rules();
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let first = put(&mut game, player, "Opt", Zone::Hand);
        cast(&mut game, first, None);
        act(&mut game, Action::Pass);
        let counter = put(&mut game, player.other(), "Negate", Zone::Hand);
        cast(&mut game, counter, Some(Target::Spell(first)));
        settle(&mut game);
        assert_eq!(game.state.players[player.index()].life, 20);
        assert_eq!(game.object_view(first).zone, Zone::Graveyard);

        game.move_object(first, Zone::Hand);
        cast(&mut game, first, None);
        assert!(
            !game
                .state
                .stack
                .last()
                .unwrap()
                .cast
                .unwrap()
                .first_spell_cast_this_game
        );
        settle(&mut game);
        assert_eq!(game.state.players[player.index()].life, 20);
    }
}

#[test]
fn rejected_cast_and_raw_index_restore_preserve_game_long_first_cast_history() {
    let _rules = rules();
    for player in [Player::P0, Player::P1] {
        let mut parent = scenario(player);
        let first = put(&mut parent, player, "Opt", Zone::Hand);
        parent.state.players[player.index()].mana = [0; 6];
        admit(&parent, Action::AnnounceCast { card: first });
        parent.state.players[player.index()].mana = [8; 6];

        let action = parent
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::Cast { card, target: None, .. } if *card == first))
            .unwrap();
        let checkpoint = parent.snapshot();
        let index = parent
            .legal_actions()
            .iter()
            .position(|a| a == &action)
            .unwrap();
        let mut raw = parent.clone();
        let mut indexed = parent.clone();
        let revision = indexed.revision();
        act(&mut raw, action);
        indexed.apply_index(player, revision, index).unwrap();
        assert_eq!(raw.snapshot(), indexed.snapshot());
        drop(parent);

        for branch in [&mut raw, &mut indexed] {
            assert!(
                branch.state.stack[0]
                    .cast
                    .unwrap()
                    .first_spell_cast_this_game
            );
            let later = put(branch, player, "Opt", Zone::Hand);
            cast(branch, later, None);
            settle(branch);
            assert_eq!(branch.state.players[player.index()].life, 22);
        }
        assert_eq!(raw.snapshot(), indexed.snapshot());
        indexed.restore(&checkpoint).unwrap();
        assert_eq!(indexed.snapshot(), checkpoint);
        cast(&mut indexed, first, None);
        settle(&mut indexed);
        assert_eq!(indexed.state.players[player.index()].life, 22);
    }
}
