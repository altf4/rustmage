//! Exact pre-move departure filters and replacement-adjusted counter events.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const MARKED_NONTOKEN_DEATHS: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::BattlefieldDeparture(DepartureRule {
            kinds: 1,
            relation: CastRelation::You,
            destinations: &[Zone::Graveyard],
            nontoken: true,
            counters: DepartureCounters::Kind(CounterKind::PlusOne),
            ..DepartureRule::ANY
        }),
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokens {
            token: Card::from_catalogue_id(5241),
            amount: 1,
            tapped: false,
        }],
    }],
    ..CardRules::EMPTY
};

const DEATH_OR_EXILE: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::BattlefieldDeparture(DepartureRule {
                kinds: 1,
                relation: CastRelation::You,
                subject: DepartureSubject::Other,
                destinations: &[Zone::Graveyard, Zone::Exile],
                ..DepartureRule::ANY
            }),
            targets: TargetPlan::NONE,
            program: &[
                Instruction::PutCounters {
                    subject: Subject::Source,
                    kind: CounterKind::PlusOne,
                    amount: 1,
                },
                Instruction::GainLife(PlayerSet::Controller, 1),
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::BattlefieldDeparture(DepartureRule {
                subject: DepartureSubject::SelfOnly,
                destinations: &[Zone::Graveyard, Zone::Exile],
                minimum_power: Some(4),
                ..DepartureRule::ANY
            }),
            targets: TargetPlan::Single(TargetSpec {
                rule: TargetRule::Object {
                    zone: TargetZone::Battlefield,
                    kinds: 0,
                    relation: -1,
                    other: false,
                    nonland: true,
                    tapped: false,
                    min_mana: 0,
                    max_mana: u32::MAX,
                },
                min: 0,
                max: 1,
            }),
            program: &[Instruction::Destroy],
        },
    ],
    ..CardRules::EMPTY
};

const OPTIONAL_COUNTER_DRAW: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::CounterPlacedByYou {
            kind: CounterKind::PlusOne,
            filter: CountFilter {
                zone: CountZone::Battlefield,
                relation: 0,
                kind: 1,
                exclude: 0,
                subtype: None,
                basic: false,
                other: false,
                cards_only: false,
            },
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::MayOncePerTurn { skip: 2 },
            Instruction::RecordOncePerTurn,
            Instruction::DrawScaled(PlayerSet::Controller, AmountValue::CounterPlacementAmount),
        ],
    }],
    ..CardRules::EMPTY
};

fn packets(game: &Game, source: CardId) -> impl Iterator<Item = &SpellView> {
    game.state
        .stack
        .iter()
        .chain(&game.state.pending_triggers)
        .chain(&game.state.ordering_triggers)
        .filter(move |packet| packet.source.is_some_and(|entry| entry.id == source))
}

