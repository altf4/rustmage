//! Card-level manifest, face-down and exact linked-effect scenarios.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

const PLAYER: Player = Player::P0;
const OPPONENT: Player = Player::P1;
const BLUE: Mana = [0, 1, 0, 0, 0, 0];
const POOL: &str = "Bottomless Pool // Locker Room";

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn resolve_to_choice(game: &mut Game) {
    for _ in 0..24 {
        if game.state.pending != Pending::Priority || game.state.stack.is_empty() {
            return;
        }
        act(game, Action::Pass);
    }
    panic!("manifest scenario did not reach its next choice");
}

fn library_pair(game: &mut Game, player: Player, creature: &str) -> (CardId, CardId) {
    let remainder = put(game, player, "Shock", Zone::Library);
    let selected = put(game, player, creature, Zone::Library);
    (selected, remainder)
}

fn manifest(game: &mut Game, player: Player, card: CardId) -> ObjectRef {
    assert_eq!(game.actor(), Some(player));
    assert!(game.observe(player).manifest_choice.is_some());
    assert!(game.observe(player.other()).manifest_choice.is_none());
    let object = game.object_ref(card);
    act(game, Action::ChooseManifest { object });
    let view = game.object_view(card);
    assert_eq!((view.zone, view.controller), (Zone::Battlefield, player));
    assert_ne!(view.reference, object);
    assert_eq!(view.name, "");
    assert!(
        game.observe(player.other())
            .face_down_permanents
            .iter()
            .any(|row| { row.object == view.reference && row.underlying.is_none() })
    );
    view.reference
}

fn choose_target(game: &mut Game, target: Target) {
    act(game, Action::ChooseTarget { target });
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(game, Action::FinishTargets);
    }
}

fn activation(card: CardId) -> Action {
    Action::AnnounceActivate {
        card,
        ability: 0,
        counter_payment: Counters::default(),
    }
}

fn face_up(game: &mut Game, object: ObjectRef) {
    choose(
        game,
        |action| matches!(action, Action::BeginFaceUp { object: candidate, .. } if *candidate == object),
    );
    choose(game, |action| matches!(action, Action::PayFaceUp { .. }));
    assert_eq!(game.object_ref(object.card), object);
    assert!(!game.is_face_down(object));
}

#[test]
fn oculus_requires_six_exact_graveyard_cards_even_with_omniscience_and_manifests_off_turn() {
    let mut game = scenario(PLAYER);
    let oculus = put(&mut game, PLAYER, "Abhorrent Oculus", Zone::Hand);
    put(&mut game, PLAYER, "Omniscience", Zone::Battlefield);
    let (top, remainder) = library_pair(&mut game, PLAYER, "Bear Cub");
    let foreign = put(&mut game, OPPONENT, "Forest", Zone::Graveyard);
    let mut payment: Vec<_> = (0..5)
        .map(|_| put(&mut game, PLAYER, "Forest", Zone::Graveyard))
        .collect();
    let free = Action::AnnounceFree {
        card: oculus,
        face: CastFace::Normal,
        flash_surcharge: false,
    };
    reject(&mut game, free.clone());
    payment.push(put(&mut game, PLAYER, "Forest", Zone::Graveyard));
    let spare = put(&mut game, PLAYER, "Forest", Zone::Graveyard);
    game.state.players[PLAYER.index()].mana = [0; 6];
    act(&mut game, free);
    let enemy_cost = game.object_ref(foreign);
    reject(
        &mut game,
        Action::ChooseAdditionalCost { object: enemy_cost },
    );
    for &card in &payment {
        let object = game.object_ref(card);
        act(&mut game, Action::ChooseAdditionalCost { object });
    }
    pay(&mut game);
    settle(&mut game);
    assert!(
        payment
            .iter()
            .all(|&card| game.object_view(card).zone == Zone::Exile)
    );
    for card in [foreign, spare] {
        assert_eq!(game.object_view(card).zone, Zone::Graveyard);
    }
    assert_eq!(stats(&game, oculus), (5, 5));
    assert!(game.object_view(oculus).keywords.contains(Keywords::FLYING));
    game.begin_step(Step::Upkeep);
    game.prepare_triggers();
    assert!(game.state.stack.is_empty());
    game.state.active = OPPONENT;
    game.state.priority = OPPONENT;
    game.begin_step(Step::Upkeep);
    game.prepare_triggers();
    pass_pair(&mut game);
    manifest(&mut game, PLAYER, top);
    assert_eq!(game.object_view(remainder).zone, Zone::Graveyard);
}

