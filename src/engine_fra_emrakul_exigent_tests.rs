use super::*;

#[test]
fn emrakul_cast_trigger_untaps_controlled_lands_even_after_spell_is_countered() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let land = put(&mut g, player, "Forest", Zone::Battlefield);
        let enemy = put(&mut g, player.other(), "Island", Zone::Battlefield);
        let creature = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        g.state.objects[land as usize].tapped = true;
        g.state.objects[enemy as usize].tapped = true;
        g.state.objects[creature as usize].tapped = true;
        let spell = put(&mut g, player, "Bear Cub", Zone::Hand);
        g.state.objects[spell as usize].card = Card::from_name("Emrakul, the Exigent Doom").unwrap();
        g.state.players[player.index()].mana = [0, 0, 0, 0, 0, 10];
        act(&mut g, Action::AnnounceCast { card: spell });
        pay(&mut g);
        g.prepare_triggers();
        let stack_id = g.state.stack.iter().find(|item| item.kind == StackKind::Spell && item.card == spell).unwrap().id;
        assert!(g.counter_stack_item(stack_id, false));
        let saved = g.snapshot();
        for _ in 0..2 {
            settle(&mut g);
            assert!(!g.object_view(land).tapped);
            assert!(g.object_view(enemy).tapped);
            assert!(g.object_view(creature).tapped);
            assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
            g.restore(&saved).unwrap();
        }
    }
}

#[test]
fn emrakul_cast_untap_uses_live_control_types_and_stun() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let stolen = put(&mut g, player.other(), "Forest", Zone::Battlefield);
        let blocked = put(&mut g, player, "Island", Zone::Battlefield);
        let stolen_ref = g.object_ref(stolen);
        g.change_controller(stolen_ref, player);
        g.state.objects[stolen as usize].tapped = true;
        g.state.objects[blocked as usize].tapped = true;
        g.state.objects[blocked as usize].counters.stun = 1;
        let spell = put(&mut g, player, "Bear Cub", Zone::Hand);
        g.state.objects[spell as usize].card = Card::from_name("Emrakul, the Exigent Doom").unwrap();
        g.state.players[player.index()].mana = [0, 0, 0, 0, 0, 10];
        act(&mut g, Action::AnnounceCast { card: spell });
        pay(&mut g);
        g.prepare_triggers();
        let saved = g.snapshot();
        for _ in 0..2 {
            settle(&mut g);
            assert!(!g.object_view(stolen).tapped);
            assert!(g.object_view(blocked).tapped);
            assert_eq!(g.counter_count(blocked, CounterKind::Stun), 0);
            assert_eq!(stats(&g, spell), (12, 12));
            assert!(g.keywords(spell).contains(Keywords::FLYING.union(Keywords::TRAMPLE)));
            g.restore(&saved).unwrap();
        }
    }
}

#[test]
fn emrakul_ward_sacrifice_three_is_selected_before_atomic_payment() {
    for player in [Player::P0, Player::P1] {
        for (pay_ward, redirect) in [(false, false), (true, false), (true, true)] {
            let mut g = scenario(player);
            if redirect { put(&mut g, player, "Rest in Peace", Zone::Battlefield); }
            let emrakul = put(&mut g, player, "Bear Cub", Zone::Battlefield);
            g.state.objects[emrakul as usize].card = Card::from_name("Emrakul, the Exigent Doom").unwrap();
            g.register_battlefield_sources(emrakul);
            let payer = player.other();
            let permanents: Vec<_> = (0..3).map(|_| put(&mut g, payer, "Forest", Zone::Battlefield)).collect();
            let shock = put(&mut g, payer, "Shock", Zone::Hand);
            g.state.active = payer;
            g.state.priority = payer;
            cast(&mut g, shock, Some(Target::Card(emrakul)));
            for _ in 0..48 {
                if matches!(g.state.pending, Pending::EffectChoice { player: p } if p == payer) { break; }
                choose(&mut g, |action| matches!(action, Action::Pass | Action::PutTrigger { .. }));
            }
            assert!(matches!(g.state.pending, Pending::EffectChoice { player: p } if p == payer));
            let saved = g.snapshot();
            for _ in 0..2 {
                if pay_ward {
                    for (i, id) in permanents.iter().copied().enumerate() {
                        let reference = g.object_ref(id);
                        act(&mut g, Action::Sacrifice { object: reference });
                        if i < 2 { assert!(permanents.iter().all(|&id| g.object_view(id).zone == Zone::Battlefield)); }
                    }
                } else {
                    let reference = g.object_ref(permanents[0]);
                    act(&mut g, Action::Sacrifice { object: reference });
                    act(&mut g, Action::DeclineEffectPayment);
                }
                settle(&mut g);
                for &id in &permanents {
                    assert_eq!(g.object_view(id).zone, if pay_ward { if redirect { Zone::Exile } else { Zone::Graveyard } } else { Zone::Battlefield });
                }
                assert_eq!(g.object_view(emrakul).damage, if pay_ward { 2 } else { 0 });
                g.restore(&saved).unwrap();
            }
        }
    }
}

