//! A return effect that changes the entrant before replacements and observers.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;
use rustmage_core::engine::ability_world::CharacteristicWorld;
use rustmage_core::engine::funding::{FundingGoal, FundingState};
use rustmage_core::engine::*;
use support::*;

const RETURNER: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        cycling: false,
        origin: ActivationOrigin::Battlefield,
        cost: ActivationCost {
            tap: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Object {
            zone: TargetZone::Graveyard,
            kinds: 1,
            relation: 0,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        })),
        program: &[Instruction::ReturnToBattlefieldAsSkeletonWithoutAbilities],
        sorcery: true,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};

const ENTRANT: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    keywords: Keywords::FLYING,
    entry: EntryTapped::PayLife(2),
    enters: InitialCounters {
        plus_one: 2,
        ..CardRules::EMPTY.enters
    },
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 5)],
    }],
    extras: &CardExtras {
        entry_choice: Some(EntryChoiceKind::CreatureTypes(&["Giant"])),
        entry_copy: Some(EntryCopySpec {
            filters: &[BattlefieldFilter {
                kinds: 1,
                relation: -1,
                nonland: false,
                tapped: None,
                without: Keywords::NONE,
                subtype: None,
                max_mana: u32::MAX,
            }],
            optional: true,
        }),
        devour: Some(DevourSpec {
            kinds: 2,
            counters_per_permanent: 3,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

const OBSERVER: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::ControlledSubtypeEntryCounters {
            subtype: "Skeleton",
            kind: CounterKind::PlusOne,
            amount: 1,
        }],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SubtypeEntersYou("Skeleton"),
        targets: TargetPlan::NONE,
        program: &[Instruction::Draw(PlayerSet::Controller, 1)],
    }],
    ..CardRules::EMPTY
};

#[test]
fn skeleton_return_suppresses_intrinsic_entry_rules_in_the_actual_and_projected_worlds() {
    let observer = Card::from_name("Impact Tremors").unwrap();
    let _fixture = fixture_rules::activate(&[
        (Card::GrizzlyBears, &RETURNER),
        (Card::HillGiant, &ENTRANT),
        (observer, &OBSERVER),
    ]);
    let p = Player::P0;
    let mut game = scenario(p);
    let source = put(&mut game, p, "Grizzly Bears", Zone::Battlefield);
    spawn(&mut game, p, observer, Zone::Battlefield);
    let forest = put(&mut game, p, "Forest", Zone::Battlefield);
    let returned = put(&mut game, p, "Hill Giant", Zone::Graveyard);
    let foreign = put(&mut game, p.other(), "Hill Giant", Zone::Graveyard);
    let origin = game.object_ref(source);

    let mut projected_entry = game.entry_move(returned, Zone::Battlefield, false);
    projected_entry.return_modification = Some((ReturnKind::SkeletonWithoutAbilities, origin));
    let funding = FundingState::from_game(
        &game,
        &FundingGoal {
            actor: p,
            required: [0; 6],
            generic: 0,
            reserve: None,
            source: None,
        },
    );
    let mut projections = 0;
    funding.visit_entry_moves(vec![projected_entry], true, None, &mut |world| {
        projections += 1;
        assert!(!world.object(returned).tapped);
        assert_eq!(world.object(forest).zone, Zone::Battlefield);
        assert_eq!(world.counter_count(returned, CounterKind::PlusOne), 1);
        assert!(world.subtypes(returned).has(Card::HillGiant, "Skeleton"));
        assert!(!world.printed_abilities_available(returned));
        false
    });
    assert_eq!(projections, 1);

    // The older post-entry removal observes the ordinary entrant and its own ETB.
    let mut later_blank = game.clone();
    let mut movement = later_blank.entry_move(returned, Zone::Battlefield, false);
    movement.return_modification = Some((ReturnKind::WithoutAbilities, origin));
    later_blank.stage_entries(vec![movement], None);
    assert!(later_blank.advance_entries());
    act(&mut later_blank, Action::ChooseEntryCopy { object: None });
    assert!(later_blank.observe(p).entry_devour.is_some());
    act(&mut later_blank, Action::FinishEntryDevour);
    choose(&mut later_blank, |action| {
        matches!(action, Action::ChooseEntryValue { .. })
    });
    act(&mut later_blank, Action::ChooseEntryLife { pay: false });
    settle(&mut later_blank);
    assert_eq!(later_blank.observe(p).players[p.index()].life, 25);
    assert_eq!(later_blank.counter_count(returned, CounterKind::PlusOne), 2);
    assert!(later_blank.object_view(returned).tapped);
    assert!(!later_blank.printed_abilities_available(returned));

    act(
        &mut game,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(foreign),
        },
    );
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(returned),
        },
    );
    if game
        .legal_actions()
        .iter()
        .any(|action| matches!(action, Action::PayCosts { .. }))
    {
        pay(&mut game);
    }
    settle(&mut game);
    let entered = game.object_view(returned);
    assert_eq!(entered.zone, Zone::Battlefield);
    assert!(entered.has_subtype("Skeleton"));
    assert!(entered.has_subtype("Giant"));
    assert!(!entered.tapped);
    assert!(!entered.keywords.contains(Keywords::FLYING));
    assert!(!entered.printed_abilities_available());
    assert_eq!(game.counter_count(returned, CounterKind::PlusOne), 1);
    assert_eq!(game.object_view(forest).zone, Zone::Battlefield);
    assert_eq!(game.observe(p).players[p.index()].life, 20);
    assert_eq!(game.hand_count(p), 1);

    game.move_object(source, Zone::Graveyard);
    game.change_controller(entered.reference, p.other());
    game.grant_keyword_effects(&[returned], entered.reference, Keywords::HASTE);
    let changed = game.object_view(returned);
    assert_eq!(changed.controller, p.other());
    assert!(changed.has_subtype("Skeleton"));
    assert!(changed.keywords.contains(Keywords::HASTE));
    assert!(!changed.printed_abilities_available());
    game.move_object(returned, Zone::Exile);
    assert!(!game.object_view(returned).has_subtype("Skeleton"));
    assert!(game.observe(p).resolved_ability_modifications.is_none());
}
