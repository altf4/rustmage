//! Shared Crew/Saddle rules proofs; these fixtures admit no ordinary DFT card.
mod support;
use rustmage::Card;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const PARTICIPANT: TriggerCondition = TriggerCondition::When {
    guard: StepGuard::YourMainPhase,
    event: &TriggerCondition::SelfCrewsOrSaddles,
};
const FLYING_PARTICIPANT: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: PARTICIPANT,
        targets: TargetPlan::NONE,
        program: &[Instruction::GrantKeywords {
            subject: Subject::Entering,
            keywords: Keywords::FLYING,
        }],
    }],
    ..CardRules::EMPTY
};
const BOOST_PARTICIPANT: CardRules = CardRules {
    effect: Effect::Creature(2, 1),
    triggers: &[TriggeredAbility {
        condition: PARTICIPANT,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::Grow {
                subject: Subject::Entering,
                power: 2,
                toughness: 0,
            },
            Instruction::GrantKeywords {
                subject: Subject::Entering,
                keywords: Keywords::TRAMPLE,
            },
        ],
    }],
    ..CardRules::EMPTY
};
const TOUGHNESS_PARTICIPANT: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::CrewSaddleWithToughness],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::When {
            guard: StepGuard::YourTurn,
            event: &TriggerCondition::OncePerTurn(&TriggerCondition::SelfBecomesTapped),
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::UntapSource],
    }],
    ..CardRules::EMPTY
};
const ABILITY_COUNTER: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Ability)),
        &[Instruction::Counter],
    ),
    ..CardRules::EMPTY
};
const DIRECT_SADDLE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[Instruction::MarkSaddled(Subject::Targets)],
    ),
    ..CardRules::EMPTY
};
const SELF_SADDLE: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost::ZERO,
        targets: TargetPlan::NONE,
        program: &[Instruction::MarkSaddled(Subject::Source)],
        sorcery: false,
        once_per_turn: false,
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfSaddledFirst,
        targets: TargetPlan::NONE,
        program: &[Instruction::PutCounters {
            subject: Subject::Source,
            kind: CounterKind::PlusOne,
            amount: 1,
        }],
    }],
    ..CardRules::EMPTY
};
const ENTRY_DESIGNATIONS: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::PermanentEntersYou {
                kinds: 0,
                nontoken: false,
                subtype: Some("Mount"),
                include_source: false,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::MarkSaddled(Subject::Entering)],
        },
        TriggeredAbility {
            condition: TriggerCondition::PermanentEntersYou {
                kinds: 0,
                nontoken: false,
                subtype: Some("Vehicle"),
                include_source: false,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::AnimateSubject {
                subject: Subject::Entering,
                spec: AnimationSpec {
                    artifact: true,
                    creature_types: AnimationCreatureTypes::Unchanged,
                    base: None,
                    keywords: Keywords::NONE,
                    duration: AnimationDuration::EndOfTurn,
                },
            }],
        },
    ],
    ..CardRules::EMPTY
};

