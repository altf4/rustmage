//! Projected colors, dynamic production and mana lots use the same owned rules world.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;
const BASE: ActivatedAbility = ActivatedAbility {
    exhaust: false,
    guard: ActivationGuard::Always,
    origin: ActivationOrigin::Battlefield,
    cycling: false,
    cost: ActivationCost::ZERO,
    targets: TargetPlan::NONE,
    program: &[],
    sorcery: false,
    once_per_turn: false,
};
const BLOOM: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            tap: true,
            ..ActivationCost::ZERO
        },
        program: &[Instruction::AddMana(
            ManaProduction::ColorsAmongControlledPermanents,
        )],
        ..BASE
    }],
    ..CardRules::EMPTY
};
const WICKER: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            generic: 1,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        once_per_turn: true,
        program: &[Instruction::AddMana(
            ManaProduction::AnyColorChangingSource(1),
        )],
        ..BASE
    }],
    ..CardRules::EMPTY
};
const DRAW: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::Draw(PlayerSet::Controller, 1)],
    ),
    ..CardRules::EMPTY
};
const AURA: CardRules = CardRules {
    effect: Effect::Permanent,
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
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Color,
            fragments: &[StaticAbilityFragment::AttachedChosenColor],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::AttachmentManaBonus(
                ManaProduction::ChosenColor,
            )],
        },
    ],
    ..CardRules::EMPTY
};
const OTHERS: StatValue = StatValue::Count(CountFilter {
    zone: CountZone::Battlefield,
    relation: 0,
    kind: 1,
    exclude: 0,
    subtype: None,
    basic: false,
    other: true,
    cards_only: false,
});
const FILTER: SacrificeFilter = SacrificeFilter {
    keywords: Keywords::NONE,
    source_only: false,
    kinds: 1,
    nonland: false,
    other: true,
    tokens: None,
    artifact_enchantment_or_token: false,
    subtype: None,
};
fn direct(card: CardId, payment: Mana, production: Mana) -> Action {
    Action::ActivateMana {
        card,
        ability: 0,
        payment,
        production,
        counter_payment: Counters::default(),
    }
}
#[test]
fn source_color_change_is_projected_before_a_later_dynamic_producer_and_owned_branches() {
    let _rules = fixture_rules::activate(&[
        (Card::from_name("Short Bow").unwrap(), &WICKER),
        (Card::GrizzlyBears, &BLOOM),
        (Card::from_name("Cancel").unwrap(), &DRAW),
    ]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let wicker = put(&mut g, p, "Short Bow", Zone::Battlefield);
        let bloom = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let spell = put(&mut g, p, "Cancel", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        let saved = g.snapshot();
        // Only a blue recoloring can produce the second blue for this 1UU spell.
        act(&mut g, Action::AnnounceCast { card: spell });
        let action = direct(wicker, [0, 0, 0, 0, 0, 1], [0, 1, 0, 0, 0, 0]);
        let mut indexed = g.clone();
        let ix = indexed
            .legal_actions()
            .iter()
            .position(|a| *a == action)
            .unwrap();
        indexed.apply_index(p, indexed.revision(), ix).unwrap();
        reject(
            &mut g,
            direct(wicker, [0, 0, 0, 0, 0, 1], [0, 0, 0, 1, 0, 0]),
        );
        act(&mut g, action);
        assert_eq!(g.snapshot(), indexed.snapshot());
        assert_eq!(g.object_view(wicker).colors, 2);
        act(&mut g, direct(bloom, [0; 6], [0, 1, 0, 0, 1, 0]));
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.hand_count(p), 1);
        indexed.restore(&saved).unwrap();
        assert_eq!(indexed.object_view(wicker).colors, 0);
        assert_eq!(indexed.state.players[p.index()].mana, [0, 0, 0, 0, 0, 1]);
    }
}
#[test]
fn multicolored_permanents_each_supply_all_their_colors_and_old_incarnations_expire() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &BLOOM)]);
    let mut g = scenario(Player::P0);
    let bloom = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let bear = put(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    g.set_color_effects(&[bear], 1 | 2 | 8, AnimationDuration::Permanent);
    g.state.players[0].mana = [0; 6];
    act(&mut g, direct(bloom, [0; 6], [1, 1, 0, 1, 1, 0]));
    assert_eq!(g.state.players[0].mana, [1, 1, 0, 1, 1, 0]);
    let original = g.object_ref(bear);
    g.move_object(bear, Zone::Exile);
    g.move_object(bear, Zone::Battlefield);
    assert_ne!(g.object_ref(bear), original);
    assert_eq!(g.object_view(bear).colors, 16);
}
#[test]
fn creature_count_mana_samples_after_sacrifice_cost_and_never_splits() {
    const COUNT: CardRules = CardRules {
        effect: Effect::Creature(2, 2),
        activated: &[ActivatedAbility {
            cost: ActivationCost {
                tap: true,
                additional: Some(AdditionalCost {
                    kind: EffectChoiceKind::Sacrifice,
                    filter: FILTER,
                }),
                ..ActivationCost::ZERO
            },
            program: &[Instruction::AddMana(ManaProduction::ChoiceStat {
                colors: 17,
                amount: &OTHERS,
            })],
            ..BASE
        }],
        ..CardRules::EMPTY
    };
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &COUNT)]);
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let paid = put(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    put(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    put(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
    g.state.players[0].mana = [0; 6];
    act(
        &mut g,
        Action::AnnounceMana {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
            production: [0; 6],
        },
    );
    let object = g.object_ref(paid);
    act(&mut g, Action::ChooseAdditionalCost { object });
    if matches!(g.state.pending, Pending::PayCosts { .. }) {
        pay(&mut g);
    }
    assert!(matches!(g.state.pending, Pending::EffectMana { .. }));
    reject(
        &mut g,
        Action::ChooseMana {
            production: [1, 0, 0, 0, 1, 0],
        },
    );
    act(
        &mut g,
        Action::ChooseMana {
            production: [1, 0, 0, 0, 0, 0],
        },
    );
    assert_eq!(g.state.players[0].mana, [1, 0, 0, 0, 0, 0]);
    assert_eq!(g.state.objects[paid as usize].zone, Zone::Graveyard);
}
#[test]
fn chosen_color_aura_and_basic_land_mana_bonus_are_unrestricted_and_apply_to_opponents() {
    const LAVA: CardRules = CardRules {
        effect: Effect::Permanent,
        static_groups: &[StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::BasicLandTapManaBonus(1)],
        }],
        ..CardRules::EMPTY
    };
    let _rules = fixture_rules::activate(&[
        (Card::from_name("Short Bow").unwrap(), &LAVA),
        (Card::from_name("Pacifism").unwrap(), &AURA),
    ]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p.other(), "Short Bow", Zone::Battlefield);
        let land = put(&mut g, p, "Forest", Zone::Battlefield);
        let aura = put(&mut g, p.other(), "Pacifism", Zone::Battlefield);
        g.state.objects[aura as usize].attached_to = Some(g.object_ref(land));
        g.set_entry_value(
            g.object_ref(aura),
            EntryChoiceValue::Color(ChoiceColor::Blue),
        );
        assert_eq!(g.object_view(land).colors, 2);
        g.state.players[p.index()].mana = [0; 6];
        act(&mut g, Action::TapMana { card: land });
        assert_eq!(g.state.players[p.index()].mana, [0, 1, 0, 0, 2, 0]);
        assert!(g.observe(p).mana_lots.is_empty());
        g.move_object(aura, Zone::Graveyard);
        assert_eq!(g.object_view(land).colors, 0);
    }
}
#[test]
fn multitype_land_bonus_choice_preserves_parent_payment_and_captured_departed_source() {
    const LAND: CardRules = CardRules {
        effect: Effect::Permanent,
        activated: &[ActivatedAbility {
            cost: ActivationCost {
                tap: true,
                sacrifice: true,
                ..ActivationCost::ZERO
            },
            program: &[Instruction::AddMana(ManaProduction::Fixed([
                0, 1, 0, 0, 1, 0,
            ]))],
            ..BASE
        }],
        ..CardRules::EMPTY
    };
    const LAVA: CardRules = CardRules {
        effect: Effect::Permanent,
        static_groups: &[StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::BasicLandTapManaBonus(1)],
        }],
        ..CardRules::EMPTY
    };
    let _rules = fixture_rules::activate(&[
        (Card::Forest, &LAND),
        (Card::from_name("Pacifism").unwrap(), &AURA),
        (Card::from_name("Short Bow").unwrap(), &LAVA),
        (Card::from_name("Cancel").unwrap(), &DRAW),
    ]);
    let mut g = scenario(Player::P0);
    let land = put(&mut g, Player::P0, "Forest", Zone::Battlefield);
    let aura = put(&mut g, Player::P0, "Pacifism", Zone::Battlefield);
    g.state.objects[aura as usize].attached_to = Some(g.object_ref(land));
    g.set_entry_value(
        g.object_ref(aura),
        EntryChoiceValue::Color(ChoiceColor::Blue),
    );
    let returning = put(&mut g, Player::P1, "Short Bow", Zone::Exile);
    g.state.exile_links.push(ExileLink {
        ability_origin: None,
        source: g.object_ref(land),
        exiled: g.object_ref(returning),
        return_zone: Zone::Battlefield,
    });
    let spell = put(&mut g, Player::P0, "Cancel", Zone::Hand);
    g.state.players[0].mana = [0; 6];
    act(&mut g, Action::AnnounceCast { card: spell });
    act(&mut g, direct(land, [0; 6], [0, 1, 0, 0, 1, 0]));
    assert_eq!(g.state.objects[land as usize].zone, Zone::Graveyard);
    assert!(g.on_battlefield(returning));
    // The aura no longer has an enchanted land at the mana event.
    assert_eq!(g.state.players[0].mana, [0, 1, 0, 0, 1, 0]);
    assert!(matches!(g.state.pending, Pending::EffectMana { .. }));
    reject(
        &mut g,
        Action::ChooseMana {
            production: [0, 0, 0, 0, 1, 0],
        },
    );
    let mut other = g.clone();
    let snapshot = g.snapshot();
    act(
        &mut g,
        Action::ChooseMana {
            production: [0, 1, 0, 0, 0, 0],
        },
    );
    pay(&mut g);
    settle(&mut g);
    assert_eq!(g.hand_count(Player::P0), 1);
    other.restore(&snapshot).unwrap();
    act(
        &mut other,
        Action::ChooseMana {
            production: [0, 1, 0, 0, 0, 0],
        },
    );
    pay(&mut other);
    settle(&mut other);
    assert_eq!(g.snapshot(), other.snapshot());
}
#[test]
fn paying_for_mana_can_remove_the_only_global_tap_bonus_provider() {
    const LAND: CardRules = CardRules {
        effect: Effect::Permanent,
        activated: &[ActivatedAbility {
            cost: ActivationCost {
                tap: true,
                additional: Some(AdditionalCost {
                    kind: EffectChoiceKind::Sacrifice,
                    filter: FILTER,
                }),
                ..ActivationCost::ZERO
            },
            program: &[Instruction::AddMana(ManaProduction::Fixed([
                0, 0, 0, 0, 1, 0,
            ]))],
            ..BASE
        }],
        ..CardRules::EMPTY
    };
    const LAVA: CardRules = CardRules {
        effect: Effect::Creature(2, 2),
        static_groups: &[StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::BasicLandTapManaBonus(1)],
        }],
        ..CardRules::EMPTY
    };
    let _rules = fixture_rules::activate(&[
        (Card::Forest, &LAND),
        (Card::from_name("Elemental Mascot").unwrap(), &LAVA),
    ]);
    let p = Player::P0;
    let mut g = scenario(p);
    let land = put(&mut g, p, "Forest", Zone::Battlefield);
    let provider = put(&mut g, p, "Elemental Mascot", Zone::Battlefield);
    let donor = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let spell = put(&mut g, p, "Druid of the Cowl", Zone::Hand);
    g.state.players[0].mana = [0; 6];
    let mut independent = g.clone();
    let announce = |game: &mut Game| {
        choose(
            game,
            |action| matches!(action, Action::AnnounceMana { card, ability: 0, .. } if *card == land),
        );
    };
    let sacrifice = |game: &Game, card| Action::ChooseAdditionalCost {
        object: game.object_ref(card),
    };
    act(&mut g, Action::AnnounceCast { card: spell });
    announce(&mut g);
    let invalid = sacrifice(&g, provider);
    reject(&mut g, invalid);
    let payment = sacrifice(&g, donor);
    act(&mut g, payment);
    if g.state.players[0].mana == [0; 6] {
        pay(&mut g);
    }
    assert_eq!(g.state.players[0].mana, [0, 0, 0, 0, 2, 0]);
    assert_eq!(g.object_view(donor).zone, Zone::Graveyard);
    pay(&mut g);
    assert_eq!(g.object_view(spell).zone, Zone::Stack);

    announce(&mut independent);
    let payment = sacrifice(&independent, provider);
    act(&mut independent, payment);
    if matches!(independent.state.pending, Pending::PayCosts { .. }) {
        pay(&mut independent);
    }
    assert_eq!(independent.object_view(provider).zone, Zone::Graveyard);
    assert_eq!(independent.state.players[0].mana, [0, 0, 0, 0, 1, 0]);
}

