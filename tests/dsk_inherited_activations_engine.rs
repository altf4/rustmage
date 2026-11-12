//! Layer-six activation inheritance, exact ability instances and mana funding.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;
use rustmage_core::engine::characteristic_layers::AnimationTypeMode;
use rustmage_core::engine::*;
use support::*;

const PLAYER: Player = Player::P0;
const ZERO: Mana = [0; 6];
const GREEN: Mana = [0, 0, 0, 0, 1, 0];
const WHITE: Mana = [1, 0, 0, 0, 0, 0];
const COUNTER: ActivatedAbility = ActivatedAbility {
    cost: ActivationCost::ZERO,
    guard: ActivationGuard::Always,
    origin: ActivationOrigin::Battlefield,
    targets: TargetPlan::NONE,
    program: &[Instruction::PutCounters {
        subject: Subject::Source,
        kind: CounterKind::PlusOne,
        amount: 1,
    }],
    cycling: false,
    exhaust: false,
    sorcery: false,
    once_per_turn: true,
};
const MIMIC: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::InheritControlledCreatureActivations],
    }],
    ..CardRules::EMPTY
};
const DONOR: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    activated: &[COUNTER],
    ..CardRules::EMPTY
};
const LINK_DONOR: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    extras: &CardExtras {
        borrows_exiled_activations: true,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const SELF_TEMPLATE: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[COUNTER],
    extras: &CardExtras {
        self_granted_activation_templates: &[0],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const AURA: CardRules = CardRules {
    effect: Effect::Aura(TargetPlan::Single(TargetSpec::one(TargetRule::Creature))),
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
    ..CardRules::EMPTY
};
const AURA_ACTIVATIONS: CardRules = CardRules {
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[
            StaticAbilityFragment::AttachmentActivation {
                ability: 0,
                tap_attachment: false,
            },
            StaticAbilityFragment::AttachmentTrigger(1),
        ],
    }],
    activated: &[COUNTER, COUNTER],
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::AttachedCreatureBecomesTapped,
            targets: TargetPlan::NONE,
            program: &[],
        },
        TriggeredAbility {
            condition: TriggerCondition::AttachedCreatureBecomesTapped,
            targets: TargetPlan::NONE,
            program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
        },
    ],
    ..AURA
};
const AURA_MANA: CardRules = CardRules {
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::AttachmentMana(
            ManaProduction::Fixed(WHITE),
        )],
    }],
    ..AURA
};
const COPY_ABILITY: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::YourAbility)),
        &[Instruction::CopyTargetAbility { group: 0 }],
    ),
    ..CardRules::EMPTY
};

fn named(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn inherited(game: &Game, source: CardId) -> Vec<InheritedAbilityView> {
    game.observe(PLAYER)
        .inherited_abilities
        .into_iter()
        .filter(|ability| ability.source == game.object_ref(source))
        .collect()
}

fn announcement(row: InheritedAbilityView) -> Action {
    Action::AnnounceInheritedAbility {
        source: row.source,
        slot: row.slot,
        counter_payment: Counters::default(),
    }
}

#[test]
fn live_names_controllers_and_exact_donors_gate_independent_source_use() {
    let mimic = named("Grizzly Bears");
    let donor_card = named("Hill Giant");
    let _rules = fixture_rules::activate(&[(mimic, &MIMIC), (donor_card, &DONOR)]);
    let mut game = scenario(PLAYER);
    let source = spawn(&mut game, PLAYER, mimic, Zone::Battlefield);
    let donor = spawn(&mut game, PLAYER, donor_card, Zone::Battlefield);
    spawn(&mut game, PLAYER.other(), donor_card, Zone::Battlefield);
    let renamed = spawn(&mut game, PLAYER, donor_card, Zone::Battlefield);
    let mut recipe = CopyRecipe::new(donor_card);
    recipe.overrides = Some(CopyOverrides {
        name: Some(mimic.name()),
        ..CopyOverrides::NONE
    });
    game.apply_temporary_copy(game.object_ref(renamed), recipe, CopyDeadline::EndOfTurn);
    let [row]: [InheritedAbilityView; 1] = inherited(&game, source).try_into().unwrap();
    assert_eq!(row.donor, game.object_ref(donor));

    let action = announcement(row);
    let mut stolen = game.clone();
    stolen.change_controller(row.donor, PLAYER.other());
    assert!(inherited(&stolen, source).is_empty());
    reject(&mut stolen, action.clone());
    let checkpoint = game.snapshot();
    let index = game
        .legal_actions()
        .iter()
        .position(|candidate| *candidate == action)
        .unwrap();
    let mut indexed = game.clone();
    indexed.apply_index(PLAYER, game.revision(), index).unwrap();
    act(&mut game, action.clone());
    assert_eq!(game.snapshot(), indexed.snapshot());
    indexed.restore(&checkpoint).unwrap();
    indexed
        .apply_index(PLAYER, indexed.revision(), index)
        .unwrap();
    assert_eq!(game.snapshot(), indexed.snapshot());
    pay(&mut game);
    reject(&mut game, action.clone());
    game.move_object(donor, Zone::Exile);
    assert!(inherited(&game, source).is_empty());
    settle(&mut game);
    assert_eq!(game.counter_count(source, CounterKind::PlusOne), 1);
    assert_eq!(game.counter_count(donor, CounterKind::PlusOne), 0);

    game.move_object(donor, Zone::Battlefield);
    let [fresh]: [InheritedAbilityView; 1] = inherited(&game, source).try_into().unwrap();
    assert_ne!(fresh.donor, row.donor);
    assert_ne!(fresh.slot, row.slot);
    reject(&mut game, action);
    act(&mut game, announcement(fresh));
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.counter_count(source, CounterKind::PlusOne), 2);
}

