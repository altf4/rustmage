//! The entry controller is historical even when the exact object acquires a new form.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::engine::funding::{FundingGoal, FundingState};
use rustmage_core::{cards::*, engine::*};
use support::*;

const EXACT: StatValue = StatValue::SourceEnteredUnderYourControlThisTurn;
const SOURCE_OR_ARTIFACT: StatValue = StatValue::Sum(&[EXACT, StatValue::ArtifactEnteredThisTurn]);
const TEST_ABILITY: ActivatedAbility = ActivatedAbility {
    exhaust: false,
    guard: ActivationGuard::StatAtLeast {
        value: EXACT,
        minimum: 1,
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
};
const COPY_TARGET: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[
        TEST_ABILITY,
        ActivatedAbility {
            guard: ActivationGuard::StatAtLeast {
                value: SOURCE_OR_ARTIFACT,
                minimum: 1,
            },
            ..TEST_ABILITY
        },
    ],
    ..CardRules::EMPTY
};

fn definition() -> Card {
    Card::from_name("Short Bow").unwrap()
}

fn copy(game: &mut Game, object: ObjectRef) {
    game.apply_temporary_copy(
        object,
        CopyRecipe::new(definition()),
        CopyDeadline::EndOfTurn,
    );
    assert_eq!(game.object_view(object.card).card, definition());
    assert_eq!(game.object_ref(object.card), object);
}

fn announce(card: CardId, ability: u16) -> Action {
    Action::AnnounceActivate {
        card,
        ability,
        counter_payment: Counters::default(),
    }
}

#[test]
fn projected_entry_keeps_the_same_controller_and_exact_guard_as_committed_entry() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let source = put(&mut game, player.other(), "Bear Cub", Zone::Exile);
        let mut movement = game.entry_move(source, Zone::Battlefield, false);
        movement.controller = player;
        let goal = FundingGoal {
            actor: player,
            required: [0; 6],
            generic: 0,
            reserve: None,
            source: None,
        };
        let world = FundingState::from_game(&game, &goal);
        let mut projected = None;
        world.visit_entry_moves(vec![movement.clone()], true, None, &mut |next| {
            let query = characteristic_query::CharacteristicQuery::in_world(next);
            let view = query.object_view(source);
            assert!(view.entered_this_turn);
            assert_eq!(view.entry_controller_this_turn, Some(player));
            assert_eq!(
                game.stat_value_in_world(next, EXACT, view.reference, player),
                1
            );
            assert_eq!(
                game.stat_value_in_world(next, EXACT, view.reference, player.other()),
                0,
            );
            projected = Some(view);
            false
        });
        game.stage_entries(vec![movement], None);
        game.advance_entries();
        assert_eq!(game.object_view(source), projected.unwrap());
    }
}

fn new_turn(game: &mut Game) {
    let previous = game.state.active;
    game.state.step = Step::End;
    game.state.priority = previous;
    act(game, Action::Pass);
    act(game, Action::Pass);
    assert_eq!(game.state.active, previous.other());
}

#[test]
fn a_nonartifact_entry_can_acquire_and_use_the_exact_source_guard() {
    let _rules = fixture_rules::activate(&[(definition(), &COPY_TARGET)]);
    for player in [Player::P0, Player::P1] {
        let mut root = scenario(player);
        let entrant = put(&mut root, player.other(), "Bear Cub", Zone::Hand);
        let mut movement = root.entry_move(entrant, Zone::Battlefield, false);
        movement.controller = player;
        root.stage_entries(vec![movement], None);
        root.advance_entries();
        let reference = root.object_ref(entrant);
        let entry = root.object_view(entrant);
        assert_eq!(entry.owner, player.other());
        assert_eq!(entry.controller, player);
        assert_eq!(entry.kinds, 1);
        assert!(entry.entered_this_turn);
        assert_eq!(entry.entry_controller_this_turn, Some(player));
        assert_eq!(
            root.stat_value(StatValue::ArtifactEnteredThisTurn, reference, player),
            0
        );
        copy(&mut root, reference);
        assert_eq!(
            root.object_view(entrant).entry_controller_this_turn,
            Some(player)
        );

        let action = announce(entrant, 0);
        let revision = root.revision();
        let index = root
            .legal_actions()
            .iter()
            .position(|a| *a == action)
            .unwrap();
        let checkpoint = root.snapshot();
        let mut raw = root.clone();
        let mut indexed = root.clone();
        act(&mut raw, action);
        indexed.apply_index(player, revision, index).unwrap();
        for branch in [&mut raw, &mut indexed] {
            pay(branch);
            settle(branch);
            assert_eq!(branch.state.players[player.index()].life, STARTING_LIFE + 1);
        }
        assert_eq!(raw.snapshot(), indexed.snapshot());
        indexed.restore(&checkpoint).unwrap();
        assert_eq!(
            indexed.apply_index(player, revision, index),
            Err(Error::StaleRevision)
        );
        assert_eq!(
            indexed.object_view(entrant).entry_controller_this_turn,
            Some(player)
        );

        new_turn(&mut root);
        assert!(!root.object_view(entrant).entered_this_turn);
        assert_eq!(root.object_view(entrant).entry_controller_this_turn, None);
        copy(&mut root, reference);
        root.state.priority = player;
        reject(&mut root, announce(entrant, 0));
        reject(&mut root, announce(entrant, 1));
    }
}

#[test]
fn stolen_entries_keep_their_original_controller_and_artifact_history_persists() {
    let _rules = fixture_rules::activate(&[(definition(), &COPY_TARGET)]);
    for player in [Player::P0, Player::P1] {
        let other = player.other();
        let mut game = scenario(player);
        let entrant = put(&mut game, other, "Bear Cub", Zone::Hand);
        game.move_object(entrant, Zone::Battlefield);
        let reference = game.object_ref(entrant);
        game.record_control(reference, player, None);
        copy(&mut game, reference);
        assert_eq!(game.object_view(entrant).controller, player);
        assert!(game.object_view(entrant).entered_this_turn);
        assert_eq!(
            game.object_view(entrant).entry_controller_this_turn,
            Some(other)
        );
        reject(&mut game, announce(entrant, 0));
        reject(&mut game, announce(entrant, 1));

        let artifact = put(&mut game, player, "Swiftfoot Boots", Zone::Hand);
        game.move_object(artifact, Zone::Battlefield);
        game.move_object(artifact, Zone::Exile);
        reject(&mut game, announce(entrant, 0));
        act(&mut game, announce(entrant, 1));
        pay(&mut game);
        settle(&mut game);
        assert_eq!(game.state.players[player.index()].life, STARTING_LIFE + 1);
    }
}
