use super::*;

#[test]
fn unrelated_nonland_in_layer_six_evaluation_does_not_create_mana_or_panic() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let land = put(&mut g, player, "Forest", Zone::Battlefield);
        let creature = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        let aura = put(
            &mut g,
            player.other(),
            "Witness Protection",
            Zone::Battlefield,
        );
        let creature_ref = g.object_ref(creature);
        g.state.objects[aura as usize].attached_to = Some(creature_ref);
        assert!(ability_world::CharacteristicWorld::has_ability_removal(&g));
        assert!(!g.keywords(creature).contains(Keywords::VIGILANCE));
        assert!(g.legal_actions().contains(&Action::TapMana { card: land }));
        assert!(
            !g.legal_actions()
                .contains(&Action::TapMana { card: creature })
        );
        let before = g.snapshot();
        for replay in 0..2 {
            if replay == 1 {
                g.restore(&before).unwrap();
            }
            g.state.players[player.index()].mana = [0; 6];
            act(&mut g, Action::TapMana { card: land });
            assert_eq!(g.state.players[player.index()].mana, [0, 0, 0, 0, 1, 0]);
        }
    }
}
