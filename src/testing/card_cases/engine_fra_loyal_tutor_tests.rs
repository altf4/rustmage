//! Focused printed outcomes; shared engine invariants live in core tests.
use super::*;

fn search_ready(g: &mut Game, player: Player, spell: CardId) {
    cast(g, spell, None);
    for _ in 0..48 {
        if matches!(g.state.pending, Pending::EffectSearch { .. }) {
            break;
        }
        g.prepare_triggers();
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::Pass | Action::PutTrigger { .. }))
            .expect("search progression");
        act(g, action);
    }
    assert_eq!(g.actor(), Some(player));
    assert!(g.observe(player).search_choice.is_some());
    assert!(g.observe(player.other()).search_choice.is_none());
}

#[test]
fn loyal_tutor_empty_library_search_finishes() {
    {
        let player = Player::P0;
        let mut g = scenario(player);
        let cards = g.state.players[player.index()].library.clone();
        for card in cards {
            g.move_object(card, Zone::Exile);
        }
        assert!(g.state.players[player.index()].library.is_empty());
        let spell = put(&mut g, player, "Loyal Tutor", Zone::Hand);
        search_ready(&mut g, player, spell);
        assert!(g.legal_actions().contains(&Action::FinishSearch));
        assert!(
            !g.legal_actions()
                .iter()
                .any(|a| matches!(a, Action::ChooseSearch { .. }))
        );

        act(&mut g, Action::FinishSearch);
        settle(&mut g);
        assert!(g.state.players[player.index()].library.is_empty());
        assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
        assert!(g.observe(player.other()).known_library_top[player.index()].is_none());
    }
}
