//! Focused printed outcomes; shared engine invariants live in core tests.
use super::*;

#[test]
fn puppet_rejects_plain_creatures_and_aura_enchantments_and_returns_from_graveyard() {
    {
        let player = Player::P0;
        let mut g = scenario(player);
        let puppet = put(&mut g, player, "Puppet Crafting", Zone::Graveyard);
        let card = g.state.objects[puppet as usize].card;
        let creature = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        let aura = put(&mut g, player, "Witness Protection", Zone::Battlefield);
        let artifact = put(&mut g, player, "Gideon's Memorial", Zone::Battlefield);
        g.state.objects[aura as usize].attached_to = Some(g.object_ref(creature));
        assert!(!g.aura_can_enchant(card, player, g.object_ref(creature)));
        assert!(!g.aura_can_enchant(card, player, g.object_ref(aura)));
        assert!(g.aura_can_enchant(card, player, g.object_ref(artifact)));
        g.state.players[player.index()].mana = [0, 0, 0, 0, 1, 4];
        act(
            &mut g,
            Action::AnnounceActivate {
                card: puppet,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(puppet).zone, Zone::Hand);
    }
}
