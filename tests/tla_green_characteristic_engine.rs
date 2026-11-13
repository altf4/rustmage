//! Rule fixtures for effective subtype bonuses and live, projected mana.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::engine::ability_world::CharacteristicWorld;
use rustmage_core::engine::funding::FundingGoal;
use rustmage_core::{cards::*, engine::*};
use support::*;

const GREEN: Mana = [0, 0, 0, 0, 1, 0];
const TWO_GREEN: Mana = [0, 0, 0, 0, 2, 0];
const WHITE: Mana = [1, 0, 0, 0, 0, 0];
const FIVE_BLUE: Mana = [0, 5, 0, 0, 0, 0];
const EMPTY: Mana = [0; 6];

const TAP_ABILITY: ActivatedAbility = ActivatedAbility {
    guard: ActivationGuard::Always,
    origin: ActivationOrigin::Battlefield,
    cycling: false,
    exhaust: false,
    once_per_turn: false,
    sorcery: false,
    cost: ActivationCost {
        tap: true,
        ..ActivationCost::ZERO
    },
    targets: TargetPlan::NONE,
    program: &[],
};
const UNTAPPED_BONUS: ScalingModifier = ScalingModifier {
    subject: Subject::Source,
    value: StatValue::SourceUntapped,
    stats: Stats {
        power: 2,
        toughness: 0,
    },
};
const AUDIENCE: CardRules = CardRules {
    effect: Effect::Creature(2, 1),
    activated: &[ActivatedAbility {
        program: &[Instruction::AddMana(
            ManaProduction::TapManaAtCreaturePowerThreshold {
                color: 4,
                power: 4,
                below: 1,
                at_least: 2,
            },
        )],
        ..TAP_ABILITY
    }],
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Modify,
        fragments: &[StaticAbilityFragment::ScalingModifier(0, UNTAPPED_BONUS)],
    }],
    extras: &CardExtras {
        scaling_modifiers: &[UNTAPPED_BONUS],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const GREATEST_POWER: CardRules = CardRules {
    effect: Effect::Creature(-2, 4),
    activated: &[ActivatedAbility {
        program: &[Instruction::AddMana(
            ManaProduction::TapGreatestControlledCreaturePower { colors: 31 },
        )],
        ..TAP_ABILITY
    }],
    ..CardRules::EMPTY
};
const GUIDE: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::ControlledKindOrSubtypeTapMana {
            kinds: 2,
            subtype: "Ally",
            production: ManaProduction::any_color(1),
        }],
    }],
    ..CardRules::EMPTY
};
const ZOOKEEPER: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Modify,
        fragments: &[StaticAbilityFragment::ControlledCreatureBonusPerSubtype {
            exclude: "Human",
            maximum: 10,
        }],
    }],
    ..CardRules::EMPTY
};
const LAND_COUNTER_STATS: DynamicStats = DynamicStats {
    power: Some(StatExpr {
        value: StatValue::ControlledLandCounters(CounterKind::PlusOne),
        multiplier: 1,
        offset: 0,
    }),
    toughness: None,
};
const LAND_COUNTERS: CardRules = CardRules {
    effect: Effect::Creature(0, 3),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Cda,
        fragments: &[StaticAbilityFragment::DynamicStats(0, LAND_COUNTER_STATS)],
    }],
    extras: &CardExtras {
        dynamic_stats: Some(LAND_COUNTER_STATS),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const LAND_CREATURE_OR_ENTRY: StatValue = StatValue::AtLeast {
    value: &StatValue::Sum(&[
        StatValue::CountAllKinds(CountFilter {
            zone: CountZone::Battlefield,
            relation: 0,
            kind: 3,
            exclude: 0,
            subtype: None,
            basic: false,
            other: false,
            cards_only: false,
        }),
        StatValue::LandsEnteredThisTurn,
    ]),
    minimum: 1,
};
const WRESTLER_BONUS: ScalingModifier = ScalingModifier {
    subject: Subject::Source,
    value: LAND_CREATURE_OR_ENTRY,
    stats: Stats {
        power: 1,
        toughness: 0,
    },
};
const WRESTLERS: CardRules = CardRules {
    effect: Effect::Creature(3, 4),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[
            StaticAbilityFragment::SourceConditionalKeywords {
                condition: LAND_CREATURE_OR_ENTRY,
                keywords: Keywords::TRAMPLE,
            },
            StaticAbilityFragment::ScalingModifier(0, WRESTLER_BONUS),
        ],
    }],
    extras: &CardExtras {
        scaling_modifiers: &[WRESTLER_BONUS],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

fn named(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn fund(game: &Game, actor: Player, required: Mana) -> bool {
    game.can_fund(FundingGoal {
        actor,
        required,
        generic: 0,
        reserve: None,
        source: None,
    })
}

fn mana(card: CardId, production: Mana) -> Action {
    Action::ActivateMana {
        card,
        ability: 0,
        payment: EMPTY,
        counter_payment: Counters::default(),
        production,
    }
}

fn animate(game: &mut Game, card: CardId, types: AnimationCreatureTypes, base: (i128, i128)) {
    game.change_object_form(
        game.object_ref(card),
        AnimationSpec {
            artifact: false,
            creature_types: types,
            base: Some(base),
            keywords: Keywords::NONE,
            duration: AnimationDuration::Permanent,
        },
        characteristic_layers::AnimationTypeMode::AddCreature,
    );
}

#[test]
fn a_land_or_ally_gets_one_exact_grant_even_when_both_conditions_hold() {
    let _rules = fixture_rules::activate(&[(named("Grizzly Bears"), &GUIDE)]);
    let p = Player::P1;
    let mut game = scenario(p);
    game.state.players[p.index()].mana = EMPTY;
    let guide = put(&mut game, p, "Grizzly Bears", Zone::Battlefield);
    let both = put(&mut game, p, "Forest", Zone::Battlefield);
    let land = put(&mut game, p, "Island", Zone::Battlefield);
    let nonally = put(&mut game, p, "Bear Cub", Zone::Battlefield);
    let enemy = put(&mut game, p.other(), "Forest", Zone::Battlefield);
    animate(
        &mut game,
        guide,
        AnimationCreatureTypes::Additional(&["Ally"]),
        (2, 3),
    );
    animate(
        &mut game,
        both,
        AnimationCreatureTypes::Additional(&["Ally"]),
        (1, 1),
    );

    for card in [guide, both, land] {
        let grants = CharacteristicWorld::granted_mana(&game, card);
        assert_eq!(grants.len(), 1);
        assert_eq!(grants[0].source, game.object_ref(guide));
    }
    for card in [nonally, enemy] {
        assert!(CharacteristicWorld::granted_mana(&game, card).is_empty());
    }
    game.state.objects[guide as usize].summoning_sick = true;
    game.state.objects[land as usize].tapped = true;
    let castable = put(&mut game, p, "Healer's Hawk", Zone::Hand);
    let announce = Action::AnnounceCast { card: castable };
    let tap = Action::TapGrantedMana {
        card: both,
        grantor: game.object_ref(guide),
        production: WHITE,
    };
    let mut sick = game.clone();
    sick.state.objects[both as usize].summoning_sick = true;
    reject(&mut sick, tap.clone());
    reject(&mut sick, announce.clone());
    assert!(fund(&game, p, WHITE));
    act(&mut game, announce);
    act(&mut game, tap);
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(castable).zone, Zone::Battlefield);
}

#[test]
fn conditional_tap_mana_checks_power_after_the_tap_in_both_worlds() {
    let _rules = fixture_rules::activate(&[(named("Grizzly Bears"), &AUDIENCE)]);
    let p = Player::P0;
    let mut game = scenario(p);
    game.state.players[p.index()].mana = EMPTY;
    let source = put(&mut game, p, "Grizzly Bears", Zone::Battlefield);
    let enemy = put(&mut game, p.other(), "Hill Giant", Zone::Battlefield);
    game.add_counter(enemy, CounterKind::PlusOne, 2);
    assert_eq!(stats(&game, source), (4, 1));
    assert!(fund(&game, p, GREEN));
    assert!(!fund(&game, p, TWO_GREEN));
    reject(&mut game, mana(source, TWO_GREEN));
    let mut unboosted = game.clone();
    act(&mut unboosted, mana(source, GREEN));
    assert_eq!(unboosted.state.players[p.index()].mana, GREEN);
    assert_eq!(stats(&unboosted, source), (2, 1));

    let own = put(&mut game, p, "Hill Giant", Zone::Battlefield);
    game.add_counter(own, CounterKind::PlusOne, 1);
    assert!(fund(&game, p, TWO_GREEN));
    let castable = put(&mut game, p, "Bear Cub", Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: castable });
    act(&mut game, mana(source, TWO_GREEN));
    assert_eq!(game.state.players[p.index()].mana, TWO_GREEN);
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(castable).zone, Zone::Battlefield);
}

