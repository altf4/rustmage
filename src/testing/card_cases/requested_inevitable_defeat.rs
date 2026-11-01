//! Focused printed outcomes; shared engine invariants live in core tests.
//! Targeted exile preserves the target controller and gains life independently.
use super::big_helpers::{cast, checked, settle};
use super::*;

#[test]
fn inevitable_defeat_exiles_and_uses_the_controllers_not_owners() {
    {
        let player = Player::P0;
        let mut g = scenario(player);
        let spell = put(&mut g, player, "Inevitable Defeat", Zone::Hand);
        let victim = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        g.state.objects[victim as usize].controller = player.other();
        let own_life = g.state.players[player.index()].life;
        let enemy_life = g.state.players[player.other().index()].life;
        cast(&mut g, spell, None, &[Target::Card(victim)]);
        settle(&mut g);
        assert_eq!(g.object_view(victim).zone, Zone::Exile);
        assert_eq!(g.state.players[player.index()].life, own_life + 3);
        assert_eq!(g.state.players[player.other().index()].life, enemy_life - 3);
    }
}

#[test]
fn inevitable_defeat_resolves_through_counterspell() {
    {
        let player = Player::P0;
        let mut g = scenario(player);
        let spell = put(&mut g, player, "Inevitable Defeat", Zone::Hand);
        let victim = put(&mut g, player.other(), "Bear Cub", Zone::Battlefield);
        cast(&mut g, spell, None, &[Target::Card(victim)]);
        checked(&mut g, Action::Pass);
        let counter = put(&mut g, player.other(), "Cancel", Zone::Hand);
        cast(&mut g, counter, None, &[Target::Spell(spell)]);
        settle(&mut g);
        assert_eq!(g.object_view(victim).zone, Zone::Exile);
    }
}
