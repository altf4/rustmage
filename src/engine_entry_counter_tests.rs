use super::tests::{act, scenario, spawn};
use super::*;

fn named(g: &mut Game, player: Player, name: &str, zone: Zone) -> CardId {
    let card = Card::from_name(name).unwrap();
    assert!(card.is_supported(), "{name}");
    spawn(g, player, card, zone)
}
fn replay(g: &mut Game, action: Action) {
    let saved = g.snapshot();
    let revision = g.revision();
    let actor = g.actor().unwrap();
    let mut fork = g.clone();
    act(&mut fork, action.clone());
    act(g, action.clone());
    let expected = g.snapshot();
    assert_eq!(fork.snapshot(), expected);
    g.restore(&saved).unwrap();
    assert_eq!(g.apply_index(actor, revision, 0), Err(Error::StaleRevision));
    act(g, action);
    assert_eq!(g.snapshot(), expected);
}
fn cast(g: &mut Game, id: CardId) {
    let (mut mana, generic) = g.state.objects[id as usize].card.cost();
    mana[5] += generic;
    g.state.players[g.state.priority.index()].mana = mana;
    let action = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::Cast { card, target: None, .. } if *card == id))
        .expect("legal cast");
    replay(g, action);
}
fn resolve(g: &mut Game) {
    replay(g, Action::Pass);
    replay(g, Action::Pass);
}
fn stats(g: &Game, id: CardId, counters: u64, power: i128, toughness: i128) {
    let o = g.object_view(id);
    assert_eq!(o.zone, Zone::Battlefield);
    assert_eq!(o.counters.plus_one, counters);
    assert_eq!(o.power, power);
    assert_eq!(o.toughness, toughness);
}

#[test]
fn raid_entry_counters_exist_before_priority_and_only_use_controllers_attack() {
    for attacked in [false, true] {
        let mut g = scenario();
        // Commit real attackers for the selected player, then cast in main two.
        let attacker = named(
            &mut g,
            if attacked { Player::P0 } else { Player::P1 },
            "Ordinary Bear",
            Zone::Battlefield,
        );
        g.state.active = if attacked { Player::P0 } else { Player::P1 };
        g.state.pending = Pending::Attackers;
        g.state.step = Step::Attackers;
        replay(&mut g, Action::Attack { card: attacker });
        replay(&mut g, Action::FinishAttackers);
        g.state.active = Player::P0;
        g.state.step = Step::Main2;
        g.give_priority();
        let id = named(&mut g, Player::P0, "Goblin Boarders", Zone::Hand);
        cast(&mut g, id);
        assert_eq!(g.object_view(id).zone, Zone::Stack);
        assert_eq!(g.object_view(id).counters.plus_one, 0);
        resolve(&mut g);
        let n = u64::from(attacked);
        stats(&g, id, n, 3 + i128::from(n), 2 + i128::from(n));
        assert!(g.state.pending_triggers.is_empty() && g.state.stack.is_empty());
        assert_eq!(g.observe(Player::P1).schema, 42);
    }
}

#[test]
fn master_counts_its_own_completed_cast_and_ignores_opponent_casts() {
    for previous in [0, 1] {
        let mut g = scenario();
        g.state.players[1].spells_cast_this_turn = 9;
        if previous == 1 {
            let first = named(&mut g, Player::P0, "Goblin Boarders", Zone::Hand);
            cast(&mut g, first);
            resolve(&mut g);
        }
        let id = named(&mut g, Player::P0, "Effortless Master", Zone::Hand);
        cast(&mut g, id);
        assert_eq!(g.state.players[0].spells_cast_this_turn, previous + 1);
        resolve(&mut g);
        let n = if previous == 1 { 2 } else { 0 };
        stats(&g, id, n, 4 + i128::from(n), 3 + i128::from(n));
        assert!(g.keywords(id).contains(Keywords::VIGILANCE));
        assert!(g.keywords(id).contains(Keywords::MENACE));
        g.begin_turn();
        assert_eq!(g.state.players[0].spells_cast_this_turn, 0);
        assert_eq!(g.object_view(id).counters.plus_one, n);
    }
}

#[test]
fn breacher_counts_controlled_union_once_at_resolution_not_announcement() {
    let mut g = scenario();
    let existing = named(&mut g, Player::P0, "Ordinary Bear", Zone::Battlefield);
    named(&mut g, Player::P0, "Hullcarver", Zone::Battlefield);
    named(&mut g, Player::P0, "Gold Pan", Zone::Battlefield);
    named(&mut g, Player::P0, "Forest", Zone::Battlefield);
    named(&mut g, Player::P1, "Ordinary Bear", Zone::Battlefield);
    named(&mut g, Player::P1, "Gold Pan", Zone::Battlefield);
    let id = named(&mut g, Player::P0, "Luxknight Breacher", Zone::Hand);
    cast(&mut g, id);
    g.move_object(existing, Zone::Graveyard);
    resolve(&mut g);
    stats(&g, id, 2, 4, 4);
    assert!(g.state.pending_triggers.is_empty());
}

