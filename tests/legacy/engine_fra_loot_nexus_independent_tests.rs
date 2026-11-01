use super::*;

#[test]
fn loot_mana_distinguishes_negative_zero_and_repeated_power() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let loot = put(&mut g, p, "Loot, the Nexus", Zone::Battlefield);
        let minus = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let zero = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let same = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.state.objects[minus as usize].growth.power = -3;
        g.state.objects[zero as usize].growth.power = -2;
        g.state.objects[same as usize].growth.power = -3;
        g.state.objects[loot as usize].summoning_sick = false;
        g.state.players[p.index()].mana = [0; 6];
        let actions: Vec<_> = g
            .legal_actions()
            .into_iter()
            .filter(|a| {
                matches!(a,
            Action::ActivateMana { card, ability: 0, .. } if *card == loot)
            })
            .collect();
        assert_eq!(actions.len(), 5);
        assert!(
            actions
                .iter()
                .all(|a| matches!(a, Action::ActivateMana { production, .. }
            if production.iter().sum::<u32>() == 3))
        );
        let saved = g.snapshot();
        act(&mut g, actions[0].clone());
        assert_eq!(g.state.players[p.index()].mana.iter().sum::<u32>(), 3);
        g.restore(&saved).unwrap();
        act(&mut g, actions[0].clone());
        assert_eq!(g.state.players[p.index()].mana.iter().sum::<u32>(), 3);
    }
}

#[test]
fn loot_zero_creature_production_has_one_zero_mana_action() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        // Stand-in noncreature bearer for the same tapped inherited mana ability.
        let source = put(&mut g, p, "Territory Forge", Zone::Battlefield);
        let production = ManaProduction::DistinctCreaturePowers.in_world(&g, source);
        let mut choices = Vec::new();
        production.visit(|mana| {
            choices.push(mana);
            false
        });
        assert_eq!(choices, vec![[0; 6]], "zero mana is one distinct action");
    }
}
