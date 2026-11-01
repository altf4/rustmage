use super::*;

#[test]
fn solitary_cell_requires_a_legendary_discard_and_preserves_private_hand_and_rollback() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let cell = put(&mut g, p, "Solitary Cell", Zone::Battlefield);
        let nonlegend = put(&mut g, p, "Bear Cub", Zone::Hand);
        let foreign = put(&mut g, p.other(), "Ajani, Caller of the Pride", Zone::Hand);
        let action = Action::AnnounceActivate {
            card: cell,
            ability: 0,
            counter_payment: Counters::default(),
        };
        reject(&mut g, action.clone());
        let legend = put(&mut g, p, "Ajani, Caller of the Pride", Zone::Hand);
        assert!(!g.observe(p.other()).objects.iter().any(|o| o.id == legend));
        g.state.players[p.index()].mana = [0; 6];
        reject(&mut g, action.clone());
        assert_eq!(g.object_view(legend).zone, Zone::Hand);
        g.state.players[p.index()].mana[5] = 1;
        act(&mut g, action);
        let own_ref = g.object_ref(legend);
        let nonlegend_ref = g.object_ref(nonlegend);
        let foreign_ref = g.object_ref(foreign);
        reject(
            &mut g,
            Action::ChooseAdditionalCost {
                object: nonlegend_ref,
            },
        );
        reject(
            &mut g,
            Action::ChooseAdditionalCost {
                object: foreign_ref,
            },
        );
        let saved = g.snapshot();
        super::super::contracts::branch_contract(g.clone());
        for replay in 0..2 {
            if replay == 1 {
                g.restore(&saved).unwrap();
            }
            act(&mut g, Action::ChooseAdditionalCost { object: own_ref });
            pay(&mut g);
            assert!(g.object_view(cell).tapped);
            assert_eq!(g.object_view(legend).zone, Zone::Graveyard);
            let drawn = g.state.players[p.index()].cards_drawn_this_turn;
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, drawn + 1);
        }
    }
}

#[test]
fn solitary_cell_exiles_only_opposing_small_nonlands_and_returns_on_departure() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let target = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let land = put(&mut g, p.other(), "Forest", Zone::Battlefield);
        let huge = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let cell = put(&mut g, p, "Solitary Cell", Zone::Hand);
        g.move_object(cell, Zone::Battlefield);
        g.prepare_triggers();
        for _ in 0..32 {
            if g.legal_actions().contains(&Action::ChooseTarget {
                target: Target::Card(target),
            }) {
                break;
            }
            choose(&mut g, |a| {
                matches!(a, Action::Pass | Action::PutTrigger { .. })
            });
        }
        for id in [own, land, huge] {
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(id),
                },
            );
        }
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            act(&mut g, Action::FinishTargets);
        }
        settle(&mut g);
        assert_eq!(g.object_view(target).zone, Zone::Exile);
        g.move_object(cell, Zone::Graveyard);
        assert_eq!(g.object_view(target).zone, Zone::Battlefield);
        assert_eq!(g.object_view(target).controller, p.other());
    }
}