#[test]
fn an_older_inheritor_depends_on_a_later_activation_grant_and_ability_loss() {
    let mimic = named("Grizzly Bears");
    let aura_card = named("Ringing Strike Mastery");
    let _rules = fixture_rules::activate(&[(mimic, &MIMIC), (aura_card, &AURA_ACTIVATIONS)]);
    let mut game = scenario(PLAYER);
    let source = spawn(&mut game, PLAYER, mimic, Zone::Battlefield);
    let donor = put(&mut game, PLAYER, "Hill Giant", Zone::Battlefield);
    let aura = spawn(&mut game, PLAYER, aura_card, Zone::Battlefield);
    game.state.objects[aura as usize].attached_to = Some(game.object_ref(donor));

    let [row]: [InheritedAbilityView; 1] = inherited(&game, source).try_into().unwrap();
    assert_eq!(
        (row.donor, row.definition, row.ability),
        (game.object_ref(donor), aura_card, 0)
    );
    let mut blanked_donor = game.clone();
    let donor_ref = blanked_donor.object_ref(donor);
    blanked_donor.remove_abilities_exact(
        &[donor_ref],
        donor_ref,
        PLAYER,
        AnimationDuration::Permanent,
    );
    assert!(inherited(&blanked_donor, source).is_empty());
    reject(&mut blanked_donor, announcement(row));
    let mut blanked_source = game.clone();
    blanked_source.remove_abilities_exact(
        &[row.source],
        row.source,
        PLAYER,
        AnimationDuration::Permanent,
    );
    assert!(inherited(&blanked_source, source).is_empty());
    reject(&mut blanked_source, announcement(row));

    act(&mut game, announcement(row));
    pay(&mut game);
    game.move_object(aura, Zone::Exile);
    settle(&mut game);
    assert_eq!(game.counter_count(source, CounterKind::PlusOne), 1);
    assert_eq!(game.counter_count(donor, CounterKind::PlusOne), 0);
}

#[test]
fn differently_named_inheritors_break_dependency_loops_in_timestamp_order() {
    let first = named("Grizzly Bears");
    let second = named("Bear Cub");
    let donor = named("Hill Giant");
    let _rules = fixture_rules::activate(&[(first, &MIMIC), (second, &MIMIC), (donor, &DONOR)]);
    let mut game = scenario(PLAYER);
    let older = spawn(&mut game, PLAYER, first, Zone::Battlefield);
    let newer = spawn(&mut game, PLAYER, second, Zone::Battlefield);
    let donor = spawn(&mut game, PLAYER, donor, Zone::Battlefield);
    let older_rows = inherited(&game, older);
    let newer_rows = inherited(&game, newer);
    assert_eq!(older_rows.len(), 1);
    assert_eq!(older_rows[0].donor, game.object_ref(donor));
    assert_eq!(newer_rows.len(), 2);
    assert!(
        newer_rows
            .iter()
            .any(|row| row.donor == game.object_ref(older))
    );
    assert!(
        newer_rows
            .iter()
            .any(|row| row.donor == game.object_ref(donor))
    );
    assert_ne!(newer_rows[0].slot, newer_rows[1].slot);
    for row in newer_rows {
        let action = announcement(row);
        act(&mut game, action.clone());
        pay(&mut game);
        settle(&mut game);
        reject(&mut game, action);
    }
    assert_eq!(game.counter_count(newer, CounterKind::PlusOne), 2);
    assert_eq!(game.counter_count(older, CounterKind::PlusOne), 0);
    assert_eq!(game.counter_count(donor, CounterKind::PlusOne), 0);
}

