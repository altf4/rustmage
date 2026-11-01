//! Focused printed outcomes; shared engine invariants live in core tests.
//! Live assignment and defender permissions from a controlled source.
use super::*;

#[test]
fn ghalta_grants_live_assignment_and_defender_permission() {
    {
        let player = Player::P0;
        let mut game = scenario(player);
        let wall = put(&mut game, player, "Monoist Sentry", Zone::Battlefield);
        let enemy_wall = put(
            &mut game,
            player.other(),
            "Monoist Sentry",
            Zone::Battlefield,
        );
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
    }
}

#[test]
fn ghalta_cost_reduction_uses_current_controlled_creature_toughness() {
    {
        let player = Player::P0;
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
fn unrelated_static_sources_do_not_grant_combat_rules() {
    {
        let player = Player::P0;
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
fn ghalta_does_not_replace_power_for_noncombat_damage() {
    {
        let player = Player::P0;
        let mut game = scenario(player);
        put(&mut game, player, "Ghalta the Immovable", Zone::Battlefield);
        let attacker = put(&mut game, player, "Monoist Sentry", Zone::Battlefield);
        let enemy = put(
            &mut game,
            player.other(),
            "Monoist Sentry",
            Zone::Battlefield,
        );
        game.state.objects[attacker as usize].growth.toughness = 7;
        game.state.objects[enemy as usize].growth.toughness = 7;
        let spell = put(&mut game, player, "Bite Down", Zone::Hand);
        big_helpers::cast(
            &mut game,
            spell,
            None,
            &[Target::Card(attacker), Target::Card(enemy)],
        );
        settle(&mut game);
        assert_eq!(game.state.objects[enemy as usize].damage, 4);
    }
}
