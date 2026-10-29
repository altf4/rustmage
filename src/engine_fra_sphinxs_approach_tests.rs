use super::*;

fn put_approach(g: &mut Game, player: Player, zone: Zone) -> CardId {
    let id = put(g, player, "Shock", zone);
    g.state.objects[id as usize].card = Card::test_index(4016);
    id
}

fn approach_until_choice(g: &mut Game) {
    for _ in 0..96 {
        if matches!(g.state.pending, Pending::EffectChoice { .. }) {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("approach choice not offered: {:?}", g.state.pending);
}

#[test]
fn approach_selects_four_atomically_and_searches_with_snapshot_replay() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let mut graves = Vec::new();
        for _ in 0..5 {
            graves.push(put_approach(&mut g, p, Zone::Graveyard));
        }
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
        reject(
            &mut g,
            Action::SelectResolutionExile {
                object: foreign_ref,
            },
        );
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
        assert!(
            graves
                .iter()
                .all(|&id| g.object_view(id).zone == Zone::Graveyard)
        );
        let partial = g.snapshot();
        super::contracts::branch_contract(g.clone());
        for _ in 0..2 {
            for &id in &graves[1..4] {
                let object = g.object_ref(id);
                act(&mut g, Action::SelectResolutionExile { object });
            }
            assert_eq!(g.object_view(spell).zone, Zone::Exile);
            for &id in &graves[..4] {
                assert_eq!(g.object_view(id).zone, Zone::Exile);
            }
            assert_eq!(g.object_view(graves[4]).zone, Zone::Graveyard);
            assert!(
                g.legal_actions()
                    .contains(&Action::ChooseSearch { card: sphinx })
            );
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
            let cards: Vec<_> = (0..count)
                .map(|_| put_approach(&mut g, p, Zone::Graveyard))
                .collect();
            let spell = put_approach(&mut g, p, Zone::Hand);
            cast(&mut g, spell, None);
            for _ in 0..96 {
                if matches!(g.state.pending, Pending::EffectChoice { .. })
                    || (g.state.pending == Pending::Priority
                        && g.state.stack.is_empty()
                        && g.state.resolving.is_none())
                {
                    break;
                }
                choose(&mut g, |a| {
                    matches!(a, Action::Pass | Action::PutTrigger { .. })
                });
            }
            if count == 4 {
                let reference = g.object_ref(cards[0]);
                act(&mut g, Action::SelectResolutionExile { object: reference });
                act(&mut g, Action::DeclineEffectPayment);
                settle(&mut g);
            }
            assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
            assert!(
                cards
                    .iter()
                    .all(|&id| g.object_view(id).zone == Zone::Graveyard)
            );
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 2);
        }
    }
}

#[test]
fn approach_spell_copy_can_choose_the_resolution_payment_independently() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let cards: Vec<_> = (0..4)
            .map(|_| put_approach(&mut g, p, Zone::Graveyard))
            .collect();
        let sphinx = put(&mut g, p, "Curator of Destinies", Zone::Library);
        put(&mut g, p, "Island", Zone::Library);
        put(&mut g, p, "Island", Zone::Library);
        let spell = put_approach(&mut g, p, Zone::Hand);
        cast(&mut g, spell, None);
        let original = g.state.stack.last().unwrap().clone();
        let copy = g.allocate_spell_copy(&original, p);
        let copy_object = copy.source.unwrap().object;
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
            let cards: Vec<_> = (0..4)
                .map(|_| put_approach(&mut g, p, Zone::Graveyard))
                .collect();
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
                g.state
                    .resolving
                    .as_mut()
                    .unwrap()
                    .choice
                    .as_mut()
                    .unwrap()
                    .selected[p.index()]
                .push(reference);
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
            for &id in &cards {
                assert_eq!(g.object_view(id).zone, Zone::Graveyard);
            }
            assert_eq!(g.object_view(spell).zone, Zone::Stack);
        }
    }
}

