use super::*;

#[test]
fn puppet_animates_artifacts_and_non_aura_enchantments_without_removing_types() {
    for player in [Player::P0, Player::P1] {
        for (name, kind) in [("Gideon's Memorial", 16u16), ("Omniscience", 32u16)] {
            let mut g = scenario(player);
            let target = put(&mut g, player, name, Zone::Battlefield);
            let original_color = g.object_view(target).colors;
            let aura = put(&mut g, player, "Puppet Crafting", Zone::Hand);
            cast(&mut g, aura, Some(Target::Card(target)));
            settle(&mut g);
            assert_eq!(g.object_view(aura).zone, Zone::Battlefield);
            assert_eq!(g.object_view(aura).attached_to, Some(g.object_ref(target)));
            assert_eq!(g.current_kinds(target) & (kind | 1), kind | 1);
            assert!(g.has_subtype(target, "Construct"));
            assert_eq!(stats(&g, target), (5, 5));
            assert_eq!(g.object_view(target).colors, original_color);
            let saved = g.snapshot();
            g.move_object(aura, Zone::Graveyard);
            assert!(!g.is_creature(target));
            g.restore(&saved).unwrap();
            assert_eq!(stats(&g, target), (5, 5));
            assert!(g.has_subtype(target, "Construct"));
        }
    }
}

#[test]
fn puppet_rejects_plain_creatures_and_aura_enchantments_and_returns_from_graveyard() {
    for player in [Player::P0, Player::P1] {
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
        act(&mut g, Action::AnnounceActivate { card: puppet, ability: 0, counter_payment: Counters::default() });
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(puppet).zone, Zone::Hand);
    }
}
