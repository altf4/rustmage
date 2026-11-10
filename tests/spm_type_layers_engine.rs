//! Exact permanent type effects and a controller-owned activation granted to a land.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const ARTIFACT_CREATURE: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    keywords: Keywords::HASTE,
    ..CardRules::EMPTY
};
const LEGENDARY: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[
            Instruction::PutCounters {
                subject: Subject::Targets,
                kind: CounterKind::PlusOne,
                amount: 1,
            },
            Instruction::AddLegendaryCreatureTypes {
                subject: Subject::Targets,
                types: &["Spider", "Hero"],
            },
        ],
    ),
    ..CardRules::EMPTY
};

const LIZARD_FORM: CardRules = CardRules {
    effect: Effect::Creature(5, 5),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Single(TargetSpec {
            rule: TargetRule::Filtered {
                base: &TargetRule::Creature,
                predicate: &TargetPredicate::OtherThanSource,
            },
            min: 0,
            max: 1,
        }),
        program: &[
            Instruction::RemoveAbilities {
                subject: Subject::Targets,
                duration: AnimationDuration::Permanent,
            },
            Instruction::ReplaceTargetCreatureForm(AnimationSpec {
                artifact: false,
                creature_types: AnimationCreatureTypes::Named(&["Lizard"]),
                base: Some((4, 4)),
                keywords: Keywords::NONE,
                duration: AnimationDuration::Permanent,
            }),
            Instruction::SetColors {
                subject: Subject::Targets,
                colors: ColorValue::Fixed(16),
                duration: AnimationDuration::Permanent,
            },
        ],
    }],
    ..CardRules::EMPTY
};

const LAND_GRANT: CardRules = CardRules {
    effect: Effect::Aura(TargetPlan::Single(TargetSpec::one(TargetRule::Object {
        zone: TargetZone::Battlefield,
        kinds: 2,
        relation: -1,
        other: false,
        nonland: false,
        tapped: false,
        min_mana: 0,
        max_mana: u32::MAX,
    }))),
    aura: Some(AuraSpec {
        kinds: 2,
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
        fragments: &[StaticAbilityFragment::AttachmentActivation {
            ability: 0,
            tap_attachment: false,
        }],
    }],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            generic: 1,
            mana_component: true,
            tap: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        program: &[Instruction::GrowScaled {
            subject: Subject::Targets,
            amount: AmountValue::Stat(StatValue::Count(CountFilter {
                zone: CountZone::Battlefield,
                relation: 0,
                kind: 1,
                exclude: 0,
                subtype: None,
                basic: false,
                other: false,
                cards_only: false,
            })),
            power: 1,
            toughness: 1,
        }],
        sorcery: true,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};

