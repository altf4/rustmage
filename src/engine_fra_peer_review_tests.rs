//! Prepared Peer Review face identities and complete printed outcomes.
use super::*;

fn announce_peer(g: &mut Game, action: Action) {
    act(g, action);
    for _ in 0..16 {
        if g.state.announcing.is_none() {
            return;
        }
        choose(g, |a| {
            matches!(
                a,
                Action::ChooseSpellCost { .. } | Action::FinishTargets | Action::PayCosts { .. }
            )
        });
    }
    panic!("announcement did not finish");
}

fn finish_peer(g: &mut Game) {
    for _ in 0..100 {
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.resolving.is_none()
            && g.state.pending_triggers.is_empty()
        {
            return;
        }
        choose(g, |a| {
            matches!(
                a,
                Action::Pass
                    | Action::PutTrigger { .. }
                    | Action::ArrangeLibrary {
                        destination: LibraryDestination::Top,
                        ..
                    }
            )
        });
    }
    panic!("resolution did not finish");
}

#[test]
fn fra_peer_review_faces_are_independent_and_pay_both_hybrid_colors() {
    for p in [Player::P0, Player::P1] {
        for blue in [false, true] {
            for (name, face, size) in [
                ("Fatehold Chronologist // Peer Review", Card(5444), (1, 2)),
                ("Semester Foreseer // Peer Review", Card(5445), (3, 4)),
                ("Prudent Fateseer // Peer Review", Card(5446), (1, 4)),
            ] {
                let mut g = scenario(p);
                let source = put(&mut g, p, name, Zone::Hand);
                announce_peer(&mut g, Action::AnnounceCast { card: source });
                finish_peer(&mut g);
                assert_eq!(stats(&g, source), size);
                let permanent = g.object_ref(source);
                let prepared = g
                    .state
                    .preparations
                    .as_ref()
                    .unwrap()
                    .iter()
                    .find(|r| r.permanent == permanent)
                    .unwrap()
                    .spell_copy;
                assert_eq!(g.state.objects[prepared.card as usize].card, face);
                let top = put(&mut g, p, "Island", Zone::Library);
                g.state.players[p.index()].mana = if blue {
                    [0, 1, 0, 0, 0, 2]
                } else {
                    [1, 0, 0, 0, 0, 2]
                };
                super::super::contracts::branch_contract(g.clone());
                announce_peer(
                    &mut g,
                    Action::AnnounceExile {
                        card: prepared.card,
                    },
                );
                assert!(!g.is_prepared(permanent));
                finish_peer(&mut g);
                assert_eq!(g.state.objects[prepared.card as usize].zone, Zone::Outside);
                assert_eq!(g.state.players[p.index()].library.last(), Some(&top));
                let cadets: Vec<_> = g
                    .state
                    .objects
                    .iter()
                    .enumerate()
                    .filter(|(_, o)| {
                        o.zone == Zone::Battlefield && o.controller == p && o.card == Card(5298)
                    })
                    .map(|(id, _)| id as CardId)
                    .collect();
                assert_eq!(cadets.len(), 1);
                assert_eq!(
                    stats(&g, cadets[0]),
                    if face == Card(5446) { (3, 2) } else { (2, 2) }
                );
                assert!(g.observe(p.other()).players[p.index()].scried_or_surveilled_this_turn);
            }
        }
    }
}

#[test]
fn fra_peer_review_sorcery_timing_and_failed_payment_keep_prepared_copy() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(
            &mut g,
            p,
            "Fatehold Chronologist // Peer Review",
            Zone::Hand,
        );
        announce_peer(&mut g, Action::AnnounceCast { card: source });
        finish_peer(&mut g);
        let permanent = g.object_ref(source);
        let copy = g
            .state
            .preparations
            .as_ref()
            .unwrap()
            .iter()
            .find(|r| r.permanent == permanent)
            .unwrap()
            .spell_copy;
        g.state.active = p.other();
        g.state.priority = p;
        reject(&mut g, Action::AnnounceExile { card: copy.card });
        assert!(g.is_prepared(permanent));
        g.state.active = p;
        g.state.players[p.index()].mana = [0; 6];
        reject(&mut g, Action::AnnounceExile { card: copy.card });
        assert!(g.is_prepared(permanent));
        assert_eq!(g.object_ref(copy.card), copy);
    }
}

