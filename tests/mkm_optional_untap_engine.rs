//! Optional untap choices and exact source-tapped continuous effects.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const NO_FILTER: SacrificeFilter = SacrificeFilter {
    keywords: Keywords::NONE,
    source_only: false,
    kinds: 0,
    nonland: false,
    other: false,
    tokens: None,
    artifact_enchantment_or_token: false,
    subtype: None,
};
const ANIMATOR: CardRules = CardRules {
    effect: Effect::Creature(0, 3),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::MaySkipOwnUntap],
    }],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            colored: [0, 0, 0, 0, 1, 0],
            generic: 3,
            mana_component: true,
            tap: true,
            additional: Some(AdditionalCost {
                kind: EffectChoiceKind::CollectEvidence(4),
                filter: NO_FILTER,
            }),
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 2,
            relation: 0,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        })),
        program: &[Instruction::AnimateTargetsWhileSourceTapped {
            creature_types: &["Plant", "Boar"],
            colors: 1 << 4,
            base: (5, 5),
            keywords: Keywords::HASTE,
        }],
        sorcery: true,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
const LOCKED: CardRules = CardRules {
    effect: Effect::Creature(0, 3),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[
            StaticAbilityFragment::MaySkipOwnUntap,
            StaticAbilityFragment::SelfUntap(UntapRestriction::ControllerStep),
        ],
    }],
    ..CardRules::EMPTY
};
const UNTAP: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[Instruction::Untap],
    ),
    ..CardRules::EMPTY
};
const CALENDAR: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::OwnUntapStepUntaps,
        targets: TargetPlan::NONE,
        program: &[Instruction::PutCountersScaled {
            subject: Subject::Source,
            kind: CounterKind::Time,
            amount: AmountValue::UntapStepCount,
        }],
    }],
    ..CardRules::EMPTY
};

fn named(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn activation(source: CardId) -> Action {
    Action::AnnounceActivate {
        card: source,
        ability: 0,
        counter_payment: Counters::default(),
    }
}

fn activate(game: &mut Game, source: CardId, land: CardId, evidence: CardId) {
    act(game, activation(source));
    act(
        game,
        Action::ChooseTarget {
            target: Target::Card(land),
        },
    );
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(game, Action::FinishTargets);
    }
    let object = game.object_ref(evidence);
    act(game, Action::ChooseEvidence { object });
    act(game, Action::FinishEvidence);
    pay(game);
    assert!(game.object_view(source).tapped);
    assert_eq!(game.object_view(evidence).zone, Zone::Exile);
}

fn assert_animated(game: &Game, land: CardId) {
    let view = game.object_view(land);
    assert_eq!(view.kinds & 3, 3);
    assert_eq!((view.power, view.toughness), (5, 5));
    assert_eq!(view.colors, 1 << 4);
    for subtype in ["Mountain", "Plant", "Boar"] {
        assert!(view.has_subtype(subtype));
    }
    assert!(view.keywords.contains(Keywords::HASTE));
}

fn advance_to_untap(game: &mut Game, player: Player) {
    for _ in 0..70 {
        if game.state.step == Step::Untap {
            assert_eq!(game.actor(), Some(player));
            return;
        }
        choose(game, |action| {
            matches!(
                action,
                Action::Pass | Action::FinishAttackers | Action::FinishBlockers
            )
        });
    }
    panic!("did not reach the player's optional untap");
}