#[test]
fn returned_trigger_multiplier_is_visible_to_the_projected_mana_payment() {
    const LAVA: CardRules = CardRules {
        effect: Effect::Creature(2, 2),
        static_groups: &[StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::BasicLandTapManaBonus(1)],
        }],
        ..CardRules::EMPTY
    };
    const MULTIPLIER: CardRules = CardRules {
        effect: Effect::Creature(2, 2),
        static_groups: &[StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::AdditionalTriggers {
                scope: AdditionalTriggerScope::OtherControlledSubtype("Elemental"),
                condition: StatValue::Fixed(1),
            }],
        }],
        ..CardRules::EMPTY
    };
    let _rules = fixture_rules::activate(&[
        (Card::from_name("Elemental Mascot").unwrap(), &LAVA),
        (Card::from_name("Bear Cub").unwrap(), &MULTIPLIER),
        (Card::from_name("Confiscate").unwrap(), &DRAW),
    ]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p, "Elemental Mascot", Zone::Battlefield);
        let bearer = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let ring = put(&mut g, p, "Lotus Ring", Zone::Battlefield);
        let island = put(&mut g, p, "Island", Zone::Battlefield);
        let returning = put(&mut g, p, "Bear Cub", Zone::Exile);
        let spell = put(&mut g, p, "Confiscate", Zone::Hand);
        let source = g.object_ref(bearer);
        let grantor = g.object_ref(ring);
        g.state.objects[ring as usize].attached_to = Some(source);
        g.state.players[p.index()].mana = [0; 6];
        let cast = Action::AnnounceCast { card: spell };
        reject(&mut g, cast.clone());
        g.state.exile_links.push(ExileLink {
            ability_origin: None,
            source,
            exiled: g.object_ref(returning),
            return_zone: Zone::Battlefield,
        });
        act(&mut g, cast);
        reject(&mut g, Action::TapMana { card: island });
        act(
            &mut g,
            Action::TapGrantedMana {
                card: bearer,
                grantor,
                production: [0, 3, 0, 0, 0, 0],
            },
        );
        assert_eq!(g.object_view(returning).zone, Zone::Battlefield);
        act(&mut g, Action::TapMana { card: island });
        assert_eq!(g.state.players[p.index()].mana, [0, 6, 0, 0, 0, 0]);
        pay(&mut g);
        assert_eq!(g.object_view(spell).zone, Zone::Stack);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}
