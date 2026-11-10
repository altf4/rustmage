//! The activation's untargeted counter payment uses its final live artifacts.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use std::num::NonZeroU16;
use support::*;

const BASE: ActivatedAbility = ActivatedAbility {
    guard: ActivationGuard::Always,
    origin: ActivationOrigin::Battlefield,
    cycling: false,
    exhaust: false,
    once_per_turn: false,
    sorcery: false,
    cost: ActivationCost::ZERO,
    targets: TargetPlan::NONE,
    program: &[],
};
const DRAW: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            generic: 2,
            mana_component: true,
            additional: Some(AdditionalCost {
                kind: EffectChoiceKind::RemovePermanentCounters {
                    amount: NonZeroU16::new(2).unwrap(),
                    kind: CounterKind::PlusOne,
                    bearer_kinds: 16,
                },
                filter: SacrificeFilter {
                    kinds: 0,
                    source_only: false,
                    other: false,
                    nonland: false,
                    tokens: None,
                    subtype: None,
                    keywords: Keywords::NONE,
                    artifact_enchantment_or_token: false,
                },
            }),
            ..ActivationCost::ZERO
        },
        program: &[Instruction::Draw(PlayerSet::Controller, 1)],
        ..BASE
    }],
    ..CardRules::EMPTY
};
const COLORLESS_TWO: Mana = [0, 0, 0, 0, 0, 2];
const PRODUCE: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            tap: true,
            ..ActivationCost::ZERO
        },
        program: &[Instruction::AddMana(ManaProduction::Fixed(COLORLESS_TWO))],
        ..BASE
    }],
    extras: &CardExtras {
        activation_put_counters: &[(0, CounterKind::PlusOne, 1)],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

fn draw_activation(card: CardId) -> Action {
    Action::AnnounceActivate {
        card,
        ability: 0,
        counter_payment: Counters::default(),
    }
}

fn remove(object: ObjectRef) -> Action {
    Action::ChooseCreatureCounterCost {
        object,
        counter: CounterKind::PlusOne,
    }
}

#[test]
fn repeated_and_split_selections_pay_exact_artifacts_without_targets() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Solemn Simulacrum").unwrap(), &DRAW)]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let source = put(&mut game, player, "Solemn Simulacrum", Zone::Battlefield);
    let equipment = put(&mut game, player, "Swiftfoot Boots", Zone::Battlefield);
    let vehicle = put(&mut game, player, "Voyager Glidecar", Zone::Battlefield);
    let creature = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let enemy = put(&mut game, opponent, "Swiftfoot Boots", Zone::Battlefield);
    for id in [creature, enemy] {
        game.add_counter(id, CounterKind::PlusOne, 3);
    }
    game.add_counter(equipment, CounterKind::Shield, 1);
    game.add_counter(equipment, CounterKind::PlusOne, 1);
    reject(&mut game, draw_activation(source));
    game.add_counter(equipment, CounterKind::PlusOne, 1);
    game.add_counter(vehicle, CounterKind::PlusOne, 1);
    assert!(!game.is_creature(vehicle));
    act(&mut game, draw_activation(source));
    let equipment_ref = game.object_ref(equipment);
    let vehicle_ref = game.object_ref(vehicle);
    for id in [creature, enemy] {
        let object = game.object_ref(id);
        reject(&mut game, remove(object));
    }
    reject(
        &mut game,
        Action::ChooseCreatureCounterCost {
            object: equipment_ref,
            counter: CounterKind::Shield,
        },
    );
    reject(
        &mut game,
        remove(ObjectRef {
            card: vehicle,
            generation: vehicle_ref.generation + 1,
        }),
    );
    let mut single_counter_first = game.clone();
    act(&mut game, remove(equipment_ref));
    assert_eq!(game.counter_count(equipment, CounterKind::PlusOne), 2);

    let saved = game.snapshot();
    let revision = game.revision();
    let second = remove(equipment_ref);
    let index = game
        .legal_actions()
        .iter()
        .position(|action| *action == second)
        .unwrap();
    assert_eq!(game.apply(opponent, second.clone()), Err(Error::WrongActor));
    assert_eq!(game.snapshot(), saved);
    let mut raw = game.clone();
    act(&mut raw, second);
    game.apply_index(player, revision, index).unwrap();
    assert_eq!(raw.snapshot(), game.snapshot());
    pay(&mut raw);
    assert!(raw.state.stack[0].targets.is_empty());
    assert_eq!(raw.counter_count(equipment, CounterKind::PlusOne), 0);
    assert_eq!(raw.counter_count(equipment, CounterKind::Shield), 1);
    assert_eq!(raw.counter_count(vehicle, CounterKind::PlusOne), 1);
    settle(&mut raw);
    assert_eq!(raw.hand_count(player), 1);

    game.restore(&saved).unwrap();
    assert_eq!(
        game.apply_index(player, revision, index),
        Err(Error::StaleRevision)
    );
    act(&mut single_counter_first, remove(vehicle_ref));
    reject(&mut single_counter_first, remove(vehicle_ref));
    act(&mut single_counter_first, remove(equipment_ref));
    pay(&mut single_counter_first);
    assert_eq!(
        single_counter_first.counter_count(vehicle, CounterKind::PlusOne),
        0
    );

    act(&mut game, remove(vehicle_ref));
    pay(&mut game);
    assert_eq!(game.counter_count(equipment, CounterKind::PlusOne), 1);
    assert_eq!(game.counter_count(vehicle, CounterKind::PlusOne), 0);
    settle(&mut game);
    assert_eq!(game.hand_count(player), 1);
}

#[test]
fn mana_window_can_create_the_missing_counter_on_a_noncreature_artifact() {
    let _rules = fixture_rules::activate(&[
        (Card::from_name("Solemn Simulacrum").unwrap(), &DRAW),
        (Card::from_name("Swiftfoot Boots").unwrap(), &PRODUCE),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);
    game.state.players[player.index()].mana = [0; 6];
    let source = put(&mut game, player, "Solemn Simulacrum", Zone::Battlefield);
    let artifact = put(&mut game, player, "Swiftfoot Boots", Zone::Battlefield);
    game.add_counter(artifact, CounterKind::PlusOne, 1);
    let object = game.object_ref(artifact);
    assert!(!game.is_creature(artifact));
    act(&mut game, draw_activation(source));
    reject(&mut game, remove(object));
    // The fixed mana activation has a real additional counter-placement cost.
    let (index, mana) = game
        .legal_actions()
        .into_iter()
        .enumerate()
        .find(|(_, action)| {
            matches!(action,
                Action::ActivateMana { card, .. } | Action::AnnounceMana { card, .. } if *card == artifact
            )
        })
        .expect("counter-producing mana action is reachable before payment");
    let mut raw = game.clone();
    act(&mut raw, mana);
    game.apply_index(player, game.revision(), index).unwrap();
    assert_eq!(raw.snapshot(), game.snapshot());
    if game.state.mana_announcement_stack.is_some() {
        pay(&mut game);
    }
    assert_eq!(game.counter_count(artifact, CounterKind::PlusOne), 2);
    assert!(game.object_view(artifact).tapped);
    pay(&mut game);
    choose(&mut game, |action| {
        matches!(action, Action::ChooseCastCost { .. })
    });
    act(&mut game, remove(object));
    assert_eq!(game.counter_count(artifact, CounterKind::PlusOne), 2);
    act(&mut game, remove(object));
    assert_eq!(game.counter_count(artifact, CounterKind::PlusOne), 0);
    settle(&mut game);
    assert_eq!(game.hand_count(player), 1);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
}
