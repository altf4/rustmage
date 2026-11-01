use super::*;

#[test]
fn revealed_discard_empty_or_ineligible_hand_and_target_invalidation() {
    for p in [Player::P0, Player::P1] {
        for name in ["Solve for Disappointment", "Stinging Vitriol"] {
            for case in 0..3 {
                let mut g = scenario(p);
                let hand = (case != 0).then(|| put(&mut g, p.other(), "Forest", Zone::Hand));
                let spell = put(&mut g, p, name, Zone::Hand);
                let life = g.state.players[p.other().index()].life;
                if let Some(card) = hand {
                    assert!(!g.observe(p).objects.iter().any(|o| o.id == card));
                }
                cast(&mut g, spell, Some(Target::Player(p.other())));
                if case == 2 {
                    g.state.players[p.other().index()].hexproof_until_end_turn = true;
                }
                let before = g.snapshot();
                super::super::contracts::branch_contract(g.clone());
                for replay in 0..2 {
                    if replay == 1 {
                        g.restore(&before).unwrap();
                    }
                    settle(&mut g);
                    assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
                    if let Some(card) = hand {
                        assert_eq!(g.object_view(card).zone, Zone::Hand);
                        assert_eq!(g.observe(p).objects.iter().any(|o| o.id == card), case != 2);
                    }
                    let damage = u32::from(name == "Stinging Vitriol" && case != 2);
                    assert_eq!(
                        g.state.players[p.other().index()].life,
                        life - i128::from(2 * damage)
                    );
                    let jaces = fra_jaces(&g, p);
                    let empower = name == "Solve for Disappointment" && case != 2;
                    assert_eq!(jaces.len(), usize::from(empower));
                    if empower {
                        assert_eq!(g.counter_count(jaces[0], CounterKind::Loyalty), 1);
                    }
                }
            }
        }
    }
}

#[test]
fn revealed_discard_opponent_replacement_enters_before_next_effect() {
    for p in [Player::P0, Player::P1] {
        for name in ["Solve for Disappointment", "Stinging Vitriol"] {
            let mut g = scenario(p);
            let liege = put(&mut g, p.other(), "Wilt-Leaf Liege", Zone::Hand);
            let land = put(&mut g, p.other(), "Forest", Zone::Hand);
            let spell = put(&mut g, p, name, Zone::Hand);
            let life = g.state.players[p.other().index()].life;
            assert!(!g.observe(p).objects.iter().any(|o| o.id == liege));
            cast(&mut g, spell, Some(Target::Player(p.other())));
            for _ in 0..48 {
                if matches!(g.state.pending, Pending::RevealedDiscard { .. }) {
                    break;
                }
                act(&mut g, Action::Pass);
            }
            assert!(matches!(g.state.pending, Pending::RevealedDiscard { .. }));
            assert_eq!(g.actor(), Some(p));
            assert!(g.observe(p).objects.iter().any(|o| o.id == liege));
            let land_ref = g.object_ref(land);
            reject(&mut g, Action::ChooseRevealedDiscard { object: land_ref });
            let before = g.snapshot();
            super::super::contracts::branch_contract(g.clone());
            for replay in 0..2 {
                if replay == 1 {
                    g.restore(&before).unwrap();
                }
                let object = g.object_ref(liege);
                act(&mut g, Action::ChooseRevealedDiscard { object });
                settle(&mut g);
                assert_eq!(g.object_view(liege).zone, Zone::Battlefield);
                assert_eq!(g.object_view(liege).controller, p.other());
                assert_eq!(g.object_view(land).zone, Zone::Hand);
                assert_eq!(
                    g.state.players[p.other().index()].life,
                    life - if name == "Stinging Vitriol" { 2 } else { 0 }
                );
                let jaces = fra_jaces(&g, p);
                assert_eq!(jaces.len(), usize::from(name == "Solve for Disappointment"));
                if !jaces.is_empty() {
                    assert_eq!(g.counter_count(jaces[0], CounterKind::Loyalty), 1);
                }
            }
        }
    }
}
