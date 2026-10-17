//! Complete FDN search/recursion behavior through paid actions and owned replay.
use super::ability_timeline_tests::{cast, checked, named, settle, setup};
use super::*;
use crate::SearchDestination;

fn until(g: &mut Game, stop: impl Fn(&Game) -> bool) {
    g.prepare_triggers();
    for _ in 0..80 {
        if stop(g) {
            return;
        }
        let actions = g.legal_actions();
        let action = if g.state.pending == Pending::Priority {
            Action::Pass
        } else {
            actions
                .iter()
                .find(|a| matches!(a, Action::FinishTargets | Action::PayCosts { .. }))
                .or_else(|| {
                    actions
                        .iter()
                        .find(|a| matches!(a, Action::PutTrigger { .. }))
                })
                .unwrap_or_else(|| panic!("unexpected choice {:?}: {actions:?}", g.state.pending))
                .clone()
        };
        checked(g, action);
    }
    panic!("did not reach expected choice: {:?}", g.state.pending);
}
fn reject(g: &mut Game, action: Action) {
    let before = g.snapshot();
    let revision = g.revision();
    assert_eq!(
        g.apply(g.actor().unwrap(), action),
        Err(Error::IllegalAction)
    );
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), revision);
}
fn guide(g: &mut Game, p: Player) -> CardId {
    let id = named(g, p, "Campus Guide", Zone::Hand);
    g.state.players[p.index()].mana = [2, 0, 0, 0, 0, 0];
    cast(g, id, None, &[]);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    until(g, |g| {
        matches!(g.state.pending, Pending::EffectOptional { .. })
    });
    assert_eq!(g.object_view(id).power, 2);
    assert_eq!(g.object_view(id).toughness, 1);
    id
}
fn search(g: &mut Game) {
    checked(g, Action::ChooseOptional { accept: true });
    assert!(matches!(g.state.pending, Pending::EffectSearch { .. }));
}

#[test]
fn campus_guide_paid_search_reveals_only_committed_top_and_replays_after_parent_disposal() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let nonbasic = named(&mut g, p, "Commercial District", Zone::Library);
        let other = named(&mut g, p.other(), "Forest", Zone::Library);
        let body = named(&mut g, p, "Bear Cub", Zone::Library);
        let hand = named(&mut g, p, "Forest", Zone::Hand);
        let source = guide(&mut g, p);
        // The already-resolving trigger owns its controller and survives its source.
        g.move_object(source, Zone::Exile);
        let original = g.state.players[p.index()].library.clone();
        let rng = g.state.rng;
        search(&mut g);
        let selected = original[0];
        let reference = g.object_ref(selected);
        assert_eq!(
            g.observe(p).search_choice.unwrap().destination,
            SearchDestination::Top
        );
        for bad in [nonbasic, other, body, hand, u32::MAX] {
            reject(&mut g, Action::ChooseSearch { card: bad });
        }
        let before = g.snapshot();
        checked(&mut g, Action::ChooseSearch { card: selected });
        reject(&mut g, Action::ChooseSearch { card: selected });
        assert_eq!(g.state.players[p.index()].library, original);
        assert_eq!(g.state.rng, rng);
        let hidden = g.observe(p.other());
        assert!(hidden.search_choice.is_none());
        assert!(hidden.actions.is_empty());
        assert!(hidden.objects.iter().all(|o| o.zone != Zone::Library));
        assert!(hidden.known_library_top[p.index()].is_none());
        let partial = g.snapshot();
        let mut branch = {
            let parent = g.clone();
            parent.clone()
        };
        checked(&mut branch, Action::FinishSearch);
        checked(&mut g, Action::FinishSearch);
        assert_eq!(g.snapshot(), branch.snapshot());
        let mut expected = original.clone();
        let mut expected_rng = rng;
        expected_rng.shuffle(&mut expected);
        let position = expected.iter().position(|&id| id == selected).unwrap();
        expected.remove(position);
        expected.push(selected);
        assert_eq!(g.state.players[p.index()].library, expected);
        assert_eq!(g.state.rng, expected_rng);
        assert_eq!(g.object_ref(selected), reference);
        for viewer in [p, p.other()] {
            assert_eq!(
                g.observe(viewer).known_library_top[p.index()]
                    .unwrap()
                    .object,
                reference
            );
        }
        let after = g.snapshot();
        branch.restore(&partial).unwrap();
        checked(&mut branch, Action::FinishSearch);
        assert_eq!(branch.snapshot(), after);
        branch.restore(&before).unwrap();
        checked(&mut branch, Action::ChooseSearch { card: selected });
        checked(&mut branch, Action::FinishSearch);
        assert_eq!(branch.snapshot(), after);
        // A later searched-but-unfound basic clears the public known top.
        let bushwhack = named(&mut g, p, "Bushwhack", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 0];
        cast(&mut g, bushwhack, Some(0), &[]);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectSearch { .. })
        });
        checked(&mut g, Action::FinishSearch);
        assert!(g.observe(p.other()).known_library_top[p.index()].is_none());
        branch.draw(p);
        assert_eq!(branch.state.objects[selected as usize].zone, Zone::Hand);
        assert!(branch.observe(p.other()).known_library_top[p.index()].is_none());
        assert!(
            branch
                .observe(p.other())
                .objects
                .iter()
                .any(|o| o.id == selected && o.zone == Zone::Hand)
        );
    }
}

