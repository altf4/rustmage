//! Paid scenarios for the green and multicolored Duskmourn token and creature cards.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

const BLUE: u8 = 1 << 1;
const BLACK: u8 = 1 << 2;
const RED: u8 = 1 << 3;
const GREEN: u8 = 1 << 4;
const GREMLIN: Card = Card::from_catalogue_id(5316);
const INSECT: Card = Card::from_catalogue_id(5244);
const SPIRIT: Card = Card::from_catalogue_id(5675);
const SHARD: Card = Card::from_catalogue_id(5676);
const EVERYWHERE: Card = Card::from_catalogue_id(5677);
const SPIDER: Card = Card::from_catalogue_id(5678);
const PRIMO: Card = Card::from_catalogue_id(5679);
const SMOKY_ROOM: &str = "Smoky Lounge // Misty Salon";

fn progress_until(game: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..120 {
        if ready(game) {
            return;
        }
        choose(game, |action| {
            matches!(
                action,
                Action::Pass
                    | Action::FinishAttackers
                    | Action::FinishBlockers
                    | Action::PutTrigger { .. }
            )
        });
    }
    panic!("scenario exceeded its action budget");
}

fn settle_ordered(game: &mut Game) {
    progress_until(game, |game| {
        game.state.pending == Pending::Priority
            && game.state.stack.is_empty()
            && game.state.resolving.is_none()
    });
}

fn next_main(game: &mut Game, player: Player) {
    let turn = game.state.turn;
    progress_until(game, |game| {
        game.state.turn > turn && game.state.active == player && game.state.step == Step::Main1
    });
}

fn attack(game: &mut Game, card: CardId) {
    progress_until(game, |game| game.state.step == Step::Attackers);
    act(game, Action::Attack { card });
    act(game, Action::FinishAttackers);
}

fn cast_named(game: &mut Game, player: Player, name: &str) -> CardId {
    let card = put(game, player, name, Zone::Hand);
    cast(game, card, None);
    card
}

fn controlled_tokens(game: &Game, player: Player, card: Card) -> Vec<ObjectView> {
    game.observe(player)
        .objects
        .into_iter()
        .filter(|view| {
            view.zone == Zone::Battlefield
                && view.controller == player
                && view.card == card
                && view.is_token()
        })
        .collect()
}

fn activation(card: CardId, ability: u16) -> Action {
    Action::AnnounceActivate {
        card,
        ability,
        counter_payment: Counters::default(),
    }
}

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

fn finish_targets_and_pay(game: &mut Game) {
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(game, Action::FinishTargets);
    }
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(game);
    }
}

fn choose_cast_route(game: &mut Game, player: Player, card: CardId, impending: bool) {
    act(game, Action::AnnounceCast { card });
    if let Some(option) = game
        .observe(player)
        .cost_choices
        .into_iter()
        .find(|option| option.impending == impending)
    {
        act(game, Action::ChooseSpellCost { cost: option.index });
    }
    pay(game);
    assert!(
        game.state.stack[0]
            .cast
            .is_some_and(|cast| cast.impending == impending)
    );
    settle(game);
}

