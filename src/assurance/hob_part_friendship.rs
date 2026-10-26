//! Reveal-until-creature, live land threshold and the printed trigger cap.
use super::*;
fn until(g: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..80 {
        if ready(g) {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("Friendship decision budget {:?}", g.state.pending);
}
fn finish(g: &mut Game) {
    until(g, |g| {
        g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.resolving.is_none()
            && g.state.pending_triggers.is_empty()
    });
}
fn reveal(g: &mut Game) {
    g.prepare_triggers();
    until(g, |g| {
        matches!(g.state.pending, Pending::EffectLibrarySelection { .. })
    });
}
fn pick(g: &mut Game, id: CardId) {
    act(g, Action::SelectLibrary { card: id });
    act(g, Action::FinishLibrarySelection);
}
fn source(g: &mut Game, p: Player) -> CardId {
    put(g, p, "Part in Friendship", Zone::Battlefield)
}
fn die(g: &mut Game, p: Player) -> CardId {
    let id = put(g, p, "Bear Cub", Zone::Battlefield);
    g.move_object(id, Zone::Graveyard);
    id
}
fn land(g: &mut Game, p: Player, n: usize) {
    for _ in 0..n {
        put(g, p, "Forest", Zone::Battlefield);
    }
}
fn top(g: &mut Game, p: Player, name: &str) -> CardId {
    put(g, p, name, Zone::Library)
}
#[test]
fn hob_friendship_paid_enchantment_real_paid_kill_and_public_reveal_both_seats() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let s = put(&mut g, p, "Part in Friendship", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 4];
        cast(&mut g, s, None);
        finish(&mut g);
        assert_eq!(g.object_view(s).zone, Zone::Battlefield);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        land(&mut g, p, 2);
        let deeper = top(&mut g, p, "Giant Growth");
        let c = top(&mut g, p, "Bear Cub");
        let f = top(&mut g, p, "Forest");
        let n = top(&mut g, p, "Shock");
        let victim = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let spell = put(&mut g, p, "Shock", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 0];
        cast(&mut g, spell, Some(Target::Card(victim)));
        reveal(&mut g);
        assert_eq!(g.object_view(victim).zone, Zone::Graveyard);
        for v in [p, p.other()] {
            let view = g.observe(v).library_selection.unwrap();
            assert!(view.public && view.reveal);
            assert_eq!(view.destination, "battlefield");
            assert_eq!(
                view.remaining.iter().map(|o| o.id).collect::<Vec<_>>(),
                vec![n, f, c]
            );
            assert_eq!(view.candidates, vec![g.object_ref(c)]);
            assert!(!view.remaining.iter().any(|o| o.id == deeper));
        }
        reject(&mut g, Action::FinishLibrarySelection);
        reject(&mut g, Action::SelectLibrary { card: f });
        contracts::branch_contract(g.clone());
        let rng = g.state.rng;
        pick(&mut g, c);
        finish(&mut g);
        assert_ne!(
            g.state.rng, rng,
            "random remainder consumes owned randomness"
        );
        assert_eq!(g.object_view(c).zone, Zone::Battlefield);
        assert!(!g.state.objects[c as usize].tapped);
        assert_eq!(g.state.players[p.index()].library.last(), Some(&deeper));
        let mut bottom = g.state.players[p.index()].library[..2].to_vec();
        bottom.sort();
        let mut want = vec![n, f];
        want.sort();
        assert_eq!(bottom, want);
        assert!(g.observe(p.other()).library_selection.is_none());
    }
}
#[test]
fn hob_friendship_land_count_current_controller_nonbasic_and_mana_value_boundary() {
    for p in [Player::P0, Player::P1] {
        for (lands, mv, expected) in [
            (0, "Bear Cub", Zone::Hand),
            (1, "Bear Cub", Zone::Hand),
            (2, "Bear Cub", Zone::Battlefield),
            (3, "Bear Cub", Zone::Battlefield),
            (2, "Adaptive Automaton", Zone::Hand),
        ] {
            let mut g = scenario(p);
            source(&mut g, p);
            land(&mut g, p, lands);
            land(&mut g, p.other(), 5);
            let c = top(&mut g, p, mv);
            die(&mut g, p);
            reveal(&mut g);
            pick(&mut g, c);
            finish(&mut g);
            assert_eq!(g.object_view(c).zone, expected);
            if expected == Zone::Hand {
                assert!(g.observe(p.other()).known_hand.contains(&g.object_ref(c)));
            }
        }
        let mut g = scenario(p);
        source(&mut g, p);
        land(&mut g, p, 1);
        let l = put(&mut g, p.other(), "Raucous Theater", Zone::Battlefield);
        let c = top(&mut g, p, "Bear Cub");
        die(&mut g, p);
        g.change_controller(g.object_ref(l), p);
        reveal(&mut g);
        pick(&mut g, c);
        finish(&mut g);
        assert_eq!(g.object_view(c).zone, Zone::Battlefield);
    }
}
#[test]
fn hob_friendship_no_creature_or_empty_reveals_all_without_draw_or_loss() {
    for p in [Player::P0, Player::P1] {
        for empty in [false, true] {
            let mut g = scenario(p);
            source(&mut g, p);
            if empty {
                let ids = g.state.players[p.index()].library.to_vec();
                for id in ids {
                    g.move_object(id, Zone::Exile);
                }
            }
            let before = g.state.players[p.index()].library.to_vec();
            die(&mut g, p);
            if empty {
                finish(&mut g);
            } else {
                reveal(&mut g);
                for v in [p, p.other()] {
                    let view = g.observe(v).library_selection.unwrap();
                    assert_eq!(view.remaining.len(), 8);
                    assert_eq!((view.minimum, view.maximum), (0, 0));
                }
                act(&mut g, Action::FinishLibrarySelection);
                finish(&mut g);
            }
            let mut got = g.state.players[p.index()].library.to_vec();
            got.sort();
            let mut expected = before;
            expected.sort();
            assert_eq!(got, expected);
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
            assert!(g.outcome().is_none());
        }
    }
}
#[test]
fn hob_friendship_cap_counts_simultaneous_deaths_countered_packets_and_resets() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        source(&mut g, p);
        let a = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let b = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let observers = g.death_trigger_sources();
        let events = g.move_objects_raw(&[a, b], Zone::Graveyard);
        g.finish_zone_events(&events, &observers);
        assert_eq!(g.state.pending_triggers.len(), 1);
        g.prepare_triggers();
        until(&mut g, |g| !g.state.stack.is_empty());
        let id = g.state.stack.last().unwrap().id;
        assert!(g.counter_stack_item(id, false));
        finish(&mut g);
        die(&mut g, p);
        assert!(g.state.pending_triggers.is_empty());
        g.state.active = p.other();
        g.begin_turn();
        g.state.pending = Pending::Priority;
        let c = top(&mut g, p, "Bear Cub");
        die(&mut g, p);
        reveal(&mut g);
        pick(&mut g, c);
        finish(&mut g);
        assert_eq!(g.object_view(c).zone, Zone::Hand);
    }
}
#[test]
fn hob_friendship_tokens_opponents_nondeath_and_replacement_do_not_consume_cap() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        source(&mut g, p);
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.move_object(enemy, Zone::Graveyard);
        g.create_tokens(p, Card::from_name("@token/hob-bear").unwrap(), 1, false);
        let token = (g.state.objects.len() - 1) as CardId;
        g.move_object(token, Zone::Graveyard);
        let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.move_object(own, Zone::Hand);
        assert!(g.state.pending_triggers.is_empty());
        let rip = put(&mut g, p.other(), "Rest in Peace", Zone::Battlefield);
        let replaced = die(&mut g, p);
        assert_eq!(g.object_view(replaced).zone, Zone::Exile);
        assert!(g.state.pending_triggers.is_empty());
        g.move_object(rip, Zone::Exile);
        let c = top(&mut g, p, "Bear Cub");
        let stolen = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.change_controller(g.object_ref(stolen), p);
        g.move_object(stolen, Zone::Graveyard);
        reveal(&mut g);
        pick(&mut g, c);
        finish(&mut g);
        assert_eq!(g.object_view(c).zone, Zone::Hand);
    }
}
#[test]
fn hob_friendship_source_departure_and_controller_change_preserve_packet_controller() {
    for p in [Player::P0, Player::P1] {
        for depart in [false, true] {
            let mut g = scenario(p);
            let s = source(&mut g, p);
            land(&mut g, p, 2);
            let c = top(&mut g, p, "Bear Cub");
            die(&mut g, p);
            if depart {
                g.move_object(s, Zone::Graveyard);
            } else {
                g.change_controller(g.object_ref(s), p.other());
            }
            reveal(&mut g);
            assert_eq!(g.actor(), Some(p));
            pick(&mut g, c);
            finish(&mut g);
            assert_eq!(g.object_view(c).zone, Zone::Battlefield);
            assert_eq!(g.state.objects[c as usize].controller, p);
        }
    }
}
#[test]
fn hob_friendship_paused_entry_owned_remainder_and_entry_choice() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        source(&mut g, p);
        land(&mut g, p, 3);
        let deep = top(&mut g, p, "Shock");
        let c = top(&mut g, p, "Adaptive Automaton");
        let f = top(&mut g, p, "Forest");
        die(&mut g, p);
        reveal(&mut g);
        pick(&mut g, c);
        assert!(
            g.legal_actions()
                .iter()
                .any(|a| matches!(a, Action::ChooseEntryValue { .. }))
        );
        contracts::branch_contract(g.clone());
        reject(&mut g, Action::FinishLibrarySelection);
        choose(
            &mut g,
            |a| matches!(a,Action::ChooseEntryValue{value:EntryChoiceValue::CreatureType(t)} if t.name()=="Dwarf"),
        );
        finish(&mut g);
        assert_eq!(g.object_view(c).zone, Zone::Battlefield);
        assert!(g.has_subtype(c, "Dwarf"));
        assert_eq!(g.state.players[p.index()].library.first(), Some(&f));
        assert_eq!(g.state.players[p.index()].library.last(), Some(&deep));
    }
}
fn animate(g: &mut Game, id: CardId) {
    g.change_object_form(
        g.object_ref(id),
        crate::cards::AnimationSpec {
            artifact: false,
            creature_types: crate::cards::AnimationCreatureTypes::Named(&["Dwarf"]),
            base: Some((2, 2)),
            keywords: Keywords::NONE,
            duration: crate::cards::AnimationDuration::Permanent,
        },
        super::super::characteristic_layers::AnimationTypeMode::AddCreature,
    );
}
#[test]
fn hob_friendship_animated_self_death_and_multiplicity_cap() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let s = source(&mut g, p);
        animate(&mut g, s);
        put(&mut g, p, "Bifur, Melodic Rider", Zone::Battlefield);
        put(&mut g, p, "Goldvein Pick", Zone::Battlefield);
        put(&mut g, p, "Goldvein Pick", Zone::Battlefield);
        g.refresh_enduring_stories();
        assert!(g.state.players[p.index()].enduring_story);
        let c = top(&mut g, p, "Bear Cub");
        g.move_object(s, Zone::Graveyard);
        assert_eq!(g.state.pending_triggers.len(), 1);
        reveal(&mut g);
        assert!(g.state.stack.is_empty());
        pick(&mut g, c);
        finish(&mut g);
        assert_eq!(g.object_view(c).zone, Zone::Hand);
    }
}
#[test]
fn hob_friendship_ability_suppression_does_not_consume_cap_and_new_incarnation_is_fresh() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let s = source(&mut g, p);
        animate(&mut g, s);
        let aura = put(&mut g, p, "Witness Protection", Zone::Hand);
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 0];
        cast(&mut g, aura, Some(Target::Card(s)));
        finish(&mut g);
        assert!(!g.printed_abilities_available(s));
        die(&mut g, p);
        assert!(g.state.pending_triggers.is_empty());
        g.move_object(aura, Zone::Graveyard);
        let c = top(&mut g, p, "Bear Cub");
        die(&mut g, p);
        reveal(&mut g);
        pick(&mut g, c);
        finish(&mut g);
        g.move_object(s, Zone::Hand);
        g.move_object(s, Zone::Battlefield);
        let d = top(&mut g, p, "Bear Cub");
        die(&mut g, p);
        reveal(&mut g);
        pick(&mut g, d);
        finish(&mut g);
        assert_eq!(g.object_view(d).zone, Zone::Hand);
    }
}

#[test]
fn hob_friendship_x_is_zero_in_library_and_zero_toughness_entry_still_spends_cap() {
    for p in [Player::P0, Player::P1] {
        for lands in [0, 1] {
            let mut g = scenario(p);
            source(&mut g, p);
            land(&mut g, p, lands);
            let c = top(&mut g, p, "Wildwood Scourge");
            let rest = top(&mut g, p, "Shock");
            die(&mut g, p);
            reveal(&mut g);
            assert_eq!(
                g.observe(p).library_selection.unwrap().destination,
                if lands == 0 { "hand" } else { "battlefield" }
            );
            pick(&mut g, c);
            finish(&mut g);
            assert_eq!(
                g.object_view(c).zone,
                if lands == 0 {
                    Zone::Hand
                } else {
                    Zone::Graveyard
                }
            );
            assert_eq!(g.state.players[p.index()].library.first(), Some(&rest));
            assert!(g.state.pending_triggers.is_empty());
        }
    }
}
