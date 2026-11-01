//! Current printed outcomes for inherited landfall and land-count consumers.
use super::*;
fn until(g: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..32 {
        if ready(g) {
            return;
        }
        choose(g, |a| matches!(a, Action::PutTrigger { .. } | Action::Pass));
    }
    panic!("landfall choice budget");
}
fn drive(g: &mut Game) {
    for _ in 0..64 {
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty()
            && g.state.resolving.is_none()
        {
            return;
        }
        choose(g, |a| matches!(a, Action::PutTrigger { .. } | Action::Pass));
    }
    panic!("landfall resolution budget");
}
fn paid(g: &mut Game, p: Player, name: &str, mana: Mana) -> CardId {
    let id = put(g, p, name, Zone::Hand);
    g.state.players[p.index()].mana = mana;
    cast(g, id, None);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    id
}
fn land(g: &mut Game, p: Player) -> CardId {
    let id = put(g, p, "Forest", Zone::Hand);
    act(g, Action::PlayLand { card: id });
    id
}
fn extra_entry(g: &mut Game, p: Player) -> CardId {
    let id = put(g, p, "Forest", Zone::Hand);
    g.move_object(id, Zone::Battlefield);
    g.give_priority();
    g.prepare_triggers();
    id
}
#[test]
fn hob_inherited_lands_paid_growth_each_own_entry_cleanup_and_exact_source() {
    for p in [Player::P0, Player::P1] {
        for (name, mana, base, delta) in [
            ("Attercop", [0, 0, 0, 0, 1, 1], (2, 1), 1),
            ("Boughside Wanderers", [0, 0, 0, 0, 2, 4], (4, 4), 2),
        ] {
            let mut g = scenario(p);
            let source = paid(&mut g, p, name, mana);
            if name == "Boughside Wanderers" {
                until(&mut g, |g| {
                    g.legal_actions().contains(&Action::FinishLibrarySelection)
                });
                act(&mut g, Action::FinishLibrarySelection);
            }
            drive(&mut g);
            assert_eq!(stats(&g, source), base);
            if name == "Attercop" {
                assert!(
                    g.keywords(source)
                        .contains(Keywords::REACH.union(Keywords::DEATHTOUCH))
                );
            }
            land(&mut g, p);
            if name == "Attercop" {
                let mut stale = g.clone();
                stale.move_object(source, Zone::Hand);
                stale.move_object(source, Zone::Battlefield);
                stale.give_priority();
                stale.prepare_triggers();
                drive(&mut stale);
                assert_eq!(
                    stats(&stale, source),
                    base,
                    "old growth cannot bind a new incarnation"
                );
            }
            drive(&mut g);
            assert_eq!(stats(&g, source), (base.0 + delta, base.1 + delta));
            extra_entry(&mut g, p);
            drive(&mut g);
            assert_eq!(stats(&g, source), (base.0 + 2 * delta, base.1 + 2 * delta));
            extra_entry(&mut g, p.other());
            drive(&mut g);
            assert_eq!(stats(&g, source), (base.0 + 2 * delta, base.1 + 2 * delta));
            g.finish_cleanup();
            assert_eq!(stats(&g, source), base);
        }
    }
}
fn finish_pick(g: &mut Game, action: Action) {
    act(g, action);
    if g.legal_actions().contains(&Action::FinishLibrarySelection) {
        act(g, Action::FinishLibrarySelection);
    }
    drive(g);
}
#[test]
fn hob_inherited_lands_wanderer_private_optional_permanent_reveal_random_bottom_owned() {
    for p in [Player::P0, Player::P1] {
        for pick in [None, Some(0usize), Some(1usize), Some(3usize)] {
            let mut g = scenario(p);
            let below = put(&mut g, p, "Mountain", Zone::Library);
            let mut top: Vec<_> = ["Goldvein Pick", "Island", "Shock", "Grizzly Bears"]
                .into_iter()
                .rev()
                .map(|n| put(&mut g, p, n, Zone::Library))
                .collect();
            top.reverse();
            let source = paid(&mut g, p, "Boughside Wanderers", [0, 0, 0, 0, 2, 4]);
            until(&mut g, |g| {
                g.legal_actions().contains(&Action::FinishLibrarySelection)
            });
            assert!(g.observe(p).library_selection.is_some());
            assert!(g.observe(p.other()).library_selection.is_none());
            for &id in top.iter().chain([below].iter()) {
                assert!(g.observe(p.other()).objects.iter().all(|v| v.id != id));
            }
            reject(&mut g, Action::SelectLibrary { card: top[2] });
            reject(&mut g, Action::SelectLibrary { card: below });
            let action = pick.map_or(Action::FinishLibrarySelection, |i| Action::SelectLibrary {
                card: top[i],
            });
            let saved = g.snapshot();
            let before_rng = g.state.rng;
            if p == Player::P0 && pick == Some(0) {
                let parent = g.clone();
                let mut indexed = parent.clone();
                drop(parent);
                let actor = indexed.actor().unwrap();
                let rev = indexed.revision();
                let index = indexed
                    .legal_actions()
                    .iter()
                    .position(|a| a == &action)
                    .unwrap();
                indexed.apply_index(actor, rev, index).unwrap();
                if indexed
                    .legal_actions()
                    .contains(&Action::FinishLibrarySelection)
                {
                    act(&mut indexed, Action::FinishLibrarySelection);
                }
                drive(&mut indexed);
                finish_pick(&mut g, action.clone());
                assert_eq!(g.snapshot(), indexed.snapshot());
                indexed.restore(&saved).unwrap();
                finish_pick(&mut indexed, action.clone());
                assert_eq!(g.snapshot(), indexed.snapshot());
            } else {
                finish_pick(&mut g, action);
            }
            assert_eq!(stats(&g, source), (4, 4));
            assert_ne!(g.state.rng, before_rng);
            assert_eq!(g.state.players[p.index()].library.last(), Some(&below));
            let expected: std::collections::BTreeSet<_> = top
                .iter()
                .enumerate()
                .filter_map(|(i, id)| (pick != Some(i)).then_some(*id))
                .collect();
            let actual: std::collections::BTreeSet<_> = g.state.players[p.index()]
                .library
                .iter()
                .take(expected.len())
                .copied()
                .collect();
            assert_eq!(
                actual, expected,
                "only looked remainder goes to random bottom"
            );
            for (i, &id) in top.iter().enumerate() {
                assert_eq!(
                    g.object_view(id).zone,
                    if pick == Some(i) {
                        Zone::Hand
                    } else {
                        Zone::Library
                    }
                );
                if pick == Some(i) {
                    assert!(g.observe(p.other()).known_hand.contains(&g.object_ref(id)));
                } else {
                    assert!(g.observe(p.other()).objects.iter().all(|v| v.id != id));
                }
            }
        }
    }
}
#[test]
fn hob_inherited_lands_meditator_optional_base_setting_layers_and_expires() {
    for p in [Player::P0, Player::P1] {
        for accept in [false, true] {
            let mut g = scenario(p);
            let source = paid(&mut g, p, "Mirkwood Meditator", [0, 1, 0, 0, 0, 2]);
            drive(&mut g);
            assert_eq!(stats(&g, source), (2, 4));
            g.state.objects[source as usize].counters.plus_one = 1;
            extra_entry(&mut g, p.other());
            drive(&mut g);
            assert_eq!(stats(&g, source), (3, 5));
            land(&mut g, p);
            until(&mut g, |g| {
                g.legal_actions()
                    .contains(&Action::ChooseOptional { accept })
            });
            act(&mut g, Action::ChooseOptional { accept });
            drive(&mut g);
            let expected = if accept { (5, 3) } else { (3, 5) };
            assert_eq!(stats(&g, source), expected);
            extra_entry(&mut g, p);
            until(&mut g, |g| {
                g.legal_actions()
                    .contains(&Action::ChooseOptional { accept })
            });
            act(&mut g, Action::ChooseOptional { accept });
            drive(&mut g);
            assert_eq!(
                stats(&g, source),
                expected,
                "base setting does not add another4/2"
            );
            let growth = put(&mut g, p, "Giant Growth", Zone::Hand);
            g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 0];
            cast(&mut g, growth, Some(Target::Card(source)));
            drive(&mut g);
            assert_eq!(stats(&g, source), (expected.0 + 3, expected.1 + 3));
            g.finish_cleanup();
            assert_eq!(
                stats(&g, source),
                (3, 5),
                "counter persists; base setting and growth expire"
            );
        }
    }
}
#[test]
fn hob_inherited_lands_raft_both_modes_controller_legality_and_stale_targets() {
    for p in [Player::P0, Player::P1] {
        for mode in [0, 1] {
            for stale in [false, true] {
                let mut g = scenario(p);
                let source = paid(&mut g, p, "Elven Raft-Steerer", [0, 1, 0, 0, 0, 2]);
                drive(&mut g);
                assert_eq!(stats(&g, source), (3, 2));
                let own = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
                g.state.objects[own as usize].tapped = true;
                let enemy = put(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
                extra_entry(&mut g, p.other());
                drive(&mut g);
                assert!(g.object_view(own).tapped);
                assert!(!g.object_view(enemy).tapped);
                land(&mut g, p);
                until(&mut g, |g| {
                    g.legal_actions().contains(&Action::ChooseMode { mode })
                });
                act(&mut g, Action::ChooseMode { mode });
                let wanted = if mode == 0 { enemy } else { own };
                let wrong = if mode == 0 { own } else { enemy };
                reject(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(wrong),
                    },
                );
                act(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(wanted),
                    },
                );
                if g.legal_actions().contains(&Action::FinishTargets) {
                    act(&mut g, Action::FinishTargets);
                }
                if stale {
                    g.move_object(wanted, Zone::Hand);
                    g.move_object(wanted, Zone::Battlefield);
                    g.state.objects[wanted as usize].tapped = mode == 1;
                    g.give_priority();
                } else {
                    g.move_object(source, Zone::Graveyard);
                    g.give_priority();
                }
                drive(&mut g);
                assert_eq!(
                    g.object_view(wanted).tapped,
                    if stale { mode == 1 } else { mode == 0 }
                );
            }
        }
    }
}
#[test]
fn hob_inherited_lands_pathmaker_cda_all_zones_live_control_and_owned_views() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let one = put(&mut g, p, "Forest", Zone::Battlefield);
        let two = put(&mut g, p, "Forest", Zone::Battlefield);
        let steal = put(&mut g, p.other(), "Forest", Zone::Battlefield);
        put(&mut g, p.other(), "Forest", Zone::Battlefield);
        let source = put(&mut g, p, "Mirkwood Pathmaker", Zone::Hand);
        assert_eq!(stats(&g, source), (2, 2));
        g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 2];
        cast(&mut g, source, None);
        assert_eq!(g.object_view(source).zone, Zone::Stack);
        assert_eq!(stats(&g, source), (2, 2));
        drive(&mut g);
        assert_eq!(stats(&g, source), (2, 2));
        land(&mut g, p);
        drive(&mut g);
        assert_eq!(stats(&g, source), (3, 3));
        let parent = g.clone();
        let steal_ref = g.object_ref(steal);
        g.record_control(steal_ref, p, None);
        assert_eq!(stats(&g, source), (4, 4));
        assert_eq!(stats(&parent, source), (3, 3));
        let one_ref = g.object_ref(one);
        g.record_control(one_ref, p.other(), None);
        assert_eq!(stats(&g, source), (3, 3));
        let source_ref = g.object_ref(source);
        g.record_control(source_ref, p.other(), None);
        assert_eq!(stats(&g, source), (2, 2), "use current controller's lands");
        g.record_control(source_ref, p, None);
        assert_eq!(stats(&g, source), (3, 3));
        g.move_object(two, Zone::Hand);
        g.give_priority();
        assert_eq!(stats(&g, source), (2, 2));
        g.move_object(source, Zone::Hand);
        assert_eq!(stats(&g, source), (2, 2));
        g.move_object(source, Zone::Graveyard);
        assert_eq!(stats(&g, source), (2, 2));
        let mut zero = scenario(p);
        let zero_source = paid(&mut zero, p, "Mirkwood Pathmaker", [0, 0, 0, 0, 1, 2]);
        assert_eq!(stats(&zero, zero_source), (0, 0));
        drive(&mut zero);
        assert_eq!(zero.object_view(zero_source).zone, Zone::Graveyard);
        assert_eq!(stats(&zero, zero_source), (0, 0));
    }
}
