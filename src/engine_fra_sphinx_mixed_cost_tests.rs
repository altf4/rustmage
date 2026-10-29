//! Sphinx's real resolution payment with synthetic replacement providers.
use super::*;

#[test]
fn sphinx_competing_exile_replacements_replay_atomically() {
    use zone_replacements::{TEST_PREVENT_ZONE_MOVE, TEST_REDIRECT_EXILE_TO_BATTLEFIELD};
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            TEST_PREVENT_ZONE_MOVE.with(|p| p.set(None));
            TEST_REDIRECT_EXILE_TO_BATTLEFIELD.with(|p| p.set(None));
        }
    }
    let _reset = Reset;
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let cards: Vec<_> = (0..4)
            .map(|_| {
                let id = put(&mut g, player, "Shock", Zone::Graveyard);
                g.state.objects[id as usize].card = Card::test_index(4016);
                id
            })
            .collect();
        let source = put(&mut g, player, "Shock", Zone::Hand);
        g.state.objects[source as usize].card = Card::test_index(4016);
        cast(&mut g, source, None);
        for _ in 0..96 {
            if matches!(g.state.pending, Pending::EffectChoice { .. }) {
                break;
            }
            choose(&mut g, |a| {
                matches!(a, Action::Pass | Action::PutTrigger { .. })
            });
        }
        let reference = g.object_ref(cards[0]);
        TEST_PREVENT_ZONE_MOVE.with(|p| p.set(Some(reference)));
        TEST_REDIRECT_EXILE_TO_BATTLEFIELD.with(|p| p.set(Some(reference)));
        // Synthetic providers bypass normal card registration.
        g.state.zone_rules.get_or_insert_default();
        for &id in &cards[..3] {
            let object = g.object_ref(id);
            act(&mut g, Action::SelectResolutionExile { object });
        }
        let last = g.object_ref(cards[3]);
        act(&mut g, Action::SelectResolutionExile { object: last });
        assert!(matches!(g.state.pending, Pending::ZoneReplacement { player: p } if p == player));
        let owner_view = g.observe(player);
        let choice = owner_view.zone_replacement_choice.as_ref().unwrap();
        assert_eq!(choice.card, Card::test_index(4016));
        assert_eq!(choice.owner, player);
        assert_eq!(choice.from, Zone::Graveyard);
        assert_eq!(
            choice.choices,
            vec![
                ZoneReplacementKind::Prevent,
                ZoneReplacementKind::Battlefield
            ]
        );
        for effect in [
            ZoneReplacementKind::Prevent,
            ZoneReplacementKind::Battlefield,
        ] {
            assert!(
                owner_view
                    .actions
                    .contains(&Action::ChooseZoneReplacement { effect })
            );
        }
        let opponent_view = g.observe(player.other());
        assert!(opponent_view.zone_replacement_choice.is_none());
        assert!(opponent_view.actions.is_empty());
        for observed in [&owner_view, &opponent_view] {
            assert_eq!(observed.actor, Some(player));
            assert!(
                observed
                    .objects
                    .iter()
                    .any(|object| object.id == source && object.zone == Zone::Stack)
            );
            for &id in &cards {
                assert!(
                    observed
                        .objects
                        .iter()
                        .any(|object| object.id == id && object.zone == Zone::Graveyard)
                );
            }
        }
        let pending = g.snapshot();
        let view = |game: &Game, p| {
            let mut value = serde_json::to_value(game.observe(p)).unwrap();
            value.as_object_mut().unwrap().remove("revision");
            value
        };
        let views = [player, player.other()].map(|p| view(&g, p));
        for outcome in [
            ZoneReplacementKind::Prevent,
            ZoneReplacementKind::Battlefield,
        ] {
            let prior_revision = g.revision();
            let old_index = g
                .legal_actions()
                .iter()
                .position(|a| matches!(a, Action::ChooseZoneReplacement { .. }))
                .unwrap();
            g.restore(&pending).unwrap();
            assert!(g.revision() > prior_revision);
            assert!(g.apply_index(player, prior_revision, old_index).is_err());
            assert_eq!([player, player.other()].map(|p| view(&g, p)), views);
            let action = Action::ChooseZoneReplacement { effect: outcome };
            let index = g.legal_actions().iter().position(|a| *a == action).unwrap();
            let mut raw = g.clone();
            let mut indexed = g.clone();
            assert!(raw.apply(player.other(), action.clone()).is_err());
            assert_eq!(raw.snapshot(), pending);
            raw.apply(player, action).unwrap();
            indexed.apply_index(player, g.revision(), index).unwrap();
            assert_eq!(raw.snapshot(), indexed.snapshot());
            assert_eq!(raw.object_view(source).zone, Zone::Exile);
            for &id in &cards[1..] {
                assert_eq!(raw.object_view(id).zone, Zone::Exile);
            }
            assert_eq!(
                raw.object_view(cards[0]).zone,
                if outcome == ZoneReplacementKind::Prevent {
                    Zone::Graveyard
                } else {
                    Zone::Battlefield
                }
            );
        }
        TEST_PREVENT_ZONE_MOVE.with(|p| p.set(None));
        TEST_REDIRECT_EXILE_TO_BATTLEFIELD.with(|p| p.set(None));
    }
}

