//! Whole-card scenario for Ashiok and its life-payment replacement.
use super::big_helpers::{cast as paid_cast, checked, settle as finish};
use super::*;

fn put_ashiok(g: &mut Game, player: Player, zone: Zone) -> CardId {
    let id = put(g, player, "Ashiok, Wicked Manipulator", zone);
    if zone == Zone::Battlefield {
        g.state.objects[id as usize].counters.loyalty = 5;
    }
    id
}

#[test]
fn ashiok_wicked_manipulator_full_card() {
    let nightmare = Card::from_name("@token/woe-ashiok-nightmare").unwrap();
    for player in [Player::P0, Player::P1] {
        let mut loot = scenario(player);
        let ashiok = put_ashiok(&mut loot, player, Zone::Hand);
        paid_cast(&mut loot, ashiok, None, &[]);
        finish(&mut loot);
        assert_eq!(loot.counter_count(ashiok, CounterKind::Loyalty), 5);
        let chosen = put(&mut loot, player, "Shock", Zone::Library);
        let other = put(&mut loot, player, "Bear Cub", Zone::Library);
        checked(
            &mut loot,
            Action::AnnounceActivate {
                card: ashiok,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut loot);
        loot.resolve_top();
        assert!(loot.observe(player).library_selection.is_some());
        assert!(loot.observe(player.other()).library_selection.is_none());
        checked(&mut loot, Action::SelectLibrary { card: chosen });
        checked(&mut loot, Action::FinishLibrarySelection);
        finish(&mut loot);
        assert_eq!(loot.object_view(chosen).zone, Zone::Exile);
        assert_eq!(loot.object_view(other).zone, Zone::Hand);

        let mut payment = scenario(player);
        put_ashiok(&mut payment, player, Zone::Battlefield);
        let dredger = put(&mut payment, player, "Treasure Dredger", Zone::Battlefield);
        let top = payment.top_library_card(player).unwrap();
        checked(
            &mut payment,
            Action::AnnounceActivate {
                card: dredger,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut payment);
        finish(&mut payment);
        assert_eq!(payment.state.players[player.index()].life, 20);
        assert_eq!(payment.object_view(top).zone, Zone::Exile);
        let shock = put(&mut payment, player, "Shock", Zone::Hand);
        paid_cast(&mut payment, shock, None, &[Target::Player(player)]);
        finish(&mut payment);
        assert_eq!(payment.state.players[player.index()].life, 18);

        // Funding must account for the first replaced payment before checking
        // whether the enclosing ability's remaining life cost is payable.
        let mut funded = scenario(player);
        put_ashiok(&mut funded, player, Zone::Battlefield);
        let dredger = put(&mut funded, player, "Treasure Dredger", Zone::Battlefield);
        let town = put(&mut funded, player, "Starting Town", Zone::Battlefield);
        funded.state.players[player.index()].life = 1;
        funded.state.players[player.index()].mana = [0; 6];
        let top = funded.state.players[player.index()]
            .library
            .iter()
            .rev()
            .take(2)
            .copied()
            .collect::<Vec<_>>();
        checked(
            &mut funded,
            Action::AnnounceActivate {
                card: dredger,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        checked(
            &mut funded,
            Action::ActivateMana {
                card: town,
                ability: 1,
                payment: [0; 6],
                counter_payment: Counters::default(),
                production: [0, 0, 0, 0, 1, 0],
            },
        );
        pay(&mut funded);
        finish(&mut funded);
        assert_eq!(funded.state.players[player.index()].life, 1);
        assert!(
            top.iter()
                .all(|&id| funded.object_view(id).zone == Zone::Exile)
        );

        let mut tokens = scenario(player);
        let ashiok = put_ashiok(&mut tokens, player, Zone::Battlefield);
        checked(
            &mut tokens,
            Action::AnnounceActivate {
                card: ashiok,
                ability: 1,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut tokens);
        finish(&mut tokens);
        let ids: Vec<_> = tokens
            .state
            .objects
            .iter()
            .enumerate()
            .filter_map(|(id, o)| {
                (o.card == nightmare && o.zone == Zone::Battlefield).then_some(id as CardId)
            })
            .collect();
        assert_eq!(ids.len(), 2);
        let top = tokens.top_library_card(player).unwrap();
        tokens.move_object(top, Zone::Exile);
        tokens.begin_step(Step::BeginCombat);
        tokens.give_priority();
        finish(&mut tokens);
        for id in ids {
            assert_eq!(tokens.counter_count(id, CounterKind::PlusOne), 1);
        }

        let mut ultimate = scenario(player);
        let ashiok = put_ashiok(&mut ultimate, player, Zone::Battlefield);
        ultimate.state.objects[ashiok as usize].counters.loyalty = 7;
        put(&mut ultimate, player, "Bear Cub", Zone::Exile);
        put(&mut ultimate, player, "Shock", Zone::Exile);
        let before = ultimate.library_card_count(player.other());
        checked(
            &mut ultimate,
            Action::AnnounceActivate {
                card: ashiok,
                ability: 2,
                counter_payment: Counters::default(),
            },
        );
        checked(
            &mut ultimate,
            Action::ChooseTarget {
                target: Target::Player(player.other()),
            },
        );
        if ultimate.legal_actions().contains(&Action::FinishTargets) {
            checked(&mut ultimate, Action::FinishTargets);
        }
        pay(&mut ultimate);
        finish(&mut ultimate);
        assert_eq!(ultimate.library_card_count(player.other()), before - 3);
    }
}
