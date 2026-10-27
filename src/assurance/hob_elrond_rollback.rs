//! Frozen actual mana-trigger packets and their exact per-object latch replay.
use super::*;
#[test]
fn hob_elrond_retained_mana_replays_draw_and_latch_only_for_kept_event() {
    for p in [Player::P0, Player::P1] {
        for keep in [false, true] {
            let (mut g, silver, enemy, prison) = setup(p);
            let source = put(&mut g, p, "Elrond, Moon-Reader", Zone::Battlefield);
            g.state.objects[source as usize].growth.power = -3; // observer must not rescue failed casualty
            let rubble = put(&mut g, p, "Rubble Rouser", Zone::Battlefield);
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
            assert!(
                g.state.stack.is_empty(),
                "mana observer must wait for announced payment"
            );
            let captured = &g.state.cast_rollback.as_ref().unwrap().events[0].triggers;
            assert_eq!(captured.iter().filter(|s| s.card == source).count(), 1);
            fail(&mut g, silver);
            contracts::branch_contract(g.clone());
            checked(&mut g, Action::ChooseManaRollback { keep });
            assert_eq!(
                g.object_view(fuel).zone,
                if keep { Zone::Exile } else { Zone::Graveyard }
            );
            let before = g.state.players[p.index()].cards_drawn_this_turn;
            crate::engine::assurance::hob_elrond::finish(&mut g);
            assert_eq!(
                g.state.players[p.index()].cards_drawn_this_turn,
                before + u64::from(keep)
            );
            assert_eq!(
                g.state
                    .triggered_this_turn
                    .iter()
                    .filter(|u| u.source.card == source)
                    .count(),
                usize::from(keep)
            );
            let elf = put(&mut g, p, "Llanowar Elves", Zone::Battlefield);
            crate::engine::assurance::hob_elrond::mana(&mut g, elf);
            crate::engine::assurance::hob_elrond::finish(&mut g);
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, before + 1);
        }
    }
}
