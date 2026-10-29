//! Committed loyalty activation witnesses, independent of resolution.
use super::*;

#[test]
fn gideon_observes_only_opponents_committed_loyalty_abilities() {
    for actor in [Player::P0, Player::P1] {
        for enemy in [false, true] {
            let mut game = scenario(actor);
            game.create_tokens(actor, Card::JacePlaneswalkerToken, 1, false);
            let walker = game.state.objects.iter().enumerate().find_map(|(id, object)| {
                (object.zone == Zone::Battlefield && object.controller == actor && object.is_token()
                    && game.has_subtype(id as CardId, "Jace")).then_some(id as CardId)
            }).unwrap();
            game.state.objects[walker as usize].counters.loyalty = 5;
            let observer = if enemy { actor.other() } else { actor };
            let gideon = put(&mut game, observer, "Gideon the Oathless", Zone::Battlefield);
            let life = game.state.players[actor.index()].life;
            act(&mut game, Action::AnnounceActivate { card: walker, ability: 1, counter_payment: Counters::default() });
            pay(&mut game);
            let triggers = game.state.stack.iter().filter(|item| {
                item.card == gideon && matches!(item.kind, StackKind::TriggeredAbility { .. })
            }).count();
            assert_eq!(triggers, usize::from(enemy));
            if enemy {
                game.move_object(gideon, Zone::Graveyard);
                settle(&mut game);
                assert_eq!(game.state.players[actor.index()].life, life - 1);
            }
        }
    }
}

#[test]
fn gideon_ward_retains_the_targeting_player_after_control_changes() {
    for payer in [Player::P0, Player::P1] {
        for discard in [false, true] {
            let mut game = scenario(payer);
            let gideon = put(&mut game, payer.other(), "Gideon the Oathless", Zone::Battlefield);
            let shock = put(&mut game, payer, "Shock", Zone::Hand);
            let secret = put(&mut game, payer, "Forest", Zone::Hand);
            cast(&mut game, shock, Some(Target::Card(gideon)));
            game.change_controller(game.object_ref(gideon), payer);
            game.resolve_top();
            assert!(matches!(game.state.pending, Pending::ResolutionMana { player } if player == payer));
            let request = game.observe(payer).resolution_payment.unwrap();
            assert!(request.discard_only);
            assert_eq!(request.payer, payer);
            assert!(!game.legal_actions().iter().any(|a| matches!(a, Action::PayResolutionMana { .. })));
            assert!(!game.observe(payer.other()).objects.iter().any(|o| o.id == secret));
            super::super::contracts::branch_contract(game.clone());
            if discard {
                act(&mut game, Action::Discard { card: secret });
            } else {
                act(&mut game, Action::DeclineResolutionMana);
            }
            settle(&mut game);
            assert_eq!(game.object_view(secret).zone, if discard { Zone::Graveyard } else { Zone::Hand });
            assert_eq!(game.state.objects[gideon as usize].damage, if discard { 2 } else { 0 });
            assert_eq!(game.object_view(shock).zone, Zone::Graveyard);
        }
    }
}

#[test]
fn gideon_ward_with_empty_hand_can_only_be_declined() {
    for payer in [Player::P0, Player::P1] {
        let mut game = scenario(payer);
        let gideon = put(&mut game, payer.other(), "Gideon the Oathless", Zone::Battlefield);
        let shock = put(&mut game, payer, "Shock", Zone::Hand);
        cast(&mut game, shock, Some(Target::Card(gideon)));
        game.resolve_top();
        assert_eq!(game.actor(), Some(payer));
        assert!(!game.legal_actions().iter().any(|a| matches!(a, Action::Discard { .. } | Action::PayResolutionMana { .. })));
        act(&mut game, Action::DeclineResolutionMana);
        settle(&mut game);
        assert_eq!(game.state.objects[gideon as usize].damage, 0);
        assert_eq!(game.object_view(shock).zone, Zone::Graveyard);
    }
}

#[test]
fn gideon_ward_offers_captured_payer_choice_after_related_spell_leaves() {
    for payer in [Player::P0, Player::P1] {
        for discard in [false, true] {
            let mut game = scenario(payer);
            let gideon = put(&mut game, payer.other(), "Gideon the Oathless", Zone::Battlefield);
            let shock = put(&mut game, payer, "Shock", Zone::Hand);
            let secret = put(&mut game, payer, "Forest", Zone::Hand);
            cast(&mut game, shock, Some(Target::Card(gideon)));
            let id = game.state.stack.iter().find(|s| s.card == shock).unwrap().id;
            assert!(game.counter_stack_item(id, false));
            game.resolve_top();
            assert!(matches!(game.state.pending, Pending::ResolutionMana { player } if player == payer));
            assert!(!game.observe(payer.other()).objects.iter().any(|o| o.id == secret));
            if discard { act(&mut game, Action::Discard { card: secret }); }
            else { act(&mut game, Action::DeclineResolutionMana); }
            settle(&mut game);
            assert_eq!(game.object_view(secret).zone, if discard { Zone::Graveyard } else { Zone::Hand });
        }
    }
}

#[test]
fn gideon_ward_captures_the_spell_controller_at_targeting() {
    for payer in [Player::P0, Player::P1] {
        let mut game = scenario(payer);
        let gideon = put(&mut game, payer.other(), "Gideon the Oathless", Zone::Battlefield);
        let shock = put(&mut game, payer, "Shock", Zone::Hand);
        let discard = put(&mut game, payer, "Forest", Zone::Hand);
        cast(&mut game, shock, Some(Target::Card(gideon)));
        game.state.stack.iter_mut().find(|s| s.card == shock).unwrap().controller = payer.other();
        game.resolve_top();
        assert_eq!(game.actor(), Some(payer));
        assert!(game.legal_actions().contains(&Action::Discard { card: discard }));
        act(&mut game, Action::Discard { card: discard });
        assert_eq!(game.object_view(discard).zone, Zone::Graveyard);
    }
}

