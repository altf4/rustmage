use super::*;

fn veiled_fixture(g: &mut Game, player: Player) -> CardId {
    let id = put(g, player, "Bear Cub", Zone::Battlefield);
    g.state.objects[id as usize].card = Card::from_name("Garruk, Veiled Butcher").unwrap();
    g.state.objects[id as usize].counters.loyalty = 5;
    g.register_battlefield_sources(id);
    id
}

#[test]
fn veiled_opponents_creature_deaths_are_exiled_without_reflexive_trigger() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let source = veiled_fixture(&mut g, player);
        let ally = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        let enemy = put(&mut g, player.other(), "Bear Cub", Zone::Battlefield);
        let saved = g.snapshot();
        for _ in 0..2 {
            let before = g.death_trigger_sources();
            let events = g.move_objects_raw(&[ally, enemy], Zone::Graveyard);
            g.finish_zone_events(&events, &before);
            assert_eq!(g.object_view(ally).zone, Zone::Graveyard);
            assert_eq!(g.object_view(enemy).zone, Zone::Exile);
            assert!(!g.state.pending_triggers.iter().any(|item| item.card == source));
            g.restore(&saved).unwrap();
        }
    }
}

#[test]
fn veiled_death_replacement_follows_control_and_ability_removal() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let source = veiled_fixture(&mut g, player);
        let enemy = put(&mut g, player.other(), "Bear Cub", Zone::Battlefield);
        let saved = g.snapshot();
        let source_ref = g.object_ref(source);
        g.change_controller(source_ref, player.other());
        g.move_object(enemy, Zone::Graveyard);
        assert_eq!(g.object_view(enemy).zone, Zone::Graveyard);
        g.restore(&saved).unwrap();
        let mut item = g.activation_item(player, source, 0);
        item.targets = vec![BoundTarget { group: 0, target: Target::Card(source), object: Some(source_ref) }];
        g.remove_target_abilities_until_end_of_turn(&item);
        g.move_object(enemy, Zone::Graveyard);
        assert_eq!(g.object_view(enemy).zone, Zone::Graveyard);
    }
}

#[test]
fn veiled_sacrifice_choices_are_simultaneous_and_token_requires_own_sacrifice() {
    for player in [Player::P0, Player::P1] {
        for own_creature in [false, true] {
            let mut g = scenario(player);
            let source = veiled_fixture(&mut g, player);
            let ally = own_creature.then(|| put(&mut g, player, "Bear Cub", Zone::Battlefield));
            let enemy = put(&mut g, player.other(), "Bear Cub", Zone::Battlefield);
            act(&mut g, Action::AnnounceActivate { card: source, ability: 1, counter_payment: Counters::default() });
            pay(&mut g);
            act(&mut g, Action::Pass);
            act(&mut g, Action::Pass);
            let saved = g.snapshot();
            for _ in 0..2 {
                if let Some(ally) = ally {
                    let object = g.object_ref(ally);
                    act(&mut g, Action::Sacrifice { object });
                    assert_eq!(g.object_view(ally).zone, Zone::Battlefield);
                }
                let object = g.object_ref(enemy);
                act(&mut g, Action::Sacrifice { object });
                assert_eq!(g.object_view(enemy).zone, Zone::Exile);
                if let Some(ally) = ally { assert_eq!(g.object_view(ally).zone, Zone::Graveyard); }
                let count = g.state.objects.iter().filter(|o| o.zone == Zone::Battlefield
                    && o.controller == player && o.card == Card::test_index(5239)).count();
                assert_eq!(count, usize::from(own_creature));
                g.restore(&saved).unwrap();
            }
        }
    }
}

