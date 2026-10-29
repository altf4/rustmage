use super::*;

fn finish_diviner(g: &mut Game) {
    for _ in 0..96 {
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
    panic!("Diviner resolution did not finish");
}
fn make_diviner(g: &mut Game, p: Player) -> (CardId, ObjectRef) {
    let id = put(g, p, "Diviner of Victory // Unwind History", Zone::Hand);
    g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 0];
    act(g, Action::AnnounceCast { card: id });
    pay(g);
    finish_diviner(g);
    let copy = g
        .state
        .preparations
        .as_ref()
        .unwrap()
        .iter()
        .find(|r| r.permanent == g.object_ref(id))
        .unwrap()
        .spell_copy;
    (id, copy)
}
fn wait_library(g: &mut Game) {
    for _ in 0..48 {
        if matches!(g.state.pending, Pending::EffectLibrary { .. }) {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("missing surveil selection");
}

#[test]
fn unwind_returns_stolen_creature_to_its_owner_and_requires_full_cost() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let (source, copy) = make_diviner(&mut g, p);
        let victim = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.change_controller(g.object_ref(victim), p.other());
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 0];
        let before = g.snapshot();
        reject(&mut g, Action::AnnounceExile { card: copy.card });
        assert_eq!(g.snapshot(), before);
        assert!(g.is_prepared(g.object_ref(source)));
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 1];
        act(&mut g, Action::AnnounceExile { card: copy.card });
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(victim),
            },
        );
        pay(&mut g);
        wait_library(&mut g);
        assert_eq!(g.object_view(victim).zone, Zone::Hand);
        assert_eq!(g.state.objects[victim as usize].owner, p);
        choose(&mut g, |a| {
            matches!(
                a,
                Action::ArrangeLibrary {
                    destination: LibraryDestination::Top,
                    ..
                }
            )
        });
        finish_diviner(&mut g);
        assert_eq!(stats(&g, source), (2, 2));
    }
}

#[test]
fn diviner_surveil_on_empty_library_still_triggers_twice_with_wizards_staff() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let (source, copy) = make_diviner(&mut g, p);
        let staff = put(&mut g, p, "Wizard's Staff", Zone::Battlefield);
        g.state.objects[staff as usize].attached_to = Some(g.object_ref(source));
        let victim = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let library = g.state.players[p.index()].library.clone();
        for id in library {
            g.move_object(id, Zone::Exile);
        }
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 1];
        act(&mut g, Action::AnnounceExile { card: copy.card });
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(victim),
            },
        );
        pay(&mut g);
        finish_diviner(&mut g);
        assert_eq!(g.object_view(victim).zone, Zone::Hand);
        assert!(g.state.players[p.index()].library.is_empty());
        // Wizard's Staff doubles both prowess on the cast and the surveil trigger.
        assert_eq!(stats(&g, source), (5, 5));
        g.finish_cleanup();
        assert_eq!(stats(&g, source), (1, 1));
    }
}

#[test]
fn diviner_queued_growth_does_not_affect_new_incarnation() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let (source, copy) = make_diviner(&mut g, p);
        let victim = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 1];
        act(&mut g, Action::AnnounceExile { card: copy.card });
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(victim),
            },
        );
        pay(&mut g);
        wait_library(&mut g);
        choose(&mut g, |a| {
            matches!(
                a,
                Action::ArrangeLibrary {
                    destination: LibraryDestination::Top,
                    ..
                }
            )
        });
        let original = g.object_ref(source);
        assert!(
            g.state
                .pending_triggers
                .iter()
                .chain(g.state.stack.iter())
                .any(|item| item.object == original
                    && matches!(item.kind, StackKind::TriggeredAbility { .. }))
        );
        g.move_object(source, Zone::Exile);
        g.move_object(source, Zone::Battlefield);
        finish_diviner(&mut g);
        assert_eq!(stats(&g, source), (1, 1));
    }
}
