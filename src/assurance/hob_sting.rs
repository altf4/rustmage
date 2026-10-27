//! Oracle outcomes for Sting, including independently surviving target groups.
use super::*;

fn enter(g: &mut Game, p: Player, bearer: Option<CardId>) -> CardId {
    let id = put(g, p, "Sting, Bilbo's Sword", Zone::Hand);
    g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
    cast(g, id, None);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    act(g, Action::Pass);
    act(g, Action::Pass);
    reject(
        g,
        Action::ChooseTarget {
            target: Target::Player(p),
        },
    );
    act(
        g,
        Action::ChooseTarget {
            target: Target::Player(p.other()),
        },
    );
    if let Some(bearer) = bearer {
        act(
            g,
            Action::ChooseTarget {
                target: Target::Card(bearer),
            },
        );
    }
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
    id
}

#[test]
fn sting_paid_flash_counts_current_opponent_creatures_and_equips_only_as_sorcery() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let a = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let b = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let removed = put(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        put(&mut g, p.other(), "Prophetic Prism", Zone::Battlefield);
        g.state.active = p.other();
        g.state.step = Step::End;
        let blade = enter(&mut g, p, Some(a));
        // The quantity is evaluated during resolution, not frozen at announcement.
        g.move_object(removed, Zone::Graveyard);
        for _ in 0..2 {
            put(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        }
        g.give_priority();
        settle(&mut g);
        assert_eq!(g.counter_count(blade, CounterKind::Hone), 2);
        assert_eq!(g.object_view(blade).attached_to, Some(g.object_ref(a)));
        assert_eq!(stats(&g, a), (4, 2));
        assert!(g.object_view(a).modified);
        let activate = Action::Activate {
            card: blade,
            ability: 0,
            target: Some(Target::Card(b)),
            payment: [0, 0, 0, 0, 0, 3],
            counter_payment: Counters::default(),
        };
        reject(&mut g, activate.clone());
        g.state.active = p;
        g.state.step = Step::Main1;
        g.state.priority = p;
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
        act(&mut g, activate);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(stats(&g, a), (2, 2));
        assert!(!g.object_view(a).modified);
        assert!(g.object_view(b).modified);
        assert_eq!(stats(&g, b), (4, 2));
    }
}

#[test]
fn sting_optional_attach_zero_count_and_counter_replacement_are_independent() {
    for (enemy_count, attach, doubling) in [(0, true, false), (2, false, false), (2, true, true)] {
        let p = Player::P0;
        let mut g = scenario(p);
        let bearer = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        for _ in 0..enemy_count {
            put(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        }
        if doubling {
            put(&mut g, p, "Doubling Season", Zone::Battlefield);
        }
        let blade = enter(&mut g, p, attach.then_some(bearer));
        settle(&mut g);
        let expected = enemy_count * if doubling { 2 } else { 1 };
        assert_eq!(g.object_view(bearer).modified, attach);
        assert_eq!(g.counter_count(blade, CounterKind::Hone), expected);
        assert_eq!(
            g.object_view(blade).attached_to,
            attach.then(|| g.object_ref(bearer))
        );
        assert_eq!(
            stats(&g, bearer),
            (2 + if attach { i128::from(expected) } else { 0 }, 2)
        );
    }
}

#[test]
fn sting_illegal_player_target_does_not_count_but_surviving_creature_is_attached() {
    let p = Player::P0;
    let mut g = scenario(p);
    let bearer = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    put(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
    let blade = enter(&mut g, p, Some(bearer));
    // Fixture supplies the continuous player-hexproof response boundary.
    put(&mut g, p.other(), "Crystal Barricade", Zone::Battlefield);
    g.give_priority();
    contracts::branch_contract(g.clone());
    settle(&mut g);
    assert_eq!(g.counter_count(blade, CounterKind::Hone), 0);
    assert_eq!(g.object_view(blade).attached_to, Some(g.object_ref(bearer)));
}

#[test]
fn sting_stale_creature_target_keeps_count_but_old_source_trigger_gets_nothing() {
    for source_leaves in [false, true] {
        let p = Player::P0;
        let mut g = scenario(p);
        let bearer = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        put(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        let blade = enter(&mut g, p, Some(bearer));
        let changed = if source_leaves { blade } else { bearer };
        g.move_object(changed, Zone::Graveyard);
        g.move_object(changed, Zone::Battlefield);
        g.give_priority();
        if source_leaves {
            // The new Sting has its own ETB; decline its attachment. Its single
            // counter must be the only contribution, and the old trigger must
            // neither add another nor attach the new Sting to the old target.
            for _ in 0..12 {
                if matches!(g.state.pending, Pending::Targets { .. }) {
                    act(
                        &mut g,
                        Action::ChooseTarget {
                            target: Target::Player(p.other()),
                        },
                    );
                    if g.legal_actions().contains(&Action::FinishTargets) {
                        act(&mut g, Action::FinishTargets);
                    }
                    break;
                }
                act(&mut g, Action::Pass);
            }
        }
        settle(&mut g);
        assert_eq!(g.counter_count(blade, CounterKind::Hone), 1);
        assert_eq!(g.object_view(blade).attached_to, None);
        assert_eq!(stats(&g, bearer), (2, 2));
    }
}

#[test]
fn sting_all_targets_illegal_fizzles_without_counters() {
    let p = Player::P0;
    let mut g = scenario(p);
    put(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
    let blade = enter(&mut g, p, None);
    put(&mut g, p.other(), "Crystal Barricade", Zone::Battlefield);
    g.give_priority();
    settle(&mut g);
    assert_eq!(g.counter_count(blade, CounterKind::Hone), 0);
    assert_eq!(g.object_view(blade).attached_to, None);
}
