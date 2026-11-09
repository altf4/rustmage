//! Pre-event permanent death filters and power-qualified draw packets.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::cards::*;
use rustmage_core::engine::characteristic_layers::AnimationTypeMode;
use rustmage_core::engine::*;
use support::*;

const BATCH_OBSERVER: CardRules = CardRules {
    effect: Effect::Creature(3, 5),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::OncePerTurn(&TriggerCondition::PermanentDiesBatch {
            kinds: 1 | 16,
            relation: CastRelation::You,
            other: true,
        }),
        targets: TargetPlan::NONE,
        program: &[Instruction::Draw(PlayerSet::Controller, 1)],
    }],
    ..CardRules::EMPTY
};
const MUTANT_OBSERVER: CardRules = CardRules {
    effect: Effect::Creature(1, 5),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::When {
            guard: StepGuard::YourTurn,
            event: &TriggerCondition::ControlledPermanentWithSubtypeDies("Mutant"),
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::DrawScaled(
            PlayerSet::Controller,
            AmountValue::DeathEventPower,
        )],
    }],
    ..CardRules::EMPTY
};
const SWEEP: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::MoveBattlefield {
            filters: &[BattlefieldFilter {
                kinds: 1 | 16,
                relation: -1,
                nonland: false,
                tapped: None,
                without: Keywords::NONE,
                subtype: None,
                max_mana: u32::MAX,
            }],
            exile: false,
        }],
    ),
    ..CardRules::EMPTY
};

fn packets(g: &Game, source: CardId) -> Vec<&SpellView> {
    g.state
        .stack
        .iter()
        .chain(&g.state.pending_triggers)
        .chain(&g.state.ordering_triggers)
        .filter(|item| {
            item.kind == StackKind::TriggeredAbility { ability: 0 }
                && item.source.is_some_and(|s| s.id == source)
        })
        .collect()
}

fn drain(g: &mut Game) {
    for _ in 0..20 {
        if g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty()
            && g.state.ordering_triggers.is_empty()
            && g.state.pending == Pending::Priority
        {
            return;
        }
        g.prepare_triggers();
        choose(g, |action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. })
        });
    }
    panic!("20-action departure resolution boundary");
}

fn change_type(g: &mut Game, id: CardId, kinds: AnimationTypeMode, types: AnimationCreatureTypes) {
    g.change_object_form(
        g.object_ref(id),
        AnimationSpec {
            artifact: false,
            creature_types: types,
            base: None,
            keywords: Keywords::NONE,
            duration: AnimationDuration::EndOfTurn,
        },
        kinds,
    );
}

#[test]
fn controlled_death_batch_includes_artifact_only_and_dual_type_once_with_source_lki() {
    let _rules = fixture_rules::activate(&[
        (Card::GrizzlyBears, &BATCH_OBSERVER),
        (Card::from_name("Day of Judgment").unwrap(), &SWEEP),
    ]);
    let p = Player::P0;
    let mut g = scenario(p);
    let source = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let artifact = put(&mut g, p.other(), "Soul-Guide Lantern", Zone::Battlefield);
    g.state.objects[artifact as usize].controller = p;
    let vehicle = put(&mut g, p, "Clamorous Ironclad", Zone::Battlefield);
    g.change_object_form(
        g.object_ref(vehicle),
        AnimationSpec {
            artifact: false,
            creature_types: AnimationCreatureTypes::Unchanged,
            base: Some((2, 2)),
            keywords: Keywords::NONE,
            duration: AnimationDuration::EndOfTurn,
        },
        AnimationTypeMode::AddCreature,
    );
    let opponent = put(&mut g, p, "Hill Giant", Zone::Battlefield);
    g.state.objects[opponent as usize].controller = p.other();
    let old = [artifact, vehicle, source, opponent].map(|id| g.object_ref(id));
    let wrath = put(&mut g, p, "Day of Judgment", Zone::Hand);
    cast(&mut g, wrath, None);
    act(&mut g, Action::Pass);
    let checkpoint = g.snapshot();
    let revision = g.revision();
    let pass = g
        .legal_actions()
        .iter()
        .position(|a| *a == Action::Pass)
        .unwrap();
    let mut indexed = g.clone();
    act(&mut g, Action::Pass);
    indexed.apply_index(p.other(), revision, pass).unwrap();
    assert_eq!(g.snapshot(), indexed.snapshot());

    assert_eq!(g.object_view(source).zone, Zone::Graveyard);
    let packets = packets(&g, source);
    assert_eq!(packets.len(), 1);
    let event = packets[0].event_object.as_ref().unwrap();
    let mut observed = vec![DamageSourceRef::Object(event.object)];
    observed.extend(
        event
            .semantic_objects
            .iter()
            .flat_map(|rows| rows.iter())
            .copied(),
    );
    let expected = [
        old[0],
        old[1],
        g.object_ref(artifact),
        g.object_ref(vehicle),
    ];
    assert_eq!(observed.len(), expected.len());
    assert!(
        expected
            .into_iter()
            .all(|id| observed.contains(&DamageSourceRef::Object(id)))
    );
    assert!(!observed.contains(&DamageSourceRef::Object(old[2])));
    assert!(!observed.contains(&DamageSourceRef::Object(old[3])));
    drain(&mut g);
    assert_eq!(g.hand_count(p), 1);
    indexed.restore(&checkpoint).unwrap();
    assert_eq!(
        indexed.apply_index(p.other(), revision, pass),
        Err(Error::StaleRevision)
    );
}