fn sphinx_card(g: &mut Game, player: Player, zone: Zone) -> CardId {
    let id = put(g, player, "Shock", zone);
    g.state.objects[id as usize].card = Card::test_index(4016);
    id
}
fn wait_for_sphinx_choice(g: &mut Game) {
    for _ in 0..96 {
        if matches!(g.state.pending, Pending::EffectChoice { .. }) {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("missing Sphinx choice: {:?}", g.state.pending);
}

#[test]
fn sphinx_prevented_component_still_pays_and_queues_one_departure_observer() {
    use zone_replacements::TEST_PREVENT_ZONE_MOVE;
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            TEST_PREVENT_ZONE_MOVE.with(|p| p.set(None));
        }
    }
    let _reset = Reset;
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let observer = put(&mut g, player, "Owlin Historian", Zone::Battlefield);
        let cards: Vec<_> = (0..4)
            .map(|_| sphinx_card(&mut g, player, Zone::Graveyard))
            .collect();
        let source = sphinx_card(&mut g, player, Zone::Hand);
        cast(&mut g, source, None);
        wait_for_sphinx_choice(&mut g);
        let before = g.snapshot();
        TEST_PREVENT_ZONE_MOVE.with(|p| p.set(Some(g.object_ref(cards[0]))));
        for _ in 0..2 {
            for &id in &cards {
                let object = g.object_ref(id);
                act(&mut g, Action::SelectResolutionExile { object });
            }
            assert_eq!(g.object_view(cards[0]).zone, Zone::Graveyard);
            assert!(
                cards[1..]
                    .iter()
                    .all(|&id| g.object_view(id).zone == Zone::Exile)
            );
            assert_eq!(g.object_view(source).zone, Zone::Exile);
            assert_eq!(
                g.state
                    .pending_triggers
                    .iter()
                    .filter(|item| item.source.is_some_and(|source| source.id == observer))
                    .count(),
                1
            );
            assert!(
                g.state.resolving.is_some(),
                "observer waits for Sphinx's search"
            );
            g.restore(&before).unwrap();
        }
        TEST_PREVENT_ZONE_MOVE.with(|p| p.set(None));
    }
}

#[test]
fn sphinx_copy_created_through_legal_cast_exiles_then_ceases_after_resolution() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let cards: Vec<_> = (0..4)
            .map(|_| sphinx_card(&mut g, player, Zone::Graveyard))
            .collect();
        let sphinx = put(&mut g, player, "Sphinx of the Final Word", Zone::Library);
        put(&mut g, player, "Island", Zone::Library);
        put(&mut g, player, "Island", Zone::Library);
        let source = sphinx_card(&mut g, player, Zone::Hand);
        cast(&mut g, source, None);
        let sparks = put(&mut g, player, "Choreographed Sparks", Zone::Hand);
        act(&mut g, Action::AnnounceCast { card: sparks });
        act(&mut g, Action::ChooseMode { mode: 0 });
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Spell(source),
            },
        );
        pay(&mut g);
        wait_for_sphinx_choice(&mut g);
        let copy = g
            .state
            .resolving
            .as_ref()
            .unwrap()
            .item
            .source
            .unwrap()
            .object;
        assert!(g.state.resolving.as_ref().unwrap().item.is_copy);
        let before = g.snapshot();
        for _ in 0..2 {
            let action = Action::SelectResolutionExile {
                object: g.object_ref(cards[0]),
            };
            assert!(g.apply(player.other(), action.clone()).is_err());
            assert_eq!(g.snapshot(), before);
            act(&mut g, action);
            for &id in &cards[1..] {
                let object = g.object_ref(id);
                act(&mut g, Action::SelectResolutionExile { object });
            }
            // Requires the separately drafted copy-card lifecycle foundation.
            assert_eq!(g.object_view(copy.card).zone, Zone::Exile);
            act(&mut g, Action::ChooseSearch { card: sphinx });
            act(&mut g, Action::FinishSearch);
            settle(&mut g);
            assert_eq!(g.object_view(copy.card).zone, Zone::Outside);
            g.restore(&before).unwrap();
        }
    }
}

