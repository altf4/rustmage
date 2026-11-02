//! Successful-entry followups use exact result witnesses and existing delayed queues.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const ARTIFACT_GRAVE: TargetRule = TargetRule::Object {
    zone: TargetZone::Graveyard,
    kinds: 16,
    relation: 0,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
};
const CREATURE_GRAVE: TargetRule = TargetRule::Object {
    zone: TargetZone::Graveyard,
    kinds: 1,
    relation: -1,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: 4,
};
const VEHICLE_FORM: AnimationSpec = AnimationSpec {
    artifact: true,
    creature_types: AnimationCreatureTypes::Unchanged,
    base: None,
    keywords: Keywords::NONE,
    duration: AnimationDuration::Permanent,
};
const TUNE_PROGRAM: &[Instruction] = &[
    Instruction::CaptureEntryResult,
    Instruction::ReturnToBattlefieldUnderControl,
    Instruction::AnimateEntryResultIfSubtype {
        subtype: "Vehicle",
        spec: VEHICLE_FORM,
    },
];
const TUNE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(ARTIFACT_GRAVE)),
        TUNE_PROGRAM,
    ),
    ..CardRules::EMPTY
};
const SUSPENDED_VEHICLE: CardRules = CardRules {
    effect: Effect::Permanent,
    entry: EntryTapped::PayLife(2),
    extras: &CardExtras {
        vehicle_stats: Some(Stats {
            power: 4,
            toughness: 2,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const HAND_ENTRY: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::CaptureEntryResult,
            Instruction::PutHandFiltered {
                filter: SearchFilter::AnyOf(&[
                    SearchFilter::Creature,
                    SearchFilter::Subtype("Vehicle"),
                ]),
                skip: 0,
            },
            Instruction::PutEntryResultCountersIfKinds {
                kinds: 16,
                kind: CounterKind::PlusOne,
                amount: 2,
            },
        ],
    ),
    ..CardRules::EMPTY
};
const ENTRY_COUNTER_WITNESS: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::When {
            event: &TriggerCondition::SelfEnters,
            guard: StepGuard::Expression {
                value: StatValue::SourceCounters(CounterKind::PlusOne),
                minimum: Some(1),
                maximum: None,
            },
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 7)],
    }],
    ..CardRules::EMPTY
};
const COALSTOKE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(CREATURE_GRAVE)),
        &[
            Instruction::CaptureEntryResult,
            Instruction::ReturnToBattlefieldWithCounter(CounterKind::Finality),
            Instruction::GrantEntryResultKeywords {
                keywords: Keywords::MENACE
                    .union(Keywords::DEATHTOUCH)
                    .union(Keywords::HASTE),
                permanent: true,
            },
            Instruction::ScheduleEntryResultExile { own_end_step: true },
        ],
    ),
    ..CardRules::EMPTY
};
const PUSH: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::ReturnGraveyardWithDelayedSacrifice(SearchFilter::AnyOf(&[
                SearchFilter::Subtype("Mount"),
                SearchFilter::Subtype("Vehicle"),
            ])),
            Instruction::AnimatePermanentGroup {
                filters: &[BattlefieldFilter {
                    kinds: 0,
                    relation: 0,
                    nonland: false,
                    tapped: None,
                    without: Keywords::NONE,
                    subtype: Some("Vehicle"),
                    max_mana: u32::MAX,
                }],
                spec: AnimationSpec {
                    duration: AnimationDuration::EndOfTurn,
                    ..VEHICLE_FORM
                },
            },
            Instruction::GrantCreatures {
                filter: CreatureFilter {
                    relation: 0,
                    other: false,
                    subtype: None,
                    keyword: Keywords::NONE,
                    attacking: false,
                    tokens: None,
                    artifact: false,
                },
                keywords: Keywords::HASTE,
            },
        ],
    ),
    ..CardRules::EMPTY
};

fn reach_choice(g: &mut Game) {
    for _ in 0..20 {
        if g.state.pending != Pending::Priority {
            return;
        }
        assert!(!g.state.stack.is_empty() || g.state.resolving.is_some());
        act(g, Action::Pass);
    }
    panic!("expected a bounded choice");
}
fn end_step(g: &mut Game, active: Player) {
    g.state.active = active;
    g.state.priority = active;
    g.begin_step(Step::End);
    g.prepare_triggers();
    settle(g);
}