#[test]
fn greatest_creature_power_mana_has_one_color_and_zero_for_negative_power() {
    let _rules = fixture_rules::activate(&[(named("Bear Cub"), &GREATEST_POWER)]);
    let p = Player::P1;
    let mut game = scenario(p);
    game.state.players[p.index()].mana = EMPTY;
    let source = put(&mut game, p, "Bear Cub", Zone::Battlefield);
    assert!(!fund(&game, p, GREEN));
    let mut negative = game.clone();
    act(&mut negative, mana(source, EMPTY));
    assert!(negative.object_view(source).tapped);
    assert_eq!(negative.state.players[p.index()].mana, EMPTY);

    let own = put(&mut game, p, "Hill Giant", Zone::Battlefield);
    let enemy = put(&mut game, p.other(), "Hill Giant", Zone::Battlefield);
    game.add_counter(own, CounterKind::PlusOne, 2);
    game.add_counter(enemy, CounterKind::PlusOne, 4);
    assert!(fund(&game, p, FIVE_BLUE));
    assert!(!fund(&game, p, [3, 2, 0, 0, 0, 0]));
    assert!(!fund(&game, p, [0, 6, 0, 0, 0, 0]));

    let action = mana(source, FIVE_BLUE);
    let checkpoint = game.snapshot();
    let index = game
        .legal_actions()
        .iter()
        .position(|a| *a == action)
        .unwrap();
    let mut indexed = game.clone();
    indexed.apply_index(p, game.revision(), index).unwrap();
    act(&mut game, action);
    assert_eq!(game.snapshot(), indexed.snapshot());
    assert_eq!(game.state.players[p.index()].mana, FIVE_BLUE);
    indexed.restore(&checkpoint).unwrap();
    act(&mut indexed, mana(source, [0, 0, 0, 5, 0, 0]));
    assert_eq!(indexed.state.players[p.index()].mana, [0, 0, 0, 5, 0, 0]);
}