#[test]
fn emrakul_hand_exile_grants_land_mana_until_that_exile_incarnation_is_cast() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let source = put(&mut g, player, "Bear Cub", Zone::Hand);
        g.state.objects[source as usize].card = Card::from_name("Emrakul, the Exigent Doom").unwrap();
        let land = put(&mut g, player.other(), "Forest", Zone::Battlefield);
        g.state.players[player.index()].mana = [0, 0, 0, 0, 0, 3];
        act(&mut g, Action::AnnounceActivate { card: source, ability: 0, counter_payment: Counters::default() });
        act(&mut g, Action::ChooseTarget { target: Target::Card(land) });
        pay(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Exile);
        let exile_ref = g.object_ref(source);
        settle(&mut g);
        assert!(g.can_play_exiled(player, source));
        let controller = g.object_ref(land);
        g.change_controller(controller, player);
        g.state.players[player.index()].mana = [0; 6];
        let action = Action::TapGrantedMana { card: land, grantor: exile_ref, production: [0, 0, 0, 0, 0, 2] };
        assert!(g.legal_actions().contains(&action));
        let saved = g.snapshot();
        for _ in 0..2 {
            act(&mut g, action.clone());
            assert_eq!(g.state.players[player.index()].mana, [0, 0, 0, 0, 0, 2]);
            g.state.objects[land as usize].tapped = false;
            g.state.players[player.index()].mana = [0, 0, 0, 0, 0, 10];
            act(&mut g, Action::AnnounceExile { card: source });
            pay(&mut g);
            assert!(!g.legal_actions().contains(&action));
            g.restore(&saved).unwrap();
        }
    }
}

#[test]
fn emrakul_cast_before_hand_ability_resolves_does_not_start_ended_grant() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        put(&mut g, player, "High Fae Trickster", Zone::Battlefield);
        let source = put(&mut g, player, "Bear Cub", Zone::Hand);
        g.state.objects[source as usize].card = Card::from_name("Emrakul, the Exigent Doom").unwrap();
        let land = put(&mut g, player, "Forest", Zone::Battlefield);
        g.state.players[player.index()].mana = [0, 0, 0, 0, 0, 13];
        act(&mut g, Action::AnnounceActivate { card: source, ability: 0, counter_payment: Counters::default() });
        act(&mut g, Action::ChooseTarget { target: Target::Card(land) });
        pay(&mut g);
        let exile_ref = g.object_ref(source);
        // Model a separate already-existing permission; the pending activation
        // has not yet created its own permission.
        g.grant_exile_play_permission(player, exile_ref, false);
        act(&mut g, Action::AnnounceExile { card: source });
        pay(&mut g);
        let saved = g.snapshot();
        for _ in 0..2 {
            settle(&mut g);
            assert!(!g.state.roles.as_ref().is_some_and(|roles| roles.source_linked_effects.iter().any(|row|
                row.source == exile_ref && row.kind == SourceLinkedEffectKind::GrantTwoColorlessUntilSourceCast)));
            assert!(!g.state.roles.as_ref().is_some_and(|roles| roles.exile_grant_cast_history.contains(&exile_ref)));
            g.restore(&saved).unwrap();
            assert!(g.state.roles.as_ref().is_some_and(|roles| roles.exile_grant_cast_history.contains(&exile_ref)));
        }
    }
}

