use super::*;

fn vindictive_finish(g: &mut Game) {
    for _ in 0..96 {
        g.prepare_triggers();
        if g.state.pending == Pending::Priority && g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty() && g.state.resolving.is_none() { return; }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("Vindictive Triumph continuation did not finish: {:?}", g.state.pending);
}

#[test]
fn vindictive_returns_small_target_tapped_under_caster_then_exiles_exact_incarnation() {
    for p in [Player::P0, Player::P1] {
        for blink in [false, true] {
            let mut g = scenario(p);
            let target = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let source = put(&mut g, p, "Vindictive Triumph", Zone::Hand);
            g.state.players[p.index()].mana = [1, 0, 2, 0, 0, 0];
            let prior = g.object_ref(target);
            cast(&mut g, source, Some(Target::Card(target)));
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            vindictive_finish(&mut g);
            assert_ne!(g.object_ref(target), prior);
            assert_eq!(g.object_view(target).zone, Zone::Battlefield);
            assert_eq!(g.object_view(target).controller, p);
            assert!(g.object_view(target).tapped);
            assert_eq!(g.state.delayed_returns.as_ref().unwrap().len(), 1);
            if blink {
                g.move_object(target, Zone::Exile);
                g.move_object(target, Zone::Battlefield);
            }
            let saved = g.snapshot();
            for _ in 0..2 {
                g.begin_step(Step::End);
                vindictive_finish(&mut g);
                assert_eq!(g.object_view(target).zone, if blink { Zone::Battlefield } else { Zone::Exile });
                g.restore(&saved).unwrap();
            }
        }
    }
}

#[test]
fn vindictive_large_permanent_is_exiled_and_invalidated_target_fizzles() {
    for p in [Player::P0, Player::P1] {
        for invalidated in [false, true] {
            let mut g = scenario(p);
            let target = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
            let source = put(&mut g, p, "Vindictive Triumph", Zone::Hand);
            cast(&mut g, source, Some(Target::Card(target)));
            if invalidated {
                g.move_object(target, Zone::Exile);
                g.move_object(target, Zone::Battlefield);
            }
            vindictive_finish(&mut g);
            assert_eq!(g.object_view(target).zone, if invalidated { Zone::Battlefield } else { Zone::Exile });
            assert!(g.state.delayed_returns.is_none());
        }
    }
}

#[test]
fn vindictive_exiled_creature_token_does_not_return() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        g.create_tokens(p.other(), Card::test_index(5298), 1, false);
        let target = g.state.objects.iter().enumerate().find_map(|(id, o)|
            (o.zone == Zone::Battlefield && o.controller == p.other() && o.is_token()).then_some(id as CardId)).unwrap();
        let source = put(&mut g, p, "Vindictive Triumph", Zone::Hand);
        cast(&mut g, source, Some(Target::Card(target)));
        vindictive_finish(&mut g);
        assert!(!g.on_battlefield(target));
        assert!(g.state.delayed_returns.is_none());
    }
}

#[test]
fn vindictive_delayed_exile_tracks_control_and_can_be_countered() {
    for p in [Player::P0, Player::P1] {
        for countered in [false, true] {
            let mut g = scenario(p);
            let target = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let source = put(&mut g, p, "Vindictive Triumph", Zone::Hand);
            cast(&mut g, source, Some(Target::Card(target)));
            vindictive_finish(&mut g);
            g.change_controller(g.object_ref(target), p.other());
            g.begin_step(Step::End);
            g.prepare_triggers();
            if countered {
                let id = g.state.stack.iter().find(|s| s.kind == StackKind::DelayedReturn).unwrap().id;
                assert!(g.counter_stack_item(id, false));
            }
            vindictive_finish(&mut g);
            assert_eq!(g.object_view(target).zone, if countered { Zone::Battlefield } else { Zone::Exile });
        }
    }
}
