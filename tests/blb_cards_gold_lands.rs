//! Villages and Three Tree City, including typal histories and restricted mana.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

const COLORLESS: Mana = [0, 0, 0, 0, 0, 1];

fn mana(card: CardId, ability: u16, production: Mana) -> Action {
    Action::ActivateMana {
        card,
        ability,
        payment: [0; 6],
        counter_payment: Counters::default(),
        production,
    }
}

fn activate(card: CardId, ability: u16) -> Action {
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

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

#[test]
fn each_village_has_unrestricted_colorless_and_restricted_colored_mana() {
    let player = Player::P0;
    for (name, color) in [
        ("Lupinflower Village", 0),
        ("Lilypad Village", 1),
        ("Mudflat Village", 2),
        ("Rockface Village", 3),
        ("Oakhollow Village", 4),
    ] {
        let mut game = scenario(player);
        game.state.players[player.index()].mana = [0; 6];
        let land = put(&mut game, player, name, Zone::Hand);
        act(&mut game, Action::PlayLand { card: land });
        let mut plain = game.clone();
        act(&mut plain, mana(land, 0, COLORLESS));
        assert_eq!(plain.state.players[player.index()].mana, COLORLESS);
        assert!(plain.observe(player).mana_lots.is_empty());

        let mut colored = [0; 6];
        colored[color] = 1;
        act(&mut game, mana(land, 1, colored));
        let lots = game.observe(player).mana_lots;
        assert_eq!(lots.len(), 1);
        assert_eq!(lots[0].restriction, ManaRestrictionView::CreatureSpells);
        assert_eq!(lots[0].amounts[color], 1);
        reject(&mut game, mana(land, 0, COLORLESS));
        if color == 1 {
            let opt = put(&mut game, player, "Opt", Zone::Hand);
            let bird = put(&mut game, player, "Spyglass Siren", Zone::Hand);
            reject(&mut game, Action::AnnounceCast { card: opt });
            cast(&mut game, bird, None);
            settle(&mut game);
            assert_eq!(game.object_view(bird).zone, Zone::Battlefield);
            assert_eq!(game.state.players[player.index()].mana, [0; 6]);
        }
    }
}

#[test]
fn lilypad_requires_entry_under_your_control_and_surveils_after_that_object_leaves() {
    let player = Player::P1;
    let mut game = scenario(player);
    let land = put(&mut game, player, "Lilypad Village", Zone::Battlefield);
    let action = activate(land, 2);
    reject(&mut game, action.clone());
    let bird = put(&mut game, player, "Healer's Hawk", Zone::Hand);
    cast(&mut game, bird, None);
    settle(&mut game);
    game.move_object(bird, Zone::Exile);

    let lower = put(&mut game, player, "Plains", Zone::Library);
    let top = put(&mut game, player, "Island", Zone::Library);
    act(&mut game, action);
    act(
        &mut game,
        Action::PayCosts {
            payment: [0, 1, 0, 0, 0, 0],
        },
    );
    pass_pair(&mut game);
    let looked = game.observe(player).library_choice.unwrap();
    assert!(looked.surveil);
    assert_eq!(looked.remaining.len(), 2);
    assert!(game.observe(player.other()).library_choice.is_none());
    act(
        &mut game,
        Action::ArrangeLibrary {
            card: top,
            destination: LibraryDestination::Graveyard,
        },
    );
    act(
        &mut game,
        Action::ArrangeLibrary {
            card: lower,
            destination: LibraryDestination::Top,
        },
    );
    assert_eq!(game.object_view(top).zone, Zone::Graveyard);
    assert_eq!(
        game.state.players[player.index()].library.last(),
        Some(&lower)
    );
}

#[test]
fn lupinflower_keeps_unchosen_cards_private_and_reveals_one_qualifying_pick() {
    let player = Player::P0;
    let mut game = scenario(player);
    let land = put(&mut game, player, "Lupinflower Village", Zone::Battlefield);
    let outside = put(&mut game, player, "Healer's Hawk", Zone::Library);
    let miss = put(&mut game, player, "Grizzly Bears", Zone::Library);
    let pick = put(&mut game, player, "Three Tree Mascot", Zone::Library);
    let other = [(); 4].map(|_| put(&mut game, player, "Island", Zone::Library));
    act(&mut game, activate(land, 2));
    act(
        &mut game,
        Action::PayCosts {
            payment: [1, 0, 0, 0, 0, 1],
        },
    );
    assert_eq!(game.object_view(land).zone, Zone::Graveyard);
    pass_pair(&mut game);
    let private = game.observe(player).library_selection.unwrap();
    assert_eq!(private.remaining.len(), 6);
    assert!(game.observe(player.other()).library_selection.is_none());
    reject(&mut game, Action::SelectLibrary { card: outside });
    reject(&mut game, Action::SelectLibrary { card: miss });
    let mut decline = game.clone();
    act(&mut decline, Action::FinishLibrarySelection);
    assert_eq!(decline.object_view(pick).zone, Zone::Library);
    act(&mut game, Action::SelectLibrary { card: pick });
    act(&mut game, Action::FinishLibrarySelection);
    assert_eq!(game.object_view(pick).zone, Zone::Hand);
    let public = game.observe(player.other());
    assert!(public.objects.iter().any(|o| o.id == pick));
    assert!(
        public
            .objects
            .iter()
            .all(|o| !other.contains(&o.id) && o.id != miss)
    );
    let library = &game.state.players[player.index()].library;
    assert_eq!(library.last(), Some(&outside));
    for id in other.into_iter().chain([miss]) {
        assert!(library[..5].contains(&id));
    }
}

#[test]
fn mudflat_targets_own_matching_kindred_cards_and_pays_the_sacrifice_first() {
    let player = Player::P1;
    let mut game = scenario(player);
    let land = put(&mut game, player, "Mudflat Village", Zone::Battlefield);
    let own = put(&mut game, player, "Firdoch Core", Zone::Graveyard);
    let wrong_type = put(&mut game, player, "Healer's Hawk", Zone::Graveyard);
    let foreign = put(&mut game, player.other(), "Firdoch Core", Zone::Graveyard);
    act(&mut game, activate(land, 2));
    reject(&mut game, target(wrong_type));
    reject(&mut game, target(foreign));
    act(&mut game, target(own));
    act(
        &mut game,
        Action::PayCosts {
            payment: [0, 0, 1, 0, 0, 1],
        },
    );
    assert_eq!(game.object_view(land).zone, Zone::Graveyard);
    let mut stale = game.clone();
    stale.move_object(own, Zone::Exile);
    stale.move_object(own, Zone::Graveyard);
    settle(&mut stale);
    assert_eq!(stale.object_view(own).zone, Zone::Graveyard);
    assert_eq!(stale.object_view(land).zone, Zone::Graveyard);
    settle(&mut game);
    assert_eq!(game.object_view(own).zone, Zone::Hand);
}

#[test]
fn oakhollow_counts_each_current_matching_new_permanent_once() {
    let player = Player::P0;
    let mut game = scenario(player);
    let land = put(&mut game, player, "Oakhollow Village", Zone::Battlefield);
    let old = put(&mut game, player, "Three Tree Mascot", Zone::Battlefield);
    game.begin_turn();
    game.state.step = Step::Main1;
    game.state.pending = Pending::Priority;
    game.state.priority = player;
    game.state.players[player.index()].mana[4] = 1;
    let ours = put(&mut game, player, "Three Tree Mascot", Zone::Hand);
    let stolen = put(&mut game, player.other(), "Firdoch Core", Zone::Hand);
    let foreign = put(&mut game, player.other(), "Three Tree Mascot", Zone::Hand);
    let off_type = put(&mut game, player, "Grizzly Bears", Zone::Hand);
    for card in [ours, stolen, foreign, off_type] {
        game.move_object(card, Zone::Battlefield);
    }
    game.record_control(game.object_ref(stolen), player, None);
    act(&mut game, activate(land, 2));
    act(
        &mut game,
        Action::PayCosts {
            payment: [0, 0, 0, 0, 1, 0],
        },
    );
    settle(&mut game);
    for card in [ours, stolen] {
        assert_eq!(game.counter_count(card, CounterKind::PlusOne), 1);
    }
    for card in [old, foreign, off_type] {
        assert_eq!(game.counter_count(card, CounterKind::PlusOne), 0);
    }
}

#[test]
fn rockface_buffs_a_matching_permanent_and_requires_sorcery_timing() {
    let player = Player::P0;
    let mut game = scenario(player);
    let land = put(&mut game, player, "Rockface Village", Zone::Battlefield);
    let core = put(&mut game, player, "Firdoch Core", Zone::Battlefield);
    let bears = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let foreign = put(&mut game, player.other(), "Firdoch Core", Zone::Battlefield);
    game.state.step = Step::BeginCombat;
    reject(&mut game, activate(land, 2));
    game.state.step = Step::Main1;
    act(&mut game, activate(land, 2));
    reject(&mut game, target(bears));
    reject(&mut game, target(foreign));
    act(&mut game, target(core));
    act(
        &mut game,
        Action::PayCosts {
            payment: [0, 0, 0, 1, 0, 0],
        },
    );
    settle(&mut game);
    assert!(game.keywords(core).contains(Keywords::HASTE));
    assert!(!game.is_creature(core));
    act(&mut game, activate(core, 1));
    act(
        &mut game,
        Action::PayCosts {
            payment: [0, 0, 0, 0, 0, 4],
        },
    );
    settle(&mut game);
    assert_eq!(stats(&game, core), (5, 4));
    game.finish_cleanup();
    assert!(!game.is_creature(core));
    assert!(!game.keywords(core).contains(Keywords::HASTE));
}

#[test]
fn three_tree_city_chooses_a_type_and_produces_one_color_from_the_live_count() {
    let player = Player::P1;
    let mut game = scenario(player);
    let land = put(&mut game, player, "Three Tree City", Zone::Hand);
    act(&mut game, Action::PlayLand { card: land });
    act(
        &mut game,
        Action::ChooseEntryValue {
            value: EntryChoiceValue::CreatureType(CreatureType::from_name("Bird").unwrap()),
        },
    );
    put(&mut game, player, "Healer's Hawk", Zone::Battlefield);
    put(&mut game, player, "Three Tree Mascot", Zone::Battlefield);
    put(&mut game, player, "Firdoch Core", Zone::Battlefield);
    put(
        &mut game,
        player.other(),
        "Healer's Hawk",
        Zone::Battlefield,
    );

    let mut plain = game.clone();
    plain.state.players[player.index()].mana = [0; 6];
    act(&mut plain, mana(land, 0, COLORLESS));
    assert_eq!(plain.state.players[player.index()].mana, COLORLESS);

    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 2];
    let activation = |production| Action::ActivateMana {
        card: land,
        ability: 1,
        payment: [0, 0, 0, 0, 0, 2],
        counter_payment: Counters::default(),
        production,
    };
    let wanted = [0, 2, 0, 0, 0, 0];
    if game.legal_actions().contains(&activation(wanted)) {
        act(&mut game, activation(wanted));
    } else {
        act(&mut game, activation([0; 6]));
        reject(
            &mut game,
            Action::ChooseMana {
                production: [1, 1, 0, 0, 0, 0],
            },
        );
        act(&mut game, Action::ChooseMana { production: wanted });
    }
    assert_eq!(game.state.players[player.index()].mana, wanted);
    assert!(game.object_view(land).tapped);
}
