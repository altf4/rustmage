use super::*;

fn announce_cell_target(g: &mut Game, cell: CardId, target: CardId) {
    g.move_object(cell, Zone::Battlefield);
    g.prepare_triggers();
    for _ in 0..32 {
        if g.legal_actions().contains(&Action::ChooseTarget {
            target: Target::Card(target),
        }) {
            break;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    act(
        g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
}

#[test]
fn solitary_cell_source_or_target_leaving_before_resolution_prevents_exile() {
    for player in [Player::P0, Player::P1] {
        for remove_source in [false, true] {
            let mut g = scenario(player);
            let target = put(&mut g, player.other(), "Bear Cub", Zone::Battlefield);
            let cell = put(&mut g, player, "Solitary Cell", Zone::Hand);
            announce_cell_target(&mut g, cell, target);
            if remove_source {
                g.move_object(cell, Zone::Graveyard);
            } else {
                let old = g.object_ref(target);
                g.move_object(target, Zone::Exile);
                g.move_object(target, Zone::Battlefield);
                assert_ne!(g.object_ref(target), old);
            }
            let saved = g.snapshot();
            for replay in 0..2 {
                if replay == 1 {
                    g.restore(&saved).unwrap();
                }
                settle(&mut g);
                assert_eq!(g.object_view(target).zone, Zone::Battlefield);
            }
        }
    }
}

#[test]
fn solitary_cell_discard_replacement_still_pays_and_draws() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        put(&mut g, player, "Rest in Peace", Zone::Battlefield);
        let cell = put(&mut g, player, "Solitary Cell", Zone::Battlefield);
        let legend = put(&mut g, player, "Ajani, Caller of the Pride", Zone::Hand);
        let drawn = g.state.players[player.index()].cards_drawn_this_turn;
        act(
            &mut g,
            Action::AnnounceActivate {
                card: cell,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        let object = g.object_ref(legend);
        act(&mut g, Action::ChooseAdditionalCost { object });
        pay(&mut g);
        assert!(g.object_view(cell).tapped);
        assert_eq!(g.object_view(legend).zone, Zone::Exile);
        let saved = g.snapshot();
        for replay in 0..2 {
            if replay == 1 {
                g.restore(&saved).unwrap();
            }
            settle(&mut g);
            assert_eq!(
                g.state.players[player.index()].cards_drawn_this_turn,
                drawn + 1
            );
        }
    }
}
