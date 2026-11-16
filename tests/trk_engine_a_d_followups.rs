//! Shared rules needed by Star Trek A–D cards, exercised without card catalogue coupling.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const DESTROY_NONARTIFACT_CREATURES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::DestroyCreaturesWithoutKinds(16)],
    ),
    ..CardRules::EMPTY
};
const FORMIDABLE_TARGET: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::creature_you_control())),
        &[
            Instruction::Grow {
                subject: Subject::Targets,
                power: 1,
                toughness: 0,
            },
            Instruction::IfAmountAtLeast {
                value: AmountValue::ControlledTotalCreaturePower,
                minimum: 8,
                skip: 1,
            },
            Instruction::GrantKeywords {
                subject: Subject::Targets,
                keywords: Keywords::MENACE,
            },
        ],
    ),
    ..CardRules::EMPTY
};
const TAP_ARTIFACT_CREATURES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    activated: &[ActivatedAbility {
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        exhaust: false,
        once_per_turn: false,
        sorcery: false,
        cost: ActivationCost {
            additional: Some(AdditionalCost {
                kind: EffectChoiceKind::FilteredCost {
                    action: CostAction::TapPermanents(3),
                    any: &[CostMatch::AllKinds(1 | 16)],
                },
                filter: SacrificeFilter {
                    keywords: Keywords::NONE,
                    source_only: false,
                    kinds: 0,
                    nonland: false,
                    other: false,
                    tokens: None,
                    artifact_enchantment_or_token: false,
                    subtype: None,
                },
            }),
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::Draw(PlayerSet::Controller, 1)],
    }],
    ..CardRules::EMPTY
};
const COUNTER_TRAMPLE: CardRules = CardRules {
    effect: Effect::Creature(3, 1),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::ControlledCreatureCounterRule {
            kind: None,
            keywords: Keywords::TRAMPLE,
        }],
    }],
    ..CardRules::EMPTY
};

#[test]
fn counter_bearing_creatures_receive_rule_keywords_without_other_layer_six_effects() {
    let _rules = fixture_rules::activate(&[(Card::Shock, &COUNTER_TRAMPLE)]);
    let mut game = scenario(Player::P0);
    let provider = spawn(&mut game, Player::P0, Card::Shock, Zone::Battlefield);
    let ally = put(&mut game, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let plain = put(&mut game, Player::P0, "Bear Cub", Zone::Battlefield);
    let enemy = put(&mut game, Player::P1, "Hill Giant", Zone::Battlefield);
    game.set_counters(ally, Counters::of(CounterKind::PlusOne, 1));
    game.set_counters(enemy, Counters::of(CounterKind::PlusOne, 1));
    assert!(game.keywords(ally).contains(Keywords::TRAMPLE));
    assert!(!game.keywords(plain).contains(Keywords::TRAMPLE));
    assert!(!game.keywords(enemy).contains(Keywords::TRAMPLE));
    assert!(!game.keywords(provider).contains(Keywords::TRAMPLE));
}

#[test]
fn simultaneous_destruction_spares_live_artifact_creatures_and_artifacts() {
    let _rules = fixture_rules::activate(&[(Card::Shock, &DESTROY_NONARTIFACT_CREATURES)]);
    let mut game = scenario(Player::P0);
    let own = put(&mut game, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let enemy = put(&mut game, Player::P1, "Hill Giant", Zone::Battlefield);
    let borg = spawn(&mut game, Player::P0, Card(6007), Zone::Battlefield);
    let equipment = put(&mut game, Player::P0, "Tricorder", Zone::Battlefield);
    let spell = put(&mut game, Player::P0, "Shock", Zone::Hand);
    cast(&mut game, spell, None);
    settle(&mut game);
    assert_eq!(game.object_view(own).zone, Zone::Graveyard);
    assert_eq!(game.object_view(enemy).zone, Zone::Graveyard);
    assert_eq!(game.object_view(borg).zone, Zone::Battlefield);
    assert_eq!(game.object_view(equipment).zone, Zone::Battlefield);
}

#[test]
fn total_power_checks_after_target_growth_and_excludes_opponents() {
    let _rules = fixture_rules::activate(&[(Card::Shock, &FORMIDABLE_TARGET)]);
    for meets_threshold in [false, true] {
        let mut game = scenario(Player::P0);
        let target = put(&mut game, Player::P0, "Grizzly Bears", Zone::Battlefield);
        let _ally = put(&mut game, Player::P0, "Grizzly Bears", Zone::Battlefield);
        if meets_threshold {
            put(&mut game, Player::P0, "Hill Giant", Zone::Battlefield);
        } else {
            put(&mut game, Player::P1, "Hill Giant", Zone::Battlefield);
        }
        let spell = put(&mut game, Player::P0, "Shock", Zone::Hand);
        cast(&mut game, spell, Some(Target::Card(target)));
        settle(&mut game);
        assert_eq!(stats(&game, target), (3, 2));
        assert_eq!(
            game.keywords(target).contains(Keywords::MENACE),
            meets_threshold
        );
    }
}

#[test]
fn exact_artifact_creature_tap_cost_rejects_either_type_alone() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &TAP_ARTIFACT_CREATURES)]);
    let mut game = scenario(Player::P0);
    let source = put(&mut game, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let wrong_creature = put(&mut game, Player::P0, "Hill Giant", Zone::Battlefield);
    let wrong_artifact = put(&mut game, Player::P0, "Tricorder", Zone::Battlefield);
    let first = spawn(&mut game, Player::P0, Card(6007), Zone::Battlefield);
    let second = spawn(&mut game, Player::P0, Card(6007), Zone::Battlefield);
    let activate = Action::AnnounceActivate {
        card: source,
        ability: 0,
        counter_payment: Counters::default(),
    };
    assert!(!game.legal_actions().contains(&activate));
    let third = spawn(&mut game, Player::P0, Card(6007), Zone::Battlefield);
    let draw = put(&mut game, Player::P0, "Forest", Zone::Library);
    assert!(game.legal_actions().contains(&activate));
    act(&mut game, activate);
    for wrong in [wrong_creature, wrong_artifact] {
        assert!(
            !game
                .legal_actions()
                .contains(&Action::ChooseAdditionalCost {
                    object: game.object_ref(wrong),
                })
        );
    }
    for exact in [first, second, third] {
        let exact_ref = game.object_ref(exact);
        act(
            &mut game,
            Action::ChooseAdditionalCost { object: exact_ref },
        );
    }
    pay(&mut game);
    settle(&mut game);
    assert!(game.object_view(first).tapped);
    assert!(game.object_view(second).tapped);
    assert!(game.object_view(third).tapped);
    assert_eq!(game.object_view(draw).zone, Zone::Hand);
}