#[test]
fn fra_prudent_fateseer_triggers_once_and_only_current_creatures_grow() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Prudent Fateseer // Peer Review", Zone::Hand);
        announce_peer(&mut g, Action::AnnounceCast { card: source });
        finish_peer(&mut g);
        let copy = g
            .state
            .preparations
            .as_ref()
            .unwrap()
            .iter()
            .find(|r| r.permanent == g.object_ref(source))
            .unwrap()
            .spell_copy;
        announce_peer(&mut g, Action::AnnounceExile { card: copy.card });
        finish_peer(&mut g);
        assert_eq!(stats(&g, source), (2, 4));
        let late = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let other = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let second = put(&mut g, p, "Plunge into Winter", Zone::Hand);
        announce_peer(&mut g, Action::AnnounceCast { card: second });
        finish_peer(&mut g);
        assert_eq!(stats(&g, source), (2, 4));
        assert_eq!(stats(&g, late), (2, 2));
        assert_eq!(stats(&g, other), (2, 2));
        g.finish_cleanup();
        assert_eq!(stats(&g, source), (1, 4));
    }
}

#[test]
fn fra_peer_creature_hybrid_costs_accept_each_printed_combination() {
    for p in [Player::P0, Player::P1] {
        for (name, white, blue, generic) in [
            ("Fatehold Chronologist // Peer Review", 1, 0, 1),
            ("Fatehold Chronologist // Peer Review", 0, 1, 1),
            ("Prudent Fateseer // Peer Review", 2, 0, 1),
            ("Prudent Fateseer // Peer Review", 1, 1, 1),
            ("Prudent Fateseer // Peer Review", 0, 2, 1),
            ("Semester Foreseer // Peer Review", 0, 1, 3),
        ] {
            let mut g = scenario(p);
            let source = put(&mut g, p, name, Zone::Hand);
            g.state.players[p.index()].mana = [white, blue, 0, 0, 0, generic];
            announce_peer(&mut g, Action::AnnounceCast { card: source });
            finish_peer(&mut g);
            assert_eq!(g.state.objects[source as usize].zone, Zone::Battlefield);
            assert!(g.is_prepared(g.object_ref(source)));
        }
    }
}

#[test]
fn fra_peer_copy_follows_control_and_is_removed_when_source_leaves() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(
            &mut g,
            p,
            "Fatehold Chronologist // Peer Review",
            Zone::Hand,
        );
        announce_peer(&mut g, Action::AnnounceCast { card: source });
        finish_peer(&mut g);
        let original = g.object_ref(source);
        let copy = g
            .state
            .preparations
            .as_ref()
            .unwrap()
            .iter()
            .find(|r| r.permanent == original)
            .unwrap()
            .spell_copy;
        g.change_controller(original, p.other());
        assert!(!g.can_cast_prepared(p, copy.card));
        assert!(g.can_cast_prepared(p.other(), copy.card));
        super::super::contracts::branch_contract(g.clone());
        g.move_object(source, Zone::Exile);
        assert!(!g.is_prepared(original));
        assert_eq!(g.state.objects[copy.card as usize].zone, Zone::Outside);
        g.move_object(source, Zone::Battlefield);
        let replacement = g.object_ref(source);
        assert_ne!(original, replacement);
        assert!(g.is_prepared(replacement));
        let next = g
            .state
            .preparations
            .as_ref()
            .unwrap()
            .iter()
            .find(|r| r.permanent == replacement)
            .unwrap()
            .spell_copy;
        assert_ne!(copy, next);
        assert_eq!(g.state.objects[next.card as usize].card, Card(5444));
    }
}