#[test]
fn copied_spell_exile_emits_zone_event_and_ceases_only_at_sba() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let spell = put(&mut g, p, "Shock", Zone::Hand);
        let target = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        cast(&mut g, spell, Some(Target::Card(target)));
        let original = g.state.stack.last().unwrap().clone();
        let copy = g.allocate_spell_copy(&original, p);
        let original_ref = copy.source.unwrap().object;
        g.state.stack.push(copy);
        let event = g.move_object(original_ref.card, Zone::Exile);
        assert!(event.moved);
        assert_eq!(event.before.reference, original_ref);
        assert_eq!(event.before.zone, Zone::Stack);
        assert_eq!(event.destination, Zone::Exile);
        assert_eq!(g.object_view(original_ref.card).zone, Zone::Exile);
        assert!(!g.state.cards_exiled_this_turn); // a spell copy is not a card
        let checkpoint = g.snapshot();
        for _ in 0..2 {
            g.state_based_actions();
            assert_eq!(g.object_view(original_ref.card).zone, Zone::Outside);
            assert_ne!(g.object_ref(original_ref.card), event.after);
            g.restore(&checkpoint).unwrap();
            assert_eq!(g.object_view(original_ref.card).zone, Zone::Exile);
        }
    }
}

#[test]
fn copied_spell_moves_between_zones_before_sba_and_reuses_slot_safely() {
    for p in [Player::P0, Player::P1] {
        for destination in [Zone::Graveyard, Zone::Hand, Zone::Library] {
            let mut g = scenario(p);
            let spell = put(&mut g, p, "Shock", Zone::Hand);
            let target = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            cast(&mut g, spell, Some(Target::Card(target)));
            let original = g.state.stack.last().unwrap().clone();
            let copy = g.allocate_spell_copy(&original, p);
            let initial = copy.source.unwrap().object;
            g.state.stack.push(copy);
            let first = g.move_object(initial.card, Zone::Exile);
            assert!(first.moved);
            assert_eq!(first.destination, Zone::Exile);
            let second = g.move_object(initial.card, destination);
            assert!(second.moved);
            assert_eq!(second.before.reference, first.after);
            assert_eq!(second.before.zone, Zone::Exile);
            assert_eq!(second.destination, destination);
            assert_eq!(g.object_view(initial.card).zone, destination);
            assert!(!g.state.players[p.index()].card_left_graveyard_this_turn);
            let checkpoint = g.snapshot();
            for _ in 0..2 {
                g.state_based_actions();
                assert_eq!(g.object_view(initial.card).zone, Zone::Outside);
                if destination == Zone::Library {
                    assert!(!g.state.players[p.index()].library.contains(&initial.card));
                }
                let replacement = g.allocate_spell_copy(&original, p);
                assert_eq!(replacement.card, initial.card);
                assert_ne!(replacement.object, second.after);
                assert_eq!(g.object_view(spell).zone, Zone::Stack);
                g.restore(&checkpoint).unwrap();
                assert_eq!(g.object_ref(initial.card), second.after);
            }
        }
    }
}

#[test]
fn copied_spell_move_from_stack_to_nonbattlefield_zone_is_an_event() {
    for p in [Player::P0, Player::P1] {
        for destination in [Zone::Graveyard, Zone::Hand, Zone::Library] {
            let mut g = scenario(p);
            let spell = put(&mut g, p, "Shock", Zone::Hand);
            let target = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            cast(&mut g, spell, Some(Target::Card(target)));
            let original = g.state.stack.last().unwrap().clone();
            let copy = g.allocate_spell_copy(&original, p);
            let reference = copy.source.unwrap().object;
            g.state.stack.push(copy);
            let event = g.move_object(reference.card, destination);
            assert!(event.moved);
            assert_eq!(event.before.reference, reference);
            assert_eq!(event.destination, destination);
            assert_eq!(g.object_view(reference.card).zone, destination);
            g.state_based_actions();
            assert_eq!(g.object_view(reference.card).zone, Zone::Outside);
        }
    }
}

#[test]
fn copied_spell_is_not_a_card_for_graveyard_replacement() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p, "Rest in Peace", Zone::Battlefield);
        let spell = put(&mut g, p, "Shock", Zone::Hand);
        let target = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        cast(&mut g, spell, Some(Target::Card(target)));
        let original = g.state.stack.last().unwrap().clone();
        let copy = g.allocate_spell_copy(&original, p);
        let reference = copy.source.unwrap().object;
        g.state.stack.push(copy);
        let event = g.move_object(reference.card, Zone::Graveyard);
        assert!(event.moved);
        assert_eq!(event.destination, Zone::Graveyard);
        assert!(!g.state.players[p.index()].card_left_graveyard_this_turn);
        g.state_based_actions();
        assert_eq!(g.object_view(reference.card).zone, Zone::Outside);
    }
}