#[test]
fn copy_retained_self_granted_and_linked_activations_are_each_inherited() {
    let mimic = named("Grizzly Bears");
    let printed = named("Hill Giant");
    let link_donor = named("Bear Cub");
    let self_template = named("Fountainport Bell");
    let _rules = fixture_rules::activate(&[
        (mimic, &MIMIC),
        (
            printed,
            &CardRules {
                activated: &[ActivatedAbility {
                    once_per_turn: false,
                    ..COUNTER
                }],
                ..DONOR
            },
        ),
        (link_donor, &LINK_DONOR),
        (self_template, &SELF_TEMPLATE),
    ]);
    let mut game = scenario(PLAYER);
    let source = spawn(&mut game, PLAYER, mimic, Zone::Battlefield);
    let donor = spawn(&mut game, PLAYER, link_donor, Zone::Battlefield);
    let copier = put(&mut game, PLAYER, "Healer's Hawk", Zone::Battlefield);
    let exiled = spawn(&mut game, PLAYER.other(), printed, Zone::Exile);
    let mut recipe = CopyRecipe::new(named("Healer's Hawk"));
    recipe.exceptions.retained_activation = Some(RetainedCopyActivation {
        definition: printed,
        ability: 0,
    });
    game.apply_temporary_copy(game.object_ref(copier), recipe, CopyDeadline::EndOfTurn);
    let packet = game.activation_packet(PLAYER, donor, 0);
    game.remember_borrowed_exile(&packet, game.object_ref(exiled), false);
    let time = game.next_role_timestamp();
    game.grant_recipient_ability(
        game.object_ref(donor),
        self_template,
        SelfGrantedAbilityKind::Activated { ability: 0 },
        time,
        None,
    );
    let rows = inherited(&game, source);
    assert_eq!(rows.len(), 3);
    assert!(
        rows.iter()
            .any(|row| row.donor == game.object_ref(copier) && row.definition == printed)
    );
    assert!(
        rows.iter()
            .any(|row| row.donor == game.object_ref(donor) && row.definition == printed)
    );
    assert!(
        rows.iter()
            .any(|row| row.donor == game.object_ref(donor) && row.definition == self_template)
    );
    for row in rows {
        act(&mut game, announcement(row));
        pay(&mut game);
        settle(&mut game);
    }
    assert_eq!(game.counter_count(source, CounterKind::PlusOne), 3);
    game.move_object(exiled, Zone::Hand);
    assert_eq!(inherited(&game, source).len(), 2);
}

#[test]
fn equipment_grants_keep_exact_equipment_costs_and_use_the_inheritors_tap() {
    let mimic = named("Grizzly Bears");
    let _rules = fixture_rules::activate(&[(mimic, &MIMIC)]);
    let mut game = scenario(PLAYER);
    let source = spawn(&mut game, PLAYER, mimic, Zone::Battlefield);
    let donor = put(&mut game, PLAYER, "Hill Giant", Zone::Battlefield);
    let hammer = put(
        &mut game,
        PLAYER,
        "Deconstruction Hammer",
        Zone::Battlefield,
    );
    let victim = put(
        &mut game,
        PLAYER.other(),
        "Fountainport Bell",
        Zone::Battlefield,
    );
    assert!(game.attach_equipment(game.object_ref(hammer), game.object_ref(donor)));
    let [row]: [InheritedAbilityView; 1] = inherited(&game, source).try_into().unwrap();
    assert_eq!(row.donor, game.object_ref(donor));
    act(&mut game, announcement(row));
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(victim),
        },
    );
    pay(&mut game);
    assert!(game.object_view(source).tapped);
    assert!(!game.object_view(donor).tapped);
    assert_eq!(game.object_view(hammer).zone, Zone::Graveyard);
    assert!(inherited(&game, source).is_empty());
    settle(&mut game);
    assert_eq!(game.object_view(victim).zone, Zone::Graveyard);
}

