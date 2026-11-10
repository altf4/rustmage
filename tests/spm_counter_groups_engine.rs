//! Live conjunctions and overlapping groups in simultaneous counter effects.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const ARTIFACT_CREATURE_OR_VEHICLE: &[ControlledPermanentFilter] = &[
    ControlledPermanentFilter {
        all_kinds: 1 | 16,
        ..ControlledPermanentFilter::ANY
    },
    ControlledPermanentFilter {
        subtype: Some("Vehicle"),
        ..ControlledPermanentFilter::ANY
    },
];
const SPIDER_OR_LEGENDARY_CREATURE: &[ControlledPermanentFilter] = &[
    ControlledPermanentFilter {
        subtype: Some("Spider"),
        ..ControlledPermanentFilter::ANY
    },
    ControlledPermanentFilter {
        all_kinds: 1,
        legendary: true,
        ..ControlledPermanentFilter::ANY
    },
];
const ADD: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    activated: &[ActivatedAbility {
        origin: ActivationOrigin::Battlefield,
        guard: ActivationGuard::Always,
        cycling: false,
        exhaust: false,
        once_per_turn: false,
        sorcery: false,
        cost: ActivationCost {
            tap: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::ChangeControlledPermanentGroupCounters {
            filters: ARTIFACT_CREATURE_OR_VEHICLE,
            effect: PermanentGroupCounters::Add {
                kind: CounterKind::PlusOne,
                amount: 1,
            },
        }],
    }],
    ..CardRules::EMPTY
};
const DOUBLE: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AttackDeclared {
            minimum: 1,
            power_min: None,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::ChangeControlledPermanentGroupCounters {
            filters: SPIDER_OR_LEGENDARY_CREATURE,
            effect: PermanentGroupCounters::DoubleEachKind,
        }],
    }],
    ..CardRules::EMPTY
};

fn activate(game: &mut Game, card: CardId, ability: u16) {
    act(
        game,
        Action::AnnounceActivate {
            card,
            ability,
            counter_payment: Counters::default(),
        },
    );
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(game);
    }
}

#[test]
fn counters_go_to_both_types_of_vehicle_and_artifact_creatures_once() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Solemn Simulacrum").unwrap(), &ADD)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Solemn Simulacrum", Zone::Battlefield);
    let parked = put(&mut game, player, "Voyager Glidecar", Zone::Battlefield);
    let vehicle = put(&mut game, player, "Voyager Glidecar", Zone::Battlefield);
    let pilot = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let equipment = put(&mut game, player, "Swiftfoot Boots", Zone::Battlefield);
    let opponent_artifact = put(
        &mut game,
        player.other(),
        "Solemn Simulacrum",
        Zone::Battlefield,
    );
    activate(&mut game, vehicle, 1);
    let object = game.object_ref(pilot);
    act(&mut game, Action::ChooseCrewer { object });
    act(&mut game, Action::FinishCrew);
    settle(&mut game);
    assert!(game.is_creature(vehicle));
    assert!(!game.is_creature(parked));

    activate(&mut game, source, 0);
    assert!(game.object_view(source).tapped);
    assert!(game.state.stack[0].targets.is_empty());
    settle(&mut game);
    for id in [source, parked, vehicle] {
        assert_eq!(game.counter_count(id, CounterKind::PlusOne), 1);
    }
    for id in [pilot, equipment, opponent_artifact] {
        assert_eq!(game.counter_count(id, CounterKind::PlusOne), 0);
    }
}

#[test]
fn attack_doubles_all_kinds_once_and_applies_counter_replacements_to_new_counters() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Grizzly Bears").unwrap(), &DOUBLE)]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let source = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let both = put(
        &mut game,
        player,
        "Spider-Byte, Web Warden",
        Zone::Battlefield,
    );
    let spider = put(&mut game, player, "Treetop Snarespinner", Zone::Battlefield);
    let live_spider = put(&mut game, player, "Treetop Snarespinner", Zone::Battlefield);
    let legend = put(&mut game, player, "Agent Venom", Zone::Battlefield);
    let noncreature = put(&mut game, player, "The Wind Crystal", Zone::Battlefield);
    let opposing_spider = put(
        &mut game,
        opponent,
        "Treetop Snarespinner",
        Zone::Battlefield,
    );
    put(&mut game, player, "Doubling Season", Zone::Battlefield);
    for id in [
        source,
        both,
        spider,
        live_spider,
        legend,
        noncreature,
        opposing_spider,
    ] {
        game.add_counter(id, CounterKind::PlusOne, 2);
        game.add_counter(id, CounterKind::Shield, 1);
    }
    let blank = put(&mut game, player, "Witness Protection", Zone::Hand);
    cast(&mut game, blank, Some(Target::Card(spider)));
    settle(&mut game);
    assert!(!game.has_subtype(spider, "Spider"));

    for _ in 0..24 {
        if game.state.pending == Pending::Attackers {
            break;
        }
        act(&mut game, Action::Pass);
    }
    act(&mut game, Action::Attack { card: source });
    act(&mut game, Action::FinishAttackers);
    settle(&mut game);
    for id in [both, live_spider, legend] {
        assert_eq!(game.counter_count(id, CounterKind::PlusOne), 6);
        assert_eq!(game.counter_count(id, CounterKind::Shield), 3);
    }
    for id in [source, spider, noncreature, opposing_spider] {
        assert_eq!(game.counter_count(id, CounterKind::PlusOne), 2);
        assert_eq!(game.counter_count(id, CounterKind::Shield), 1);
    }
}
