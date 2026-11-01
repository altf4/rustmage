//! Focused printed outcomes; shared engine invariants live in core tests.
//! Unlinked tests for the isolated outside-game pool foundation.
use super::*;

fn config(rules: DeckRules) -> Config {
    Config {
        decks: [vec![Card::Forest; 60], vec![Card::Forest; 60]],
        seed: 7,
        starting_player: Player::P0,
        deck_rules: rules,
    }
}
fn pool(mode: OutsideGameMode, a: Vec<Card>, b: Vec<Card>) -> OutsideGameConfig {
    OutsideGameConfig {
        mode,
        pools: [a, b],
    }
}
#[test]
fn state_bytes_counts_initialized_outside_and_combat_rows() {
    let mut g = Game::new_with_outside(
        config(DeckRules::Freeform),
        pool(
            OutsideGameMode::CasualCollection,
            vec![Card::Forest],
            vec![],
        ),
    )
    .unwrap();
    let before = g.state_bytes();
    let outside = g.state.outside_game.as_mut().unwrap();
    outside.pools[0].reserve(64);
    outside.known[0].reserve(64);
    assert_eq!(g.state_bytes(), before);
    g.state.outside_game.as_mut().unwrap().known[0].push(0);
    assert_eq!(g.state_bytes(), before + std::mem::size_of::<u16>());

    g.state.combat_defender_types = Some(Vec::new().into());
    let empty = g.state_bytes();
    g.state.combat_defender_types = Some(vec![(g.object_ref(0), 1)].into());
    assert_eq!(
        g.state_bytes(),
        empty + std::mem::size_of::<(ObjectRef, u16)>()
    );
}
#[test]
fn outside_constructor_validates_combined_limits_and_preserves_old_constructor() {
    let shock = Card::from_name("Shock").unwrap();
    assert!(Game::new(config(DeckRules::Constructed)).is_ok());
    let mut cfg = config(DeckRules::Constructed);
    cfg.decks[0].extend([shock; 4]);
    assert!(
        Game::new_with_outside(
            cfg.clone(),
            pool(OutsideGameMode::ConstructedSideboard, vec![shock], vec![])
        )
        .is_err()
    );
    cfg.decks[0].pop();
    let g = Game::new_with_outside(
        cfg,
        pool(OutsideGameMode::ConstructedSideboard, vec![shock], vec![]),
    )
    .unwrap();
    assert_eq!(g.observe(Player::P1).sideboard_counts, Some([1, 0]));
    assert!(g.observe(Player::P1).owned_outside.is_none());
    assert!(
        Game::new_with_outside(
            config(DeckRules::Constructed),
            pool(
                OutsideGameMode::ConstructedSideboard,
                vec![Card::Forest; 16],
                vec![]
            )
        )
        .is_err()
    );
    assert!(
        Game::new_with_outside(
            config(DeckRules::Freeform),
            pool(OutsideGameMode::ConstructedSideboard, vec![], vec![])
        )
        .is_err()
    );
    assert!(
        Game::new_with_outside(
            config(DeckRules::Freeform),
            pool(
                OutsideGameMode::CasualCollection,
                vec![Card::Forest; 16],
                vec![]
            )
        )
        .is_ok()
    );
}

fn outside_scenario(player: Player, cards: Vec<Card>) -> Game {
    let mut g = scenario(player);
    let mut pools = [vec![], vec![]];
    pools[player.index()] = cards;
    g.state.outside_game = Some(Box::new(OutsideGameState {
        mode: OutsideGameMode::CasualCollection,
        pools,
        choice: None,
        pending_entry: None,
        pool_versions: [0; 2],
        next_choice_id: 0,
        known: [Vec::new(), Vec::new()],
    }));
    g
}

#[test]
fn outside_empty_or_duplicate_only_pool_never_offers_an_incomplete_pair() {
    {
        let player = Player::P0;
        for cards in [vec![], vec![Card::Forest], vec![Card::Forest, Card::Forest]] {
            let mut g = outside_scenario(player, cards.clone());
            let source = put(&mut g, player, "Shock", Zone::Hand);
            g.state.objects[source as usize].card =
                Card::from_name("Extrapolate the Impossible").unwrap();
            act(&mut g, Action::AnnounceCast { card: source });
            pay(&mut g);
            for _ in 0..48 {
                if matches!(g.state.pending, Pending::OutsideGameChoice { .. }) {
                    break;
                }
                act(&mut g, Action::Pass);
            }
            assert!(
                matches!(g.state.pending, Pending::OutsideGameChoice { player: p } if p == player)
            );
            assert!(!g.legal_actions().iter().any(|a| matches!(
                a,
                Action::ChooseOutsideCard { .. } | Action::ChooseOutsideBasic { .. }
            )));
            assert!(
                g.observe(player.other())
                    .outside_choice
                    .as_ref()
                    .unwrap()
                    .pool
                    .is_none()
            );
            act(&mut g, Action::DeclineOutsideCards);
            settle(&mut g);
            assert!(g.state.outside_game.as_ref().unwrap().choice.is_none());
            assert_eq!(
                g.state.outside_game.as_ref().unwrap().pools[player.index()],
                cards
            );
        }
    }
}
