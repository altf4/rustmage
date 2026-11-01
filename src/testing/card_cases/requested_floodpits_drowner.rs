//! Focused printed outcomes; shared engine invariants live in core tests.
//! Floodpits Drowner's entry trigger and shuffle activation.
use super::big_helpers::{cast, checked, settle};
use super::*;

#[test]
fn floodpits_taps_and_stuns_then_shuffles_each_owner() {
    {
        let player = Player::P0;
        let mut g = scenario(player);
        let source = put(&mut g, player, "Floodpits Drowner", Zone::Hand);
        let target = put(&mut g, player.other(), "Bear Cub", Zone::Battlefield);
        cast(&mut g, source, None, &[]);
        settle(&mut g);

        assert!(g.object_view(target).tapped);
        assert_eq!(g.counter_count(target, CounterKind::Stun), 1);
        g.state.objects[source as usize].summoning_sick = false;
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: source,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Library);
        assert_eq!(g.object_view(target).zone, Zone::Library);
    }
}
