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
