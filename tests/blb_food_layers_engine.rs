//! Food type changes, granted abilities, entry replacements and departure LKI.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules, one};
use rustmage_core::{cards::*, engine::*};
use support::*;

const FOOD_ABILITY: ActivatedAbility = ActivatedAbility {
    exhaust: false,
    guard: ActivationGuard::Always,
    origin: ActivationOrigin::Battlefield,
    cycling: false,
    cost: ActivationCost {
        generic: 2,
        mana_component: true,
        tap: true,
        sacrifice: true,
        ..ActivationCost::ZERO
    },
    targets: TargetPlan::NONE,
    program: &[Instruction::GainLife(PlayerSet::Controller, 3)],
    sorcery: false,
    once_per_turn: false,
};
const WORLD_FOOD: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Type,
        fragments: &[StaticAbilityFragment::OtherCreaturesFood(0)],
    }],
    activated: &[FOOD_ABILITY],
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::Ward,
            targets: TargetPlan::NONE,
            program: &[Instruction::CounterRelatedUnlessSacrificeSubtype("Food")],
        },
        TriggeredAbility {
            condition: TriggerCondition::BattlefieldDeparture(DepartureRule {
                subtype: Some("Food"),
                destinations: &[Zone::Graveyard],
                ..DepartureRule::ANY
            }),
            targets: TargetPlan::NONE,
            program: &[Instruction::PutCounters {
                subject: Subject::Source,
                kind: CounterKind::PlusOne,
                amount: 2,
            }],
        },
    ],
    ..CardRules::EMPTY
};
const CREATURE_OR_FOOD: TargetRule = TargetRule::Union(&[
    TargetRule::Creature,
    TargetRule::Filtered {
        base: &TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 0,
            relation: -1,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        },
        predicate: &TargetPredicate::Subtype("Food"),
    },
]);
const FOOD_AURA: CardRules = CardRules {
    effect: Effect::Aura(one(CREATURE_OR_FOOD)),
    keywords: Keywords::FLASH,
    aura: Some(AuraSpec {
        kinds: 1,
        relation: -1,
        bonus: EquipmentSpec {
            stats: Stats {
                power: 0,
                toughness: 0,
            },
            keywords: Keywords::NONE,
        },
    }),
    extras: &CardExtras {
        aura_kind_or_subtype: Some("Food"),
        ..CardExtras::EMPTY
    },
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Type,
        fragments: &[
            StaticAbilityFragment::AttachedCharacteristics(&AttachedCharacteristics {
                name: None,
                form: PermanentForm::FoodArtifact,
                creature_types: &[],
                colors: &[],
                base: None,
            }),
            StaticAbilityFragment::RemoveAttachedAbilities,
            StaticAbilityFragment::AttachmentActivation {
                ability: 0,
                tap_attachment: false,
            },
        ],
    }],
    activated: &[FOOD_ABILITY],
    ..CardRules::EMPTY
};
const ANIMATE: CardRules = CardRules {
    effect: Effect::Land,
    activated: &[ActivatedAbility {
        cost: ActivationCost::ZERO,
        program: &[Instruction::AnimateSource(AnimationSpec {
            artifact: false,
            creature_types: AnimationCreatureTypes::Named(&["Bear"]),
            base: Some((2, 2)),
            keywords: Keywords::NONE,
            duration: AnimationDuration::EndOfTurn,
        })],
        ..FOOD_ABILITY
    }],
    ..CardRules::EMPTY
};