#[test]
fn emrakul_exile_departure_and_reentry_does_not_rebind_old_grant() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let source = put(&mut g, player, "Bear Cub", Zone::Hand);
        g.state.objects[source as usize].card = Card::from_name("Emrakul, the Exigent Doom").unwrap();
        let land = put(&mut g, player, "Forest", Zone::Battlefield);
        g.state.players[player.index()].mana = [0, 0, 0, 0, 0, 3];
        act(&mut g, Action::AnnounceActivate { card: source, ability: 0, counter_payment: Counters::default() });
        act(&mut g, Action::ChooseTarget { target: Target::Card(land) });
        pay(&mut g);
        let old = g.object_ref(source);
        settle(&mut g);
        let saved = g.snapshot();
        for _ in 0..2 {
            g.move_object(source, Zone::Hand);
            g.move_object(source, Zone::Exile);
            let new = g.object_ref(source);
            assert_ne!(old, new);
            assert!(!g.can_play_exiled(player, source));
            let action = Action::TapGrantedMana { card: land, grantor: old, production: [0, 0, 0, 0, 0, 2] };
            assert!(g.legal_actions().contains(&action));
            g.grant_exile_play_permission(player, new, false);
            g.state.players[player.index()].mana = [0, 0, 0, 0, 0, 10];
            act(&mut g, Action::AnnounceExile { card: source });
            pay(&mut g);
            assert!(g.legal_actions().contains(&action));
            g.restore(&saved).unwrap();
        }
    }
}

#[test]
fn copied_emrakul_hand_ability_grants_each_controller_independent_permission() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let source = put(&mut g, player, "Bear Cub", Zone::Hand);
        g.state.objects[source as usize].card = Card::from_name("Emrakul, the Exigent Doom").unwrap();
        let land = put(&mut g, player, "Forest", Zone::Battlefield);
        g.state.players[player.index()].mana = [0, 0, 0, 0, 0, 3];
        act(&mut g, Action::AnnounceActivate { card: source, ability: 0, counter_payment: Counters::default() });
        act(&mut g, Action::ChooseTarget { target: Target::Card(land) });
        pay(&mut g);
        let exile_ref = g.object_ref(source);
        let original = g.state.stack.last().unwrap().clone();
        let mut copier = original.clone();
        copier.controller = player.other();
        copier.targets = vec![BoundTarget { group: 0, target: Target::Stack(original.id), object: None }];
        g.copy_target_ability(&copier, 0);
        if g.legal_actions().contains(&Action::FinishStackTargets) { act(&mut g, Action::FinishStackTargets); }
        let saved = g.snapshot();
        for _ in 0..2 {
            settle(&mut g);
            assert!(g.can_play_exiled(player, source));
            assert!(g.can_play_exiled(player.other(), source));
            let rows = &g.state.roles.as_ref().unwrap().source_linked_effects;
            assert_eq!(rows.iter().filter(|row| row.source == exile_ref
                && row.kind == SourceLinkedEffectKind::GrantTwoColorlessUntilSourceCast).count(), 2);
            g.restore(&saved).unwrap();
        }
    }
}

