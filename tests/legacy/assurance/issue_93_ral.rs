//! Real planeswalker coverage using the ordinary action and replay paths.
use super::big_helpers::{cast as paid_cast, checked, settle as finish};
use super::*;

#[test]
fn ral_crackling_wit_full_card_and_emblem() {
    let otter_card = Card::from_name("@token/blb-ral-otter").unwrap();
    let emblem_card = Card::from_name("@emblem/blb-ral-crackling-wit").unwrap();
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let ral = put(&mut g, player, "Ral, Crackling Wit", Zone::Hand);
        paid_cast(&mut g, ral, None, &[]);
        finish(&mut g);
        assert_eq!(g.counter_count(ral, CounterKind::Loyalty), 4);
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: ral,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut g);
        finish(&mut g);
        assert_eq!(g.counter_count(ral, CounterKind::Loyalty), 5);
        let otter = g
            .state
            .objects
            .iter()
            .position(|o| o.card == otter_card && o.zone == Zone::Battlefield)
            .unwrap() as CardId;
        assert_eq!(g.object_view(otter).colors, 10);
        assert_eq!(stats(&g, otter), (1, 1));
        let creature = put(&mut g, player, "Bear Cub", Zone::Hand);
        paid_cast(&mut g, creature, None, &[]);
        finish(&mut g);
        assert_eq!(g.counter_count(ral, CounterKind::Loyalty), 5);
        assert_eq!(stats(&g, otter), (1, 1));
        let shock = put(&mut g, player, "Shock", Zone::Hand);
        paid_cast(&mut g, shock, None, &[Target::Player(player.other())]);
        finish(&mut g);
        assert_eq!(g.counter_count(ral, CounterKind::Loyalty), 6);
        assert_eq!(stats(&g, otter), (2, 2));
        g.state.priority = player.other();
        let opposing_shock = put(&mut g, player.other(), "Shock", Zone::Hand);
        paid_cast(&mut g, opposing_shock, None, &[Target::Player(player)]);
        finish(&mut g);
        assert_eq!(g.counter_count(ral, CounterKind::Loyalty), 6);
        assert_eq!(stats(&g, otter), (2, 2));
        g.finish_cleanup();
        assert_eq!(stats(&g, otter), (1, 1));

        let mut loot = scenario(player);
        let ral = put(&mut loot, player, "Ral, Crackling Wit", Zone::Battlefield);
        loot.state.objects[ral as usize].counters.loyalty = 3;
        checked(
            &mut loot,
            Action::AnnounceActivate {
                card: ral,
                ability: 1,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut loot);
        finish(&mut loot);
        assert_eq!(loot.object_view(ral).zone, Zone::Graveyard);
        assert_eq!(
            loot.state
                .objects
                .iter()
                .filter(|o| o.owner == player && o.zone == Zone::Hand)
                .count(),
            1
        );

        let mut ultimate = scenario(player);
        let ral = put(
            &mut ultimate,
            player,
            "Ral, Crackling Wit",
            Zone::Battlefield,
        );
        ultimate.state.objects[ral as usize].counters.loyalty = 10;
        checked(
            &mut ultimate,
            Action::AnnounceActivate {
                card: ral,
                ability: 2,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut ultimate);
        finish(&mut ultimate);
        assert_eq!(ultimate.object_view(ral).zone, Zone::Graveyard);
        let emblem = ultimate
            .state
            .objects
            .iter()
            .position(|o| {
                o.card == emblem_card && o.zone == Zone::Command && o.controller == player
            })
            .unwrap() as CardId;
        assert!(!ultimate.object_view(emblem).entered_this_turn);
        ultimate.state.players[player.index()].spells_cast_this_turn = 1;
        ultimate.state.players[player.other().index()].spells_cast_this_turn = 2;
        let shock = put(&mut ultimate, player, "Shock", Zone::Hand);
        paid_cast(
            &mut ultimate,
            shock,
            None,
            &[Target::Player(player.other())],
        );
        let storm = ultimate
            .state
            .stack
            .iter()
            .chain(&ultimate.state.pending_triggers)
            .chain(&ultimate.state.ordering_triggers)
            .find(|item| item.copy_event.is_some() && item.trigger_definition() == emblem_card)
            .expect("emblem grants storm");
        assert_eq!(storm.copy_event.as_ref().unwrap().amount, 3);
        finish(&mut ultimate);
        assert_eq!(ultimate.state.players[player.other().index()].life, 12);
        ultimate.create_emblem(player, emblem_card);
        ultimate.state.players[player.other().index()].life = 100;
        let second = put(&mut ultimate, player, "Shock", Zone::Hand);
        paid_cast(
            &mut ultimate,
            second,
            None,
            &[Target::Player(player.other())],
        );
        let triggers = ultimate
            .state
            .stack
            .iter()
            .chain(&ultimate.state.pending_triggers)
            .chain(&ultimate.state.ordering_triggers)
            .filter(|item| item.copy_event.is_some() && item.trigger_definition() == emblem_card)
            .count();
        assert_eq!(triggers, 2);
        finish(&mut ultimate);
        assert_eq!(ultimate.state.players[player.other().index()].life, 82);
    }
}
