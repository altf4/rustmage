//! Frozen extra instances survive retained-mana event replay, not a sidecar.
use super::*;
#[test]
fn hob_extra_triggers_retained_mana_rollback_preserves_owned_death_instances() {
    for p in [Player::P0, Player::P1] {
        for keep in [false, true] {
            let (mut g, silver, enemy, prison) = setup(p);
            // Recruit needs a real private draw/discard after the retained event.
            put(&mut g, p, "Forest", Zone::Library);
            put(&mut g, p, "Forest", Zone::Library);
            let lookout = put(&mut g, p, "Lake-town Lookout", Zone::Battlefield);
            let ring = put(&mut g, p, "Lotus Ring", Zone::Battlefield);
            let staff = put(&mut g, p, "Wizard's Staff", Zone::Battlefield);
            assert!(g.attach_equipment(g.object_ref(ring), g.object_ref(lookout)));
            assert!(g.attach_equipment(g.object_ref(staff), g.object_ref(lookout)));
            g.state.objects[lookout as usize].summoning_sick = false;
            let spell = put(&mut g, p, "Candy Grapple", Zone::Hand);
            g.state.players[p.index()].mana = [0; 6];
            propose(&mut g, spell, prison, enemy);
            let grantor = g.object_ref(ring);
            checked(
                &mut g,
                Action::TapGrantedMana {
                    card: lookout,
                    grantor,
                    production: [0, 0, 3, 0, 0, 0],
                },
            );
            let captured = &g.state.cast_rollback.as_ref().unwrap().events[0].triggers;
            assert_eq!(captured.len(), 1);
            assert_eq!(captured[0].additional_triggers, 1);
            fail(&mut g, silver);
            if keep {
                crate::engine::assurance::contracts::branch_contract(g.clone());
            }
            checked(&mut g, Action::ChooseManaRollback { keep });
            assert_eq!(g.object_view(spell).zone, Zone::Hand);
            assert_eq!(
                g.object_view(lookout).zone,
                if keep {
                    Zone::Graveyard
                } else {
                    Zone::Battlefield
                }
            );
            assert_eq!(
                g.state.players[p.index()].mana,
                if keep { [0, 0, 3, 0, 0, 0] } else { [0; 6] }
            );
            if !keep {
                assert_eq!(
                    g.object_view(staff).attached_to,
                    Some(g.object_ref(lookout))
                );
                assert!(g.state.pending_triggers.is_empty());
                continue;
            }
            let before = g.state.players[p.index()].cards_drawn_this_turn;
            let mut discards = 0;
            for _ in 0..64 {
                if g.state.pending == Pending::Priority
                    && g.state.stack.is_empty()
                    && g.state.pending_triggers.is_empty()
                    && g.state.resolving.is_none()
                {
                    break;
                }
                let actions = g.legal_actions();
                let action = if matches!(g.state.pending, Pending::EffectDiscard { .. }) {
                    assert!(g.observe(p.other()).effect_discards.is_empty());
                    discards += 1;
                    actions
                        .into_iter()
                        .find(|a| matches!(a, Action::Discard { .. }))
                } else {
                    actions.into_iter().find(|a| {
                        matches!(
                            a,
                            Action::PutTrigger { .. } | Action::FinishTargets | Action::Pass
                        )
                    })
                }
                .unwrap_or_else(|| {
                    panic!("unexpected rollback replay choice {:?}", g.state.pending)
                });
                checked(&mut g, action);
            }
            assert_eq!(discards, 2);
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, before + 2);
            assert!(g.state.cast_rollback.is_none());
        }
    }
}