#[test]
fn campus_guide_decline_differs_from_failed_search_and_empty_search_never_draws() {
    for p in [Player::P0, Player::P1] {
        for accept in [false, true] {
            let mut g = setup(p);
            guide(&mut g, p);
            let before = g.state.players[p.index()].library.clone();
            let mut rng = g.state.rng;
            checked(&mut g, Action::ChooseOptional { accept });
            if accept {
                // A qualified hidden search may fail even with twenty legal cards.
                checked(&mut g, Action::FinishSearch);
                let mut expected = before.clone();
                rng.shuffle(&mut expected);
                assert_eq!(g.state.players[p.index()].library, expected);
            } else {
                assert_eq!(g.state.players[p.index()].library, before);
            }
            assert_eq!(g.state.rng, rng);
            assert!(g.observe(p.other()).known_library_top[p.index()].is_none());
            assert!(!g.state.players[p.index()].attempted_empty_draw);
        }
        let mut g = setup(p);
        for id in g.state.players[p.index()].library.clone() {
            g.move_object(id, Zone::Exile);
        }
        guide(&mut g, p);
        search(&mut g);
        assert_eq!(
            g.legal_actions()
                .into_iter()
                .filter(|a| *a != Action::Concede)
                .collect::<Vec<_>>(),
            vec![Action::FinishSearch]
        );
        checked(&mut g, Action::FinishSearch);
        assert!(!g.state.players[p.index()].attempted_empty_draw);
    }
}

fn plot(g: &mut Game, p: Player, mode: u16, target: CardId) {
    let id = named(g, p, "Deadly Plot", Zone::Hand);
    g.state.players[p.index()].mana = [3, 0, 1, 0, 0, 0];
    cast(g, id, Some(mode), &[Target::Card(target)]);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
}

#[test]
fn deadly_plot_paid_modes_destroy_creatures_planeswalkers_and_return_own_zombie_tapped() {
    for p in [Player::P0, Player::P1] {
        for name in ["Bear Cub", "Ajani, Caller of the Pride"] {
            let mut g = setup(p);
            let id = named(&mut g, p.other(), name, Zone::Battlefield);
            g.state.objects[id as usize].counters.loyalty = 6;
            plot(&mut g, p, 0, id);
            settle(&mut g);
            assert_eq!(g.object_view(id).zone, Zone::Graveyard);
        }
        let mut g = setup(p);
        let id = named(&mut g, p, "Hungry Ghoul", Zone::Graveyard);
        let old = g.object_ref(id);
        plot(&mut g, p, 1, id);
        settle(&mut g);
        let view = g.object_view(id);
        assert_eq!(view.zone, Zone::Battlefield);
        assert_eq!(view.controller, p);
        assert!(view.tapped);
        assert!(view.summoning_sick);
        assert_ne!(view.reference, old);
    }
}

