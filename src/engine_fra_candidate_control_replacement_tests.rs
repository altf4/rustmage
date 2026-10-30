use super::*;

#[test]
fn gallia_committed_activation_survives_departure_and_replaces_counters() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let gallia = put(&mut g, p, "Gallia, the Merrymaker", Zone::Battlefield);
        let target = put(&mut g, p.other(), "Bear Cub", Zone::Hand);
        g.move_object(target, Zone::Battlefield);
        put(&mut g, p.other(), "Doubling Season", Zone::Battlefield);
        g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 1];
        act(
            &mut g,
            Action::AnnounceActivate {
                card: gallia,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        pay(&mut g);
        g.move_object(gallia, Zone::Graveyard);
        let saved = g.snapshot();
        super::super::contracts::branch_contract(g.clone());
        for replay in 0..2 {
            if replay == 1 {
                g.restore(&saved).unwrap();
            }
            settle(&mut g);
            assert_eq!(g.counter_count(target, CounterKind::PlusOne), 2);
            assert_eq!(g.object_view(target).controller, p.other());
        }
    }
}

#[test]
fn hound_queued_trigger_keeps_controller_when_source_changes_control() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let hound = put(&mut g, p, "Flickering Hound", Zone::Battlefield);
        let target = put(&mut g, p, "Hill Giant", Zone::Battlefield);
        let old = g.object_ref(target);
        let spell = put(&mut g, p, "Bear Cub", Zone::Hand);
        cast(&mut g, spell, None);
        let mut selected = false;
        for _ in 0..64 {
            g.prepare_triggers();
            let actions = g.legal_actions();
            let choice = Action::ChooseTarget {
                target: Target::Card(target),
            };
            if actions.contains(&choice) {
                act(&mut g, choice);
                selected = true;
                break;
            } else if let Some(action) = actions
                .into_iter()
                .find(|a| matches!(a, Action::Pass | Action::PutTrigger { .. }))
            {
                act(&mut g, action);
            } else {
                panic!("unexpected Hound choice {:?}", g.state.pending);
            }
        }
        assert!(selected);
        let reference = g.object_ref(hound);
        g.change_controller(reference, p.other());
        let saved = g.snapshot();
        super::super::contracts::branch_contract(g.clone());
        for replay in 0..2 {
            if replay == 1 {
                g.restore(&saved).unwrap();
            }
            settle(&mut g);
            assert_eq!(g.object_view(target).zone, Zone::Battlefield);
            assert_ne!(g.object_ref(target), old);
            assert_eq!(g.object_view(target).controller, p);
        }
    }
}
