use super::*;

// Draft-only front face coverage: Molten Tide remains unsupported and gated.
#[test]
fn pyre_rhymer_casts_as_three_three_and_enters_prepared() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let source = put(&mut g, player, "Pyre Rhymer // Molten Tide", Zone::Hand);
        g.state.players[player.index()].mana = [0, 0, 0, 2, 0, 1];
        let saved = g.snapshot();
        for _ in 0..2 {
            act(&mut g, Action::AnnounceCast { card: source });
            pay(&mut g);
            assert_eq!(g.state.players[player.index()].mana, [0; 6]);
            settle(&mut g);
            assert_eq!(stats(&g, source), (3, 3));
            let reference = g.object_ref(source);
            assert!(g.is_prepared(reference));
            let row = g.state.preparations.as_ref().unwrap().iter()
                .find(|row| row.permanent == reference).unwrap();
            assert_eq!(g.state.objects[row.spell_copy.card as usize].card, Card::test_index(5461));
            g.restore(&saved).unwrap();
        }
    }
}

#[test]
fn pyre_rhymer_prowess_responds_only_to_own_noncreature_cast() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let source = put(&mut g, player, "Pyre Rhymer // Molten Tide", Zone::Battlefield);
        let creature = put(&mut g, player, "Bear Cub", Zone::Hand);
        cast(&mut g, creature, None);
        settle(&mut g);
        assert_eq!(stats(&g, source), (3, 3));
        let spell = put(&mut g, player, "Shock", Zone::Hand);
        cast(&mut g, spell, Some(Target::Player(player.other())));
        settle(&mut g);
        assert_eq!(stats(&g, source), (4, 4));
        g.finish_cleanup();
        assert_eq!(stats(&g, source), (3, 3));
    }
}