#[test]
fn mana_value_restriction_splits_x_intervals_before_and_after_mana_is_produced() {
    const X: CardRules = CardRules {
        extras: &CardExtras {
            x_symbols: 1,
            ..CardExtras::EMPTY
        },
        ..DRAW
    };
    const PRODUCER: CardRules = CardRules {
        effect: Effect::Permanent,
        activated: &[ActivatedAbility {
            cost: ActivationCost {
                tap: true,
                ..ActivationCost::ZERO
            },
            program: &[Instruction::AddMana(ManaProduction::Tagged {
                production: &ManaProduction::Fixed([0, 4, 0, 0, 0, 0]),
                quality: &ManaQuality {
                    restriction: ManaRestriction::SpellManaValueAtLeast(4),
                    spell_bonus: ManaSpellBonus::None,
                    retention: ManaRetention::Step,
                },
            })],
            ..BASE
        }],
        ..CardRules::EMPTY
    };
    let _rules = fixture_rules::activate(&[
        (Card::from_name("Short Bow").unwrap(), &PRODUCER),
        (Card::Divination, &X),
    ]);
    for floated in [false, true] {
        let mut g = scenario(Player::P0);
        let producer = put(&mut g, Player::P0, "Short Bow", Zone::Battlefield);
        let spell = put(&mut g, Player::P0, "Divination", Zone::Hand);
        g.state.players[0].mana = [0; 6];
        if floated {
            act(&mut g, direct(producer, [0; 6], [0, 4, 0, 0, 0, 0]));
            g.move_object(producer, Zone::Graveyard);
        }
        act(&mut g, Action::AnnounceCast { card: spell });
        reject(&mut g, Action::ChooseX { value: 0 });
        act(&mut g, Action::ChooseX { value: 1 });
        if !floated {
            act(&mut g, direct(producer, [0; 6], [0, 4, 0, 0, 0, 0]));
        }
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.hand_count(Player::P0), 1);
    }
}
#[test]
fn color_and_graveyard_subtype_activation_guards_count_owned_physical_objects() {
    let mut g = scenario(Player::P0);
    let witness = put(&mut g, Player::P0, "Llanowar Elves", Zone::Battlefield);
    g.set_color_effects(&[witness], 31, AnimationDuration::Permanent);
    assert!(g.activation_guard_available(
        ActivationGuard::ControlColorsAtLeast(5),
        witness,
        Player::P0
    ));
    assert!(!g.activation_guard_available(
        ActivationGuard::ControlColorsAtLeast(5),
        witness,
        Player::P1
    ));
    assert!(!g.activation_guard_available(
        ActivationGuard::GraveyardSubtype("Elf"),
        witness,
        Player::P0
    ));
    g.move_object(witness, Zone::Graveyard);
    assert!(g.activation_guard_available(
        ActivationGuard::GraveyardSubtype("Elf"),
        witness,
        Player::P0
    ));
    assert!(!g.activation_guard_available(
        ActivationGuard::ControlColorsAtLeast(5),
        witness,
        Player::P0
    ));
}
#[test]
fn generic_resolution_color_choice_is_owned_and_does_not_add_unprinted_hexproof() {
    const COLOR: CardRules = CardRules {
        effect: Effect::Permanent,
        triggers: &[TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[
                Instruction::ChooseColor,
                Instruction::SetColors {
                    subject: Subject::Source,
                    colors: ColorValue::Chosen,
                    duration: AnimationDuration::Permanent,
                },
            ],
        }],
        ..CardRules::EMPTY
    };
    let _rules = fixture_rules::activate(&[(Card::from_name("Short Bow").unwrap(), &COLOR)]);
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P0, "Short Bow", Zone::Hand);
    cast(&mut g, source, None);
    for _ in 0..12 {
        if matches!(g.state.pending, Pending::ResolutionColor { .. }) {
            break;
        }
        choose(&mut g, |a| {
            matches!(a, Action::Pass | Action::PutTrigger { .. })
        });
    }
    let snapshot = g.snapshot();
    let mut branch = g.clone();
    let action = Action::ChooseResolutionColor {
        color: ChoiceColor::Blue,
    };
    let ix = branch
        .legal_actions()
        .iter()
        .position(|a| *a == action)
        .unwrap();
    branch
        .apply_index(Player::P0, branch.revision(), ix)
        .unwrap();
    act(&mut g, action);
    assert_eq!(g.snapshot(), branch.snapshot());
    assert_eq!(g.object_view(source).colors, 2);
    assert!(!g.keywords(source).contains(Keywords::HEXPROOF));
    branch.restore(&snapshot).unwrap();
    act(
        &mut branch,
        Action::ChooseResolutionColor {
            color: ChoiceColor::Red,
        },
    );
    assert_eq!(branch.object_view(source).colors, 8);
    assert_eq!(g.object_view(source).colors, 2);
    assert_eq!(
        g.object_view(source).kinds,
        Card::from_name("Short Bow").unwrap().kind_mask()
    );
}
#[test]
fn chosen_subtype_mana_is_frozen_and_qualifies_noncreature_spells_and_activations() {
    let mut g = scenario(Player::P0);
    let producer = put(&mut g, Player::P0, "Short Bow", Zone::Battlefield);
    let elf = CreatureType::from_name("Elf").unwrap();
    g.set_entry_value(g.object_ref(producer), EntryChoiceValue::CreatureType(elf));
    let quality = mana_lots::Quality::capture(
        &g,
        producer,
        ManaQuality {
            restriction: ManaRestriction::ChosenSubtypeSpellOrActivation,
            spell_bonus: ManaSpellBonus::None,
            retention: ManaRetention::Step,
        },
    );
    let mut source = mana_lots::Source::in_world(&g, producer);
    source.kinds = 8;
    source.subtypes.chosen = Some(elf);
    assert!(quality.eligible(mana_lots::Purpose::Spell(source)));
    assert!(quality.eligible(mana_lots::Purpose::Activation(source)));
    assert!(!quality.eligible(mana_lots::Purpose::Other));
    source.subtypes.chosen = Some(CreatureType::from_name("Dwarf").unwrap());
    assert!(!quality.eligible(mana_lots::Purpose::Spell(source)));
    g.move_object(producer, Zone::Exile);
    g.move_object(producer, Zone::Battlefield);
    g.set_entry_value(
        g.object_ref(producer),
        EntryChoiceValue::CreatureType(CreatureType::from_name("Dwarf").unwrap()),
    );
    source.subtypes.chosen = Some(elf);
    assert!(quality.eligible(mana_lots::Purpose::Spell(source)));
}

