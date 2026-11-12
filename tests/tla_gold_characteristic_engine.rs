//! Live land and Ally grants and the exact subtype of a triggering spell.
#[path = "support/otj_e_l.rs"]
mod steps;
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::engine::characteristic_layers::AnimationTypeMode;
use rustmage_core::{cards::*, engine::*};
use support::*;

const ALLIES: CountFilter = CountFilter {
    zone: CountZone::Battlefield,
    relation: 0,
    kind: 0,
    exclude: 0,
    subtype: Some("Ally"),
    basic: false,
    other: false,
    cards_only: false,
};
const VIGILANCE: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::ControlledPermanentAllKindsKeywords {
            kinds: 3,
            keywords: Keywords::VIGILANCE,
        }],
    }],
    ..CardRules::EMPTY
};
const MENACE: KeywordGrant = KeywordGrant {
    condition: None,
    requires_life_gain: false,
    filter: CreatureFilter {
        relation: 0,
        other: true,
        subtype: Some("Ally"),
        keyword: Keywords::NONE,
        attacking: false,
        tokens: None,
        artifact: false,
    },
    source_only: false,
    own_turn: false,
    keywords: Keywords::MENACE,
};
const CAST_NONCREATURE: TriggerCondition = TriggerCondition::SpellCast {
    relation: CastRelation::You,
    filter: CastFilter::Noncreature,
    ordinal: None,
};
const PROWESS: &[Instruction] = &[Instruction::Grow {
    subject: Subject::Source,
    power: 1,
    toughness: 1,
}];
const ALLY_GRANTS: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    keywords: Keywords::MENACE,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[
            StaticAbilityFragment::KeywordGrant(0, MENACE),
            StaticAbilityFragment::ControlledSubtypeCastTrigger {
                ability: 1,
                other: true,
                subtype: "Ally",
            },
        ],
    }],
    triggers: &[
        TriggeredAbility {
            condition: CAST_NONCREATURE,
            targets: TargetPlan::NONE,
            program: PROWESS,
        },
        TriggeredAbility {
            condition: TriggerCondition::Granted(&CAST_NONCREATURE),
            targets: TargetPlan::NONE,
            program: PROWESS,
        },
    ],
    extras: &CardExtras {
        keyword_grants: &[MENACE],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const LAND: TargetSpec = TargetSpec::one(TargetRule::Object {
    zone: TargetZone::Battlefield,
    kinds: 2,
    relation: 0,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
});
const PLUS_ONE_ON_TARGET: Instruction = Instruction::PutCounters {
    subject: Subject::Targets,
    kind: CounterKind::PlusOne,
    amount: 1,
};
const EARTHBEND_LESSON: CardRules = CardRules {
    effect: Effect::Creature(3, 4),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SpellCast {
            relation: CastRelation::You,
            filter: CastFilter::Any,
            ordinal: None,
        },
        targets: TargetPlan::Single(LAND),
        program: &[
            Instruction::AnimateEarthbendTargets,
            PLUS_ONE_ON_TARGET,
            Instruction::RegisterEarthbendReturn,
            Instruction::IfEventObjectSubtype {
                subtype: "Lesson",
                skip: 1,
            },
            PLUS_ONE_ON_TARGET,
        ],
    }],
    ..CardRules::EMPTY
};
const COUNTER_OBSERVER: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::CounterPlacedByYou {
            kind: CounterKind::PlusOne,
            filter: CountFilter {
                kind: 2,
                subtype: None,
                ..ALLIES
            },
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLifeScaled(
            PlayerSet::Controller,
            AmountValue::CounterPlacementAmount,
        )],
    }],
    ..CardRules::EMPTY
};
const COLOR_SCALE: ScalingModifier = ScalingModifier {
    subject: Subject::Source,
    value: StatValue::Colors(ALLIES),
    stats: Stats {
        power: 1,
        toughness: 0,
    },
};
const ALLY_COLORS: CardRules = CardRules {
    effect: Effect::Creature(0, 2),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Modify,
        fragments: &[StaticAbilityFragment::ScalingModifier(0, COLOR_SCALE)],
    }],
    extras: &CardExtras {
        scaling_modifiers: &[COLOR_SCALE],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const SCAVENGE: CardRules = CardRules {
    effect: Effect::Creature(0, 4),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfDies,
            targets: TargetPlan::NONE,
            program: &[
                Instruction::MayExileDepartedSource { skip: 1 },
                Instruction::ExileDepartedSourceReflexive { ability: 1 },
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::Reflexive,
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::GraveCreatureSourcePower {
                maximum: None,
            })),
            program: &[Instruction::ReturnToBattlefield],
        },
    ],
    ..CardRules::EMPTY
};

