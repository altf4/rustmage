//! Actual failed paid graveyard casts restore their enclosing owned resolution frame.
use super::*;
#[test]
fn hob_bilbo_failed_paid_cast_restores_quota_and_exact_graveyard_with_retained_or_reversed_mana() {
    for p in [Player::P0, Player::P1] {
        for keep in [false, true] {
            let (mut g, silver, enemy, prison) = setup(p);
            let bilbo = put(&mut g, p, "Bilbo, Thief in the Night", Zone::Battlefield);
            g.state.objects[bilbo as usize].growth.power = -2;
            let firebrand = put(&mut g, p, "Fanatical Firebrand", Zone::Battlefield);
            let ring = put(&mut g, p, "Lotus Ring", Zone::Battlefield);
            assert!(g.attach_equipment(g.object_ref(ring), g.object_ref(firebrand)));
            g.state.objects[firebrand as usize].growth.power = -4;
            let spell = put(&mut g, p, "Candy Grapple", Zone::Graveyard);
            let original = g.object_ref(spell);
            g.state.players[p.index()].mana = [0; 6];
            g.state.step = Step::Attackers;
            g.state.pending = Pending::Attackers;
            checked(&mut g, Action::Attack { card: bilbo });
            checked(&mut g, Action::FinishAttackers);
            g.prepare_triggers();
            for _ in 0..32 {
                if matches!(g.state.pending, Pending::ResolutionCasts { .. }) {
                    break;
                }
                let a = g
                    .legal_actions()
                    .into_iter()
                    .find(|a| matches!(a, Action::Pass | Action::PutTrigger { .. }))
                    .unwrap();
                checked(&mut g, a);
            }
            assert!(matches!(g.state.pending, Pending::ResolutionCasts { .. }));
            checked(
                &mut g,
                Action::CastExiledDuringResolution {
                    object: original,
                    face: CastFace::Normal,
                },
            );
            let sacrifice = Some(g.object_ref(prison));
            checked(&mut g, Action::ChooseBargain { sacrifice });
            checked(&mut g, Action::ChooseCasualty { pay: true });
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(enemy),
                },
            );
            let grantor = g.object_ref(ring);
            checked(
                &mut g,
                Action::TapGrantedMana {
                    card: firebrand,
                    grantor,
                    production: [0, 0, 3, 0, 0, 0],
                },
            );
            fail(&mut g, silver);
            crate::engine::assurance::contracts::branch_contract(g.clone());
            checked(&mut g, Action::ChooseManaRollback { keep });
            assert_eq!(g.object_ref(spell), original);
            assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
            assert_eq!(
                g.state
                    .resolution_free_casts
                    .as_ref()
                    .unwrap()
                    .casts_remaining,
                Some(1)
            );
            assert!(
                g.state
                    .resolution_free_casts
                    .as_ref()
                    .unwrap()
                    .remaining
                    .contains(&original)
            );
            assert_eq!(g.state.pending, Pending::ResolutionCasts { player: p });
            assert_eq!(
                g.object_view(firebrand).zone,
                if keep {
                    Zone::Graveyard
                } else {
                    Zone::Battlefield
                }
            );
            assert_eq!(
                g.state.players[p.index()].spells_cast_this_turn,
                1,
                "only setup's Banishing Light was cast"
            );
            assert_eq!(
                g.state.players[p.index()].mana,
                if keep { [0, 0, 3, 0, 0, 0] } else { [0; 6] }
            );
            checked(&mut g, Action::FinishResolutionCasts);
            finish(&mut g);
            assert!(g.state.cast_rollback.is_none());
        }
    }
}
