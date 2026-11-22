//! Assimilate is an owned, source-independent effect on exact objects.
use crate::support::*;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};

const GRAVE_CREATURE: TargetRule = TargetRule::Object {
    zone: TargetZone::Graveyard,
    kinds: 1,
    relation: -1,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
};
const BATTLEFIELD: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[Instruction::Assimilate(Subject::Targets)],
    ),
    ..CardRules::EMPTY
};
const GRAVEYARD: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(GRAVE_CREATURE)),
        &[Instruction::Assimilate(Subject::Targets)],
    ),
    ..CardRules::EMPTY
};
const CUBE_ACTION: CardRules = CardRules {
    effect: Effect::Creature(6, 6),
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost::ZERO,
        targets: TargetPlan::Single(TargetSpec {
            rule: TargetRule::Creature,
            min: 0,
            max: 1,
        }),
        program: &[Instruction::AssimilateLinkedExileOrExileTarget],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
const BORG_OBSERVER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::OtherSubtypeEntersYou("Borg"),
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 3)],
    }],
    ..CardRules::EMPTY
};

fn rules() -> fixture_rules::Guard {
    fixture_rules::activate(&[
        (Card::from_name("Opt").unwrap(), &BATTLEFIELD),
        (Card::from_name("Unsummon").unwrap(), &GRAVEYARD),
        (Card::GrizzlyBears, &CUBE_ACTION),
        (Card::from_name("Bear Cub").unwrap(), &BORG_OBSERVER),
    ])
}

fn activate_cube(game: &mut Game, cube: CardId, target: Option<CardId>) {
    act(
        game,
        Action::AnnounceActivate {
            card: cube,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    if let Some(target) = target {
        act(
            game,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
    }
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(game, Action::FinishTargets);
    }
    pay(game);
    act(game, Action::Pass);
    act(game, Action::Pass);
}

#[test]
fn battlefield_assimilation_changes_control_counter_and_layer_four_without_resetting_identity() {
    let _rules = rules();
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let creature = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
        let original = game.object_ref(creature);
        let spell = put(&mut game, player, "Opt", Zone::Hand);
        cast(&mut game, spell, Some(Target::Card(creature)));
        settle(&mut game);
        let view = game.object_view(creature);
        assert_eq!(view.controller, player);
        assert_eq!(view.owner, player.other());
        assert_eq!(view.reference, original);
        assert_eq!(game.counter_count(creature, CounterKind::PlusOne), 1);
        assert_eq!(view.kinds & (1 | 16), 1 | 16);
        assert!(game.has_subtype(creature, "Borg"));
        assert!(!game.has_subtype(creature, "Giant"));
        assert_eq!(stats(&game, creature), (4, 4));

        // A creature land keeps its Forest subtype and land type while the
        // assimilation replaces only its creature types.
        let land = put(&mut game, player.other(), "Forest", Zone::Battlefield);
        game.change_object_form(
            game.object_ref(land),
            AnimationSpec {
                artifact: false,
                creature_types: AnimationCreatureTypes::Additional(&["Elemental"]),
                base: Some((1, 1)),
                keywords: Keywords::NONE,
                duration: AnimationDuration::Permanent,
            },
            characteristic_layers::AnimationTypeMode::AddCreature,
        );
        let spell = put(&mut game, player, "Opt", Zone::Hand);
        cast(&mut game, spell, Some(Target::Card(land)));
        settle(&mut game);
        assert_eq!(game.object_view(land).kinds & (1 | 2 | 16), 1 | 2 | 16);
        assert!(game.has_subtype(land, "Forest"));
        assert!(game.has_subtype(land, "Borg"));
        assert!(!game.has_subtype(land, "Elemental"));

        // A later layer-four effect can replace Borg; the permanent action is
        // timestamped and does not mutate copiable printed creature types.
        game.change_object_form(
            original,
            AnimationSpec {
                artifact: false,
                creature_types: AnimationCreatureTypes::Named(&["Elf"]),
                base: None,
                keywords: Keywords::NONE,
                duration: AnimationDuration::Permanent,
            },
            characteristic_layers::AnimationTypeMode::PreserveCardTypes,
        );
        assert!(game.has_subtype(creature, "Elf"));
        assert!(!game.has_subtype(creature, "Borg"));
        game.move_object(creature, Zone::Graveyard);
        assert_eq!(
            game.object_ref(creature).generation,
            original.generation + 1
        );
        assert!(game.has_subtype(creature, "Giant"));
    }
}

#[test]
fn graveyard_assimilation_enters_under_controller_with_replaceable_counter_and_borg_entry_fact() {
    let _rules = rules();
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let creature = put(&mut game, player.other(), "Hill Giant", Zone::Graveyard);
        put(&mut game, player, "Doubling Season", Zone::Battlefield);
        put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let spell = put(&mut game, player, "Unsummon", Zone::Hand);
        cast(&mut game, spell, Some(Target::Card(creature)));
        settle(&mut game);
        let view = game.object_view(creature);
        assert_eq!(view.zone, Zone::Battlefield);
        assert_eq!(view.controller, player);
        assert_eq!(view.owner, player.other());
        assert_eq!(game.counter_count(creature, CounterKind::PlusOne), 2);
        assert_eq!(view.kinds & (1 | 16), 1 | 16);
        assert!(game.has_subtype(creature, "Borg"));
        assert!(!game.has_subtype(creature, "Giant"));
        assert_eq!(game.state.players[player.index()].life, 23);
        for viewer in [player, player.other()] {
            let observed = game.observe(viewer);
            let entry = observed.objects.iter().find(|o| o.id == creature).unwrap();
            assert_eq!(entry.zone, Zone::Battlefield);
            assert_eq!(entry.controller, player);
            assert_eq!(entry.kinds & (1 | 16), 1 | 16);
        }
    }
}

