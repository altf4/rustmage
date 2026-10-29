//! Live assignment and defender permissions from a controlled source.
use super::*;

#[test]
fn ghalta_grants_live_assignment_and_defender_permission() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let wall = put(&mut game, player, "Monoist Sentry", Zone::Battlefield);
        let enemy_wall = put(&mut game, player.other(), "Monoist Sentry", Zone::Battlefield);
        game.state.objects[wall as usize].growth.toughness = 7;
        game.state.objects[enemy_wall as usize].growth.toughness = 7;
        assert!(!game.can_declare_attacker(player, wall));
        assert_eq!(game.combat_assignment_amount(wall), 4);
        let source = put(&mut game, player, "Ghalta the Immovable", Zone::Battlefield);
        assert!(game.can_declare_attacker(player, wall));
        assert_eq!(game.combat_assignment_amount(wall), 8);
        assert_eq!(game.combat_assignment_amount(enemy_wall), 4);
        assert_eq!(game.stats(wall).power, 4);
        let aura = put(&mut game, player.other(), "Fresh Start", Zone::Battlefield);
        game.state.objects[aura as usize].attached_to = Some(game.object_ref(source));
        assert!(!game.can_declare_attacker(player, wall));
        assert_eq!(game.combat_assignment_amount(wall), 4);
        game.move_object(aura, Zone::Graveyard);
        assert_eq!(game.combat_assignment_amount(wall), 8);
        game.change_controller(game.object_ref(source), player.other());
        assert!(!game.can_declare_attacker(player, wall));
        assert_eq!(game.combat_assignment_amount(wall), 4);
        assert!(game.can_declare_attacker(player.other(), enemy_wall));
        assert_eq!(game.combat_assignment_amount(enemy_wall), 8);
        super::super::contracts::branch_contract(game);
    }
}

#[test]
fn ghalta_cost_reduction_uses_current_controlled_creature_toughness() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let spell = put(&mut game, player, "Ghalta the Immovable", Zone::Hand);
        assert_eq!(game.spell_reduction(player, spell), 0);
        let creature = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let enemy = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
        game.state.objects[enemy as usize].growth.toughness = 20;
        assert_eq!(game.spell_reduction(player, spell), 2);
        game.state.objects[creature as usize].growth.toughness = 5;
        assert_eq!(game.spell_reduction(player, spell), 7);
        game.change_controller(game.object_ref(creature), player.other());
        assert_eq!(game.spell_reduction(player, spell), 0);
    }
}

#[test]
fn ghalta_enables_defender_to_attack_and_deal_actual_toughness_damage() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        put(&mut game, player, "Ghalta the Immovable", Zone::Battlefield);
        let wall = put(&mut game, player, "Monoist Sentry", Zone::Battlefield);
        game.state.objects[wall as usize].growth.toughness = 7;
        game.state.step = Step::Attackers;
        game.state.pending = Pending::Attackers;
        act(&mut game, Action::Attack { card: wall });
        act(&mut game, Action::FinishAttackers);
        for _ in 0..32 {
            if game.state.pending == Pending::Blockers { break; }
            act(&mut game, Action::Pass);
        }
        assert_eq!(game.state.pending, Pending::Blockers);
        act(&mut game, Action::FinishBlockers);
        for _ in 0..32 {
            if game.state.players[player.other().index()].life == 12 { break; }
            act(&mut game, Action::Pass);
        }
        assert_eq!(game.state.players[player.other().index()].life, 12);
        assert_eq!(game.stats(wall).power, 4);
    }
}

#[test]
fn unrelated_static_sources_do_not_grant_combat_rules() {
    for player in [Player::P0, Player::P1] {
        for evaluate in [false, true] {
            let mut game = scenario(player);
            put(&mut game, player, "Spelunking", Zone::Battlefield);
            let wall = put(&mut game, player, "Monoist Sentry", Zone::Battlefield);
            game.state.objects[wall as usize].growth.toughness = 7;
            if evaluate {
                let bear = put(&mut game, player, "Bear Cub", Zone::Battlefield);
                let aura = put(&mut game, player.other(), "Fresh Start", Zone::Battlefield);
                game.state.objects[aura as usize].attached_to = Some(game.object_ref(bear));
            }
            assert!(!game.can_declare_attacker(player, wall));
            assert_eq!(game.combat_assignment_amount(wall), 4);
        }
    }
}

#[test]
fn copying_and_losing_ghalta_acquires_and_removes_combat_rules() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let bearer = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let wall = put(&mut game, player, "Monoist Sentry", Zone::Battlefield);
        game.state.objects[wall as usize].growth.toughness = 7;
        let reference = game.object_ref(bearer);
        assert!(!game.can_declare_attacker(player, wall));
        game.apply_temporary_copy(reference, CopyRecipe::new(Card::from_name("Ghalta the Immovable").unwrap()), CopyDeadline::EndOfTurn);
        assert!(game.can_declare_attacker(player, wall));
        assert_eq!(game.combat_assignment_amount(wall), 8);
        game.apply_temporary_copy(reference, CopyRecipe::new(Card::from_name("Bear Cub").unwrap()), CopyDeadline::EndOfTurn);
        assert!(!game.can_declare_attacker(player, wall));
        assert_eq!(game.combat_assignment_amount(wall), 4);
    }
}