#[test]
fn hauntwoods_creates_everywhere_on_both_cast_routes_and_on_attack() {
    let player = Player::P0;
    let mut root = scenario(player);
    let overlord = put(&mut root, player, "Overlord of the Hauntwoods", Zone::Hand);

    let mut impending = root.clone();
    impending.state.players[player.index()].mana = [0, 0, 0, 0, 2, 1];
    choose_cast_route(&mut impending, player, overlord, true);
    let view = impending.object_view(overlord);
    assert_eq!(view.kinds, 32);
    assert!(view.cast_for_impending);
    assert_eq!(impending.counter_count(overlord, CounterKind::Time), 4);
    assert_eq!(controlled_tokens(&impending, player, EVERYWHERE).len(), 1);
    progress_until(&mut impending, |game| game.state.step == Step::End);
    settle(&mut impending);
    assert_eq!(impending.counter_count(overlord, CounterKind::Time), 3);
    assert!(!impending.object_view(overlord).is_creature());

    root.state.players[player.index()].mana = [0, 0, 0, 0, 2, 3];
    choose_cast_route(&mut root, player, overlord, false);
    assert_eq!(stats(&root, overlord), (6, 5));
    assert_eq!(root.counter_count(overlord, CounterKind::Time), 0);
    let tokens = controlled_tokens(&root, player, EVERYWHERE);
    assert_eq!(tokens.len(), 1);
    let land = &tokens[0];
    assert_eq!(land.name, "Everywhere");
    assert_eq!((land.kinds, land.colors), (2, 0));
    assert!(!land.card.is_basic_land());
    assert!(land.tapped);
    for subtype in ["Plains", "Island", "Swamp", "Mountain", "Forest"] {
        assert!(land.has_subtype(subtype), "missing {subtype}");
    }

    next_main(&mut root, player);
    let mut colors = [false; 6];
    for action in root.legal_actions() {
        match action {
            Action::TapMana { card } if card == land.id => colors[0] = true,
            Action::TapGrantedMana {
                card, production, ..
            } if card == land.id => {
                for (color, amount) in production.into_iter().enumerate() {
                    colors[color] |= amount == 1;
                }
            }
            _ => {}
        }
    }
    assert_eq!(colors, [true, true, true, true, true, false]);
    choose(&mut root, |action| {
        matches!(action, Action::TapGrantedMana { card, production, .. }
            if *card == land.id && *production == [0, 1, 0, 0, 0, 0])
    });
    assert!(root.object_view(land.id).tapped);
    assert_eq!(root.state.players[player.index()].mana, [0, 1, 0, 0, 0, 0]);
    attack(&mut root, overlord);
    settle_ordered(&mut root);
    let tokens = controlled_tokens(&root, player, EVERYWHERE);
    assert_eq!(tokens.len(), 2);
    assert!(tokens.iter().all(|token| token.tapped));
}

#[test]
fn twitching_doll_funds_a_spell_and_uses_all_departure_counters_for_spiders() {
    let player = Player::P1;
    let mut game = scenario(player);
    put(&mut game, player, "Doubling Season", Zone::Battlefield);
    let doll = cast_named(&mut game, player, "Twitching Doll");
    settle(&mut game);
    assert_eq!(stats(&game, doll), (2, 2));
    reject(&mut game, activation(doll, 1));

    next_main(&mut game, player);
    let spell = put(&mut game, player, "Bear Cub", Zone::Hand);
    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 1];
    act(&mut game, Action::AnnounceCast { card: spell });
    act(
        &mut game,
        Action::ActivateMana {
            card: doll,
            ability: 0,
            counter_payment: Counters::default(),
            payment: [0; 6],
            production: [0, 0, 0, 0, 1, 0],
        },
    );
    assert!(game.state.stack.is_empty());
    assert!(game.object_view(doll).tapped);
    assert_eq!(game.counter_count(doll, CounterKind::Nest), 2);
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(spell).zone, Zone::Battlefield);
    reject(&mut game, activation(doll, 1));

    next_main(&mut game, player);
    game.add_counter(doll, CounterKind::PlusOne, 1);
    act(&mut game, activation(doll, 1));
    finish_targets_and_pay(&mut game);
    assert_eq!(game.object_view(doll).zone, Zone::Graveyard);
    settle(&mut game);
    let spiders = controlled_tokens(&game, player, SPIDER);
    assert_eq!(spiders.len(), 6);
    for token in spiders {
        assert_eq!((token.power, token.toughness, token.colors), (2, 2, GREEN));
        assert!(token.has_subtype("Spider") && token.keywords.contains(Keywords::REACH));
    }
}

#[test]
fn zimone_requires_land_entry_and_puts_the_live_prime_count_on_primo() {
    let player = Player::P0;
    let mut game = scenario(player);
    cast_named(&mut game, player, "Zimone, All-Questioning");
    settle(&mut game);
    for _ in 0..36 {
        put(&mut game, player, "Forest", Zone::Battlefield);
    }
    let land = put(&mut game, player, "Island", Zone::Hand);
    let mut no_entry = game.clone();
    put(&mut no_entry, player, "Island", Zone::Battlefield);
    progress_until(&mut no_entry, |game| game.state.step == Step::End);
    assert!(no_entry.state.stack.is_empty());
    assert!(controlled_tokens(&no_entry, player, PRIMO).is_empty());

    act(&mut game, Action::PlayLand { card: land });
    progress_until(&mut game, |game| game.state.step == Step::End);
    assert_eq!(game.state.stack.len(), 1);
    settle(&mut game);
    let primo = controlled_tokens(&game, player, PRIMO);
    assert_eq!(primo.len(), 1);
    assert_eq!(primo[0].name, "Primo, the Indivisible");
    assert!(primo[0].is_legendary() && primo[0].has_subtype("Fractal"));
    assert_eq!(primo[0].colors, GREEN | BLUE);
    assert_eq!((primo[0].power, primo[0].toughness), (37, 37));
    assert_eq!(game.counter_count(primo[0].id, CounterKind::PlusOne), 37);
}