#[test]
fn creeping_peeper_funds_only_enchantments_unlocking_or_turning_face_up() {
    let mut root = scenario(PLAYER);
    let peeper = put(&mut root, PLAYER, "Creeping Peeper", Zone::Battlefield);
    assert_eq!(stats(&root, peeper), (2, 1));
    let twins = put(&mut root, PLAYER, "Unsettling Twins", Zone::Hand);
    let (sailor, _) = library_pair(&mut root, PLAYER, "Spectral Sailor");
    cast(&mut root, twins, None);
    resolve_to_choice(&mut root);
    let hidden = manifest(&mut root, PLAYER, sailor);
    let aura = put(&mut root, PLAYER, "Unable to Scream", Zone::Hand);
    let room_spell = put(&mut root, PLAYER, POOL, Zone::Hand);
    let room = put(&mut root, PLAYER, POOL, Zone::Battlefield);
    root.open_room_on_entry(root.object_ref(room), RoomDoor::Right);
    let opt = put(&mut root, PLAYER, "Opt", Zone::Hand);
    let ordinary_activation = put(&mut root, PLAYER, "Spectral Sailor", Zone::Battlefield);
    root.state.players[PLAYER.index()].mana = [0; 6];
    let produce = Action::ActivateMana {
        card: peeper,
        ability: 0,
        counter_payment: Counters::default(),
        payment: [0; 6],
        production: BLUE,
    };

    let mut prohibited = root.clone();
    reject(&mut prohibited, Action::AnnounceCast { card: opt });
    act(&mut prohibited, produce.clone());
    let lots = prohibited.observe(PLAYER).mana_lots;
    assert_eq!(lots.len(), 1);
    assert_eq!(lots[0].amounts, BLUE);
    assert_eq!(
        lots[0].restriction,
        ManaRestrictionView::EnchantmentSpellOrUnlockDoorOrFaceUp
    );
    reject(&mut prohibited, Action::AnnounceCast { card: opt });
    let mut activation_payment = root.clone();
    activation_payment.state.players[PLAYER.index()].mana[5] = 3;
    reject(&mut activation_payment, activation(ordinary_activation));
    act(&mut activation_payment, produce.clone());
    reject(&mut activation_payment, activation(ordinary_activation));

    let mut enchantment = root.clone();
    act(&mut enchantment, Action::AnnounceCast { card: aura });
    choose_target(&mut enchantment, Target::Card(sailor));
    act(&mut enchantment, produce.clone());
    pay(&mut enchantment);
    settle(&mut enchantment);
    assert_eq!(enchantment.object_view(aura).attached_to, Some(hidden));
    assert_eq!(enchantment.state.players[PLAYER.index()].mana, [0; 6]);

    let mut room_cast = root.clone();
    act(
        &mut room_cast,
        Action::AnnounceRoom {
            card: room_spell,
            door: RoomDoor::Left,
            flash_surcharge: false,
        },
    );
    act(&mut room_cast, produce.clone());
    pay(&mut room_cast);
    resolve_to_choice(&mut room_cast);
    act(&mut room_cast, Action::FinishTargets);
    settle(&mut room_cast);
    assert_eq!(room_cast.object_view(room_spell).name, "Bottomless Pool");
    assert_eq!(room_cast.state.players[PLAYER.index()].mana, [0; 6]);

    let mut unlock = root.clone();
    let object = unlock.object_ref(room);
    let cost = ManaCost {
        colored: [0, 1, 0, 0, 0, 0],
        generic: 0,
    };
    act(
        &mut unlock,
        Action::BeginRoomUnlock {
            object,
            door: RoomDoor::Left,
            cost,
        },
    );
    act(&mut unlock, produce.clone());
    act(&mut unlock, Action::PayRoomUnlock { payment: BLUE });
    act(&mut unlock, Action::FinishTargets);
    settle(&mut unlock);
    assert_eq!(
        unlock.object_view(room).room_doors,
        Some(RoomDoors {
            left: true,
            right: true
        })
    );
    assert_eq!(unlock.state.players[PLAYER.index()].mana, [0; 6]);

    let mut turn_up = root;
    act(
        &mut turn_up,
        Action::BeginFaceUp {
            object: hidden,
            cost,
        },
    );
    act(&mut turn_up, produce);
    act(&mut turn_up, Action::PayFaceUp { payment: BLUE });
    assert_eq!(turn_up.object_ref(sailor), hidden);
    assert_eq!(turn_up.object_view(sailor).name, "Spectral Sailor");
    assert_eq!(turn_up.state.players[PLAYER.index()].mana, [0; 6]);
}

