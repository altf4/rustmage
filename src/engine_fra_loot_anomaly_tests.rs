use super::*;

fn loot(g: &mut Game, player: Player) -> CardId {
    let id = put(g, player, "Bear Cub", Zone::Battlefield);
    g.state.objects[id as usize].card = Card::from_name("Loot, the Anomaly").unwrap();
    g.register_battlefield_sources(id);
    id
}

#[test]
fn loot_negative_power_assigns_combat_damage_without_changing_power() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let source = loot(&mut g, player);
        assert_eq!(stats(&g, source), (-2, 4));
        assert_eq!(g.combat_assignment_amount(source), 2);
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        act(&mut g, Action::Attack { card: source });
        act(&mut g, Action::FinishAttackers);
        for _ in 0..32 {
            if g.state.pending == Pending::Blockers { break; }
            act(&mut g, Action::Pass);
        }
        act(&mut g, Action::FinishBlockers);
        let saved = g.snapshot();
        for _ in 0..2 {
            for _ in 0..32 {
                if g.state.players[player.other().index()].life == 18 { break; }
                act(&mut g, Action::Pass);
            }
            assert_eq!(g.state.players[player.other().index()].life, 18);
            assert_eq!(stats(&g, source), (-2, 4));
            g.restore(&saved).unwrap();
        }
    }
}

#[test]
fn loot_toughness_assignment_precedes_negative_power_and_blanking_removes_rule() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let source = loot(&mut g, player);
        g.state.objects[source as usize].growth.power = -6;
        assert_eq!(stats(&g, source), (-8, 4));
        assert_eq!(g.combat_assignment_amount(source), 8);
        put(&mut g, player, "Ghalta the Immovable", Zone::Battlefield);
        assert_eq!(g.combat_assignment_amount(source), 4);
        let copy = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        let object = g.object_ref(copy);
        g.apply_temporary_copy(object, CopyRecipe::new(Card::from_name("Loot, the Anomaly").unwrap()), CopyDeadline::EndOfTurn);
        assert_eq!(g.combat_assignment_amount(copy), 4);
        let saved = g.snapshot();
        let mut item = g.activation_item(player, source, 0);
        item.targets = vec![BoundTarget { group: 0, target: Target::Card(source), object: Some(g.object_ref(source)) }];
        g.remove_target_abilities_until_end_of_turn(&item);
        // Ghalta's independent rule continues to apply after Loot is blanked.
        assert_eq!(g.combat_assignment_amount(source), 4);
        g.restore(&saved).unwrap();
    }
}

#[test]
fn loot_threshold_cost_excludes_self_and_accepts_creature_or_planeswalker() {
    for player in [Player::P0, Player::P1] {
        for planeswalker in [false, true] {
            let mut g = scenario(player);
            let source = loot(&mut g, player);
            let fodder = put(&mut g, player, if planeswalker { "Ajani, Caller of the Pride" } else { "Bear Cub" }, Zone::Battlefield);
            if planeswalker { g.state.objects[fodder as usize].counters.loyalty = 3; }
            let enemy = put(&mut g, player.other(), "Bear Cub", Zone::Battlefield);
            for _ in 0..6 { put(&mut g, player, "Forest", Zone::Graveyard); }
            g.state.active = player.other();
            g.state.priority = player;
            g.state.step = Step::BeginCombat;
            let action = Action::AnnounceActivate { card: source, ability: 0, counter_payment: Counters::default() };
            reject(&mut g, action.clone());
            put(&mut g, player, "Forest", Zone::Graveyard);
            act(&mut g, action);
            let self_ref = g.object_ref(source);
            let enemy_ref = g.object_ref(enemy);
            reject(&mut g, Action::ChooseAdditionalCost { object: self_ref });
            reject(&mut g, Action::ChooseAdditionalCost { object: enemy_ref });
            let saved = g.snapshot();
            for _ in 0..2 {
                let choice = g.object_ref(fodder);
                act(&mut g, Action::ChooseAdditionalCost { object: choice });
                pay(&mut g);
                assert_eq!(g.object_view(fodder).zone, Zone::Graveyard);
                settle(&mut g);
                assert_eq!(stats(&g, source), (-4, 4));
                assert_eq!(g.combat_assignment_amount(source), 4);
                g.restore(&saved).unwrap();
            }
        }
    }
}

#[test]
fn loot_rule_tracks_blanking_copy_and_power_sign() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let source = loot(&mut g, player);
        let other = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        assert_eq!(g.combat_assignment_amount(source), 2);
        let mut item = g.activation_item(player, source, 0);
        item.targets = vec![BoundTarget { group: 0, target: Target::Card(source), object: Some(g.object_ref(source)) }];
        let saved = g.snapshot();
        g.remove_target_abilities_until_end_of_turn(&item);
        assert_eq!(g.combat_assignment_amount(source), 0);
        g.restore(&saved).unwrap();
        g.state.objects[source as usize].growth.power = 2;
        assert_eq!(g.combat_assignment_amount(source), 0);
        g.state.objects[source as usize].growth.power = 5;
        assert_eq!(g.combat_assignment_amount(source), 3);
        let object = g.object_ref(other);
        g.apply_temporary_copy(object, CopyRecipe::new(Card::from_name("Loot, the Anomaly").unwrap()), CopyDeadline::EndOfTurn);
        assert_eq!(g.combat_assignment_amount(other), 2);
    }
}

#[test]
fn loot_sacrifice_replacement_pays_and_queued_effect_tracks_incarnation() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let source = loot(&mut g, player);
        for _ in 0..7 { put(&mut g, player, "Forest", Zone::Graveyard); }
        let fodder = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        put(&mut g, player, "Rest in Peace", Zone::Battlefield);
        let action = Action::AnnounceActivate { card: source, ability: 0, counter_payment: Counters::default() };
        act(&mut g, action);
        let object = g.object_ref(fodder);
        act(&mut g, Action::ChooseAdditionalCost { object });
        pay(&mut g);
        assert_eq!(g.object_view(fodder).zone, Zone::Exile);
        let saved = g.snapshot();
        for reenter in [false, true] {
            if reenter {
                g.move_object(source, Zone::Exile);
                g.move_object(source, Zone::Battlefield);
            }
            settle(&mut g);
            assert_eq!(stats(&g, source), (if reenter { -2 } else { -4 }, 4));
            g.restore(&saved).unwrap();
        }
    }
}