#[test]
fn broodspinner_surveils_privately_then_counts_its_own_sacrifice() {
    let player = Player::P1;
    let mut game = scenario(player);
    put(&mut game, player, "Forest", Zone::Graveyard);
    put(&mut game, player, "Short Bow", Zone::Graveyard);
    let kept = put(&mut game, player, "Mountain", Zone::Library);
    let milled = put(&mut game, player, "Shock", Zone::Library);
    let broodspinner = cast_named(&mut game, player, "Broodspinner");
    progress_until(&mut game, |game| {
        game.observe(player).library_choice.is_some()
    });
    let choice = game.observe(player).library_choice.unwrap();
    assert!(choice.surveil);
    assert_eq!(choice.remaining.len(), 2);
    assert!(game.observe(player.other()).library_choice.is_none());
    act(
        &mut game,
        Action::ArrangeLibrary {
            card: milled,
            destination: LibraryDestination::Graveyard,
        },
    );
    act(
        &mut game,
        Action::ArrangeLibrary {
            card: kept,
            destination: LibraryDestination::Top,
        },
    );
    settle(&mut game);
    assert_eq!(game.object_view(milled).zone, Zone::Graveyard);
    assert_eq!(
        game.state.players[player.index()].library.last(),
        Some(&kept)
    );
    assert!(
        game.object_view(broodspinner)
            .keywords
            .contains(Keywords::REACH)
    );
    reject(&mut game, activation(broodspinner, 0));

    next_main(&mut game, player);
    game.state.players[player.index()].mana = [0, 0, 1, 0, 1, 4];
    act(&mut game, activation(broodspinner, 0));
    finish_targets_and_pay(&mut game);
    assert_eq!(game.object_view(broodspinner).zone, Zone::Graveyard);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    settle(&mut game);
    let tokens = controlled_tokens(&game, player, INSECT);
    assert_eq!(tokens.len(), 4);
    for token in tokens {
        assert_eq!(
            (token.power, token.toughness, token.colors),
            (1, 1, BLACK | GREEN)
        );
        assert!(token.has_subtype("Insect") && token.keywords.contains(Keywords::FLYING));
    }
}

fn announce_room(card: CardId, door: RoomDoor) -> Action {
    Action::AnnounceRoom {
        card,
        door,
        flash_surcharge: false,
    }
}

fn unlock_right(game: &mut Game, room: CardId) {
    let object = game.object_ref(room);
    act(
        game,
        Action::BeginRoomUnlock {
            object,
            door: RoomDoor::Right,
            cost: ManaCost {
                colored: [0, 1, 0, 0, 0, 0],
                generic: 3,
            },
        },
    );
    choose(game, |action| {
        matches!(action, Action::PayRoomUnlock { .. })
    });
}

#[test]
fn gremlin_tamer_triggers_once_for_entry_and_once_for_a_full_unlock() {
    let player = Player::P0;
    let mut game = scenario(player);
    cast_named(&mut game, player, "Gremlin Tamer");
    settle(&mut game);
    assert!(controlled_tokens(&game, player, GREMLIN).is_empty());
    cast_named(&mut game, player, "Bear Cub");
    settle(&mut game);
    assert!(controlled_tokens(&game, player, GREMLIN).is_empty());

    let room = put(&mut game, player, SMOKY_ROOM, Zone::Hand);
    act(&mut game, announce_room(room, RoomDoor::Left));
    pay(&mut game);
    settle_ordered(&mut game);
    assert_eq!(controlled_tokens(&game, player, GREMLIN).len(), 1);
    unlock_right(&mut game, room);
    settle_ordered(&mut game);
    let gremlins = controlled_tokens(&game, player, GREMLIN);
    assert_eq!(gremlins.len(), 2);
    for token in gremlins {
        assert_eq!((token.power, token.toughness, token.colors), (1, 1, RED));
        assert!(token.has_subtype("Gremlin"));
    }

    next_main(&mut game, player.other());
    game.state.players[player.other().index()].mana = [0, 0, 0, 1, 0, 2];
    let opposing_room = put(&mut game, player.other(), SMOKY_ROOM, Zone::Hand);
    act(&mut game, announce_room(opposing_room, RoomDoor::Left));
    pay(&mut game);
    settle_ordered(&mut game);
    assert_eq!(controlled_tokens(&game, player, GREMLIN).len(), 2);
}

