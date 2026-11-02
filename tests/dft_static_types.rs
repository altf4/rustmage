//! Live static type effects and shared Aura eligibility use one characteristic world.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;
use rustmage_core::engine::ability_world::CharacteristicWorld;
use rustmage_core::engine::funding::{FundingGoal, FundingState};
use rustmage_core::engine::*;
use support::*;

const VEHICLE_AURA_SPEC: AuraSpec = AuraSpec {
    kinds: 1,
    relation: -1,
    bonus: EquipmentSpec {
        stats: Stats {
            power: 1,
            toughness: 2,
        },
        keywords: Keywords::REACH,
    },
};
const VEHICLE_AURA: CardRules = CardRules {
    effect: Effect::Aura(TargetPlan::Single(TargetSpec::one(
        TargetRule::CreatureOrVehicle { relation: -1 },
    ))),
    aura: Some(VEHICLE_AURA_SPEC),
    keywords: Keywords::FLASH,
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Ability,
        fragments: &[
            StaticAbilityFragment::AttachmentStats(Stats {
                power: 1,
                toughness: 2,
            }),
            StaticAbilityFragment::AttachmentKeywords(Keywords::REACH),
        ],
    }],
    extras: &CardExtras {
        aura: Some(VEHICLE_AURA_SPEC),
        aura_kind_or_subtype: Some("Vehicle"),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const VEHICLE_ANIMATION: AnimationSpec = AnimationSpec {
    artifact: true,
    creature_types: AnimationCreatureTypes::Unchanged,
    base: None,
    keywords: Keywords::NONE,
    duration: AnimationDuration::EndOfTurn,
};
fn animate(g: &mut Game, id: CardId) {
    g.change_object_form(
        g.object_ref(id),
        VEHICLE_ANIMATION,
        characteristic_layers::AnimationTypeMode::AddCreature,
    );
}
fn goal(actor: Player) -> FundingGoal {
    FundingGoal {
        actor,
        required: [0; 6],
        generic: 0,
        reserve: None,
        source: None,
    }
}

#[test]
fn aura_can_enchant_uncrewed_vehicle_and_its_keyword_survives_animation_cleanup() {
    let _scope = fixture_rules::activate(&[(Card::from_name("Pacifism").unwrap(), &VEHICLE_AURA)]);
    let mut g = scenario(Player::P0);
    let vehicle = put(&mut g, Player::P0, "Veloheart Bike", Zone::Battlefield);
    let aura = put(&mut g, Player::P0, "Pacifism", Zone::Hand);
    assert!(!g.is_creature(vehicle));
    assert!(g.aura_can_enchant(
        Card::from_name("Pacifism").unwrap(),
        Player::P0,
        g.object_ref(vehicle)
    ));
    cast(&mut g, aura, Some(Target::Card(vehicle)));
    settle(&mut g);
    assert_eq!(
        g.state.objects[aura as usize].attached_to,
        Some(g.object_ref(vehicle))
    );
    assert!(g.keywords(vehicle).contains(Keywords::REACH));
    animate(&mut g, vehicle);
    assert_eq!(stats(&g, vehicle), (5, 4));
    assert!(g.expire_animations());
    g.state_based_actions();
    assert!(!g.is_creature(vehicle));
    assert_eq!(g.state.objects[aura as usize].zone, Zone::Battlefield);
    assert!(g.keywords(vehicle).contains(Keywords::REACH));
}

#[test]
fn aura_resolution_rechecks_the_live_union_and_sba_rejects_a_later_ineligible_form() {
    let _scope = fixture_rules::activate(&[(Card::from_name("Pacifism").unwrap(), &VEHICLE_AURA)]);
    let mut g = scenario(Player::P0);
    let vehicle = put(&mut g, Player::P1, "Veloheart Bike", Zone::Battlefield);
    animate(&mut g, vehicle);
    let aura = put(&mut g, Player::P0, "Pacifism", Zone::Hand);
    cast(&mut g, aura, Some(Target::Card(vehicle)));
    g.expire_animations();
    settle(&mut g);
    assert_eq!(g.state.objects[aura as usize].zone, Zone::Battlefield);
    assert!(g.keywords(vehicle).contains(Keywords::REACH));
    let object = g.object_ref(vehicle);
    let timestamp = g.next_role_timestamp();
    g.state
        .roles
        .as_mut()
        .unwrap()
        .animations
        .push(characteristic_layers::food_form(object, timestamp));
    assert!(!g.aura_can_enchant(Card::from_name("Pacifism").unwrap(), Player::P0, object));
    g.state_based_actions();
    assert_eq!(g.state.objects[aura as usize].zone, Zone::Graveyard);
}

#[test]
fn aura_union_rejects_an_ordinary_noncreature_artifact_transactionally() {
    let _scope = fixture_rules::activate(&[(Card::from_name("Pacifism").unwrap(), &VEHICLE_AURA)]);
    let mut g = scenario(Player::P0);
    let vehicle = put(&mut g, Player::P0, "Veloheart Bike", Zone::Battlefield);
    let artifact = put(&mut g, Player::P0, "Swiftfoot Boots", Zone::Battlefield);
    let aura = put(&mut g, Player::P0, "Pacifism", Zone::Hand);
    act(&mut g, Action::AnnounceCast { card: aura });
    reject(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(artifact),
        },
    );
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(vehicle),
        },
    );
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(&mut g, Action::FinishTargets);
    }
    pay(&mut g);
    settle(&mut g);
    assert_eq!(
        g.state.objects[aura as usize].attached_to,
        Some(g.object_ref(vehicle))
    );
}