#[test]
fn optional_choices_precede_the_simultaneous_untap_and_respect_stun_and_prohibitions() {
    let _rules = fixture_rules::activate(&[
        (named("Bear Cub"), &ANIMATOR),
        (named("Healer's Hawk"), &LOCKED),
        (named("Soul-Guide Lantern"), &CALENDAR),
    ]);
    let player = Player::P1;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let other = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let locked = put(&mut game, player, "Healer's Hawk", Zone::Battlefield);
    let calendar = put(&mut game, player, "Soul-Guide Lantern", Zone::Battlefield);
    let land = put(&mut game, player, "Mountain", Zone::Battlefield);
    let evidence = put(&mut game, player, "Hill Giant", Zone::Graveyard);
    activate(&mut game, source, land, evidence);
    settle(&mut game);
    assert_animated(&game, land);
    game.tap_objects_as(player, &[other, locked, land]);
    for id in [other, locked] {
        game.add_counter(id, CounterKind::Stun, 1);
    }

    advance_to_untap(&mut game, player);
    let source_ref = game.object_ref(source);
    let other_ref = game.object_ref(other);
    assert_eq!(game.observe(player).untap_choice, Some(source_ref));
    assert_eq!(game.observe(player.other()).untap_choice, None);
    reject(&mut game, Action::Pass);
    let untap = Action::ChooseOptional { accept: true };
    let stay = Action::ChooseOptional { accept: false };
    let checkpoint = game.snapshot();
    assert_eq!(
        game.apply(player.other(), untap.clone()),
        Err(Error::WrongActor)
    );
    assert_eq!(game.snapshot(), checkpoint);
    let index = game
        .legal_actions()
        .iter()
        .position(|action| *action == untap)
        .unwrap();
    let mut indexed = game.clone();
    indexed.apply_index(player, game.revision(), index).unwrap();
    act(&mut game, untap.clone());
    assert_eq!(indexed.snapshot(), game.snapshot());
    assert_eq!(game.observe(player).untap_choice, Some(other_ref));
    assert_animated(&game, land);
    assert!(game.object_view(source).tapped);
    assert!(game.object_view(land).tapped);
    assert!(game.state.stack.is_empty());

    act(&mut game, stay.clone());
    assert_eq!(game.state.step, Step::Upkeep);
    assert!(!game.object_view(source).tapped);
    assert!(!game.object_view(land).tapped);
    assert!(!game.is_creature(land));
    assert_eq!(game.counter_count(other, CounterKind::Stun), 1);
    assert_eq!(game.counter_count(locked, CounterKind::Stun), 1);
    settle(&mut game);
    assert_eq!(game.counter_count(calendar, CounterKind::Time), 2);

    indexed.restore(&checkpoint).unwrap();
    act(&mut indexed, stay);
    act(&mut indexed, untap);
    assert_eq!(indexed.state.step, Step::Upkeep);
    assert!(indexed.object_view(source).tapped);
    assert!(indexed.object_view(other).tapped);
    assert_eq!(indexed.counter_count(other, CounterKind::Stun), 0);
    assert_eq!(indexed.counter_count(locked, CounterKind::Stun), 1);
    assert!(!indexed.object_view(land).tapped);
    assert_animated(&indexed, land);
    settle(&mut indexed);
    assert_eq!(indexed.counter_count(calendar, CounterKind::Time), 1);
}

#[test]
fn a_tapped_duration_never_restarts_after_a_real_untap_or_departure() {
    let _rules = fixture_rules::activate(&[(named("Bear Cub"), &ANIMATOR), (named("Opt"), &UNTAP)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let land = put(&mut game, player, "Mountain", Zone::Battlefield);
    let evidence = put(&mut game, player, "Hill Giant", Zone::Graveyard);
    let untap_spell = put(&mut game, player, "Opt", Zone::Hand);
    let next_untap = put(&mut game, player, "Opt", Zone::Hand);

    let mut too_early = game.clone();
    too_early.state.step = Step::Upkeep;
    reject(&mut too_early, activation(source));
    let mut no_evidence = game.clone();
    no_evidence.move_object(evidence, Zone::Exile);
    reject(&mut no_evidence, activation(source));

    activate(&mut game, source, land, evidence);
    let mut untapped_before_resolution = game.clone();
    cast(
        &mut untapped_before_resolution,
        untap_spell,
        Some(Target::Card(source)),
    );
    settle(&mut untapped_before_resolution);
    assert!(!untapped_before_resolution.object_view(source).tapped);
    assert!(!untapped_before_resolution.is_creature(land));

    let mut left_before_resolution = game.clone();
    left_before_resolution.move_object(source, Zone::Exile);
    left_before_resolution.move_object(source, Zone::Battlefield);
    left_before_resolution.tap_objects_as(player, &[source]);
    settle(&mut left_before_resolution);
    assert!(!left_before_resolution.is_creature(land));

    pass_pair(&mut game);
    assert_animated(&game, land);
    let mut departed = game.clone();
    departed.move_object(source, Zone::Exile);
    departed.move_object(source, Zone::Battlefield);
    departed.tap_objects_as(player, &[source]);
    assert!(!departed.is_creature(land));
    assert!(departed.observe(player).animations.is_empty());

    game.add_counter(source, CounterKind::Stun, 1);
    cast(&mut game, untap_spell, Some(Target::Card(source)));
    settle(&mut game);
    assert_eq!(game.counter_count(source, CounterKind::Stun), 0);
    assert!(game.object_view(source).tapped);
    assert_animated(&game, land);
    cast(&mut game, next_untap, Some(Target::Card(source)));
    settle(&mut game);
    assert!(!game.object_view(source).tapped);
    assert!(!game.is_creature(land));
    assert!(game.observe(player).animations.is_empty());
    game.tap_objects_as(player, &[source]);
    assert!(!game.is_creature(land));
}