fn definition(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

#[test]
fn legendary_and_added_creature_types_preserve_other_types_and_are_not_copied() {
    let gingerbrute = definition("Gingerbrute");
    let _rules = fixture_rules::activate(&[
        (definition("Opt"), &LEGENDARY),
        (gingerbrute, &ARTIFACT_CREATURE),
    ]);
    let mut game = scenario(Player::P0);
    let recipient = spawn(&mut game, Player::P0, gingerbrute, Zone::Battlefield);
    let spell = put(&mut game, Player::P0, "Opt", Zone::Hand);
    cast(&mut game, spell, Some(Target::Card(recipient)));
    settle(&mut game);
    assert_eq!(game.object_view(spell).zone, Zone::Graveyard);
    assert_eq!(game.counter_count(recipient, CounterKind::PlusOne), 1);
    let result = game.object_view(recipient);
    assert!(result.is_legendary());
    assert_eq!(result.kinds, 17);
    for subtype in ["Food", "Golem", "Spider", "Hero"] {
        assert!(result.has_subtype(subtype), "missing {subtype}");
    }
    game.expire_animations();
    assert!(game.object_view(recipient).is_legendary());

    let copy = put(&mut game, Player::P0, "Waxen Shapethief", Zone::Hand);
    cast(&mut game, copy, None);
    pass_pair(&mut game);
    let object = Some(game.object_ref(recipient));
    act(&mut game, Action::ChooseEntryCopy { object });
    settle(&mut game);
    let copy = game.object_view(copy);
    assert!(!copy.is_legendary());
    assert!(copy.has_subtype("Food"));
    assert!(!copy.has_subtype("Hero"));

    game.move_object(recipient, Zone::Exile);
    game.move_object(recipient, Zone::Battlefield);
    assert!(!game.object_view(recipient).is_legendary());
    assert!(!game.object_view(recipient).has_subtype("Spider"));
}

#[test]
fn setting_creature_form_removes_other_card_types_but_preserves_the_permanents_identity() {
    let gingerbrute = definition("Gingerbrute");
    let _rules = fixture_rules::activate(&[
        (definition("Savannah Lions"), &LIZARD_FORM),
        (definition("Opt"), &LEGENDARY),
        (gingerbrute, &ARTIFACT_CREATURE),
    ]);
    let mut game = scenario(Player::P0);
    let target = spawn(&mut game, Player::P1, gingerbrute, Zone::Battlefield);
    let legend = put(&mut game, Player::P0, "Opt", Zone::Hand);
    cast(&mut game, legend, Some(Target::Card(target)));
    settle(&mut game);
    let source = put(&mut game, Player::P0, "Savannah Lions", Zone::Hand);
    cast(&mut game, source, None);
    pass_pair(&mut game);
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(source),
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

    let result = game.object_view(target);
    assert_eq!(result.name, "Gingerbrute");
    assert_eq!(result.kinds, 1);
    assert_eq!(result.colors, 16);
    assert!(result.is_legendary());
    assert!(result.has_subtype("Lizard"));
    assert!(!result.has_subtype("Food"));
    assert!(!game.has_subtype(target, "Food"));
    assert!(result.card.has_subtype("Food"));
    assert!(!result.has_subtype("Golem"));
    assert!(!result.has_subtype("Spider"));
    assert!(!result.keywords.contains(Keywords::HASTE));
    assert!(!result.printed_abilities_available());
    assert_eq!(stats(&game, target), (5, 5));
    game.move_object(source, Zone::Graveyard);
    game.expire_animations();
    assert_eq!(game.object_view(target).kinds, 1);
    assert_eq!(stats(&game, target), (5, 5));

    let lizard_state = game.snapshot();
    game.change_object_form(
        result.reference,
        AnimationSpec {
            artifact: true,
            creature_types: AnimationCreatureTypes::Unchanged,
            base: None,
            keywords: Keywords::NONE,
            duration: AnimationDuration::EndOfTurn,
        },
        characteristic_layers::AnimationTypeMode::PreserveCardTypes,
    );
    let artifact = game.object_view(target);
    assert_eq!(artifact.reference, result.reference);
    assert_eq!(artifact.kinds, 17);
    assert!(artifact.has_subtype("Food"));
    assert!(game.has_subtype(target, "Food"));
    assert!(artifact.has_subtype("Lizard"));
    assert!(!artifact.has_subtype("Golem"));
    assert!(!result.has_subtype("Food"));

    assert!(game.expire_animations());
    assert!(!game.object_view(target).has_subtype("Food"));
    game.restore(&lizard_state).unwrap();
    assert_eq!(game.object_view(target), result);

    game.move_object(target, Zone::Exile);
    game.move_object(target, Zone::Battlefield);
    let original = game.object_view(target);
    assert_eq!(original.kinds, 17);
    assert!(original.keywords.contains(Keywords::HASTE));
    assert!(original.has_subtype("Food"));
    assert!(original.has_subtype("Golem"));
    assert!(!original.is_legendary());
}

#[test]
fn an_enchanted_land_grants_its_controller_a_sorcery_speed_activation() {
    let _rules = fixture_rules::activate(&[(definition("Ringing Strike Mastery"), &LAND_GRANT)]);
    let mut game = scenario(Player::P0);
    let land = put(&mut game, Player::P1, "Forest", Zone::Battlefield);
    let ally = put(&mut game, Player::P1, "Bear Cub", Zone::Battlefield);
    let opposing = put(&mut game, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let aura = put(&mut game, Player::P0, "Ringing Strike Mastery", Zone::Hand);
    cast(&mut game, aura, Some(Target::Card(land)));
    settle(&mut game);
    assert_eq!(
        game.object_view(aura).attached_to,
        Some(game.object_ref(land))
    );
    let action = Action::AnnounceAttachedAbility {
        grantor: game.object_ref(aura),
        ability: 0,
    };
    reject(&mut game, action.clone());

    act(&mut game, Action::Pass);
    reject(&mut game, action.clone());
    game.state.active = Player::P1;
    game.state.priority = Player::P1;
    game.state.passes = 0;
    game.state.players[1].mana = [0; 6];
    reject(&mut game, action.clone());
    game.state.players[1].mana[5] = 1;

    let before = game.snapshot();
    let mut indexed = game.clone();
    let index = indexed
        .legal_actions()
        .iter()
        .position(|a| *a == action)
        .unwrap();
    indexed
        .apply_index(Player::P1, indexed.revision(), index)
        .unwrap();
    act(&mut game, action.clone());
    assert_eq!(game.snapshot(), indexed.snapshot());
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(opposing),
        },
    );
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(&mut game, Action::FinishTargets);
    }
    pay(&mut game);
    assert!(game.object_view(land).tapped);
    assert!(!game.object_view(aura).tapped);
    game.move_object(aura, Zone::Graveyard);
    put(&mut game, Player::P1, "Savannah Lions", Zone::Battlefield);
    settle(&mut game);
    assert_eq!(stats(&game, opposing), (4, 4));
    assert_eq!(stats(&game, ally), (2, 2));

    game.restore(&before).unwrap();
    game.move_object(aura, Zone::Graveyard);
    reject(&mut game, action);
}