#[test]
fn returned_vehicle_animation_waits_for_exact_successful_entry_and_survives_cleanup() {
    let opt = Card::from_name("Opt").unwrap();
    let bike = Card::from_name("Veloheart Bike").unwrap();
    let _scope = fixture_rules::activate(&[(opt, &TUNE), (bike, &SUSPENDED_VEHICLE)]);
    let mut parent = scenario(Player::P0);
    let vehicle = put(&mut parent, Player::P0, "Veloheart Bike", Zone::Graveyard);
    let old = parent.object_ref(vehicle);
    let spell = put(&mut parent, Player::P0, "Opt", Zone::Hand);
    cast(&mut parent, spell, Some(Target::Card(vehicle)));
    reach_choice(&mut parent);
    assert!(matches!(parent.state.pending, Pending::EntryLife { .. }));
    assert_eq!(parent.object_ref(vehicle), old);
    assert_eq!(parent.state.objects[vehicle as usize].zone, Zone::Graveyard);
    reject(&mut parent, Action::ChooseSearch { card: vehicle });
    let saved = parent.snapshot();
    let mut raw = parent.clone();
    let mut indexed = parent.clone();
    let action = Action::ChooseEntryLife { pay: false };
    let index = indexed
        .legal_actions()
        .iter()
        .position(|candidate| *candidate == action)
        .unwrap();
    let actor = indexed.actor().unwrap();
    let revision = indexed.revision();
    act(&mut raw, action);
    indexed.apply_index(actor, revision, index).unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    drop(parent);
    settle(&mut raw);
    assert!(raw.is_creature(vehicle));
    assert_eq!(stats(&raw, vehicle), (4, 2));
    assert_ne!(raw.object_ref(vehicle), old);
    raw.expire_animations();
    assert!(
        raw.is_creature(vehicle),
        "permanent form survives EOT animation expiry"
    );
    indexed.restore(&saved).unwrap();
    act(&mut indexed, Action::ChooseEntryLife { pay: true });
    settle(&mut indexed);
    assert_eq!(indexed.state.players[0].life, 18);
    assert_eq!(raw.state.players[0].life, 20);
    assert!(indexed.is_creature(vehicle));
}

#[test]
fn returned_nonvehicle_artifact_is_not_animated() {
    let _scope = fixture_rules::activate(&[(Card::from_name("Opt").unwrap(), &TUNE)]);
    let mut g = scenario(Player::P0);
    let artifact = put(&mut g, Player::P0, "Swiftfoot Boots", Zone::Graveyard);
    let spell = put(&mut g, Player::P0, "Opt", Zone::Hand);
    cast(&mut g, spell, Some(Target::Card(artifact)));
    settle(&mut g);
    assert_eq!(g.state.objects[artifact as usize].zone, Zone::Battlefield);
    assert!(!g.is_creature(artifact));
}

#[test]
fn optional_hand_artifact_gets_counters_after_entry_observer_facts() {
    let _scope = fixture_rules::activate(&[
        (Card::from_name("Opt").unwrap(), &HAND_ENTRY),
        (
            Card::from_name("Gleaming Barrier").unwrap(),
            &ENTRY_COUNTER_WITNESS,
        ),
    ]);
    let mut g = scenario(Player::P0);
    let artifact = put(&mut g, Player::P0, "Gleaming Barrier", Zone::Hand);
    let spell = put(&mut g, Player::P0, "Opt", Zone::Hand);
    cast(&mut g, spell, None);
    reach_choice(&mut g);
    assert!(
        g.observe(Player::P1)
            .objects
            .iter()
            .all(|object| object.id != artifact),
        "opponent does not see chosen private card"
    );
    let selected = g.object_ref(artifact);
    act(&mut g, Action::PutHandPermanent { object: selected });
    settle(&mut g);
    assert_eq!(g.counter_count(artifact, CounterKind::PlusOne), 2);
    assert_eq!(
        g.state.players[0].life, 20,
        "entry guard must see zero counters before post-entry placement"
    );
}

#[test]
fn optional_hand_nonartifact_and_decline_do_not_receive_counters() {
    let _scope = fixture_rules::activate(&[(Card::from_name("Opt").unwrap(), &HAND_ENTRY)]);
    for decline in [true, false] {
        let mut g = scenario(Player::P0);
        let creature = put(&mut g, Player::P0, "Savannah Lions", Zone::Hand);
        let spell = put(&mut g, Player::P0, "Opt", Zone::Hand);
        cast(&mut g, spell, None);
        reach_choice(&mut g);
        if decline {
            act(&mut g, Action::DeclineHandEntry);
        } else {
            let selected = g.object_ref(creature);
            act(&mut g, Action::PutHandPermanent { object: selected });
        }
        settle(&mut g);
        assert_eq!(g.counter_count(creature, CounterKind::PlusOne), 0);
        assert_eq!(
            g.state.objects[creature as usize].zone,
            if decline {
                Zone::Hand
            } else {
                Zone::Battlefield
            }
        );
    }
}

