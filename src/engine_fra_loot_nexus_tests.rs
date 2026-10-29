use super::*;

#[test]
fn loot_mana_counts_distinct_controlled_creature_powers_and_each_color() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let loot = put(&mut g, player, "Loot, the Nexus", Zone::Battlefield);
        let same = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        let other = put(&mut g, player, "Campus Crier", Zone::Battlefield);
        put(&mut g, player.other(), "Campus Crier", Zone::Battlefield);
        g.state.objects[loot as usize].summoning_sick = false;
        g.state.players[player.index()].mana = [0; 6];
        assert_eq!(stats(&g, loot).0, stats(&g, same).0);
        assert_ne!(stats(&g, loot).0, stats(&g, other).0);
        let actions: Vec<_> = g.legal_actions().into_iter().filter(|action| {
            matches!(action, Action::ActivateMana { card, ability: 0, .. } if *card == loot)
        }).collect();
        assert_eq!(actions.len(), 5);
        assert!(actions.iter().all(|action| matches!(action,
            Action::ActivateMana { production, .. } if production.iter().sum::<u32>() == 2 && production.iter().filter(|n| **n != 0).count() == 1)));
        let before = g.snapshot();
        act(&mut g, actions[0].clone());
        assert_eq!(g.state.players[player.index()].mana.iter().sum::<u32>(), 2);
        assert!(g.object_view(loot).tapped);
        g.restore(&before).unwrap();
        act(&mut g, actions[0].clone());
        assert_eq!(g.state.players[player.index()].mana.iter().sum::<u32>(), 2);
    }
}

#[test]
fn loot_production_rechecks_current_creature_control_and_departures() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let loot = put(&mut g, player, "Loot, the Nexus", Zone::Battlefield);
        let other = put(&mut g, player, "Campus Crier", Zone::Battlefield);
        assert_eq!(ManaProduction::DistinctCreaturePowers.in_world(&g, loot), ManaProduction::any_color(2));
        let object = g.object_ref(other);
        g.change_controller(object, player.other());
        assert_eq!(ManaProduction::DistinctCreaturePowers.in_world(&g, loot), ManaProduction::any_color(1));
        g.change_controller(object, player);
        g.move_object(other, Zone::Graveyard);
        assert_eq!(ManaProduction::DistinctCreaturePowers.in_world(&g, loot), ManaProduction::any_color(1));
    }
}

#[test]
fn loot_projected_mana_funds_a_spell_using_one_chosen_color() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let loot = put(&mut g, player, "Loot, the Nexus", Zone::Battlefield);
        put(&mut g, player, "Campus Crier", Zone::Battlefield);
        let spell = put(&mut g, player, "Bear Cub", Zone::Hand);
        g.state.objects[loot as usize].summoning_sick = false;
        g.state.players[player.index()].mana = [0; 6];
        assert!(g.legal_actions().contains(&Action::AnnounceCast { card: spell }));
        act(&mut g, Action::AnnounceCast { card: spell });
        let green = g.legal_actions().into_iter().find(|action| matches!(action,
            Action::ActivateMana { card, ability: 0, production: [0, 0, 0, 0, 2, 0], .. } if *card == loot)).unwrap();
        act(&mut g, green);
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(spell).zone, Zone::Battlefield);
    }
}

#[test]
fn loot_zero_creature_production_has_one_zero_mana_choice() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        // Loot's ability can be borrowed by a noncreature source.
        let source = put(&mut g, player, "Forest", Zone::Battlefield);
        let production = ManaProduction::DistinctCreaturePowers.in_world(&g, source);
        assert_eq!(production, ManaProduction::Fixed([0; 6]));
        let mut rows = Vec::new();
        production.visit(|mana| { rows.push(mana); false });
        assert_eq!(rows, vec![[0; 6]]);
    }
}