#[test]
fn deadly_plot_rejects_wrong_modes_targets_and_insufficient_mana_transactionally() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let spell = named(&mut g, p, "Deadly Plot", Zone::Hand);
        let good = named(&mut g, p, "Hungry Ghoul", Zone::Graveyard);
        let wrong_type = named(&mut g, p, "Bear Cub", Zone::Graveyard);
        let wrong_owner = named(&mut g, p.other(), "Hungry Ghoul", Zone::Graveyard);
        let wrong_zone = named(&mut g, p, "Hungry Ghoul", Zone::Battlefield);
        g.state.players[p.index()].mana = [0; 6];
        reject(&mut g, Action::AnnounceCast { card: spell });
        g.state.players[p.index()].mana = [3, 0, 1, 0, 0, 0];
        checked(&mut g, Action::AnnounceCast { card: spell });
        reject(&mut g, Action::ChooseMode { mode: 2 });
        checked(&mut g, Action::ChooseMode { mode: 1 });
        reject(&mut g, Action::FinishTargets);
        for target in [wrong_type, wrong_owner, wrong_zone] {
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(target),
                },
            );
        }
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(good),
            },
        );
        until(&mut g, |g| g.state.announcing.is_none());
        settle(&mut g);
        assert!(g.object_view(good).tapped);
    }
}

#[test]
fn deadly_plot_resolution_rechecks_incarnation_and_respects_indestructible() {
    for mode in [0, 1] {
        let p = Player::P0;
        let mut g = setup(p);
        let zone = if mode == 0 {
            Zone::Battlefield
        } else {
            Zone::Graveyard
        };
        let target = named(&mut g, p, "Hungry Ghoul", zone);
        plot(&mut g, p, mode, target);
        g.move_object(target, Zone::Exile);
        g.move_object(target, zone);
        let reference = g.object_ref(target);
        settle(&mut g);
        assert_eq!(g.object_view(target).zone, zone);
        assert_eq!(g.object_ref(target), reference);
    }
    let mut g = setup(Player::P0);
    let target = named(
        &mut g,
        Player::P1,
        "Zetalpa, Primal Dawn",
        Zone::Battlefield,
    );
    plot(&mut g, Player::P0, 0, target);
    settle(&mut g);
    assert_eq!(g.object_view(target).zone, Zone::Battlefield);
}

fn dying_driver(g: &mut Game, p: Player) -> CardId {
    let driver = named(g, p, "Driver of the Dead", Zone::Hand);
    g.state.players[p.index()].mana = [3, 0, 1, 0, 0, 0];
    cast(g, driver, None, &[]);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    settle(g);
    assert_eq!(g.object_view(driver).power, 3);
    assert_eq!(g.object_view(driver).toughness, 2);
    let removal = named(g, p, "Murder", Zone::Hand);
    g.state.players[p.index()].mana = [1, 0, 2, 0, 0, 0];
    cast(g, removal, None, &[Target::Card(driver)]);
    driver
}

