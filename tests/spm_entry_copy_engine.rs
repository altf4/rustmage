//! As-entry copy exceptions and the independent trigger linked to a graveyard copy.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;
use rustmage_core::engine::ability_world::CharacteristicWorld;
use rustmage_core::engine::funding::{FundingGoal, FundingState};
use rustmage_core::engine::*;
use support::*;

const BATTLEFIELD_COPY: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    extras: &CardExtras {
        entry_copy: Some(EntryCopySpec {
            filters: &[BattlefieldFilter {
                kinds: 1,
                relation: 0,
                nonland: false,
                tapped: None,
                without: Keywords::NONE,
                subtype: None,
                max_mana: u32::MAX,
            }],
            optional: true,
        }),
        entry_copy_options: Some(&EntryCopyOptions {
            name: Some("Bear Cub"),
            ..EntryCopyOptions::DEFAULT
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const GRAVEYARD_COPY: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    extras: &CardExtras {
        entry_copy: Some(EntryCopySpec {
            filters: &[],
            optional: true,
        }),
        entry_copy_options: Some(&EntryCopyOptions {
            source: EntryCopySource::GraveyardCreature { exile_trigger: 0 },
            name: Some("Ghalta, Primal Hunger"),
            exceptions: CopyExceptions {
                base: Some((4, 4)),
                spider_human_hero: true,
                ..CopyExceptions::NONE
            },
        }),
        ..CardExtras::EMPTY
    },
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Reflexive,
        targets: TargetPlan::NONE,
        program: &[Instruction::ExileEventGraveyardCard],
    }],
    ..CardRules::EMPTY
};
const COPIED_ETB: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    entry: EntryTapped::Always,
    enters: InitialCounters {
        plus_one: 2,
        ..CardRules::EMPTY.enters
    },
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEntersIfCast,
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 3)],
    }],
    ..CardRules::EMPTY
};

