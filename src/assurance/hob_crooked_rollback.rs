//! A real graveyard-consuming mana event keeps or reverses its frozen amass packet.
use super::*;
#[test]
fn hob_crooked_galion_retained_mana_keeps_graveyard_departure_once_and_reverse_discards() {
    for p in [Player::P0, Player::P1] {
        for keep in [false, true] {
            let (mut g, silver, enemy, prison) = setup(p);
            let source = put(&mut g, p, "Along the Crooked Way", Zone::Battlefield);
            let rubble = put(&mut g, p, "Rubble Rouser", Zone::Battlefield);
            // This observer-producing mana fixture must not rescue the casualty
            // payment whose deliberate failure opens the ordinary rollback choice.
            g.state.objects[rubble as usize].growth.power = -1;
            let fuel = put(&mut g, p, "Bear Cub", Zone::Graveyard);
            let spell = put(&mut g, p, "Candy Grapple", Zone::Hand);
            g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 0];
            propose(&mut g, spell, prison, enemy);
            checked(
                &mut g,
                Action::AnnounceMana {
                    card: rubble,
                    ability: 0,
                    counter_payment: Counters::default(),
                    production: [0, 0, 0, 1, 0, 0],
                },
            );
            let object = g.object_ref(fuel);
            checked(&mut g, Action::ChooseAdditionalCost { object });
            pay(&mut g);
            let captured = &g.state.cast_rollback.as_ref().unwrap().events[0].triggers;
            assert_eq!(captured.iter().filter(|s| s.source.is_some_and(|identity| identity.id == source)).count(), 1);
            fail(&mut g, silver);
            crate::engine::assurance::contracts::branch_contract(g.clone());
            checked(&mut g, Action::ChooseManaRollback { keep });
            assert_eq!(
                g.object_view(fuel).zone,
                if keep { Zone::Exile } else { Zone::Graveyard }
            );
            crate::engine::assurance::hob_crooked_galion::drive(&mut g, None);
            let armies = crate::engine::assurance::hob_crooked_galion::armies(&g, p);
            assert_eq!(armies.len(), usize::from(keep));
            if keep {
                assert_eq!(g.counter_count(armies[0], CounterKind::PlusOne), 1);
            }
            assert!(g.state.cast_rollback.is_none());
        }
    }
}