fn setup_token(game: &mut Game, player: Player, token: Card) -> CardId {
    let id = game.state.objects.len() as CardId;
    game.create_tokens(player, token, 1, false);
    id
}

#[test]
fn midnight_mayhem_grants_three_keywords_to_the_existing_gremlin_cohort() {
    let player = Player::P1;
    let mut game = scenario(player);
    let existing = setup_token(&mut game, player, GREMLIN);
    let opposing = setup_token(&mut game, player.other(), GREMLIN);
    cast_named(&mut game, player, "Midnight Mayhem");
    settle(&mut game);
    let tokens = controlled_tokens(&game, player, GREMLIN);
    assert_eq!(tokens.len(), 4);
    let abilities = Keywords::MENACE
        .union(Keywords::LIFELINK)
        .union(Keywords::HASTE);
    assert!(tokens.iter().any(|token| token.id == existing));
    assert!(
        tokens
            .iter()
            .all(|token| token.keywords.contains(abilities))
    );
    assert!(!game.object_view(opposing).keywords.intersects(abilities));
    let later = setup_token(&mut game, player, GREMLIN);
    assert!(!game.object_view(later).keywords.intersects(abilities));

    let new = tokens.iter().find(|token| token.id != existing).unwrap().id;
    attack(&mut game, new);
    progress_until(&mut game, |game| game.state.step == Step::Main2);
    assert_eq!(game.observe(player).players[player.index()].life, 21);
    assert_eq!(
        game.observe(player).players[player.other().index()].life,
        19
    );
    next_main(&mut game, player.other());
    for token in tokens {
        assert!(!game.object_view(token.id).keywords.intersects(abilities));
    }
}

#[test]
fn niko_creates_working_shards_and_temporarily_copies_exiled_creature_values() {
    let player = Player::P0;
    let mut game = scenario(player);
    let niko = cast_named(&mut game, player, "Niko, Light of Hope");
    settle(&mut game);
    assert_eq!(stats(&game, niko), (3, 4));
    let shards = controlled_tokens(&game, player, SHARD);
    assert_eq!(shards.len(), 2);
    assert!(
        shards
            .iter()
            .all(|token| token.kinds == 32 && token.colors == 0 && token.has_subtype("Shard"))
    );

    let drawn = put(&mut game, player, "Forest", Zone::Library);
    let bottom = put(&mut game, player, "Shock", Zone::Library);
    act(&mut game, activation(shards[0].id, 0));
    finish_targets_and_pay(&mut game);
    assert_ne!(game.object_view(shards[0].id).zone, Zone::Battlefield);
    progress_until(&mut game, |game| {
        game.observe(player).library_choice.is_some()
    });
    assert!(!game.observe(player).library_choice.unwrap().surveil);
    assert!(game.observe(player.other()).library_choice.is_none());
    act(
        &mut game,
        Action::ArrangeLibrary {
            card: bottom,
            destination: LibraryDestination::Bottom,
        },
    );
    settle(&mut game);
    assert_eq!(game.object_view(drawn).zone, Zone::Hand);
    assert_eq!(game.state.players[player.index()].library[0], bottom);

    next_main(&mut game, player);
    let foe = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    let donor = put(&mut game, player.other(), "Oaken Siren", Zone::Battlefield);
    let donor_reference = game.object_ref(donor);
    game.record_control(donor_reference, player, None);
    game.add_counter(donor, CounterKind::PlusOne, 2);
    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 2];
    act(&mut game, activation(niko, 0));
    reject(&mut game, target(foe));
    reject(&mut game, target(niko));
    act(&mut game, target(donor));
    finish_targets_and_pay(&mut game);
    settle(&mut game);
    let shard = shards[1].reference;
    let copied = game.object_view(shard.card);
    assert_eq!(copied.reference, shard);
    assert_eq!(copied.name, "Oaken Siren");
    assert_eq!(
        (copied.power, copied.toughness, copied.colors),
        (1, 2, BLUE)
    );
    assert!(copied.has_subtype("Siren"));
    assert!(
        copied
            .keywords
            .contains(Keywords::FLYING.union(Keywords::VIGILANCE))
    );
    assert_eq!(game.object_view(donor).zone, Zone::Exile);

    progress_until(&mut game, |game| game.state.step == Step::End);
    let restored = game.object_view(shard.card);
    assert_eq!(restored.reference, shard);
    assert_eq!(restored.card, SHARD);
    assert!(restored.has_subtype("Shard"));
    assert_eq!(game.object_view(donor).zone, Zone::Exile);
    settle(&mut game);
    let returned = game.object_view(donor);
    assert_eq!(returned.zone, Zone::Battlefield);
    assert_eq!(returned.controller, player.other());
    assert_ne!(returned.reference, donor_reference);
}