#[test]
fn land_type_and_attachment_mana_use_the_inheritor_in_actual_and_projected_payment() {
    let mimic = named("Grizzly Bears");
    let aura_card = named("Ringing Strike Mastery");
    let _rules = fixture_rules::activate(&[(mimic, &MIMIC), (aura_card, &AURA_MANA)]);
    let mut parent = scenario(PLAYER);
    let source = spawn(&mut parent, PLAYER, mimic, Zone::Battlefield);
    let creature = put(&mut parent, PLAYER, "Hill Giant", Zone::Battlefield);
    let aura = spawn(&mut parent, PLAYER, aura_card, Zone::Battlefield);
    parent.state.objects[aura as usize].attached_to = Some(parent.object_ref(creature));
    let forest = put(&mut parent, PLAYER, "Forest", Zone::Battlefield);
    parent.change_object_form(
        parent.object_ref(forest),
        AnimationSpec {
            artifact: false,
            creature_types: AnimationCreatureTypes::Unchanged,
            base: Some((2, 2)),
            keywords: Keywords::NONE,
            duration: AnimationDuration::EndOfTurn,
        },
        AnimationTypeMode::AddCreature,
    );
    for donor in [creature, forest] {
        parent.state.objects[donor as usize].tapped = true;
    }
    parent.state.players[PLAYER.index()].mana = ZERO;
    let rows = inherited(&parent, source);
    assert_eq!(rows.len(), 2);
    for (donor, production, spell, target) in [
        (forest, GREEN, "Giant Growth", Some(Target::Card(source))),
        (creature, WHITE, "Healer's Hawk", None),
    ] {
        let row = rows.iter().find(|row| row.donor.card == donor).unwrap();
        let mut game = parent.clone();
        let spell = put(&mut game, PLAYER, spell, Zone::Hand);
        let action = Action::AnnounceCast { card: spell };
        let mut absent = game.clone();
        absent.move_object(donor, Zone::Exile);
        reject(&mut absent, action.clone());
        act(&mut game, action);
        if let Some(target) = target {
            act(&mut game, Action::ChooseTarget { target });
        }
        let mana = Action::ActivateInheritedMana {
            source: row.source,
            slot: row.slot,
            counter_payment: Counters::default(),
            payment: ZERO,
            production,
        };
        act(&mut game, mana.clone());
        assert!(game.object_view(source).tapped);
        assert_eq!(game.state.players[PLAYER.index()].mana, production);
        reject(&mut game, mana);
        pay(&mut game);
        settle(&mut game);
        assert_eq!(game.state.players[PLAYER.index()].mana, ZERO);
        if target.is_some() {
            assert_eq!(stats(&game, source), (5, 5));
        } else {
            assert_eq!(game.object_view(spell).zone, Zone::Battlefield);
        }
    }
}

fn animated_vacuum(game: &mut Game) -> CardId {
    let vacuum = put(game, PLAYER, "Ghost Vacuum", Zone::Battlefield);
    game.change_object_form(
        game.object_ref(vacuum),
        AnimationSpec {
            artifact: false,
            creature_types: AnimationCreatureTypes::Unchanged,
            base: Some((2, 2)),
            keywords: Keywords::NONE,
            duration: AnimationDuration::EndOfTurn,
        },
        AnimationTypeMode::AddCreature,
    );
    vacuum
}

fn inherited_link_action(game: &Game, source: CardId, donor: CardId, ability: u16) -> Action {
    announcement(
        inherited(game, source)
            .into_iter()
            .find(|row| row.donor == game.object_ref(donor) && row.ability == ability)
            .unwrap(),
    )
}

fn exile_with_inherited_link(game: &mut Game, source: CardId, donor: CardId, victim: CardId) {
    game.state.objects[source as usize].tapped = false;
    let action = inherited_link_action(game, source, donor, 0);
    act(game, action);
    act(
        game,
        Action::ChooseTarget {
            target: Target::Card(victim),
        },
    );
    pay(game);
    settle(game);
    assert_eq!(game.object_view(victim).zone, Zone::Exile);
}