#[test]
fn driver_paid_death_mandatory_target_filters_mana_value_owner_and_card_type() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let two = named(&mut g, p, "Bear Cub", Zone::Graveyard);
        let one = named(&mut g, p, "Llanowar Elves", Zone::Graveyard);
        let three = named(&mut g, p, "Crow of Dark Tidings", Zone::Graveyard);
        let other = named(&mut g, p.other(), "Bear Cub", Zone::Graveyard);
        let land = named(&mut g, p, "Forest", Zone::Graveyard);
        let driver = dying_driver(&mut g, p);
        until(&mut g, |g| {
            g.legal_actions().contains(&Action::ChooseTarget {
                target: Target::Card(two),
            })
        });
        assert_eq!(g.object_view(driver).zone, Zone::Graveyard);
        assert!(g.legal_actions().contains(&Action::ChooseTarget {
            target: Target::Card(one)
        }));
        reject(&mut g, Action::FinishTargets);
        reject(&mut g, Action::ChooseOptional { accept: false });
        for bad in [three, other, land, driver] {
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(bad),
                },
            );
        }
        let selected = if p == Player::P0 { two } else { one };
        let remaining = if selected == two { one } else { two };
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(selected),
            },
        );
        settle(&mut g);
        assert_eq!(g.object_view(selected).zone, Zone::Battlefield);
        assert!(!g.object_view(selected).tapped);
        assert_eq!(g.object_view(remaining).zone, Zone::Graveyard);
    }
}

#[test]
fn driver_resolves_without_source_and_stale_target_does_not_return_new_incarnation() {
    for stale in [false, true] {
        let p = Player::P0;
        let mut g = setup(p);
        let target = named(&mut g, p, "Bear Cub", Zone::Graveyard);
        let driver = dying_driver(&mut g, p);
        until(&mut g, |g| {
            g.legal_actions().contains(&Action::ChooseTarget {
                target: Target::Card(target),
            })
        });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        until(&mut g, |g| g.state.announcing.is_none());
        g.move_object(driver, Zone::Exile);
        if stale {
            g.move_object(target, Zone::Hand);
            g.move_object(target, Zone::Graveyard);
        }
        settle(&mut g);
        assert_eq!(
            g.object_view(target).zone,
            if stale {
                Zone::Graveyard
            } else {
                Zone::Battlefield
            }
        );
        assert_eq!(g.object_view(driver).zone, Zone::Exile);
    }
}

#[test]
fn driver_without_legal_target_never_blocks_priority() {
    let p = Player::P0;
    let mut g = setup(p);
    let driver = dying_driver(&mut g, p);
    settle(&mut g);
    assert_eq!(g.object_view(driver).zone, Zone::Graveyard);
    assert!(g.state.pending_triggers.is_empty());
    assert_eq!(g.state.pending, Pending::Priority);
}

#[test]
fn campus_guide_singleton_shuffle_keeps_known_top_when_search_fails() {
    let p = Player::P0;
    let mut g = setup(p);
    guide(&mut g, p);
    let ids = g.state.players[0].library.clone();
    let top = ids[0];
    for &id in ids.iter().skip(1) {
        g.move_object(id, Zone::Exile);
    }
    let reference = g.object_ref(top);
    g.remember_library_top(p, reference);
    search(&mut g);
    checked(&mut g, Action::FinishSearch);
    assert_eq!(
        g.observe(p.other()).known_library_top[0].unwrap().object,
        reference
    );
}

#[test]
fn driver_dies_uses_last_controller_graveyard_even_when_owner_differs() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let own = named(&mut g, p, "Bear Cub", Zone::Graveyard);
        let other = named(&mut g, p.other(), "Bear Cub", Zone::Graveyard);
        let driver = dying_driver(&mut g, p);
        // Equivalent owned scenario to a permanent controlled by a nonowner.
        g.state.objects[driver as usize].owner = p.other();
        until(&mut g, |g| {
            g.legal_actions().contains(&Action::ChooseTarget {
                target: Target::Card(own),
            })
        });
        assert_eq!(g.object_view(driver).controller, p.other());
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(other),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(own),
            },
        );
        settle(&mut g);
        assert_eq!(g.object_view(own).controller, p);
        assert_eq!(g.object_view(own).zone, Zone::Battlefield);
        assert_eq!(g.object_view(other).zone, Zone::Graveyard);
    }
}
