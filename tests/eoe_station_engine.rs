//! Station's exact paid subject, threshold abilities and projected mana.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const TALL: CardRules = CardRules {
    effect: Effect::Creature(2, 5),
    ..CardRules::EMPTY
};
const WARDEN: CardRules = CardRules {
    effect: Effect::Creature(3, 4),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::ControlledStationWithGreaterToughness],
    }],
    ..CardRules::EMPTY
};
const CHARGE_ABILITY: ActivationGuard = ActivationGuard::SourceCounterAtLeast {
    kind: CounterKind::Charge,
    minimum: 2,
};
const CHARGED: StatValue = StatValue::AtLeast {
    value: &StatValue::SourceCounters(CounterKind::Charge),
    minimum: 2,
};
const ACTIVATE: ActivatedAbility = ActivatedAbility {
    exhaust: false,
    guard: CHARGE_ABILITY,
    cycling: false,
    origin: ActivationOrigin::Battlefield,
    cost: ActivationCost::ZERO,
    targets: TargetPlan::NONE,
    program: &[Instruction::Draw(PlayerSet::Controller, 1)],
    sorcery: false,
    once_per_turn: false,
};
const SHIP_BASE: BaseStatSetter = BaseStatSetter {
    power: Some(4),
    toughness: Some(3),
    own_turn: false,
    condition: Some(CHARGED),
};
const SHIP_GROUPS: &[StaticAbilityGroup] = &[
    StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Type,
        fragments: &[
            StaticAbilityFragment::SourceCounterCreature {
                own_turn: false,
                kind: CounterKind::Charge,
                minimum: 2,
                form: SourceCreatureForm::ArtifactCreature,
                subtypes: &[],
                keywords: Keywords::FLYING,
            },
            StaticAbilityFragment::BaseStatSetter(0, SHIP_BASE),
        ],
    },
    StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Ability,
        fragments: &[
            StaticAbilityFragment::ControlledOtherTappedLegendaryCreatureKeywords {
                condition: CHARGED,
                keywords: Keywords::INDESTRUCTIBLE,
            },
        ],
    },
];
const SHIP: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: SHIP_GROUPS,
    extras: &CardExtras {
        base_stat_setter: Some(SHIP_BASE),
        static_ability_groups: SHIP_GROUPS,
        ..CardExtras::EMPTY
    },
    activated: &[ACTIVATE],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::When {
            guard: StepGuard::Expression {
                value: StatValue::SourceCounters(CounterKind::Charge),
                minimum: Some(2),
                maximum: None,
            },
            event: &TriggerCondition::AtStep {
                step: TriggerStep::BeginCombat,
                relation: CastRelation::You,
                guard: StepGuard::Always,
            },
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::Draw(PlayerSet::Controller, 1)],
    }],
    ..CardRules::EMPTY
};
const REMOVE_CHARGE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[Instruction::RemoveCounter {
            subject: Subject::Targets,
            kind: CounterKind::Charge,
        }],
    ),
    ..CardRules::EMPTY
};
const MANA_SHIP: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            tap: true,
            ..ActivationCost::ZERO
        },
        program: &[Instruction::AddMana(ManaProduction::ChoiceStat {
            colors: 16,
            amount: &StatValue::SourceCounters(CounterKind::Charge),
        })],
        ..ACTIVATE
    }],
    ..CardRules::EMPTY
};

fn activate(card: CardId) -> Action {
    Action::AnnounceActivate {
        card,
        ability: 0,
        counter_payment: Counters::default(),
    }
}

fn tap_cost(game: &Game, donor: CardId) -> Action {
    Action::ChooseAdditionalCost {
        object: game.object_ref(donor),
    }
}

fn station(game: &mut Game, ship: CardId, donor: CardId) {
    act(game, activate(ship));
    let choice = tap_cost(game, donor);
    act(game, choice);
    pay(game);
}

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

