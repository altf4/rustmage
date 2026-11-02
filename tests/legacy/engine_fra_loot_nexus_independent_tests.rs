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
        let offered = g.legal_actions();
        let actions: Vec<_> = offered
            .iter()
            .filter(|a| {
                matches!(a,
            Action::ActivateMana { card, ability: 0, production, .. }
                if *card == loot && *production != [0; 6])
            })
            .cloned()
            .collect();
        let late = offered
            .iter()
            .filter(|action| {
                matches!(action,
            Action::ActivateMana { card, ability: 0, production: [0, 0, 0, 0, 0, 0], .. }
                if *card == loot)
            })
            .count();
        assert_eq!(late, 1, "one unselected direct start");
        assert_eq!(
            offered
                .iter()
                .filter(|action| matches!(action,
            Action::AnnounceMana { card, ability: 0, production: [0, 0, 0, 0, 0, 0], .. }
                if *card == loot))
                .count(),
            1,
            "one unselected announcement"
        );
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

        g.restore(&saved).unwrap();
        act(
            &mut g,
            Action::AnnounceMana {
                card: loot,
                ability: 0,
                counter_payment: Counters::default(),
                production: [0; 6],
            },
        );
        pay(&mut g);
        assert!(g.object_view(loot).tapped);
        assert_eq!(
            g.state.players[p.index()].mana,
            [0; 6],
            "unselected output is not zero production"
        );
        assert!(matches!(g.state.pending, Pending::EffectMana { .. }));
        let choices = g.legal_actions();
        assert_eq!(
            choices
                .iter()
                .filter(|a| matches!(a, Action::ChooseMana { .. }))
                .count(),
            5
        );
        for color in 0..5 {
            let mut production = [0; 6];
            production[color] = 3;
            assert!(choices.contains(&Action::ChooseMana { production }));
        }
        let pending = g.snapshot();
        let choice = Action::ChooseMana {
            production: [0, 0, 3, 0, 0, 0],
        };
        act(&mut g, choice.clone());
        assert_eq!(g.state.players[p.index()].mana, [0, 0, 3, 0, 0, 0]);
        let completed = g.snapshot();
        g.restore(&pending).unwrap();
        act(&mut g, choice);
        assert_eq!(g.snapshot(), completed);
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
