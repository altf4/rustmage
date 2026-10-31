//! Whole-card coverage for Ajani using normal actions and owned replay.
use super::big_helpers::{cast as paid_cast, checked, settle as finish};
use super::*;

#[test]
fn ajani_outland_chaperone_full_card() {
    let token = Card::from_name("@token/c48e9e904360e4cee1deeed8").unwrap();
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let ajani = put(&mut g, player, "Ajani, Outland Chaperone", Zone::Hand);
        paid_cast(&mut g, ajani, None, &[]);
        finish(&mut g);
        assert_eq!(g.counter_count(ajani, CounterKind::Loyalty), 3);
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: ajani,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut g);
        finish(&mut g);
        assert_eq!(g.counter_count(ajani, CounterKind::Loyalty), 4);
        let kithkin = g
            .state
            .objects
            .iter()
            .position(|o| o.card == token && o.zone == Zone::Battlefield)
            .unwrap() as CardId;
        assert_eq!(g.object_view(kithkin).colors, 17);
        assert_eq!(stats(&g, kithkin), (1, 1));

        let mut damage = scenario(player);
        let ajani = put(
            &mut damage,
            player,
            "Ajani, Outland Chaperone",
            Zone::Battlefield,
        );
        damage.state.objects[ajani as usize].counters.loyalty = 3;
        let tapped = put(&mut damage, player.other(), "Bear Cub", Zone::Battlefield);
        let untapped = put(&mut damage, player.other(), "Bear Cub", Zone::Battlefield);
        damage.state.objects[tapped as usize].tapped = true;
        checked(
            &mut damage,
            Action::AnnounceActivate {
                card: ajani,
                ability: 1,
                counter_payment: Counters::default(),
            },
        );
        reject(
            &mut damage,
            Action::ChooseTarget {
                target: Target::Card(untapped),
            },
        );
        checked(
            &mut damage,
            Action::ChooseTarget {
                target: Target::Card(tapped),
            },
        );
        if damage.legal_actions().contains(&Action::FinishTargets) {
            checked(&mut damage, Action::FinishTargets);
        }
        pay(&mut damage);
        finish(&mut damage);
        assert_eq!(damage.object_view(tapped).zone, Zone::Graveyard);
        assert_eq!(damage.counter_count(ajani, CounterKind::Loyalty), 1);

        let mut ultimate = scenario(player);
        let ajani = put(
            &mut ultimate,
            player,
            "Ajani, Outland Chaperone",
            Zone::Battlefield,
        );
        ultimate.state.objects[ajani as usize].counters.loyalty = 8;
        ultimate.state.players[player.index()].life = 5;
        let bear = put(&mut ultimate, player, "Bear Cub", Zone::Library);
        let expensive = put(&mut ultimate, player, "Gilded Lotus", Zone::Library);
        let instant = put(&mut ultimate, player, "Shock", Zone::Library);
        let elf = put(&mut ultimate, player, "Llanowar Elves", Zone::Library);
        let land = put(&mut ultimate, player, "Forest", Zone::Library);
        checked(
            &mut ultimate,
            Action::AnnounceActivate {
                card: ajani,
                ability: 2,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut ultimate);
        ultimate.resolve_top();
        assert_eq!(ultimate.object_view(ajani).zone, Zone::Graveyard);
        assert!(ultimate.observe(player).library_selection.is_some());
        assert!(ultimate.observe(player.other()).library_selection.is_none());
        for card in [expensive, instant, land] {
            reject(&mut ultimate, Action::SelectLibrary { card });
        }
        checked(&mut ultimate, Action::SelectLibrary { card: bear });
        checked(&mut ultimate, Action::SelectLibrary { card: elf });
        checked(&mut ultimate, Action::FinishLibrarySelection);
        finish(&mut ultimate);
        assert_eq!(ultimate.object_view(bear).zone, Zone::Battlefield);
        assert_eq!(ultimate.object_view(elf).zone, Zone::Battlefield);
        for card in [expensive, instant, land] {
            assert_eq!(ultimate.object_view(card).zone, Zone::Library);
        }
    }
}