#[test]
fn station_uses_one_legal_tap_cost_and_owns_independent_choices() {
    let p = Player::P0;
    let mut parent = scenario(p);
    let ship = put(&mut parent, p, "Warmaker Gunship", Zone::Battlefield);
    let bear = put(&mut parent, p, "Grizzly Bears", Zone::Battlefield);
    let giant = put(&mut parent, p, "Hill Giant", Zone::Battlefield);
    let opponent = put(&mut parent, p.other(), "Bear Cub", Zone::Battlefield);
    let tapped = put(&mut parent, p, "Bear Cub", Zone::Battlefield);
    parent.state.objects[giant as usize].summoning_sick = true;
    parent.state.objects[tapped as usize].tapped = true;
    parent.add_counter(ship, CounterKind::Charge, 3);

    parent.state.step = Step::BeginCombat;
    reject(&mut parent, activate(ship));
    parent.state.step = Step::Main1;
    act(&mut parent, activate(ship));
    for invalid in [ship, opponent, tapped] {
        let choice = tap_cost(&parent, invalid);
        reject(&mut parent, choice);
    }
    let saved = parent.snapshot();
    let mut raw = parent.clone();
    let mut indexed = parent.clone();
    let mut restored = parent.clone();
    drop(parent);

    let choice = tap_cost(&raw, bear);
    act(&mut raw, choice);
    pay(&mut raw);
    settle(&mut raw);
    assert_eq!(raw.counter_count(ship, CounterKind::Charge), 5);
    assert!(!raw.is_creature(ship));
    assert!(raw.object_view(bear).tapped && !raw.object_view(giant).tapped);

    let choice = tap_cost(&indexed, giant);
    let position = indexed
        .legal_actions()
        .iter()
        .position(|action| *action == choice)
        .unwrap();
    indexed
        .apply_index(p, indexed.revision(), position)
        .unwrap();
    pay(&mut indexed);
    settle(&mut indexed);
    assert_eq!(indexed.counter_count(ship, CounterKind::Charge), 6);
    assert_eq!(stats(&indexed, ship), (4, 3));
    assert!(indexed.object_view(ship).has_subtype("Spacecraft"));
    assert!(indexed.keywords(ship).contains(Keywords::FLYING));
    assert!(!indexed.object_view(bear).tapped && indexed.object_view(giant).tapped);
    act(&mut indexed, activate(ship));
    let self_payment = tap_cost(&indexed, ship);
    reject(&mut indexed, self_payment);

    restored.restore(&saved).unwrap();
    let choice = tap_cost(&restored, bear);
    act(&mut restored, choice);
    pay(&mut restored);
    settle(&mut restored);
    assert_eq!(restored.snapshot(), raw.snapshot());
}

const GIVE_CREATURES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec {
            rule: TargetRule::Creature,
            min: 2,
            max: 2,
        }),
        &[Instruction::GainControl {
            subject: Subject::Targets,
            controller: PlayerSet::Opponent,
        }],
    ),
    ..CardRules::EMPTY
};

#[test]
fn station_reads_the_live_or_departed_donor_and_current_toughness_provider() {
    let _rules = fixture_rules::activate(&[
        (Card::GrizzlyBears, &TALL),
        (Card::HillGiant, &WARDEN),
        (Card::from_name("Opt").unwrap(), &GIVE_CREATURES),
    ]);
    let p = Player::P0;
    let mut game = scenario(p);
    let ship = put(&mut game, p, "Warmaker Gunship", Zone::Battlefield);
    let donor = put(&mut game, p, "Grizzly Bears", Zone::Battlefield);
    let warden = put(&mut game, p, "Hill Giant", Zone::Battlefield);
    let growth = put(&mut game, p, "Giant Growth", Zone::Hand);
    let bounce = put(&mut game, p, "Unsummon", Zone::Hand);
    let transfer = put(&mut game, p, "Opt", Zone::Hand);
    station(&mut game, ship, donor);
    cast(&mut game, growth, Some(Target::Card(donor)));
    pass_pair(&mut game);
    assert_eq!(stats(&game, donor), (5, 8));

    let mut live = game.clone();
    settle(&mut live);
    assert_eq!(live.counter_count(ship, CounterKind::Charge), 8);
    assert_eq!(stats(&live, donor), (5, 8));

    let mut no_provider = game.clone();
    cast(&mut no_provider, bounce, Some(Target::Card(warden)));
    settle(&mut no_provider);
    assert_eq!(no_provider.counter_count(ship, CounterKind::Charge), 5);

    let mut other_controller = game.clone();
    announce(
        &mut other_controller,
        transfer,
        &[Target::Card(donor), Target::Card(warden)],
    );
    settle(&mut other_controller);
    assert_eq!(other_controller.object_view(donor).controller, p.other());
    assert_eq!(other_controller.object_view(warden).controller, p.other());
    assert_eq!(other_controller.counter_count(ship, CounterKind::Charge), 5);

    let exact_donor = game.object_ref(donor);
    cast(&mut game, bounce, Some(Target::Card(donor)));
    pass_pair(&mut game);
    assert_eq!(game.object_view(donor).zone, Zone::Hand);
    assert_ne!(game.object_ref(donor), exact_donor);
    settle(&mut game);
    assert_eq!(game.counter_count(ship, CounterKind::Charge), 8);
}