fn definition(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn select(game: &mut Game, object: Option<ObjectRef>) {
    act(game, Action::ChooseEntryCopy { object });
}

fn wait_copy(game: &mut Game) {
    for _ in 0..8 {
        if matches!(game.state.pending, Pending::EntryCopy { .. }) {
            return;
        }
        assert_eq!(game.state.pending, Pending::Priority);
        act(game, Action::Pass);
    }
    panic!("missing as-entry copy choice");
}

fn cast_copy(game: &mut Game, name: &str) -> CardId {
    let entrant = put(game, Player::P0, name, Zone::Hand);
    cast(game, entrant, None);
    wait_copy(game);
    entrant
}

#[test]
fn battlefield_choice_keeps_its_name_and_the_chosen_creatures_supertypes() {
    let _rules = fixture_rules::activate(&[(definition("Bear Cub"), &BATTLEFIELD_COPY)]);
    let mut game = scenario(Player::P0);
    let protected = put(&mut game, Player::P0, "Progenitus", Zone::Battlefield);
    let ordinary = put(&mut game, Player::P0, "Llanowar Elves", Zone::Battlefield);
    let enemy = put(&mut game, Player::P1, "Savannah Lions", Zone::Battlefield);
    game.state.objects[ordinary as usize].counters.plus_one = 2;
    let entrant = cast_copy(&mut game, "Bear Cub");
    let protected = game.object_ref(protected);
    let ordinary = game.object_ref(ordinary);
    let enemy = game.object_ref(enemy);
    reject(
        &mut game,
        Action::ChooseEntryCopy {
            object: Some(enemy),
        },
    );
    let before = game.snapshot();

    select(&mut game, Some(protected));
    settle(&mut game);
    let result = game.object_view(entrant);
    assert_eq!(result.name, "Bear Cub");
    assert!(result.is_legendary());
    assert!(result.keywords.contains(Keywords::PROTECTION_EVERYTHING));
    assert_eq!(stats(&game, entrant), (10, 10));

    game.restore(&before).unwrap();
    select(&mut game, Some(ordinary));
    settle(&mut game);
    let result = game.object_view(entrant);
    assert_eq!(result.name, "Bear Cub");
    assert!(!result.is_legendary());
    assert_eq!(stats(&game, entrant), (1, 1));
    assert!(result.has_subtype("Elf"));

    let follower = cast_copy(&mut game, "Waxen Shapethief");
    let recipient = game.object_ref(entrant);
    select(&mut game, Some(recipient));
    settle(&mut game);
    assert_eq!(game.object_view(follower).name, "Bear Cub");
    assert!(game.object_view(follower).has_subtype("Elf"));

    game.restore(&before).unwrap();
    select(&mut game, None);
    settle(&mut game);
    assert_eq!(game.object_view(entrant).card, definition("Bear Cub"));
    assert_eq!(stats(&game, entrant), (1, 1));
}

#[test]
fn graveyard_choice_preserves_copied_entry_rules_and_queues_an_exact_nontargeted_trigger() {
    let _rules = fixture_rules::activate(&[
        (definition("Ghalta, Primal Hunger"), &GRAVEYARD_COPY),
        (definition("Helpful Hunter"), &COPIED_ETB),
    ]);
    let mut game = scenario(Player::P0);
    let donor = put(&mut game, Player::P1, "Helpful Hunter", Zone::Graveyard);
    let own = put(&mut game, Player::P0, "Progenitus", Zone::Graveyard);
    let instant = put(&mut game, Player::P0, "Opt", Zone::Graveyard);
    let exiled = put(&mut game, Player::P1, "Savannah Lions", Zone::Exile);
    let entrant = cast_copy(&mut game, "Ghalta, Primal Hunger");
    let original = game.object_ref(donor);
    let own = game.object_ref(own);
    for viewer in [Player::P0, Player::P1] {
        let choice = game.observe(viewer).entry_copy.unwrap();
        assert_eq!(choice.candidates, vec![original, own]);
        assert!(choice.optional);
    }
    for id in [instant, exiled] {
        let invalid = game.object_ref(id);
        reject(
            &mut game,
            Action::ChooseEntryCopy {
                object: Some(invalid),
            },
        );
    }
    let before = game.snapshot();
    let action = Action::ChooseEntryCopy {
        object: Some(original),
    };
    let mut indexed = game.clone();
    let index = indexed
        .legal_actions()
        .iter()
        .position(|a| *a == action)
        .unwrap();
    indexed
        .apply_index(Player::P0, indexed.revision(), index)
        .unwrap();
    act(&mut game, action);
    assert_eq!(game.snapshot(), indexed.snapshot());

    let result = game.object_view(entrant);
    assert_eq!(result.name, "Ghalta, Primal Hunger");
    assert!(!result.is_legendary());
    assert!(result.tapped);
    assert_eq!(stats(&game, entrant), (6, 6));
    for subtype in ["Cat", "Spider", "Human", "Hero"] {
        assert!(result.has_subtype(subtype), "missing {subtype}");
    }
    assert_eq!(game.object_view(donor).zone, Zone::Graveyard);
    assert_eq!(game.state.ordering_triggers.len(), 2);
    let linked = game
        .state
        .ordering_triggers
        .iter()
        .find(|item| {
            item.source
                .is_some_and(|source| source.card == definition("Ghalta, Primal Hunger"))
        })
        .unwrap();
    assert_eq!(linked.event_object.as_ref().unwrap().object, original);
    assert!(linked.targets.is_empty());
    let id = linked.id;
    let triggered = game.snapshot();

    act(&mut game, Action::PutTrigger { id });
    settle(&mut game);
    assert_eq!(game.state.players[0].life, 23);
    assert_eq!(game.object_view(donor).zone, Zone::Exile);

    let follower = cast_copy(&mut game, "Waxen Shapethief");
    let blueprint = game.object_ref(entrant);
    select(&mut game, Some(blueprint));
    settle(&mut game);
    let result = game.object_view(follower);
    assert_eq!(result.name, "Ghalta, Primal Hunger");
    assert_eq!(stats(&game, follower), (6, 6));
    assert!(result.has_subtype("Hero"));

    game.restore(&triggered).unwrap();
    act(&mut game, Action::PutTrigger { id });
    game.move_object(entrant, Zone::Hand);
    game.move_object(donor, Zone::Exile);
    game.move_object(donor, Zone::Graveyard);
    assert_ne!(game.object_ref(donor), original);
    settle(&mut game);
    assert_eq!(game.state.players[0].life, 23);
    assert_eq!(game.object_view(donor).zone, Zone::Graveyard);

    game.restore(&before).unwrap();
    select(&mut game, Some(own));
    settle(&mut game);
    let result = game.object_view(entrant);
    assert!(result.is_legendary());
    assert!(result.has_subtype("Hydra"));
    assert!(result.has_subtype("Spider"));
    assert_eq!(stats(&game, entrant), (4, 4));

    game.restore(&before).unwrap();
    select(&mut game, None);
    settle(&mut game);
    assert!(game.object_view(entrant).is_legendary());
    assert_eq!(game.object_view(donor).zone, Zone::Graveyard);
}

#[test]
fn a_simultaneous_graveyard_entrant_is_a_blueprint_in_game_and_funding() {
    let _rules = fixture_rules::activate(&[
        (definition("Ghalta, Primal Hunger"), &GRAVEYARD_COPY),
        (definition("Helpful Hunter"), &COPIED_ETB),
    ]);
    let mut game = scenario(Player::P0);
    let entrant = put(
        &mut game,
        Player::P0,
        "Ghalta, Primal Hunger",
        Zone::Graveyard,
    );
    let donor = put(&mut game, Player::P1, "Helpful Hunter", Zone::Graveyard);
    let selected = game.object_ref(donor);
    let moves = vec![
        game.entry_move(entrant, Zone::Battlefield, false),
        game.entry_move(donor, Zone::Battlefield, false),
    ];
    let goal = FundingGoal {
        actor: Player::P0,
        required: [0; 6],
        generic: 0,
        reserve: None,
        source: None,
    };
    let funding = FundingState::from_game(&game, &goal);
    let mut projection = None;
    funding.visit_entry_moves(moves.clone(), true, None, &mut |world| {
        if world.object(entrant).card == definition("Helpful Hunter") {
            assert_eq!(world.object(donor).zone, Zone::Battlefield);
            projection = Some(
                characteristic_query::CharacteristicQuery::in_world(world).object_view(entrant),
            );
            return true;
        }
        false
    });

    game.stage_entries(moves, None);
    assert!(game.advance_entries());
    select(&mut game, Some(selected));
    settle(&mut game);
    let actual = game.object_view(entrant);
    let projected = projection.unwrap();
    assert_eq!(actual.name, projected.name);
    assert_eq!(
        (actual.power, actual.toughness),
        (projected.power, projected.toughness)
    );
    assert_eq!(actual.granted_subtypes, projected.granted_subtypes);
    assert!(actual.has_subtype("Hero"));
    assert_eq!(stats(&game, entrant), (6, 6));
    assert_eq!(game.object_view(donor).zone, Zone::Battlefield);
    assert_eq!(game.state.players[0].life, 20);
}

#[test]
fn a_later_as_entry_copy_displaces_the_first_copys_linked_trigger() {
    let _rules = fixture_rules::activate(&[
        (definition("Ghalta, Primal Hunger"), &GRAVEYARD_COPY),
        (definition("Bear Cub"), &BATTLEFIELD_COPY),
    ]);
    let mut game = scenario(Player::P0);
    let graveyard = put(&mut game, Player::P1, "Bear Cub", Zone::Graveyard);
    let battlefield = put(&mut game, Player::P0, "Savannah Lions", Zone::Battlefield);
    let entrant = cast_copy(&mut game, "Ghalta, Primal Hunger");
    let first = game.object_ref(graveyard);
    let second = game.object_ref(battlefield);
    select(&mut game, Some(first));
    assert!(matches!(game.state.pending, Pending::EntryCopy { .. }));
    select(&mut game, Some(second));
    settle(&mut game);
    assert_eq!(game.object_view(graveyard).zone, Zone::Graveyard);
    let result = game.object_view(entrant);
    assert_eq!(result.name, "Bear Cub");
    assert!(!result.has_subtype("Spider"));
    assert_eq!(stats(&game, entrant), (2, 1));
}

#[test]
fn suppression_of_creature_etbs_leaves_the_linked_exile_trigger() {
    let _rules = fixture_rules::activate(&[
        (definition("Ghalta, Primal Hunger"), &GRAVEYARD_COPY),
        (definition("Helpful Hunter"), &COPIED_ETB),
    ]);
    let mut game = scenario(Player::P0);
    put(&mut game, Player::P1, "Torpor Orb", Zone::Battlefield);
    let donor = put(&mut game, Player::P1, "Helpful Hunter", Zone::Graveyard);
    let entrant = cast_copy(&mut game, "Ghalta, Primal Hunger");
    let selected = game.object_ref(donor);
    select(&mut game, Some(selected));
    game.move_object(entrant, Zone::Hand);
    settle(&mut game);
    assert_eq!(game.object_view(donor).zone, Zone::Exile);
    assert_eq!(game.state.players[0].life, 20);
}