#[test]
fn windbreaker_attaches_without_targeting_then_equip_moves_its_flying_grant() {
    let mut game = scenario(PLAYER);
    let equipment = put(&mut game, PLAYER, "Cursed Windbreaker", Zone::Hand);
    let alternate = put(&mut game, PLAYER, "Llanowar Elves", Zone::Battlefield);
    let foreign = put(&mut game, OPPONENT, "Bear Cub", Zone::Battlefield);
    let removal = put(&mut game, PLAYER, "Disenchant", Zone::Hand);
    let (top, remainder) = library_pair(&mut game, PLAYER, "Grizzly Bears");
    cast(&mut game, equipment, None);
    pass_pair(&mut game);
    let mut departed = game.clone();
    cast(&mut departed, removal, Some(Target::Card(equipment)));
    resolve_to_choice(&mut departed);
    manifest(&mut departed, PLAYER, remainder);
    assert_eq!(departed.object_view(equipment).zone, Zone::Graveyard);
    assert!(
        !departed
            .object_view(remainder)
            .keywords
            .contains(Keywords::FLYING)
    );

    pass_pair(&mut game);
    let hidden = manifest(&mut game, PLAYER, top);
    assert_eq!(game.object_view(equipment).attached_to, Some(hidden));
    assert!(game.object_view(top).keywords.contains(Keywords::FLYING));
    act(&mut game, activation(equipment));
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(foreign),
        },
    );
    choose_target(&mut game, Target::Card(alternate));
    pay(&mut game);
    settle(&mut game);
    assert_eq!(
        game.object_view(equipment).attached_to,
        Some(game.object_ref(alternate))
    );
    assert!(
        game.object_view(alternate)
            .keywords
            .contains(Keywords::FLYING)
    );
    assert!(!game.object_view(top).keywords.contains(Keywords::FLYING));
}

#[test]
fn fear_of_impostors_can_flash_in_and_the_targeted_spells_controller_manifests() {
    for uncounterable in [false, true] {
        let mut game = scenario(OPPONENT);
        let fear = put(&mut game, PLAYER, "Fear of Impostors", Zone::Hand);
        let spell = put(&mut game, OPPONENT, "Bear Cub", Zone::Hand);
        let (top, remainder) = library_pair(&mut game, OPPONENT, "Llanowar Elves");
        if uncounterable {
            put(
                &mut game,
                OPPONENT,
                "Chimil, the Inner Sun",
                Zone::Battlefield,
            );
        }
        cast(&mut game, spell, None);
        act(&mut game, Action::Pass);
        assert_eq!(game.actor(), Some(PLAYER));
        cast(&mut game, fear, None);
        pass_pair(&mut game);
        choose_target(&mut game, Target::Spell(spell));
        assert_eq!(stats(&game, fear), (3, 2));
        assert!(game.object_view(fear).keywords.contains(Keywords::FLASH));
        if !uncounterable {
            let mut stale = game.clone();
            if stale.actor() != Some(OPPONENT) {
                act(&mut stale, Action::Pass);
            }
            let cancel = put(&mut stale, OPPONENT, "Cancel", Zone::Hand);
            cast(&mut stale, cancel, Some(Target::Spell(spell)));
            settle(&mut stale);
            assert!(stale.observe(OPPONENT).face_down_permanents.is_empty());
            assert_eq!(stale.object_view(top).zone, Zone::Library);
        }
        pass_pair(&mut game);
        assert_eq!(
            game.object_view(spell).zone,
            if uncounterable {
                Zone::Stack
            } else {
                Zone::Graveyard
            }
        );
        manifest(&mut game, OPPONENT, top);
        assert_eq!(game.object_view(remainder).zone, Zone::Graveyard);
        settle(&mut game);
        if uncounterable {
            assert_eq!(game.object_view(spell).zone, Zone::Battlefield);
        }
    }
}