#[test]
fn projected_aura_entry_accepts_the_same_noncreature_vehicle_as_actual_entry() {
    let _scope = fixture_rules::activate(&[(Card::from_name("Pacifism").unwrap(), &VEHICLE_AURA)]);
    let mut g = scenario(Player::P0);
    let vehicle = put(&mut g, Player::P0, "Veloheart Bike", Zone::Battlefield);
    let aura = put(&mut g, Player::P0, "Pacifism", Zone::Graveyard);
    let saved = g.snapshot();
    let movement = g.entry_move(aura, Zone::Battlefield, false);
    let world = FundingState::from_game(&g, &goal(Player::P0));
    let mut projected = None;
    world.visit_entry_moves(vec![movement.clone()], true, None, &mut |next| {
        if next.object(aura).zone == Zone::Battlefield {
            assert_eq!(next.object(aura).attached_to, Some(next.reference(vehicle)));
            assert!(
                characteristic_query::CharacteristicQuery::in_world(next)
                    .keywords(vehicle)
                    .contains(Keywords::REACH)
            );
            projected = Some(next.object(aura).clone());
        }
        true
    });
    assert!(projected.is_some());
    assert_eq!(g.snapshot(), saved);
    let mut actual = g.clone();
    actual.stage_entries(vec![movement], None);
    assert!(actual.advance_entries());
    choose(
        &mut actual,
        |action| matches!(action, Action::ChooseAuraAttachment { object } if object.card == vehicle),
    );
    assert_eq!(actual.state.objects[aura as usize], projected.unwrap());
    assert!(actual.keywords(vehicle).contains(Keywords::REACH));
}