#[test]
fn veiled_stat_modifier_survives_source_and_cleanup_until_controllers_next_turn() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let source = veiled_fixture(&mut g, player);
        let target = put(&mut g, player.other(), "Hill Giant", Zone::Battlefield);
        act(&mut g, Action::AnnounceActivate { card: source, ability: 0, counter_payment: Counters::default() });
        act(&mut g, Action::ChooseTarget { target: Target::Card(target) });
        if g.legal_actions().contains(&Action::FinishTargets) { act(&mut g, Action::FinishTargets); }
        pay(&mut g);
        g.move_object(source, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(stats(&g, target), (-1, 2));
        for viewer in [player, player.other()] { assert_eq!(g.observe(viewer).until_next_turn_stats.len(), 1); }
        let saved = g.snapshot();
        for _ in 0..2 {
            g.finish_cleanup();
            assert_eq!(stats(&g, target), (-1, 2));
            g.state.active = player.other();
            g.begin_turn();
            assert_eq!(stats(&g, target), (-1, 2));
            g.state.active = player;
            g.begin_turn();
            assert_eq!(stats(&g, target), (3, 3));
            g.restore(&saved).unwrap();
        }
        g.move_object(target, Zone::Exile);
        g.move_object(target, Zone::Battlefield);
        assert_eq!(stats(&g, target), (3, 3));
    }
}

#[test]
fn veiled_draw_depends_on_actual_nonland_discards_including_replacement() {
    for player in [Player::P0, Player::P1] {
        for hand in [vec![], vec!["Shock"], vec!["Shock", "Forest"],
                     vec!["Shock", "Shock"], vec!["Wilt-Leaf Liege", "Shock"]] {
            let mut g = scenario(player);
            let source = veiled_fixture(&mut g, player);
            let cards = hand.iter().map(|name| put(&mut g, player.other(), name, Zone::Hand)).collect::<Vec<_>>();
            act(&mut g, Action::AnnounceActivate { card: source, ability: 2, counter_payment: Counters::default() });
            pay(&mut g);
            act(&mut g, Action::Pass);
            act(&mut g, Action::Pass);
            let saved = g.snapshot();
            for _ in 0..2 {
                for &card in &cards { act(&mut g, Action::Discard { card }); }
                settle(&mut g);
                let nonlands = hand.iter().filter(|name| **name != "Forest").count();
                assert_eq!(g.state.players[player.index()].cards_drawn_this_turn, u64::from(nonlands < 2));
                if hand.first() == Some(&"Wilt-Leaf Liege") {
                    assert_eq!(g.object_view(cards[0]).zone, Zone::Battlefield);
                }
                g.restore(&saved).unwrap();
            }
        }
    }
}

#[test]
fn veiled_plain_exile_and_reflexive_exile_are_distinct_replacement_choices() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        veiled_fixture(&mut g, player);
        let head = put(&mut g, player, "Head of the Hunt", Zone::Battlefield);
        let enemy = put(&mut g, player.other(), "Bear Cub", Zone::Battlefield);
        let shock = put(&mut g, player, "Shock", Zone::Hand);
        cast(&mut g, shock, Some(Target::Card(enemy)));
        for _ in 0..24 {
            if matches!(g.state.pending, Pending::ZoneReplacement { .. }) { break; }
            act(&mut g, Action::Pass);
        }
        assert!(matches!(g.state.pending, Pending::ZoneReplacement { player: actor } if actor == player.other()));
        let head_ref = g.object_ref(head);
        let saved = g.snapshot();
        for use_head in [false, true] {
            g.restore(&saved).unwrap();
            let effect = if use_head {
                ZoneReplacementKind::ExileReflexive { source: head_ref, ability: 0 }
            } else { ZoneReplacementKind::Exile };
            act(&mut g, Action::ChooseZoneReplacement { effect });
            settle(&mut g);
            assert_eq!(g.object_view(enemy).zone, Zone::Exile);
            let tokens = g.state.objects.iter().filter(|o| o.zone == Zone::Battlefield
                && o.controller == player && o.card == Card::test_index(5412)).count();
            assert_eq!(tokens, usize::from(use_head));
        }
    }
}