#[test]
fn niko_waits_for_a_linked_return_to_complete_before_copying_shards() {
    let player = Player::P0;
    let mut game = scenario(player);
    let bandit = put(&mut game, player, "Visage Bandit", Zone::Battlefield);
    let donor = put(&mut game, player, "Oaken Siren", Zone::Battlefield);
    let niko = cast_named(&mut game, player, "Niko, Light of Hope");
    settle(&mut game);
    let shards = controlled_tokens(&game, player, SHARD);
    assert_eq!(shards.len(), 2);

    let bodyguard = cast_named(&mut game, player, "Werefox Bodyguard");
    let exile = target(bandit);
    progress_until(&mut game, |game| game.legal_actions().contains(&exile));
    act(&mut game, exile);
    finish_targets_and_pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(bandit).zone, Zone::Exile);

    next_main(&mut game, player);
    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 2];
    act(&mut game, activation(niko, 0));
    act(&mut game, target(bodyguard));
    finish_targets_and_pay(&mut game);
    progress_until(&mut game, |game| {
        matches!(game.state.pending, Pending::EntryCopy { .. })
    });
    assert_eq!(game.object_view(bandit).zone, Zone::Exile);
    assert_eq!(game.object_view(bodyguard).zone, Zone::Exile);
    for shard in &shards {
        reject(
            &mut game,
            Action::ChooseEntryCopy {
                object: Some(shard.reference),
            },
        );
        assert_eq!(game.object_view(shard.id).card, SHARD);
    }

    let choice = Action::ChooseEntryCopy {
        object: Some(game.object_ref(donor)),
    };
    let checkpoint = game.snapshot();
    let index = game
        .legal_actions()
        .iter()
        .position(|action| *action == choice)
        .unwrap();
    let mut indexed = game.clone();
    act(&mut game, choice);
    indexed
        .apply_index(player, indexed.revision(), index)
        .unwrap();
    assert_eq!(indexed.snapshot(), game.snapshot());
    indexed.restore(&checkpoint).unwrap();
    assert_eq!(indexed.snapshot(), checkpoint);
    settle(&mut game);

    assert_eq!(game.object_view(bandit).name, "Oaken Siren");
    for shard in shards {
        let copied = game.object_view(shard.id);
        assert_eq!(copied.reference, shard.reference);
        assert_eq!(copied.name, "Werefox Bodyguard");
    }
    assert!(game.observe(player).delayed_returns.iter().any(|entry| {
        entry.returning.object == game.object_ref(bodyguard)
            && entry.returning.zone == Zone::Exile
            && entry.returning.controller == player
    }));
}