const OWN_CREATURES: CreatureFilter = CreatureFilter {
    relation: 0,
    other: false,
    subtype: None,
    keyword: Keywords::NONE,
    attacking: false,
    tokens: None,
    artifact: false,
};
const CAPTAIN: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Modify,
        fragments: &[StaticAbilityFragment::CreatureGroupBonus {
            filters: &[
                CreatureFilter {
                    subtype: Some("Mount"),
                    ..OWN_CREATURES
                },
                CreatureFilter {
                    subtype: Some("Vehicle"),
                    ..OWN_CREATURES
                },
            ],
            stats: Stats {
                power: 1,
                toughness: 1,
            },
        }],
    }],
    ..CardRules::EMPTY
};
const TYPE_ENGINE: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Type,
            fragments: &[StaticAbilityFragment::ControlledCreatureSubtype {
                grant: CreatureSubtypeGrant::Chosen,
                required_subtype: Some("Vehicle"),
            }],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Modify,
            fragments: &[StaticAbilityFragment::ChosenModifier(ChosenModifier {
                kind: EntryChoiceKind::CreatureType,
                other: true,
                stats: Stats {
                    power: 1,
                    toughness: 1,
                },
                source_counter: None,
            })],
        },
    ],
    extras: &CardExtras {
        entry_choice: Some(EntryChoiceKind::CreatureType),
        vehicle_stats: Some(Stats {
            power: 4,
            toughness: 4,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const CHOSEN_VEHICLE: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Type,
        fragments: &[StaticAbilityFragment::ChosenCreatureType],
    }],
    extras: &CardExtras {
        entry_choice: Some(EntryChoiceKind::CreatureType),
        vehicle_stats: Some(Stats {
            power: 6,
            toughness: 6,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const OOZE_BASE: LiveStaticEffect = LiveStaticEffect {
    recipients: LiveStatRecipients::Creatures {
        relation: 0,
        other: false,
        enchanted_by_controller: false,
    },
    condition: None,
    power: Some(LiveStatValue::Fixed(6)),
    toughness: Some(LiveStatValue::Fixed(6)),
    bonus_power: 0,
    bonus_toughness: 0,
    keywords: Keywords::NONE,
    cannot_block_subtype: None,
};
const OOZE_WORLD: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Type,
        fragments: &[
            StaticAbilityFragment::ControlledCreatureSubtype {
                grant: CreatureSubtypeGrant::Named("Ooze"),
                required_subtype: None,
            },
            StaticAbilityFragment::LiveBase(0, OOZE_BASE),
        ],
    }],
    extras: &CardExtras {
        live_static_effects: &[OOZE_BASE],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const OFF_TURN: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Type,
        fragments: &[StaticAbilityFragment::SourceOffTurnCreature],
    }],
    extras: &CardExtras {
        vehicle_stats: Some(Stats {
            power: 3,
            toughness: 3,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const BLANK_AURA: CardRules = CardRules {
    effect: Effect::Aura(TargetPlan::NONE),
    aura: Some(AuraSpec {
        kinds: 0,
        relation: -1,
        bonus: EquipmentSpec {
            stats: Stats {
                power: 0,
                toughness: 0,
            },
            keywords: Keywords::NONE,
        },
    }),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::RemoveAttachedAbilities],
    }],
    ..CardRules::EMPTY
};
fn choose_type(g: &mut Game, id: CardId, name: &str) {
    g.set_entry_value(
        g.object_ref(id),
        EntryChoiceValue::CreatureType(CreatureType::from_name(name).unwrap()),
    );
}
fn form(
    g: &mut Game,
    id: CardId,
    types: &'static [&'static str],
    preserve: bool,
    duration: AnimationDuration,
) {
    g.change_object_form(
        g.object_ref(id),
        AnimationSpec {
            artifact: false,
            creature_types: AnimationCreatureTypes::Named(types),
            base: None,
            keywords: Keywords::NONE,
            duration,
        },
        if preserve {
            characteristic_layers::AnimationTypeMode::PreserveCardTypes
        } else {
            characteristic_layers::AnimationTypeMode::AddCreature
        },
    );
}
fn attach_blank(g: &mut Game, id: CardId) -> CardId {
    let aura = put(g, Player::P0, "Pacifism", Zone::Battlefield);
    g.state.objects[aura as usize].attached_to = Some(g.object_ref(id));
    aura
}
fn retain(mut goal: FundingGoal, object: ObjectRef) -> FundingGoal {
    goal.reserve = Some((
        object,
        AdditionalCost {
            kind: EffectChoiceKind::Sacrifice,
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
        },
    ));
    goal
}

