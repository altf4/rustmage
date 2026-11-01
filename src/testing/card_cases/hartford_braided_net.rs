//! Focused printed outcomes; shared engine invariants live in core tests.
//! Complete paid Braided Net and Braided Quipu interactions.
use super::big_helpers::{cast, checked, settle};
use super::hartford_craft_core::{begin_craft, craft_mana, craft_material, craft_source};
use super::*;

#[test]
fn net_enters_with_counters_and_locks_activated_abilities_until_untap() {
    use crate::engine::ability_world::CharacteristicWorld;
    {
        let player = Player::P0;
        let mut g = scenario(player);
        let source = put(&mut g, player, "Braided Net", Zone::Hand);
        let target = put(&mut g, player.other(), "Short Bow", Zone::Battlefield);
        let land = put(&mut g, player.other(), "Forest", Zone::Battlefield);
        cast(&mut g, source, None, &[]);
        settle(&mut g);
        assert_eq!(g.counter_count(source, CounterKind::Net), 3);
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: source,
                ability: 0,
                counter_payment: Counters::of(CounterKind::Net, 1),
            },
        );
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(source),
            },
        );
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(land),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        pay(&mut g);
        assert_eq!(g.counter_count(source, CounterKind::Net), 2);
        assert!(g.object_view(source).tapped);
        g.move_object(source, Zone::Graveyard);
        settle(&mut g);
        assert!(g.object_view(target).tapped);
        assert!(CharacteristicWorld::activation_prohibited(
            &g,
            player.other(),
            target
        ));
        g.untap_objects(&[target], untap_rules::UntapCause::Effect);
        g.tap_objects(&[target]);
        assert!(!CharacteristicWorld::activation_prohibited(
            &g,
            player.other(),
            target
        ));
    }
}

#[test]
fn quipu_draws_for_current_artifacts_then_moves_to_owners_library_third() {
    {
        let player = Player::P0;
        let mut g = scenario(player);
        let source = put(&mut g, player.other(), "Braided Net", Zone::Battlefield);
        g.state.objects[source as usize].controller = player;
        let material = put(&mut g, player, "Short Bow", Zone::Graveyard);
        begin_craft(&mut g, source, 1);
        craft_material(&mut g, material);
        craft_source(&mut g);
        craft_mana(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(source).controller, player.other());
        assert_eq!(g.object_view(source).card.name(), "Braided Quipu");
        g.state.objects[source as usize].controller = player;
        put(&mut g, player, "Short Bow", Zone::Battlefield);
        put(&mut g, player, "Short Bow", Zone::Battlefield);
        let hand = g.hand_count(player);
        let owner = player.other();
        let top: Vec<_> = g.state.players[owner.index()]
            .library
            .iter()
            .rev()
            .take(2)
            .copied()
            .collect();
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: source,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.hand_count(player), hand + 3);
        assert_eq!(g.object_view(source).zone, Zone::Library);
        let lib = &g.state.players[owner.index()].library;
        assert_eq!(lib[lib.len() - 3], source);
        assert_eq!(lib.iter().rev().take(2).copied().collect::<Vec<_>>(), top);
    }
}
