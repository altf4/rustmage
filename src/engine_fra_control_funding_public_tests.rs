//! Real-card projected control funding interaction; link under FRA assurance.
use super::*;

#[test]
fn saga_control_returns_land_during_lotus_ring_payment() {
    for player in [Player::P0, Player::P1] {
        let opponent = player.other();
        let mut g = scenario(opponent);
        // Starting permanents and token are fixture setup. The interaction
        // from the Saga cast onward uses public actions.
        put(
            &mut g,
            player,
            "Samut, Hazoret's Champion",
            Zone::Battlefield,
        );
        g.create_tokens(player, Card::test_index(5454), 1, false);
        let land = g
            .state
            .objects
            .iter()
            .enumerate()
            .find_map(|(i, o)| {
                (o.owner == player && o.is_token() && o.card == Card::test_index(5454))
                    .then_some(i as CardId)
            })
            .unwrap();
        let saga = put(&mut g, opponent, "The Super Hero Civil War", Zone::Hand);
        cast(&mut g, saga, None);
        for _ in 0..80 {
            if g.legal_actions().contains(&Action::ChooseTarget {
                target: Target::Card(land),
            }) {
                act(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(land),
                    },
                );
                if g.legal_actions().contains(&Action::FinishTargets) {
                    act(&mut g, Action::FinishTargets);
                }
                break;
            }
            choose(&mut g, |a| {
                matches!(a, Action::Pass | Action::PutTrigger { .. })
            });
        }
        settle(&mut g);
        assert_eq!(g.object_view(land).controller, opponent);
        // Advance the opponent's turn through public priority/combat actions.
        for _ in 0..100 {
            if g.state.active == player
                && g.state.step == Step::Main1
                && g.state.pending == Pending::Priority
            {
                break;
            }
            choose(&mut g, |a| {
                matches!(
                    a,
                    Action::Pass
                        | Action::FinishAttackers
                        | Action::FinishBlockers
                        | Action::PutTrigger { .. }
                )
            });
        }
        assert_eq!(g.state.active, player);
        assert_eq!(g.state.step, Step::Main1);
        // Mana pools empty at each step; fixture funds the setup spells after
        // the public turn transition. The final payment is tested from zero.
        g.state.players[player.index()].mana = [8; 6];
        let confiscate = put(&mut g, player, "Confiscate", Zone::Hand);
        cast(&mut g, confiscate, Some(Target::Card(saga)));
        settle(&mut g);
        assert_eq!(g.object_view(saga).controller, player);
        assert_eq!(g.object_view(land).controller, opponent);
        let puppet = put(&mut g, player, "Puppet Crafting", Zone::Hand);
        cast(&mut g, puppet, Some(Target::Card(saga)));
        settle(&mut g);
        assert_ne!(g.current_kinds(saga) & 1, 0);
        let ring = put(&mut g, player, "Lotus Ring", Zone::Battlefield);
        act(
            &mut g,
            Action::AnnounceActivate {
                card: ring,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(saga),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            act(&mut g, Action::FinishTargets);
        }
        pay(&mut g);
        settle(&mut g);
        assert_eq!(
            g.state.objects[ring as usize].attached_to,
            Some(g.object_ref(saga))
        );
        let spell = put(&mut g, player, "Titanbones, Towering Heart", Zone::Hand);
        // Strip setup mana so the cost requires both the sacrifice and land.
        g.state.players[player.index()].mana = [0; 6];
        let before = g.snapshot();
        let revision = g.revision();
        let action = Action::AnnounceCast { card: spell };
        let index = g
            .legal_actions()
            .iter()
            .position(|a| *a == action)
            .expect("projected control permits funding");
        for _ in 0..2 {
            act(&mut g, action.clone());
            let mana = g.legal_actions().into_iter().find(|a| matches!(a,
                Action::TapGrantedMana { card, grantor, .. } if *card == saga && *grantor == g.object_ref(ring))).expect("Saga may be sacrificed for mana");
            act(&mut g, mana);
            assert_eq!(g.object_view(saga).zone, Zone::Graveyard);
            assert_eq!(g.object_view(land).controller, player);
            act(&mut g, Action::TapMana { card: land });
            pay(&mut g);
            assert_eq!(g.object_view(spell).zone, Zone::Stack);
            g.restore(&before).unwrap();
        }
        assert!(g.revision() > revision);
        assert_eq!(
            g.apply_index(player, revision, index),
            Err(Error::StaleRevision)
        );
        let fresh = g.revision();
        let index = g.legal_actions().iter().position(|a| *a == action).unwrap();
        assert!(g.apply_index(player, fresh, index).is_ok());
    }
}