#[test]
fn selective_rollback_retains_source_recolor_only_with_its_paid_mana_event() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Short Bow").unwrap(), &WICKER)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Short Bow", Zone::Battlefield);
        let spell = put(&mut g, p, "Divination", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
        // Exercise the completed-event boundary used by foreign-choice rollback.
        // The foreign Aura failure itself is covered by dft_mana_output.
        g.state.cast_rollback = Some(Box::new(cast_rollback::CastRollback::new(
            g.state.clone(),
            p,
            spell,
        )));
        act(&mut g, Action::AnnounceCast { card: spell });
        act(
            &mut g,
            direct(source, [0, 0, 0, 0, 0, 1], [0, 1, 0, 0, 0, 0]),
        );
        let receipt = &g.state.cast_rollback.as_ref().unwrap().events[0];
        assert_eq!(receipt.recolor, Some(2));
        assert_eq!(receipt.production, [0, 1, 0, 0, 0, 0]);
        g.finish_cast_rollback(false);
        let snapshot = g.snapshot();
        let mut reversed = g.clone();
        act(&mut g, Action::ChooseManaRollback { keep: true });
        assert_eq!(g.object_view(source).colors, 2);
        assert_eq!(g.state.players[p.index()].mana, [0, 1, 0, 0, 0, 2]);
        assert_eq!(g.state.objects[spell as usize].zone, Zone::Hand);
        assert_eq!(g.state.objects[source as usize].used_abilities, 1);
        reversed.restore(&snapshot).unwrap();
        act(&mut reversed, Action::ChooseManaRollback { keep: false });
        assert_eq!(reversed.object_view(source).colors, 0);
        assert_eq!(reversed.state.players[p.index()].mana, [0, 0, 0, 0, 0, 3]);
        assert_eq!(reversed.state.objects[source as usize].used_abilities, 0);
        // Keeping the mana event also keeps its color change, with the original
        // until-end-of-turn duration; selective replay cannot make it permanent.
        let turn = g.state.turn;
        for _ in 0..48 {
            if g.state.turn != turn {
                break;
            }
            choose(&mut g, |action| {
                matches!(
                    action,
                    Action::Pass | Action::FinishAttackers | Action::FinishBlockers
                )
            });
        }
        assert_ne!(g.state.turn, turn);
        assert_eq!(g.object_view(source).colors, 0);
    }
}
#[test]
fn delayed_produced_type_bonus_belongs_to_the_same_selectively_replayed_mana_event() {
    const LAND: CardRules = CardRules {
        effect: Effect::Permanent,
        activated: &[ActivatedAbility {
            cost: ActivationCost {
                tap: true,
                sacrifice: true,
                ..ActivationCost::ZERO
            },
            program: &[Instruction::AddMana(ManaProduction::Fixed([
                0, 1, 0, 0, 1, 0,
            ]))],
            ..BASE
        }],
        ..CardRules::EMPTY
    };
    const LAVA: CardRules = CardRules {
        effect: Effect::Permanent,
        static_groups: &[StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::BasicLandTapManaBonus(1)],
        }],
        ..CardRules::EMPTY
    };
    let _rules = fixture_rules::activate(&[
        (Card::Forest, &LAND),
        (Card::from_name("Short Bow").unwrap(), &LAVA),
        (Card::from_name("Cancel").unwrap(), &DRAW),
    ]);
    let mut g = scenario(Player::P0);
    let land = put(&mut g, Player::P0, "Forest", Zone::Battlefield);
    put(&mut g, Player::P1, "Short Bow", Zone::Battlefield);
    let spell = put(&mut g, Player::P0, "Cancel", Zone::Hand);
    g.state.players[0].mana = [0; 6];
    g.state.cast_rollback = Some(Box::new(cast_rollback::CastRollback::new(
        g.state.clone(),
        Player::P0,
        spell,
    )));
    act(&mut g, Action::AnnounceCast { card: spell });
    act(&mut g, direct(land, [0; 6], [0, 1, 0, 0, 1, 0]));
    let rollback = g.state.cast_rollback.as_ref().unwrap();
    assert!(rollback.capture.as_ref().unwrap().produced);
    assert!(
        rollback.events.is_empty(),
        "deferred bonus is part of this event"
    );
    act(
        &mut g,
        Action::ChooseMana {
            production: [0, 1, 0, 0, 0, 0],
        },
    );
    let rollback = g.state.cast_rollback.as_ref().unwrap();
    assert!(rollback.capture.is_none());
    assert_eq!(rollback.events.len(), 1);
    assert_eq!(rollback.events[0].tap_bonus, [0, 1, 0, 0, 0, 0]);
    g.finish_cast_rollback(false);
    let mut indexed = g.clone();
    let action = Action::ChooseManaRollback { keep: true };
    let ix = indexed
        .legal_actions()
        .iter()
        .position(|a| *a == action)
        .unwrap();
    indexed
        .apply_index(Player::P0, indexed.revision(), ix)
        .unwrap();
    act(&mut g, action);
    assert_eq!(g.snapshot(), indexed.snapshot());
    assert_eq!(g.state.players[0].mana, [0, 2, 0, 0, 1, 0]);
    assert_eq!(g.state.objects[land as usize].zone, Zone::Graveyard);
    assert_eq!(g.state.objects[spell as usize].zone, Zone::Hand);
    assert!(!g.mana_tap_bonus_pending());
}

