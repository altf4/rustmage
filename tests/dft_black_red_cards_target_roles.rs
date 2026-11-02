//! Paid Trade/Junker spells preserve independently declared controller roles.
mod support;
use rustmage as _;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;
fn drain(g: &mut Game) {
    for _ in 0..64 {
        g.prepare_triggers();
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty()
            && g.state.resolving.is_none()
        {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("role response resolution bound");
}
fn reach_targets(g: &mut Game) {
    for _ in 0..24 {
        g.prepare_triggers();
        if g.legal_actions()
            .iter()
            .any(|a| matches!(a, Action::ChooseTarget { .. } | Action::FinishTargets))
        {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("target role declaration missing");
}
fn select(g: &mut Game, id: CardId) {
    act(
        g,
        Action::ChooseTarget {
            target: Target::Card(id),
        },
    );
}
fn trade(g: &mut Game, p: Player, own: CardId, enemy: CardId) -> CardId {
    let spell = put(g, p, "Trade the Helm", Zone::Hand);
    g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 4];
    act(g, Action::AnnounceCast { card: spell });
    reject(
        g,
        Action::ChooseTarget {
            target: Target::Card(enemy),
        },
    );
    select(g, own);
    reject(
        g,
        Action::ChooseTarget {
            target: Target::Card(own),
        },
    );
    select(g, enemy);
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
    pay(g);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    spell
}
fn junker(g: &mut Game, p: Player, own: Option<CardId>, enemy: Option<CardId>) -> CardId {
    let source = put(g, p, "Demonic Junker", Zone::Hand);
    g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 6];
    cast(g, source, None);
    reach_targets(g);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    if let Some(id) = own {
        select(g, id);
    } else {
        act(g, Action::FinishTargets);
    }
    if let Some(id) = enemy {
        select(g, id);
    } else {
        act(g, Action::FinishTargets);
    }
    source
}
#[test]
fn trade_paid_target_roles_revalidate_independently_after_one_or_both_control_changes() {
    for p in [Player::P0, Player::P1] {
        for changes in [0, 1, 2, 3] {
            let mut g = scenario(p);
            let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let enemy = put(&mut g, p.other(), "Short Bow", Zone::Battlefield);
            let spell = trade(&mut g, p, own, enemy);
            if changes & 1 != 0 {
                g.change_controller(g.object_ref(own), p.other());
            }
            if changes & 2 != 0 {
                g.change_controller(g.object_ref(enemy), p);
            }
            drain(&mut g);
            let expected = match changes {
                0 | 3 => (p.other(), p),
                1 => (p.other(), p.other()),
                _ => (p, p),
            };
            assert_eq!(g.object_view(own).controller, expected.0);
            assert_eq!(g.object_view(enemy).controller, expected.1);
            assert_eq!(g.object_view(own).owner, p);
            assert_eq!(g.object_view(enemy).owner, p.other());
            assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
        }
    }
}
#[test]
fn trade_owned_raw_index_resolution_and_restored_full_swap_response_keep_distinct_results() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let own = put(&mut g, p, "Short Bow", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        trade(&mut g, p, own, enemy);
        g.state.priority = p.other();
        g.state.passes = 1;
        let saved = g.snapshot();
        let mut raw = g.clone();
        let mut indexed = g.clone();
        let i = indexed
            .legal_actions()
            .iter()
            .position(|a| *a == Action::Pass)
            .unwrap();
        act(&mut raw, Action::Pass);
        indexed
            .apply_index(p.other(), indexed.revision(), i)
            .unwrap();
        assert_eq!(raw.snapshot(), indexed.snapshot());
        indexed.restore(&saved).unwrap();
        drop(g);
        indexed.change_controller(indexed.object_ref(own), p.other());
        indexed.change_controller(indexed.object_ref(enemy), p);
        drain(&mut indexed);
        assert_eq!(indexed.object_view(own).controller, p.other());
        assert_eq!(indexed.object_view(enemy).controller, p);
        indexed.restore(&saved).unwrap();
        indexed.change_controller(indexed.object_ref(own), p.other());
        drain(&mut indexed);
        assert_eq!(indexed.object_view(own).controller, p.other());
        assert_eq!(indexed.object_view(enemy).controller, p.other());
        assert_eq!(raw.object_view(enemy).controller, p);
    }
}
#[test]
fn junker_paid_entry_optional_groups_choose_either_both_or_neither_player_independently() {
    for p in [Player::P0, Player::P1] {
        for chosen in [0, 1, 2, 3] {
            let mut g = scenario(p);
            let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let source = junker(
                &mut g,
                p,
                (chosen & 1 != 0).then_some(own),
                (chosen & 2 != 0).then_some(enemy),
            );
            drain(&mut g);
            assert_eq!(
                g.object_view(own).zone,
                if chosen & 1 != 0 {
                    Zone::Graveyard
                } else {
                    Zone::Battlefield
                }
            );
            assert_eq!(
                g.object_view(enemy).zone,
                if chosen & 2 != 0 {
                    Zone::Graveyard
                } else {
                    Zone::Battlefield
                }
            );
            assert_eq!(
                g.counter_count(source, CounterKind::PlusOne),
                if chosen & 1 != 0 { 2 } else { 0 }
            );
        }
    }
}
#[test]
fn junker_response_controller_changes_leave_only_the_still_legal_player_role_destroyed() {
    for p in [Player::P0, Player::P1] {
        for changes in [1, 2, 3] {
            let mut g = scenario(p);
            let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let source = junker(&mut g, p, Some(own), Some(enemy));
            if changes & 1 != 0 {
                g.change_controller(g.object_ref(own), p.other());
            }
            if changes & 2 != 0 {
                g.change_controller(g.object_ref(enemy), p);
            }
            drain(&mut g);
            assert_eq!(
                g.object_view(own).zone,
                if changes & 1 != 0 {
                    Zone::Battlefield
                } else {
                    Zone::Graveyard
                }
            );
            assert_eq!(
                g.object_view(enemy).zone,
                if changes & 2 != 0 {
                    Zone::Battlefield
                } else {
                    Zone::Graveyard
                }
            );
            assert_eq!(
                g.counter_count(source, CounterKind::PlusOne),
                if changes == 2 { 2 } else { 0 }
            );
        }
    }
}
#[test]
fn junker_original_queued_controller_owns_target_roles_and_reward_after_source_control_changes() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let source = junker(&mut g, p, Some(own), Some(enemy));
        assert_eq!(g.state.stack.last().unwrap().controller, p);
        g.change_controller(g.object_ref(source), p.other());
        g.change_controller(g.object_ref(enemy), p);
        drain(&mut g);
        assert_eq!(g.object_view(own).zone, Zone::Graveyard);
        assert_eq!(g.object_view(enemy).zone, Zone::Battlefield);
        assert_eq!(g.object_view(source).controller, p.other());
        assert_eq!(g.counter_count(source, CounterKind::PlusOne), 2);
    }
}
#[test]
fn junker_owned_queued_response_branch_preserves_original_roles_and_partial_reward() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let source = junker(&mut g, p, Some(own), Some(enemy));
        g.state.priority = p.other();
        g.state.passes = 1;
        let saved = g.snapshot();
        let mut raw = g.clone();
        let mut indexed = g.clone();
        let i = indexed
            .legal_actions()
            .iter()
            .position(|a| *a == Action::Pass)
            .unwrap();
        act(&mut raw, Action::Pass);
        indexed
            .apply_index(p.other(), indexed.revision(), i)
            .unwrap();
        assert_eq!(raw.snapshot(), indexed.snapshot());
        assert_eq!(raw.counter_count(source, CounterKind::PlusOne), 2);
        indexed.restore(&saved).unwrap();
        drop(g);
        indexed.change_controller(indexed.object_ref(own), p.other());
        drain(&mut indexed);
        assert_eq!(indexed.object_view(own).zone, Zone::Battlefield);
        assert_eq!(indexed.object_view(enemy).zone, Zone::Graveyard);
        assert_eq!(indexed.counter_count(source, CounterKind::PlusOne), 0);
    }
}
