//! Entry filters, library choices and exact nondeath departures on authored cards.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn cast_card(game: &mut Game, player: Player, name: &str) -> CardId {
    game.state.players[player.index()].mana = [8; 6];
    let card = put(game, player, name, Zone::Hand);
    cast(game, card, None);
    card
}

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

fn drain(game: &mut Game, decision: impl Fn(&Action) -> bool) {
    for _ in 0..64 {
        if game.state.pending == Pending::Priority
            && game.state.stack.is_empty()
            && game.state.pending_triggers.is_empty()
        {
            return;
        }
        choose(game, |action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. }) || decision(action)
        });
    }
    panic!("Bloomburrow entry effects did not settle");
}

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

#[test]
fn clifftop_reveals_to_the_first_land_and_greenshell_keeps_private_land_choices() {
    let player = Player::P0;
    let opponent = player.other();
    let mut lookout = scenario(player);
    let land = put(&mut lookout, player, "Mountain", Zone::Library);
    let second = put(&mut lookout, player, "Shock", Zone::Library);
    let top = put(&mut lookout, player, "Giant Growth", Zone::Library);
    let source = cast_card(&mut lookout, player, "Clifftop Lookout");
    drain(&mut lookout, |action| {
        *action == Action::SelectLibrary { card: land } || *action == Action::FinishLibrarySelection
    });
    assert!(lookout.keywords(source).contains(Keywords::REACH));
    assert_eq!(lookout.object_view(land).zone, Zone::Battlefield);
    assert!(lookout.object_view(land).tapped);
    assert!(lookout.state.players[player.index()].library[..2].contains(&second));
    assert!(lookout.state.players[player.index()].library[..2].contains(&top));

    let mut turtle = scenario(player);
    for _ in 0..9 {
        put(&mut turtle, player, "Forest", Zone::Battlefield);
    }
    let land = put(&mut turtle, player, "Forest", Zone::Library);
    let source = cast_card(&mut turtle, player, "Fecund Greenshell");
    pass_pair(&mut turtle);
    pass_pair(&mut turtle);
    assert!(turtle.observe(player).library_selection.is_some());
    assert!(turtle.observe(opponent).library_selection.is_none());
    let mut hand = turtle.clone();
    act(&mut hand, Action::FinishLibrarySelection);
    settle(&mut hand);
    assert_eq!(hand.object_view(land).zone, Zone::Hand);
    assert_eq!(stats(&hand, source), (4, 6));

    act(&mut turtle, Action::SelectLibrary { card: land });
    settle(&mut turtle);
    assert_eq!(turtle.object_view(land).zone, Zone::Battlefield);
    assert!(turtle.object_view(land).tapped);
    assert_eq!(stats(&turtle, source), (6, 8));
    assert!(turtle.keywords(source).contains(Keywords::REACH));

    let nonland = put(&mut turtle, player, "Giant Growth", Zone::Library);
    cast_card(&mut turtle, player, "Grizzly Bears");
    settle(&mut turtle);
    assert_eq!(turtle.object_view(nonland).zone, Zone::Library);
    let entrant = cast_card(&mut turtle, player, "Kindlespark Duo");
    drain(&mut turtle, |action| {
        matches!(
            action,
            Action::FinishLibrarySelection | Action::ArrangeLibrary { .. }
        )
    });
    assert_eq!(turtle.object_view(nonland).zone, Zone::Hand);
    assert_eq!(stats(&turtle, entrant), (3, 5));
}

#[test]
fn lumra_mills_four_and_returns_old_and_new_lands_tapped() {
    let player = Player::P1;
    let mut game = scenario(player);
    put(&mut game, player, "Forest", Zone::Battlefield);
    let old = put(&mut game, player, "Forest", Zone::Graveyard);
    let land_a = put(&mut game, player, "Forest", Zone::Library);
    let spell_a = put(&mut game, player, "Shock", Zone::Library);
    let land_b = put(&mut game, player, "Swamp", Zone::Library);
    let spell_b = put(&mut game, player, "Giant Growth", Zone::Library);
    let source = cast_card(&mut game, player, "Lumra, Bellow of the Woods");
    settle(&mut game);
    for land in [old, land_a, land_b] {
        assert_eq!(game.object_view(land).zone, Zone::Battlefield);
        assert!(game.object_view(land).tapped);
    }
    for spell in [spell_a, spell_b] {
        assert_eq!(game.object_view(spell).zone, Zone::Graveyard);
    }
    assert_eq!(stats(&game, source), (4, 4));
    assert!(
        game.keywords(source)
            .contains(Keywords::REACH.union(Keywords::VIGILANCE))
    );
    let in_yard = put(
        &mut game,
        player,
        "Lumra, Bellow of the Woods",
        Zone::Graveyard,
    );
    assert_eq!(stats(&game, in_yard), (4, 4));
}

fn search_land(game: &mut Game, card: CardId) {
    let choose_land = Action::ChooseSearch { card };
    for _ in 0..24 {
        if game.state.pending == Pending::Priority
            && game.state.stack.is_empty()
            && game.state.pending_triggers.is_empty()
        {
            return;
        }
        if game.legal_actions().contains(&choose_land) {
            act(game, choose_land.clone());
        } else {
            choose(game, |action| {
                matches!(
                    action,
                    Action::Pass
                        | Action::PutTrigger { .. }
                        | Action::ChooseOptional { accept: true }
                        | Action::FinishSearch
                )
            });
        }
    }
    panic!("Heaped Harvest's search did not settle");
}

