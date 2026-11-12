//! Face-down combat cohorts and exact face-down death returns.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const FACE_DOWN_OBSERVER: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::PlayerDamaged {
                subject: DamageSubject::ControlledFaceDownCreatures,
                combat_only: true,
                opponent_only: false,
                one_or_more: true,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::Draw(PlayerSet::Controller, 1)],
        },
        TriggeredAbility {
            condition: TriggerCondition::BattlefieldDeparture(DepartureRule {
                kinds: 1,
                relation: CastRelation::You,
                destinations: &[Zone::Graveyard],
                face_down: true,
                ..DepartureRule::ANY
            }),
            targets: TargetPlan::NONE,
            program: &[
                Instruction::CaptureEntryResult,
                Instruction::ReturnDepartedPermanentFaceDown,
                Instruction::TurnEntryResultFaceUp,
            ],
        },
    ],
    ..CardRules::EMPTY
};

fn order(game: &mut Game) {
    while matches!(game.state.pending, Pending::OrderTriggers { .. }) {
        choose(game, |action| matches!(action, Action::PutTrigger { .. }));
    }
}

#[test]
fn simultaneous_deaths_retain_the_observer_and_each_exact_graveyard_arrival() {
    let player = Player::P0;
    let owner = player.other();
    let watcher = Card::from_name("Grizzly Bears").unwrap();
    let _rules = fixture_rules::activate(&[(watcher, &FACE_DOWN_OBSERVER)]);
    let mut game = scenario(player);
    let source = spawn(&mut game, player, watcher, Zone::Battlefield);
    let land = put(&mut game, owner, "Forest", Zone::Library);
    let instant = put(&mut game, player, "Shock", Zone::Library);
    let face_up = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    game.cloak_library_top(player, owner);
    assert!(!game.advance_entries());
    game.cloak_library_top(player, player);
    assert!(!game.advance_entries());
    let prior_land = game.object_ref(land);
    let prior_spell = game.object_ref(instant);
    let sweeper = put(&mut game, player, "Day of Judgment", Zone::Hand);
    cast(&mut game, sweeper, None);
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    order(&mut game);
    assert_eq!(game.state.stack.len(), 2);
    let land_death = game.object_ref(land);
    assert_ne!(land_death, prior_land);
    assert_ne!(game.object_ref(instant), prior_spell);
    let saved = game.snapshot();

    let mut stale = game.clone();
    stale.move_object(land, Zone::Exile);
    stale.move_object(land, Zone::Graveyard);
    settle(&mut stale);
    assert_eq!(stale.object_view(land).zone, Zone::Graveyard);
    assert_ne!(stale.object_ref(land), land_death);

    settle(&mut game);
    assert_eq!(game.object_view(source).zone, Zone::Graveyard);
    assert_eq!(game.object_view(face_up).zone, Zone::Graveyard);
    assert_eq!(game.object_view(instant).zone, Zone::Graveyard);
    assert_eq!(game.object_view(land).zone, Zone::Battlefield);
    assert_eq!(game.object_view(land).controller, owner);
    assert_eq!(game.object_view(land).kinds & 2, 2);
    assert!(!game.is_face_down(game.object_ref(land)));

    stale.restore(&saved).unwrap();
    settle(&mut stale);
    assert_eq!(stale.snapshot(), game.snapshot());
}

const LOCK: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Rule,
    fragments: &[StaticAbilityFragment::OpponentFaceUpProhibitedOnOwnTurn],
}];
const TURN_UP_LOCK: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: LOCK,
    extras: &CardExtras {
        static_ability_groups: LOCK,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

#[test]
fn prohibited_turn_up_still_returns_a_known_unwarded_creature_face_down() {
    let owner = Player::P0;
    let player = owner.other();
    let watcher = Card::from_name("Hill Giant").unwrap();
    let lock = Card::from_name("Doubling Season").unwrap();
    let _rules = fixture_rules::activate(&[(watcher, &FACE_DOWN_OBSERVER), (lock, &TURN_UP_LOCK)]);
    let mut game = scenario(owner);
    spawn(&mut game, player, watcher, Zone::Battlefield);
    spawn(&mut game, owner, lock, Zone::Battlefield);
    let bear = put(&mut game, owner, "Grizzly Bears", Zone::Library);
    game.cloak_library_top(player, owner);
    assert!(!game.advance_entries());

    let sweeper = put(&mut game, owner, "Day of Judgment", Zone::Hand);
    cast(&mut game, sweeper, None);
    settle(&mut game);
    let object = game.object_ref(bear);
    assert_eq!(game.object_view(bear).controller, owner);
    assert_eq!(stats(&game, bear), (2, 2));
    let public = &game.observe(player).face_down_permanents[0];
    assert_eq!(public.object, object);
    assert_eq!(public.cause, FaceDownCause::OtherEffect);
    assert_eq!(public.underlying, Card::from_name("Grizzly Bears"));
    reject(
        &mut game,
        Action::BeginFaceUp {
            object,
            cost: ManaCost {
                colored: [0, 0, 0, 0, 1, 0],
                generic: 1,
            },
        },
    );
    game.state.priority = player;
    let bolt = put(&mut game, player, "Shock", Zone::Hand);
    cast(&mut game, bolt, Some(Target::Card(bear)));
    assert_eq!(game.state.stack.len(), 1);
    settle(&mut game);
    assert_eq!(game.object_view(bear).zone, Zone::Graveyard);
}

#[test]
fn combat_damage_counts_only_controlled_face_down_creatures_and_groups_by_player() {
    let player = Player::P1;
    let opponent = player.other();
    let watcher = Card::from_name("Hill Giant").unwrap();
    let _rules = fixture_rules::activate(&[(watcher, &FACE_DOWN_OBSERVER)]);
    let mut game = scenario(player);
    let source = spawn(&mut game, player, watcher, Zone::Battlefield);
    let first = put(&mut game, player, "Forest", Zone::Library);
    game.cloak_library_top(player, player);
    assert!(!game.advance_entries());
    let second = put(&mut game, opponent, "Forest", Zone::Library);
    game.cloak_library_top(player, opponent);
    assert!(!game.advance_entries());
    let hostile = put(&mut game, opponent, "Forest", Zone::Library);
    game.cloak_library_top(opponent, opponent);
    assert!(!game.advance_entries());
    let hit = |id, target, amount, combat| DamageEvent {
        source: game.damage_source(id),
        target: Target::Player(target),
        amount,
        combat,
        preventable: true,
    };
    let events = [
        hit(first, opponent, 2, true),
        hit(second, opponent, 2, true),
        hit(source, opponent, 4, true),
        hit(hostile, player, 2, true),
        hit(first, player, 1, false),
    ];
    game.apply_damage_batch(&events);
    assert_eq!(game.state.pending_triggers.len(), 1);
    let trigger = game.state.pending_triggers.last().unwrap();
    let event = trigger.damage_event.as_ref().unwrap();
    assert_eq!(event.amount, 4);
    assert_eq!(event.player, opponent);
}
