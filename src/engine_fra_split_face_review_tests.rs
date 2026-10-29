//! Regression coverage for live split second on an inset spell.
use super::*;

#[test]
fn samut_grants_split_second_to_an_adventure_spell_on_announcement_and_stack() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let samut = put(&mut game, player, "Samut, Tyrant of Naktamun", Zone::Battlefield);
        let adventure = put(&mut game, player, "Great Ugly-Looking Goblin // Clap! Snap!", Zone::Hand);
        game.state.players[player.index()].mana = [0, 0, 1, 0, 0, 2];
        act(&mut game, Action::AnnounceAdventure { card: adventure });
        assert!(game.effective_spell_card(adventure).inset_parent().is_some());
        assert!(game.keywords(adventure).contains(Keywords::SPLIT_SECOND));
        for viewer in [player, player.other()] {
            assert!(game.observe(viewer).announcing.unwrap().split_second);
        }
        pay(&mut game);
        assert!(game.observe(player).stack.last().unwrap().split_second);
        assert!(game.object_view(adventure).keywords.contains(Keywords::SPLIT_SECOND));
        game.move_object(samut, Zone::Graveyard);
        assert!(!game.keywords(adventure).contains(Keywords::SPLIT_SECOND));
        assert!(!game.observe(player).stack.last().unwrap().split_second);
    }
}