#[test]
fn sphinx_prohibition_revalidates_the_whole_payment_before_any_exile() {
    use ability_world::TEST_EXILE_PROHIBITOR;
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            TEST_EXILE_PROHIBITOR.with(|p| p.set(None));
        }
    }
    let _reset = Reset;
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let provider = put(&mut g, player.other(), "Bear Cub", Zone::Battlefield);
        let cards: Vec<_> = (0..4)
            .map(|_| sphinx_card(&mut g, player, Zone::Graveyard))
            .collect();
        let source = sphinx_card(&mut g, player, Zone::Hand);
        cast(&mut g, source, None);
        wait_for_sphinx_choice(&mut g);
        for &id in &cards[..3] {
            let object = g.object_ref(id);
            act(&mut g, Action::SelectResolutionExile { object });
        }
        TEST_EXILE_PROHIBITOR.with(|p| p.set(Some(g.object_ref(provider))));
        let saved = g.snapshot();
        for _ in 0..2 {
            let last = g.object_ref(cards[3]);
            let prohibited = Action::SelectResolutionExile { object: last };
            assert!(!g.legal_actions().contains(&prohibited));
            assert!(g.apply(player, prohibited).is_err());
            assert_eq!(g.snapshot(), saved);
            assert!(g.legal_actions().contains(&Action::DeclineEffectPayment));
            act(&mut g, Action::DeclineEffectPayment);
            assert!(
                cards
                    .iter()
                    .all(|&id| g.object_view(id).zone == Zone::Graveyard)
            );
            assert_ne!(g.object_view(source).zone, Zone::Exile);
            assert!(!matches!(g.state.pending, Pending::EffectSearch { .. }));
            g.restore(&saved).unwrap();
        }
        TEST_EXILE_PROHIBITOR.with(|p| p.set(None));
    }
}

#[test]
fn sphinx_stale_staged_entry_discards_consumed_replacement_replay() {
    use zone_replacements::{
        TEST_FAKE_ENTRY_LIFE, TEST_PREVENT_ZONE_MOVE, TEST_REDIRECT_EXILE_TO_BATTLEFIELD,
    };
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            TEST_PREVENT_ZONE_MOVE.with(|p| p.set(None));
            TEST_REDIRECT_EXILE_TO_BATTLEFIELD.with(|p| p.set(None));
            TEST_FAKE_ENTRY_LIFE.with(|p| p.set(None));
        }
    }
    let _reset = Reset;
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let cards: Vec<_> = (0..4)
            .map(|_| sphinx_card(&mut g, player, Zone::Graveyard))
            .collect();
        let source = sphinx_card(&mut g, player, Zone::Hand);
        cast(&mut g, source, None);
        wait_for_sphinx_choice(&mut g);
        let reference = g.object_ref(cards[0]);
        TEST_PREVENT_ZONE_MOVE.with(|p| p.set(Some(reference)));
        TEST_REDIRECT_EXILE_TO_BATTLEFIELD.with(|p| p.set(Some(reference)));
        // Synthetic providers bypass normal card registration.
        g.state.zone_rules.get_or_insert_default();
        TEST_FAKE_ENTRY_LIFE.with(|p| p.set(Some(reference)));
        for &id in &cards {
            let object = g.object_ref(id);
            act(&mut g, Action::SelectResolutionExile { object });
        }
        assert!(matches!(g.state.pending, Pending::ZoneReplacement { player: p } if p == player));
        act(
            &mut g,
            Action::ChooseZoneReplacement {
                effect: ZoneReplacementKind::Battlefield,
            },
        );
        assert!(matches!(g.state.pending, Pending::EntryLife { player: p } if p == player));
        let batch = g.state.entry_batch.as_ref().unwrap();
        let mut without = (**batch).clone();
        let checkpoint = without.atomic_cost.take().unwrap();
        assert_eq!(
            batch.bytes(),
            without.bytes()
                + std::mem::size_of::<entries::AtomicCostCheckpoint>()
                + checkpoint.bytes
                + checkpoint.components.len() * std::mem::size_of::<(ObjectRef, Zone)>()
        );
        // No supported nested move exists in this suspension; inject the stale
        // incarnation to exercise the exact rollback/replay boundary.
        g.move_object(cards[1], Zone::Hand);
        let suspended = g.snapshot();
        for _ in 0..2 {
            let action = Action::ChooseEntryLife { pay: true };
            assert!(g.apply(player.other(), action.clone()).is_err());
            assert_eq!(g.snapshot(), suspended);
            act(&mut g, action);
            assert!(
                g.state
                    .zone_rules
                    .as_ref()
                    .is_none_or(|rules| !rules.has_checkpoint())
            );
            assert!(
                cards
                    .iter()
                    .all(|&id| g.object_view(id).zone == Zone::Graveyard)
            );
            assert_ne!(g.object_view(source).zone, Zone::Exile);
            g.restore(&suspended).unwrap();
        }
    }
}

