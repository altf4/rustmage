//! As-entry copies choose flat blueprints before replacements, with exact physical identity.
mod support;
use rustmage as _;
use rustmage_core::CardName;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;
use rustmage_core::engine::ability_world::CharacteristicWorld;
use rustmage_core::engine::funding::{FundingGoal, FundingState};
use rustmage_core::engine::*;
use support::*;

const COPY: CardRules = CardRules {
    effect: Effect::Creature(0, 0),
    keywords: Keywords::FLASH,
    extras: &CardExtras {
        entry_copy: Some(EntryCopySpec {
            filters: &[BattlefieldFilter {
                kinds: 17,
                relation: 0,
                nonland: false,
                tapped: None,
                without: Keywords::NONE,
                subtype: None,
                max_mana: u32::MAX,
            }],
            optional: true,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const PROTECTED: CardRules = CardRules {
    effect: Effect::Creature(2, 1),
    keywords: Keywords::PROTECTION_EVERYTHING,
    ..CardRules::EMPTY
};
const COUNTERED: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    entry: EntryTapped::Always,
    enters: InitialCounters {
        plus_one: 3,
        ..CardRules::EMPTY.enters
    },
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::When {
            event: &TriggerCondition::SelfEnters,
            guard: StepGuard::Expression {
                value: StatValue::SourceCounters(CounterKind::PlusOne),
                minimum: Some(3),
                maximum: None,
            },
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 3)],
    }],
    ..CardRules::EMPTY
};
const CHOOSE_TYPE: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Type,
        fragments: &[StaticAbilityFragment::ChosenCreatureType],
    }],
    extras: &CardExtras {
        entry_choice: Some(EntryChoiceKind::CreatureType),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const CHOOSE_NAME: CardRules = CardRules {
    effect: Effect::Permanent,
    extras: &CardExtras {
        entry_choice: Some(EntryChoiceKind::LandCardName),
        named_activation_lock: true,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
fn goal(actor: Player) -> FundingGoal {
    FundingGoal {
        actor,
        required: [0; 6],
        generic: 0,
        reserve: None,
        source: None,
    }
}
fn reach_choice(g: &mut Game) {
    for _ in 0..20 {
        if g.state.pending != Pending::Priority {
            return;
        }
        assert!(!g.state.stack.is_empty() || g.state.resolving.is_some());
        act(g, Action::Pass);
    }
    panic!("bounded entry choice expected");
}
fn copy_cast(g: &mut Game) -> CardId {
    let id = put(g, Player::P0, "Grizzly Bears", Zone::Hand);
    cast(g, id, None);
    reach_choice(g);
    id
}
fn select_copy(g: &mut Game, id: CardId) {
    let object = g.object_ref(id);
    act(
        g,
        Action::ChooseEntryCopy {
            object: Some(object),
        },
    );
}

#[test]
fn as_entry_copy_is_nontargeted_and_the_physical_stack_object_waits_for_commit() {
    let _scope = fixture_rules::activate(&[
        (Card::from_name("Grizzly Bears").unwrap(), &COPY),
        (Card::from_name("Savannah Lions").unwrap(), &PROTECTED),
    ]);
    let mut g = scenario(Player::P0);
    let blueprint = put(&mut g, Player::P0, "Savannah Lions", Zone::Battlefield);
    let enemy = put(&mut g, Player::P1, "Hill Giant", Zone::Battlefield);
    let copy = copy_cast(&mut g);
    assert!(matches!(g.state.pending, Pending::EntryCopy { .. }));
    assert_eq!(
        g.state.objects[copy as usize].card,
        Card::from_name("Grizzly Bears").unwrap()
    );
    assert_eq!(g.state.objects[copy as usize].zone, Zone::Stack);
    let invalid = g.object_ref(enemy);
    reject(
        &mut g,
        Action::ChooseEntryCopy {
            object: Some(invalid),
        },
    );
    select_copy(&mut g, blueprint);
    settle(&mut g);
    assert_eq!(stats(&g, copy), (2, 1));
    assert!(g.keywords(copy).contains(Keywords::PROTECTION_EVERYTHING));
    let view = g.temporary_copy_view(g.object_ref(copy)).unwrap();
    assert_eq!(
        view.physical_card,
        Card::from_name("Grizzly Bears").unwrap()
    );
    assert!(!view.physical_token);
}

#[test]
fn copying_an_artifact_survives_cleanup_and_departure_restores_the_physical_card() {
    let _scope = fixture_rules::activate(&[(Card::from_name("Grizzly Bears").unwrap(), &COPY)]);
    let mut g = scenario(Player::P0);
    let artifact = put(
        &mut g,
        Player::P0,
        "Cultivator's Caravan",
        Zone::Battlefield,
    );
    let copy = copy_cast(&mut g);
    select_copy(&mut g, artifact);
    settle(&mut g);
    assert!(!g.is_creature(copy));
    assert!(g.has_subtype(copy, "Vehicle"));
    g.expire_temporary_copies(true);
    assert_eq!(
        g.state.objects[copy as usize].card,
        Card::from_name("Cultivator's Caravan").unwrap()
    );
    g.move_object(copy, Zone::Hand);
    assert_eq!(
        g.state.objects[copy as usize].card,
        Card::from_name("Grizzly Bears").unwrap()
    );
    assert!(g.temporary_copy_view(g.object_ref(copy)).is_none());
}

#[test]
fn copying_an_existing_copy_flattens_its_blueprint_and_ignores_noncopy_modifiers() {
    let _scope = fixture_rules::activate(&[(Card::from_name("Grizzly Bears").unwrap(), &COPY)]);
    let mut g = scenario(Player::P0);
    let old = put(&mut g, Player::P0, "Swiftfoot Boots", Zone::Battlefield);
    g.apply_temporary_copy(
        g.object_ref(old),
        CopyRecipe::new(Card::from_name("Savannah Lions").unwrap()),
        CopyDeadline::EndOfTurn,
    );
    g.state.objects[old as usize].growth = Stats {
        power: 5,
        toughness: 5,
    };
    g.state.objects[old as usize].counters.plus_one = 2;
    let copy = copy_cast(&mut g);
    select_copy(&mut g, old);
    settle(&mut g);
    assert_eq!(stats(&g, copy), (2, 1));
    assert_eq!(g.counter_count(copy, CounterKind::PlusOne), 0);
    g.expire_temporary_copies(true);
    assert_eq!(
        g.state.objects[old as usize].card,
        Card::from_name("Swiftfoot Boots").unwrap()
    );
    assert_eq!(
        g.state.objects[copy as usize].card,
        Card::from_name("Savannah Lions").unwrap()
    );
}

#[test]
fn copied_entry_counters_and_the_copied_etb_ability_apply_before_observer_facts() {
    let _scope = fixture_rules::activate(&[
        (Card::from_name("Grizzly Bears").unwrap(), &COPY),
        (Card::from_name("Gleaming Barrier").unwrap(), &COUNTERED),
    ]);
    let mut g = scenario(Player::P0);
    let blueprint = put(&mut g, Player::P0, "Gleaming Barrier", Zone::Battlefield);
    let copy = copy_cast(&mut g);
    select_copy(&mut g, blueprint);
    settle(&mut g);
    assert_eq!(g.counter_count(copy, CounterKind::PlusOne), 3);
    assert_eq!(stats(&g, copy), (5, 5));
    assert!(
        g.state.objects[copy as usize].tapped,
        "the copied enters-tapped rule also applies"
    );
    assert_eq!(g.state.players[0].life, 23);
}

#[test]
fn copied_as_entry_type_choice_is_fresh_and_bound_to_its_ability_acquisition() {
    let _scope = fixture_rules::activate(&[
        (Card::from_name("Grizzly Bears").unwrap(), &COPY),
        (Card::from_name("Gleaming Barrier").unwrap(), &CHOOSE_TYPE),
    ]);
    for selected in ["Mount", "Time Lord"] {
        let mut g = scenario(Player::P0);
        let blueprint = put(&mut g, Player::P0, "Gleaming Barrier", Zone::Battlefield);
        g.set_entry_value(
            g.object_ref(blueprint),
            EntryChoiceValue::CreatureType(CreatureType::from_name("Elf").unwrap()),
        );
        let copy = copy_cast(&mut g);
        select_copy(&mut g, blueprint);
        assert!(matches!(g.state.pending, Pending::EntryValue { .. }));
        assert_eq!(g.state.objects[copy as usize].zone, Zone::Stack);
        act(
            &mut g,
            Action::ChooseEntryValue {
                value: EntryChoiceValue::CreatureType(CreatureType::from_name(selected).unwrap()),
            },
        );
        settle(&mut g);
        assert!(g.has_subtype(copy, selected) && !g.has_subtype(copy, "Elf"));
        assert_eq!(
            g.state.players[0].mount_or_vehicle_entries_this_turn,
            u64::from(selected == "Mount")
        );
        let original = g.object_ref(copy);
        g.apply_temporary_copy(
            original,
            CopyRecipe::new(Card::from_name("Savannah Lions").unwrap()),
            CopyDeadline::EndOfTurn,
        );
        assert!(g.entry_value(original).is_none());
        g.expire_temporary_copies(true);
        assert!(
            matches!(g.entry_value(original),Some(EntryChoiceValue::CreatureType(typ)) if typ.name()==selected)
        );
        assert!(g.has_subtype(copy, selected));
    }
}

#[test]
fn copied_as_entry_name_choice_is_new_and_its_named_restriction_is_live() {
    let _scope = fixture_rules::activate(&[
        (Card::from_name("Grizzly Bears").unwrap(), &COPY),
        (Card::from_name("Swiftfoot Boots").unwrap(), &CHOOSE_NAME),
    ]);
    let mut g = scenario(Player::P0);
    let blueprint = put(&mut g, Player::P0, "Swiftfoot Boots", Zone::Battlefield);
    g.set_entry_value(
        g.object_ref(blueprint),
        EntryChoiceValue::CardName(CardName::from_name("Forest").unwrap()),
    );
    let copy = copy_cast(&mut g);
    select_copy(&mut g, blueprint);
    assert!(matches!(g.state.pending, Pending::EntryValue { .. }));
    act(
        &mut g,
        Action::ChooseEntryValue {
            value: EntryChoiceValue::CardName(CardName::from_name("Island").unwrap()),
        },
    );
    settle(&mut g);
    g.move_object(blueprint, Zone::Graveyard);
    assert!(g.named_activation_prohibited_for_name("Island"));
    assert!(!g.named_activation_prohibited_for_name("Forest"));
    assert!(
        matches!(g.entry_value(g.object_ref(copy)),Some(EntryChoiceValue::CardName(name)) if name.matches("Island"))
    );
}

#[test]
fn a_live_artifact_aura_blueprint_copies_its_copiable_aura_and_chooses_new_attachment() {
    let _scope = fixture_rules::activate(&[(Card::from_name("Grizzly Bears").unwrap(), &COPY)]);
    let mut g = scenario(Player::P0);
    let first = put(&mut g, Player::P0, "Savannah Lions", Zone::Battlefield);
    let second = put(&mut g, Player::P1, "Hill Giant", Zone::Battlefield);
    let aura = put(&mut g, Player::P0, "Pacifism", Zone::Battlefield);
    g.state.objects[aura as usize].attached_to = Some(g.object_ref(first));
    g.change_object_form(
        g.object_ref(aura),
        AnimationSpec {
            artifact: true,
            creature_types: AnimationCreatureTypes::Unchanged,
            base: None,
            keywords: Keywords::NONE,
            duration: AnimationDuration::EndOfTurn,
        },
        characteristic_layers::AnimationTypeMode::PreserveCardTypes,
    );
    let copy = copy_cast(&mut g);
    select_copy(&mut g, aura);
    assert!(matches!(g.state.pending, Pending::AuraEntry { .. }));
    let recipient = g.object_ref(second);
    act(&mut g, Action::ChooseAuraAttachment { object: recipient });
    settle(&mut g);
    assert!(CharacteristicWorld::is_aura(&g, copy));
    assert!(!g.is_creature(copy));
    assert_eq!(g.state.objects[copy as usize].attached_to, Some(recipient));
    assert_eq!(
        g.current_kinds(copy) & 16,
        0,
        "the external artifact addition is not copiable"
    );
}

#[test]
fn optional_decline_and_selection_are_owned_across_raw_index_restore_and_parent_disposal() {
    let _scope = fixture_rules::activate(&[(Card::from_name("Grizzly Bears").unwrap(), &COPY)]);
    let mut parent = scenario(Player::P0);
    let blueprint = put(
        &mut parent,
        Player::P0,
        "Swiftfoot Boots",
        Zone::Battlefield,
    );
    let copy = copy_cast(&mut parent);
    let saved = parent.snapshot();
    let mut raw = parent.clone();
    let mut indexed = parent.clone();
    let decline = Action::ChooseEntryCopy { object: None };
    let index = indexed
        .legal_actions()
        .iter()
        .position(|a| *a == decline)
        .unwrap();
    let actor = indexed.actor().unwrap();
    let revision = indexed.revision();
    act(&mut raw, decline);
    indexed.apply_index(actor, revision, index).unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    drop(parent);
    settle(&mut raw);
    assert_eq!(raw.state.objects[copy as usize].zone, Zone::Graveyard);
    indexed.restore(&saved).unwrap();
    select_copy(&mut indexed, blueprint);
    settle(&mut indexed);
    assert_eq!(indexed.state.objects[copy as usize].zone, Zone::Battlefield);
    assert!(!indexed.is_creature(copy));
}

#[test]
fn a_private_hand_entry_copy_choice_does_not_expose_the_selected_card_early() {
    const PUT: CardRules = CardRules {
        effect: Effect::Spell(
            TargetPlan::NONE,
            &[Instruction::PutHandFiltered {
                filter: SearchFilter::Creature,
                skip: 0,
            }],
        ),
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[
        (Card::from_name("Grizzly Bears").unwrap(), &COPY),
        (Card::from_name("Opt").unwrap(), &PUT),
    ]);
    let mut g = scenario(Player::P0);
    let blueprint = put(&mut g, Player::P0, "Swiftfoot Boots", Zone::Battlefield);
    let copy = put(&mut g, Player::P0, "Grizzly Bears", Zone::Hand);
    let spell = put(&mut g, Player::P0, "Opt", Zone::Hand);
    cast(&mut g, spell, None);
    reach_choice(&mut g);
    let source = g.object_ref(copy);
    act(&mut g, Action::PutHandPermanent { object: source });
    assert!(matches!(g.state.pending, Pending::EntryCopy { .. }));
    assert!(g.observe(Player::P0).entry_copy.is_some());
    assert!(g.observe(Player::P1).entry_copy.is_none());
    assert!(
        g.observe(Player::P1)
            .objects
            .iter()
            .all(|view| view.id != copy)
    );
    select_copy(&mut g, blueprint);
    settle(&mut g);
    assert!(
        g.observe(Player::P1)
            .objects
            .iter()
            .any(|view| view.id == copy)
    );
}

#[test]
fn projected_copy_entry_matches_actual_view_and_later_virtual_departure_restores_physical_identity()
{
    let _scope = fixture_rules::activate(&[(Card::from_name("Grizzly Bears").unwrap(), &COPY)]);
    let mut g = scenario(Player::P0);
    let blueprint = put(
        &mut g,
        Player::P0,
        "Cultivator's Caravan",
        Zone::Battlefield,
    );
    let copy = put(&mut g, Player::P0, "Grizzly Bears", Zone::Exile);
    let mut movement = g.entry_move(copy, Zone::Battlefield, false);
    entry_copies::apply_blueprint(&mut movement, g.copiable_recipe(g.object_ref(blueprint)));
    let saved = g.snapshot();
    let world = FundingState::from_game(&g, &goal(Player::P0));
    let mut projected = None;
    let mut departed = false;
    world.visit_entry_moves(vec![movement.clone()], true, None, &mut |next| {
        projected =
            Some(characteristic_query::CharacteristicQuery::in_world(next).object_view(copy));
        next.cost_departures(&[next.reference(copy)], Zone::Graveyard, &mut |after| {
            assert_eq!(
                after.object(copy).card,
                Card::from_name("Grizzly Bears").unwrap()
            );
            assert_eq!(after.object(copy).zone, Zone::Graveyard);
            assert!(after.temporary_copy_view(after.reference(copy)).is_none());
            departed = true;
            false
        });
        false
    });
    assert!(departed);
    assert_eq!(g.snapshot(), saved);
    g.stage_entries(vec![movement], None);
    g.advance_entries();
    assert_eq!(g.object_view(copy), projected.unwrap());
    g.move_object(copy, Zone::Graveyard);
    assert_eq!(
        g.state.objects[copy as usize].card,
        Card::from_name("Grizzly Bears").unwrap()
    );
}

#[test]
fn an_as_entry_copy_can_supply_the_remaining_parent_cast_mana_and_decline_is_unfundable() {
    const SAC_MANA: CardRules = CardRules {
        effect: Effect::Creature(1, 1),
        activated: &[ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            cycling: false,
            origin: ActivationOrigin::Battlefield,
            cost: ActivationCost {
                tap: true,
                sacrifice: true,
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::AddMana(ManaProduction::Fixed([
                1, 0, 0, 0, 0, 0,
            ]))],
            sorcery: false,
            once_per_turn: false,
        }],
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[
        (Card::from_name("Grizzly Bears").unwrap(), &COPY),
        (Card::from_name("Llanowar Elves").unwrap(), &SAC_MANA),
    ]);
    let mut g = scenario(Player::P0);
    let mana = put(&mut g, Player::P0, "Llanowar Elves", Zone::Battlefield);
    let blueprint = put(
        &mut g,
        Player::P0,
        "Cultivator's Caravan",
        Zone::Battlefield,
    );
    g.state.objects[blueprint as usize].tapped = true;
    let returning = put(&mut g, Player::P0, "Grizzly Bears", Zone::Exile);
    g.state.exile_links.push(ExileLink {
        source: g.object_ref(mana),
        exiled: g.object_ref(returning),
        return_zone: Zone::Battlefield,
    });
    let target = put(&mut g, Player::P0, "Savannah Lions", Zone::Battlefield);
    let spell = put(&mut g, Player::P0, "Pacifism", Zone::Hand);
    g.state.players[0].mana = [0; 6];
    assert!(
        g.legal_actions()
            .contains(&Action::AnnounceCast { card: spell }),
        "the projected returned copy provides the second mana"
    );
    act(&mut g, Action::AnnounceCast { card: spell });
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(&mut g, Action::FinishTargets);
    }
    choose(
        &mut g,
        |action| matches!(action,Action::ActivateMana {card,ability:0,production:[1,0,0,0,0,0],..} if *card==mana),
    );
    assert!(matches!(g.state.pending, Pending::EntryCopy { .. }));
    assert_eq!(g.state.objects[mana as usize].zone, Zone::Graveyard);
    reject(&mut g, Action::ChooseEntryCopy { object: None });
    select_copy(&mut g, blueprint);
    assert_eq!(
        g.state.objects[returning as usize].card,
        Card::from_name("Cultivator's Caravan").unwrap()
    );
    choose(
        &mut g,
        |action| matches!(action,Action::ActivateMana {card,ability:0,..} if *card==returning),
    );
    pay(&mut g);
    settle(&mut g);
    assert_eq!(
        g.state.objects[spell as usize].attached_to,
        Some(g.object_ref(target))
    );
    assert_eq!(
        g.temporary_copy_view(g.object_ref(returning))
            .unwrap()
            .physical_card,
        Card::from_name("Grizzly Bears").unwrap()
    );
}

#[test]
fn foreign_as_entry_copy_that_invalidates_a_parent_cast_is_kept_with_its_mana_event() {
    const SAC_MANA: CardRules = CardRules {
        effect: Effect::Creature(1, 1),
        activated: &[ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            cycling: false,
            origin: ActivationOrigin::Battlefield,
            cost: ActivationCost {
                tap: true,
                sacrifice: true,
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::AddMana(ManaProduction::Fixed([
                1, 0, 0, 0, 0, 0,
            ]))],
            sorcery: false,
            once_per_turn: false,
        }],
        ..CardRules::EMPTY
    };
    const TAX: CardRules = CardRules {
        effect: Effect::Permanent,
        static_groups: &[StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::OpponentNoncreatureSpellTax(3)],
        }],
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[
        (Card::from_name("Grizzly Bears").unwrap(), &COPY),
        (Card::from_name("Llanowar Elves").unwrap(), &SAC_MANA),
        (Card::from_name("Swiftfoot Boots").unwrap(), &TAX),
    ]);
    let mut g = scenario(Player::P0);
    let mana = put(&mut g, Player::P0, "Llanowar Elves", Zone::Battlefield);
    put(
        &mut g,
        Player::P0,
        "Cultivator's Caravan",
        Zone::Battlefield,
    );
    let blueprint = put(&mut g, Player::P1, "Swiftfoot Boots", Zone::Battlefield);
    let returning = put(&mut g, Player::P1, "Grizzly Bears", Zone::Exile);
    g.state.exile_links.push(ExileLink {
        source: g.object_ref(mana),
        exiled: g.object_ref(returning),
        return_zone: Zone::Battlefield,
    });
    let target = put(&mut g, Player::P0, "Savannah Lions", Zone::Battlefield);
    let spell = put(&mut g, Player::P0, "Pacifism", Zone::Hand);
    g.state.players[0].mana = [0, 0, 0, 0, 0, 3];
    assert!(
        g.legal_actions()
            .contains(&Action::AnnounceCast { card: spell })
    );
    act(&mut g, Action::AnnounceCast { card: spell });
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(&mut g, Action::FinishTargets);
    }
    choose(
        &mut g,
        |action| matches!(action,Action::ActivateMana {card,ability:0,..} if *card==mana),
    );
    assert_eq!(g.actor(), Some(Player::P1));
    select_copy(&mut g, blueprint);
    assert_eq!(
        g.state.objects[spell as usize].zone,
        Zone::Hand,
        "the added tax makes the parent cast fail"
    );
    assert_eq!(g.state.objects[mana as usize].zone, Zone::Graveyard);
    assert_eq!(
        g.state.objects[returning as usize].zone,
        Zone::Battlefield,
        "the foreign decision survives failure of the parent proposal"
    );
    assert_eq!(
        g.state.objects[returning as usize].card,
        Card::from_name("Swiftfoot Boots").unwrap()
    );
    assert_eq!(
        g.temporary_copy_view(g.object_ref(returning))
            .unwrap()
            .physical_card,
        Card::from_name("Grizzly Bears").unwrap()
    );
}