#[test]
fn jolly_balloon_man_immediately_makes_a_copiable_balloon_and_sacrifices_it() {
    let player = Player::P1;
    let mut game = scenario(player);
    let donor = put(&mut game, player, "Oaken Siren", Zone::Battlefield);
    let opposing = put(&mut game, player.other(), "Oaken Siren", Zone::Battlefield);
    game.add_counter(donor, CounterKind::PlusOne, 2);
    let jolly = cast_named(&mut game, player, "The Jolly Balloon Man");
    settle(&mut game);
    assert!(game.object_view(jolly).keywords.contains(Keywords::HASTE));
    act(&mut game, activation(jolly, 0));
    reject(&mut game, target(jolly));
    reject(&mut game, target(opposing));
    act(&mut game, target(donor));
    finish_targets_and_pay(&mut game);
    settle(&mut game);
    let token = game
        .observe(player)
        .objects
        .into_iter()
        .find(|object| {
            object.is_token() && object.has_subtype("Balloon") && object.zone == Zone::Battlefield
        })
        .unwrap();
    assert_eq!(token.name, "Oaken Siren");
    assert_eq!(
        (token.power, token.toughness, token.colors, token.kinds),
        (1, 1, BLUE | RED, 1 | 16)
    );
    assert!(token.has_subtype("Siren") && token.has_subtype("Pirate"));
    assert!(
        token.keywords.contains(
            Keywords::FLYING
                .union(Keywords::VIGILANCE)
                .union(Keywords::HASTE)
        )
    );
    assert_eq!(game.counter_count(token.id, CounterKind::PlusOne), 0);

    let removal = put(&mut game, player, "Murder", Zone::Hand);
    cast(&mut game, removal, Some(Target::Card(jolly)));
    settle(&mut game);
    progress_until(&mut game, |game| game.state.step == Step::End);
    assert_eq!(game.object_view(token.id).reference, token.reference);
    assert_eq!(game.object_view(token.id).zone, Zone::Battlefield);
    settle(&mut game);
    assert_ne!(game.object_view(token.id).zone, Zone::Battlefield);
    assert_eq!(game.object_view(donor).zone, Zone::Battlefield);
}

#[test]
fn swarmweaver_grants_one_live_bonus_to_its_subtype_union_at_delirium() {
    let player = Player::P0;
    let mut game = scenario(player);
    put(&mut game, player, "Forest", Zone::Graveyard);
    put(&mut game, player, "Shock", Zone::Graveyard);
    put(&mut game, player, "Short Bow", Zone::Graveyard);
    let spider = put(&mut game, player, "Twitching Doll", Zone::Battlefield);
    let both = put(&mut game, player, "Three Tree Mascot", Zone::Battlefield);
    let enemy = put(
        &mut game,
        player.other(),
        "Three Tree Mascot",
        Zone::Battlefield,
    );
    let unrelated = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let fourth = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let ooze = put(&mut game, player, "Scavenging Ooze", Zone::Battlefield);
    let base_both = stats(&game, both);
    assert!(
        game.object_view(both).has_subtype("Insect")
            && game.object_view(both).has_subtype("Spider")
    );
    cast_named(&mut game, player, "The Swarmweaver");
    settle(&mut game);
    let insects = controlled_tokens(&game, player, INSECT);
    assert_eq!(insects.len(), 2);
    assert!(
        insects
            .iter()
            .all(
                |token| (token.power, token.toughness, token.colors) == (1, 1, BLACK | GREEN)
                    && token.keywords.contains(Keywords::FLYING)
            )
    );
    assert_eq!(stats(&game, both), base_both);
    assert!(
        !game
            .object_view(spider)
            .keywords
            .contains(Keywords::DEATHTOUCH)
    );

    let removal = put(&mut game, player, "Murder", Zone::Hand);
    cast(&mut game, removal, Some(Target::Card(fourth)));
    settle(&mut game);
    assert_eq!(stats(&game, both), (base_both.0 + 1, base_both.1 + 1));
    assert_eq!(stats(&game, spider), (3, 3));
    for id in [both, spider, insects[0].id, insects[1].id] {
        assert!(game.object_view(id).keywords.contains(Keywords::DEATHTOUCH));
    }
    assert_eq!(stats(&game, insects[0].id), (2, 2));
    assert_eq!(stats(&game, enemy), base_both);
    assert_eq!(stats(&game, unrelated), (3, 3));
    assert!(
        !game
            .object_view(enemy)
            .keywords
            .contains(Keywords::DEATHTOUCH)
    );
    assert!(
        !game
            .object_view(unrelated)
            .keywords
            .contains(Keywords::DEATHTOUCH)
    );

    act(&mut game, activation(ooze, 0));
    act(&mut game, target(fourth));
    finish_targets_and_pay(&mut game);
    settle(&mut game);
    assert_eq!(stats(&game, spider), (2, 2));
    assert_eq!(stats(&game, both), base_both);
    assert!(
        !game
            .object_view(insects[0].id)
            .keywords
            .contains(Keywords::DEATHTOUCH)
    );
}