#[test]
fn transient_spell_copy_is_not_a_card_for_draw_and_search_during_resolution() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let graves: Vec<_> = (0..4)
            .map(|_| put_approach(&mut g, p, Zone::Graveyard))
            .collect();
        let sphinx = put(&mut g, p, "Curator of Destinies", Zone::Library);
        put(&mut g, p, "Island", Zone::Library);
        put(&mut g, p, "Island", Zone::Library);
        let curator = put(&mut g, p, "Curator of Destinies", Zone::Hand);
        cast(&mut g, curator, None);
        let original = g.state.stack.last().unwrap().clone();
        let spell = put_approach(&mut g, p, Zone::Hand);
        cast(&mut g, spell, None);
        approach_until_choice(&mut g);
        // Put a copy above a real card while resolution is suspended, before SBA.
        let copy = g.allocate_spell_copy(&original, p);
        let fake = copy.source.unwrap().id;
        g.state.stack.push(copy);
        let real_top = put(&mut g, p, "Island", Zone::Library);
        g.move_object(fake, Zone::Library);
        assert!(!g.state.objects[fake as usize].is_card());
        let checkpoint = g.snapshot();
        assert!(g.draw_one_unreplaced(p));
        assert_eq!(g.object_view(real_top).zone, Zone::Hand);
        assert_eq!(g.object_view(fake).zone, Zone::Library);
        g.restore(&checkpoint).unwrap();
        for id in graves {
            let object = g.object_ref(id);
            act(&mut g, Action::SelectResolutionExile { object });
        }
        assert!(
            g.legal_actions()
                .contains(&Action::ChooseSearch { card: sphinx })
        );
        assert!(
            !g.legal_actions()
                .contains(&Action::ChooseSearch { card: fake })
        );
    }
}

#[test]
fn transient_named_spell_copy_cannot_pay_graveyard_card_cost() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let cards: Vec<_> = (0..4)
            .map(|_| put_approach(&mut g, p, Zone::Graveyard))
            .collect();
        let spell = put_approach(&mut g, p, Zone::Hand);
        cast(&mut g, spell, None);
        let original = g.state.stack.last().unwrap().clone();
        approach_until_choice(&mut g);
        let copy = g.allocate_spell_copy(&original, p);
        let fake = copy.source.unwrap().id;
        g.state.stack.push(copy);
        g.move_object(fake, Zone::Graveyard);
        let checkpoint = g.snapshot();
        let fake_ref = g.object_ref(fake);
        assert!(
            !g.legal_actions()
                .contains(&Action::SelectResolutionExile { object: fake_ref })
        );
        reject(&mut g, Action::SelectResolutionExile { object: fake_ref });
        assert_eq!(g.snapshot(), checkpoint);
        for id in cards {
            let object = g.object_ref(id);
            act(&mut g, Action::SelectResolutionExile { object });
        }
        assert_eq!(g.object_view(fake).zone, Zone::Graveyard);
    }
}

#[test]
fn shuffle_keeps_transient_noncard_position_and_replays_card_order() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let spell = put(&mut g, p, "Shock", Zone::Hand);
        let target = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        cast(&mut g, spell, Some(Target::Card(target)));
        let original = g.state.stack.last().unwrap().clone();
        let copy = g.allocate_spell_copy(&original, p);
        let fake = copy.source.unwrap().id;
        g.state.stack.push(copy);
        g.move_object(fake, Zone::Library);
        put(&mut g, p, "Island", Zone::Library);
        let before = g.state.players[p.index()].library.clone();
        let position = before.iter().position(|&id| id == fake).unwrap();
        let checkpoint = g.snapshot();
        g.shuffle_library_cards(p);
        let shuffled = g.state.players[p.index()].library.clone();
        assert_eq!(shuffled[position], fake);
        let mut sorted_before = before.clone();
        let mut sorted_after = shuffled.clone();
        sorted_before.sort_unstable();
        sorted_after.sort_unstable();
        assert_eq!(sorted_before, sorted_after);
        g.restore(&checkpoint).unwrap();
        g.shuffle_library_cards(p);
        assert_eq!(g.state.players[p.index()].library, shuffled);
    }
}