#[test]
fn heaped_harvest_searches_at_entry_and_above_its_sacrifice_life_gain() {
    let player = Player::P0;
    let mut game = scenario(player);
    let first = put(&mut game, player, "Mountain", Zone::Library);
    let second = put(&mut game, player, "Swamp", Zone::Library);
    let harvest = cast_card(&mut game, player, "Heaped Harvest");
    search_land(&mut game, first);
    assert_eq!(game.object_view(first).zone, Zone::Battlefield);
    assert!(game.object_view(first).tapped);
    choose(&mut game, |action| {
        matches!(action,
        Action::Activate { card, ability: 0, .. } | Action::AnnounceActivate { card, ability: 0, .. }
        if *card == harvest)
    });
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(&mut game);
    }
    assert_eq!(game.object_view(harvest).zone, Zone::Graveyard);
    assert_eq!(game.state.players[player.index()].life, 20);
    search_land(&mut game, second);
    assert_eq!(game.object_view(second).zone, Zone::Battlefield);
    assert!(game.object_view(second).tapped);
    assert_eq!(game.state.players[player.index()].life, 23);
}

#[test]
fn mistbreath_and_scribe_observe_actual_returns_and_scribe_observes_its_own_departure() {
    let player = Player::P1;
    let mut game = scenario(player);
    let elder = put(&mut game, player, "Mistbreath Elder", Zone::Battlefield);
    let mut alone = game.clone();
    alone.begin_step(Step::Upkeep);
    alone.prepare_triggers();
    drain(&mut alone, |action| {
        *action == Action::ChooseOptional { accept: true }
    });
    assert_eq!(alone.object_view(elder).zone, Zone::Hand);

    let scribe = put(&mut game, player, "Three Tree Scribe", Zone::Battlefield);
    let friend = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    game.begin_step(Step::Upkeep);
    game.prepare_triggers();
    let pick = Action::ChooseEffectObject {
        object: game.object_ref(friend),
    };
    drain(&mut game, |action| {
        *action == pick || *action == target(scribe)
    });
    assert_eq!(game.object_view(friend).zone, Zone::Hand);
    assert_eq!(game.counter_count(elder, CounterKind::PlusOne), 1);
    assert_eq!(game.counter_count(scribe, CounterKind::PlusOne), 1);

    game.state.step = Step::Main1;
    let sentinel = cast_card(&mut game, player, "Stickytongue Sentinel");
    pass_pair(&mut game);
    act(&mut game, target(scribe));
    drain(&mut game, |action| *action == target(sentinel));
    assert_eq!(game.object_view(scribe).zone, Zone::Hand);
    assert_eq!(game.counter_count(sentinel, CounterKind::PlusOne), 1);
    assert!(game.keywords(sentinel).contains(Keywords::REACH));
}

#[test]
fn druid_and_treeguard_apply_distinct_entry_bonuses_and_mightcaller_counts_one_entrant() {
    let player = Player::P0;
    let mut game = scenario(player);
    let valley = put(&mut game, player, "Valley Mightcaller", Zone::Battlefield);
    let druid = cast_card(&mut game, player, "Sunshower Druid");
    drain(&mut game, |action| *action == target(druid));
    assert_eq!(game.counter_count(druid, CounterKind::PlusOne), 1);
    assert_eq!(game.state.players[player.index()].life, 21);
    assert_eq!(game.counter_count(valley, CounterKind::PlusOne), 1);
    let duo = cast_card(&mut game, player, "Treeguard Duo");
    drain(&mut game, |action| *action == target(druid));
    assert_eq!(game.counter_count(valley, CounterKind::PlusOne), 2);
    assert!(game.keywords(valley).contains(Keywords::TRAMPLE));
    assert_eq!(stats(&game, druid), (4, 6));
    assert!(game.keywords(druid).contains(Keywords::VIGILANCE));
    assert_eq!(stats(&game, duo), (3, 4));
}

#[test]
fn red_entry_cards_use_actual_opponent_life_loss_and_exact_top_exile() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let shock = put(&mut game, player, "Shock", Zone::Hand);
    cast(&mut game, shock, Some(Target::Player(opponent)));
    settle(&mut game);
    game.gain_life(opponent, 3);
    let gecko = put(&mut game, player, "Flamecache Gecko", Zone::Hand);
    game.state.players[player.index()].mana = [0, 0, 0, 1, 0, 1];
    cast(&mut game, gecko, None);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].mana, [0, 0, 1, 1, 0, 0]);

    let discard = put(&mut game, player, "Forest", Zone::Hand);
    choose(&mut game, |action| {
        matches!(action,
        Action::Activate { card, ability: 0, .. } | Action::AnnounceActivate { card, ability: 0, .. }
        if *card == gecko)
    });
    for _ in 0..6 {
        let object = game.object_ref(discard);
        if matches!(game.state.pending, Pending::AdditionalCost { .. }) {
            act(&mut game, Action::ChooseAdditionalCost { object });
        } else if matches!(game.state.pending, Pending::PayCosts { .. }) {
            pay(&mut game);
        } else {
            break;
        }
    }
    settle(&mut game);
    assert_eq!(game.object_view(discard).zone, Zone::Graveyard);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);

    let frilled = cast_card(&mut game, player, "Frilled Sparkshooter");
    settle(&mut game);
    assert_eq!(game.counter_count(frilled, CounterKind::PlusOne), 1);
    assert!(
        game.keywords(frilled)
            .contains(Keywords::REACH.union(Keywords::MENACE))
    );
    let land = put(&mut game, player, "Mountain", Zone::Library);
    cast_card(&mut game, player, "Alania's Pathmaker");
    settle(&mut game);
    let permission = game
        .observe(player)
        .exile_permissions
        .into_iter()
        .find(|p| p.object == game.object_ref(land))
        .unwrap();
    assert_eq!(permission.expiry, ExilePlayExpiry::EndOfNextTurn);
    act(&mut game, Action::PlayLand { card: land });
    assert_eq!(game.object_view(land).zone, Zone::Battlefield);
}
