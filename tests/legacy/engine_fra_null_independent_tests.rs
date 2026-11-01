//! Independent Null Summoner interaction tests; link under FRA assurance.
use super::*;

fn null_trigger(g: &mut Game, player: Player) -> CardId {
    let source = put(g, player, "Bear Cub", Zone::Hand);
    g.state.objects[source as usize].card = Card::test_index(2976);
    act(g, Action::AnnounceCast { card: source });
    pay(g);
    for _ in 0..60 {
        if g.legal_actions().contains(&Action::ChooseTarget {
            target: Target::Player(player.other()),
        }) {
            act(
                g,
                Action::ChooseTarget {
                    target: Target::Player(player.other()),
                },
            );
            if g.legal_actions().contains(&Action::FinishTargets) {
                act(g, Action::FinishTargets);
            }
            return source;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("Null trigger target not reached: {:?}", g.state.pending);
}

fn hand_choice(g: &mut Game) {
    for _ in 0..30 {
        if matches!(g.state.pending, Pending::HandCardChoice { .. }) {
            return;
        }
        act(g, Action::Pass);
    }
    panic!("hand choice not reached: {:?}", g.state.pending);
}

#[test]
fn null_legal_echo_copy_exiles_two_cards_under_same_linked_ability() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let echo = put(
            &mut g,
            player,
            "Echo, Perceptive Prodigy",
            Zone::Battlefield,
        );
        let first = put(&mut g, player.other(), "Spectral Sailor", Zone::Hand);
        let second = put(&mut g, player.other(), "Bear Cub", Zone::Hand);
        null_trigger(&mut g, player);
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
        let saved = g.snapshot();
        for _ in 0..2 {
            act(&mut g, Action::FinishStackTargets);
            hand_choice(&mut g);
            let chosen = g.object_ref(first);
            act(
                &mut g,
                Action::ChooseHandCard {
                    object: Some(chosen),
                },
            );
            hand_choice(&mut g);
            let chosen = g.object_ref(second);
            act(
                &mut g,
                Action::ChooseHandCard {
                    object: Some(chosen),
                },
            );
            assert_eq!(g.object_view(first).zone, Zone::Exile);
            assert_eq!(g.object_view(second).zone, Zone::Exile);
            for _ in 0..7 {
                put(&mut g, player, "Forest", Zone::Graveyard);
            }
            assert!(g.can_play_exiled(player, first));
            assert!(g.can_play_exiled(player, second));
            settle(&mut g);
            // Neither spell's printed colored symbol is red. Each copied-trigger
            // link must separately grant the spending permission.
            for (card, red) in [(first, 1), (second, 2)] {
                g.state.players[player.index()].mana = [0, 0, 0, red, 0, 0];
                act(&mut g, Action::AnnounceExile { card });
                pay(&mut g);
                assert_eq!(g.state.players[player.index()].mana, [0; 6]);
                assert_eq!(g.object_view(card).zone, Zone::Stack);
                settle(&mut g);
                assert_eq!(g.object_view(card).zone, Zone::Battlefield);
            }
            g.restore(&saved).unwrap();
        }
    }
}

#[test]
fn null_target_becomes_illegal_before_trigger_resolves() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let card = put(&mut g, player.other(), "Spectral Sailor", Zone::Hand);
        null_trigger(&mut g, player);
        g.state.players[player.other().index()].hexproof_until_end_turn = true;
        let saved = g.snapshot();
        for _ in 0..2 {
            act(&mut g, Action::Pass);
            act(&mut g, Action::Pass);
            assert!(!matches!(g.state.pending, Pending::HandCardChoice { .. }));
            assert_eq!(g.object_view(card).zone, Zone::Hand);
            assert!(!g.observe(player).known_hand.contains(&g.object_ref(card)));
            g.restore(&saved).unwrap();
        }
    }
}

#[test]
fn null_exiled_incarnation_does_not_rebind_after_leaving_and_returning() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let card = put(&mut g, player.other(), "Spectral Sailor", Zone::Hand);
        null_trigger(&mut g, player);
        hand_choice(&mut g);
        let chosen = g.object_ref(card);
        act(
            &mut g,
            Action::ChooseHandCard {
                object: Some(chosen),
            },
        );
        for _ in 0..7 {
            put(&mut g, player, "Forest", Zone::Graveyard);
        }
        assert!(g.can_play_exiled(player, card));
        let saved = g.snapshot();
        for _ in 0..2 {
            let old = g.object_ref(card);
            g.move_object(card, Zone::Graveyard);
            g.move_object(card, Zone::Exile);
            assert_ne!(g.object_ref(card), old);
            assert!(!g.can_play_exiled(player, card));
            g.restore(&saved).unwrap();
            assert!(g.can_play_exiled(player, card));
        }
    }
}