fn named(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn advance_priority(g: &mut Game) {
    choose(g, |action| {
        matches!(action, Action::Pass | Action::PutTrigger { .. })
    });
}

fn settle_triggers(g: &mut Game) {
    for _ in 0..32 {
        if g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty()
            && g.state.ordering_triggers.is_empty()
        {
            assert_eq!(g.state.pending, Pending::Priority);
            return;
        }
        advance_priority(g);
    }
    panic!("Food trigger resolution exceeded its action bound");
}

#[test]
fn food_world_types_apply_to_entry_replacements_and_later_creature_animation() {
    let _rules =
        fixture_rules::activate(&[(Card::HillGiant, &WORLD_FOOD), (Card::Forest, &ANIMATE)]);
    let actor = Player::P0;
    let enemy = actor.other();
    let mut g = scenario(actor);
    let provider = put(&mut g, enemy, "Hill Giant", Zone::Battlefield);
    let dismantler = put(&mut g, enemy, "Dauntless Dismantler", Zone::Battlefield);
    let ordinary = put(&mut g, actor, "Bear Cub", Zone::Hand);
    let land = put(&mut g, actor, "Forest", Zone::Battlefield);
    let bow = put(&mut g, actor, "Short Bow", Zone::Battlefield);
    cast(&mut g, ordinary, None);
    settle(&mut g);
    let view = g.object_view(ordinary);
    assert!(view.tapped);
    assert_eq!(view.kinds, 1 | 16);
    assert!(view.has_subtype("Food") && view.has_subtype("Bear"));
    assert!(!g.has_subtype(provider, "Food"));
    assert!(g.has_subtype(dismantler, "Food"));
    assert!(!g.has_subtype(land, "Food") && !g.has_subtype(bow, "Food"));

    let trainer = put(&mut g, actor, "Dragon Trainer", Zone::Hand);
    cast(&mut g, trainer, None);
    settle(&mut g);
    let token = g
        .state
        .objects
        .iter()
        .enumerate()
        .find(|(_, object)| object.zone == Zone::Battlefield && object.is_token())
        .map(|(id, _)| g.object_view(id as CardId))
        .unwrap();
    assert!(token.tapped && token.has_subtype("Food") && token.has_subtype("Dragon"));

    choose(
        &mut g,
        |action| matches!(action, Action::Activate { card, ability: 0, .. } if *card == land),
    );
    settle(&mut g);
    let animated = g.object_view(land);
    assert_eq!(animated.kinds, 1 | 2 | 16);
    assert!(animated.has_subtype("Food") && animated.has_subtype("Bear"));
    let granted = Action::AnnounceArtifactFood {
        source: animated.reference,
        grantor: g.object_ref(provider),
        ability: 0,
    };
    assert!(g.legal_actions().contains(&granted));
    g.finish_cleanup();
    assert_eq!(g.object_view(land).kinds, 2);
    assert!(!g.has_subtype(land, "Food"));
}

#[test]
fn food_aura_sets_color_and_types_removes_abilities_and_grants_the_recipient_a_cost() {
    let _rules = fixture_rules::activate(&[(named("Witness Protection"), &FOOD_AURA)]);
    let actor = Player::P0;
    let enemy = actor.other();
    let mut g = scenario(actor);
    let creature = put(&mut g, enemy, "Llanowar Elves", Zone::Battlefield);
    let bow = put(&mut g, enemy, "Short Bow", Zone::Battlefield);
    assert!(g.attach_equipment(g.object_ref(bow), g.object_ref(creature)));
    let aura = put(&mut g, actor, "Witness Protection", Zone::Hand);
    act(&mut g, Action::AnnounceCast { card: aura });
    reject(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(bow),
        },
    );
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(creature),
        },
    );
    pay(&mut g);
    settle(&mut g);
    let view = g.object_view(creature);
    assert_eq!(view.name, "Llanowar Elves");
    assert_eq!(view.kinds, 16);
    assert_eq!(view.colors, 0);
    assert!(!view.is_token());
    assert!(view.has_subtype("Food") && !view.has_subtype("Elf"));
    assert_eq!(g.object_view(bow).attached_to, None);
    let activation = Action::AnnounceAttachedAbility {
        grantor: g.object_ref(aura),
        ability: 0,
    };
    reject(&mut g, activation.clone());
    act(&mut g, Action::Pass);
    assert!(!g.legal_actions().iter().any(|action| {
        matches!(action, Action::ActivateMana { card, .. } | Action::TapMana { card } if *card == creature)
    }));
    act(&mut g, activation);
    pay(&mut g);
    settle(&mut g);
    assert_eq!(g.object_view(creature).zone, Zone::Graveyard);
    assert_eq!(g.object_view(aura).zone, Zone::Graveyard);
    assert_eq!(g.state.players[enemy.index()].life, 23);
    assert_eq!(g.state.players[actor.index()].life, 20);
}