#[test]
fn charge_removal_disables_new_abilities_without_canceling_queued_packets() {
    let definition = Card::from_name("The Seriema").unwrap();
    let _rules =
        fixture_rules::activate(&[(definition, &SHIP), (Card::GiantGrowth, &REMOVE_CHARGE)]);
    let p = Player::P0;
    let mut game = scenario(p);
    let ship = spawn(&mut game, p, definition, Zone::Battlefield);
    let legendary = put(
        &mut game,
        p,
        "Linden, the Steadfast Queen",
        Zone::Battlefield,
    );
    let ordinary = put(&mut game, p, "Grizzly Bears", Zone::Battlefield);
    let opponent = put(
        &mut game,
        p.other(),
        "Linden, the Steadfast Queen",
        Zone::Battlefield,
    );
    let forest = put(&mut game, p, "Forest", Zone::Battlefield);
    let removal = put(&mut game, p, "Giant Growth", Zone::Hand);
    for id in [ship, legendary, ordinary, opponent] {
        game.state.objects[id as usize].tapped = true;
    }
    game.add_counter(ship, CounterKind::Charge, 1);
    reject(&mut game, activate(ship));
    let mut below = game.clone();
    pass_pair(&mut below);
    assert_eq!(below.state.step, Step::BeginCombat);
    assert!(below.state.stack.is_empty());

    game.add_counter(ship, CounterKind::Charge, 1);
    assert!(game.is_creature(ship));
    assert_eq!(stats(&game, ship), (4, 3));
    assert!(game.keywords(legendary).contains(Keywords::INDESTRUCTIBLE));
    for unprotected in [ship, ordinary, opponent] {
        assert!(
            !game
                .keywords(unprotected)
                .contains(Keywords::INDESTRUCTIBLE)
        );
    }
    let mut blanked = game.clone();
    let source = blanked.object_ref(ship);
    blanked.remove_abilities_exact(&[source], source, p, AnimationDuration::Permanent);
    assert!(blanked.is_creature(ship));
    assert_eq!(stats(&blanked, ship), (4, 3));
    assert!(!blanked.keywords(ship).contains(Keywords::FLYING));
    assert!(
        !blanked
            .keywords(legendary)
            .contains(Keywords::INDESTRUCTIBLE)
    );
    reject(&mut blanked, activate(ship));

    pass_pair(&mut game);
    assert_eq!(game.state.step, Step::BeginCombat);
    assert_eq!(game.state.stack.len(), 1);
    act(&mut game, activate(ship));
    pay(&mut game);
    act(&mut game, Action::TapMana { card: forest });
    cast(&mut game, removal, Some(Target::Card(ship)));
    pass_pair(&mut game);

    assert_eq!(game.counter_count(ship, CounterKind::Charge), 1);
    assert!(!game.is_creature(ship));
    assert!(!game.keywords(ship).contains(Keywords::FLYING));
    assert!(!game.keywords(legendary).contains(Keywords::INDESTRUCTIBLE));
    reject(&mut game, activate(ship));
    assert_eq!(game.state.stack.len(), 2);
    settle(&mut game);
    assert_eq!(game.hand_count(p), 2);
}

#[test]
fn threshold_mana_can_fund_a_cast_only_while_the_source_qualifies() {
    let definition = Card::from_name("Warmaker Gunship").unwrap();
    let _rules = fixture_rules::activate(&[(definition, &MANA_SHIP)]);
    let p = Player::P1;
    let mut game = scenario(p);
    game.state.players[p.index()].mana = [0; 6];
    let ship = spawn(&mut game, p, definition, Zone::Battlefield);
    let spell = put(&mut game, p, "Llanowar Elves", Zone::Hand);
    let announce = Action::AnnounceCast { card: spell };
    game.add_counter(ship, CounterKind::Charge, 1);
    reject(&mut game, announce.clone());
    game.add_counter(ship, CounterKind::Charge, 1);
    act(&mut game, announce);
    act(
        &mut game,
        Action::ActivateMana {
            card: ship,
            ability: 0,
            payment: [0; 6],
            counter_payment: Counters::default(),
            production: [0; 6],
        },
    );
    act(
        &mut game,
        Action::ChooseMana {
            production: [0, 0, 0, 0, 2, 0],
        },
    );
    assert!(game.object_view(ship).tapped);
    assert_eq!(game.state.players[p.index()].mana, [0, 0, 0, 0, 2, 0]);
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(spell).zone, Zone::Battlefield);
    assert_eq!(game.state.players[p.index()].mana, [0, 0, 0, 0, 1, 0]);
}