#[test]
fn sphinx_synchronous_staged_failure_keeps_live_replay_until_caller_consumes_it() {
    use zone_replacements::{
        TEST_INVALIDATE_STAGED_COST, TEST_PREVENT_ZONE_MOVE, TEST_REDIRECT_EXILE_TO_BATTLEFIELD,
    };
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            TEST_PREVENT_ZONE_MOVE.with(|p| p.set(None));
            TEST_REDIRECT_EXILE_TO_BATTLEFIELD.with(|p| p.set(None));
            TEST_INVALIDATE_STAGED_COST.with(|p| p.set(None));
        }
    }
    let _reset = Reset;
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let cards: Vec<_> = (0..4)
            .map(|_| sphinx_card(&mut g, player, Zone::Graveyard))
            .collect();
        let source = sphinx_card(&mut g, player, Zone::Hand);
        cast(&mut g, source, None);
        wait_for_sphinx_choice(&mut g);
        let redirect = g.object_ref(cards[0]);
        TEST_PREVENT_ZONE_MOVE.with(|p| p.set(Some(redirect)));
        TEST_REDIRECT_EXILE_TO_BATTLEFIELD.with(|p| p.set(Some(redirect)));
        // Synthetic providers bypass normal card registration.
        g.state.zone_rules.get_or_insert_default();
        TEST_INVALIDATE_STAGED_COST.with(|p| p.set(Some(g.object_ref(cards[1]))));
        for &id in &cards {
            let object = g.object_ref(id);
            act(&mut g, Action::SelectResolutionExile { object });
        }
        assert!(matches!(g.state.pending, Pending::ZoneReplacement { player: p } if p == player));
        let original_refs = cards.iter().map(|&id| g.object_ref(id)).collect::<Vec<_>>();
        let saved = g.snapshot();
        for _ in 0..2 {
            act(
                &mut g,
                Action::ChooseZoneReplacement {
                    effect: ZoneReplacementKind::Battlefield,
                },
            );
            assert!(
                g.state
                    .zone_rules
                    .as_ref()
                    .is_none_or(|r| !r.has_checkpoint())
            );
            assert!(
                cards
                    .iter()
                    .all(|&id| g.object_view(id).zone == Zone::Graveyard)
            );
            assert_ne!(g.object_view(source).zone, Zone::Exile);
            assert_eq!(
                cards.iter().map(|&id| g.object_ref(id)).collect::<Vec<_>>(),
                original_refs
            );
            g.restore(&saved).unwrap();
        }
    }
}

#[test]
fn sphinx_preexisting_graveyard_exile_prohibition_skips_optional_payment() {
    use ability_world::TEST_EXILE_PROHIBITOR;
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            TEST_EXILE_PROHIBITOR.with(|p| p.set(None));
        }
    }
    let _reset = Reset;
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let provider = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
        let cards: Vec<_> = (0..4)
            .map(|_| sphinx_card(&mut game, player, Zone::Graveyard))
            .collect();
        let source = sphinx_card(&mut game, player, Zone::Hand);
        TEST_EXILE_PROHIBITOR.with(|p| p.set(Some(game.object_ref(provider))));
        cast(&mut game, source, None);
        let checkpoint = game.snapshot();
        for _ in 0..2 {
            for _ in 0..32 {
                assert!(!matches!(game.state.pending, Pending::EffectChoice { .. }));
                assert!(!matches!(game.state.pending, Pending::EffectSearch { .. }));
                if game.state.pending == Pending::Priority
                    && game.state.stack.is_empty()
                    && game.state.resolving.is_none()
                {
                    break;
                }
                act(&mut game, Action::Pass);
            }
            assert!(game.state.stack.is_empty() && game.state.resolving.is_none());
            assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 2);
            assert_eq!(game.object_view(source).zone, Zone::Graveyard);
            assert!(
                cards
                    .iter()
                    .all(|&id| game.object_view(id).zone == Zone::Graveyard)
            );
            game.restore(&checkpoint).unwrap();
        }
        TEST_EXILE_PROHIBITOR.with(|p| p.set(None));
    }
}
