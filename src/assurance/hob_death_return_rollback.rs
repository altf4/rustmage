use super::*;
#[test]
fn hob_death_return_retained_mana_replays_batch_witness_once_and_reverse_discards_it() {
    for p in [Player::P0, Player::P1] {
        for keep in [false, true] {
            let (mut g, silver, enemy, prison) = setup(p);
            let bee = put(&mut g, p, "Great Fierce Bee", Zone::Battlefield);
            // Keep the observer alive but ineligible for the casualty payment whose failure
            // opens rollback after the mana sacrifice.
            g.state.objects[bee as usize].growth.power = -2;
            let victim = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let staff = put(&mut g, p, "Wizard's Staff", Zone::Battlefield);
            let ring = put(&mut g, p, "Lotus Ring", Zone::Battlefield);
            assert!(g.attach_equipment(g.object_ref(staff), g.object_ref(bee)));
            assert!(g.attach_equipment(g.object_ref(ring), g.object_ref(victim)));
            let spell = put(&mut g, p, "Candy Grapple", Zone::Hand);
            g.state.players[p.index()].mana = [0; 6];
            propose(&mut g, spell, prison, enemy);
            let grantor = g.object_ref(ring);
            checked(
                &mut g,
                Action::TapGrantedMana {
                    card: victim,
                    grantor,
                    production: [0, 0, 3, 0, 0, 0],
                },
            );
            let captured = &g.state.cast_rollback.as_ref().unwrap().events[0].triggers;
            let packets: Vec<_> = captured.iter().filter(|s| s.source.is_some_and(|source| source.id == bee)).collect();
            assert_eq!(packets.len(), 1);
            assert_eq!(packets[0].additional_triggers, 1);
            fail(&mut g, silver);
            checked(&mut g, Action::ChooseManaRollback { keep });
            assert_eq!(
                g.object_view(victim).zone,
                if keep {
                    Zone::Graveyard
                } else {
                    Zone::Battlefield
                }
            );
            assert_eq!(
                crate::engine::assurance::hob_death_return::drive(&mut g),
                if keep { 2 } else { 0 }
            );
            assert!(g.state.cast_rollback.is_none());
        }
    }
}