#[test]
fn static_creature_group_union_applies_once_and_follows_control_and_source_zone() {
    let _scope = fixture_rules::activate(&[(Card::from_name("Llanowar Elves").unwrap(), &CAPTAIN)]);
    let mut g = scenario(Player::P0);
    let both = put(&mut g, Player::P0, "Veloheart Bike", Zone::Battlefield);
    form(
        &mut g,
        both,
        &["Mount"],
        false,
        AnimationDuration::Permanent,
    );
    let mount = put(&mut g, Player::P0, "Alacrian Jaguar", Zone::Battlefield);
    let enemy = put(&mut g, Player::P1, "Alacrian Jaguar", Zone::Battlefield);
    let captain = put(&mut g, Player::P0, "Llanowar Elves", Zone::Battlefield);
    assert!(g.has_subtype(both, "Mount") && g.has_subtype(both, "Vehicle"));
    assert_eq!(
        stats(&g, both),
        (5, 3),
        "one effect for a member of both groups"
    );
    assert_eq!(stats(&g, mount), (5, 5));
    assert_eq!(stats(&g, enemy), (4, 4));
    assert_eq!(stats(&g, captain), (2, 3));
    let saved = g.snapshot();
    g.change_controller(g.object_ref(captain), Player::P1);
    assert_eq!(stats(&g, both), (4, 2));
    assert_eq!(stats(&g, mount), (4, 4));
    assert_eq!(stats(&g, enemy), (5, 5));
    g.move_object(captain, Zone::Graveyard);
    assert_eq!(stats(&g, enemy), (4, 4));
    g.restore(&saved).unwrap();
    assert_eq!(stats(&g, both), (5, 3));
}

#[test]
fn multiple_chosen_type_providers_accumulate_with_an_independent_self_choice() {
    let _scope = fixture_rules::activate(&[
        (Card::from_name("Veloheart Bike").unwrap(), &TYPE_ENGINE),
        (
            Card::from_name("Cultivator's Caravan").unwrap(),
            &CHOSEN_VEHICLE,
        ),
    ]);
    let mut g = scenario(Player::P0);
    let first = put(&mut g, Player::P0, "Veloheart Bike", Zone::Battlefield);
    let second = put(&mut g, Player::P0, "Veloheart Bike", Zone::Battlefield);
    choose_type(&mut g, first, "Alien");
    choose_type(&mut g, second, "Time Lord");
    let vehicle = put(
        &mut g,
        Player::P0,
        "Cultivator's Caravan",
        Zone::Battlefield,
    );
    choose_type(&mut g, vehicle, "Capybara");
    animate(&mut g, vehicle);
    let creature = put(&mut g, Player::P0, "Savannah Lions", Zone::Battlefield);
    form(
        &mut g,
        creature,
        &["Alien"],
        true,
        AnimationDuration::Permanent,
    );
    for name in ["Alien", "Time Lord", "Capybara", "Vehicle"] {
        assert!(g.has_subtype(vehicle, name), "missing {name}");
    }
    assert!(
        !g.has_subtype(creature, "Time Lord"),
        "nonvehicle is not in the type-grant group"
    );
    assert_eq!(stats(&g, vehicle), (8, 8));
    assert_eq!(
        stats(&g, creature),
        (3, 2),
        "natural chosen-type creatures also get the bonus"
    );
    animate(&mut g, first);
    assert_eq!(
        stats(&g, first),
        (5, 5),
        "its own independent lord effect excludes itself"
    );
    let frozen = g.object_view(vehicle);
    let facts = DamageFacts::from_view(&frozen);
    g.change_controller(g.object_ref(second), Player::P1);
    assert!(g.has_subtype(vehicle, "Alien") && !g.has_subtype(vehicle, "Time Lord"));
    assert_eq!(stats(&g, vehicle), (7, 7));
    g.move_object(first, Zone::Graveyard);
    assert!(!g.has_subtype(vehicle, "Alien"));
    assert!(g.has_subtype(vehicle, "Capybara"));
    assert_eq!(stats(&g, vehicle), (6, 6));
    drop(g);
    for name in ["Alien", "Time Lord", "Capybara"] {
        assert!(
            frozen.has_subtype(name) && facts.has_subtype(name),
            "frozen views retain {name}"
        );
    }
    let wire = serde_json::to_value(frozen.granted_subtypes).unwrap();
    assert_eq!(
        wire["additional_creature_types"],
        serde_json::json!(["Alien", "Time Lord"])
    );
    assert_eq!(wire["chosen"], "Capybara");
    assert!(
        serde_json::to_value(GrantedSubtypes::default())
            .unwrap()
            .get("additional_creature_types")
            .is_none()
    );
}

