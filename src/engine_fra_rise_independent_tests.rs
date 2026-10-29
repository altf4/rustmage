use super::*;

#[test]
fn rise_revealed_miracle_completes_before_draw_triggers_from_departed_source() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p, "Lorehold, the Historian", Zone::Battlefield);
        let watcher = put(&mut g, p.other(), "Scrawling Crawler", Zone::Battlefield);
        let revealed = put(&mut g, p, "Group Project", Zone::Library);
        let spell = put(&mut g, p, "Rise of the Deathbringer", Zone::Hand);
        let life = g.state.players[p.index()].life;
        act(&mut g, Action::AnnounceCast { card: spell });
        act(&mut g, Action::ChooseMode { mode: 0 });
        pay(&mut g);
        for _ in 0..32 {
            if matches!(g.state.pending, Pending::MiracleReveal { .. }) {
                break;
            }
            act(&mut g, Action::Pass);
        }
        assert_eq!(g.observe(p).miracle.unwrap().object.card, revealed);
        assert_eq!(g.state.players[p.index()].life, life);
        let before = g.snapshot();
        super::super::contracts::branch_contract(g.clone());
        for replay in 0..2 {
            if replay == 1 {
                g.restore(&before).unwrap();
            }
            act(&mut g, Action::RevealMiracle);
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 5);
            assert_eq!(g.state.players[p.index()].life, life - 5);
            assert!(g.state.known_hand.contains(&g.object_ref(revealed)));
            g.move_object(watcher, Zone::Exile);
            g.move_object(watcher, Zone::Battlefield);
            for _ in 0..100 {
                g.prepare_triggers();
                let actions = g.legal_actions();
                if actions.contains(&Action::FinishResolutionCasts) {
                    act(&mut g, Action::FinishResolutionCasts);
                } else if let Some(action) = actions
                    .into_iter()
                    .find(|a| matches!(a, Action::PutTrigger { .. } | Action::Pass))
                {
                    if g.state.pending == Pending::Priority
                        && g.state.stack.is_empty()
                        && g.state.pending_triggers.is_empty()
                        && g.state.resolving.is_none()
                    {
                        break;
                    }
                    act(&mut g, action);
                } else {
                    panic!("unexpected Rise continuation {:?}", g.state.pending);
                }
            }
            assert_eq!(g.state.players[p.index()].life, life - 10);
            assert_eq!(g.object_view(revealed).zone, Zone::Hand);
        }
    }
}

#[test]
fn rise_draw_uses_creature_power_and_controller_at_resolution() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let angel = put(&mut g, p.other(), "Serra Angel", Zone::Battlefield);
        let spell = put(&mut g, p, "Rise of the Deathbringer", Zone::Hand);
        let life = g.state.players[p.index()].life;
        act(&mut g, Action::AnnounceCast { card: spell });
        act(&mut g, Action::ChooseMode { mode: 0 });
        pay(&mut g);
        g.state.objects[bear as usize].growth.power = -1;
        let reference = g.object_ref(angel);
        g.change_controller(reference, p);
        let before = g.snapshot();
        super::super::contracts::branch_contract(g.clone());
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 4);
        assert_eq!(g.state.players[p.index()].life, life - 4);
        g.restore(&before).unwrap();
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, life - 4);
    }
}

#[test]
fn rise_wipe_applies_to_resolution_recipients_only_and_expires() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let angel = put(&mut g, p.other(), "Serra Angel", Zone::Battlefield);
        let spell = put(&mut g, p, "Rise of the Deathbringer", Zone::Hand);
        act(&mut g, Action::AnnounceCast { card: spell });
        act(&mut g, Action::ChooseMode { mode: 1 });
        pay(&mut g);
        let before = g.snapshot();
        super::super::contracts::branch_contract(g.clone());
        settle(&mut g);
        assert_eq!(stats(&g, angel), (1, 1));
        let later = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        assert_eq!(stats(&g, later), (2, 2));
        g.finish_cleanup();
        assert_eq!(stats(&g, angel), (4, 4));
        assert_eq!(stats(&g, later), (2, 2));
        g.restore(&before).unwrap();
        settle(&mut g);
        assert_eq!(stats(&g, angel), (1, 1));
    }
}
