use super::*;

#[test]
fn ingris_uses_attacker_latest_lifelink_lki_and_trigger_controller() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let ingris = put(&mut g, player, "Ingris Stingerquill", Zone::Battlefield);
        let attacker = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        let life = g.state.players[player.index()].life;
        let opposing_life = g.state.players[player.other().index()].life;
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        act(&mut g, Action::Attack { card: attacker });
        act(&mut g, Action::FinishAttackers);
        // The attacker gains lifelink after the declaration, before it leaves.
        g.state.objects[attacker as usize].gained_keywords = Keywords::LIFELINK;
        g.move_object(attacker, Zone::Graveyard);
        let source = g.object_ref(ingris);
        g.change_controller(source, player.other());
        let before = g.snapshot();
        for replay in 0..2 {
            if replay == 1 {
                g.restore(&before).unwrap();
            }
            settle(&mut g);
            assert_eq!(g.state.players[player.index()].life, life + 1);
            assert_eq!(
                g.state.players[player.other().index()].life,
                opposing_life - 1
            );
        }
    }
}

#[test]
fn ingris_triggers_for_creature_attacking_an_opposing_planeswalker() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        put(&mut g, player, "Ingris Stingerquill", Zone::Battlefield);
        let attacker = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        g.create_tokens(player.other(), Card::JacePlaneswalkerToken, 1, false);
        let defender = g
            .state
            .objects
            .iter()
            .enumerate()
            .find_map(|(id, o)| {
                (o.zone == Zone::Battlefield
                    && o.controller == player.other()
                    && o.is_token()
                    && g.has_subtype(id as CardId, "Jace"))
                .then_some(id as CardId)
            })
            .unwrap();
        // Give the fixture planeswalker loyalty so it survives state-based actions.
        g.state.objects[defender as usize].counters.loyalty = 3;
        let life = g.state.players[player.other().index()].life;
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        let action = Action::AttackPlaneswalker {
            card: attacker,
            defender: g.object_ref(defender),
        };
        assert!(g.legal_actions().contains(&action));
        act(&mut g, action);
        act(&mut g, Action::FinishAttackers);
        settle(&mut g);
        assert_eq!(g.state.players[player.other().index()].life, life - 1);
    }
}
