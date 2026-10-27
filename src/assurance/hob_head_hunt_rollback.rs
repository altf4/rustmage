//! Actual granted sacrifice mana retained/reversed through the foreign Aura failure.
use super::*;
#[test]
fn hob_hunt_mana_rollback_keeps_exact_reflexive_packet_once_or_reverses_it_entirely() {
    for p in [Player::P0, Player::P1] {
        for keep in [false, true] {
            let (mut g, silver, enemy, prison) = setup(p);
            let h = put(&mut g, p.other(), "Head of the Hunt", Zone::Battlefield);
            let source = g.object_ref(h);
            let staff = put(&mut g, p.other(), "Wizard's Staff", Zone::Battlefield);
            assert!(g.attach_equipment(g.object_ref(staff), source));
            assert_eq!(g.object_view(h).additional_triggers, 1);
            let fodder = put(&mut g, p, "Fanatical Firebrand", Zone::Battlefield);
            g.state.objects[fodder as usize].growth.power = -1;
            let ring = put(&mut g, p, "Lotus Ring", Zone::Battlefield);
            assert!(g.attach_equipment(g.object_ref(ring), g.object_ref(fodder)));
            // Ring's stat bonus must not make casualty1 newly payable after the foreign Aura.
            g.state.objects[fodder as usize].growth.power = -4;
            let spell = put(&mut g, p, "Candy Grapple", Zone::Hand);
            g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 0];
            propose(&mut g, spell, prison, enemy);
            let grantor = g.object_ref(ring);
            checked(
                &mut g,
                Action::TapGrantedMana {
                    card: fodder,
                    grantor,
                    production: [0, 0, 3, 0, 0, 0],
                },
            );
            assert_eq!(g.object_view(fodder).zone, Zone::Exile);
            assert_eq!(
                g.state
                    .pending_triggers
                    .iter()
                    .filter(|t| t.object == source)
                    .count(),
                1
            );
            fail(&mut g, silver);
            crate::engine::assurance::contracts::branch_contract(g.clone());
            checked(&mut g, Action::ChooseManaRollback { keep });
            finish(&mut g);
            assert_eq!(
                g.object_view(fodder).zone,
                if keep { Zone::Exile } else { Zone::Battlefield }
            );
            assert_eq!(g.object_view(spell).zone, Zone::Hand);
            let wolves = g
                .state
                .objects
                .iter()
                .filter(|o| {
                    o.zone == Zone::Battlefield
                        && o.controller == p.other()
                        && o.is_token()
                        && o.card.index() == 5412
                })
                .count();
            assert_eq!(wolves, usize::from(keep));
            assert!(g.state.cast_rollback.is_none());
        }
    }
}