#[test]
fn the_subtype_bonus_counts_only_effective_creature_types_and_caps_at_ten() {
    let _rules = fixture_rules::activate(&[(named("Grizzly Bears"), &ZOOKEEPER)]);
    let p = Player::P0;
    let mut game = scenario(p);
    let provider = put(&mut game, p, "Grizzly Bears", Zone::Battlefield);
    let bear = put(&mut game, p, "Bear Cub", Zone::Battlefield);
    let human = put(
        &mut game,
        p,
        "Linden, the Steadfast Queen",
        Zone::Battlefield,
    );
    let forest = put(&mut game, p, "Forest", Zone::Battlefield);
    let enemy = put(&mut game, p.other(), "Bear Cub", Zone::Battlefield);
    assert_eq!(stats(&game, bear), (3, 3));
    assert_eq!(stats(&game, human), (3, 3));
    assert_eq!(stats(&game, enemy), (2, 2));
    animate(
        &mut game,
        forest,
        AnimationCreatureTypes::Named(&["Cat", "Dog"]),
        (2, 2),
    );
    assert!(game.object_view(forest).has_subtype("Forest"));
    assert_eq!(stats(&game, forest), (4, 4));

    let types = &[
        "Cat", "Dog", "Bear", "Bird", "Beast", "Fox", "Goblin", "Dragon", "Dinosaur", "Elephant",
        "Raccoon",
    ];
    animate(
        &mut game,
        bear,
        AnimationCreatureTypes::Named(types),
        (2, 2),
    );
    assert_eq!(stats(&game, bear), (12, 12));
    animate(&mut game, bear, AnimationCreatureTypes::All, (2, 2));
    assert!(game.object_view(bear).has_subtype("Human"));
    assert_eq!(stats(&game, bear), (2, 2));
    game.move_object(provider, Zone::Graveyard);
    assert_eq!(stats(&game, forest), (2, 2));
}

#[test]
fn the_land_counter_cda_uses_each_counter_and_the_current_land_controller() {
    let _rules = fixture_rules::activate(&[(named("Grizzly Bears"), &LAND_COUNTERS)]);
    let p = Player::P1;
    let mut game = scenario(p);
    let source = put(&mut game, p, "Grizzly Bears", Zone::Graveyard);
    let plain = put(&mut game, p, "Forest", Zone::Battlefield);
    let animated = put(&mut game, p, "Island", Zone::Battlefield);
    let enemy = put(&mut game, p.other(), "Forest", Zone::Battlefield);
    animate(
        &mut game,
        animated,
        AnimationCreatureTypes::Unchanged,
        (2, 2),
    );
    for (land, counters) in [(plain, 2), (animated, 3), (enemy, 6)] {
        game.add_counter(land, CounterKind::PlusOne, counters);
    }
    assert_eq!(stats(&game, source), (5, 3));
    game.move_object(source, Zone::Battlefield);
    game.add_counter(source, CounterKind::PlusOne, 1);
    assert_eq!(stats(&game, source), (6, 4));
    game.change_controller(game.object_ref(plain), p.other());
    assert_eq!(stats(&game, source), (4, 4));
}

#[test]
fn the_land_entry_condition_outlives_the_land_and_resets_next_turn() {
    let _rules = fixture_rules::activate(&[(named("Grizzly Bears"), &WRESTLERS)]);
    let p = Player::P0;
    let mut game = scenario(p);
    let source = put(&mut game, p, "Grizzly Bears", Zone::Battlefield);
    let forest = put(&mut game, p, "Forest", Zone::Battlefield);
    assert_eq!(stats(&game, source), (3, 4));
    let mut animated = game.clone();
    animate(
        &mut animated,
        forest,
        AnimationCreatureTypes::Unchanged,
        (2, 2),
    );
    assert_eq!(stats(&animated, source), (4, 4));
    assert!(
        animated
            .object_view(source)
            .keywords
            .contains(Keywords::TRAMPLE)
    );

    let land = put(&mut game, p, "Island", Zone::Hand);
    act(&mut game, Action::PlayLand { card: land });
    game.move_object(land, Zone::Graveyard);
    assert_eq!(stats(&game, source), (4, 4));
    assert!(
        game.object_view(source)
            .keywords
            .contains(Keywords::TRAMPLE)
    );
    game.state.step = Step::End;
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    assert_eq!(stats(&game, source), (3, 4));
    assert!(
        !game
            .object_view(source)
            .keywords
            .contains(Keywords::TRAMPLE)
    );
}