#[test]
fn smoky_lounge_mana_funds_both_room_routes_and_misty_salon_samples_live_doors() {
    let player = Player::P0;
    let mut game = scenario(player);
    let first = put(&mut game, player, SMOKY_ROOM, Zone::Hand);
    act(&mut game, announce_room(first, RoomDoor::Left));
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(first).name, "Smoky Lounge");
    let second = put(&mut game, player, SMOKY_ROOM, Zone::Hand);
    let shock = put(&mut game, player, "Shock", Zone::Hand);
    next_main(&mut game, player);
    assert_eq!(game.state.stack.len(), 1);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].mana, [0, 0, 0, 2, 0, 0]);
    assert!(game.observe(player).mana_lots.iter().any(|lot| {
        lot.player == player && lot.restriction == ManaRestrictionView::RoomSpellOrUnlockDoor
    }));
    reject(&mut game, Action::AnnounceCast { card: shock });
    game.state.players[player.index()].mana[1] = 1;
    game.state.players[player.index()].mana[5] = 1;

    let mut unlock = game.clone();
    unlock_right(&mut unlock, first);
    settle(&mut unlock);
    assert_eq!(unlock.state.players[player.index()].mana, [0; 6]);
    let full_room_spirit = controlled_tokens(&unlock, player, SPIRIT);
    assert_eq!(full_room_spirit.len(), 1);
    assert_eq!(
        (full_room_spirit[0].power, full_room_spirit[0].toughness),
        (2, 2)
    );

    act(&mut game, announce_room(second, RoomDoor::Right));
    pay(&mut game);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    assert_eq!(game.object_view(second).name, "Misty Salon");
    assert_eq!(game.state.stack.len(), 1);
    act(&mut game, Action::Pass);
    let foe = player.other();
    game.state.players[foe.index()].mana = [1, 0, 0, 0, 0, 1];
    let removal = put(&mut game, foe, "Disenchant", Zone::Hand);
    cast(&mut game, removal, Some(Target::Card(first)));
    settle(&mut game);
    let spirit = controlled_tokens(&game, player, SPIRIT);
    assert_eq!(spirit.len(), 1);
    assert_eq!(
        (spirit[0].power, spirit[0].toughness, spirit[0].colors),
        (1, 1, BLUE)
    );
    assert!(spirit[0].has_subtype("Spirit") && spirit[0].keywords.contains(Keywords::FLYING));
}

#[test]
fn enduring_vitality_retains_mana_grants_when_it_returns_to_its_owner() {
    let player = Player::P1;
    let mut game = scenario(player);
    let first = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let second = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let enemy = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    let enduring = cast_named(&mut game, player, "Enduring Vitality");
    settle(&mut game);
    assert!(
        game.object_view(enduring)
            .keywords
            .contains(Keywords::VIGILANCE)
    );
    let original = game.object_ref(enduring);
    let blue_mana = |card, grantor| Action::TapGrantedMana {
        card,
        grantor,
        production: [0, 1, 0, 0, 0, 0],
    };
    game.state.players[player.index()].mana = [0; 6];
    reject(&mut game, blue_mana(enemy, original));
    let opt = put(&mut game, player, "Opt", Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: opt });
    act(&mut game, blue_mana(first, original));
    pay(&mut game);
    progress_until(&mut game, |game| {
        game.observe(player).library_choice.is_some()
    });
    choose(&mut game, |action| {
        matches!(
            action,
            Action::ArrangeLibrary {
                destination: LibraryDestination::Top,
                ..
            }
        )
    });
    settle(&mut game);
    assert_eq!(game.object_view(opt).zone, Zone::Graveyard);
    assert!(game.object_view(first).tapped);

    game.record_control(original, player.other(), None);
    game.state.players[player.index()].mana = [1, 0, 2, 0, 0, 2];
    let removal = put(&mut game, player, "Murder", Zone::Hand);
    cast(&mut game, removal, Some(Target::Card(enduring)));
    settle(&mut game);
    let returned = game.object_view(enduring);
    assert_eq!(returned.controller, player);
    assert_eq!(returned.kinds, 32);
    assert_ne!(returned.reference, original);
    act(&mut game, blue_mana(second, returned.reference));
    assert!(game.object_view(second).tapped);
    let removal = put(&mut game, player, "Disenchant", Zone::Hand);
    cast(&mut game, removal, Some(Target::Card(enduring)));
    settle(&mut game);
    assert_eq!(game.object_view(enduring).zone, Zone::Graveyard);
}