#[test]
fn emrakul_granted_mana_funding_respects_ability_layer_order_and_restore() {
    for player in [Player::P0, Player::P1] {
        for blank_before in [false, true] {
            let mut g = scenario(player);
            let source = put(&mut g, player, "Bear Cub", Zone::Hand);
            g.state.objects[source as usize].card = Card::from_name("Emrakul, the Exigent Doom").unwrap();
            let land = put(&mut g, player, "Forest", Zone::Battlefield);
            let mut blank = g.activation_item(player, land, 0);
            blank.targets = vec![BoundTarget { group: 0, target: Target::Card(land), object: Some(g.object_ref(land)) }];
            if blank_before { g.remove_target_abilities_until_end_of_turn(&blank); }
            g.state.players[player.index()].mana = [0, 0, 0, 0, 0, 3];
            act(&mut g, Action::AnnounceActivate { card: source, ability: 0, counter_payment: Counters::default() });
            act(&mut g, Action::ChooseTarget { target: Target::Card(land) });
            pay(&mut g);
            let exile_ref = g.object_ref(source);
            settle(&mut g);
            g.state.players[player.index()].mana = [0; 6];
            let action = Action::TapGrantedMana { card: land, grantor: exile_ref, production: [0, 0, 0, 0, 0, 2] };
            let goal = FundingGoal { actor: player, required: [0, 0, 0, 0, 0, 2], generic: 0, reserve: None, source: None };
            assert!(g.legal_actions().contains(&action));
            assert!(g.can_fund(goal.clone()));
            let saved = g.snapshot();
            for _ in 0..2 {
                g.remove_target_abilities_until_end_of_turn(&blank);
                assert!(!g.legal_actions().contains(&action));
                assert!(!g.can_fund(goal.clone()));
                g.restore(&saved).unwrap();
                assert!(g.legal_actions().contains(&action));
                assert!(g.can_fund(goal.clone()));
            }
        }
    }
}

#[test]
fn exile_cast_history_waits_for_retained_stack_packets_and_restores() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let id = put(&mut g, player, "Forest", Zone::Battlefield);
        let reference = g.object_ref(id);
        g.state.roles.get_or_insert_default().exile_grant_cast_history.push(reference);
        let packet = g.activation_item(player, id, 0);
        g.state.stack.push(packet);
        let saved = g.snapshot();
        for _ in 0..2 {
            g.prune_exile_grant_cast_history();
            assert!(g.state.roles.as_ref().unwrap().exile_grant_cast_history.contains(&reference));
            g.state.stack.clear();
            g.prune_exile_grant_cast_history();
            assert!(!g.state.roles.as_ref().unwrap().exile_grant_cast_history.contains(&reference));
            g.restore(&saved).unwrap();
        }
    }
}

#[test]
fn ward_sacrifice_prohibition_and_partial_decline_preserve_all_permanents() {
    use ability_world::{CharacteristicWorld, TEST_SACRIFICE_PROHIBITOR};
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) { TEST_SACRIFICE_PROHIBITOR.with(|p| p.set(None)); }
    }
    let _reset = Reset;
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let emrakul = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        g.state.objects[emrakul as usize].card = Card::from_name("Emrakul, the Exigent Doom").unwrap();
        g.register_battlefield_sources(emrakul);
        let payer = player.other();
        let permanents: Vec<_> = (0..3).map(|_| put(&mut g, payer, "Forest", Zone::Battlefield)).collect();
        let provider = put(&mut g, payer, "Bear Cub", Zone::Battlefield);
        let shock = put(&mut g, payer, "Shock", Zone::Hand);
        g.state.active = payer;
        g.state.priority = payer;
        cast(&mut g, shock, Some(Target::Card(emrakul)));
        for _ in 0..48 {
            if matches!(g.state.pending, Pending::EffectChoice { player: p } if p == payer) { break; }
            choose(&mut g, |action| matches!(action, Action::Pass | Action::PutTrigger { .. }));
        }
        let saved = g.snapshot();
        for _ in 0..2 {
            TEST_SACRIFICE_PROHIBITOR.with(|p| p.set(None));
            let first = g.object_ref(permanents[0]);
            act(&mut g, Action::Sacrifice { object: first });
            TEST_SACRIFICE_PROHIBITOR.with(|p| p.set(Some(g.object_ref(provider))));
            assert!(g.sacrifice_prohibited(permanents[0]));
            assert!(!g.legal_actions().iter().any(|a| matches!(a, Action::Sacrifice { .. })));
            act(&mut g, Action::DeclineEffectPayment);
            settle(&mut g);
            assert!(permanents.iter().all(|&id| g.object_view(id).zone == Zone::Battlefield));
            assert_eq!(g.object_view(emrakul).damage, 0);
            g.restore(&saved).unwrap();
        }
        TEST_SACRIFICE_PROHIBITOR.with(|p| p.set(None));
    }
}