fn resolve_top(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

#[test]
fn death_observer_includes_itself_and_reads_prior_control_token_and_counter_kind() {
    let _fixture = fixture_rules::activate(&[(Card::GrizzlyBears, &MARKED_NONTOKEN_DEATHS)]);
    let p = Player::P0;
    let mut game = scenario(p);
    let source = put(&mut game, p, "Grizzly Bears", Zone::Battlefield);
    let stolen = put(&mut game, p.other(), "Hill Giant", Zone::Battlefield);
    game.state.objects[stolen as usize].controller = p;
    let enemy = put(&mut game, p.other(), "Hill Giant", Zone::Battlefield);
    let unmarked = put(&mut game, p, "Hill Giant", Zone::Battlefield);
    game.add_counter(unmarked, CounterKind::Stun, 1);
    let token = game.state.objects.len() as CardId;
    game.create_tokens(p, Card::from_catalogue_id(5241), 1, false);
    for id in [source, stolen, enemy, token] {
        game.add_counter(id, CounterKind::PlusOne, 1);
    }
    let wrath = put(&mut game, p, "Day of Judgment", Zone::Hand);
    let qualifying = [source, stolen].map(|id| game.object_ref(id));

    let mut exile = game.clone();
    put(&mut exile, p, "Rest in Peace", Zone::Battlefield);
    cast(&mut exile, wrath, None);
    settle(&mut exile);
    assert_eq!(exile.object_view(source).zone, Zone::Exile);
    assert_eq!(packets(&exile, source).count(), 0);
    assert!(
        !exile
            .state
            .objects
            .iter()
            .any(|object| object.zone == Zone::Battlefield && object.is_token())
    );

    cast(&mut game, wrath, None);
    resolve_top(&mut game);
    let observed: Vec<_> = packets(&game, source)
        .map(|packet| packet.event_object.as_ref().unwrap().object)
        .collect();
    assert_eq!(observed.len(), qualifying.len());
    assert!(qualifying.iter().all(|object| observed.contains(object)));
    choose(&mut game, |action| {
        matches!(action, Action::PutTrigger { .. })
    });
    settle(&mut game);
    assert_eq!(
        game.state
            .objects
            .iter()
            .filter(|object| object.zone == Zone::Battlefield && object.is_token())
            .count(),
        2
    );
}

#[test]
fn actual_death_or_exile_checks_each_other_creature_and_self_power_before_the_batch() {
    let _fixture = fixture_rules::activate(&[(Card::GrizzlyBears, &DEATH_OR_EXILE)]);
    let p = Player::P1;
    for destination in [Zone::Graveyard, Zone::Exile] {
        let mut game = scenario(p);
        if destination == Zone::Exile {
            put(&mut game, p, "Rest in Peace", Zone::Battlefield);
        }
        let source = put(&mut game, p, "Grizzly Bears", Zone::Battlefield);
        let ally = put(&mut game, p, "Hill Giant", Zone::Battlefield);
        put(&mut game, p.other(), "Hill Giant", Zone::Battlefield);
        let target = put(
            &mut game,
            p.other(),
            "Soul-Guide Lantern",
            Zone::Battlefield,
        );
        let land = put(&mut game, p.other(), "Forest", Zone::Battlefield);
        let wrath = put(&mut game, p, "Day of Judgment", Zone::Hand);
        game.add_counter(source, CounterKind::PlusOne, 1);

        let mut low_power = game.clone();
        cast(&mut low_power, wrath, None);
        resolve_top(&mut low_power);
        let abilities: Vec<_> = packets(&low_power, source)
            .map(|packet| packet.kind)
            .collect();
        assert_eq!(abilities, [StackKind::TriggeredAbility { ability: 0 }]);

        game.add_counter(source, CounterKind::PlusOne, 1);
        let mut bounce = game.clone();
        let unsummon = put(&mut bounce, p, "Unsummon", Zone::Hand);
        cast(&mut bounce, unsummon, Some(Target::Card(source)));
        settle(&mut bounce);
        assert_eq!(bounce.object_view(source).zone, Zone::Hand);
        assert_eq!(bounce.object_view(target).zone, Zone::Battlefield);

        let ally_before = game.object_ref(ally);
        cast(&mut game, wrath, None);
        resolve_top(&mut game);
        assert_eq!(game.object_view(source).zone, destination);
        assert_eq!(game.object_view(ally).zone, destination);
        let observed: Vec<_> = packets(&game, source).collect();
        assert_eq!(observed.len(), 2);
        let other = observed
            .iter()
            .find(|packet| packet.kind == StackKind::TriggeredAbility { ability: 0 })
            .unwrap();
        assert_eq!(other.event_object.as_ref().unwrap().object, ally_before);
        let own = observed
            .iter()
            .find(|packet| packet.kind == StackKind::TriggeredAbility { ability: 1 })
            .unwrap();
        assert_eq!(own.source.unwrap().facts.power, 4);
        let id = own.id;
        act(&mut game, Action::PutTrigger { id });
        reject(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(land),
            },
        );
        act(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        if game.legal_actions().contains(&Action::FinishTargets) {
            act(&mut game, Action::FinishTargets);
        }
        settle(&mut game);
        assert_eq!(game.object_view(target).zone, destination);
        assert_eq!(game.observe(p).players[p.index()].life, 21);
    }
}

#[test]
fn counter_amount_survives_departure_and_only_acceptance_uses_the_turn_allowance() {
    let _fixture = fixture_rules::activate(&[(
        Card::from_name("Impact Tremors").unwrap(),
        &OPTIONAL_COUNTER_DRAW,
    )]);
    let p = Player::P0;
    let mut game = scenario(p);
    let source = put(&mut game, p, "Impact Tremors", Zone::Battlefield);
    put(&mut game, p, "Doubling Season", Zone::Battlefield);
    let marked = put(&mut game, p, "Hill Giant", Zone::Battlefield);
    let remaining = put(&mut game, p, "Hill Giant", Zone::Battlefield);
    game.add_counter(marked, CounterKind::PlusOne, 1);

    let veil = put(&mut game, p, "Snakeskin Veil", Zone::Hand);
    cast(&mut game, veil, Some(Target::Card(marked)));
    resolve_top(&mut game);
    resolve_top(&mut game);
    act(&mut game, Action::ChooseOptional { accept: false });
    assert_eq!(game.hand_count(p), 0);

    let veil = put(&mut game, p, "Snakeskin Veil", Zone::Hand);
    cast(&mut game, veil, Some(Target::Card(marked)));
    resolve_top(&mut game);
    assert_eq!(game.counter_count(marked, CounterKind::PlusOne), 5);
    let before = game.object_ref(marked);
    let unsummon = put(&mut game, p, "Unsummon", Zone::Hand);
    cast(&mut game, unsummon, Some(Target::Card(marked)));
    resolve_top(&mut game);
    assert_eq!(game.object_view(marked).zone, Zone::Hand);
    let event = packets(&game, source)
        .next()
        .unwrap()
        .event_object
        .as_ref()
        .unwrap();
    assert_eq!(event.object, before);
    assert_eq!(event.facts, EventObjectFacts::CountersPlaced { amount: 2 });
    resolve_top(&mut game);

    let checkpoint = game.snapshot();
    let revision = game.revision();
    let accept = Action::ChooseOptional { accept: true };
    let index = game
        .legal_actions()
        .iter()
        .position(|action| *action == accept)
        .unwrap();
    let mut indexed = game.clone();
    indexed.apply_index(p, revision, index).unwrap();
    act(&mut game, accept);
    assert_eq!(indexed.snapshot(), game.snapshot());
    assert_eq!(game.hand_count(p), 3);
    indexed.restore(&checkpoint).unwrap();
    assert_eq!(
        indexed.apply_index(p, revision, index),
        Err(Error::StaleRevision)
    );
    act(&mut indexed, Action::ChooseOptional { accept: false });
    assert_eq!(indexed.hand_count(p), 1);

    let veil = put(&mut game, p, "Snakeskin Veil", Zone::Hand);
    cast(&mut game, veil, Some(Target::Card(remaining)));
    settle(&mut game);
    assert_eq!(game.counter_count(remaining, CounterKind::PlusOne), 2);
    assert_eq!(game.hand_count(p), 3);
}

const NONTOKEN_ARTIFACT_ENTRIES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::FilteredEntersYou(EntryPredicate::NontokenArtifact),
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
    }],
    ..CardRules::EMPTY
};

