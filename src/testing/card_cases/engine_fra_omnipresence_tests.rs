//! Focused printed outcomes; shared engine invariants live in core tests.
use super::*;

fn free(card: CardId) -> Action {
    Action::AnnounceFree {
        card,
        face: CastFace::Normal,
        flash_surcharge: false,
    }
}

#[test]
fn omnipresence_permission_follows_its_controller_and_leaves_with_source() {
    {
        let player = Player::P0;
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
