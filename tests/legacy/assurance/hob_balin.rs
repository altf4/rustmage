//! Balin's actual discards, independent entry packets, and paused private draws.
use super::*;
fn until(g: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..64 {
        if ready(g) {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("Balin decision budget: {:?}", g.state.pending);
}
fn optional(g: &mut Game) {
    g.prepare_triggers();
    until(g, |g| {
        g.legal_actions()
            .contains(&Action::ChooseOptional { accept: true })
    });
}
fn finish(g: &mut Game) {
    until(g, |g| {
        g.state.stack.is_empty()
            && g.state.resolving.is_none()
            && g.state.pending == Pending::Priority
            && g.state.pending_triggers.is_empty()
    });
}
fn source(g: &mut Game, p: Player) -> CardId {
    put(g, p, "Balin, Loremaster", Zone::Battlefield)
}
fn hand(g: &mut Game, p: Player, n: usize) -> Vec<CardId> {
    (0..n).map(|_| put(g, p, "Shock", Zone::Hand)).collect()
}
fn story(g: &mut Game, p: Player) {
    put(g, p, "Goldvein Pick", Zone::Battlefield);
    put(g, p, "Goldvein Pick", Zone::Battlefield);
    g.refresh_enduring_stories();
    assert!(g.state.players[p.index()].enduring_story);
}
fn dwarf(g: &mut Game, p: Player) {
    let token = Card::from_name("@token/hob-dwarf").unwrap();
    g.create_tokens(p, token, 1, false);
}
fn private(g: &Game, p: Player) {
    for o in g
        .state
        .objects
        .iter()
        .enumerate()
        .filter(|(_, o)| o.zone == Zone::Hand && o.owner == p)
    {
        assert!(g.observe(p).objects.iter().any(|v| v.id == o.0 as CardId));
        assert!(
            g.observe(p.other())
                .objects
                .iter()
                .all(|v| v.id != o.0 as CardId)
        );
    }
}
#[test]
fn hob_balin_paid_body_self_entry_and_optional_zero_decline_or_whole_hand() {
    for p in [Player::P0, Player::P1] {
        for (n, accept) in [(0, true), (2, false), (3, true)] {
            let mut g = scenario(p);
            let id = put(&mut g, p, "Balin, Loremaster", Zone::Hand);
            let old = hand(&mut g, p, n);
            g.state.players[p.index()].mana = [0, 0, 0, 2, 0, 3];
            cast(&mut g, id, None);
            optional(&mut g);
            assert_eq!(stats(&g, id), (4, 4));
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert_eq!(g.actor(), Some(p));
            private(&g, p);
            reject(&mut g, Action::Discard { card: id });
            act(&mut g, Action::ChooseOptional { accept });
            finish(&mut g);
            assert_eq!(
                g.state.players[p.index()].cards_discarded_this_turn,
                if accept { n as u64 } else { 0 }
            );
            assert_eq!(
                g.state.players[p.index()].cards_drawn_this_turn,
                if accept { n as u64 } else { 0 }
            );
            assert_eq!(g.state.players[p.other().index()].life, 20);
            for card in old {
                assert_eq!(
                    g.object_view(card).zone,
                    if accept { Zone::Graveyard } else { Zone::Hand }
                );
            }
            private(&g, p);
        }
    }
}
#[test]
fn hob_balin_entry_is_self_or_each_other_current_controlled_dwarf() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        source(&mut g, p);
        dwarf(&mut g, p.other());
        g.create_tokens(p, Card::from_name("@token/hob-bear").unwrap(), 1, false);
        assert!(g.state.pending_triggers.is_empty());
        g.create_tokens(p, Card::from_name("@token/hob-dwarf").unwrap(), 2, false);
        assert_eq!(g.state.pending_triggers.len(), 2);
        optional(&mut g);
        act(&mut g, Action::ChooseOptional { accept: false });
        optional(&mut g);
        act(&mut g, Action::ChooseOptional { accept: false });
        finish(&mut g);
    }
}
fn replace_types(g: &mut Game, id: CardId, types: &'static [&'static str]) {
    let reference = g.object_ref(id);
    let timestamp = g.next_role_timestamp();
    g.state
        .roles
        .get_or_insert_with(Default::default)
        .creature_types
        .push(subtypes::CreatureTypesEffectView {
            recipient: reference,
            source: reference,
            types,
            additional: false,
            timestamp,
        });
}
#[test]
fn hob_balin_self_without_dwarf_and_other_changed_subtypes_use_entry_time_facts() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let b = put(&mut g, p, "Balin, Loremaster", Zone::Hand);
        let events = g.move_objects_raw(&[b], Zone::Battlefield);
        replace_types(&mut g, b, &["Bear"]);
        assert!(!g.has_subtype(b, "Dwarf"));
        g.finish_zone_events(&events, &[]);
        assert_eq!(
            g.state.pending_triggers.len(),
            1,
            "self clause ignores subtype"
        );
        optional(&mut g);
        act(&mut g, Action::ChooseOptional { accept: false });
        finish(&mut g);
        let bear = put(&mut g, p, "Bear Cub", Zone::Hand);
        let events = g.move_objects_raw(&[bear], Zone::Battlefield);
        replace_types(&mut g, bear, &["Dwarf"]);
        g.finish_zone_events(&events, &[]);
        assert_eq!(g.state.pending_triggers.len(), 1);
        replace_types(&mut g, bear, &["Bear"]);
        optional(&mut g);
        act(&mut g, Action::ChooseOptional { accept: false });
        finish(&mut g);
        let other = put(&mut g, p, "Dwarven Mauler", Zone::Hand);
        let events = g.move_objects_raw(&[other], Zone::Battlefield);
        replace_types(&mut g, other, &["Bear"]);
        g.finish_zone_events(&events, &[]);
        assert!(g.state.pending_triggers.is_empty());
    }
}
#[test]
fn hob_balin_story_gained_after_trigger_checks_at_resolution_then_remains_permanent() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let b = source(&mut g, p);
        hand(&mut g, p, 2);
        dwarf(&mut g, p);
        assert!(!g.state.players[p.index()].enduring_story);
        story(&mut g, p);
        optional(&mut g);
        let picks: Vec<_> = g
            .state
            .objects
            .iter()
            .enumerate()
            .filter(|(_, o)| o.zone == Zone::Battlefield && o.card.identity() == "Goldvein Pick")
            .map(|(i, _)| i as CardId)
            .collect();
        for id in picks {
            g.move_object(id, Zone::Graveyard);
        }
        g.move_object(b, Zone::Graveyard);
        act(&mut g, Action::ChooseOptional { accept: true });
        finish(&mut g);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 2);
        assert_eq!(g.state.players[p.other().index()].life, 18);
        assert_eq!(g.object_view(b).zone, Zone::Graveyard);
    }
}
#[test]
fn hob_balin_packet_controller_survives_control_change_and_source_departure() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let b = source(&mut g, p);
        story(&mut g, p);
        hand(&mut g, p, 2);
        let foreign = hand(&mut g, p.other(), 1)[0];
        dwarf(&mut g, p);
        optional(&mut g);
        g.state.objects[b as usize].controller = p.other();
        g.move_object(b, Zone::Exile);
        // The original ability's controller owns both discard and opponent damage.
        act(&mut g, Action::ChooseOptional { accept: true });
        finish(&mut g);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 2);
        assert_eq!(g.state.players[p.other().index()].life, 18);
        assert_eq!(g.object_view(foreign).zone, Zone::Hand);
    }
}
#[test]
fn hob_balin_simultaneous_entries_and_doubled_triggers_get_independent_discard_results() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        source(&mut g, p);
        put(&mut g, p, "Bifur, Melodic Rider", Zone::Battlefield);
        put(&mut g, p, "Goldvein Pick", Zone::Battlefield);
        g.refresh_enduring_stories();
        hand(&mut g, p, 2);
        g.create_tokens(p, Card::from_name("@token/hob-dwarf").unwrap(), 2, false);
        assert_eq!(
            g.state.pending_triggers.len(),
            2,
            "one base packet per entry"
        );
        g.prepare_triggers();
        assert_eq!(
            g.state.stack.len() + g.state.ordering_triggers.len(),
            4,
            "Bifur expands both source-qualified packets before stacking"
        );
        for i in 0..4 {
            optional(&mut g);
            act(&mut g, Action::ChooseOptional { accept: i != 1 });
        }
        finish(&mut g);
        assert_eq!(g.state.players[p.index()].cards_discarded_this_turn, 6);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 6);
        assert_eq!(g.state.players[p.other().index()].life, 14);
    }
}
#[test]
fn hob_balin_replaced_to_exile_discards_still_count_and_private_draws_are_new_cards() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        source(&mut g, p);
        story(&mut g, p);
        put(&mut g, p.other(), "Rest in Peace", Zone::Battlefield);
        let cards = hand(&mut g, p, 3);
        dwarf(&mut g, p);
        optional(&mut g);
        act(&mut g, Action::ChooseOptional { accept: true });
        finish(&mut g);
        assert_eq!(g.state.players[p.index()].cards_discarded_this_turn, 3);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 3);
        assert_eq!(g.state.players[p.other().index()].life, 17);
        for id in cards {
            assert_eq!(g.object_view(id).zone, Zone::Exile);
        }
        private(&g, p);
    }
}
#[test]
fn hob_balin_shuffle_replacement_counts_discard_without_using_card_characteristics() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        source(&mut g, p);
        story(&mut g, p);
        let colossus = put(&mut g, p, "Darksteel Colossus", Zone::Hand);
        hand(&mut g, p, 1);
        dwarf(&mut g, p);
        optional(&mut g);
        act(&mut g, Action::ChooseOptional { accept: true });
        finish(&mut g);
        assert_ne!(g.object_view(colossus).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].cards_discarded_this_turn, 2);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 2);
        assert_eq!(g.state.players[p.other().index()].life, 18);
    }
}
#[test]
fn hob_balin_optional_owned_raw_index_decline_accept_and_restore_diverge_honestly() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        source(&mut g, p);
        story(&mut g, p);
        let cards = hand(&mut g, p, 2);
        dwarf(&mut g, p);
        optional(&mut g);
        contracts::branch_contract(g.clone());
        let saved = g.snapshot();
        let mut raw = g.clone();
        let mut indexed = g.clone();
        drop(g);
        act(&mut raw, Action::ChooseOptional { accept: true });
        finish(&mut raw);
        let index = indexed
            .legal_actions()
            .iter()
            .position(|a| *a == Action::ChooseOptional { accept: true })
            .unwrap();
        indexed.apply_index(p, indexed.revision(), index).unwrap();
        finish(&mut indexed);
        assert_eq!(raw.snapshot(), indexed.snapshot());
        indexed.restore(&saved).unwrap();
        act(&mut indexed, Action::ChooseOptional { accept: false });
        finish(&mut indexed);
        assert_eq!(indexed.state.players[p.other().index()].life, 20);
        assert_eq!(raw.state.players[p.other().index()].life, 18);
        for id in cards {
            assert_eq!(indexed.object_view(id).zone, Zone::Hand);
        }
    }
}
#[test]
fn hob_balin_miracle_draw_pause_keeps_exact_count_and_defers_story_damage_owned_restore() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let b = source(&mut g, p);
        story(&mut g, p);
        put(&mut g, p, "Lorehold, the Historian", Zone::Battlefield);
        let old = g.state.players[p.index()].library.clone();
        for id in old {
            g.move_object(id, Zone::Outside);
        }
        for _ in 0..5 {
            put(&mut g, p, "Shock", Zone::Library);
        }
        hand(&mut g, p, 3);
        dwarf(&mut g, p);
        optional(&mut g);
        act(&mut g, Action::ChooseOptional { accept: true });
        assert!(matches!(g.state.pending, Pending::MiracleReveal { .. }));
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
        assert_eq!(g.state.players[p.other().index()].life, 20);
        assert!(g.observe(p).miracle.unwrap().can_reveal);
        assert!(g.observe(p.other()).miracle.is_none());
        assert_eq!(g.state.resolving.as_ref().unwrap().moved_count, Some(3));
        assert_eq!(g.observe(p).group_move_count, Some(3));
        assert_eq!(
            g.observe(p.other()).group_move_count,
            Some(3),
            "discard amount is public, draw identities remain private"
        );
        g.move_object(b, Zone::Exile);
        contracts::branch_contract(g.clone());
        let saved = g.snapshot();
        let mut child = g.clone();
        drop(g);
        act(&mut child, Action::DeclineMiracle);
        finish(&mut child);
        assert_eq!(child.state.players[p.index()].cards_drawn_this_turn, 3);
        assert_eq!(child.state.players[p.other().index()].life, 17);
        private(&child, p);
        let expected = child.snapshot();
        child.restore(&saved).unwrap();
        act(&mut child, Action::DeclineMiracle);
        finish(&mut child);
        assert_eq!(child.snapshot(), expected);
    }
}
#[test]
fn hob_balin_damage_replacement_changes_damage_without_changing_discard_or_draw_x() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        source(&mut g, p);
        story(&mut g, p);
        put(&mut g, p, "Twinflame Tyrant", Zone::Battlefield);
        hand(&mut g, p, 2);
        dwarf(&mut g, p);
        optional(&mut g);
        act(&mut g, Action::ChooseOptional { accept: true });
        finish(&mut g);
        assert_eq!(g.state.players[p.index()].cards_discarded_this_turn, 2);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 2);
        assert_eq!(g.state.players[p.other().index()].life, 16);
    }
}

#[test]
fn hob_balin_concurrent_self_and_other_entries_observe_the_whole_committed_batch() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let b = put(&mut g, p, "Balin, Loremaster", Zone::Hand);
        let own = put(&mut g, p, "Dwarven Mauler", Zone::Hand);
        let enemy = put(&mut g, p.other(), "Dwarven Mauler", Zone::Hand);
        let bear = put(&mut g, p, "Bear Cub", Zone::Hand);
        hand(&mut g, p, 2);
        let moves = [b, own, enemy, bear]
            .into_iter()
            .map(|id| g.entry_move(id, Zone::Battlefield, false))
            .collect();
        g.stage_entries(moves, None);
        assert_eq!(
            g.state.pending_triggers.len(),
            2,
            "new Balin sees itself and the other own Dwarf exactly once each"
        );
        for _ in 0..2 {
            optional(&mut g);
            act(&mut g, Action::ChooseOptional { accept: true });
        }
        finish(&mut g);
        assert_eq!(g.state.players[p.index()].cards_discarded_this_turn, 4);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 4);
        assert_eq!(g.state.players[p.other().index()].life, 20);
    }
}
