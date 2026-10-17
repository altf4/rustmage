//! Complete paid Adventure bodies, owned branching and turn-bound land allowances.
use super::tests::{scenario, spawn};
use super::*;
const NAMES: [&str; 8] = [
    "Beanstalk Wurm // Plant Beans",
    "Beorn, Reluctant Host // Till and Tend",
    "Bellowing Bruiser // Beat a Path",
    "Beluna's Gatekeeper // Entry Denied",
    "Bofur, Reliable Guardian // Concerted Care",
    "Ishgard, the Holy See // Faith & Grief",
    "Velvetwing Butterflies // Gaze in Wonder",
    "Woodland Acolyte // Mend the Wilds",
];
fn act(g: &mut Game, a: Action) {
    let actor = g.actor().unwrap();
    let saved = g.snapshot();
    let rev = g.revision();
    let index = g
        .legal_actions()
        .iter()
        .position(|x| *x == a)
        .unwrap_or_else(|| panic!("illegal {a:?}, {:?}", g.observe(actor)));
    assert_eq!(g.apply(actor.other(), a.clone()), Err(Error::WrongActor));
    assert_eq!(saved, g.snapshot());
    let mut child = {
        let parent = g.clone();
        let c = parent.clone();
        drop(parent);
        c
    };
    child.apply_index(actor, rev, index).unwrap();
    g.apply(actor, a.clone()).unwrap();
    assert_eq!(g.snapshot(), child.snapshot());
    for viewer in [Player::P0, Player::P1] {
        assert_eq!(
            serde_json::to_value(g.observe(viewer)).unwrap(),
            serde_json::to_value(child.observe(viewer)).unwrap()
        );
    }
    let done = g.snapshot();
    g.restore(&saved).unwrap();
    assert_eq!(g.apply_index(actor, rev, index), Err(Error::StaleRevision));
    g.apply(actor, a).unwrap();
    assert_eq!(g.snapshot(), done);
}
fn setup(p: Player) -> Game {
    let mut g = scenario();
    g.state.active = p;
    g.state.priority = p;
    for player in &mut g.state.players {
        player.mana = [30; 6];
    }
    g
}
fn named(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    let c = Card::from_name(name).unwrap();
    assert!(c.is_supported(), "{name}");
    spawn(g, p, c, zone)
}
fn reject(g: &mut Game, a: Action) {
    let saved = g.snapshot();
    let revision = g.revision();
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), saved);
    assert_eq!(g.revision(), revision);
}
fn choose(g: &mut Game) {
    let actions = g.legal_actions();
    let a = if g.state.pending == Pending::Priority {
        Action::Pass
    } else {
        actions
            .iter()
            .find(|a| matches!(a, Action::FinishTargets))
            .or_else(|| actions.iter().find(|a| !matches!(a, Action::Concede)))
            .unwrap()
            .clone()
    };
    act(g, a);
}
fn settle(g: &mut Game) {
    for _ in 0..150 {
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.resolving.is_none()
            && g.state.announcing.is_none()
            && g.state.pending_triggers.is_empty()
        {
            return;
        }
        choose(g);
    }
    panic!("unsettled {:?}", g.observe(g.actor().unwrap()));
}
fn inset(g: &mut Game, id: CardId, targets: &[CardId]) {
    act(g, Action::AnnounceAdventure { card: id });
    for &target in targets {
        act(
            g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
    }
    settle(g);
}
fn priority(g: &mut Game, p: Player) {
    for _ in 0..30 {
        if g.actor() == Some(p) && g.state.pending == Pending::Priority {
            return;
        }
        choose(g);
    }
    panic!("priority");
}
#[test]
fn eight_whole_pairs_resolve_paid_insets_then_the_exact_normal_incarnation_both_seats() {
    for p in [Player::P0, Player::P1] {
        for (i, name) in NAMES.iter().enumerate() {
            let mut g = setup(p);
            let own = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let dead = named(&mut g, p, "Candy Trail", Zone::Graveyard);
            let id = named(&mut g, p, name, Zone::Hand);
            let old = g.object_ref(id);
            let mana_before: u32 = g.state.players[p.index()].mana.iter().sum();
            let targets = match i {
                2 | 3 | 6 => vec![enemy],
                4 => vec![own],
                5 | 7 => vec![dead],
                _ => vec![],
            };
            inset(&mut g, id, &targets);
            assert!(g.state.players[p.index()].mana.iter().sum::<u32>() < mana_before);
            assert_eq!(g.state.objects[id as usize].zone, Zone::Exile, "{name}");
            assert_ne!(g.object_ref(id), old);
            assert!(g.can_play_adventure(p, id));
            match i {
                0 | 1 => assert_eq!(
                    g.observe(p).players[p.index()].extra_land_plays_remaining,
                    1
                ),
                2 => assert!(g.keywords(enemy).contains(Keywords::CANT_BLOCK)),
                3 => assert_eq!(g.state.objects[enemy as usize].zone, Zone::Hand),
                4 => {
                    assert!(g.keywords(own).contains(Keywords::HEXPROOF));
                    assert!(g.keywords(own).contains(Keywords::INDESTRUCTIBLE));
                }
                5 => assert_eq!(g.state.objects[dead as usize].zone, Zone::Hand),
                6 => assert!(g.state.objects[enemy as usize].tapped),
                7 => assert_eq!(g.state.players[p.index()].library.last(), Some(&dead)),
                _ => unreachable!(),
            }
            priority(&mut g, p);
            let a = if i == 5 {
                Action::PlayLand { card: id }
            } else {
                Action::AnnounceExile { card: id }
            };
            act(&mut g, a);
            settle(&mut g);
            assert_eq!(
                g.state.objects[id as usize].zone,
                Zone::Battlefield,
                "{name}"
            );
            assert!(!g.can_play_adventure(p, id));
            if i == 5 {
                assert!(g.state.objects[id as usize].tapped);
            }
            if i == 7 {
                assert_eq!(g.state.objects[dead as usize].zone, Zone::Hand);
            }
        }
    }
}
#[test]
fn multiple_land_grants_stack_with_interleaved_real_land_plays_and_expire() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let lands: Vec<_> = (0..5)
            .map(|_| named(&mut g, p, "Forest", Zone::Hand))
            .collect();
        act(&mut g, Action::PlayLand { card: lands[0] });
        reject(&mut g, Action::PlayLand { card: lands[1] });
        for (i, name) in NAMES[..2].iter().enumerate() {
            let id = named(&mut g, p, name, Zone::Hand);
            inset(&mut g, id, &[]);
            priority(&mut g, p);
            assert_eq!(
                g.observe(p.other()).players[p.index()].extra_land_plays_remaining,
                1
            );
            act(&mut g, Action::PlayLand { card: lands[i + 1] });
            assert_eq!(g.state.players[p.index()].extra_land_plays_remaining, 0);
            reject(&mut g, Action::PlayLand { card: lands[i + 2] });
        }
        for name in &NAMES[..2] {
            let id = named(&mut g, p, name, Zone::Hand);
            inset(&mut g, id, &[]);
            priority(&mut g, p);
        }
        assert_eq!(g.state.players[p.index()].extra_land_plays_remaining, 2);
        act(&mut g, Action::PlayLand { card: lands[3] });
        assert_eq!(g.state.players[p.index()].extra_land_plays_remaining, 1);
        let turn = g.state.turn;
        for _ in 0..100 {
            if g.state.turn != turn {
                break;
            }
            choose(&mut g);
        }
        assert_ne!(g.state.turn, turn);
        assert_eq!(g.state.players[p.index()].extra_land_plays_remaining, 0);
    }
}
#[test]
fn whirlwing_allows_offturn_grant_but_never_offturn_land_and_grant_expires() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        g.state.active = p.other();
        named(
            &mut g,
            p,
            "Whirlwing Stormbrood // Dynamic Soar",
            Zone::Battlefield,
        );
        let id = named(&mut g, p, NAMES[0], Zone::Hand);
        let land = named(&mut g, p, "Forest", Zone::Hand);
        inset(&mut g, id, &[]);
        priority(&mut g, p);
        assert_eq!(g.state.players[p.index()].extra_land_plays_remaining, 1);
        reject(&mut g, Action::PlayLand { card: land });
        let turn = g.state.turn;
        for _ in 0..120 {
            if g.state.turn != turn {
                break;
            }
            choose(&mut g);
        }
        assert_ne!(g.state.turn, turn);
        assert_eq!(g.state.active, p);
        assert_eq!(g.state.players[p.index()].extra_land_plays_remaining, 0);
        for _ in 0..40 {
            if g.legal_actions().contains(&Action::PlayLand { card: land }) {
                break;
            }
            choose(&mut g);
        }
        act(&mut g, Action::PlayLand { card: land });
        let second = named(&mut g, p, "Forest", Zone::Hand);
        reject(&mut g, Action::PlayLand { card: second });
    }
}
#[test]
fn declared_cardinalities_reject_duplicates_and_partial_resolution_preserves_other_target() {
    for p in [Player::P0, Player::P1] {
        for index in [2, 5, 6] {
            let mut g = setup(p);
            let zone = if index == 5 {
                Zone::Graveyard
            } else {
                Zone::Battlefield
            };
            let body = if index == 5 {
                "Candy Trail"
            } else {
                "Bear Cub"
            };
            let a = named(&mut g, p, body, zone);
            let b = named(&mut g, p.other(), body, zone);
            let c = named(&mut g, p, body, zone);
            let id = named(&mut g, p, NAMES[index], Zone::Hand);
            act(&mut g, Action::AnnounceAdventure { card: id });
            if index == 6 {
                reject(&mut g, Action::FinishTargets);
            }
            if index == 5 {
                reject(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(b),
                    },
                );
            }
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(a),
                },
            );
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(a),
                },
            );
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(c),
                },
            );
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(b),
                },
            );
            while g.state.pending != Pending::Priority {
                choose(&mut g);
            }
            // A response changes one selected incarnation; the second remains legal.
            g.move_object(a, Zone::Hand);
            settle(&mut g);
            assert_eq!(g.state.objects[id as usize].zone, Zone::Exile);
            match index {
                2 => assert!(g.keywords(c).contains(Keywords::CANT_BLOCK)),
                5 => assert_eq!(g.state.objects[c as usize].zone, Zone::Hand),
                6 => assert!(g.state.objects[c as usize].tapped),
                _ => unreachable!(),
            }
        }
    }
}
#[test]
fn live_mana_value_owner_and_graveyard_type_targets_are_enforced_and_fizzle() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let own = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let high = named(&mut g, p.other(), NAMES[3], Zone::Battlefield);
        let low = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let id = named(&mut g, p, NAMES[3], Zone::Hand);
        act(&mut g, Action::AnnounceAdventure { card: id });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(own),
            },
        );
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(high),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(low),
            },
        );
        while g.state.pending != Pending::Priority {
            choose(&mut g);
        }
        // Leave/reenter of the same numeric slot does not retarget the spell.
        g.move_object(low, Zone::Hand);
        g.move_object(low, Zone::Battlefield);
        settle(&mut g);
        assert_eq!(g.state.objects[id as usize].zone, Zone::Graveyard);
        assert!(!g.can_play_adventure(p, id));
        assert_eq!(g.state.objects[low as usize].zone, Zone::Battlefield);
        priority(&mut g, p);
        let instant = named(&mut g, p, "Shock", Zone::Graveyard);
        let foreign = named(&mut g, p.other(), "Candy Trail", Zone::Graveyard);
        let permanent = named(&mut g, p, "Forest", Zone::Graveyard);
        let top = named(&mut g, p, NAMES[7], Zone::Hand);
        act(&mut g, Action::AnnounceAdventure { card: top });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(instant),
            },
        );
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(foreign),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(permanent),
            },
        );
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].library.last(), Some(&permanent));
        assert_eq!(g.public_library_top(p), Some(g.object_ref(permanent)));
        assert_eq!(
            g.observe(p.other())
                .last_target_library_move
                .unwrap()
                .destination,
            LibraryDestination::Top
        );
    }
}
#[test]
fn artifact_protection_rejects_foreign_objects_and_ends_during_cleanup() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let own = named(&mut g, p, "Candy Trail", Zone::Battlefield);
        let foreign = named(&mut g, p.other(), "Candy Trail", Zone::Battlefield);
        let land = named(&mut g, p, "Forest", Zone::Battlefield);
        let id = named(&mut g, p, NAMES[4], Zone::Hand);
        act(&mut g, Action::AnnounceAdventure { card: id });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(foreign),
            },
        );
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(land),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(own),
            },
        );
        settle(&mut g);
        assert!(g.keywords(own).contains(Keywords::HEXPROOF));
        assert!(g.keywords(own).contains(Keywords::INDESTRUCTIBLE));
        let turn = g.state.turn;
        for _ in 0..100 {
            if g.state.turn != turn {
                break;
            }
            choose(&mut g);
        }
        assert_ne!(g.state.turn, turn);
        assert!(!g.keywords(own).contains(Keywords::HEXPROOF));
        assert!(!g.keywords(own).contains(Keywords::INDESTRUCTIBLE));
    }
}
#[test]
fn overflowing_land_allowance_restores_full_state_after_paid_spell_is_on_stack() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        g.state.players[p.index()].extra_land_plays_remaining = u64::MAX;
        let id = named(&mut g, p, NAMES[0], Zone::Hand);
        act(&mut g, Action::AnnounceAdventure { card: id });
        while g.state.pending != Pending::Priority {
            choose(&mut g);
        }
        act(&mut g, Action::Pass);
        let saved = g.snapshot();
        let revision = g.revision();
        let actor = g.actor().unwrap();
        let result = g.apply(actor, Action::Pass);
        assert!(
            matches!(result, Err(Error::CapacityExceeded(_))),
            "{result:?}"
        );
        assert_eq!(g.snapshot(), saved);
        assert_eq!(g.revision(), revision);
    }
}
