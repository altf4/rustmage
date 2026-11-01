use super::*;
use super::*;
pub use rustmage_core::engine::outside_choices::*;
#[cfg(test)]
mod accounting_tests {
    use super::*;

    #[test]
    fn pending_entry_counts_initialized_elements_only() {
        let mut game = Game::new_with_outside(
            Config {
                decks: [vec![Card::Forest; 60], vec![Card::Forest; 60]],
                seed: 7,
                starting_player: Player::P0,
                deck_rules: DeckRules::Freeform,
            },
            OutsideGameConfig {
                mode: OutsideGameMode::CasualCollection,
                pools: [
                    vec![Card::Forest, Card::from_name("Shock").unwrap()],
                    vec![],
                ],
            },
        )
        .unwrap();
        assert!(game.start_outside_choice(Player::P0));
        game.choose_outside_card(Selection::Pool(0));
        game.choose_outside_card(Selection::Pool(1));
        assert!(game.plan_outside_hand(0));
        let before = game.state_bytes();
        let event = game
            .state
            .outside_game
            .as_mut()
            .unwrap()
            .pending_entry
            .as_mut()
            .unwrap();
        event.choices.reserve(16);
        event.applied.reserve(16);
        event.self_applied.reserve(16);
        event.self_choices.reserve(16);
        event.provider_applied.reserve(16);
        event.provider_choices.reserve(16);
        assert_eq!(game.state_bytes(), before);
        game.state
            .outside_game
            .as_mut()
            .unwrap()
            .pending_entry
            .as_mut()
            .unwrap()
            .applied
            .push(1);
        assert_eq!(game.state_bytes(), before + std::mem::size_of::<u64>());
    }
}