#[test]
fn ward_sacrifice_and_batch_departures_use_pre_event_food_with_exact_graveyard_results() {
    let _rules = fixture_rules::activate(&[(Card::HillGiant, &WORLD_FOOD)]);
    let actor = Player::P0;
    let enemy = actor.other();
    let mut g = scenario(actor);
    let provider = put(&mut g, enemy, "Hill Giant", Zone::Battlefield);
    let sacrifice = put(&mut g, actor, "Bear Cub", Zone::Battlefield);
    let plain_artifact = put(&mut g, actor, "Short Bow", Zone::Battlefield);
    let removal = put(&mut g, actor, "Hero's Downfall", Zone::Hand);
    cast(&mut g, removal, Some(Target::Card(provider)));
    for _ in 0..8 {
        if matches!(g.state.pending, Pending::EffectChoice { .. }) {
            break;
        }
        advance_priority(&mut g);
    }
    let ordinary = g.object_ref(plain_artifact);
    reject(&mut g, Action::Sacrifice { object: ordinary });
    let mut paid = g.clone();
    act(&mut g, Action::DeclineEffectPayment);
    settle_triggers(&mut g);
    assert_eq!(g.object_view(provider).zone, Zone::Battlefield);
    assert_eq!(g.object_view(sacrifice).zone, Zone::Battlefield);

    let food = paid.object_ref(sacrifice);
    act(&mut paid, Action::Sacrifice { object: food });
    for _ in 0..8 {
        if paid.counter_count(provider, CounterKind::PlusOne) == 2 {
            break;
        }
        advance_priority(&mut paid);
    }
    assert_eq!(paid.object_view(sacrifice).zone, Zone::Graveyard);
    assert_eq!(paid.counter_count(provider, CounterKind::PlusOne), 2);
    settle_triggers(&mut paid);
    assert_eq!(paid.object_view(provider).zone, Zone::Graveyard);

    let ordinary_food = put(&mut g, enemy, "Bear Cub", Zone::Battlefield);
    let finality_food = put(&mut g, actor, "Bear Cub", Zone::Battlefield);
    g.add_counter(finality_food, CounterKind::Finality, 1);
    let sweep = put(&mut g, actor, "Day of Judgment", Zone::Hand);
    g.state.priority = actor;
    cast(&mut g, sweep, None);
    act(&mut g, Action::Pass);
    act(&mut g, Action::Pass);
    let triggers = g
        .state
        .stack
        .iter()
        .chain(&g.state.pending_triggers)
        .chain(&g.state.ordering_triggers)
        .filter(|item| {
            matches!(item.kind, StackKind::TriggeredAbility { .. })
                && item.source.is_some_and(|source| source.id == provider)
        })
        .count();
    assert_eq!(triggers, 2);
    assert_eq!(g.object_view(provider).zone, Zone::Graveyard);
    assert_eq!(g.object_view(ordinary_food).zone, Zone::Graveyard);
    assert_eq!(g.object_view(sacrifice).zone, Zone::Graveyard);
    assert_eq!(g.object_view(finality_food).zone, Zone::Exile);
    settle_triggers(&mut g);
}

#[test]
fn food_effect_that_starts_in_layer_four_keeps_its_grant_after_layer_six_removal() {
    let _rules = fixture_rules::activate(&[
        (Card::HillGiant, &WORLD_FOOD),
        (named("Witness Protection"), &FOOD_AURA),
    ]);
    let actor = Player::P0;
    let enemy = actor.other();
    let mut g = scenario(actor);
    let provider = put(&mut g, actor, "Hill Giant", Zone::Battlefield);
    let creature = put(&mut g, enemy, "Bear Cub", Zone::Battlefield);
    let aura = put(&mut g, actor, "Witness Protection", Zone::Hand);
    cast(&mut g, aura, Some(Target::Card(provider)));
    settle(&mut g);
    assert_eq!(g.object_view(provider).kinds, 16);
    assert_eq!(g.object_view(creature).kinds, 1 | 16);
    assert!(g.has_subtype(creature, "Food"));
    act(&mut g, Action::Pass);
    let activation = Action::AnnounceArtifactFood {
        source: g.object_ref(creature),
        grantor: g.object_ref(provider),
        ability: 0,
    };
    act(&mut g, activation);
    pay(&mut g);
    settle_triggers(&mut g);
    assert_eq!(g.object_view(creature).zone, Zone::Graveyard);
    assert_eq!(g.state.players[enemy.index()].life, 23);
    assert_eq!(g.counter_count(provider, CounterKind::PlusOne), 0);
}
