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