#[test]
fn kona_can_put_a_land_or_a_locked_room_onto_the_battlefield_from_hand() {
    let player = Player::P0;
    let mut game = scenario(player);
    let kona = cast_named(&mut game, player, "Kona, Rescue Beastie");
    settle(&mut game);
    next_main(&mut game, player);
    let played = put(&mut game, player, "Forest", Zone::Hand);
    act(&mut game, Action::PlayLand { card: played });
    let room = put(&mut game, player, SMOKY_ROOM, Zone::Hand);
    let land = put(&mut game, player, "Island", Zone::Hand);
    let instant = put(&mut game, player, "Shock", Zone::Hand);
    attack(&mut game, kona);
    progress_until(&mut game, |game| {
        game.observe(player).effect_choice.is_some()
    });
    assert_eq!(game.state.step, Step::Main2);
    assert!(game.observe(player.other()).effect_choice.is_none());
    let candidates = game.observe(player).effect_choice.unwrap().candidates;
    assert!(
        candidates.contains(&game.object_ref(room)) && candidates.contains(&game.object_ref(land))
    );
    let instant_ref = game.object_ref(instant);
    reject(
        &mut game,
        Action::PutHandPermanent {
            object: instant_ref,
        },
    );

    let mut decline = game.clone();
    act(&mut decline, Action::DeclineHandEntry);
    settle(&mut decline);
    assert_eq!(decline.object_view(room).zone, Zone::Hand);
    assert_eq!(decline.object_view(land).zone, Zone::Hand);
    assert!(decline.observe(player.other()).known_hand.is_empty());
    let mut land_branch = game.clone();
    let land_ref = game.object_ref(land);
    act(
        &mut land_branch,
        Action::PutHandPermanent { object: land_ref },
    );
    settle(&mut land_branch);
    assert_eq!(land_branch.object_view(land).zone, Zone::Battlefield);
    assert_eq!(land_branch.object_view(room).zone, Zone::Hand);

    let room_ref = game.object_ref(room);
    act(&mut game, Action::PutHandPermanent { object: room_ref });
    settle(&mut game);
    assert_eq!(game.object_view(room).zone, Zone::Battlefield);
    assert_eq!(
        game.object_view(room).room_doors,
        Some(RoomDoors::default())
    );
    assert!(controlled_tokens(&game, player, SPIRIT).is_empty());
}

#[test]
fn rootwise_survivor_leaves_the_target_land_animated_after_haste_expires() {
    let player = Player::P1;
    let mut game = scenario(player);
    game.begin_step(Step::Main1);
    let survivor = cast_named(&mut game, player, "Rootwise Survivor");
    settle(&mut game);
    let land = put(&mut game, player, "Forest", Zone::Hand);
    let enemy_land = put(&mut game, player.other(), "Island", Zone::Battlefield);
    act(&mut game, Action::PlayLand { card: land });
    attack(&mut game, survivor);
    progress_until(&mut game, |game| game.state.step == Step::Main2);
    reject(&mut game, target(enemy_land));
    let mut no_target = game.clone();
    act(&mut no_target, Action::FinishTargets);
    settle_ordered(&mut no_target);
    assert_eq!(no_target.object_view(land).kinds, 2);
    assert_eq!(no_target.counter_count(land, CounterKind::PlusOne), 0);

    act(&mut game, target(land));
    finish_targets_and_pay(&mut game);
    settle_ordered(&mut game);
    let animated = game.object_view(land);
    assert_eq!(animated.kinds, 1 | 2);
    assert!(animated.has_subtype("Forest") && animated.has_subtype("Elemental"));
    assert_eq!(stats(&game, land), (3, 3));
    assert_eq!(game.counter_count(land, CounterKind::PlusOne), 3);
    assert!(animated.keywords.contains(Keywords::HASTE));
    act(&mut game, Action::TapMana { card: land });
    assert_eq!(game.state.players[player.index()].mana, [0, 0, 0, 0, 1, 0]);

    next_main(&mut game, player.other());
    assert!(game.object_view(land).keywords.contains(Keywords::HASTE));
    next_main(&mut game, player);
    assert_eq!(game.object_view(land).reference, animated.reference);
    assert_eq!(game.object_view(land).kinds, 1 | 2);
    assert_eq!(stats(&game, land), (3, 3));
    assert!(!game.object_view(land).keywords.contains(Keywords::HASTE));
}