#[test]
fn gideon_ward_discard_is_not_an_opponents_discard_effect() {
    for payer in [Player::P0, Player::P1] {
        let mut game = scenario(payer);
        let gideon = put(&mut game, payer.other(), "Gideon the Oathless", Zone::Battlefield);
        let shock = put(&mut game, payer, "Shock", Zone::Hand);
        let liege = put(&mut game, payer, "Wilt-Leaf Liege", Zone::Hand);
        cast(&mut game, shock, Some(Target::Card(gideon)));
        game.resolve_top();
        act(&mut game, Action::Discard { card: liege });
        assert_eq!(game.object_view(liege).zone, Zone::Graveyard);
    }
}

fn gideon_finish_events(g: &mut Game) {
    for _ in 0..96 {
        g.prepare_triggers();
        if g.state.pending == Pending::Priority && g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty() && g.state.resolving.is_none() { return; }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("Gideon triggers did not finish: {:?}", g.state.pending);
}

#[test]
fn gideon_observes_each_opponent_creature_entry_and_keeps_controller_on_source_departure() {
    for actor in [Player::P0, Player::P1] {
        let mut g = scenario(actor);
        let gideon = put(&mut g, actor.other(), "Gideon the Oathless", Zone::Battlefield);
        let life = g.state.players[actor.index()].life;
        let own = put(&mut g, actor, "Bear Cub", Zone::Hand);
        cast(&mut g, own, None);
        gideon_finish_events(&mut g);
        assert_eq!(g.state.players[actor.index()].life, life - 1);
        let before = g.state.players[actor.index()].life;
        g.create_tokens(actor, Card::test_index(5298), 2, false);
        let packets = g.state.stack.iter().chain(g.state.pending_triggers.iter()).chain(g.state.ordering_triggers.iter())
            .filter(|s| s.card == gideon && matches!(s.kind, StackKind::TriggeredAbility { .. })).count();
        assert_eq!(packets, 2);
        g.move_object(gideon, Zone::Graveyard);
        let saved = g.snapshot();
        for _ in 0..2 {
            gideon_finish_events(&mut g);
            assert_eq!(g.state.players[actor.index()].life, before - 2);
            g.restore(&saved).unwrap();
        }
    }
}

#[test]
fn gideon_does_not_observe_failed_loyalty_cost_or_own_creature_entries() {
    for actor in [Player::P0, Player::P1] {
        let mut g = scenario(actor);
        let gideon = put(&mut g, actor, "Gideon the Oathless", Zone::Battlefield);
        g.create_tokens(actor, Card::test_index(5298), 1, false);
        assert!(!g.state.pending_triggers.iter().any(|s| s.card == gideon));
        g.create_tokens(actor.other(), Card::JacePlaneswalkerToken, 1, false);
        let walker = g.state.objects.iter().enumerate().find_map(|(id, o)|
            (o.zone == Zone::Battlefield && o.controller == actor.other()
                && o.is_token() && g.has_subtype(id as CardId, "Jace")).then_some(id as CardId)).unwrap();
        g.state.objects[walker as usize].counters.loyalty = 1;
        g.state.active = actor.other();
        g.state.priority = actor.other();
        let saved = g.snapshot();
        reject(&mut g, Action::AnnounceActivate { card: walker, ability: 1, counter_payment: Counters::default() });
        assert_eq!(g.snapshot(), saved);
        assert!(!g.state.pending_triggers.iter().any(|s| s.card == gideon));
    }
}

#[test]
fn copied_gideon_triggers_keep_the_event_player() {
    for actor in [Player::P0, Player::P1] {
        for loyalty in [false, true] {
            let mut g = scenario(actor);
            let gideon = put(&mut g, actor.other(), "Gideon the Oathless", Zone::Battlefield);
            let life = g.state.players[actor.index()].life;
            let other_life = g.state.players[actor.other().index()].life;
            if loyalty {
                g.create_tokens(actor, Card::JacePlaneswalkerToken, 1, false);
                let walker = g.state.objects.iter().enumerate().find_map(|(id, object)|
                    (object.zone == Zone::Battlefield && object.controller == actor && object.is_token()
                        && g.has_subtype(id as CardId, "Jace")).then_some(id as CardId)).unwrap();
                // Ignore the Jace entry trigger to isolate the loyalty trigger.
                gideon_finish_events(&mut g);
                g.state.players[actor.index()].life = life;
                g.state.objects[walker as usize].counters.loyalty = 5;
                act(&mut g, Action::AnnounceActivate { card: walker, ability: 1, counter_payment: Counters::default() });
                pay(&mut g);
            } else {
                g.create_tokens(actor, Card::test_index(5298), 1, false);
                g.prepare_triggers();
            }
            let trigger = g.state.stack.iter().find(|item| item.card == gideon
                && matches!(item.kind, StackKind::TriggeredAbility { .. })).unwrap().clone();
            assert_eq!(trigger.event_player, Some(actor));
            let mut copier = trigger.clone();
            copier.controller = actor;
            copier.targets = vec![BoundTarget { group: 0, target: Target::Stack(trigger.id), object: None }];
            g.copy_target_ability(&copier, 0);
            assert!(g.state.stack.iter().any(|item| item.is_copy && item.controller == actor
                && item.event_player == Some(actor)));
            gideon_finish_events(&mut g);
            assert_eq!(g.state.players[actor.index()].life, life - 2);
            assert_eq!(g.state.players[actor.other().index()].life, other_life);
        }
    }
}