#[test]
fn sacrifice_prohibition_blocks_real_mana_actions_and_funding() {
    use ability_world::TEST_SACRIFICE_PROHIBITOR;
    struct Reset;
    impl Drop for Reset { fn drop(&mut self) { TEST_SACRIFICE_PROHIBITOR.with(|p| p.set(None)); } }
    let _reset = Reset;
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let treasure = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        g.state.objects[treasure as usize].card = Card::test_index(5437);
        g.state.objects[treasure as usize].summoning_sick = false;
        let provider = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        let goal = FundingGoal { actor: player, required: [0, 0, 0, 0, 1, 0], generic: 0, reserve: None, source: None };
        let saved = g.snapshot();
        for _ in 0..2 {
            TEST_SACRIFICE_PROHIBITOR.with(|p| p.set(Some(g.object_ref(provider))));
            assert!(!g.can_fund(goal.clone()));
            assert!(!g.legal_actions().iter().any(|a| matches!(a, Action::ActivateMana { card, .. } if *card == treasure)));
            TEST_SACRIFICE_PROHIBITOR.with(|p| p.set(None));
            assert!(g.can_fund(goal.clone()));
            let action = g.legal_actions().into_iter().find(|a| matches!(a, Action::ActivateMana { card, production: [0, 0, 0, 0, 1, 0], .. } if *card == treasure)).unwrap();
            act(&mut g, action);
            assert_eq!(g.state.players[player.index()].mana[4], 1);
            assert_ne!(g.object_view(treasure).zone, Zone::Battlefield);
            g.restore(&saved).unwrap();
        }
    }
}