#[test]
fn paranormal_analyst_returns_the_exact_discarded_manifest_card_from_its_event() {
    let mut game = scenario(PLAYER);
    let analyst = put(&mut game, PLAYER, "Paranormal Analyst", Zone::Battlefield);
    let twins = put(&mut game, PLAYER, "Unsettling Twins", Zone::Hand);
    let (top, remainder) = library_pair(&mut game, PLAYER, "Bear Cub");
    let unrelated = put(&mut game, PLAYER, "Forest", Zone::Graveyard);
    cast(&mut game, twins, None);
    resolve_to_choice(&mut game);
    let hidden = manifest(&mut game, PLAYER, top);
    assert_eq!(stats(&game, twins), (2, 2));
    assert_eq!(stats(&game, top), (2, 2));
    assert_eq!(stats(&game, analyst), (1, 3));
    let receipt = game.object_ref(remainder);

    let mut stale = game.clone();
    stale.move_object(remainder, Zone::Exile);
    stale.move_object(remainder, Zone::Graveyard);
    settle(&mut stale);
    assert_eq!(stale.object_view(remainder).zone, Zone::Graveyard);
    pass_pair(&mut game);
    let other = game.object_ref(unrelated);
    reject(&mut game, Action::SelectGraveyardReturn { object: other });
    act(&mut game, Action::SelectGraveyardReturn { object: receipt });
    assert_eq!(game.object_view(remainder).zone, Zone::Hand);
    assert_eq!(game.object_view(unrelated).zone, Zone::Graveyard);
    assert_eq!(game.object_ref(top), hidden);
}

#[test]
fn stay_hidden_taps_and_locks_untap_then_shuffles_its_last_attachment_before_manifest() {
    let mut game = scenario(PLAYER);
    let aura = put(&mut game, PLAYER, "Stay Hidden, Stay Silent", Zone::Hand);
    let victim = put(&mut game, OPPONENT, "Grizzly Bears", Zone::Battlefield);
    let exact = game.object_ref(victim);
    let removal = put(&mut game, PLAYER, "Disenchant", Zone::Hand);
    let (top, _) = library_pair(&mut game, PLAYER, "Bear Cub");
    cast(&mut game, aura, Some(Target::Card(victim)));
    settle(&mut game);
    assert_eq!(game.object_view(aura).attached_to, Some(exact));
    assert!(game.object_view(victim).tapped);
    let mut untap = game.clone();
    untap.state.step = Step::End;
    pass_pair(&mut untap);
    assert_eq!(untap.state.active, OPPONENT);
    assert!(untap.object_view(victim).tapped);
    game.state.step = Step::Upkeep;
    reject(&mut game, activation(aura));
    game.state.step = Step::Main1;

    act(&mut game, activation(aura));
    pay(&mut game);
    for departed in [false, true] {
        let mut branch = game.clone();
        if departed {
            cast(&mut branch, removal, Some(Target::Card(aura)));
        }
        resolve_to_choice(&mut branch);
        let shuffled = branch.object_view(victim);
        assert_eq!((shuffled.zone, shuffled.owner), (Zone::Library, OPPONENT));
        assert_ne!(shuffled.reference, exact);
        manifest(&mut branch, PLAYER, top);
    }
}

#[test]
fn twist_reality_has_separate_spell_counter_and_untargeted_manifest_modes() {
    let mut game = scenario(PLAYER);
    let twist = put(&mut game, PLAYER, "Twist Reality", Zone::Hand);
    let (top, remainder) = library_pair(&mut game, PLAYER, "Bear Cub");
    let mut counter = game.clone();
    let spell = put(&mut counter, PLAYER, "Shock", Zone::Hand);
    cast(&mut counter, spell, Some(Target::Player(OPPONENT)));
    act(&mut counter, Action::AnnounceCast { card: twist });
    act(&mut counter, Action::ChooseMode { mode: 0 });
    choose_target(&mut counter, Target::Spell(spell));
    pay(&mut counter);
    settle(&mut counter);
    assert_eq!(counter.object_view(spell).zone, Zone::Graveyard);
    assert_eq!(counter.state.players[OPPONENT.index()].life, 20);

    act(&mut game, Action::AnnounceCast { card: twist });
    reject(&mut game, Action::ChooseMode { mode: 0 });
    act(&mut game, Action::ChooseMode { mode: 1 });
    pay(&mut game);
    pass_pair(&mut game);
    manifest(&mut game, PLAYER, top);
    assert_eq!(game.object_view(remainder).zone, Zone::Graveyard);
}