#[test]
fn return_with_finality_grants_lasting_keywords_and_waits_for_original_controllers_end() {
    let _scope = fixture_rules::activate(&[(Card::from_name("Opt").unwrap(), &COALSTOKE)]);
    let mut g = scenario(Player::P1);
    let creature = put(&mut g, Player::P1, "Savannah Lions", Zone::Graveyard);
    let spell = put(&mut g, Player::P0, "Opt", Zone::Hand);
    act(&mut g, Action::Pass);
    cast(&mut g, spell, Some(Target::Card(creature)));
    settle(&mut g);
    let returned = g.object_ref(creature);
    assert_eq!(g.state.objects[creature as usize].controller, Player::P0);
    assert_eq!(g.counter_count(creature, CounterKind::Finality), 1);
    let expected = Keywords::MENACE
        .union(Keywords::DEATHTOUCH)
        .union(Keywords::HASTE);
    assert!(g.keywords(creature).contains(expected));
    assert_eq!(g.state.objects[spell as usize].zone, Zone::Graveyard);
    end_step(&mut g, Player::P1);
    assert_eq!(g.state.objects[creature as usize].zone, Zone::Battlefield);
    g.expire_keyword_effects();
    g.state.objects[creature as usize].gained_keywords = Keywords::NONE;
    assert!(
        g.keywords(creature).contains(expected),
        "permanent layer-six grant survives cleanup"
    );
    g.change_controller(returned, Player::P1);
    end_step(&mut g, Player::P0);
    assert_eq!(
        g.state.objects[creature as usize].zone,
        Zone::Exile,
        "original controller exile ignores recipient control change"
    );
}

#[test]
fn delayed_exile_does_not_rebind_a_returned_creature_after_blink() {
    let _scope = fixture_rules::activate(&[(Card::from_name("Opt").unwrap(), &COALSTOKE)]);
    let mut g = scenario(Player::P0);
    let creature = put(&mut g, Player::P0, "Savannah Lions", Zone::Graveyard);
    let spell = put(&mut g, Player::P0, "Opt", Zone::Hand);
    cast(&mut g, spell, Some(Target::Card(creature)));
    settle(&mut g);
    let old = g.object_ref(creature);
    let death = g.move_object(creature, Zone::Graveyard);
    assert_eq!(death.destination, Zone::Exile, "finality replaces death");
    g.move_object(creature, Zone::Battlefield);
    assert_ne!(g.object_ref(creature), old);
    end_step(&mut g, Player::P0);
    assert_eq!(g.state.objects[creature as usize].zone, Zone::Battlefield);
    assert_eq!(g.counter_count(creature, CounterKind::Finality), 0);
}

#[test]
fn mass_return_sacrifices_only_exact_successful_results_and_grants_group_animation_haste() {
    let _scope = fixture_rules::activate(&[(Card::from_name("Opt").unwrap(), &PUSH)]);
    let mut g = scenario(Player::P0);
    let vehicle = put(&mut g, Player::P0, "Veloheart Bike", Zone::Graveyard);
    let mount = put(&mut g, Player::P0, "Alacrian Jaguar", Zone::Graveyard);
    let ordinary = put(&mut g, Player::P0, "Savannah Lions", Zone::Graveyard);
    let enemy = put(&mut g, Player::P1, "Veloheart Bike", Zone::Graveyard);
    let prior_vehicle = put(&mut g, Player::P0, "Veloheart Bike", Zone::Battlefield);
    let prior_creature = put(&mut g, Player::P0, "Savannah Lions", Zone::Battlefield);
    let spell = put(&mut g, Player::P0, "Opt", Zone::Hand);
    cast(&mut g, spell, None);
    settle(&mut g);
    assert_eq!(g.state.objects[ordinary as usize].zone, Zone::Graveyard);
    assert_eq!(g.state.objects[enemy as usize].zone, Zone::Graveyard);
    assert!(g.is_creature(vehicle));
    assert!(g.is_creature(prior_vehicle));
    for id in [vehicle, mount, prior_vehicle, prior_creature] {
        assert!(g.keywords(id).contains(Keywords::HASTE));
    }
    assert_eq!(
        g.state.delayed_sacrifices.as_ref().unwrap().len(),
        1,
        "one delayed trigger for whole simultaneous return"
    );
    let targets = &g.state.delayed_sacrifices.as_ref().unwrap()[0].view.tokens;
    assert_eq!(targets.len(), 2);
    assert!(targets.contains(&g.object_ref(vehicle)) && targets.contains(&g.object_ref(mount)));
    let old_mount = g.object_ref(mount);
    g.move_object(mount, Zone::Exile);
    g.move_object(mount, Zone::Battlefield);
    assert_ne!(g.object_ref(mount), old_mount);
    end_step(&mut g, Player::P0);
    assert_eq!(g.state.objects[vehicle as usize].zone, Zone::Graveyard);
    assert_eq!(g.state.objects[mount as usize].zone, Zone::Battlefield);
    assert_eq!(
        g.state.objects[prior_vehicle as usize].zone,
        Zone::Battlefield
    );
    assert_eq!(
        g.state.objects[prior_creature as usize].zone,
        Zone::Battlefield
    );
}