fn begin_cost(g: &mut Game, source: CardId) {
    act(
        g,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    if matches!(g.state.pending, Pending::PayCosts { .. }) {
        pay(g);
    }
}
fn contribute(g: &mut Game, id: CardId, crew: bool) {
    let object = g.object_ref(id);
    act(
        g,
        if crew {
            Action::ChooseCrewer { object }
        } else {
            Action::ChooseSaddler { object }
        },
    );
}
fn finish_cost(g: &mut Game, crew: bool) {
    act(
        g,
        if crew {
            Action::FinishCrew
        } else {
            Action::FinishSaddle
        },
    );
}
fn settle_choices(g: &mut Game) {
    for _ in 0..64 {
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.resolving.is_none()
        {
            return;
        }
        match g.state.pending {
            Pending::OrderTriggers { .. } => choose(g, |a| matches!(a, Action::PutTrigger { .. })),
            Pending::Priority => act(g, Action::Pass),
            _ => panic!("unexpected shared-rule choice {:?}", g.state.pending),
        }
    }
    panic!("64-action shared-rule ceiling");
}

#[test]
fn crew_participant_trigger_is_nontargeted_and_independent_of_countered_crew() {
    let _rules = fixture_rules::activate(&[
        (Card::GrizzlyBears, &FLYING_PARTICIPANT),
        (Card::Counterspell, &ABILITY_COUNTER),
    ]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let pilot = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let vehicle = put(&mut g, p, "Broadcast Rambler", Zone::Battlefield);
        g.state.objects[vehicle as usize].gained_keywords = Keywords::HEXPROOF;
        begin_cost(&mut g, vehicle);
        contribute(&mut g, pilot, true);
        finish_cost(&mut g, true);
        let crew = g
            .state
            .stack
            .iter()
            .find(|item| matches!(item.kind, StackKind::ActivatedAbility { .. }))
            .unwrap()
            .id;
        let trigger = g
            .state
            .stack
            .iter()
            .find(|item| matches!(item.kind, StackKind::TriggeredAbility { .. }))
            .unwrap();
        assert!(trigger.targets.is_empty());
        assert_eq!(trigger.entering_object, Some(g.object_ref(vehicle)));
        assert!(g.object_view(pilot).tapped);
        assert!(!g.is_creature(vehicle));
        act(&mut g, Action::Pass);
        let counter = put(&mut g, p.other(), "Counterspell", Zone::Hand);
        cast(&mut g, counter, Some(Target::Stack(crew)));
        settle_choices(&mut g);
        assert!(!g.is_creature(vehicle), "countered Crew does not animate");
        assert!(
            g.keywords(vehicle).contains(Keywords::FLYING),
            "paid participant event survives countering Crew"
        );
        assert_eq!(g.stats(vehicle).power, 5);
        g.finish_cleanup();
        assert!(!g.keywords(vehicle).contains(Keywords::FLYING));
    }
}

#[test]
fn saddle_participant_boost_survives_source_departure_and_does_not_choose_targets() {
    let _rules = fixture_rules::activate(&[(Card::GoblinPiker, &BOOST_PARTICIPANT)]);
    let mut g = scenario(Player::P0);
    let pilot = put(&mut g, Player::P0, "Goblin Piker", Zone::Battlefield);
    let mount = put(&mut g, Player::P0, "Brightfield Mustang", Zone::Battlefield);
    begin_cost(&mut g, mount);
    contribute(&mut g, pilot, false);
    finish_cost(&mut g, false);
    assert!(
        !g.is_saddled(g.object_ref(mount)),
        "saddling is the unresolved activation's effect"
    );
    assert!(
        g.state.stack.iter().any(
            |item| item.entering_object == Some(g.object_ref(mount)) && item.targets.is_empty()
        )
    );
    g.move_object(pilot, Zone::Graveyard);
    settle_choices(&mut g);
    assert!(g.is_saddled(g.object_ref(mount)));
    assert_eq!(stats(&g, mount), (5, 3));
    assert!(g.keywords(mount).contains(Keywords::TRAMPLE));
    g.finish_cleanup();
    assert_eq!(stats(&g, mount), (3, 3));
    assert!(!g.is_saddled(g.object_ref(mount)));
}

#[test]
fn participant_events_use_each_creatures_controller_and_own_main_phase() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &FLYING_PARTICIPANT)]);
    for p in [Player::P0, Player::P1] {
        for (step, own_turn, expected) in [
            (Step::Main1, true, true),
            (Step::Main2, true, true),
            (Step::BeginCombat, true, false),
            (Step::Main1, false, false),
        ] {
            let mut g = scenario(p);
            g.state.step = step;
            g.state.active = if own_turn { p } else { p.other() };
            let pilot = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
            let vehicle = put(&mut g, p, "Broadcast Rambler", Zone::Battlefield);
            begin_cost(&mut g, vehicle);
            contribute(&mut g, pilot, true);
            finish_cost(&mut g, true);
            settle_choices(&mut g);
            assert_eq!(
                g.keywords(vehicle).contains(Keywords::FLYING),
                expected,
                "{p:?} {step:?} own={own_turn}"
            );
        }
    }
}

#[test]
fn participant_event_freezes_abilities_but_rejects_new_recipient_incarnations() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &FLYING_PARTICIPANT)]);
    let mut g = scenario(Player::P0);
    let pilot = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let vehicle = put(&mut g, Player::P0, "Broadcast Rambler", Zone::Battlefield);
    begin_cost(&mut g, vehicle);
    contribute(&mut g, pilot, true);
    finish_cost(&mut g, true);
    act(&mut g, Action::Pass);
    let blank = put(&mut g, Player::P1, "Stop Cold", Zone::Hand);
    cast(&mut g, blank, Some(Target::Card(pilot)));
    settle_choices(&mut g);
    assert!(!g.object_view(pilot).printed_abilities_available());
    assert!(
        g.keywords(vehicle).contains(Keywords::FLYING),
        "already queued event retains frozen source ability"
    );

    let mut stale = scenario(Player::P0);
    let pilot = put(&mut stale, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let vehicle = put(
        &mut stale,
        Player::P0,
        "Broadcast Rambler",
        Zone::Battlefield,
    );
    begin_cost(&mut stale, vehicle);
    contribute(&mut stale, pilot, true);
    finish_cost(&mut stale, true);
    let old = stale.object_ref(vehicle);
    stale.move_object(vehicle, Zone::Hand);
    stale.move_object(vehicle, Zone::Battlefield);
    assert_ne!(stale.object_ref(vehicle), old);
    settle_choices(&mut stale);
    assert!(!stale.keywords(vehicle).contains(Keywords::FLYING));
    assert!(!stale.is_creature(vehicle));
}

