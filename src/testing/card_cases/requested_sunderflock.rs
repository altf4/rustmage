//! Focused printed outcomes; shared engine invariants live in core tests.
//! Sunderflock's reduction and cast-only entry effect.
use super::big_helpers::{cast, settle};
use super::*;

#[test]
fn sunderflock_uses_greatest_elemental_mana_value_and_bounces_other_creatures() {
    {
        let player = Player::P0;
        let mut g = scenario(player);
        let flock = put(&mut g, player, "Sunderflock", Zone::Hand);
        let elemental = put(&mut g, player, "Fire Elemental", Zone::Battlefield);
        let victim = put(&mut g, player.other(), "Bear Cub", Zone::Battlefield);
        g.state.players[player.index()].mana = [0, 2, 0, 0, 0, 2];
        cast(&mut g, flock, None, &[]);
        settle(&mut g);
        assert_eq!(g.object_view(flock).zone, Zone::Battlefield);

        assert_eq!(g.object_view(elemental).zone, Zone::Battlefield);
        assert_eq!(g.object_view(victim).zone, Zone::Hand);
    }
}

#[test]
fn sunderflock_does_not_bounce_when_put_onto_battlefield() {
    {
        let player = Player::P0;
        let mut g = scenario(player);
        let flock = put(&mut g, player, "Sunderflock", Zone::Graveyard);
        let victim = put(&mut g, player.other(), "Bear Cub", Zone::Battlefield);
        g.move_object(flock, Zone::Battlefield);
        settle(&mut g);
        assert_eq!(g.object_view(victim).zone, Zone::Battlefield);
    }
}