#[test]
fn death_batch_turn_limit_ignores_self_and_exile_and_resets_next_turn() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &BATCH_OBSERVER)]);
    let p = Player::P1;
    let mut g = scenario(p);
    let source = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let first = put(&mut g, p, "Soul-Guide Lantern", Zone::Battlefield);
    let second = put(&mut g, p, "Soul-Guide Lantern", Zone::Battlefield);
    let third = put(&mut g, p, "Soul-Guide Lantern", Zone::Battlefield);

    let mut exile = g.clone();
    put(&mut exile, p, "Rest in Peace", Zone::Battlefield);
    assert_eq!(
        exile.move_object(first, Zone::Graveyard).destination,
        Zone::Exile
    );
    assert!(packets(&exile, source).is_empty());
    let mut self_only = g.clone();
    self_only.move_object(source, Zone::Graveyard);
    assert!(packets(&self_only, source).is_empty());

    g.move_object(first, Zone::Graveyard);
    drain(&mut g);
    assert_eq!(g.hand_count(p), 1);
    g.move_object(second, Zone::Graveyard);
    assert!(packets(&g, source).is_empty());
    g.state.turn += 1;
    g.state.active = p.other();
    g.begin_turn();
    g.move_object(third, Zone::Graveyard);
    drain(&mut g);
    assert_eq!(g.hand_count(p), 2);
}

#[test]
fn own_turn_mutant_deaths_use_each_pre_event_power_including_the_observer() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &MUTANT_OBSERVER)]);
    let p = Player::P0;
    for active in [p, p.other()] {
        let mut g = scenario(active);
        let source = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let stolen = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        g.state.objects[stolen as usize].controller = p;
        let negative = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.state.objects[negative as usize].growth.power = -3;
        let enemy = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.state.objects[enemy as usize].controller = p.other();
        put(&mut g, p, "Hill Giant", Zone::Battlefield);
        for id in [source, stolen, negative, enemy] {
            change_type(
                &mut g,
                id,
                AnimationTypeMode::PreserveCardTypes,
                AnimationCreatureTypes::Additional(&["Mutant"]),
            );
            assert!(g.object_view(id).has_subtype("Mutant"));
        }
        g.add_counter(source, CounterKind::PlusOne, 1);
        g.add_counter(stolen, CounterKind::PlusOne, 2);
        let expected =
            [source, stolen, negative].map(|id| (g.object_ref(id), g.object_view(id).power));
        let wrath = put(&mut g, active, "Day of Judgment", Zone::Hand);
        cast(&mut g, wrath, None);
        act(&mut g, Action::Pass);
        act(&mut g, Action::Pass);
        assert_eq!(g.object_view(source).zone, Zone::Graveyard);
        let packets = packets(&g, source);
        if active != p {
            assert!(packets.is_empty());
            continue;
        }
        assert_eq!(packets.len(), 3);
        for (reference, power) in expected {
            assert!(packets.iter().any(|item| {
                item.event_object.as_ref().is_some_and(|event| {
                    event.object == reference && matches!(
                        event.facts,
                        EventObjectFacts::CreatureComparison(facts) if facts.creature && facts.power == power
                    )
                })
            }));
        }
        g.move_object(stolen, Zone::Exile);
        g.move_object(stolen, Zone::Battlefield);
        drain(&mut g);
        assert_eq!(g.hand_count(p), 7);
    }
}

