use super::*;

#[test]
fn memorial_projected_mana_funds_only_a_planeswalker_spell() {
    for player in [Player::P0, Player::P1] {
        for walker in [false, true] {
            let mut g = scenario(player);
            let memorial = put(&mut g, player, "Gideon's Memorial", Zone::Battlefield);
            let card = put(
                &mut g,
                player,
                if walker {
                    "Ajani, Caller of the Pride"
                } else {
                    "Ajani's Pridemate"
                },
                Zone::Hand,
            );
            g.state.players[player.index()].mana = if walker {
                [1, 0, 0, 0, 0, 1]
            } else {
                [0, 0, 0, 0, 0, 1]
            };
            let announcement = Action::AnnounceCast { card };
            let before = g.snapshot();
            assert_eq!(g.legal_actions().contains(&announcement), walker);
            assert_eq!(g.snapshot(), before);
            let produce = g.legal_actions().into_iter().find(|a| matches!(a,
                Action::ActivateMana { card, ability: 0, production: [1, 0, 0, 0, 0, 0], .. } if *card == memorial)).unwrap();
            act(&mut g, produce);
            assert_eq!(g.legal_actions().contains(&announcement), walker);
            g.restore(&before).unwrap();
            assert_eq!(g.snapshot(), before);
        }
    }
}

#[test]
fn memorial_restricted_mana_cannot_pay_its_hand_activation() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player.other());
        let attacker = put(&mut g, player.other(), "Hill Giant", Zone::Battlefield);
        let battlefield_memorial = put(&mut g, player, "Gideon's Memorial", Zone::Battlefield);
        let hand_memorial = put(&mut g, player, "Gideon's Memorial", Zone::Hand);
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        act(&mut g, Action::Attack { card: attacker });
        act(&mut g, Action::FinishAttackers);
        g.state.priority = player;
        g.state.players[player.index()].mana = [0, 0, 0, 0, 0, 1];
        let activate = Action::AnnounceActivate {
            card: hand_memorial,
            ability: 1,
            counter_payment: Counters::default(),
        };
        assert!(!g.legal_actions().contains(&activate));
        let mana = g.legal_actions().into_iter().find(|a| matches!(a,
            Action::ActivateMana { card, ability: 0, production: [1, 0, 0, 0, 0, 0], .. } if *card == battlefield_memorial)).unwrap();
        act(&mut g, mana);
        assert!(!g.legal_actions().contains(&activate));
        assert_eq!(g.object_view(hand_memorial).zone, Zone::Hand);
    }
}
