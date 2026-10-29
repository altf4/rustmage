//! Independent Ghalta combat and cost boundaries.
use super::*;

#[test]
fn blocking_creature_assigns_damage_using_toughness() {
    for attacker in [Player::P0, Player::P1] {
        let mut g = scenario(attacker);
        put(
            &mut g,
            attacker.other(),
            "Ghalta the Immovable",
            Zone::Battlefield,
        );
        let wall = put(
            &mut g,
            attacker.other(),
            "Monoist Sentry",
            Zone::Battlefield,
        );
        g.state.objects[wall as usize].growth.toughness = 7;
        let bear = put(&mut g, attacker, "Bear Cub", Zone::Battlefield);
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        act(&mut g, Action::Attack { card: bear });
        act(&mut g, Action::FinishAttackers);
        for _ in 0..32 {
            if g.state.pending == Pending::Blockers {
                break;
            }
            act(&mut g, Action::Pass);
        }
        act(
            &mut g,
            Action::Block {
                blocker: wall,
                attacker: bear,
            },
        );
        act(&mut g, Action::FinishBlockers);
        for _ in 0..32 {
            if g.object_view(bear).zone != Zone::Battlefield {
                break;
            }
            act(&mut g, Action::Pass);
        }
        assert_eq!(g.object_view(bear).zone, Zone::Graveyard);
        assert_eq!(g.state.objects[wall as usize].damage, 2);
        assert_eq!(g.state.players[attacker.other().index()].life, 20);
    }
}

#[test]
fn assignment_uses_power_at_equality_and_when_power_is_greater() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        put(&mut g, player, "Ghalta the Immovable", Zone::Battlefield);
        let wall = put(&mut g, player, "Monoist Sentry", Zone::Battlefield);
        g.state.objects[wall as usize].growth.toughness = 3;
        assert_eq!(stats(&g, wall), (4, 4));
        assert_eq!(g.combat_assignment_amount(wall), 4);
        g.state.objects[wall as usize].growth.power = 3;
        assert_eq!(stats(&g, wall), (7, 4));
        assert_eq!(g.combat_assignment_amount(wall), 7);
        g.state.objects[wall as usize].growth.toughness = 8;
        assert_eq!(stats(&g, wall), (7, 9));
        assert_eq!(g.combat_assignment_amount(wall), 9);
        super::super::contracts::branch_contract(g);
    }
}

#[test]
fn casting_reduction_tracks_current_copied_creature_and_ignores_enemy() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let spell = put(&mut g, player, "Ghalta the Immovable", Zone::Hand);
        let bearer = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        let enemy = put(&mut g, player.other(), "Bear Cub", Zone::Battlefield);
        g.state.objects[enemy as usize].growth.toughness = 30;
        assert_eq!(g.spell_reduction(player, spell), 2);
        let reference = g.object_ref(bearer);
        g.apply_temporary_copy(
            reference,
            CopyRecipe::new(Card::from_name("Ghalta the Immovable").unwrap()),
            CopyDeadline::EndOfTurn,
        );
        assert_eq!(g.spell_reduction(player, spell), 7);
        let before = g.snapshot();
        super::super::contracts::branch_contract(g.clone());
        g.change_controller(reference, player.other());
        assert_eq!(g.spell_reduction(player, spell), 0);
        g.restore(&before).unwrap();
        assert_eq!(g.spell_reduction(player, spell), 7);
        g.state.players[player.index()].mana = [1, 0, 0, 0, 0, 1];
        act(&mut g, Action::AnnounceCast { card: spell });
        pay(&mut g);
        assert_eq!(g.state.players[player.index()].mana, [0; 6]);
    }
}