#[test]
fn transient_copy_in_hand_is_not_offered_as_a_discard_card() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        for _ in 0..4 {
            put_approach(&mut g, p, Zone::Graveyard);
        }
        let spell = put_approach(&mut g, p, Zone::Hand);
        cast(&mut g, spell, None);
        let original = g.state.stack.last().unwrap().clone();
        approach_until_choice(&mut g);
        let real = put(&mut g, p, "Shock", Zone::Hand);
        let before = g.hand_count(p);
        let copy = g.allocate_spell_copy(&original, p);
        let fake = copy.source.unwrap().id;
        g.state.stack.push(copy);
        g.move_object(fake, Zone::Hand);
        assert_eq!(g.hand_count(p), before);
        // Model a discard choice nested in the same unresolved effect.
        g.state.pending = Pending::EffectDiscard {
            player: p,
            remaining: 1,
        };
        assert!(g.legal_actions().contains(&Action::Discard { card: real }));
        assert!(!g.legal_actions().contains(&Action::Discard { card: fake }));
        let checkpoint = g.snapshot();
        reject(&mut g, Action::Discard { card: fake });
        assert_eq!(g.snapshot(), checkpoint);
        g.discard_batch(&[fake, real], None);
        assert_eq!(g.object_view(fake).zone, Zone::Hand);
        assert_eq!(g.object_view(real).zone, Zone::Graveyard);
        g.restore(&checkpoint).unwrap();
        assert_eq!(g.object_view(real).zone, Zone::Hand);
    }
}

#[test]
fn transient_copy_graveyard_transitions_do_not_trigger_card_observers() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p, "Owlin Historian", Zone::Battlefield);
        let spell = put(&mut g, p, "Shock", Zone::Hand);
        let target = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        cast(&mut g, spell, Some(Target::Card(target)));
        let original = g.state.stack.last().unwrap().clone();
        let copy = g.allocate_spell_copy(&original, p);
        let fake = copy.source.unwrap().id;
        g.state.stack.push(copy);
        let entry = g.move_object(fake, Zone::Graveyard);
        assert!(!entry.was_card);
        assert!(!g.graveyard_entry_this_turn(entry.after));
        let pending_before = g.state.pending_triggers.len();
        let departure = g.move_object(fake, Zone::Exile);
        assert!(!departure.was_card);
        assert_eq!(g.state.pending_triggers.len(), pending_before);
        let physical = put(&mut g, p, "Shock", Zone::Graveyard);
        let departure = g.move_object(physical, Zone::Exile);
        assert!(departure.was_card);
        assert!(g.state.pending_triggers.len() > pending_before);
    }
}

#[test]
fn second_from_top_counts_cards_around_a_transient_copy() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let spell = put(&mut g, p, "Shock", Zone::Hand);
        let target = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        cast(&mut g, spell, Some(Target::Card(target)));
        let original = g.state.stack.last().unwrap().clone();
        let copy = g.allocate_spell_copy(&original, p);
        let fake = copy.source.unwrap().id;
        g.state.stack.push(copy);
        g.move_object(fake, Zone::Library);
        let top = put(&mut g, p, "Island", Zone::Library);
        let permanent = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let object = g.object_ref(permanent);
        assert!(g.move_target_to_library(
            original.source.unwrap().object,
            object,
            LibraryDestination::SecondFromTop
        ));
        let cards: Vec<_> = g.state.players[p.index()]
            .library
            .iter()
            .rev()
            .copied()
            .filter(|&id| g.state.objects[id as usize].is_card())
            .collect();
        assert_eq!(&cards[..2], &[top, permanent]);
        assert!(g.state.players[p.index()].library.contains(&fake));
    }
}