#[test]
fn nontoken_artifact_entry_filters_apply_to_the_source_and_other_entrants() {
    let artifact = Card::from_name("Nutrient Block").unwrap();
    let enchantment = Card::from_name("Impact Tremors").unwrap();
    let _fixture = fixture_rules::activate(&[
        (artifact, &NONTOKEN_ARTIFACT_ENTRIES),
        (enchantment, &NONTOKEN_ARTIFACT_ENTRIES),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);

    let observer = spawn(&mut game, player, enchantment, Zone::Hand);
    cast(&mut game, observer, None);
    resolve_top(&mut game);
    assert_eq!(game.object_view(observer).zone, Zone::Battlefield);
    assert_eq!(packets(&game, observer).count(), 0);

    let source = spawn(&mut game, player, artifact, Zone::Hand);
    cast(&mut game, source, None);
    resolve_top(&mut game);
    assert_eq!(game.object_view(source).zone, Zone::Battlefield);
    assert_eq!(packets(&game, source).count(), 1);
    assert_eq!(packets(&game, observer).count(), 1);

    let mut copied = scenario(player);
    let observer = spawn(&mut copied, player, enchantment, Zone::Battlefield);
    copied.stage_copy_creation(player, CopyRecipe::new(artifact), 1);
    assert!(!copied.advance_entries());
    let token = copied
        .state
        .objects
        .iter()
        .enumerate()
        .find_map(|(id, object)| {
            (object.zone == Zone::Battlefield && object.is_token()).then_some(id as CardId)
        })
        .unwrap();
    assert_eq!(copied.object_view(token).kinds & 16, 16);
    assert_eq!(packets(&copied, token).count(), 0);
    assert_eq!(packets(&copied, observer).count(), 0);
}