#[test]
fn graveyard_cast_observers_pay_for_each_exact_incarnation() {
    use rustmage_core::cards::authored::CardExtras;

    const OBSERVER: CardRules = CardRules {
        effect: Effect::Creature(3, 3),
        entry: EntryTapped::Always,
        triggers: &[TriggeredAbility {
            condition: TriggerCondition::InZone {
                zone: TargetZone::Graveyard,
                event: &TriggerCondition::SpellCast {
                    relation: CastRelation::You,
                    filter: CastFilter::Noncreature,
                    ordinal: None,
                },
            },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::MayPayMana {
                    cost: ManaCost {
                        colored: [0, 0, 1, 0, 0, 0],
                        generic: 0,
                    },
                    skip: 1,
                },
                Instruction::ReturnSourceToHand(TargetZone::Graveyard),
            ],
        }],
        ..CardRules::EMPTY
    };
    const EXILE_AS_COST: CardRules = CardRules {
        effect: Effect::Spell(TargetPlan::NONE, &[]),
        extras: &CardExtras {
            additional_cost: Some(AdditionalCost {
                kind: EffectChoiceKind::ExileGraveyard,
                filter: SacrificeFilter {
                    keywords: Keywords::NONE,
                    source_only: false,
                    kinds: 1,
                    nonland: false,
                    other: false,
                    tokens: None,
                    artifact_enchantment_or_token: false,
                    subtype: None,
                },
            }),
            ..CardExtras::EMPTY
        },
        ..CardRules::EMPTY
    };
    let _rules = fixture_rules::activate(&[
        (Card::GrizzlyBears, &OBSERVER),
        (Card::from_name("Opt").unwrap(), &EXILE_AS_COST),
    ]);
    let p = Player::P1;
    let mut g = scenario(p);
    let first = put(&mut g, p, "Grizzly Bears", Zone::Graveyard);
    let second = put(&mut g, p, "Grizzly Bears", Zone::Graveyard);
    let battlefield = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let enemy = put(&mut g, p.other(), "Grizzly Bears", Zone::Graveyard);

    let mut opposing_cast = g.clone();
    opposing_cast.state.priority = p.other();
    let spell = put(&mut opposing_cast, p.other(), "Think Twice", Zone::Hand);
    cast(&mut opposing_cast, spell, None);
    assert!(packets(&opposing_cast, first).is_empty());
    assert!(packets(&opposing_cast, second).is_empty());
    assert_eq!(packets(&opposing_cast, enemy).len(), 1);

    let mut creature_cast = g.clone();
    let spell = put(&mut creature_cast, p, "Hill Giant", Zone::Hand);
    cast(&mut creature_cast, spell, None);
    assert!(packets(&creature_cast, first).is_empty());
    assert!(packets(&creature_cast, second).is_empty());

    let mut paid_away = g.clone();
    let spell = put(&mut paid_away, p, "Opt", Zone::Hand);
    act(&mut paid_away, Action::AnnounceCast { card: spell });
    let object = paid_away.object_ref(first);
    act(&mut paid_away, Action::ChooseAdditionalCost { object });
    pay(&mut paid_away);
    assert_eq!(paid_away.object_view(first).zone, Zone::Exile);
    assert!(packets(&paid_away, first).is_empty());
    assert_eq!(packets(&paid_away, second).len(), 1);

    let spell = put(&mut g, p, "Think Twice", Zone::Hand);
    cast(&mut g, spell, None);
    assert!(packets(&g, battlefield).is_empty());
    assert!(packets(&g, enemy).is_empty());
    assert_eq!(packets(&g, first).len(), 1);
    let second_packet = packets(&g, second)[0].id;
    act(&mut g, Action::PutTrigger { id: second_packet });
    act(&mut g, Action::Pass);
    act(&mut g, Action::Pass);
    assert_eq!(g.state.pending, Pending::ResolutionMana { player: p });

    let payment = Action::PayResolutionMana {
        payment: [0, 0, 1, 0, 0, 0],
    };
    let checkpoint = g.snapshot();
    let revision = g.revision();
    let index = g
        .legal_actions()
        .iter()
        .position(|action| action == &payment)
        .unwrap();
    let mut raw = g.clone();
    let mut indexed = g.clone();
    act(&mut raw, payment.clone());
    indexed.apply_index(p, revision, index).unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(raw.object_view(first).zone, Zone::Hand);
    assert_eq!(raw.object_view(second).zone, Zone::Graveyard);
    indexed.restore(&checkpoint).unwrap();
    assert_eq!(
        indexed.apply_index(p, revision, index),
        Err(Error::StaleRevision)
    );

    g.move_object(first, Zone::Exile);
    g.move_object(first, Zone::Graveyard);
    act(&mut g, payment);
    assert_eq!(g.object_view(first).zone, Zone::Graveyard);
    act(&mut g, Action::Pass);
    act(&mut g, Action::Pass);
    act(&mut g, Action::DeclineResolutionMana);
    settle(&mut g);
    assert_eq!(g.object_view(second).zone, Zone::Graveyard);
    let later = put(&mut g, p, "Think Twice", Zone::Hand);
    cast(&mut g, later, None);
    assert_eq!(packets(&g, first).len(), 1);
    assert_eq!(packets(&g, second).len(), 1);
}

#[test]
fn scaled_death_counters_read_each_opposing_creatures_actual_pre_departure_power() {
    const OBSERVER: CardRules = CardRules {
        effect: Effect::Creature(2, 2),
        triggers: &[TriggeredAbility {
            condition: TriggerCondition::CreatureDies {
                relation: CastRelation::Opponent,
                other: false,
                nontoken: false,
                subtype: None,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::PutCountersScaled {
                subject: Subject::Source,
                kind: CounterKind::PlusOne,
                amount: AmountValue::DeathEventPower,
            }],
        }],
        ..CardRules::EMPTY
    };
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &OBSERVER)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let positive = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    let negative = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    let friendly = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    game.add_counter(positive, CounterKind::PlusOne, 2);
    game.state.objects[negative as usize].growth.power = -4;
    game.record_control(game.object_ref(friendly), player, None);
    game.sacrifice_batch(&[
        (player.other(), positive),
        (player.other(), negative),
        (player, friendly),
    ]);
    game.prepare_triggers();
    assert_eq!(packets(&game, source).len(), 2);
    drain(&mut game);
    assert_eq!(game.counter_count(source, CounterKind::PlusOne), 5);
}
