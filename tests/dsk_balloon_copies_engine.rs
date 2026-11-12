//! Copiable exceptions and exact delayed cleanup for temporary token creation.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules, one};
use rustmage_core::cards::*;
use rustmage_core::engine::characteristic_layers::AnimationTypeMode;
use rustmage_core::engine::*;
use support::*;

const PLAYER: Player = Player::P0;
const WHITE: u8 = 1;
const BLUE: u8 = 1 << 1;
const BLACK: u8 = 1 << 2;
const RED: u8 = 1 << 3;
const SOURCE: &str = "Grizzly Bears";
const ROOM: Card = Card::from_catalogue_id(176); // Anthem of Champions
const LEFT: Card = Card::from_catalogue_id(1024); // Deeproot Pilgrimage, {1}{U}
const RIGHT: Card = Card::from_catalogue_id(2072); // Hopeless Nightmare, {B}
const ROOM_RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    extras: &CardExtras {
        room: Some(&RoomSpec {
            halves: [LEFT, RIGHT],
            triggers: &[],
            static_groups: &[],
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const ROOM_HALF: CardRules = CardRules {
    effect: Effect::Permanent,
    extras: &CardExtras {
        room_parent: Some(ROOM),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const OWN_OTHER_CREATURE: TargetRule = TargetRule::Object {
    zone: TargetZone::Battlefield,
    kinds: 1,
    relation: 0,
    other: true,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
};
const CREATOR: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    keywords: Keywords::HASTE,
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            generic: 1,
            mana_component: true,
            tap: true,
            ..ActivationCost::ZERO
        },
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        targets: one(OWN_OTHER_CREATURE),
        program: &[Instruction::CreateCopyWithExceptionsAndDelayedSacrifice(
            CopyExceptions::BALLOON,
        )],
        cycling: false,
        exhaust: false,
        sorcery: true,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
const COPY: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Creature),
        &[Instruction::CreateCopyToken {
            subject: Subject::Targets,
            amount: 1,
        }],
    ),
    ..CardRules::EMPTY
};

fn card(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn create_balloon(game: &mut Game, source: CardId, target: CardId) {
    act(
        game,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    act(
        game,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    pay(game);
    settle(game);
}

fn token_views(game: &Game) -> Vec<ObjectView> {
    game.observe(PLAYER)
        .objects
        .into_iter()
        .filter(|object| object.zone == Zone::Battlefield && object.is_token())
        .collect()
}

fn unlock_door(game: &mut Game, object: ObjectRef, door: RoomDoor) {
    choose(game, |action| {
        matches!(action, Action::BeginRoomUnlock {
            object: chosen, door: side, ..
        } if *chosen == object && *side == door)
    });
    choose(game, |action| {
        matches!(action, Action::PayRoomUnlock { .. })
    });
}

#[test]
fn doubled_tokens_and_a_copy_of_a_token_keep_characteristics_with_separate_cleanup() {
    let source_card = card(SOURCE);
    let copy_spell = card("Think Twice");
    let _rules = fixture_rules::activate(&[(source_card, &CREATOR), (copy_spell, &COPY)]);
    let mut game = scenario(PLAYER);
    let source = put(&mut game, PLAYER, SOURCE, Zone::Battlefield);
    let donor = put(&mut game, PLAYER, "Oaken Siren", Zone::Battlefield);
    let doubler = put(&mut game, PLAYER, "Doubling Season", Zone::Battlefield);
    game.state.objects[source as usize].summoning_sick = true;
    game.state.objects[donor as usize].tapped = true;
    game.add_counter(donor, CounterKind::PlusOne, 1);
    create_balloon(&mut game, source, donor);
    let tokens = token_views(&game);
    assert_eq!(tokens.len(), 2);
    let created: Vec<_> = tokens.iter().map(|token| token.reference).collect();
    assert_eq!(game.observe(PLAYER).delayed_sacrifices[0].tokens, created);
    game.move_object(doubler, Zone::Exile);
    let spell = spawn(&mut game, PLAYER, copy_spell, Zone::Hand);
    cast(&mut game, spell, Some(Target::Card(created[0].card)));
    settle(&mut game);
    let copies = token_views(&game);
    assert_eq!(copies.len(), 3);
    let ordinary_copy = copies
        .iter()
        .find(|token| !created.contains(&token.reference))
        .unwrap()
        .reference;
    for token in copies {
        assert_eq!(token.name, "Oaken Siren");
        assert_eq!(token.kinds, 1 | 16);
        assert_eq!(token.colors, BLUE | RED);
        assert!(
            token.has_subtype("Balloon")
                && token.has_subtype("Siren")
                && token.has_subtype("Pirate")
        );
        assert!(
            token.keywords.contains(
                Keywords::FLYING
                    .union(Keywords::HASTE)
                    .union(Keywords::VIGILANCE)
            )
        );
        assert_eq!((token.power, token.toughness), (1, 1));
        assert!(!token.tapped);
        assert_eq!(game.counter_count(token.id, CounterKind::PlusOne), 0);
    }
    game.move_object(source, Zone::Graveyard);
    game.begin_step(Step::End);
    game.prepare_triggers();
    settle(&mut game);
    assert!(created.iter().all(|token| !game.on_battlefield(token.card)));
    assert_eq!(
        game.object_view(ordinary_copy.card).reference,
        ordinary_copy
    );
    assert!(game.on_battlefield(ordinary_copy.card));
}

#[test]
fn a_temporary_land_creature_is_copied_as_a_permanent_balloon_land_creature() {
    let _rules = fixture_rules::activate(&[(card(SOURCE), &CREATOR)]);
    let mut game = scenario(PLAYER);
    let source = put(&mut game, PLAYER, SOURCE, Zone::Battlefield);
    let land = put(&mut game, PLAYER, "Forest", Zone::Battlefield);
    game.change_object_form(
        game.object_ref(land),
        AnimationSpec {
            artifact: false,
            creature_types: AnimationCreatureTypes::Additional(&["Bear"]),
            base: Some((6, 6)),
            keywords: Keywords::TRAMPLE,
            duration: AnimationDuration::EndOfTurn,
        },
        AnimationTypeMode::AddCreature,
    );
    assert_eq!(stats(&game, land), (6, 6));
    create_balloon(&mut game, source, land);
    let token = token_views(&game).pop().unwrap();
    assert_eq!(token.kinds, 1 | 2);
    assert_eq!(token.colors, RED);
    assert!(token.has_subtype("Forest") && token.has_subtype("Balloon"));
    assert!(!token.has_subtype("Bear") && !token.keywords.contains(Keywords::TRAMPLE));
    assert_eq!((token.power, token.toughness), (1, 1));
    assert!(
        game.legal_actions()
            .contains(&Action::TapMana { card: token.id })
    );
}

#[test]
fn copied_front_and_back_each_keep_their_own_copiable_types_and_colors() {
    let _rules = fixture_rules::activate(&[(card(SOURCE), &CREATOR)]);
    let mut game = scenario(PLAYER);
    let source = put(&mut game, PLAYER, SOURCE, Zone::Battlefield);
    let donor = put(&mut game, PLAYER, "Inverted Iceberg", Zone::Battlefield);
    let donor_ref = game.object_ref(donor);
    let front = game.object_view(donor);
    game.transform_permanent(donor_ref);
    let back = game.object_view(donor);
    assert_ne!(front.card, back.card);
    assert_eq!(back.kinds & 1, 1);
    create_balloon(&mut game, source, donor);
    let token = token_views(&game).pop().unwrap();
    assert_eq!(token.card, back.card);
    assert_eq!(token.colors, back.colors | RED);
    assert_eq!(token.kinds, back.kinds | 1);
    game.transform_permanent(token.reference);
    let flipped = game.object_view(token.id);
    assert_eq!(flipped.reference, token.reference);
    assert_eq!(flipped.card, front.card);
    assert_eq!(flipped.colors, front.colors | RED);
    assert_eq!(flipped.kinds, front.kinds | 1);
    assert!(flipped.has_subtype("Balloon"));
    assert!(
        flipped
            .keywords
            .contains(Keywords::FLYING.union(Keywords::HASTE))
    );
    assert_eq!((flipped.power, flipped.toughness), (1, 1));
}

#[test]
fn the_fixed_copiable_base_takes_precedence_over_a_printed_power_toughness_cda() {
    let _rules = fixture_rules::activate(&[(card(SOURCE), &CREATOR)]);
    let mut game = scenario(PLAYER);
    let source = put(&mut game, PLAYER, SOURCE, Zone::Battlefield);
    let donor = put(&mut game, PLAYER, "Regal Bunnicorn", Zone::Battlefield);
    assert_eq!(stats(&game, donor), (2, 2));
    create_balloon(&mut game, source, donor);
    let token = token_views(&game).pop().unwrap();
    assert_eq!(stats(&game, donor), (3, 3));
    assert_eq!(stats(&game, token.id), (1, 1));
    assert!(token.has_subtype("Rabbit") && token.has_subtype("Balloon"));
    assert_eq!(token.colors, 1 | RED);
}

#[test]
fn a_balloon_room_combines_copiable_colors_with_its_own_unlocked_halves() {
    let copy_spell = card("Think Twice");
    let _rules = fixture_rules::activate(&[
        (ROOM, &ROOM_RULES),
        (LEFT, &ROOM_HALF),
        (RIGHT, &ROOM_HALF),
        (card(SOURCE), &CREATOR),
        (copy_spell, &COPY),
    ]);
    let mut game = scenario(PLAYER);
    let room = spawn(&mut game, PLAYER, ROOM, Zone::Hand);
    act(
        &mut game,
        Action::AnnounceRoom {
            card: room,
            door: RoomDoor::Left,
            flash_surcharge: false,
        },
    );
    pay(&mut game);
    settle(&mut game);
    let puppet = put(&mut game, PLAYER, "Puppet Crafting", Zone::Hand);
    cast(&mut game, puppet, Some(Target::Card(room)));
    settle(&mut game);
    assert_eq!(game.object_view(room).colors, BLUE);
    assert_eq!(stats(&game, room), (5, 5));

    let creator = put(&mut game, PLAYER, SOURCE, Zone::Battlefield);
    create_balloon(&mut game, creator, room);
    let balloon = token_views(&game).pop().unwrap();
    assert_eq!(balloon.room_doors, Some(RoomDoors::default()));
    assert_eq!(balloon.colors, RED);
    assert!(balloon.has_subtype("Room") && balloon.has_subtype("Balloon"));
    assert!(!balloon.has_subtype("Construct"));
    unlock_door(&mut game, balloon.reference, RoomDoor::Left);
    assert_eq!(game.object_view(balloon.id).colors, RED | BLUE);

    // Color composition also holds after the only characteristic-layer Aura leaves.
    game.move_object(puppet, Zone::Graveyard);
    assert_eq!(game.object_view(room).kinds, 32);
    assert_eq!(game.object_view(balloon.id).colors, RED | BLUE);
    let spell = spawn(&mut game, PLAYER, copy_spell, Zone::Hand);
    cast(&mut game, spell, Some(Target::Card(balloon.id)));
    settle(&mut game);
    let copy = token_views(&game)
        .into_iter()
        .find(|token| token.reference != balloon.reference)
        .unwrap();
    assert_eq!(copy.room_doors, Some(RoomDoors::default()));
    assert_eq!(copy.colors, RED);
    unlock_door(&mut game, copy.reference, RoomDoor::Right);
    assert_eq!(game.object_view(copy.id).colors, RED | BLACK);

    let mut recipe = game.copiable_recipe(balloon.reference);
    recipe.color_override = Some(WHITE);
    game.stage_copy_creation(PLAYER, recipe, 1);
    game.advance_entries();
    game.give_priority();
    let recolored = token_views(&game)
        .into_iter()
        .find(|token| token.reference != balloon.reference && token.reference != copy.reference)
        .unwrap();
    assert_eq!(recolored.room_doors, Some(RoomDoors::default()));
    assert_eq!(recolored.colors, WHITE | RED);
    unlock_door(&mut game, recolored.reference, RoomDoor::Left);
    assert_eq!(game.object_view(recolored.id).colors, WHITE | RED);
}

#[test]
fn absorbing_man_keeps_his_copy_name_independently_of_the_doors_and_legend_rule() {
    let copy_spell = card("Think Twice");
    let _rules = fixture_rules::activate(&[
        (ROOM, &ROOM_RULES),
        (LEFT, &ROOM_HALF),
        (RIGHT, &ROOM_HALF),
        (copy_spell, &COPY),
    ]);
    let mut game = scenario(PLAYER);
    let room = spawn(&mut game, PLAYER, ROOM, Zone::Battlefield);
    let absorber = put(&mut game, PLAYER, "Absorbing Man", Zone::Battlefield);
    let absorber_ref = game.object_ref(absorber);
    game.begin_step(Step::Main1);
    game.prepare_triggers();
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(room),
        },
    );
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(&mut game, Action::FinishTargets);
    }
    settle(&mut game);
    assert_eq!(game.object_view(room).name, "");
    assert_eq!(
        game.object_view(absorber).room_doors,
        Some(RoomDoors::default())
    );
    assert_eq!(game.object_view(absorber).name, "Absorbing Man");
    unlock_door(&mut game, absorber_ref, RoomDoor::Left);
    assert_eq!(game.object_view(absorber).name, "Absorbing Man");
    unlock_door(&mut game, absorber_ref, RoomDoor::Right);
    assert_eq!(
        game.object_view(absorber).room_doors,
        Some(RoomDoors {
            left: true,
            right: true,
        })
    );
    assert_eq!(game.object_view(absorber).name, "Absorbing Man");

    let spell = spawn(&mut game, PLAYER, copy_spell, Zone::Hand);
    cast(&mut game, spell, Some(Target::Card(absorber)));
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    assert_eq!(game.state.pending, Pending::Legend { player: PLAYER });
    let token = token_views(&game).pop().unwrap();
    assert_eq!(token.room_doors, Some(RoomDoors::default()));
    assert_eq!(token.name, "Absorbing Man");
    act(
        &mut game,
        Action::KeepLegend {
            object: absorber_ref,
        },
    );
    settle(&mut game);
    assert!(game.on_battlefield(absorber));
    assert!(!game.on_battlefield(token.id));
}
