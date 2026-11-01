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
    for player in [Player::P0, Player::P1] {
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
        super::super::contracts::branch_contract(g.clone());
        act(&mut g, Action::FinishSearch);
        settle(&mut g);
        assert!(g.state.players[player.index()].library.is_empty());
        assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
        assert!(g.observe(player.other()).known_library_top[player.index()].is_none());
    }
}

#[test]
fn loyal_tutor_shuffles_and_tracks_public_information() {
    for player in [Player::P0, Player::P1] {
        for find in [false, true] {
            let mut g = scenario(player);
            let first = put(&mut g, player, "Ajani, Caller of the Pride", Zone::Library);
            let second = put(&mut g, player, "Ajani, Caller of the Pride", Zone::Library);
            let invalid = put(&mut g, player, "Bear Cub", Zone::Library);
            let old_top = g.object_ref(invalid);
            g.remember_library_top(player, old_top);
            let chosen_ref = g.object_ref(first);
            let spell = put(&mut g, player, "Loyal Tutor", Zone::Hand);
            search_ready(&mut g, player, spell);
            reject(&mut g, Action::ChooseSearch { card: invalid });
            assert!(
                g.legal_actions()
                    .contains(&Action::ChooseSearch { card: first })
            );
            assert!(
                g.legal_actions()
                    .contains(&Action::ChooseSearch { card: second })
            );
            if find {
                act(&mut g, Action::ChooseSearch { card: first });
                reject(&mut g, Action::ChooseSearch { card: first });
                reject(&mut g, Action::ChooseSearch { card: second });
            }
            let mut expected = g.state.players[player.index()].library.clone();
            let mut rng = g.state.rng;
            rng.shuffle(&mut expected);
            if find {
                let position = expected.iter().position(|&card| card == first).unwrap();
                expected.remove(position);
                expected.push(first);
            }
            let saved = g.snapshot();
            super::super::contracts::branch_contract(g.clone());
            for replay in 0..2 {
                if replay == 1 {
                    g.restore(&saved).unwrap();
                }
                act(&mut g, Action::FinishSearch);
                settle(&mut g);
                assert_eq!(g.state.players[player.index()].library, expected);
                assert_eq!(g.object_ref(first), chosen_ref);
                let public = g.observe(player.other()).known_library_top[player.index()];
                if find {
                    assert_eq!(public.unwrap().object, chosen_ref);
                } else {
                    assert!(public.is_none());
                }
            }
        }
    }
}