#[test]
fn subtype_grant_depends_on_new_animation_but_a_later_subtype_setter_replaces_it() {
    let _scope =
        fixture_rules::activate(&[(Card::from_name("Veloheart Bike").unwrap(), &TYPE_ENGINE)]);
    let mut g = scenario(Player::P0);
    let provider = put(&mut g, Player::P0, "Veloheart Bike", Zone::Battlefield);
    choose_type(&mut g, provider, "Alien");
    let vehicle = put(
        &mut g,
        Player::P0,
        "Cultivator's Caravan",
        Zone::Battlefield,
    );
    form(
        &mut g,
        vehicle,
        &["Bird"],
        false,
        AnimationDuration::Permanent,
    );
    assert!(
        g.has_subtype(vehicle, "Bird") && g.has_subtype(vehicle, "Alien"),
        "creature-only grant follows the new animation it depends on"
    );
    form(
        &mut g,
        vehicle,
        &["Cat"],
        true,
        AnimationDuration::EndOfTurn,
    );
    assert!(g.has_subtype(vehicle, "Cat") && !g.has_subtype(vehicle, "Bird"));
    assert!(
        !g.has_subtype(vehicle, "Alien"),
        "later subtype-only setter overwrites the old grant"
    );
    let later = put(&mut g, Player::P0, "Veloheart Bike", Zone::Battlefield);
    choose_type(&mut g, later, "Elf");
    assert!(g.has_subtype(vehicle, "Elf") && !g.has_subtype(vehicle, "Alien"));
    g.expire_animations();
    assert!(
        g.has_subtype(vehicle, "Bird")
            && g.has_subtype(vehicle, "Alien")
            && g.has_subtype(vehicle, "Elf")
    );
}

#[test]
fn static_named_subtype_and_base_stats_are_live_and_survive_later_ability_removal() {
    let _scope = fixture_rules::activate(&[
        (Card::from_name("Swiftfoot Boots").unwrap(), &OOZE_WORLD),
        (Card::from_name("Pacifism").unwrap(), &BLANK_AURA),
    ]);
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P0, "Swiftfoot Boots", Zone::Battlefield);
    let creature = put(&mut g, Player::P0, "Savannah Lions", Zone::Battlefield);
    let enemy = put(&mut g, Player::P1, "Savannah Lions", Zone::Battlefield);
    assert!(g.has_subtype(creature, "Ooze") && g.has_subtype(creature, "Cat"));
    assert!(!g.has_subtype(enemy, "Ooze"));
    assert_eq!(stats(&g, creature), (6, 6));
    attach_blank(&mut g, source);
    assert!(!CharacteristicWorld::printed_abilities_available(
        &g, source
    ));
    assert!(g.has_subtype(creature, "Ooze"));
    assert_eq!(
        stats(&g, creature),
        (6, 6),
        "one ability started in layer four before removal in layer six"
    );
    g.change_controller(g.object_ref(creature), Player::P1);
    assert!(!g.has_subtype(creature, "Ooze"));
    assert_eq!(stats(&g, creature), (2, 1));
    g.change_controller(g.object_ref(enemy), Player::P0);
    assert!(g.has_subtype(enemy, "Ooze"));
    assert_eq!(stats(&g, enemy), (6, 6));
    g.move_object(source, Zone::Graveyard);
    assert!(!g.has_subtype(enemy, "Ooze"));
    assert_eq!(stats(&g, enemy), (2, 1));
}

