use super::*;

fn announce_hall(g: &mut Game, hall: CardId, target: CardId) {
    act(
        g,
        Action::AnnounceActivate {
            card: hall,
            ability: 1,
            counter_payment: Counters::default(),
        },
    );
    act(
        g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
    pay(g);
}

#[test]
fn hall_stolen_after_activation_copies_for_new_controller_but_exception_stays_with_activator() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let hall = put(&mut g, p, "Hall of Echoes", Zone::Battlefield);
        let target = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        announce_hall(&mut g, hall, target);
        let r = g.object_ref(hall);
        g.change_controller(r, p.other());
        let saved = g.snapshot();
        super::super::contracts::branch_contract(g.clone());
        for replay in 0..2 {
            if replay == 1 {
                g.restore(&saved).unwrap();
            }
            settle(&mut g);
            assert_eq!(g.object_name(hall), g.object_name(target));
            assert_eq!(g.object_view(hall).controller, p.other());
            assert!(g.state.players[p.index()].legend_rule_ignored_this_turn);
            assert!(!g.state.players[p.other().index()].legend_rule_ignored_this_turn);
        }
    }
}

#[test]
fn hall_source_reentry_does_not_copy_new_incarnation_but_exception_still_applies() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let hall = put(&mut g, p, "Hall of Echoes", Zone::Battlefield);
        let target = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        announce_hall(&mut g, hall, target);
        let original = g.object_ref(hall);
        g.move_object(hall, Zone::Exile);
        g.move_object(hall, Zone::Battlefield);
        assert_ne!(g.object_ref(hall), original);
        let saved = g.snapshot();
        super::super::contracts::branch_contract(g.clone());
        for replay in 0..2 {
            if replay == 1 {
                g.restore(&saved).unwrap();
            }
            settle(&mut g);
            assert_eq!(g.object_name(hall), "Hall of Echoes");
            assert!(g.state.players[p.index()].legend_rule_ignored_this_turn);
        }
    }
}

#[test]
fn hall_taps_for_actual_colorless_mana() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let hall = put(&mut g, p, "Hall of Echoes", Zone::Battlefield);
        g.state.players[p.index()].mana = [0; 6];
        let action = g.legal_actions().into_iter().find(|a| matches!(a,
            Action::ActivateMana { card, ability: 0, production: [0, 0, 0, 0, 0, 1], .. } if *card == hall)).unwrap();
        act(&mut g, action);
        assert_eq!(g.state.players[p.index()].mana[5], 1);
        assert!(g.state.objects[hall as usize].tapped);
    }
}