const SUBTYPE_EQUIPMENT: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Type,
        fragments: &[StaticAbilityFragment::AttachedCharacteristics(
            &AttachedCharacteristics {
                name: None,
                form: PermanentForm::AddCreatureSubtypes,
                creature_types: &["Spider", "Hero"],
                colors: &[],
                base: None,
            },
        )],
    }],
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            generic: 1,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::creature_you_control())),
        program: &[Instruction::AttachSource],
        exhaust: false,
        guard: ActivationGuard::Always,
        cycling: false,
        origin: ActivationOrigin::Battlefield,
        sorcery: true,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};

#[test]
fn subtype_only_equipment_keeps_vehicle_type_and_detaches_when_crew_expires() {
    let equipment = definition("Short Bow");
    let _rules = fixture_rules::activate(&[(equipment, &SUBTYPE_EQUIPMENT)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let vehicle = put(&mut game, player, "Voyager Glidecar", Zone::Battlefield);
    let crew = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let equipment = spawn(&mut game, player, equipment, Zone::Battlefield);

    act(
        &mut game,
        Action::AnnounceActivate {
            card: vehicle,
            ability: 1,
            counter_payment: Counters::default(),
        },
    );
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(&mut game);
    }
    let object = game.object_ref(crew);
    act(&mut game, Action::ChooseCrewer { object });
    act(&mut game, Action::FinishCrew);
    settle(&mut game);

    act(
        &mut game,
        Action::AnnounceActivate {
            card: equipment,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(vehicle),
        },
    );
    pay(&mut game);
    settle(&mut game);
    let equipped = game.object_view(vehicle);
    assert_eq!(equipped.kinds, 17);
    for subtype in ["Vehicle", "Spider", "Hero"] {
        assert!(equipped.has_subtype(subtype));
    }
    assert_eq!(
        game.object_view(equipment).attached_to,
        Some(equipped.reference)
    );

    let mut animated = game.clone();
    animated.change_object_form(
        equipped.reference,
        AnimationSpec {
            artifact: true,
            creature_types: AnimationCreatureTypes::Named(&["Construct"]),
            base: None,
            keywords: Keywords::NONE,
            duration: AnimationDuration::Permanent,
        },
        characteristic_layers::AnimationTypeMode::AddCreature,
    );
    animated.begin_step(Step::End);
    animated.give_priority();
    pass_pair(&mut animated);
    let overlap = animated.object_view(vehicle);
    assert_eq!(overlap.kinds, 17);
    for subtype in ["Vehicle", "Construct", "Spider", "Hero"] {
        assert!(overlap.has_subtype(subtype), "missing {subtype}");
    }
    assert_eq!(
        animated.object_view(equipment).attached_to,
        Some(overlap.reference)
    );

    game.begin_step(Step::End);
    game.give_priority();
    pass_pair(&mut game);
    let after = game.object_view(vehicle);
    assert_eq!(after.reference, equipped.reference);
    assert_eq!(after.kinds, 16);
    assert!(after.has_subtype("Vehicle"));
    assert!(!after.has_subtype("Spider"));
    assert_eq!(game.object_view(equipment).attached_to, None);
}