#[test]
fn battlefield_counter_replacement_finishes_before_the_lasting_form() {
    let _rules = rules();
    let player = Player::P0;
    let mut game = scenario(player);
    put(&mut game, player, "Doubling Season", Zone::Battlefield);
    let creature = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    let spell = put(&mut game, player, "Opt", Zone::Hand);
    cast(&mut game, spell, Some(Target::Card(creature)));
    settle(&mut game);
    assert_eq!(game.object_view(creature).controller, player);
    assert_eq!(game.counter_count(creature, CounterKind::PlusOne), 2);
    assert!(game.has_subtype(creature, "Borg"));
    assert_eq!(stats(&game, creature), (5, 5));
}

#[test]
fn linked_exile_fallback_then_exact_public_choice_and_owned_branch_restore() {
    let _rules = rules();
    let player = Player::P0;
    let mut parent = scenario(player);
    let cube = put(&mut parent, player, "Grizzly Bears", Zone::Battlefield);
    let victim = put(&mut parent, player.other(), "Hill Giant", Zone::Battlefield);
    activate_cube(&mut parent, cube, Some(victim));
    assert_eq!(parent.object_view(victim).zone, Zone::Exile);
    let exiled = parent.object_ref(victim);
    let stale_target = put(&mut parent, player.other(), "Hill Giant", Zone::Battlefield);
    act(
        &mut parent,
        Action::AnnounceActivate {
            card: cube,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    act(
        &mut parent,
        Action::ChooseTarget {
            target: Target::Card(stale_target),
        },
    );
    pay(&mut parent);
    parent.move_object(stale_target, Zone::Hand);
    settle(&mut parent);
    assert_eq!(parent.object_view(victim).zone, Zone::Exile);
    assert_eq!(parent.object_view(stale_target).zone, Zone::Hand);
    let hidden = put(&mut parent, player.other(), "Hill Giant", Zone::Exile);
    let hidden = parent.object_ref(hidden);
    parent.link_exile_to_ability(parent.object_ref(cube), None, hidden);
    parent.remember_face_down_exile(hidden, 1 << player.other().index(), None);
    let decoy = put(
        &mut parent,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    activate_cube(&mut parent, cube, Some(decoy));
    assert_eq!(parent.state.pending, Pending::LinkedExile { player });
    let choose = Action::ChooseLinkedExile { object: exiled };
    assert!(parent.legal_actions().contains(&choose));
    reject(&mut parent, Action::ChooseLinkedExile { object: hidden });
    assert!(
        parent
            .observe(player)
            .linked_exile
            .iter()
            .any(|link| link.exiled == exiled)
    );
    assert!(
        !parent
            .observe(player)
            .linked_exile
            .iter()
            .any(|link| link.exiled == hidden)
    );
    assert!(
        parent
            .observe(player.other())
            .linked_exile
            .iter()
            .any(|link| link.exiled == hidden)
    );
    let checkpoint = parent.snapshot();
    let index = parent
        .legal_actions()
        .iter()
        .position(|a| *a == choose)
        .unwrap();
    let mut raw = parent.clone();
    let mut indexed = parent.clone();
    act(&mut raw, choose);
    indexed
        .apply_index(player, indexed.revision(), index)
        .unwrap();
    settle(&mut raw);
    settle(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
    drop(parent);
    for game in [&raw, &indexed] {
        assert_eq!(game.object_view(victim).zone, Zone::Battlefield);
        assert_eq!(game.object_view(victim).controller, player);
        assert_eq!(game.counter_count(victim, CounterKind::PlusOne), 1);
        assert!(game.has_subtype(victim, "Borg"));
        assert_eq!(game.object_view(decoy).zone, Zone::Battlefield);
    }
    indexed.restore(&checkpoint).unwrap();
    assert_eq!(indexed.snapshot(), checkpoint);
    act(&mut indexed, Action::ChooseLinkedExile { object: exiled });
    settle(&mut indexed);
    assert_eq!(indexed.snapshot(), raw.snapshot());
}

#[test]
fn illegal_and_stale_graveyard_targets_never_assimilate_a_different_incarnation() {
    let _rules = rules();
    let player = Player::P0;
    let mut game = scenario(player);
    let creature = put(&mut game, player.other(), "Hill Giant", Zone::Graveyard);
    let noncreature = put(&mut game, player.other(), "Forest", Zone::Graveyard);
    let spell = put(&mut game, player, "Unsummon", Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: spell });
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(noncreature),
        },
    );
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(creature),
        },
    );
    pay(&mut game);
    game.move_object(creature, Zone::Hand);
    settle(&mut game);
    assert_eq!(game.object_view(creature).zone, Zone::Hand);
    assert_eq!(game.object_view(creature).controller, player.other());
    assert_eq!(game.counter_count(creature, CounterKind::PlusOne), 0);
    assert!(!game.has_subtype(creature, "Borg"));
}

#[test]
fn linked_exile_is_bound_to_the_exact_source_incarnation() {
    let _rules = rules();
    let player = Player::P0;
    let mut game = scenario(player);
    let cube = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let first = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    activate_cube(&mut game, cube, Some(first));
    assert_eq!(game.object_view(first).zone, Zone::Exile);
    game.move_object(cube, Zone::Graveyard);
    game.move_object(cube, Zone::Battlefield);
    let second = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    activate_cube(&mut game, cube, Some(second));
    assert_eq!(game.object_view(second).zone, Zone::Exile);
    assert_eq!(game.object_view(first).zone, Zone::Exile);
}