#[test]
fn toughness_contribution_is_effective_signed_and_shared_by_both_cost_forms() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &TOUGHNESS_PARTICIPANT)]);
    for crew in [true, false] {
        let mut g = scenario(Player::P0);
        let pilot = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
        g.state.objects[pilot as usize].growth = Stats {
            power: -3,
            toughness: 2,
        };
        let subject = put(
            &mut g,
            Player::P0,
            if crew {
                "Detention Chariot"
            } else {
                "Unswerving Sloth"
            },
            Zone::Battlefield,
        );
        assert_eq!(
            g.crew_saddle_power(pilot, subject, if crew { "Vehicle" } else { "Mount" }),
            4
        );
        begin_cost(&mut g, subject);
        reject(
            &mut g,
            if crew {
                Action::FinishCrew
            } else {
                Action::FinishSaddle
            },
        );
        contribute(&mut g, pilot, crew);
        let already_selected = g.object_ref(pilot);
        reject(
            &mut g,
            if crew {
                Action::ChooseCrewer {
                    object: already_selected,
                }
            } else {
                Action::ChooseSaddler {
                    object: already_selected,
                }
            },
        );
        finish_cost(&mut g, crew);
        settle_choices(&mut g);
        assert!(
            !g.object_view(pilot).tapped,
            "first own-turn tap trigger untaps contributor"
        );
        if crew {
            assert!(g.is_creature(subject));
        } else {
            assert!(g.is_saddled(g.object_ref(subject)));
        }
        begin_cost(&mut g, subject);
        contribute(&mut g, pilot, crew);
        finish_cost(&mut g, crew);
        settle_choices(&mut g);
        assert!(
            g.object_view(pilot).tapped,
            "once-per-turn tap trigger is exhausted"
        );
    }
    // The shared quantity query preserves signed toughness rather than clamping
    // or accidentally using a large power. This is a representation boundary,
    // before the next SBA would remove a nonpositive-toughness creature.
    let mut g = scenario(Player::P0);
    let pilot = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let vehicle = put(&mut g, Player::P0, "Broadcast Rambler", Zone::Battlefield);
    g.state.objects[pilot as usize].growth = Stats {
        power: 10,
        toughness: -5,
    };
    assert_eq!(g.crew_saddle_power(pilot, vehicle, "Vehicle"), -3);
}

#[test]
fn suppressed_toughness_ability_cannot_pay_with_toughness() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &TOUGHNESS_PARTICIPANT)]);
    let mut g = scenario(Player::P0);
    let pilot = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    g.state.objects[pilot as usize].growth = Stats {
        power: -1,
        toughness: 2,
    };
    let vehicle = put(&mut g, Player::P0, "Detention Chariot", Zone::Battlefield);
    assert_eq!(g.crew_saddle_power(pilot, vehicle, "Vehicle"), 4);
    let blank = put(&mut g, Player::P0, "Stop Cold", Zone::Hand);
    cast(&mut g, blank, Some(Target::Card(pilot)));
    settle_choices(&mut g);
    assert_eq!(g.crew_saddle_power(pilot, vehicle, "Vehicle"), 1);
    g.state.objects[pilot as usize].tapped = false;
    assert!(!g.legal_actions().contains(&Action::AnnounceActivate {
        card: vehicle,
        ability: 0,
        counter_payment: Counters::default()
    }));
    g.move_object(blank, Zone::Graveyard);
    assert_eq!(g.crew_saddle_power(pilot, vehicle, "Vehicle"), 4);
    assert!(g.legal_actions().contains(&Action::AnnounceActivate {
        card: vehicle,
        ability: 0,
        counter_payment: Counters::default()
    }));
}

#[test]
fn direct_saddle_needs_no_saddle_ability_and_fires_first_designation_once() {
    let _rules = fixture_rules::activate(&[
        (Card::GrizzlyBears, &SELF_SADDLE),
        (Card::GiantGrowth, &DIRECT_SADDLE),
    ]);
    let mut g = scenario(Player::P0);
    let creature = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let spell = put(&mut g, Player::P0, "Giant Growth", Zone::Hand);
    cast(&mut g, spell, Some(Target::Card(creature)));
    settle_choices(&mut g);
    assert!(g.is_saddled(g.object_ref(creature)));
    assert_eq!(g.counter_count(creature, CounterKind::PlusOne), 1);
    begin_cost(&mut g, creature);
    settle_choices(&mut g);
    assert_eq!(g.counter_count(creature, CounterKind::PlusOne), 1);
    g.finish_cleanup();
    assert!(!g.is_saddled(g.object_ref(creature)));
    g.state.active = Player::P0;
    g.state.priority = Player::P0;
    g.state.step = Step::Main1;
    g.state.pending = Pending::Priority;
    begin_cost(&mut g, creature);
    settle_choices(&mut g);
    assert_eq!(g.counter_count(creature, CounterKind::PlusOne), 2);
}

