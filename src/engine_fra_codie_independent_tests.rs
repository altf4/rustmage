use super::*;

fn codie_test_prepared(g: &mut Game, p: Player, name: &str) -> ObjectRef {
    let id = put(g, p, name, Zone::Battlefield);
    let permanent = g.object_ref(id);
    g.prepare_permanent(permanent);
    g.state
        .preparations
        .as_ref()
        .unwrap()
        .iter()
        .find(|r| r.permanent == permanent)
        .unwrap()
        .spell_copy
}

fn codie_test_settle(g: &mut Game) {
    for _ in 0..128 {
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty()
            && g.state.resolving.is_none()
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
                    | Action::FinishStackTargets
            )
        });
    }
    panic!("Codie test did not settle");
}

#[test]
fn codie_retargets_prepared_copy_after_source_leaves_and_does_not_retrigger() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let codie = put(&mut g, p, "Codie, Ravenous Codex", Zone::Battlefield);
        let prepared = codie_test_prepared(&mut g, p, "Diviner of Victory // Unwind History");
        let first = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let second = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 1];
        act(
            &mut g,
            Action::AnnounceExile {
                card: prepared.card,
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(first),
            },
        );
        pay(&mut g);
        g.move_object(codie, Zone::Graveyard);
        for _ in 0..64 {
            if matches!(g.state.pending, Pending::Retarget { .. }) {
                break;
            }
            choose(&mut g, |a| {
                matches!(a, Action::Pass | Action::PutTrigger { .. })
            });
        }
        assert!(matches!(g.state.pending, Pending::Retarget { .. }));
        act(
            &mut g,
            Action::SetStackTarget {
                index: 0,
                target: Target::Card(second),
            },
        );
        act(&mut g, Action::FinishStackTargets);
        assert_eq!(
            g.state
                .stack
                .iter()
                .filter(|s| s.kind == StackKind::Spell)
                .count(),
            2
        );
        assert!(!g.state.pending_triggers.iter().any(|s| s.card == codie));
        let snapshot = g.snapshot();
        for _ in 0..2 {
            codie_test_settle(&mut g);
            assert_eq!(g.object_view(first).zone, Zone::Hand);
            assert_eq!(g.object_view(second).zone, Zone::Hand);
            g.restore(&snapshot).unwrap();
        }
    }
}

#[test]
fn codie_ignores_opponent_prepared_casts() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p.other());
        let codie = put(&mut g, p, "Codie, Ravenous Codex", Zone::Battlefield);
        let enemy = codie_test_prepared(&mut g, p.other(), "Fatehold Chronologist // Peer Review");
        act(&mut g, Action::AnnounceExile { card: enemy.card });
        choose(&mut g, |a| matches!(a, Action::ChooseSpellCost { .. }));
        pay(&mut g);
        assert!(!g.state.pending_triggers.iter().any(|s| s.card == codie));
        assert!(
            !g.state
                .stack
                .iter()
                .any(|s| s.card == codie && s.kind != StackKind::Spell)
        );
    }
}

#[test]
fn codie_copy_remains_controlled_by_trigger_controller_after_control_change() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let codie = put(&mut g, p, "Codie, Ravenous Codex", Zone::Battlefield);
        let prepared = codie_test_prepared(&mut g, p, "Fatehold Chronologist // Peer Review");
        act(
            &mut g,
            Action::AnnounceExile {
                card: prepared.card,
            },
        );
        choose(&mut g, |a| matches!(a, Action::ChooseSpellCost { .. }));
        pay(&mut g);
        assert!(
            g.state.pending_triggers.iter().any(|s| s.card == codie)
                || g.state
                    .stack
                    .iter()
                    .any(|s| s.card == codie && s.kind != StackKind::Spell)
        );
        g.change_controller(g.object_ref(codie), p.other());
        codie_test_settle(&mut g);
        let cadets = g
            .state
            .objects
            .iter()
            .filter(|o| {
                o.zone == Zone::Battlefield && o.controller == p && o.card == Card::test_index(5298)
            })
            .count();
        assert_eq!(cadets, 2);
    }
}