#[test]
fn transient_graveyard_copy_is_not_a_card_for_ability_guards() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let spell = put(&mut g, p, "Shock", Zone::Hand);
        let target = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        cast(&mut g, spell, Some(Target::Card(target)));
        let original = g.state.stack.last().unwrap().clone();
        let copy = g.allocate_spell_copy(&original, p);
        let fake = copy.source.unwrap().id;
        g.state.stack.push(copy);
        g.move_object(fake, Zone::Graveyard);
        assert!(!activations::graveyard_cards_at_least(&g, p, 0, 1));
        let checkpoint = g.snapshot();
        put(&mut g, p, "Shock", Zone::Graveyard);
        assert!(activations::graveyard_cards_at_least(&g, p, 0, 1));
        g.restore(&checkpoint).unwrap();
        assert!(!activations::graveyard_cards_at_least(&g, p, 0, 1));
    }
}

#[test]
fn revealed_discard_choice_excludes_transient_copy_and_replays() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let victim = player.other();
        let actual = put(&mut game, victim, "Bear Cub", Zone::Hand);
        let spell = put(&mut game, player, "Stinging Vitriol", Zone::Hand);
        cast(&mut game, spell, Some(Target::Player(victim)));
        let original = game.state.stack.last().unwrap().clone();
        let copy = game.allocate_spell_copy(&original, player);
        let fake = copy.source.unwrap().id;
        game.state.objects[fake as usize].owner = victim;
        game.move_object(fake, Zone::Hand);
        for _ in 0..48 {
            if matches!(game.state.pending, Pending::RevealedDiscard { .. }) {
                break;
            }
            act(&mut game, Action::Pass);
        }
        assert!(matches!(
            game.state.pending,
            Pending::RevealedDiscard { .. }
        ));
        let checkpoint = game.snapshot();
        for _ in 0..2 {
            let fake_ref = game.object_ref(fake);
            assert!(
                !game
                    .legal_actions()
                    .contains(&Action::ChooseRevealedDiscard { object: fake_ref })
            );
            reject(
                &mut game,
                Action::ChooseRevealedDiscard { object: fake_ref },
            );
            let actual_ref = game.object_ref(actual);
            act(
                &mut game,
                Action::ChooseRevealedDiscard { object: actual_ref },
            );
            assert_eq!(game.object_view(actual).zone, Zone::Graveyard);
            game.restore(&checkpoint).unwrap();
        }
    }
}

#[test]
fn half_hand_discard_quota_ignores_transient_spell_copy() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let real = put(&mut game, player, "Forest", Zone::Hand);
        let spell = put(&mut game, player, "Pox Plague", Zone::Hand);
        cast(&mut game, spell, None);
        let original = game.state.stack.last().unwrap().clone();
        let copy = game.allocate_spell_copy(&original, player);
        let fake = copy.source.unwrap().id;
        game.move_object(fake, Zone::Hand);
        let saved = game.snapshot();
        for _ in 0..2 {
            settle(&mut game);
            assert_eq!(game.object_view(real).zone, Zone::Hand);
            assert_eq!(game.object_view(fake).zone, Zone::Outside);
            game.restore(&saved).unwrap();
        }
    }
}

#[test]
fn paid_exile_top_offer_ignores_transient_copy_at_library_top() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let spell = put(&mut g, player, "Shock", Zone::Hand);
        let target = put(&mut g, player.other(), "Bear Cub", Zone::Battlefield);
        cast(&mut g, spell, Some(Target::Card(target)));
        let original = g.state.stack.last().unwrap().clone();
        let top = put(&mut g, player, "Island", Zone::Library);
        let copy = g.allocate_spell_copy(&original, player);
        let reference = copy.source.unwrap().object;
        g.state.stack.push(copy);
        g.move_object(reference.card, Zone::Library);
        assert_eq!(
            g.state.players[player.index()].library.last(),
            Some(&reference.card)
        );
        let saved = g.snapshot();
        for _ in 0..2 {
            assert!(g.exile_top_offer_paid_cast(player));
            assert_eq!(g.object_view(top).zone, Zone::Exile);
            assert_eq!(g.object_view(reference.card).zone, Zone::Library);
            assert_eq!(
                g.state.resolution_free_casts.as_ref().unwrap().remaining,
                vec![g.object_ref(top)]
            );
            g.restore(&saved).unwrap();
        }
    }
}