#[test]
fn ward_atomic_replacement_prevention_and_redirect_choice_restore() {
    use zone_replacements::TEST_PREVENT_ZONE_MOVE;
    struct Reset;
    impl Drop for Reset { fn drop(&mut self) { TEST_PREVENT_ZONE_MOVE.with(|p| p.set(None)); } }
    let _reset = Reset;
    for player in [Player::P0, Player::P1] {
        for prevent in [false, true] {
            let mut g = scenario(player);
            put(&mut g, player, "Rest in Peace", Zone::Battlefield);
            let emrakul = put(&mut g, player, "Bear Cub", Zone::Battlefield);
            g.state.objects[emrakul as usize].card = Card::from_name("Emrakul, the Exigent Doom").unwrap();
            g.register_battlefield_sources(emrakul);
            let payer = player.other();
            let permanents: Vec<_> = (0..3).map(|_| put(&mut g, payer, "Forest", Zone::Battlefield)).collect();
            let shock = put(&mut g, payer, "Shock", Zone::Hand);
            g.state.active = payer;
            g.state.priority = payer;
            cast(&mut g, shock, Some(Target::Card(emrakul)));
            for _ in 0..48 {
                if matches!(g.state.pending, Pending::EffectChoice { player: p } if p == payer) { break; }
                choose(&mut g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
            }
            let saved = g.snapshot();
            for _ in 0..2 {
                TEST_PREVENT_ZONE_MOVE.with(|p| p.set(Some(g.object_ref(permanents[1]))));
                for &id in &permanents {
                    let reference = g.object_ref(id);
                    act(&mut g, Action::Sacrifice { object: reference });
                }
                let selected = if prevent { ZoneReplacementKind::Prevent } else { ZoneReplacementKind::Exile };
                assert!(g.legal_actions().contains(&Action::ChooseZoneReplacement { effect: selected }));
                assert!(permanents.iter().all(|&id| g.object_view(id).zone == Zone::Battlefield));
                act(&mut g, Action::ChooseZoneReplacement { effect: selected });
                settle(&mut g);
                for &id in &permanents {
                    assert_eq!(g.object_view(id).zone, if prevent && id == permanents[1] { Zone::Battlefield } else { Zone::Exile });
                }
                assert_eq!(g.object_view(emrakul).damage, 2);
                g.restore(&saved).unwrap();
            }
            TEST_PREVENT_ZONE_MOVE.with(|p| p.set(None));
        }
    }
}

#[test]
fn atomic_zone_cost_preflight_preserves_each_prevented_replacement_without_movement() {
    use zone_replacements::TEST_PREVENT_ZONE_MOVE;
    struct Reset;
    impl Drop for Reset { fn drop(&mut self) { TEST_PREVENT_ZONE_MOVE.with(|p| p.set(None)); } }
    let _reset = Reset;
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let source = put(&mut g, player, "Bear Cub", Zone::Hand);
        let mut ids = vec![source];
        ids.extend((0..4).map(|_| put(&mut g, player, "Bear Cub", Zone::Graveyard)));
        let moves: Vec<_> = ids.iter().map(|&id| (g.object_ref(id), Zone::Exile)).collect();
        let saved = g.snapshot();
        for &id in &ids {
            TEST_PREVENT_ZONE_MOVE.with(|p| p.set(Some(g.object_ref(id))));
            let plans = g.plan_atomic_zone_cost(&moves).unwrap();
            assert_eq!(plans.iter().filter(|(_, plan)| plan.prevented).count(), 1);
            assert!(moves.iter().all(|&(reference, _)| g.object_ref(reference.card) == reference));
            assert_eq!(g.object_view(source).zone, Zone::Hand);
            assert!(ids[1..].iter().all(|&id| g.object_view(id).zone == Zone::Graveyard));
            g.restore(&saved).unwrap();
        }
        TEST_PREVENT_ZONE_MOVE.with(|p| p.set(Some(g.object_ref(ids[2]))));
        let plans = g.plan_atomic_zone_cost(&moves).unwrap();
        let events = g.commit_atomic_zone_cost(plans).unwrap();
        assert_eq!(events.iter().filter(|event| !event.moved).count(), 1);
        assert_eq!(g.object_view(ids[2]).zone, Zone::Graveyard);
        for &id in ids.iter().filter(|&&id| id != ids[2]) {
            assert_eq!(g.object_view(id).zone, Zone::Exile);
        }
        g.restore(&saved).unwrap();
        TEST_PREVENT_ZONE_MOVE.with(|p| p.set(None));
        assert!(g.plan_atomic_zone_cost(&moves).is_some());
    }
}

#[test]
fn replacement_prevented_sacrifice_mana_cost_is_paid_and_replays() {
    use zone_replacements::TEST_PREVENT_ZONE_MOVE;
    struct Reset;
    impl Drop for Reset { fn drop(&mut self) { TEST_PREVENT_ZONE_MOVE.with(|p| p.set(None)); } }
    let _reset = Reset;
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let treasure = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        g.state.objects[treasure as usize].card = Card::test_index(5437);
        g.state.objects[treasure as usize].summoning_sick = false;
        let reference = g.object_ref(treasure);
        TEST_PREVENT_ZONE_MOVE.with(|p| p.set(Some(reference)));
        let saved = g.snapshot();
        for _ in 0..2 {
            let action = g.legal_actions().into_iter().find(|a| matches!(a,
                Action::ActivateMana { card, production: [0, 0, 0, 0, 1, 0], .. } if *card == treasure)).unwrap();
            act(&mut g, action);
            assert_eq!(g.object_ref(treasure), reference);
            assert_eq!(g.object_view(treasure).zone, Zone::Battlefield);
            assert!(g.object_view(treasure).tapped);
            assert_eq!(g.state.players[player.index()].mana[4], 1);
            g.restore(&saved).unwrap();
        }
        TEST_PREVENT_ZONE_MOVE.with(|p| p.set(None));
    }
}
