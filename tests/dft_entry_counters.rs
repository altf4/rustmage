//! Live external entry-counter additions feed the existing replacement/prohibition event.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;
use rustmage_core::engine::ability_world::CharacteristicWorld;
use rustmage_core::engine::funding::{FundingGoal, FundingState};
use rustmage_core::engine::*;
use support::*;

const ENTRY_PROVIDER: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[
            StaticAbilityFragment::OtherControlledEntryCountersByManaValue {
                filter: ReplacementFilter::CreatureOrVehicle,
                kind: CounterKind::PlusOne,
                maximum: 4,
                at_most: 1,
                above: 3,
            },
        ],
    }],
    ..CardRules::EMPTY
};
const DOUBLE: ReplacementSpec = ReplacementSpec::Counters {
    filter: ReplacementFilter::Permanent,
    kind: Some(CounterKind::PlusOne),
    factor: 2,
    add: 0,
    effects_only: false,
};
const DOUBLER: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::Replacement(0, DOUBLE)],
    }],
    extras: &CardExtras {
        replacement_effects: &[DOUBLE],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const PROHIBIT_CREATURE: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::CounterProhibition(1)],
    }],
    ..CardRules::EMPTY
};
const OFF_TURN_VEHICLE: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Type,
        fragments: &[StaticAbilityFragment::SourceOffTurnCreature],
    }],
    extras: &CardExtras {
        vehicle_stats: Some(Stats {
            power: 3,
            toughness: 3,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
fn goal(actor: Player) -> FundingGoal {
    FundingGoal {
        actor,
        required: [0; 6],
        generic: 0,
        reserve: None,
        source: None,
    }
}
fn enter(g: &mut Game, moves: Vec<entries::EntryMove>) {
    g.stage_entries(moves, None);
    for _ in 0..16 {
        if g.state.entry_batch.is_none() {
            return;
        }
        if g.advance_entries() {
            choose(g, |action| {
                matches!(
                    action,
                    Action::ChooseReplacement { .. } | Action::ChooseEntryTimestamp { .. }
                )
            });
        }
    }
    panic!("entry transaction did not finish");
}
fn projected_counters(g: &Game, moves: &[entries::EntryMove]) -> Vec<Vec<u64>> {
    let world = FundingState::from_game(g, &goal(moves[0].controller));
    let mut results = Vec::new();
    world.visit_entry_moves(moves.to_vec(), true, None, &mut |next| {
        results.push(
            moves
                .iter()
                .map(|movement| {
                    CharacteristicWorld::counter_count(
                        next,
                        movement.before.id,
                        CounterKind::PlusOne,
                    )
                })
                .collect(),
        );
        false
    });
    results.sort();
    results.dedup();
    results
}

#[test]
fn entry_counter_threshold_uses_four_and_five_and_one_union_match() {
    let _scope =
        fixture_rules::activate(&[(Card::from_name("Veloheart Bike").unwrap(), &ENTRY_PROVIDER)]);
    let mut g = scenario(Player::P0);
    put(&mut g, Player::P0, "Veloheart Bike", Zone::Battlefield);
    let four = put(&mut g, Player::P0, "Hill Giant", Zone::Graveyard);
    let five = put(&mut g, Player::P0, "Serra Angel", Zone::Graveyard);
    let artifact_creature = put(&mut g, Player::P0, "Gleaming Barrier", Zone::Graveyard);
    let vehicle = put(&mut g, Player::P0, "Cultivator's Caravan", Zone::Graveyard);
    let artifact = put(&mut g, Player::P0, "Swiftfoot Boots", Zone::Graveyard);
    let enemy = put(&mut g, Player::P1, "Hill Giant", Zone::Graveyard);
    let ids = [four, five, artifact_creature, vehicle, artifact, enemy];
    let moves: Vec<_> = ids
        .iter()
        .map(|&id| g.entry_move(id, Zone::Battlefield, false))
        .collect();
    let saved = g.snapshot();
    assert_eq!(projected_counters(&g, &moves), vec![vec![1, 3, 1, 1, 0, 0]]);
    assert_eq!(g.snapshot(), saved);
    enter(&mut g, moves);
    for (id, expected) in ids.into_iter().zip([1, 3, 1, 1, 0, 0]) {
        assert_eq!(g.counter_count(id, CounterKind::PlusOne), expected);
    }
    assert_eq!(stats(&g, four), (4, 4));
    assert_eq!(stats(&g, five), (7, 7));
    assert!(
        !g.is_creature(vehicle),
        "an uncrewed Vehicle is still eligible"
    );
}

#[test]
fn entry_counter_additions_stack_then_normal_doubling_replacement_applies() {
    let _scope = fixture_rules::activate(&[
        (Card::from_name("Veloheart Bike").unwrap(), &ENTRY_PROVIDER),
        (Card::from_name("Swiftfoot Boots").unwrap(), &DOUBLER),
    ]);
    let mut g = scenario(Player::P0);
    put(&mut g, Player::P0, "Veloheart Bike", Zone::Battlefield);
    put(&mut g, Player::P0, "Veloheart Bike", Zone::Battlefield);
    put(&mut g, Player::P0, "Swiftfoot Boots", Zone::Battlefield);
    let four = put(&mut g, Player::P0, "Hill Giant", Zone::Graveyard);
    let five = put(&mut g, Player::P0, "Serra Angel", Zone::Graveyard);
    let moves = vec![
        g.entry_move(four, Zone::Battlefield, false),
        g.entry_move(five, Zone::Battlefield, false),
    ];
    assert_eq!(projected_counters(&g, &moves), vec![vec![4, 12]]);
    enter(&mut g, moves);
    assert_eq!(g.counter_count(four, CounterKind::PlusOne), 4);
    assert_eq!(g.counter_count(five, CounterKind::PlusOne), 12);
}

#[test]
fn simultaneous_new_providers_do_not_add_counters_until_they_are_present() {
    let _scope =
        fixture_rules::activate(&[(Card::from_name("Veloheart Bike").unwrap(), &ENTRY_PROVIDER)]);
    let mut g = scenario(Player::P0);
    let first = put(&mut g, Player::P0, "Veloheart Bike", Zone::Graveyard);
    let second = put(&mut g, Player::P0, "Veloheart Bike", Zone::Graveyard);
    let creature = put(&mut g, Player::P0, "Hill Giant", Zone::Graveyard);
    let moves = vec![
        g.entry_move(first, Zone::Battlefield, false),
        g.entry_move(second, Zone::Battlefield, false),
        g.entry_move(creature, Zone::Battlefield, false),
    ];
    assert_eq!(projected_counters(&g, &moves), vec![vec![0, 0, 0]]);
    enter(&mut g, moves);
    assert_eq!(g.counter_count(first, CounterKind::PlusOne), 0);
    assert_eq!(g.counter_count(creature, CounterKind::PlusOne), 0);
    g.move_object(second, Zone::Graveyard);
    let next = g.entry_move(second, Zone::Battlefield, false);
    enter(&mut g, vec![next]);
    assert_eq!(
        g.counter_count(second, CounterKind::PlusOne),
        1,
        "the other already-present provider applies once"
    );
}

#[test]
fn entry_counter_prohibition_has_no_amount_replacer_fast_path_escape() {
    let _scope = fixture_rules::activate(&[
        (Card::from_name("Veloheart Bike").unwrap(), &ENTRY_PROVIDER),
        (
            Card::from_name("Llanowar Elves").unwrap(),
            &PROHIBIT_CREATURE,
        ),
    ]);
    let mut g = scenario(Player::P0);
    put(&mut g, Player::P0, "Veloheart Bike", Zone::Battlefield);
    put(&mut g, Player::P0, "Llanowar Elves", Zone::Battlefield);
    let creature = put(&mut g, Player::P0, "Hill Giant", Zone::Graveyard);
    let movement = g.entry_move(creature, Zone::Battlefield, false);
    assert_eq!(
        projected_counters(&g, std::slice::from_ref(&movement)),
        vec![vec![0]]
    );
    enter(&mut g, vec![movement]);
    assert_eq!(g.counter_count(creature, CounterKind::PlusOne), 0);
}

#[test]
fn entry_prohibition_sees_the_entrants_own_type_effect_but_not_an_incoming_general_provider() {
    let _scope = fixture_rules::activate(&[
        (Card::from_name("Veloheart Bike").unwrap(), &ENTRY_PROVIDER),
        (
            Card::from_name("Llanowar Elves").unwrap(),
            &PROHIBIT_CREATURE,
        ),
        (
            Card::from_name("Cultivator's Caravan").unwrap(),
            &OFF_TURN_VEHICLE,
        ),
    ]);
    for active in [Player::P0, Player::P1] {
        let mut g = scenario(active);
        put(&mut g, Player::P0, "Veloheart Bike", Zone::Battlefield);
        put(&mut g, Player::P0, "Llanowar Elves", Zone::Battlefield);
        let vehicle = put(&mut g, Player::P0, "Cultivator's Caravan", Zone::Graveyard);
        let movement = g.entry_move(vehicle, Zone::Battlefield, false);
        let expected = u64::from(active == Player::P0);
        assert_eq!(
            projected_counters(&g, std::slice::from_ref(&movement)),
            vec![vec![expected]]
        );
        enter(&mut g, vec![movement]);
        assert_eq!(g.counter_count(vehicle, CounterKind::PlusOne), expected);
        assert_eq!(g.is_creature(vehicle), active != Player::P0);
    }
    let mut g = scenario(Player::P0);
    put(&mut g, Player::P0, "Veloheart Bike", Zone::Battlefield);
    let incoming = put(&mut g, Player::P0, "Llanowar Elves", Zone::Graveyard);
    let creature = put(&mut g, Player::P0, "Hill Giant", Zone::Graveyard);
    let moves = vec![
        g.entry_move(incoming, Zone::Battlefield, false),
        g.entry_move(creature, Zone::Battlefield, false),
    ];
    assert_eq!(projected_counters(&g, &moves), vec![vec![1, 1]]);
    enter(&mut g, moves);
    assert_eq!(g.counter_count(incoming, CounterKind::PlusOne), 1);
    assert_eq!(g.counter_count(creature, CounterKind::PlusOne), 1);
}

#[test]
fn entry_counter_bonus_uses_current_control_and_disabled_providers_are_inert() {
    const BLANK: CardRules = CardRules {
        effect: Effect::Aura(TargetPlan::NONE),
        aura: Some(AuraSpec {
            kinds: 0,
            relation: -1,
            bonus: EquipmentSpec {
                stats: Stats {
                    power: 0,
                    toughness: 0,
                },
                keywords: Keywords::NONE,
            },
        }),
        static_groups: &[StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::RemoveAttachedAbilities],
        }],
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[
        (Card::from_name("Veloheart Bike").unwrap(), &ENTRY_PROVIDER),
        (Card::from_name("Pacifism").unwrap(), &BLANK),
    ]);
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P0, "Veloheart Bike", Zone::Battlefield);
    let enemy_owned = put(&mut g, Player::P1, "Hill Giant", Zone::Graveyard);
    let mut movement = g.entry_move(enemy_owned, Zone::Battlefield, false);
    movement.controller = Player::P0;
    assert_eq!(
        projected_counters(&g, std::slice::from_ref(&movement)),
        vec![vec![1]]
    );
    let aura = put(&mut g, Player::P0, "Pacifism", Zone::Battlefield);
    g.state.objects[aura as usize].attached_to = Some(g.object_ref(source));
    assert!(!CharacteristicWorld::printed_abilities_available(
        &g, source
    ));
    assert_eq!(
        projected_counters(&g, std::slice::from_ref(&movement)),
        vec![vec![0]]
    );
    enter(&mut g, vec![movement]);
    assert_eq!(g.state.objects[enemy_owned as usize].controller, Player::P0);
    assert_eq!(g.counter_count(enemy_owned, CounterKind::PlusOne), 0);
}

#[test]
fn intrinsic_entry_counters_also_obey_prohibition_without_an_external_counter_grant() {
    const ENTERS: CardRules = CardRules {
        effect: Effect::Creature(0, 0),
        enters: InitialCounters {
            plus_one: 2,
            ..CardRules::EMPTY.enters
        },
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[
        (Card::from_name("Gleaming Barrier").unwrap(), &ENTERS),
        (
            Card::from_name("Llanowar Elves").unwrap(),
            &PROHIBIT_CREATURE,
        ),
    ]);
    let mut g = scenario(Player::P0);
    put(&mut g, Player::P0, "Llanowar Elves", Zone::Battlefield);
    let creature = put(&mut g, Player::P0, "Gleaming Barrier", Zone::Graveyard);
    assert!(g.has_entry_counter_replacements(Card::from_name("Gleaming Barrier").unwrap()));
    let movement = g.entry_move(creature, Zone::Battlefield, false);
    assert_eq!(
        projected_counters(&g, std::slice::from_ref(&movement)),
        vec![vec![0]]
    );
    enter(&mut g, vec![movement]);
    assert_eq!(g.counter_count(creature, CounterKind::PlusOne), 0);
}
