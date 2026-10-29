use super::*;

fn put_approach(g: &mut Game, player: Player, zone: Zone) -> CardId {
    let id = put(g, player, "Shock", zone);
    g.state.objects[id as usize].card = Card::test_index(4016);
    id
}

fn approach_until_choice(g: &mut Game) {
    for _ in 0..96 {
        if matches!(g.state.pending, Pending::EffectChoice { .. }) { return; }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("approach choice not offered: {:?}", g.state.pending);
}

#[test]
fn approach_selects_four_atomically_and_searches_with_snapshot_replay() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let mut graves = Vec::new();
        for _ in 0..5 { graves.push(put_approach(&mut g, p, Zone::Graveyard)); }
        let foreign = put_approach(&mut g, p.other(), Zone::Graveyard);
        let wrong = put(&mut g, p, "Bear Cub", Zone::Graveyard);
        let sphinx = put(&mut g, p, "Curator of Destinies", Zone::Library);
        // The two cards drawn are above the Sphinx.
        put(&mut g, p, "Island", Zone::Library);
        put(&mut g, p, "Island", Zone::Library);
        let spell = put_approach(&mut g, p, Zone::Hand);
        cast(&mut g, spell, None);
        approach_until_choice(&mut g);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 2);
        let before = g.snapshot();
        let foreign_ref = g.object_ref(foreign);
        let wrong_ref = g.object_ref(wrong);
        reject(&mut g, Action::SelectResolutionExile { object: foreign_ref });
        reject(&mut g, Action::SelectResolutionExile { object: wrong_ref });
        assert_eq!(g.snapshot(), before);
        let first = g.object_ref(graves[0]);
        act(&mut g, Action::SelectResolutionExile { object: first });
        for viewer in [p, p.other()] {
            let choice = g.observe(viewer).effect_choice.unwrap();
            assert_eq!(choice.selected[p.index()], vec![first]);
        }
        reject(&mut g, Action::SelectResolutionExile { object: first });
        assert_eq!(g.object_view(spell).zone, Zone::Stack);
        assert!(graves.iter().all(|&id| g.object_view(id).zone == Zone::Graveyard));
        let partial = g.snapshot();
        super::super::contracts::branch_contract(g.clone());
        for _ in 0..2 {
            for &id in &graves[1..4] {
                let object = g.object_ref(id);
                act(&mut g, Action::SelectResolutionExile { object });
            }
            assert_eq!(g.object_view(spell).zone, Zone::Exile);
            for &id in &graves[..4] { assert_eq!(g.object_view(id).zone, Zone::Exile); }
            assert_eq!(g.object_view(graves[4]).zone, Zone::Graveyard);
            assert!(g.legal_actions().contains(&Action::ChooseSearch { card: sphinx }));
            act(&mut g, Action::ChooseSearch { card: sphinx });
            act(&mut g, Action::FinishSearch);
            settle(&mut g);
            assert_eq!(g.object_view(sphinx).zone, Zone::Battlefield);
            assert_eq!(g.object_view(sphinx).controller, p);
            g.restore(&partial).unwrap();
        }
    }
}

#[test]
fn approach_decline_or_insufficient_graveyard_leaves_no_partial_payment() {
    for p in [Player::P0, Player::P1] {
        for count in [3, 4] {
            let mut g = scenario(p);
            let cards: Vec<_> = (0..count).map(|_| put_approach(&mut g, p, Zone::Graveyard)).collect();
            let spell = put_approach(&mut g, p, Zone::Hand);
            cast(&mut g, spell, None);
            for _ in 0..96 {
                if matches!(g.state.pending, Pending::EffectChoice { .. }) ||
                    (g.state.pending == Pending::Priority && g.state.stack.is_empty() && g.state.resolving.is_none()) { break; }
                choose(&mut g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
            }
            if count == 4 {
                let reference = g.object_ref(cards[0]);
                act(&mut g, Action::SelectResolutionExile { object: reference });
                act(&mut g, Action::DeclineEffectPayment);
                settle(&mut g);
            }
            assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
            assert!(cards.iter().all(|&id| g.object_view(id).zone == Zone::Graveyard));
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 2);
        }
    }
}

#[test]
fn approach_spell_copy_can_choose_the_resolution_payment_independently() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let cards: Vec<_> = (0..4).map(|_| put_approach(&mut g, p, Zone::Graveyard)).collect();
        let sphinx = put(&mut g, p, "Curator of Destinies", Zone::Library);
        put(&mut g, p, "Island", Zone::Library);
        put(&mut g, p, "Island", Zone::Library);
        let spell = put_approach(&mut g, p, Zone::Hand);
        cast(&mut g, spell, None);
        let original = g.state.stack.last().unwrap().clone();
        let copy = g.allocate_spell_copy(&original, p);
        let copy_object = copy.object;
        g.state.stack.push(copy);
        approach_until_choice(&mut g);
        for card in cards {
            let object = g.object_ref(card);
            act(&mut g, Action::SelectResolutionExile { object });
        }
        assert_eq!(g.object_view(copy_object.card).zone, Zone::Exile);
        assert!(g.state.stack.iter().any(|item| item.id == original.id));
        act(&mut g, Action::ChooseSearch { card: sphinx });
        act(&mut g, Action::FinishSearch);
        settle(&mut g);
        assert_eq!(g.object_view(sphinx).zone, Zone::Battlefield);
        assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
    }
}

#[test]
fn approach_revalidates_every_component_before_any_exile() {
    for p in [Player::P0, Player::P1] {
        for lost in 0..5 {
            let mut g = scenario(p);
            let cards: Vec<_> = (0..4).map(|_| put_approach(&mut g, p, Zone::Graveyard)).collect();
            let spell = put_approach(&mut g, p, Zone::Hand);
            cast(&mut g, spell, None);
            approach_until_choice(&mut g);
            for &id in &cards[..3] {
                let object = g.object_ref(id);
                act(&mut g, Action::SelectResolutionExile { object });
            }
            let checkpoint = g.snapshot();
            // Model an interrupted continuation whose old selected incarnation
            // became invalid before commitment; such changes require a nested
            // continuation, not ordinary priority between these selections.
            let stale = if lost < 4 { cards[lost] } else { spell };
            if lost == 3 {
                // Select the fourth object, but construct its continuation with
                // the stale exact reference before asking the commit helper.
                let reference = g.object_ref(cards[3]);
                g.state.resolving.as_mut().unwrap().choice.as_mut().unwrap().selected[p.index()].push(reference);
                g.move_object(stale, Zone::Hand);
                assert!(!g.advance_effect_choice());
                g.run_program();
            } else {
                g.move_object(stale, Zone::Hand);
                let object = g.object_ref(cards[3]);
                act(&mut g, Action::SelectResolutionExile { object });
            }
            settle(&mut g);
            for &id in &cards {
                assert_ne!(g.object_view(id).zone, Zone::Exile);
            }
            assert_ne!(g.object_view(spell).zone, Zone::Exile);
            g.restore(&checkpoint).unwrap();
            assert_eq!(g.snapshot(), checkpoint);
            for &id in &cards { assert_eq!(g.object_view(id).zone, Zone::Graveyard); }
            assert_eq!(g.object_view(spell).zone, Zone::Stack);
        }
    }
}
