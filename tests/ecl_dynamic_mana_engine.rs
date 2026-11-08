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
        (Card::from_name("Short Bow").unwrap(), &LAVA),
        (Card::from_name("Cancel").unwrap(), &DRAW),
    ]);
    let mut g = scenario(Player::P0);
    let land = put(&mut g, Player::P0, "Forest", Zone::Battlefield);
    put(&mut g, Player::P1, "Short Bow", Zone::Battlefield);
    let spell = put(&mut g, Player::P0, "Cancel", Zone::Hand);
    g.state.players[0].mana = [0; 6];
    act(&mut g, Action::AnnounceCast { card: spell });
    act(&mut g, direct(land, [0; 6], [0, 1, 0, 0, 1, 0]));
    assert_eq!(g.state.objects[land as usize].zone, Zone::Graveyard);
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