#[test]
fn delayed_sacrifice_cannot_sacrifice_an_opponent_controlled_result() {
    let _scope = fixture_rules::activate(&[(Card::from_name("Opt").unwrap(), &PUSH)]);
    let mut g = scenario(Player::P0);
    let vehicle = put(&mut g, Player::P0, "Veloheart Bike", Zone::Graveyard);
    let spell = put(&mut g, Player::P0, "Opt", Zone::Hand);
    cast(&mut g, spell, None);
    settle(&mut g);
    g.change_controller(g.object_ref(vehicle), Player::P1);
    end_step(&mut g, Player::P0);
    assert_eq!(g.state.objects[vehicle as usize].zone, Zone::Battlefield);
    assert_eq!(g.state.objects[vehicle as usize].controller, Player::P1);
}

#[test]
fn failed_entry_witness_never_binds_a_later_successful_attempt_of_same_card() {
    const RETRY_AURA: CardRules = CardRules {
        effect: Effect::Spell(
            TargetPlan::Single(TargetSpec::one(TargetRule::Object {
                zone: TargetZone::Graveyard,
                kinds: 32,
                relation: 0,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            })),
            &[
                Instruction::CaptureEntryResult,
                Instruction::ReturnToBattlefieldUnderControl,
                Instruction::PutHandFiltered {
                    filter: SearchFilter::Creature,
                    skip: 0,
                },
                Instruction::ReturnToBattlefieldUnderControl,
                Instruction::GrantEntryResultKeywords {
                    keywords: Keywords::HASTE,
                    permanent: true,
                },
            ],
        ),
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[(Card::from_name("Opt").unwrap(), &RETRY_AURA)]);
    let mut g = scenario(Player::P0);
    let aura = put(&mut g, Player::P0, "Pacifism", Zone::Graveyard);
    let creature = put(&mut g, Player::P0, "Savannah Lions", Zone::Hand);
    let spell = put(&mut g, Player::P0, "Opt", Zone::Hand);
    cast(&mut g, spell, Some(Target::Card(aura)));
    reach_choice(&mut g);
    assert_eq!(
        g.state.objects[aura as usize].zone,
        Zone::Graveyard,
        "first entry is impossible without an enchantable permanent"
    );
    assert!(
        g.state
            .resolving
            .as_ref()
            .unwrap()
            .search_result
            .as_ref()
            .unwrap()
            .closed
    );
    let recipient = g.object_ref(creature);
    act(&mut g, Action::PutHandPermanent { object: recipient });
    assert!(matches!(g.state.pending, Pending::AuraEntry { .. }));
    choose(
        &mut g,
        |action| matches!(action, Action::ChooseAuraAttachment { object } if object.card == creature),
    );
    settle(&mut g);
    assert_eq!(g.state.objects[aura as usize].zone, Zone::Battlefield);
    assert!(
        !g.keywords(aura).contains(Keywords::HASTE),
        "failed first attempt cannot rebind the later incarnation"
    );
    assert!(!g.keywords(creature).contains(Keywords::HASTE));
}

#[test]
fn post_entry_counter_followup_uses_ordinary_replacements() {
    let _scope = fixture_rules::activate(&[
        (Card::from_name("Opt").unwrap(), &HAND_ENTRY),
        (
            Card::from_name("Gleaming Barrier").unwrap(),
            &ENTRY_COUNTER_WITNESS,
        ),
    ]);
    let mut g = scenario(Player::P0);
    put(
        &mut g,
        Player::P0,
        "Caradora, Heart of Alacria",
        Zone::Battlefield,
    );
    let artifact = put(&mut g, Player::P0, "Gleaming Barrier", Zone::Hand);
    let spell = put(&mut g, Player::P0, "Opt", Zone::Hand);
    cast(&mut g, spell, None);
    reach_choice(&mut g);
    let recipient = g.object_ref(artifact);
    act(&mut g, Action::PutHandPermanent { object: recipient });
    settle(&mut g);
    assert_eq!(
        g.counter_count(artifact, CounterKind::PlusOne),
        3,
        "Caradora modifies the post-entry placement"
    );
    assert_eq!(
        g.state.players[0].life, 20,
        "counter replacement must not retroactively alter entry facts"
    );
}