#[test]
fn veiled_discard_draw_waits_for_replaced_miracle_draw_before_finishing() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let source = veiled_fixture(&mut g, player);
        put(&mut g, player, "Lorehold, the Historian", Zone::Battlefield);
        put(&mut g, player, "Bard, King of Dale", Zone::Battlefield);
        put(&mut g, player, "Island", Zone::Library);
        let miracle = put(&mut g, player, "Group Project", Zone::Library);
        act(&mut g, Action::AnnounceActivate { card: source, ability: 2, counter_payment: Counters::default() });
        pay(&mut g);
        for _ in 0..32 {
            if matches!(g.state.pending, Pending::MiracleReveal { .. }) { break; }
            act(&mut g, Action::Pass);
        }
        assert!(matches!(g.state.pending, Pending::MiracleReveal { player: actor } if actor == player));
        assert!(g.state.resolving.is_some());
        assert_eq!(g.observe(player).miracle.unwrap().object.card, miracle);
        assert!(g.observe(player.other()).miracle.is_none());
        let saved = g.snapshot();
        for _ in 0..2 {
            act(&mut g, Action::DeclineMiracle);
            settle(&mut g);
            assert!(g.state.resolving.is_none());
            assert_eq!(g.state.players[player.index()].cards_drawn_this_turn, 2);
            assert_eq!(g.object_view(miracle).zone, Zone::Hand);
            g.restore(&saved).unwrap();
        }
    }
}

#[test]
fn veiled_copied_modifier_uses_each_controller_and_stacks_with_base_set_and_blanking() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let source = veiled_fixture(&mut g, player);
        let target = put(&mut g, player.other(), "Hill Giant", Zone::Battlefield);
        act(&mut g, Action::AnnounceActivate { card: source, ability: 0, counter_payment: Counters::default() });
        act(&mut g, Action::ChooseTarget { target: Target::Card(target) });
        if g.legal_actions().contains(&Action::FinishTargets) { act(&mut g, Action::FinishTargets); }
        pay(&mut g);
        let original = g.state.stack.last().unwrap().clone();
        let mut copier = original.clone();
        copier.controller = player.other();
        copier.targets = vec![BoundTarget { group: 0, target: Target::Stack(original.id), object: None }];
        g.copy_target_ability(&copier, 0);
        g.move_object(source, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(stats(&g, target), (-5, 1));
        let mut item = g.activation_item(player, target, 0);
        item.targets = vec![BoundTarget { group: 0, target: Target::Card(target), object: Some(g.object_ref(target)) }];
        g.set_base_stats(&item, Subject::Targets, Some(10), Some(10));
        g.remove_target_abilities_until_end_of_turn(&item);
        assert_eq!(stats(&g, target), (2, 8));
        let saved = g.snapshot();
        for _ in 0..2 {
            g.state.active = player.other();
            g.begin_turn();
            assert_eq!(stats(&g, target), (6, 9));
            g.state.active = player;
            g.begin_turn();
            assert_eq!(stats(&g, target), (10, 10));
            g.restore(&saved).unwrap();
        }
    }
}

#[test]
fn veiled_modifier_expires_at_actual_extra_turn_not_skipped_turn() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let source = veiled_fixture(&mut g, player);
        let target = put(&mut g, player.other(), "Hill Giant", Zone::Battlefield);
        act(&mut g, Action::AnnounceActivate { card: source, ability: 0, counter_payment: Counters::default() });
        act(&mut g, Action::ChooseTarget { target: Target::Card(target) });
        if g.legal_actions().contains(&Action::FinishTargets) { act(&mut g, Action::FinishTargets); }
        pay(&mut g);
        settle(&mut g);
        let saved = g.snapshot();
        for _ in 0..2 {
            g.add_extra_turn(player.other(), false);
            g.advance_to_actual_turn();
            g.begin_turn();
            assert_eq!(stats(&g, target), (-1, 2));
            let mut skips = [0; 2];
            skips[player.index()] = 1;
            g.state.turn_skips = Some(Box::new(skips));
            g.advance_to_actual_turn();
            g.begin_turn();
            assert_ne!(g.state.active, player);
            assert_eq!(stats(&g, target), (-1, 2));
            g.add_extra_turn(player, false);
            g.advance_to_actual_turn();
            g.begin_turn();
            assert_eq!(g.state.active, player);
            assert_eq!(stats(&g, target), (3, 3));
            g.restore(&saved).unwrap();
        }
    }
}