#[test]
fn off_turn_creature_addition_uses_current_controller_and_keeps_vehicle_stats() {
    let _scope = fixture_rules::activate(&[
        (Card::from_name("Veloheart Bike").unwrap(), &OFF_TURN),
        (Card::from_name("Pacifism").unwrap(), &BLANK_AURA),
    ]);
    let mut g = scenario(Player::P0);
    let vehicle = put(&mut g, Player::P0, "Veloheart Bike", Zone::Battlefield);
    assert!(!g.is_creature(vehicle));
    g.state.active = Player::P1;
    assert!(g.is_creature(vehicle));
    assert_eq!(stats(&g, vehicle), (3, 3));
    attach_blank(&mut g, vehicle);
    assert!(!CharacteristicWorld::printed_abilities_available(
        &g, vehicle
    ));
    assert!(
        g.is_creature(vehicle),
        "earlier type effect survives later removal"
    );
    let view = g.object_view(vehicle);
    let facts = DamageFacts::from_view(&view);
    g.change_controller(g.object_ref(vehicle), Player::P1);
    assert!(!g.is_creature(vehicle));
    g.state.active = Player::P0;
    assert!(g.is_creature(vehicle));
    assert_eq!(stats(&g, vehicle), (3, 3));
    let event = g.move_object(vehicle, Zone::Graveyard);
    assert_eq!(event.before.kinds & 1, 1);
    assert_eq!(event.before.power, 3);
    assert!(!g.is_creature(vehicle));
    assert_eq!(view.power, 3);
    assert_eq!(facts.kinds & 1, 1);
}

#[test]
fn funding_worlds_retain_static_type_providers_and_query_projected_control_and_departure() {
    let _scope = fixture_rules::activate(&[
        (Card::from_name("Veloheart Bike").unwrap(), &TYPE_ENGINE),
        (
            Card::from_name("Cultivator's Caravan").unwrap(),
            &CHOSEN_VEHICLE,
        ),
        (Card::from_name("Llanowar Elves").unwrap(), &CAPTAIN),
    ]);
    let mut g = scenario(Player::P0);
    let provider = put(&mut g, Player::P0, "Veloheart Bike", Zone::Battlefield);
    choose_type(&mut g, provider, "Time Lord");
    let captain = put(&mut g, Player::P0, "Llanowar Elves", Zone::Battlefield);
    let vehicle = put(
        &mut g,
        Player::P0,
        "Cultivator's Caravan",
        Zone::Battlefield,
    );
    choose_type(&mut g, vehicle, "Alien");
    animate(&mut g, vehicle);
    let saved = g.snapshot();
    let goal = retain(goal(Player::P0), g.object_ref(vehicle));
    let world = FundingState::from_game(&g, &goal);
    assert!(world.resources.iter().any(|r| r.id == provider));
    assert!(world.resources.iter().any(|r| r.id == captain));
    assert!(
        world
            .subtypes(vehicle)
            .has(g.state.objects[vehicle as usize].card, "Time Lord")
    );
    assert_eq!(
        characteristic_query::CharacteristicQuery::in_world(&world)
            .stats(vehicle)
            .power,
        8
    );
    let mut branch = world.clone();
    branch
        .resources
        .iter_mut()
        .find(|r| r.id == provider)
        .unwrap()
        .object
        .controller = Player::P1;
    branch
        .resources
        .iter_mut()
        .find(|r| r.id == captain)
        .unwrap()
        .object
        .zone = Zone::Graveyard;
    assert!(
        !branch
            .subtypes(vehicle)
            .has(g.state.objects[vehicle as usize].card, "Time Lord")
    );
    assert_eq!(
        characteristic_query::CharacteristicQuery::in_world(&branch)
            .stats(vehicle)
            .power,
        6
    );
    assert_eq!(stats(&g, vehicle), (8, 8));
    assert_eq!(g.snapshot(), saved);
}