#[test]
fn fixed_land_tap_bonus_requires_positive_output_in_actual_and_projected_worlds() {
    use rustmage_core::engine::funding::{FundingGoal, FundingState};
    const ZERO: CardRules = CardRules {
        effect: Effect::Land,
        activated: &[ActivatedAbility {
            cost: ActivationCost {
                tap: true,
                ..ActivationCost::ZERO
            },
            program: &[Instruction::AddMana(ManaProduction::Fixed([0; 6]))],
            ..BASE
        }],
        ..CardRules::EMPTY
    };
    const POSITIVE: CardRules = CardRules {
        activated: &[ActivatedAbility {
            cost: ActivationCost {
                tap: true,
                ..ActivationCost::ZERO
            },
            program: &[Instruction::AddMana(ManaProduction::Fixed([
                0, 0, 0, 0, 1, 0,
            ]))],
            ..BASE
        }],
        ..ZERO
    };
    const AURA: CardRules = CardRules {
        effect: Effect::Permanent,
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
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::AttachmentManaBonus(
                ManaProduction::Fixed([0, 1, 0, 0, 0, 0]),
            )],
        }],
        ..CardRules::EMPTY
    };
    for positive in [false, true] {
        let _rules = fixture_rules::activate(&[
            (
                Card::from_name("Tarnation Vista").unwrap(),
                if positive { &POSITIVE } else { &ZERO },
            ),
            (Card::from_name("Pacifism").unwrap(), &AURA),
        ]);
        let p = Player::P1;
        let mut g = scenario(p);
        let land = put(&mut g, p, "Tarnation Vista", Zone::Battlefield);
        let aura = put(&mut g, p.other(), "Pacifism", Zone::Battlefield);
        g.state.objects[aura as usize].attached_to = Some(g.object_ref(land));
        g.state.players[p.index()].mana = [0; 6];
        let goal = FundingGoal {
            actor: p,
            required: [0, 1, 0, 0, 0, 0],
            generic: 0,
            source: None,
            reserve: None,
        };
        let world = FundingState::from_game(&g, &goal);
        let step = world
            .moves(&goal)
            .into_iter()
            .find(|step| world.resources[step.resource].id == land && step.ability == Some(0))
            .unwrap();
        let expected: Mana = if positive { [0, 1, 0, 0, 1, 0] } else { [0; 6] };
        let mut projected = None;
        world.visit_applied(&step, &mut |next| {
            projected = Some(next.mana);
            true
        });
        assert_eq!(projected, Some(expected.map(u128::from)));
        assert_eq!(world.mana, [0; 6]);
        assert_eq!(
            world.reachable(&goal, &mut std::collections::HashSet::new()),
            positive
        );
        let saved = g.snapshot();
        let mut raw = g.clone();
        let mut indexed = g.clone();
        let production = if positive { [0, 0, 0, 0, 1, 0] } else { [0; 6] };
        let action = direct(land, [0; 6], production);
        let index = indexed
            .legal_actions()
            .iter()
            .position(|a| *a == action)
            .unwrap();
        act(&mut raw, action);
        indexed.apply_index(p, indexed.revision(), index).unwrap();
        assert_eq!(raw.snapshot(), indexed.snapshot());
        assert_eq!(raw.state.players[p.index()].mana, expected);
        assert!(raw.state.objects[land as usize].tapped);
        assert!(raw.state.stack.is_empty() && raw.state.pending_triggers.is_empty());
        assert_eq!(g.snapshot(), saved);
        raw.restore(&saved).unwrap();
        assert_eq!(raw.state.players[p.index()].mana, [0; 6]);
        assert!(!raw.state.objects[land as usize].tapped);
    }
}

