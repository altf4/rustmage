//! Combat receipts from exact Crew costs, declarations and defending players.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::engine::characteristic_layers::AnimationTypeMode;
use rustmage_core::{cards::*, engine::*};
use support::*;

const PILOT: CardRules = CardRules {
    effect: Effect::Creature(4, 3),
    keywords: Keywords::REACH,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[
            StaticAbilityFragment::ControlledSubtypeKeywords {
                subtype: "Vehicle",
                keywords: Keywords::VIGILANCE.union(Keywords::REACH),
            },
            StaticAbilityFragment::ControlledCreatureScaling {
                subtype: Some("Vehicle"),
                value: StatValue::Fixed(1),
                stats: Stats {
                    power: 1,
                    toughness: 1,
                },
            },
        ],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::InterveningIf {
            guard: StepGuard::FirstCombatPhase,
            event: &TriggerCondition::CrewedVehicleAttacks,
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::MayPayMana {
                cost: ManaCost {
                    colored: [0, 0, 0, 1, 1, 0],
                    generic: 1,
                },
                skip: 1,
            },
            Instruction::AdditionalCombat,
        ],
    }],
    ..CardRules::EMPTY
};

const COUNTER_ABILITY: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Ability)),
        &[Instruction::Counter],
    ),
    ..CardRules::EMPTY
};

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn crew(game: &mut Game, vehicle: CardId, crewer: CardId) {
    act(
        game,
        Action::AnnounceActivate {
            card: vehicle,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(game);
    }
    let object = game.object_ref(crewer);
    act(game, Action::ChooseCrewer { object });
    act(game, Action::FinishCrew);
}

fn declare(game: &mut Game, attackers: &[CardId]) {
    assert_eq!(game.state.step, Step::BeginCombat);
    pass_pair(game);
    for &card in attackers {
        act(game, Action::Attack { card });
    }
    act(game, Action::FinishAttackers);
}

fn animate_vehicle(game: &mut Game, vehicle: CardId) {
    let object = game.object_ref(vehicle);
    game.change_object_form(
        object,
        AnimationSpec {
            artifact: true,
            creature_types: AnimationCreatureTypes::Unchanged,
            base: Some((5, 4)),
            keywords: Keywords::NONE,
            duration: AnimationDuration::EndOfTurn,
        },
        AnimationTypeMode::AddCreature,
    );
}

#[test]
fn each_qualified_vehicle_offers_its_own_first_combat_payment() {
    let _rules = fixture_rules::activate(&[(Card::HillGiant, &PILOT)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let pilot = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let other = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let vehicles = [pilot, pilot, other].map(|crewer| {
        let vehicle = put(&mut game, player, "Broadcast Rambler", Zone::Battlefield);
        assert!(!game.object_view(vehicle).is_creature());
        assert!(
            game.object_view(vehicle)
                .keywords
                .contains(Keywords::VIGILANCE)
        );
        crew(&mut game, vehicle, crewer);
        settle(&mut game);
        game.state.objects[crewer as usize].tapped = false;
        assert_eq!(stats(&game, vehicle), (6, 5));
        vehicle
    });
    game.begin_step(Step::BeginCombat);
    declare(&mut game, &vehicles);
    assert_eq!(game.state.pending, Pending::OrderTriggers { player });
    let finish = game
        .legal_actions()
        .into_iter()
        .find(|action| matches!(action, Action::PutTrigger { .. }))
        .unwrap();
    let revision = game.revision();
    let index = game
        .legal_actions()
        .iter()
        .position(|action| *action == finish)
        .unwrap();
    let mut indexed = game.clone();
    indexed.apply_index(player, revision, index).unwrap();
    act(&mut game, finish);
    assert_eq!(game.snapshot(), indexed.snapshot());

    let mut captured: Vec<_> = game
        .state
        .stack
        .iter()
        .map(|item| item.event_object.as_ref().unwrap().object.card)
        .collect();
    captured.sort_unstable();
    assert_eq!(captured, vehicles[..2]);
    assert!(vehicles.iter().all(|&id| !game.object_view(id).tapped));

    pass_pair(&mut game);
    choose(&mut game, |action| {
        matches!(action, Action::PayResolutionMana { .. })
    });
    pass_pair(&mut game);
    act(&mut game, Action::DeclineResolutionMana);
    assert_eq!(game.observe(player).additional_combats, 1);
    for _ in 0..16 {
        if game.state.step == Step::BeginCombat {
            break;
        }
        assert_ne!(game.state.step, Step::Main2);
        choose(&mut game, |action| {
            matches!(action, Action::Pass | Action::FinishBlockers)
        });
    }
    assert_eq!(game.state.step, Step::BeginCombat);
    assert!(!game.first_combat_phase());
    declare(&mut game, &vehicles[..1]);
    assert!(game.state.stack.is_empty());
}

#[test]
fn countering_crew_preserves_its_receipt_but_blinking_either_incarnation_does_not() {
    let _rules = fixture_rules::activate(&[
        (Card::HillGiant, &PILOT),
        (Card::Counterspell, &COUNTER_ABILITY),
    ]);
    let player = Player::P1;
    let mut game = scenario(player);
    let pilot = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let vehicle = put(&mut game, player, "Broadcast Rambler", Zone::Battlefield);
    crew(&mut game, vehicle, pilot);
    let activation = game.state.stack.last().unwrap().id;
    act(&mut game, Action::Pass);
    let counter = put(&mut game, player.other(), "Counterspell", Zone::Hand);
    cast(&mut game, counter, Some(Target::Stack(activation)));
    settle(&mut game);
    assert!(!game.object_view(vehicle).is_creature());
    let paid = game.snapshot();
    for change in [None, Some(pilot), Some(vehicle)] {
        let mut branch = game.clone();
        if let Some(object) = change {
            branch.move_object(object, Zone::Hand);
            branch.move_object(object, Zone::Battlefield);
            branch.state.objects[object as usize].summoning_sick = false;
            // This raw return is only testing the changed incarnation.
            branch.state.pending_triggers.clear();
        } else {
            // A crewer can still observe the old paid cost after control changes.
            branch.record_control(branch.object_ref(pilot), player.other(), None);
        }
        animate_vehicle(&mut branch, vehicle);
        branch.begin_step(Step::BeginCombat);
        declare(&mut branch, &[vehicle]);
        assert_eq!(branch.state.stack.len(), usize::from(change.is_none()));
        if change.is_none() {
            let trigger = branch.state.stack.last().unwrap();
            assert_eq!(trigger.controller, player.other());
            assert_eq!(
                trigger.event_object.as_ref().unwrap().object,
                branch.object_ref(vehicle)
            );
        }
    }
    assert_eq!(game.snapshot(), paid);
}
