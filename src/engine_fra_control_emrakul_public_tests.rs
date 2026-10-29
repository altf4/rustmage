//! Real-card projected control funding interaction; link under FRA assurance.
use super::*;

#[test]
fn saga_control_returns_emrakul_granted_land_during_payment() {
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
        // Fixture funding for setup spells and the real Emrakul activation.
        g.state.players[player.index()].mana = [8; 6];
        let emrakul = put(&mut g, player, "Bear Cub", Zone::Hand);
        g.state.objects[emrakul as usize].card = Card::test_index(1324);
        act(
            &mut g,
            Action::AnnounceActivate {
                card: emrakul,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(land),
            },
        );
        pay(&mut g);
        assert_eq!(g.object_view(emrakul).zone, Zone::Exile);
        let exile_ref = g.object_ref(emrakul);
        settle(&mut g);
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
        let spell = put(&mut g, player, "Gilded Lotus", Zone::Hand);
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
            let granted = Action::TapGrantedMana {
                card: land,
                grantor: exile_ref,
                production: [0, 0, 0, 0, 0, 2],
            };
            assert!(g.legal_actions().contains(&granted));
            act(&mut g, granted);
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

#[test]
fn copied_civil_war_ability_enforces_total_mana_value_on_new_targets() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let echo = put(
            &mut g,
            player,
            "Echo, Perceptive Prodigy",
            Zone::Battlefield,
        );
        let small = put(&mut g, player.other(), "Bear Cub", Zone::Battlefield);
        let large = put(&mut g, player.other(), "Hill Giant", Zone::Battlefield);
        let other_large = put(&mut g, player.other(), "Hill Giant", Zone::Battlefield);
        let other_small = put(&mut g, player.other(), "Bear Cub", Zone::Battlefield);
        let saga = put(&mut g, player, "The Super Hero Civil War", Zone::Hand);
        cast(&mut g, saga, None);
        for _ in 0..40 {
            if matches!(g.state.pending, Pending::Targets { .. }) {
                break;
            }
            choose(&mut g, |a| {
                matches!(a, Action::Pass | Action::PutTrigger { .. })
            });
        }
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(small),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(large),
            },
        );
        act(&mut g, Action::FinishTargets);
        let original = g.state.stack.last().unwrap().id;
        act(
            &mut g,
            Action::AnnounceActivate {
                card: echo,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Stack(original),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            act(&mut g, Action::FinishTargets);
        }
        pay(&mut g);
        act(&mut g, Action::Pass);
        act(&mut g, Action::Pass);
        assert!(matches!(g.state.pending, Pending::Retarget { player: p } if p == player));
        let before = g.snapshot();
        act(
            &mut g,
            Action::SetStackTarget {
                index: 0,
                target: Target::Card(other_large),
            },
        );
        assert!(!g.legal_actions().contains(&Action::FinishStackTargets));
        reject(&mut g, Action::FinishStackTargets);
        act(
            &mut g,
            Action::SetStackTarget {
                index: 0,
                target: Target::Card(other_small),
            },
        );
        assert!(g.legal_actions().contains(&Action::FinishStackTargets));
        act(&mut g, Action::FinishStackTargets);
        assert!(g.state.stack.last().unwrap().is_copy);
        g.restore(&before).unwrap();
        assert!(matches!(g.state.pending, Pending::Retarget { player: p } if p == player));
    }
}
