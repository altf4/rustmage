use super::*;

fn free(card: CardId) -> Action {
    Action::AnnounceFree { card, face: CastFace::Normal, flash_surcharge: false }
}

#[test]
fn omnipresence_uses_current_controller_creature_count_and_spell_mana_value() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let source = put(&mut g, player, "Omnipresence", Zone::Battlefield);
        let one = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        let other = put(&mut g, player.other(), "Bear Cub", Zone::Battlefield);
        let spell = put(&mut g, player, "Bear Cub", Zone::Hand);
        g.state.players[player.index()].mana = [0; 6];
        assert!(!g.legal_actions().contains(&free(spell)));
        let reference = g.object_ref(other);
        g.change_controller(reference, player);
        assert!(g.legal_actions().contains(&free(spell)));
        let before = g.snapshot();
        g.move_object(one, Zone::Graveyard);
        assert!(!g.legal_actions().contains(&free(spell)));
        g.restore(&before).unwrap();
        assert!(g.legal_actions().contains(&free(spell)));
        g.change_controller(reference, player.other());
        assert!(!g.legal_actions().contains(&free(spell)));
        g.change_controller(reference, player);
        act(&mut g, free(spell));
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(spell).zone, Zone::Battlefield);
        assert_eq!(g.state.players[player.index()].mana, [0; 6]);
        g.move_object(source, Zone::Graveyard);
    }
}

#[test]
fn omnipresence_permission_follows_its_controller_and_leaves_with_source() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let source = put(&mut g, player, "Omnipresence", Zone::Battlefield);
        put(&mut g, player, "Bear Cub", Zone::Battlefield);
        put(&mut g, player, "Campus Crier", Zone::Battlefield);
        let spell = put(&mut g, player, "Bear Cub", Zone::Hand);
        g.state.players[player.index()].mana = [0; 6];
        assert!(g.legal_actions().contains(&free(spell)));
        let reference = g.object_ref(source);
        g.change_controller(reference, player.other());
        assert!(!g.legal_actions().contains(&free(spell)));
        g.change_controller(reference, player);
        assert!(g.legal_actions().contains(&free(spell)));
        g.move_object(source, Zone::Graveyard);
        assert!(!g.legal_actions().contains(&free(spell)));
    }
}