#[test]
fn chosen_type_entry_is_owned_across_raw_indexed_restore_and_parent_disposal() {
    const RETURN: CardRules = CardRules {
        effect: Effect::Spell(
            TargetPlan::Single(TargetSpec::one(TargetRule::Object {
                zone: TargetZone::Graveyard,
                kinds: 16,
                relation: 0,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            })),
            &[Instruction::ReturnToBattlefieldUnderControl],
        ),
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[
        (Card::from_name("Veloheart Bike").unwrap(), &TYPE_ENGINE),
        (Card::from_name("Opt").unwrap(), &RETURN),
    ]);
    let mut parent = scenario(Player::P0);
    let provider = put(&mut parent, Player::P0, "Veloheart Bike", Zone::Graveyard);
    let vehicle = put(
        &mut parent,
        Player::P0,
        "Cultivator's Caravan",
        Zone::Battlefield,
    );
    animate(&mut parent, vehicle);
    let spell = put(&mut parent, Player::P0, "Opt", Zone::Hand);
    cast(&mut parent, spell, Some(Target::Card(provider)));
    for _ in 0..12 {
        if parent.state.pending != Pending::Priority {
            break;
        }
        act(&mut parent, Action::Pass);
    }
    assert!(matches!(parent.state.pending, Pending::EntryValue { .. }));
    let saved = parent.snapshot();
    let action = Action::ChooseEntryValue {
        value: EntryChoiceValue::CreatureType(CreatureType::from_name("Time Lord").unwrap()),
    };
    let mut raw = parent.clone();
    let mut indexed = parent.clone();
    let index = indexed
        .legal_actions()
        .iter()
        .position(|candidate| *candidate == action)
        .unwrap();
    let actor = indexed.actor().unwrap();
    let revision = indexed.revision();
    act(&mut raw, action);
    indexed.apply_index(actor, revision, index).unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    drop(parent);
    settle(&mut raw);
    assert!(raw.has_subtype(vehicle, "Time Lord"));
    indexed.restore(&saved).unwrap();
    act(
        &mut indexed,
        Action::ChooseEntryValue {
            value: EntryChoiceValue::CreatureType(CreatureType::from_name("Capybara").unwrap()),
        },
    );
    settle(&mut indexed);
    assert!(indexed.has_subtype(vehicle, "Capybara") && !indexed.has_subtype(vehicle, "Time Lord"));
    assert!(raw.has_subtype(vehicle, "Time Lord") && !raw.has_subtype(vehicle, "Capybara"));
}

#[test]
fn type_overlay_layout_is_reported_without_changing_hot_object_storage() {
    eprintln!(
        "static-type layout: Object={} ObjectView={} DamageFacts={} GrantedSubtypes={} taxonomy_words={}",
        std::mem::size_of::<Object>(),
        std::mem::size_of::<ObjectView>(),
        std::mem::size_of::<DamageFacts>(),
        std::mem::size_of::<GrantedSubtypes>(),
        CREATURE_TYPES.len().div_ceil(64)
    );
    assert_eq!(CREATURE_TYPES.len().div_ceil(64), 6);
}

#[test]
fn type_grant_continues_after_removal_but_its_separate_lord_ability_stops() {
    let _scope = fixture_rules::activate(&[
        (Card::from_name("Veloheart Bike").unwrap(), &TYPE_ENGINE),
        (Card::from_name("Pacifism").unwrap(), &BLANK_AURA),
    ]);
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P0, "Veloheart Bike", Zone::Battlefield);
    choose_type(&mut g, source, "Capybara");
    let vehicle = put(
        &mut g,
        Player::P0,
        "Cultivator's Caravan",
        Zone::Battlefield,
    );
    animate(&mut g, vehicle);
    assert!(g.has_subtype(vehicle, "Capybara"));
    assert_eq!(stats(&g, vehicle), (6, 6));
    attach_blank(&mut g, source);
    assert!(!CharacteristicWorld::printed_abilities_available(
        &g, source
    ));
    assert!(
        g.has_subtype(vehicle, "Capybara"),
        "the grant already started in layer four"
    );
    assert_eq!(
        stats(&g, vehicle),
        (5, 5),
        "the independent layer-seven lord never starts"
    );
}