#[test]
fn unable_to_scream_preserves_types_removes_abilities_and_prevents_only_its_attached_turn_up() {
    let mut game = scenario(PLAYER);
    let twins = put(&mut game, PLAYER, "Unsettling Twins", Zone::Hand);
    let (top, _) = library_pair(&mut game, PLAYER, "Spectral Sailor");
    let aura = put(&mut game, PLAYER, "Unable to Scream", Zone::Hand);
    let removal = put(&mut game, PLAYER, "Disenchant", Zone::Hand);
    cast(&mut game, twins, None);
    resolve_to_choice(&mut game);
    let object = manifest(&mut game, PLAYER, top);

    let mut face_up_first = game.clone();
    face_up(&mut face_up_first, object);
    assert!(
        face_up_first
            .object_view(top)
            .keywords
            .contains(Keywords::FLYING)
    );
    cast(&mut face_up_first, aura, Some(Target::Card(top)));
    settle(&mut face_up_first);
    let view = face_up_first.object_view(top);
    assert_eq!(view.kinds, 17);
    assert!(view.has_subtype("Spirit") && view.has_subtype("Pirate") && view.has_subtype("Toy"));
    assert_eq!(view.colors, 2);
    assert_eq!((view.power, view.toughness), (0, 2));
    assert!(!view.keywords.contains(Keywords::FLYING));
    reject(&mut face_up_first, activation(top));
    let nightmare = put(
        &mut face_up_first,
        OPPONENT,
        "Fear of Impostors",
        Zone::Battlefield,
    );
    let second_aura = put(&mut face_up_first, PLAYER, "Unable to Scream", Zone::Hand);
    cast(
        &mut face_up_first,
        second_aura,
        Some(Target::Card(nightmare)),
    );
    settle(&mut face_up_first);
    let view = face_up_first.object_view(nightmare);
    assert_eq!(view.kinds, 1 | 16 | 32);
    assert!(view.has_subtype("Nightmare") && view.has_subtype("Toy"));
    assert_eq!((view.power, view.toughness), (0, 2));

    cast(&mut game, aura, Some(Target::Card(top)));
    settle(&mut game);
    assert_eq!(game.object_view(aura).attached_to, Some(object));
    assert_eq!(stats(&game, top), (0, 2));
    reject(
        &mut game,
        Action::BeginFaceUp {
            object,
            cost: ManaCost {
                colored: [0, 1, 0, 0, 0, 0],
                generic: 0,
            },
        },
    );
    cast(&mut game, removal, Some(Target::Card(aura)));
    settle(&mut game);
    assert_eq!(stats(&game, top), (2, 2));
    face_up(&mut game, object);
    assert_eq!(game.object_view(top).name, "Spectral Sailor");
}

