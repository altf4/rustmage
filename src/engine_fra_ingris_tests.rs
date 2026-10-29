use super::*;

#[test]
fn ingris_triggers_for_each_attacker_with_departed_creature_damage_lki() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let ingris = put(&mut g, p, "Ingris Stingerquill", Zone::Battlefield);
        let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let giant = put(&mut g, p, "Hill Giant", Zone::Battlefield);
        g.state.objects[bear as usize].gained_keywords = Keywords::LIFELINK;
        let own_life = g.state.players[p.index()].life;
        let enemy_life = g.state.players[p.other().index()].life;
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        act(&mut g, Action::Attack { card: bear });
        act(&mut g, Action::Attack { card: giant });
        act(&mut g, Action::FinishAttackers);
        g.move_object(bear, Zone::Graveyard);
        g.move_object(ingris, Zone::Graveyard);
        while matches!(g.state.pending, Pending::OrderTriggers { .. }) {
            choose(&mut g, |a| matches!(a, Action::PutTrigger { .. }));
        }
        let saved = g.snapshot();
        super::super::contracts::branch_contract(g.clone());
        for replay in 0..2 {
            if replay == 1 {
                g.restore(&saved).unwrap();
            }
            settle(&mut g);
            assert_eq!(g.state.players[p.other().index()].life, enemy_life - 2);
            assert_eq!(g.state.players[p.index()].life, own_life + 1);
        }
    }
}

#[test]
fn ingris_only_observes_its_controllers_attackers_and_grants_haste_to_current_creatures() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let ingris = put(&mut g, p, "Ingris Stingerquill", Zone::Battlefield);
        let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        assert_eq!(stats(&g, ingris), (1, 4));
        assert!(g.keywords(ingris).contains(Keywords::FLYING));
        act(
            &mut g,
            Action::AnnounceActivate {
                card: ingris,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut g);
        g.move_object(ingris, Zone::Graveyard);
        settle(&mut g);
        let cadet = g
            .state
            .objects
            .iter()
            .enumerate()
            .find_map(|(id, o)| {
                (o.is_token()
                    && o.zone == Zone::Battlefield
                    && o.card == Card::test_index(5298)
                    && o.controller == p)
                    .then_some(id as CardId)
            })
            .unwrap();
        assert!(g.keywords(cadet).contains(Keywords::HASTE));
        assert!(g.keywords(own).contains(Keywords::HASTE));
        assert!(!g.keywords(enemy).contains(Keywords::HASTE));
        let late = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        assert!(!g.keywords(late).contains(Keywords::HASTE));
        g.finish_cleanup();
        assert!(!g.keywords(own).contains(Keywords::HASTE));
        assert!(!g.keywords(cadet).contains(Keywords::HASTE));
    }
}

#[test]
fn ingris_does_not_observe_opponents_attacking_creatures() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p.other());
        let ingris = put(&mut g, p, "Ingris Stingerquill", Zone::Battlefield);
        let attacker = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        act(&mut g, Action::Attack { card: attacker });
        act(&mut g, Action::FinishAttackers);
        assert!(
            !g.state
                .pending_triggers
                .iter()
                .chain(g.state.stack.iter())
                .any(|item| item.card == ingris)
        );
    }
}
