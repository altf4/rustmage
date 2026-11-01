//! Four Saga chapters with optional controlled permanent and exact delayed return.
use super::*;
fn until(g: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..64 {
        if ready(g) {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("Roll Saga decision budget: {:?}", g.state.pending);
}
fn finish(g: &mut Game) {
    until(g, |g| {
        g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.resolving.is_none()
            && g.state.pending_triggers.is_empty()
            && g.state.ordering_triggers.is_empty()
    });
}
fn step(g: &mut Game, s: Step) {
    g.begin_step(s);
    g.give_priority();
    g.prepare_triggers();
}
fn saga(g: &mut Game, p: Player) -> CardId {
    let id = put(g, p, "Roll-Roll-Roll-Roll", Zone::Hand);
    g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 2];
    cast(g, id, None);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    id
}
fn targets(g: &mut Game) {
    until(g, |g| matches!(g.state.pending, Pending::Targets { .. }));
}
fn bind(g: &mut Game, id: Option<CardId>) {
    if let Some(id) = id {
        act(
            g,
            Action::ChooseTarget {
                target: Target::Card(id),
            },
        );
    }
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
}
#[test]
fn roll_four_paid_chapters_blink_creature_and_land_then_sacrifice() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let land = put(&mut g, p, "Forest", Zone::Battlefield);
        let foe = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let artifact = put(&mut g, p, "Goldvein Pick", Zone::Battlefield);
        let id = saga(&mut g, p);
        for chapter in 1..=4 {
            if chapter > 1 {
                step(&mut g, Step::Main1);
            }
            targets(&mut g);
            assert_eq!(g.counter_count(id, CounterKind::Lore), chapter);
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(foe),
                },
            );
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(artifact),
                },
            );
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Player(p),
                },
            );
            let selected = if chapter % 2 == 0 { land } else { bear };
            let before = g.object_ref(selected);
            bind(&mut g, Some(selected));
            finish(&mut g);
            assert_eq!(g.object_view(selected).zone, Zone::Exile);
            assert_eq!(
                g.object_view(id).zone,
                if chapter == 4 {
                    Zone::Graveyard
                } else {
                    Zone::Battlefield
                }
            );
            step(&mut g, Step::End);
            finish(&mut g);
            assert_eq!(g.object_view(selected).zone, Zone::Battlefield);
            assert_eq!(g.object_view(selected).controller, p);
            assert_ne!(g.object_ref(selected), before);
        }
    }
}
#[test]
fn roll_no_target_source_departure_and_stale_target_are_distinct() {
    for mode in 0..3 {
        let p = Player::P0;
        let mut g = scenario(p);
        let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let id = saga(&mut g, p);
        targets(&mut g);
        contracts::branch_contract(g.clone());
        bind(&mut g, if mode == 0 { None } else { Some(bear) });
        if mode == 1 {
            g.move_object(id, Zone::Graveyard);
        }
        if mode == 2 {
            let old = g.object_ref(bear);
            g.move_object(bear, Zone::Hand);
            g.move_object(bear, Zone::Battlefield);
            assert_ne!(g.object_ref(bear), old);
        }
        finish(&mut g);
        assert_eq!(
            g.object_view(bear).zone,
            if mode == 1 {
                Zone::Exile
            } else {
                Zone::Battlefield
            }
        );
        step(&mut g, Step::End);
        finish(&mut g);
        assert_eq!(g.object_view(bear).zone, Zone::Battlefield);
    }
}
#[test]
fn roll_stolen_permanent_returns_to_owner_and_new_incarnation_not_returned() {
    for mode in 0..2 {
        let p = Player::P0;
        let mut g = scenario(p);
        let bear = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let old = g.object_ref(bear);
        g.change_controller(old, p);
        saga(&mut g, p);
        targets(&mut g);
        bind(&mut g, Some(bear));
        finish(&mut g);
        assert_eq!(g.object_view(bear).zone, Zone::Exile);
        contracts::branch_contract(g.clone());
        let snapshot = g.snapshot();
        let mut child = g.clone();
        drop(g);
        if mode == 1 {
            child.move_object(bear, Zone::Hand);
        }
        step(&mut child, Step::End);
        finish(&mut child);
        assert_eq!(
            g_owner_zone(&child, bear),
            if mode == 0 {
                (p.other(), Zone::Battlefield)
            } else {
                (p.other(), Zone::Hand)
            }
        );
        child.restore(&snapshot).unwrap();
        step(&mut child, Step::End);
        finish(&mut child);
        assert_eq!(g_owner_zone(&child, bear), (p.other(), Zone::Battlefield));
    }
}
fn g_owner_zone(g: &Game, id: CardId) -> (Player, Zone) {
    (g.object_view(id).controller, g.object_view(id).zone)
}
#[test]
fn roll_token_ceases_and_next_opponent_end_step_resolves_delayed_return() {
    for is_token in [false, true] {
        let p = Player::P0;
        let mut g = scenario(p);
        let target = put(
            &mut g,
            p,
            if is_token {
                "@token/hob-bear"
            } else {
                "Bear Cub"
            },
            Zone::Battlefield,
        );
        saga(&mut g, p);
        targets(&mut g);
        bind(&mut g, Some(target));
        finish(&mut g);
        assert_eq!(
            g.object_view(target).zone,
            if is_token { Zone::Outside } else { Zone::Exile }
        );
        g.state.active = p.other();
        step(&mut g, Step::End);
        finish(&mut g);
        assert_eq!(
            g.object_view(target).zone,
            if is_token {
                Zone::Outside
            } else {
                Zone::Battlefield
            }
        );
    }
}
