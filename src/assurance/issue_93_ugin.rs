//! Ugin's printed abilities through paid actions and owned replay.
use super::big_helpers::{checked, settle as finish};
use super::*;

#[test]
fn ugin_eye_of_the_storms_full_card() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let walker = put(&mut g, player, "Ugin, Eye of the Storms", Zone::Hand);
        let victim = put(&mut g, player.other(), "Bear Cub", Zone::Battlefield);
        checked(&mut g, Action::AnnounceCast { card: walker });
        pay(&mut g);
        g.prepare_triggers();
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(victim),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            checked(&mut g, Action::FinishTargets);
        }
        finish(&mut g);
        assert_eq!(g.object_view(victim).zone, Zone::Exile);
        assert_eq!(g.counter_count(walker, CounterKind::Loyalty), 7);
        let next = put(&mut g, player.other(), "Bear Cub", Zone::Battlefield);
        let artifact = put(&mut g, player, "Adventuring Gear", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: artifact });
        pay(&mut g);
        g.prepare_triggers();
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(next),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            checked(&mut g, Action::FinishTargets);
        }
        finish(&mut g);
        assert_eq!(g.object_view(next).zone, Zone::Exile);
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: walker,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut g);
        finish(&mut g);
        assert_eq!(g.state.players[player.index()].life, 23);
        assert_eq!(g.counter_count(walker, CounterKind::Loyalty), 9);
        assert_eq!(g.hand_count(player), 1);

        let mut mana = scenario(player);
        let walker = put(
            &mut mana,
            player,
            "Ugin, Eye of the Storms",
            Zone::Battlefield,
        );
        mana.state.objects[walker as usize].counters.loyalty = 7;
        mana.state.players[player.index()].mana = [0; 6];
        checked(
            &mut mana,
            Action::AnnounceActivate {
                card: walker,
                ability: 1,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut mana);
        finish(&mut mana);
        assert_eq!(mana.state.players[player.index()].mana[5], 3);

        let mut search = scenario(player);
        let walker = put(
            &mut search,
            player,
            "Ugin, Eye of the Storms",
            Zone::Battlefield,
        );
        search.state.objects[walker as usize].counters.loyalty = 11;
        let colorless = put(&mut search, player, "Adventuring Gear", Zone::Library);
        let colored = put(&mut search, player, "Bear Cub", Zone::Library);
        let land = put(&mut search, player, "Forest", Zone::Library);
        checked(
            &mut search,
            Action::AnnounceActivate {
                card: walker,
                ability: 2,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut search);
        search.resolve_top();
        reject(&mut search, Action::ChooseSearch { card: colored });
        reject(&mut search, Action::ChooseSearch { card: land });
        checked(&mut search, Action::ChooseSearch { card: colorless });
        checked(&mut search, Action::FinishSearch);
        finish(&mut search);
        assert_eq!(search.object_view(colorless).zone, Zone::Exile);
        search.state.players[player.index()].mana = [0; 6];
        let free = search.legal_actions().into_iter().find(|action| matches!(action, Action::AnnounceExileFree { card, .. } if *card == colorless)).expect("free exile cast");
        checked(&mut search, free);
        pay(&mut search);
        finish(&mut search);
        assert_eq!(search.object_view(colorless).zone, Zone::Battlefield);
    }
}