#[test]
fn hovership_exiles_by_toughness_crews_and_its_departure_uses_the_exiled_cards_owner() {
    let mut game = scenario(PLAYER);
    let ship = put(&mut game, PLAYER, "Unidentified Hovership", Zone::Hand);
    let five = put(&mut game, OPPONENT, "Abhorrent Oculus", Zone::Battlefield);
    let six = put(&mut game, OPPONENT, "Rampaging Baloths", Zone::Battlefield);
    let crewer = put(&mut game, PLAYER, "Healer's Hawk", Zone::Battlefield);
    let removal = put(&mut game, PLAYER, "Disenchant", Zone::Hand);
    let (top, remainder) = library_pair(&mut game, OPPONENT, "Bear Cub");
    game.state.objects[crewer as usize].summoning_sick = true;
    let stolen = game.object_ref(five);
    game.record_control(stolen, PLAYER, None);
    cast(&mut game, ship, None);
    resolve_to_choice(&mut game);
    let mut declined = game.clone();
    act(&mut declined, Action::FinishTargets);
    settle(&mut declined);
    cast(&mut declined, removal, Some(Target::Card(ship)));
    settle(&mut declined);
    assert!(declined.observe(OPPONENT).face_down_permanents.is_empty());
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(six),
        },
    );
    choose_target(&mut game, Target::Card(five));
    settle(&mut game);
    assert_eq!(game.object_view(five).zone, Zone::Exile);
    assert_eq!(game.object_view(six).zone, Zone::Battlefield);
    assert!(!game.object_view(ship).is_creature());

    act(&mut game, activation(ship));
    let object = game.object_ref(crewer);
    act(&mut game, Action::ChooseCrewer { object });
    act(&mut game, Action::FinishCrew);
    settle(&mut game);
    let crewed = game.object_view(ship);
    assert!(crewed.is_creature() && crewed.keywords.contains(Keywords::FLYING));
    assert_eq!((crewed.power, crewed.toughness), (2, 2));
    assert!(game.object_view(crewer).tapped);

    let mut stale = game.clone();
    stale.move_object(five, Zone::Graveyard);
    stale.move_object(five, Zone::Exile);
    cast(&mut stale, removal, Some(Target::Card(ship)));
    settle(&mut stale);
    assert!(stale.observe(OPPONENT).face_down_permanents.is_empty());
    cast(&mut game, removal, Some(Target::Card(ship)));
    resolve_to_choice(&mut game);
    manifest(&mut game, OPPONENT, top);
    assert_eq!(game.object_view(remainder).zone, Zone::Graveyard);
    assert_eq!(game.object_view(five).zone, Zone::Exile);
}

#[test]
fn unnerving_grasp_can_manifest_with_no_target_but_an_illegal_chosen_target_fizzles() {
    let mut game = scenario(PLAYER);
    let grasp = put(&mut game, PLAYER, "Unnerving Grasp", Zone::Hand);
    let owned_by_opponent = put(&mut game, OPPONENT, "Bear Cub", Zone::Battlefield);
    let target_ref = game.object_ref(owned_by_opponent);
    game.record_control(target_ref, PLAYER, None);
    let land = put(&mut game, OPPONENT, "Island", Zone::Battlefield);
    let (top, _) = library_pair(&mut game, PLAYER, "Grizzly Bears");
    act(&mut game, Action::AnnounceCast { card: grasp });
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(land),
        },
    );
    let mut no_target = game.clone();
    act(&mut no_target, Action::FinishTargets);
    pay(&mut no_target);
    pass_pair(&mut no_target);
    manifest(&mut no_target, PLAYER, top);
    assert_eq!(no_target.object_ref(owned_by_opponent), target_ref);

    choose_target(&mut game, Target::Card(owned_by_opponent));
    pay(&mut game);
    let mut stale = game.clone();
    stale.move_object(owned_by_opponent, Zone::Hand);
    stale.move_object(owned_by_opponent, Zone::Battlefield);
    settle(&mut stale);
    assert!(stale.observe(PLAYER).face_down_permanents.is_empty());
    pass_pair(&mut game);
    let returned = game.object_view(owned_by_opponent);
    assert_eq!((returned.zone, returned.owner), (Zone::Hand, OPPONENT));
    manifest(&mut game, PLAYER, top);
}

#[test]
fn unwanted_remake_uses_the_live_target_controller_even_if_destruction_fails() {
    for indestructible in [false, true] {
        let mut game = scenario(PLAYER);
        let remake = put(&mut game, PLAYER, "Unwanted Remake", Zone::Hand);
        let victim = put(&mut game, PLAYER, "Bear Cub", Zone::Battlefield);
        let object = game.object_ref(victim);
        let (top, remainder) = library_pair(&mut game, OPPONENT, "Grizzly Bears");
        cast(&mut game, remake, Some(Target::Card(victim)));
        game.record_control(object, OPPONENT, None);
        if indestructible {
            game.state.objects[victim as usize].gained_keywords = Keywords::INDESTRUCTIBLE;
        }
        let mut stale = game.clone();
        stale.move_object(victim, Zone::Exile);
        settle(&mut stale);
        assert!(stale.observe(OPPONENT).face_down_permanents.is_empty());
        pass_pair(&mut game);
        assert_eq!(
            game.object_view(victim).zone,
            if indestructible {
                Zone::Battlefield
            } else {
                Zone::Graveyard
            }
        );
        manifest(&mut game, OPPONENT, top);
        assert_eq!(game.object_view(remainder).zone, Zone::Graveyard);
    }
}