const EXTERNAL_TAP_MANA: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            tap: true,
            additional: Some(AdditionalCost {
                kind: EffectChoiceKind::TapPermanents(1),
                filter: SacrificeFilter {
                    other: false,
                    ..FILTER
                },
            }),
            ..ActivationCost::ZERO
        },
        program: &[Instruction::AddMana(ManaProduction::Fixed([
            0, 0, 0, 1, 0, 0,
        ]))],
        ..BASE
    }],
    ..CardRules::EMPTY
};

fn announce_external_tap_mana(card: CardId) -> Action {
    Action::AnnounceMana {
        card,
        ability: 0,
        counter_payment: Counters::default(),
        production: [0, 0, 0, 1, 0, 0],
    }
}

#[test]
fn separate_tap_mana_cost_keeps_exact_sick_witness_and_owned_payment() {
    let _rules = fixture_rules::activate(&[
        (Card::GrizzlyBears, &EXTERNAL_TAP_MANA),
        (Card::from_name("Shock").unwrap(), &DRAW),
    ]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let spell = put(&mut g, p, "Shock", Zone::Hand);
        g.state.players[p.index()].mana = [0; 6];
        let cast = Action::AnnounceCast { card: spell };
        // The source cannot pay both the tap symbol and the separate tap cost.
        reject(&mut g, cast.clone());
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let tapped = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.state.objects[tapped as usize].tapped = true;
        reject(&mut g, cast.clone());
        let witness = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.state.objects[witness as usize].summoning_sick = true;
        act(&mut g, cast);
        act(&mut g, announce_external_tap_mana(source));
        for card in [source, enemy, tapped] {
            let object = g.object_ref(card);
            reject(&mut g, Action::ChooseAdditionalCost { object });
        }
        let object = g.object_ref(witness);
        let choice = Action::ChooseAdditionalCost { object };
        let saved = g.snapshot();
        let mut raw = {
            let parent = g.clone();
            parent.clone()
        };
        let mut indexed = g.clone();
        drop(g);
        let index = indexed
            .legal_actions()
            .iter()
            .position(|a| *a == choice)
            .unwrap();
        indexed.apply_index(p, indexed.revision(), index).unwrap();
        act(&mut raw, choice.clone());
        assert_eq!(raw.snapshot(), indexed.snapshot());
        pay(&mut raw);
        pay(&mut raw);
        assert!(raw.object_view(source).tapped);
        assert!(raw.object_view(witness).tapped);
        assert!(raw.state.objects[witness as usize].summoning_sick);
        assert_eq!(raw.object_ref(witness), object);
        assert_eq!(raw.object_view(spell).zone, Zone::Stack);
        assert_eq!(raw.state.players[p.index()].mana, [0; 6]);
        let paid = raw.snapshot();
        raw.restore(&saved).unwrap();
        assert!(!raw.object_view(source).tapped);
        assert!(!raw.object_view(witness).tapped);
        act(&mut raw, choice);
        pay(&mut raw);
        pay(&mut raw);
        assert_eq!(raw.snapshot(), paid);
    }
}

#[test]
fn separate_tap_mana_cost_cannot_consume_the_enclosing_activation_tap() {
    const PARENT: CardRules = CardRules {
        effect: Effect::Creature(3, 3),
        activated: &[ActivatedAbility {
            cost: ActivationCost {
                colored: [0, 0, 0, 1, 0, 0],
                mana_component: true,
                tap: true,
                ..ActivationCost::ZERO
            },
            program: &[Instruction::Draw(PlayerSet::Controller, 1)],
            ..BASE
        }],
        ..CardRules::EMPTY
    };
    let _rules = fixture_rules::activate(&[
        (Card::GrizzlyBears, &EXTERNAL_TAP_MANA),
        (Card::HillGiant, &PARENT),
    ]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let parent = put(&mut g, p, "Hill Giant", Zone::Battlefield);
        g.state.players[p.index()].mana = [0; 6];
        let activate = Action::AnnounceActivate {
            card: parent,
            ability: 0,
            counter_payment: Counters::default(),
        };
        reject(&mut g, activate.clone());
        let spare = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.state.objects[spare as usize].summoning_sick = true;
        act(&mut g, activate);
        act(&mut g, announce_external_tap_mana(source));
        let object = g.object_ref(parent);
        reject(&mut g, Action::ChooseAdditionalCost { object });
        let object = g.object_ref(spare);
        act(&mut g, Action::ChooseAdditionalCost { object });
        pay(&mut g);
        assert!(!g.object_view(parent).tapped);
        pay(&mut g);
        assert!(g.object_view(parent).tapped);
        assert!(g.object_view(source).tapped);
        assert!(g.object_view(spare).tapped);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        settle(&mut g);
        assert_eq!(g.hand_count(p), 1);
    }
}