#[test]
fn entry_subject_saddling_and_animation_bind_exact_entrant_without_targets() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &ENTRY_DESIGNATIONS)]);
    let mut g = scenario(Player::P0);
    put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let mount = put(&mut g, Player::P0, "Brightfield Glider", Zone::Hand);
    cast(&mut g, mount, None);
    settle_choices(&mut g);
    assert!(g.is_saddled(g.object_ref(mount)));
    let vehicle = put(&mut g, Player::P0, "Air Response Unit", Zone::Hand);
    cast(&mut g, vehicle, None);
    settle_choices(&mut g);
    assert!(g.is_creature(vehicle));
    assert_eq!(stats(&g, vehicle), (3, 3));
    g.finish_cleanup();
    assert!(!g.is_creature(vehicle));
    assert!(!g.is_saddled(g.object_ref(mount)));
}

#[test]
fn crew_choice_raw_index_and_restore_branches_keep_owned_pending_and_events() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &FLYING_PARTICIPANT)]);
    let mut parent = scenario(Player::P0);
    let pilot = put(&mut parent, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let other = put(&mut parent, Player::P0, "Goblin Piker", Zone::Battlefield);
    let vehicle = put(
        &mut parent,
        Player::P0,
        "Broadcast Rambler",
        Zone::Battlefield,
    );
    begin_cost(&mut parent, vehicle);
    let saved = parent.snapshot();
    let mut raw = parent.clone();
    let mut indexed = parent.clone();
    let mut restored = parent.clone();
    drop(parent);
    contribute(&mut raw, pilot, true);
    finish_cost(&mut raw, true);
    settle_choices(&mut raw);
    let action = Action::ChooseCrewer {
        object: indexed.object_ref(other),
    };
    let position = indexed
        .legal_actions()
        .iter()
        .position(|a| *a == action)
        .unwrap();
    indexed
        .apply_index(indexed.actor().unwrap(), indexed.revision(), position)
        .unwrap();
    finish_cost(&mut indexed, true);
    settle_choices(&mut indexed);
    assert!(raw.keywords(vehicle).contains(Keywords::FLYING));
    assert!(!indexed.keywords(vehicle).contains(Keywords::FLYING));
    assert!(raw.object_view(pilot).tapped && !raw.object_view(other).tapped);
    assert!(!indexed.object_view(pilot).tapped && indexed.object_view(other).tapped);
    restored.restore(&saved).unwrap();
    contribute(&mut restored, pilot, true);
    finish_cost(&mut restored, true);
    settle_choices(&mut restored);
    assert_eq!(restored.snapshot(), raw.snapshot());
}

#[test]
fn participant_packets_capture_post_tap_continuous_characteristics() {
    const POST_TAP: CardRules = CardRules {
        static_groups: &[StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::SourceConditionalKeywords {
                condition: StatValue::TappedCount(CountFilter {
                    zone: CountZone::Battlefield,
                    relation: 0,
                    kind: 1,
                    exclude: 0,
                    subtype: None,
                    basic: false,
                    other: false,
                    cards_only: false,
                }),
                keywords: Keywords::HEXPROOF,
            }],
        }],
        ..FLYING_PARTICIPANT
    };
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &POST_TAP)]);
    let mut g = scenario(Player::P0);
    let pilot = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let vehicle = put(&mut g, Player::P0, "Broadcast Rambler", Zone::Battlefield);
    assert!(!g.keywords(pilot).contains(Keywords::HEXPROOF));
    begin_cost(&mut g, vehicle);
    contribute(&mut g, pilot, true);
    finish_cost(&mut g, true);
    assert!(g.keywords(pilot).contains(Keywords::HEXPROOF));
    let trigger = g
        .state
        .stack
        .iter()
        .find(|item| matches!(item.kind, StackKind::TriggeredAbility { .. }))
        .unwrap();
    assert!(trigger.source_tapped);
    assert!(
        trigger.source_keywords.contains(Keywords::HEXPROOF),
        "post-tap derived source facts, not prepayment facts"
    );
    settle_choices(&mut g);
    assert!(g.keywords(vehicle).contains(Keywords::FLYING));
}