#[test]
fn two_vacuums_and_a_curator_have_separate_linked_exile_through_copy_and_sacrifice() {
    let mimic = named("Grizzly Bears");
    let copier = named("Opt");
    let _rules = fixture_rules::activate(&[(mimic, &MIMIC), (copier, &COPY_ABILITY)]);
    let mut game = scenario(PLAYER);
    let source = spawn(&mut game, PLAYER, mimic, Zone::Battlefield);
    let first_vacuum = animated_vacuum(&mut game);
    let second_vacuum = animated_vacuum(&mut game);
    let curator = put(&mut game, PLAYER, "Keen-Eyed Curator", Zone::Battlefield);
    let [first, second, curated, original] =
        ["Hill Giant", "Healer's Hawk", "Bear Cub", "Savannah Lions"]
            .map(|name| put(&mut game, PLAYER.other(), name, Zone::Graveyard));
    act(
        &mut game,
        Action::AnnounceActivate {
            card: second_vacuum,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(original),
        },
    );
    pay(&mut game);
    settle(&mut game);

    exile_with_inherited_link(&mut game, source, first_vacuum, first);
    exile_with_inherited_link(&mut game, source, second_vacuum, second);
    exile_with_inherited_link(&mut game, source, curator, curated);
    let source_ref = game.object_ref(source);
    let view = game.observe(PLAYER);
    let links = view
        .linked_exile
        .iter()
        .filter(|row| row.source == source_ref)
        .collect::<Vec<_>>();
    assert_eq!(links.len(), 3);
    let cohorts = links
        .iter()
        .map(|row| row.acquired_link_cohort.unwrap())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(cohorts.len(), 3);
    let first_cohort = links[0].acquired_link_cohort;
    assert_eq!(links[0].exiled, game.object_ref(first));

    game.state.objects[source as usize].tapped = false;
    let return_first = inherited_link_action(&game, source, first_vacuum, 1);
    let checkpoint = game.snapshot();
    let index = game
        .legal_actions()
        .iter()
        .position(|action| *action == return_first)
        .unwrap();
    let mut indexed = game.clone();
    indexed
        .apply_index(PLAYER, indexed.revision(), index)
        .unwrap();
    act(&mut game, return_first);
    assert_eq!(game.snapshot(), indexed.snapshot());
    game.restore(&checkpoint).unwrap();
    game.apply_index(PLAYER, game.revision(), index).unwrap();
    assert_eq!(game.snapshot(), indexed.snapshot());
    pay(&mut game);
    assert_eq!(game.object_view(source).zone, Zone::Graveyard);
    let packet = game.state.stack.last().unwrap();
    assert_eq!(packet.source.unwrap().object, source_ref);
    assert_eq!(packet.acquired_link_cohort, first_cohort);
    let target = Target::Stack(packet.id);
    let spell = spawn(&mut game, PLAYER, copier, Zone::Hand);
    cast(&mut game, spell, Some(target));
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    let copied = game.state.stack.last().unwrap();
    assert!(copied.is_copy);
    assert_eq!(copied.acquired_link_cohort, first_cohort);
    settle(&mut game);
    assert_eq!(game.object_view(first).zone, Zone::Battlefield);
    assert_eq!(game.object_view(first).controller, PLAYER);
    assert_eq!(game.counter_count(first, CounterKind::Flying), 1);
    assert!(game.object_view(first).has_subtype("Spirit"));
    assert_eq!(stats(&game, first), (1, 1));
    for card in [second, curated, original] {
        assert_eq!(game.object_view(card).zone, Zone::Exile);
    }
}

#[test]
fn a_pair_inherited_through_another_creature_uses_its_own_linked_cohort() {
    let first = named("Grizzly Bears");
    let second = named("Bear Cub");
    let _rules = fixture_rules::activate(&[(first, &MIMIC), (second, &MIMIC)]);
    let mut game = scenario(PLAYER);
    let older = spawn(&mut game, PLAYER, first, Zone::Battlefield);
    let newer = spawn(&mut game, PLAYER, second, Zone::Battlefield);
    let vacuum = animated_vacuum(&mut game);
    let direct = put(&mut game, PLAYER.other(), "Hill Giant", Zone::Graveyard);
    let indirect = put(&mut game, PLAYER.other(), "Healer's Hawk", Zone::Graveyard);
    exile_with_inherited_link(&mut game, newer, vacuum, direct);
    exile_with_inherited_link(&mut game, newer, older, indirect);
    game.state.objects[newer as usize].tapped = false;
    let return_indirect = inherited_link_action(&game, newer, older, 1);
    act(&mut game, return_indirect);
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(newer).zone, Zone::Graveyard);
    assert_eq!(game.object_view(direct).zone, Zone::Exile);
    assert_eq!(game.object_view(indirect).zone, Zone::Battlefield);
}