fn named(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn animate(
    game: &mut Game,
    card: CardId,
    types: AnimationCreatureTypes,
    base: Option<(i128, i128)>,
) {
    game.change_object_form(
        game.object_ref(card),
        AnimationSpec {
            artifact: false,
            creature_types: types,
            base,
            keywords: Keywords::NONE,
            duration: AnimationDuration::Permanent,
        },
        AnimationTypeMode::AddCreature,
    );
}

fn packets(game: &Game, source: CardId) -> impl Iterator<Item = &SpellView> {
    game.state
        .stack
        .iter()
        .chain(&game.state.pending_triggers)
        .chain(&game.state.ordering_triggers)
        .filter(move |item| {
            matches!(item.kind, StackKind::TriggeredAbility { .. })
                && item.source().is_some_and(|row| row.id == source)
        })
}

#[test]
fn vigilance_follows_current_controller_and_requires_both_land_and_creature() {
    let _rules = fixture_rules::activate(&[(named("Hill Giant"), &VIGILANCE)]);
    let p = Player::P1;
    let mut game = scenario(p);
    let provider = put(&mut game, p, "Hill Giant", Zone::Battlefield);
    let land = put(&mut game, p, "Forest", Zone::Battlefield);
    let creature = put(&mut game, p, "Bear Cub", Zone::Battlefield);
    let animated = put(&mut game, p, "Island", Zone::Battlefield);
    let enemy = put(&mut game, p.other(), "Mountain", Zone::Battlefield);
    for card in [animated, enemy] {
        animate(
            &mut game,
            card,
            AnimationCreatureTypes::Unchanged,
            Some((2, 2)),
        );
    }
    for card in [provider, land, creature, enemy] {
        assert!(!game.keywords(card).contains(Keywords::VIGILANCE));
    }
    assert!(game.keywords(animated).contains(Keywords::VIGILANCE));

    let mut controlled = game.clone();
    controlled.change_controller(controlled.object_ref(animated), p.other());
    assert!(!controlled.keywords(animated).contains(Keywords::VIGILANCE));
    controlled.change_controller(controlled.object_ref(provider), p.other());
    assert!(controlled.keywords(animated).contains(Keywords::VIGILANCE));

    let mut silenced = game.clone();
    let source = silenced.object_ref(provider);
    silenced.remove_abilities_exact(&[source], source, p, AnimationDuration::Permanent);
    assert!(!silenced.keywords(animated).contains(Keywords::VIGILANCE));

    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    for card in [animated, creature] {
        act(&mut game, Action::Attack { card });
    }
    act(&mut game, Action::FinishAttackers);
    assert!(!game.object_view(animated).tapped);
    assert!(game.object_view(creature).tapped);
}

#[test]
fn other_allies_get_separate_prowess_instances_including_noncreature_kindred() {
    let _rules = fixture_rules::activate(&[(named("Hill Giant"), &ALLY_GRANTS)]);
    let p = Player::P0;
    let mut game = scenario(p);
    let provider = put(&mut game, p, "Hill Giant", Zone::Battlefield);
    let adept = put(&mut game, p, "Shipwreck Dowser", Zone::Battlefield);
    let kindred = put(&mut game, p, "Firdoch Core", Zone::Battlefield);
    let enemy = put(&mut game, p.other(), "Firdoch Core", Zone::Battlefield);
    let ordinary = put(&mut game, p, "Bear Cub", Zone::Battlefield);
    for card in [provider, adept] {
        animate(
            &mut game,
            card,
            AnimationCreatureTypes::Additional(&["Ally"]),
            None,
        );
    }
    for card in [provider, adept, kindred] {
        assert!(game.has_subtype(card, "Ally"));
        assert!(game.keywords(card).contains(Keywords::MENACE));
    }
    assert!(!game.is_creature(kindred));
    for card in [ordinary, enemy] {
        assert!(!game.keywords(card).contains(Keywords::MENACE));
    }
    let source = game.object_ref(provider);
    let mut removed = game.clone();
    let kindred_ref = removed.object_ref(kindred);
    removed.remove_abilities_exact(&[kindred_ref], source, p, AnimationDuration::Permanent);
    assert!(!removed.keywords(kindred).contains(Keywords::MENACE));

    let spell = put(&mut removed, p, "Divination", Zone::Hand);
    cast(&mut removed, spell, None);
    assert_eq!(packets(&removed, kindred).count(), 0);
    assert_eq!(packets(&removed, adept).count(), 2);

    let spell = put(&mut game, p, "Divination", Zone::Hand);
    cast(&mut game, spell, None);
    let origins = |id| {
        packets(&game, id)
            .map(|item| item.attachment_trigger.as_ref().map(|grant| grant.grantor))
            .collect::<Vec<_>>()
    };
    assert_eq!(origins(provider), [None]);
    let both = origins(adept);
    assert_eq!(both.len(), 2);
    assert!(both.contains(&None) && both.contains(&Some(source)));
    assert_eq!(origins(kindred), [Some(source)]);
    assert!(origins(ordinary).is_empty() && origins(enemy).is_empty());
    game.move_object(provider, Zone::Exile);
    steps::resolve(&mut game);
    assert_eq!(stats(&game, adept), (5, 5));
    animate(
        &mut game,
        kindred,
        AnimationCreatureTypes::Unchanged,
        Some((2, 2)),
    );
    assert_eq!(stats(&game, kindred), (3, 3));
    assert!(!game.keywords(adept).contains(Keywords::MENACE));
}

#[test]
fn lesson_earthbend_uses_the_exact_departed_spell_and_two_counter_placements() {
    let _rules = fixture_rules::activate(&[
        (named("Hill Giant"), &EARTHBEND_LESSON),
        (named("Impact Tremors"), &COUNTER_OBSERVER),
    ]);
    let p = Player::P1;
    let mut initial = scenario(p);
    let provider = put(&mut initial, p, "Hill Giant", Zone::Battlefield);
    let observer = put(&mut initial, p, "Impact Tremors", Zone::Battlefield);
    let forest = put(&mut initial, p, "Forest", Zone::Battlefield);
    let island = put(&mut initial, p, "Island", Zone::Battlefield);
    let target = Action::ChooseTarget {
        target: Target::Card(forest),
    };
    for (is_lesson, countered) in [(true, false), (true, true), (false, false)] {
        let mut game = initial.clone();
        let spell = if is_lesson {
            let card = put(&mut game, p, "Firebending Lesson", Zone::Hand);
            act(&mut game, Action::AnnounceCast { card });
            act(&mut game, Action::ChooseKicker { pay: false });
            act(
                &mut game,
                Action::ChooseTarget {
                    target: Target::Card(provider),
                },
            );
            pay(&mut game);
            card
        } else {
            let card = put(&mut game, p, "Lightning Strike", Zone::Hand);
            cast(&mut game, card, Some(Target::Player(p.other())));
            card
        };
        steps::apply_raw_and_index(&mut game, target.clone());
        assert_eq!(packets(&game, provider).count(), 1);
        if countered {
            act(&mut game, Action::Pass);
            let counter = put(&mut game, p.other(), "Negate", Zone::Hand);
            cast(&mut game, counter, Some(Target::Spell(spell)));
            pass_pair(&mut game);
            assert_eq!(game.object_view(spell).zone, Zone::Graveyard);
        }
        pass_pair(&mut game);
        let expected = if is_lesson { 2 } else { 1 };
        assert_eq!(game.counter_count(forest, CounterKind::PlusOne), expected);
        assert_eq!(game.counter_count(island, CounterKind::PlusOne), 0);
        assert_eq!(game.state.players[p.index()].turn_facts.bent_elements, 2);
        assert_eq!(game.earthbend_views().len(), 1);
        let counters = packets(&game, observer)
            .map(|item| {
                let event = item.event_object.as_deref().unwrap();
                assert_eq!(event.object, game.object_ref(forest));
                assert_eq!(event.facts, EventObjectFacts::CountersPlaced { amount: 1 });
            })
            .count();
        assert_eq!(counters as u64, expected);
        steps::resolve(&mut game);
        assert_eq!(
            game.state.players[p.index()].life,
            20 + i128::from(expected)
        );
    }
}

#[test]
fn effective_ally_colors_include_kindred_and_count_each_live_color_once() {
    let _rules = fixture_rules::activate(&[(named("Grizzly Bears"), &ALLY_COLORS)]);
    let p = Player::P0;
    let mut game = scenario(p);
    let provider = put(&mut game, p, "Grizzly Bears", Zone::Battlefield);
    let ally_a = put(&mut game, p, "Shipwreck Dowser", Zone::Battlefield);
    let ally_b = put(&mut game, p, "Shipwreck Dowser", Zone::Battlefield);
    let non_ally = put(&mut game, p, "Healer's Hawk", Zone::Battlefield);
    let opposing = put(&mut game, p.other(), "Hill Giant", Zone::Battlefield);
    for card in [provider, ally_a, ally_b, opposing] {
        animate(
            &mut game,
            card,
            AnimationCreatureTypes::Additional(&["Ally"]),
            None,
        );
    }
    assert!(!game.has_subtype(non_ally, "Ally"));
    assert_eq!(stats(&game, provider), (2, 2));

    let kindred = put(&mut game, p, "Boggart Mischief", Zone::Battlefield);
    let object = game.object_ref(kindred);
    let mut recipe = CopyRecipe::new(game.object_view(kindred).card);
    recipe.replacement_subtype = CreatureType::from_name("Ally");
    game.set_copy_recipe(object, recipe);
    assert!(game.has_subtype(kindred, "Ally"));
    assert!(!game.is_creature(kindred));
    assert_eq!(stats(&game, provider), (3, 2));

    game.set_color_effects(&[kindred], 2 | 8, AnimationDuration::Permanent);
    assert_eq!(stats(&game, provider), (3, 2));
    game.change_controller(object, p.other());
    assert_eq!(stats(&game, provider), (2, 2));
}

#[test]
fn departed_source_power_stays_signed_through_reflexive_target_selection() {
    let _rules = fixture_rules::activate(&[(named("Bear Cub"), &SCAVENGE)]);
    let p = Player::P0;
    for power in [-1, 0, 2] {
        let mut game = scenario(p);
        let source = put(&mut game, p, "Bear Cub", Zone::Battlefield);
        let zero = put(&mut game, p, "Marketback Walker", Zone::Graveyard);
        let two = put(&mut game, p, "Grizzly Bears", Zone::Graveyard);
        let large = put(&mut game, p, "Hill Giant", Zone::Graveyard);
        let enemy = put(&mut game, p.other(), "Grizzly Bears", Zone::Graveyard);
        game.state.objects[source as usize].growth.power = power;
        game.move_object(source, Zone::Graveyard);
        game.prepare_triggers();
        pass_pair(&mut game);
        act(&mut game, Action::ChooseOptional { accept: true });
        assert_eq!(game.object_view(source).zone, Zone::Exile);
        let choices = game.legal_actions();
        let target = |card| Action::ChooseTarget {
            target: Target::Card(card),
        };
        assert_eq!(choices.contains(&target(zero)), power >= 0);
        assert_eq!(choices.contains(&target(two)), power >= 2);
        assert!(!choices.contains(&target(large)) && !choices.contains(&target(enemy)));
        if power == 2 {
            steps::apply_raw_and_index(&mut game, target(two));
            steps::resolve(&mut game);
            assert_eq!(game.object_view(two).zone, Zone::Battlefield);
        }
    }
}
