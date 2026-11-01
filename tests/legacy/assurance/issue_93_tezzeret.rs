//! Printed Tezzeret abilities through regular actions and owned replay.
use super::big_helpers::{cast as paid_cast, checked, settle as finish};
use super::*;

#[test]
fn tezzeret_cruel_captain_full_card() {
    let emblem = Card::from_name("@emblem/eoe-tezzeret-cruel-captain").unwrap();
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let walker = put(&mut g, player, "Tezzeret, Cruel Captain", Zone::Hand);
        paid_cast(&mut g, walker, None, &[]);
        finish(&mut g);
        assert_eq!(g.counter_count(walker, CounterKind::Loyalty), 4);
        let robot = put(&mut g, player, "Clockwork Percussionist", Zone::Hand);
        paid_cast(&mut g, robot, None, &[]);
        finish(&mut g);
        assert_eq!(g.counter_count(walker, CounterKind::Loyalty), 5);
        g.state.objects[robot as usize].tapped = true;
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: walker,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(robot),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            checked(&mut g, Action::FinishTargets);
        }
        pay(&mut g);
        finish(&mut g);
        assert!(!g.object_view(robot).tapped);
        assert_eq!(g.counter_count(robot, CounterKind::PlusOne), 1);

        let mut search = scenario(player);
        let walker = put(
            &mut search,
            player,
            "Tezzeret, Cruel Captain",
            Zone::Battlefield,
        );
        search.state.objects[walker as usize].counters.loyalty = 4;
        let small = put(&mut search, player, "Aerial Doombot", Zone::Library);
        let large = put(&mut search, player, "Gilded Lotus", Zone::Library);
        checked(
            &mut search,
            Action::AnnounceActivate {
                card: walker,
                ability: 1,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut search);
        search.resolve_top();
        reject(&mut search, Action::ChooseSearch { card: large });
        checked(&mut search, Action::ChooseSearch { card: small });
        checked(&mut search, Action::FinishSearch);
        finish(&mut search);
        assert_eq!(search.object_view(small).zone, Zone::Hand);
        assert_eq!(search.counter_count(walker, CounterKind::Loyalty), 1);

        let mut ultimate = scenario(player);
        let walker = put(
            &mut ultimate,
            player,
            "Tezzeret, Cruel Captain",
            Zone::Battlefield,
        );
        ultimate.state.objects[walker as usize].counters.loyalty = 7;
        let artifact = put(&mut ultimate, player, "Gilded Lotus", Zone::Battlefield);
        checked(
            &mut ultimate,
            Action::AnnounceActivate {
                card: walker,
                ability: 2,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut ultimate);
        finish(&mut ultimate);
        assert_eq!(ultimate.object_view(walker).zone, Zone::Graveyard);
        assert!(
            ultimate
                .state
                .objects
                .iter()
                .any(|o| o.card == emblem && o.zone == Zone::Command)
        );
        ultimate.begin_step(Step::BeginCombat);
        ultimate.give_priority();
        ultimate.prepare_triggers();
        checked(
            &mut ultimate,
            Action::ChooseTarget {
                target: Target::Card(artifact),
            },
        );
        if ultimate.legal_actions().contains(&Action::FinishTargets) {
            checked(&mut ultimate, Action::FinishTargets);
        }
        finish(&mut ultimate);
        assert_eq!(ultimate.counter_count(artifact, CounterKind::PlusOne), 3);
        assert!(ultimate.object_view(artifact).has_subtype("Robot"));
        assert_eq!(stats(&ultimate, artifact), (3, 3));
        ultimate.finish_cleanup();
        assert_eq!(stats(&ultimate, artifact), (3, 3));
    }
}