#[test]
fn copied_own_off_turn_type_is_visible_to_creature_only_entry_replacements() {
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
    const TAP_CREATURES: CardRules = CardRules {
        effect: Effect::Permanent,
        static_groups: &[StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::OpponentEntryTapped(0, 1)],
        }],
        extras: &CardExtras {
            opponent_entry_tapped: 1,
            ..CardExtras::EMPTY
        },
        ..CardRules::EMPTY
    };
    let _scope = fixture_rules::activate(&[
        (Card::from_name("Spectral Sailor").unwrap(), &COPY),
        (Card::from_name("Veloheart Bike").unwrap(), &OFF_TURN),
        (Card::from_name("Swiftfoot Boots").unwrap(), &TAP_CREATURES),
    ]);
    let mut g = scenario(Player::P1);
    put(&mut g, Player::P1, "Swiftfoot Boots", Zone::Battlefield);
    let blueprint = put(&mut g, Player::P0, "Veloheart Bike", Zone::Battlefield);
    let incoming = put(&mut g, Player::P0, "Spectral Sailor", Zone::Hand);
    act(&mut g, Action::Pass);
    cast(&mut g, incoming, None);
    reach_choice(&mut g);
    select_copy(&mut g, blueprint);
    settle(&mut g);
    assert!(g.is_creature(incoming));
    assert_eq!(stats(&g, incoming), (3, 3));
    assert!(g.state.objects[incoming as usize].tapped);
}

#[test]
fn a_copied_aura_that_cannot_enter_never_changes_the_physical_card() {
    let _scope = fixture_rules::activate(&[(Card::from_name("Grizzly Bears").unwrap(), &COPY)]);
    let mut g = scenario(Player::P0);
    let incoming = put(&mut g, Player::P0, "Grizzly Bears", Zone::Hand);
    let mut movement = g.entry_move(incoming, Zone::Battlefield, false);
    entry_copies::apply_blueprint(
        &mut movement,
        CopyRecipe::new(Card::from_name("Pacifism").unwrap()),
    );
    g.stage_entries(vec![movement], None);
    g.advance_entries();
    assert_eq!(g.state.objects[incoming as usize].zone, Zone::Hand);
    assert_eq!(
        g.state.objects[incoming as usize].card,
        Card::from_name("Grizzly Bears").unwrap()
    );
    assert!(g.temporary_copy_view(g.object_ref(incoming)).is_none());
}
