//! Focused printed outcomes; shared engine invariants live in core tests.
//! Both modes and exact owned-object targeting for Get Out.
use super::big_helpers::{cast, checked, settle};
use super::*;

#[test]
fn get_out_returns_owned_creatures_even_when_opponent_controls_them() {
    {
        let player = Player::P0;
        let mut g = scenario(player);
        let spell = put(&mut g, player, "Get Out", Zone::Hand);
        let mine = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        let stolen = put(&mut g, player, "Hill Giant", Zone::Battlefield);
        g.state.objects[stolen as usize].controller = player.other();
        let theirs = put(&mut g, player.other(), "Bear Cub", Zone::Battlefield);
        let land = put(&mut g, player, "Forest", Zone::Battlefield);
        let artifact = put(&mut g, player, "Short Bow", Zone::Battlefield);
        g.state.objects[theirs as usize].controller = player;
        checked(&mut g, Action::AnnounceCast { card: spell });
        checked(&mut g, Action::ChooseMode { mode: 1 });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(theirs),
            },
        );
        for invalid in [land, artifact] {
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(invalid),
                },
            );
        }
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(mine),
            },
        );
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(mine),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(stolen),
            },
        );
        pay(&mut g);
        g.move_object(mine, Zone::Exile);
        g.move_object(mine, Zone::Battlefield);
        settle(&mut g);
        assert_eq!(g.object_view(mine).zone, Zone::Battlefield);
        assert_eq!(g.object_view(stolen).zone, Zone::Hand);
        assert_eq!(g.object_view(theirs).zone, Zone::Battlefield);
    }
}

#[test]
fn get_out_counters_a_creature_spell() {
    {
        let player = Player::P0;
        let mut g = scenario(player.other());
        let creature = put(&mut g, player.other(), "Bear Cub", Zone::Hand);
        cast(&mut g, creature, None, &[]);
        checked(&mut g, Action::Pass);
        let spell = put(&mut g, player, "Get Out", Zone::Hand);
        cast(&mut g, spell, Some(0), &[Target::Spell(creature)]);
        settle(&mut g);
        assert_eq!(g.object_view(creature).zone, Zone::Graveyard);
    }
}
