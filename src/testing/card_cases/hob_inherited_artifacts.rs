//! Focused printed outcomes; shared engine invariants live in core tests.
use super::*;
fn until(g: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..48 {
        if ready(g) {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("artifact decision budget {:?}", g.state.pending);
}
fn finish(g: &mut Game) {
    for _ in 0..48 {
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.resolving.is_none()
            && g.state.pending_triggers.is_empty()
            && g.state.ordering_triggers.is_empty()
        {
            return;
        }
        choose(g, |a| {
            matches!(
                a,
                Action::Pass | Action::PutTrigger { .. } | Action::FinishTargets
            )
        });
    }
    panic!("artifact resolution budget {:?}", g.state.pending);
}
fn paid(g: &mut Game, p: Player, name: &str, mana: Mana) -> CardId {
    let id = put(g, p, name, Zone::Hand);
    g.state.players[p.index()].mana = mana;
    cast(g, id, None);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    id
}
fn bind(g: &mut Game, target: Target) {
    act(g, Action::ChooseTarget { target });
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
    if g.legal_actions()
        .iter()
        .any(|a| matches!(a, Action::PayCosts { .. }))
    {
        pay(g);
    }
}
fn announce(g: &mut Game, id: CardId) {
    act(
        g,
        Action::AnnounceActivate {
            card: id,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
}
fn library(g: &mut Game, p: Player, names: &[&str]) -> Vec<CardId> {
    for id in g.state.players[p.index()].library.iter().copied() {
        g.state.objects[id as usize].zone = Zone::Outside;
    }
    g.state.players[p.index()].library.clear();
    names
        .iter()
        .map(|name| put(g, p, name, Zone::Library))
        .collect()
}

#[test]
fn hob_inherited_artifacts_four_paid_equips_keep_stats_control_restrictions_and_timing() {
    {
        let p = Player::P0;
        for (name, cost, expected) in [
            ("Crude Bent Blade", 2, (4, 3)),
            ("Dwarven Mattock", 3, (4, 4)),
            ("Ragged Short Spear", 3, (4, 2)),
            ("Well-Worn Spatula", 1, (3, 3)),
        ] {
            let mut g = scenario(p);
            let equipment = put(&mut g, p, name, Zone::Battlefield);
            let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            g.state.objects[equipment as usize].owner = p.other();
            g.state.objects[bear as usize].owner = p.other();
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, cost - 1];
            reject(
                &mut g,
                Action::AnnounceActivate {
                    card: equipment,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, cost];
            g.state.active = p.other();
            g.state.priority = p;
            reject(
                &mut g,
                Action::AnnounceActivate {
                    card: equipment,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            g.state.active = p;
            g.state.priority = p;
            announce(&mut g, equipment);
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(enemy),
                },
            );
            bind(&mut g, Target::Card(bear));
            finish(&mut g);
            assert_eq!(
                g.object_view(equipment).attached_to,
                Some(g.object_ref(bear)),
                "{name}"
            );
            assert_eq!(stats(&g, bear), expected, "{name}");
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            g.move_object(equipment, Zone::Graveyard);
            assert_eq!(stats(&g, bear), (2, 2));
            assert_eq!(g.state.objects[equipment as usize].owner, p.other());
        }
    }
}

#[test]
fn hob_inherited_artifacts_mattock_automatic_attachment_only_targets_own_dwarf() {
    {
        let p = Player::P0;
        for stale in [false, true] {
            let mut g = scenario(p);
            let dwarf = put(&mut g, p, "Axgard Cavalry", Zone::Battlefield);
            let enemy = put(&mut g, p.other(), "Axgard Cavalry", Zone::Battlefield);
            let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let mattock = paid(&mut g, p, "Dwarven Mattock", [0, 0, 0, 0, 0, 2]);
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::Targets { .. })
            });
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(enemy),
                },
            );
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(bear),
                },
            );
            bind(&mut g, Target::Card(dwarf));
            if stale {
                g.move_object(dwarf, Zone::Hand);
                g.move_object(dwarf, Zone::Battlefield);
            }
            finish(&mut g);
            assert_eq!(
                g.object_view(mattock).attached_to,
                if stale {
                    None
                } else {
                    Some(g.object_ref(dwarf))
                }
            );
            assert_eq!(stats(&g, dwarf), if stale { (2, 2) } else { (4, 4) });
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        }
        let mut g = scenario(p);
        put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let mattock = paid(&mut g, p, "Dwarven Mattock", [0, 0, 0, 0, 0, 2]);
        finish(&mut g);
        assert_eq!(g.object_view(mattock).attached_to, None);
    }
}

#[test]
fn hob_inherited_artifacts_map_search_reveals_only_selected_basic_and_shuffles_even_fail_to_find() {
    {
        let p = Player::P0;
        for find in [false, true] {
            let mut g = scenario(p);
            let ids = library(
                &mut g,
                p,
                &["Shock", "Forest", "Island", "Goblin-town", "Bear Cub"],
            );
            let map = paid(&mut g, p, "Thrór's Map", [0, 0, 0, 0, 0, 2]);
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::EffectSearch { .. })
            });
            assert!(g.observe(p).search_choice.is_some());
            assert!(g.observe(p.other()).search_choice.is_none());
            assert!(
                g.observe(p.other())
                    .objects
                    .iter()
                    .all(|o| !ids.contains(&o.id))
            );
            reject(&mut g, Action::ChooseSearch { card: ids[0] });
            reject(&mut g, Action::ChooseSearch { card: ids[3] });
            let before_rng = g.state.rng;
            if find {
                act(&mut g, Action::ChooseSearch { card: ids[1] });
            }
            if g.legal_actions().contains(&Action::FinishSearch) {
                act(&mut g, Action::FinishSearch);
            }
            finish(&mut g);
            assert_ne!(g.state.rng, before_rng, "search always shuffles");
            let mut expected = ids.clone();
            if find {
                expected.retain(|id| *id != ids[1]);
            }
            expected.sort_unstable();
            let mut remaining = g.state.players[p.index()].library.to_vec();
            remaining.sort_unstable();
            assert_eq!(remaining, expected);
            assert_eq!(g.object_view(map).zone, Zone::Battlefield);
            if find {
                assert_eq!(g.object_view(ids[1]).zone, Zone::Hand);
                assert!(
                    g.observe(p.other())
                        .objects
                        .iter()
                        .any(|o| o.id == ids[1] && o.card == Card::Forest)
                );
            } else {
                assert!(
                    g.state.players[p.index()].library.contains(&ids[1]),
                    "qualified hidden-zone search may legally fail to find"
                );
            }
        }
    }
}
