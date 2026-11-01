//! Focused printed outcomes; shared engine invariants live in core tests.
use super::*;

fn vindictive_finish(g: &mut Game) {
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
    panic!(
        "Vindictive Triumph continuation did not finish: {:?}",
        g.state.pending
    );
}

#[test]
fn vindictive_exiled_creature_token_does_not_return() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        g.create_tokens(
            p.other(),
            Card::from_name("@token/adcb48b581ad80b8654a11b9").unwrap(),
            1,
            false,
        );
        let target = g
            .state
            .objects
            .iter()
            .enumerate()
            .find_map(|(id, o)| {
                (o.zone == Zone::Battlefield && o.controller == p.other() && o.is_token())
                    .then_some(id as CardId)
            })
            .unwrap();
        let source = put(&mut g, p, "Vindictive Triumph", Zone::Hand);
        cast(&mut g, source, Some(Target::Card(target)));
        vindictive_finish(&mut g);
        assert!(!g.on_battlefield(target));
        assert!(g.state.delayed_returns.is_none());
    }
}

#[test]
fn vindictive_delayed_exile_tracks_control_and_can_be_countered() {
    {
        let p = Player::P0;
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
                let id = g
                    .state
                    .stack
                    .iter()
                    .find(|s| s.kind == StackKind::DelayedReturn)
                    .unwrap()
                    .id;
                assert!(g.counter_stack_item(id, false));
            }
            vindictive_finish(&mut g);
            assert_eq!(
                g.object_view(target).zone,
                if countered {
                    Zone::Battlefield
                } else {
                    Zone::Exile
                }
            );
        }
    }
}