#[test]
fn separate_tap_mana_witnesses_share_one_announcement_and_distinct_owned_choices() {
    let _rules = fixture_rules::activate(&[
        (Card::GrizzlyBears, &EXTERNAL_TAP_MANA),
        (Card::from_name("Shock").unwrap(), &DRAW),
    ]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let first = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let second = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let spell = put(&mut g, p, "Shock", Zone::Hand);
        g.state.players[p.index()].mana = [0; 6];
        act(&mut g, Action::AnnounceCast { card: spell });
        let announcement = announce_external_tap_mana(source);
        let actions = g.legal_actions();
        assert_eq!(actions.iter().filter(|a| **a == announcement).count(), 1);
        let index = actions.iter().position(|a| *a == announcement).unwrap();
        let mut indexed = g.clone();
        indexed.apply_index(p, indexed.revision(), index).unwrap();
        act(&mut g, announcement);
        assert_eq!(g.snapshot(), indexed.snapshot());
        for (chosen, other) in [(first, second), (second, first)] {
            let mut branch = g.clone();
            let object = branch.object_ref(chosen);
            act(&mut branch, Action::ChooseAdditionalCost { object });
            pay(&mut branch);
            pay(&mut branch);
            assert!(branch.object_view(source).tapped);
            assert!(branch.object_view(chosen).tapped);
            assert!(!branch.object_view(other).tapped);
            assert_eq!(branch.object_view(spell).zone, Zone::Stack);
            assert_eq!(branch.state.players[p.index()].mana, [0; 6]);
        }
    }
}

#[test]
fn selective_rollback_retains_exact_separate_tap_cost_and_owned_mana() {
    let _rules = fixture_rules::activate(&[
        (Card::GrizzlyBears, &EXTERNAL_TAP_MANA),
        (Card::from_name("Shock").unwrap(), &DRAW),
    ]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let witness = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let spell = put(&mut g, p, "Shock", Zone::Hand);
        g.state.objects[witness as usize].summoning_sick = true;
        g.state.players[p.index()].mana = [0; 6];
        let object = g.object_ref(witness);
        // Reach the same completed-event boundary as a failed foreign entry
        // choice, with the actual additional choice and mana ability payment.
        g.state.cast_rollback = Some(Box::new(cast_rollback::CastRollback::new(
            g.state.clone(),
            p,
            spell,
        )));
        act(&mut g, Action::AnnounceCast { card: spell });
        act(&mut g, announce_external_tap_mana(source));
        act(&mut g, Action::ChooseAdditionalCost { object });
        pay(&mut g);
        assert!(g.object_view(source).tapped);
        assert!(g.object_view(witness).tapped);
        assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 1, 0, 0]);
        let receipt = &g.state.cast_rollback.as_ref().unwrap().events[0];
        let cast_rollback_mana::ManaEventKind::Activated { additional, .. } = &receipt.kind else {
            panic!("expected paid activated mana receipt");
        };
        assert_eq!(
            *additional,
            Some((
                EXTERNAL_TAP_MANA.activated[0].cost.additional.unwrap(),
                object
            ))
        );
        g.finish_cast_rollback(false);
        let saved = g.snapshot();
        let action = Action::ChooseManaRollback { keep: true };
        let mut indexed = g.clone();
        let index = indexed
            .legal_actions()
            .iter()
            .position(|candidate| *candidate == action)
            .expect("the exact untapped witness permits retaining its paid event");
        indexed.apply_index(p, indexed.revision(), index).unwrap();
        act(&mut g, action.clone());
        assert_eq!(g.snapshot(), indexed.snapshot());
        assert!(g.object_view(source).tapped);
        assert!(g.object_view(witness).tapped);
        assert!(g.state.objects[witness as usize].summoning_sick);
        assert_eq!(g.object_ref(witness), object);
        assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 1, 0, 0]);
        assert_eq!(g.object_view(spell).zone, Zone::Hand);
        let kept = g.snapshot();
        g.restore(&saved).unwrap();
        for invalid in 0..4 {
            let mut rejected = g.clone();
            match invalid {
                0 => rejected.state.objects[witness as usize].tapped = true,
                1 => rejected.state.objects[witness as usize].controller = p.other(),
                2 => rejected.state.objects[witness as usize].generation += 1,
                _ => {
                    // Even a creature source cannot pay its tap symbol and the
                    // separate tap cost with the same permanent.
                    let source = rejected.object_ref(source);
                    let receipt = &mut rejected.state.cast_rollback.as_mut().unwrap().events[0];
                    let cast_rollback_mana::ManaEventKind::Activated { additional, .. } =
                        &mut receipt.kind
                    else {
                        unreachable!();
                    };
                    additional.as_mut().unwrap().1 = source;
                }
            }
            assert!(
                !rejected.legal_actions().contains(&action),
                "invalid witness {invalid}"
            );
            reject(&mut rejected, action.clone());
        }
        act(&mut g, Action::ChooseManaRollback { keep: false });
        assert!(!g.object_view(source).tapped);
        assert!(!g.object_view(witness).tapped);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        g.restore(&saved).unwrap();
        act(&mut g, action);
        assert_eq!(g.snapshot(), kept);
    }
}

