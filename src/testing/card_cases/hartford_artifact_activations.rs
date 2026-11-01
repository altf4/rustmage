//! Focused printed outcomes; shared engine invariants live in core tests.
//! Paid artifact activations and station's live counter threshold.
use super::big_helpers::{cast, checked, settle, setup};
use super::*;

#[test]
fn gene_pollinator_pays_both_taps_for_colored_mana() {
    {
        let player = Player::P0;
        let mut g = setup(player);
        g.state.players[player.index()].mana = [0; 6];
        let gene = put(&mut g, player, "Gene Pollinator", Zone::Battlefield);
        let other = put(&mut g, player, "Forest", Zone::Battlefield);
        checked(
            &mut g,
            Action::AnnounceMana {
                production: [0, 1, 0, 0, 0, 0],
                card: gene,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        let object = g.object_ref(other);
        checked(&mut g, Action::ChooseAdditionalCost { object });
        pay(&mut g);
        settle(&mut g);
        assert!(g.object_view(gene).tapped && g.object_view(other).tapped);
        assert_eq!(g.state.players[player.index()].mana, [0, 1, 0, 0, 0, 0]);
        assert!(g.state.stack.is_empty());
    }
}

#[test]
fn warmaker_damages_then_stations_using_current_power_on_both_turns() {
    {
        let player = Player::P0;
        let mut g = setup(player);
        put(&mut g, player, "Gene Pollinator", Zone::Battlefield);
        put(&mut g, player.other(), "Mazemind Tome", Zone::Battlefield);
        let target = put(&mut g, player.other(), "Fire Elemental", Zone::Battlefield);
        let ship = put(&mut g, player, "Warmaker Gunship", Zone::Hand);
        cast(&mut g, ship, None, &[]);
        settle(&mut g);
        assert_eq!(g.object_view(target).zone, Zone::Battlefield);
        assert_eq!(g.object_view(target).damage, 2);
        assert!(!g.is_creature(ship));
        let creature = put(&mut g, player, "Fire Elemental", Zone::Battlefield);
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: ship,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        let object = g.object_ref(creature);
        checked(&mut g, Action::ChooseAdditionalCost { object });
        pay(&mut g);
        // Station reads the creature's power when the ability resolves.
        g.add_counter(creature, CounterKind::PlusOne, 1);
        settle(&mut g);
        assert!(g.object_view(creature).tapped);
        assert_eq!(g.counter_count(ship, CounterKind::Charge), 6);
        assert!(g.is_creature(ship));
        assert_eq!(
            g.stats(ship),
            Stats {
                power: 4,
                toughness: 3
            }
        );
        assert!(g.keywords(ship).contains(Keywords::FLYING));
        g.state.active = player.other();
        assert!(g.is_creature(ship));
        assert!(g.keywords(ship).contains(Keywords::FLYING));
    }
}
