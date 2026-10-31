//! Kaito’s printed abilities through ordinary gameplay.
use super::big_helpers::{cast as paid_cast, checked, settle as finish};
use super::*;

#[test]
fn kaito_bane_of_nightmares_full_card() {
    let emblem = Card::from_name("@emblem/dsk-kaito-bane-of-nightmares").unwrap();
    for player in [Player::P0, Player::P1] {
        let mut casted = scenario(player);
        let cast_card = put(&mut casted, player, "Kaito, Bane of Nightmares", Zone::Hand);
        paid_cast(&mut casted, cast_card, None, &[]);
        finish(&mut casted);
        assert_eq!(casted.object_view(cast_card).zone, Zone::Battlefield);
        assert_eq!(casted.counter_count(cast_card, CounterKind::Loyalty), 4);
        assert_eq!(stats(&casted, cast_card), (3, 4));

        let mut g = scenario(player);
        let hand = put(&mut g, player, "Kaito, Bane of Nightmares", Zone::Hand);
        let attacker = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        g.state.step = Step::Blockers;
        g.state.attackers.push(AttackDeclaration {
            card: attacker,
            defender: Some(CombatDefender::Player(player.other())),
            blocked: false,
        });
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: hand,
                ability: 1,
                counter_payment: Counters::default(),
            },
        );
        assert!(
            g.observe(player.other())
                .known_hand
                .contains(&g.object_ref(hand))
        );
        let object = g.object_ref(attacker);
        checked(&mut g, Action::ChooseAdditionalCost { object });
        pay(&mut g);
        finish(&mut g);
        assert_eq!(g.object_view(attacker).zone, Zone::Hand);
        assert_eq!(g.object_view(hand).zone, Zone::Battlefield);
        assert!(g.object_view(hand).tapped);
        assert!(g.state.attackers.iter().any(|a| a.card == hand
            && a.defender == Some(CombatDefender::Player(player.other()))
            && !a.blocked));
        assert_eq!(g.counter_count(hand, CounterKind::Loyalty), 4);
        assert_eq!(stats(&g, hand), (3, 4));
        assert!(g.has_subtype(hand, "Ninja"));
        assert!(g.keywords(hand).contains(Keywords::HEXPROOF));
        let mut other_turn = g.clone();
        other_turn.state.active = player.other();
        assert!(!other_turn.is_creature(hand));
        assert!(!other_turn.keywords(hand).contains(Keywords::HEXPROOF));

        g.state.step = Step::Main1;
        g.state.attackers.clear();
        let mut surveil = g.clone();
        let mut stun = g.clone();
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: hand,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut g);
        finish(&mut g);
        assert!(
            g.state
                .objects
                .iter()
                .any(|o| o.card == emblem && o.zone == Zone::Command)
        );
        assert_eq!(stats(&g, hand), (4, 5));

        let shock = put(&mut surveil, player, "Shock", Zone::Hand);
        paid_cast(&mut surveil, shock, None, &[Target::Player(player.other())]);
        finish(&mut surveil);
        let before = surveil.hand_count(player);
        checked(
            &mut surveil,
            Action::AnnounceActivate {
                card: hand,
                ability: 2,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut surveil);
        finish(&mut surveil);
        assert_eq!(surveil.hand_count(player), before + 1);

        let enemy = put(&mut stun, player.other(), "Bear Cub", Zone::Battlefield);
        checked(
            &mut stun,
            Action::AnnounceActivate {
                card: hand,
                ability: 3,
                counter_payment: Counters::default(),
            },
        );
        checked(
            &mut stun,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        if stun.legal_actions().contains(&Action::FinishTargets) {
            checked(&mut stun, Action::FinishTargets);
        }
        pay(&mut stun);
        finish(&mut stun);
        assert!(stun.object_view(enemy).tapped);
        assert_eq!(stun.counter_count(enemy, CounterKind::Stun), 2);
    }
}