#[test]
fn acquired_separate_tap_mana_is_unique_and_preserves_parent_tap_reservations() {
    const W: Mana = [1, 0, 0, 0, 0, 0];
    const DONOR: CardRules = CardRules {
        effect: Effect::Permanent,
        activated: &[
            ActivatedAbility {
                program: &[Instruction::AddMana(ManaProduction::Fixed(W))],
                ..EXTERNAL_TAP_MANA.activated[0]
            },
            ActivatedAbility {
                cost: ActivationCost {
                    tap: true,
                    ..ActivationCost::ZERO
                },
                program: &[Instruction::AddMana(ManaProduction::Fixed(W))],
                ..BASE
            },
        ],
        ..CardRules::EMPTY
    };
    const GRANTED_DONOR: CardRules = CardRules {
        extras: &CardExtras {
            self_granted_activation_templates: &[0, 1],
            ..CardExtras::EMPTY
        },
        ..DONOR
    };
    const PARENT: CardRules = CardRules {
        effect: Effect::Creature(3, 3),
        activated: &[ActivatedAbility {
            cost: ActivationCost {
                colored: W,
                mana_component: true,
                tap: true,
                ..ActivationCost::ZERO
            },
            program: &[Instruction::Draw(PlayerSet::Controller, 1)],
            ..BASE
        }],
        ..CardRules::EMPTY
    };
    let donor_definition = Card::from_name("Gilded Lotus").unwrap();
    let _rules = fixture_rules::activate(&[
        (donor_definition, &DONOR),
        (Card::LlanowarElves, &GRANTED_DONOR),
        (Card::HillGiant, &PARENT),
    ]);
    for p in [Player::P0, Player::P1] {
        for borrowed in [false, true] {
            let mut g = scenario(p);
            let (source, external, ordinary) = if borrowed {
                let donor = put(&mut g, p, "Gilded Lotus", Zone::Battlefield);
                let source = put(&mut g, p, "Territory Forge", Zone::Hand);
                cast(&mut g, source, None);
                for _ in 0..48 {
                    if g.state.pending == Pending::Priority
                        && g.state.stack.is_empty()
                        && g.state.pending_triggers.is_empty()
                        && g.state.resolving.is_none()
                    {
                        break;
                    }
                    let target = Action::ChooseTarget {
                        target: Target::Card(donor),
                    };
                    if g.legal_actions().contains(&target) {
                        act(&mut g, target);
                    } else if g.legal_actions().contains(&Action::FinishTargets) {
                        act(&mut g, Action::FinishTargets);
                    } else {
                        choose(&mut g, |action| {
                            matches!(action, Action::Pass | Action::PutTrigger { .. })
                        });
                    }
                }
                assert_eq!(g.object_view(donor).zone, Zone::Exile);
                let exiled = g.object_ref(donor);
                let action = |ability| Action::AnnounceBorrowedMana {
                    card: source,
                    exiled,
                    ability,
                    counter_payment: Counters::default(),
                    production: W,
                };
                (source, action(0), action(1))
            } else {
                let source = put(&mut g, p, "Short Bow", Zone::Battlefield);
                let object = g.object_ref(source);
                let mut grant = |ability| {
                    let timestamp = g.next_role_timestamp();
                    g.grant_recipient_ability(
                        object,
                        Card::LlanowarElves,
                        SelfGrantedAbilityKind::Activated { ability },
                        timestamp,
                        None,
                    );
                    Action::AnnounceSelfGrantedMana {
                        source: object,
                        grant_timestamp: timestamp,
                        counter_payment: Counters::default(),
                        production: W,
                    }
                };
                (source, grant(0), grant(1))
            };
            let parent = put(&mut g, p, "Hill Giant", Zone::Battlefield);
            let plains = put(&mut g, p, "Plains", Zone::Battlefield);
            g.state.players[p.index()].mana = [0; 6];
            act(
                &mut g,
                Action::AnnounceActivate {
                    card: parent,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            // The parent can use Plains, but its own reserved tap cannot also
            // pay the acquired Drum cost. A packet without a witness is illegal.
            assert!(!g.legal_actions().contains(&external));
            reject(&mut g, external.clone());
            assert!(
                g.legal_actions()
                    .contains(&Action::TapMana { card: plains })
            );
            assert_eq!(
                g.legal_actions().iter().filter(|a| **a == ordinary).count(),
                1
            );
            let first = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let second = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            for card in [first, second] {
                g.state.objects[card as usize].summoning_sick = true;
            }
            let actions = g.legal_actions();
            assert_eq!(actions.iter().filter(|a| **a == external).count(), 1);
            // The non-external granted activation also overlaps both visitors;
            // it remains one distinct action rather than being discarded.
            assert_eq!(actions.iter().filter(|a| **a == ordinary).count(), 1);
            let saved = g.snapshot();
            let mut indexed = g.clone();
            let index = actions.iter().position(|a| *a == external).unwrap();
            indexed.apply_index(p, indexed.revision(), index).unwrap();
            act(&mut g, external.clone());
            assert_eq!(g.snapshot(), indexed.snapshot());
            let object = g.object_ref(parent);
            reject(&mut g, Action::ChooseAdditionalCost { object });
            for (chosen, other) in [(first, second), (second, first)] {
                let mut branch = g.clone();
                let object = branch.object_ref(chosen);
                act(&mut branch, Action::ChooseAdditionalCost { object });
                pay(&mut branch);
                assert_eq!(branch.state.players[p.index()].mana, W);
                assert!(!branch.object_view(parent).tapped);
                assert!(!branch.object_view(plains).tapped);
                pay(&mut branch);
                assert!(branch.object_view(source).tapped);
                assert!(branch.object_view(parent).tapped);
                assert!(branch.object_view(chosen).tapped);
                assert!(!branch.object_view(other).tapped);
                assert!(branch.state.objects[chosen as usize].summoning_sick);
                assert_eq!(branch.state.players[p.index()].mana, [0; 6]);
                settle(&mut branch);
                assert_eq!(branch.hand_count(p), 1);
            }
            g.restore(&saved).unwrap();
            assert!(!g.object_view(source).tapped);
            assert_eq!(
                g.legal_actions().iter().filter(|a| **a == external).count(),
                1
            );
        }
    }
}
