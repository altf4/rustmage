//! A watched exact creature death creates an owner-controlled next-end-step return.
use crate::support::*;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};

const WATCH: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost::ZERO,
        targets: TargetPlan::Single(TargetSpec {
            rule: TargetRule::Creature,
            min: 0,
            max: 1,
        }),
        program: &[Instruction::DelayTargetEvent {
            event: DelayedTargetEvent::DiesOnce,
            ability: 0,
        }],
        sorcery: false,
        once_per_turn: false,
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::DelayedTargetEvent,
        targets: TargetPlan::NONE,
        program: &[Instruction::ScheduleDelayedDeathReturnWithCounter(
            CounterKind::PlusOne,
        )],
    }],
    ..CardRules::EMPTY
};

fn setup() -> (fixture_rules::Guard, Game, CardId) {
    let rules = fixture_rules::activate(&[(Card::HillGiant, &WATCH)]);
    let mut game = scenario(Player::P0);
    let source = put(&mut game, Player::P0, "Hill Giant", Zone::Battlefield);
    (rules, game, source)
}

fn drive(game: &mut Game) {
    for _ in 0..64 {
        if game.state.pending == Pending::Priority && !game.state.pending_triggers.is_empty() {
            game.prepare_triggers();
        }
        if game.state.pending == Pending::Priority
            && game.state.stack.is_empty()
            && game.state.pending_triggers.is_empty()
            && game.state.resolving.is_none()
        {
            return;
        }
        choose(game, |action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. })
        });
    }
    panic!("death return did not settle: {:?}", game.state.pending);
}

fn watch(game: &mut Game, source: CardId, target: Option<CardId>) {
    choose(game, |action| {
        matches!(action, Action::Activate { card, ability: 0, target: selected, .. }
            if *card == source && *selected == target.map(Target::Card))
    });
    drive(game);
}

fn die(game: &mut Game, target: CardId) {
    game.move_object(target, Zone::Graveyard);
    drive(game);
}

fn end(game: &mut Game) {
    game.begin_step(Step::End);
    game.prepare_triggers();
    drive(game);
}

#[test]
fn source_gone_opponent_owned_creature_returns_under_owner_with_entry_counter() {
    let (_rules, mut game, source) = setup();
    let target = put(&mut game, Player::P1, "Grizzly Bears", Zone::Battlefield);
    watch(&mut game, source, Some(target));
    game.move_object(source, Zone::Graveyard);
    die(&mut game, target);
    let grave = game.object_ref(target);
    assert_eq!(game.object_view(target).zone, Zone::Graveyard);
    assert_eq!(game.state.delayed_returns.as_ref().unwrap().len(), 1);
    assert_eq!(
        game.state.delayed_returns.as_ref().unwrap()[0]
            .view
            .returning
            .object,
        grave
    );
    end(&mut game);
    assert_eq!(game.object_view(target).zone, Zone::Battlefield);
    assert_eq!(game.object_view(target).controller, Player::P1);
    assert_eq!(game.counter_count(target, CounterKind::PlusOne), 1);
}

#[test]
fn zero_target_token_and_new_graveyard_incarnation_do_not_return() {
    let (_rules, mut game, source) = setup();
    watch(&mut game, source, None);
    assert!(game.state.delayed_target_events.is_none());

    game.create_tokens(Player::P1, Card::DrakeToken, 1, false);
    let token = game
        .state
        .objects
        .iter()
        .enumerate()
        .find(|(_, object)| object.zone == Zone::Battlefield && object.is_token())
        .unwrap()
        .0 as CardId;
    watch(&mut game, source, Some(token));
    die(&mut game, token);
    end(&mut game);
    assert_ne!(game.object_view(token).zone, Zone::Battlefield);

    let target = put(&mut game, Player::P1, "Grizzly Bears", Zone::Battlefield);
    watch(&mut game, source, Some(target));
    die(&mut game, target);
    game.move_object(target, Zone::Exile);
    game.move_object(target, Zone::Graveyard);
    end(&mut game);
    assert_eq!(game.object_view(target).zone, Zone::Graveyard);
}

#[test]
fn a_death_after_end_step_begins_waits_for_the_following_end_step() {
    let (_rules, mut game, source) = setup();
    let target = put(&mut game, Player::P1, "Grizzly Bears", Zone::Battlefield);
    watch(&mut game, source, Some(target));
    end(&mut game);
    die(&mut game, target);
    assert_eq!(game.object_view(target).zone, Zone::Graveyard);
    assert_eq!(game.state.delayed_returns.as_ref().unwrap().len(), 1);
    game.state.turn += 1;
    game.state.active = Player::P1;
    end(&mut game);
    assert_eq!(game.object_view(target).zone, Zone::Battlefield);
    assert_eq!(game.counter_count(target, CounterKind::PlusOne), 1);
}

#[test]
fn watch_registration_and_death_return_use_owned_raw_index_restore() {
    let (_rules, mut game, source) = setup();
    let target = put(&mut game, Player::P1, "Grizzly Bears", Zone::Battlefield);
    let action = game.legal_actions().into_iter().find(|action| {
        matches!(action, Action::Activate { card, ability: 0, target: Some(Target::Card(id)), .. }
            if *card == source && *id == target)
    }).unwrap();
    let index = game
        .legal_actions()
        .iter()
        .position(|candidate| *candidate == action)
        .unwrap();
    let revision = game.revision();
    let before = game.snapshot();
    let mut indexed = game.clone();
    act(&mut game, action.clone());
    indexed.apply_index(Player::P0, revision, index).unwrap();
    assert_eq!(game.snapshot(), indexed.snapshot());
    drive(&mut game);
    drive(&mut indexed);
    let watched = game.snapshot();
    die(&mut game, target);
    die(&mut indexed, target);
    assert_eq!(game.snapshot(), indexed.snapshot());
    let mut restored = indexed.clone();
    restored.restore(&watched).unwrap();
    die(&mut restored, target);
    assert_eq!(restored.snapshot(), game.snapshot());
    restored.restore(&before).unwrap();
    assert_eq!(
        restored.apply_index(Player::P0, revision, index),
        Err(Error::StaleRevision)
    );
}
