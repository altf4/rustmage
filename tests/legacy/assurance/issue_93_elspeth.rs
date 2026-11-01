//! The printed abilities of Elspeth, Storm Slayer through ordinary actions.
use super::big_helpers::{cast as paid_cast, checked, settle as finish};
use super::*;

#[test]
fn elspeth_storm_slayer_full_card() {
    let soldier = Card::from_name("@token/dba7f52527d069d0ccdbfd5e").unwrap();
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let elspeth = put(&mut g, player, "Elspeth, Storm Slayer", Zone::Hand);
        paid_cast(&mut g, elspeth, None, &[]);
        finish(&mut g);
        assert_eq!(g.counter_count(elspeth, CounterKind::Loyalty), 5);
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: elspeth,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut g);
        finish(&mut g);
        assert_eq!(g.counter_count(elspeth, CounterKind::Loyalty), 6);
        let soldiers: Vec<_> = g
            .state
            .objects
            .iter()
            .enumerate()
            .filter(|(_, o)| o.card == soldier && o.zone == Zone::Battlefield)
            .map(|(id, _)| id as CardId)
            .collect();
        assert_eq!(soldiers.len(), 2);
        assert_eq!(g.object_view(soldiers[0]).colors, 1);
        assert_eq!(stats(&g, soldiers[0]), (1, 1));

        let mut flight = scenario(player);
        let elspeth = put(
            &mut flight,
            player,
            "Elspeth, Storm Slayer",
            Zone::Battlefield,
        );
        flight.state.objects[elspeth as usize].counters.loyalty = 5;
        let bear = put(&mut flight, player, "Bear Cub", Zone::Battlefield);
        let opponent = put(&mut flight, player.other(), "Bear Cub", Zone::Battlefield);
        checked(
            &mut flight,
            Action::AnnounceActivate {
                card: elspeth,
                ability: 1,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut flight);
        finish(&mut flight);
        assert_eq!(flight.counter_count(bear, CounterKind::PlusOne), 1);
        assert!(flight.keywords(bear).contains(Keywords::FLYING));
        assert!(!flight.keywords(opponent).contains(Keywords::FLYING));
        let later = put(&mut flight, player, "Bear Cub", Zone::Battlefield);
        assert!(!flight.keywords(later).contains(Keywords::FLYING));
        flight.finish_cleanup();
        flight.state.active = player.other();
        flight.begin_turn();
        assert!(flight.keywords(bear).contains(Keywords::FLYING));
        flight.finish_cleanup();
        flight.state.active = player;
        flight.begin_turn();
        assert!(!flight.keywords(bear).contains(Keywords::FLYING));

        let mut destroy = scenario(player);
        let elspeth = put(
            &mut destroy,
            player,
            "Elspeth, Storm Slayer",
            Zone::Battlefield,
        );
        destroy.state.objects[elspeth as usize].counters.loyalty = 5;
        let own = put(&mut destroy, player, "Armasaur Guide", Zone::Battlefield);
        let small = put(&mut destroy, player.other(), "Bear Cub", Zone::Battlefield);
        let large = put(
            &mut destroy,
            player.other(),
            "Armasaur Guide",
            Zone::Battlefield,
        );
        checked(
            &mut destroy,
            Action::AnnounceActivate {
                card: elspeth,
                ability: 2,
                counter_payment: Counters::default(),
            },
        );
        for card in [own, small] {
            reject(
                &mut destroy,
                Action::ChooseTarget {
                    target: Target::Card(card),
                },
            );
        }
        checked(
            &mut destroy,
            Action::ChooseTarget {
                target: Target::Card(large),
            },
        );
        if destroy.legal_actions().contains(&Action::FinishTargets) {
            checked(&mut destroy, Action::FinishTargets);
        }
        pay(&mut destroy);
        finish(&mut destroy);
        assert_eq!(destroy.object_view(large).zone, Zone::Graveyard);
        assert_eq!(destroy.counter_count(elspeth, CounterKind::Loyalty), 2);
    }
}