#[test]
fn simultaneous_noncasting_entries_freeze_quantities_and_are_order_independent() {
    for reverse in [false, true] {
        let mut g = scenario();
        named(&mut g, Player::P0, "Ordinary Bear", Zone::Battlefield);
        named(&mut g, Player::P0, "Gold Pan", Zone::Battlefield);
        let a = named(&mut g, Player::P0, "Luxknight Breacher", Zone::Graveyard);
        let b = named(&mut g, Player::P0, "Luxknight Breacher", Zone::Exile);
        let c = named(&mut g, Player::P0, "Ordinary Bear", Zone::Hand);
        let mut entries = vec![
            g.entry_move(a, Zone::Battlefield, false),
            g.entry_move(b, Zone::Battlefield, false),
            g.entry_move(c, Zone::Battlefield, false),
        ];
        if reverse {
            entries.reverse();
        }
        let saved = g.snapshot();
        let mut fork = g.clone();
        g.stage_entries(entries.clone(), None);
        fork.stage_entries(entries.clone(), None);
        let expected = g.snapshot();
        assert_eq!(expected, fork.snapshot());
        stats(&g, a, 2, 4, 4);
        stats(&g, b, 2, 4, 4);
        g.restore(&saved).unwrap();
        g.stage_entries(entries, None);
        assert_eq!(g.snapshot(), expected);
        // A later entry sees all three permanents from the first batch.
        let later = named(&mut g, Player::P0, "Luxknight Breacher", Zone::Graveyard);
        g.move_object(later, Zone::Battlefield);
        stats(&g, later, 5, 7, 7);
    }
}

#[test]
fn conditional_noncasting_entries_use_history_without_incrementing_casts_and_reset_on_reentry() {
    let mut g = scenario();
    let master = named(&mut g, Player::P0, "Effortless Master", Zone::Graveyard);
    let boarders = named(&mut g, Player::P0, "Goblin Boarders", Zone::Graveyard);
    g.state.players[0].spells_cast_this_turn = 2;
    g.state.players[0].attacked_this_turn = true;
    let entries = vec![
        g.entry_move(master, Zone::Battlefield, false),
        g.entry_move(boarders, Zone::Battlefield, false),
    ];
    g.stage_entries(entries, None);
    stats(&g, master, 2, 6, 5);
    stats(&g, boarders, 1, 4, 3);
    assert_eq!(g.state.players[0].spells_cast_this_turn, 2);
    g.move_object(master, Zone::Hand);
    g.move_object(boarders, Zone::Hand);
    assert_eq!(g.object_view(master).counters, Counters::default());
    assert_eq!(g.object_view(boarders).counters, Counters::default());
    g.begin_turn();
    g.move_object(master, Zone::Battlefield);
    g.move_object(boarders, Zone::Battlefield);
    stats(&g, master, 0, 4, 3);
    stats(&g, boarders, 0, 3, 2);
}

#[test]
fn fixed_entry_counters_and_loyalty_survive_the_shared_pre_entry_path() {
    let mut g = scenario();
    let fixed = named(&mut g, Player::P0, "Burdened Stoneback", Zone::Hand);
    let walker = named(&mut g, Player::P0, "Professor Dellian Fel", Zone::Hand);
    let entries = vec![
        g.entry_move(fixed, Zone::Battlefield, false),
        g.entry_move(walker, Zone::Battlefield, false),
    ];
    g.stage_entries(entries, None);
    assert_eq!(g.object_view(fixed).counters.minus_one, 2);
    assert_eq!(
        g.object_view(walker).counters.loyalty,
        g.object_view(walker).card.starting_loyalty()
    );
}

#[test]
fn suspended_life_entry_batch_keeps_all_objects_outside_until_commit_and_replays() {
    let mut g = scenario();
    named(&mut g, Player::P0, "Gold Pan", Zone::Battlefield);
    let breacher = named(&mut g, Player::P0, "Luxknight Breacher", Zone::Graveyard);
    let shock = named(&mut g, Player::P0, "Temple Garden", Zone::Hand);
    let incoming = named(&mut g, Player::P0, "Ordinary Bear", Zone::Graveyard);
    let entries = vec![
        g.entry_move(shock, Zone::Battlefield, false),
        g.entry_move(incoming, Zone::Battlefield, false),
        g.entry_move(breacher, Zone::Battlefield, false),
    ];
    g.stage_entries(entries, None);
    assert!(g.advance_entries());
    assert_eq!(g.object_view(breacher).zone, Zone::Graveyard);
    assert_eq!(g.object_view(incoming).zone, Zone::Graveyard);
    assert_eq!(g.observe(Player::P0).decision, "entry_life");
    replay(&mut g, Action::ChooseEntryLife { pay: true });
    assert!(g.state.entry_batch.is_none());
    assert_eq!(g.state.players[0].life, 18);
    stats(&g, breacher, 1, 3, 3);
    assert_eq!(g.object_view(incoming).zone, Zone::Battlefield);
    assert_eq!(g.object_view(shock).zone, Zone::Battlefield);
}

#[test]
fn raw_simultaneous_helper_also_freezes_all_counter_quantities() {
    for reverse in [false, true] {
        let mut g = scenario();
        named(&mut g, Player::P0, "Gold Pan", Zone::Battlefield);
        let a = named(&mut g, Player::P0, "Luxknight Breacher", Zone::Graveyard);
        let b = named(&mut g, Player::P0, "Luxknight Breacher", Zone::Hand);
        let bear = named(&mut g, Player::P0, "Ordinary Bear", Zone::Hand);
        let mut ids = [a, b, bear];
        if reverse {
            ids.reverse();
        }
        let saved = g.snapshot();
        let mut fork = g.clone();
        let events = g.move_objects_raw(&ids, Zone::Battlefield);
        g.finish_zone_events(&events, &[]);
        let other = fork.move_objects_raw(&ids, Zone::Battlefield);
        fork.finish_zone_events(&other, &[]);
        let expected = g.snapshot();
        assert_eq!(expected, fork.snapshot());
        stats(&g, a, 1, 3, 3);
        stats(&g, b, 1, 3, 3);
        g.restore(&saved).unwrap();
        let events = g.move_objects_raw(&ids, Zone::Battlefield);
        g.finish_zone_events(&events, &[]);
        assert_eq!(g.snapshot(), expected);
    }
}
