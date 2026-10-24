//! Paid interactions between the independently authored MSH prerequisites.
use super::assurance::big_helpers::{cast, checked, named, settle, setup};
use super::*;
fn front(g: &mut Game, p: Player, name: &str) -> CardId {
    let id = named(g, p, name, Zone::Hand);
    cast(g, id, None, &[]);
    settle(g);
    id
}
#[test]
fn msh_combined_photon_counter_batches_ant_once_and_kid_history_compose_with_coulson() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let kid = front(&mut g, p, "Kid Loki");
        let ant = front(&mut g, p, "Ant-Man, Colony Commander");
        let coulson = front(&mut g, p, "Agent Phil Coulson");
        let photon = named(&mut g, p, "Monica Rambeau", Zone::Hand);
        checked(&mut g, Action::AnnounceBack { card: photon });
        while g.state.announcing.is_some() {
            let action = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::FinishTargets | Action::PayCosts { .. }))
                .unwrap();
            checked(&mut g, action);
        }
        settle(&mut g);
        front(&mut g, p, "Opt");
        let insects = |g: &Game| {
            g.state
                .objects
                .iter()
                .enumerate()
                .filter(|(_, o)| {
                    o.zone == Zone::Battlefield
                        && o.controller == p
                        && o.is_token()
                        && o.card.identity() == "@token/3512fba4bfa3f3fcf0f1eb9a"
                })
                .map(|(id, _)| id as CardId)
                .collect::<Vec<_>>()
        };
        let token = insects(&g);
        assert_eq!(token.len(), 1);
        let insect = token[0];
        for id in [kid, ant, coulson] {
            assert_eq!(g.counter_count(id, CounterKind::PlusOne), 1);
            assert!(g.keywords(id).contains(Keywords::HEXPROOF));
        }
        assert_eq!(g.counter_count(insect, CounterKind::PlusOne), 0);
        assert!(!g.keywords(insect).contains(Keywords::HEXPROOF));
        front(&mut g, p, "Opt");
        assert_eq!(insects(&g), vec![insect]);
        assert_eq!(g.counter_count(kid, CounterKind::PlusOne), 3);
        assert_eq!(g.counter_count(ant, CounterKind::PlusOne), 2);
        assert_eq!(g.counter_count(coulson, CounterKind::PlusOne), 2);
        assert_eq!(g.counter_count(insect, CounterKind::PlusOne), 1);
        assert!(g.keywords(insect).contains(Keywords::HEXPROOF));
        assert_eq!(g.counter_count(photon, CounterKind::PlusOne), 0);
        g.state.objects[coulson as usize].summoning_sick = false;
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: coulson,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        while g.state.announcing.is_some() {
            let action = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::PayCosts { .. }))
                .unwrap();
            checked(&mut g, action);
        }
        settle(&mut g);
        assert_eq!(g.counter_count(kid, CounterKind::PlusOne), 4);
        assert_eq!(g.counter_count(ant, CounterKind::PlusOne), 3);
        assert_eq!(g.counter_count(photon, CounterKind::PlusOne), 1);
        assert_eq!(g.counter_count(coulson, CounterKind::PlusOne), 2);
        assert_eq!(g.counter_count(insect, CounterKind::PlusOne), 1);
        assert_eq!(insects(&g), vec![insect]);
    }
}
