use super::*;

#[test]
fn forest_tentacle_mana_tracks_copied_identity_and_ability_removal() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        g.create_tokens(player, Card::test_index(5454), 1, false);
        let token = g
            .state
            .objects
            .iter()
            .enumerate()
            .find_map(|(id, object)| {
                (object.is_token()
                    && object.controller == player
                    && object.zone == Zone::Battlefield
                    && object.card == Card::test_index(5454))
                .then_some(id as CardId)
            })
            .unwrap();
        g.state.objects[token as usize].summoning_sick = false;
        let source = g.object_ref(token);
        assert!(g.has_subtype(token, "Forest"));
        assert!(g.legal_actions().contains(&Action::TapMana { card: token }));
        let saved = g.snapshot();
        let aura = put(
            &mut g,
            player.other(),
            "Witness Protection",
            Zone::Battlefield,
        );
        g.state.objects[aura as usize].attached_to = Some(source);
        assert!(!g.has_subtype(token, "Forest"));
        assert!(!g.legal_actions().contains(&Action::TapMana { card: token }));
        g.move_object(aura, Zone::Graveyard);
        assert!(g.has_subtype(token, "Forest"));
        assert!(g.legal_actions().contains(&Action::TapMana { card: token }));
        g.restore(&saved).unwrap();
        let recipe = CopyRecipe::new(Card::from_name("Bear Cub").unwrap());
        g.apply_temporary_copy(source, recipe, CopyDeadline::EndOfTurn);
        assert!(!g.has_subtype(token, "Forest"));
        assert!(!g.legal_actions().contains(&Action::TapMana { card: token }));
        g.restore(&saved).unwrap();
        g.state.players[player.index()].mana = [0; 6];
        act(&mut g, Action::TapMana { card: token });
        assert_eq!(g.state.players[player.index()].mana, [0, 0, 0, 0, 1, 0]);
    }
}
