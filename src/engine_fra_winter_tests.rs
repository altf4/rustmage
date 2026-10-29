//! Optional payment and linked reflexive sacrifice, with live graveyard scaling.
use super::*;

fn wait_for_choice(game: &mut Game, player: Player) {
    for _ in 0..60 {
        if matches!(game.state.pending, Pending::EffectChoice { player: chooser } if chooser == player) { return; }
        choose(game, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("missing Winter choice: {:?}", game.state.pending);
}

#[test]
fn winter_can_sacrifice_itself_or_another_creature_and_then_an_opponent_chooses() {
    for player in [Player::P0, Player::P1] {
        for (sacrifice_self, exile_replacement) in [(false, false), (true, false), (true, true)] {
            let mut game = scenario(player);
            if exile_replacement { put(&mut game, player.other(), "Rest in Peace", Zone::Battlefield); }
            let fodder = put(&mut game, player, "Bear Cub", Zone::Battlefield);
            let enemy = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
            let winter = put(&mut game, player, "Winter, Tormented Loner", Zone::Hand);
            cast(&mut game, winter, None);
            wait_for_choice(&mut game, player);
            let forbidden = game.object_ref(enemy);
            reject(&mut game, Action::Sacrifice { object: forbidden });
            let chosen = game.object_ref(if sacrifice_self { winter } else { fodder });
            super::super::contracts::branch_contract(game.clone());
            act(&mut game, Action::Sacrifice { object: chosen });
            wait_for_choice(&mut game, player.other());
            let victim = game.object_ref(enemy);
            act(&mut game, Action::Sacrifice { object: victim });
            settle(&mut game);
            let destination = if exile_replacement { Zone::Exile } else { Zone::Graveyard };
            assert_eq!(game.object_view(chosen.card).zone, destination);
            assert_eq!(game.object_view(enemy).zone, destination);
            if !sacrifice_self { assert_eq!(game.object_view(winter).zone, Zone::Battlefield); }
        }
    }
}

#[test]
fn winter_declining_payment_does_not_create_the_reflexive_ability() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let enemy = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
        let winter = put(&mut game, player, "Winter, Tormented Loner", Zone::Hand);
        cast(&mut game, winter, None);
        wait_for_choice(&mut game, player);
        act(&mut game, Action::DeclineEffectPayment);
        settle(&mut game);
        assert_eq!(game.object_view(enemy).zone, Zone::Battlefield);
        assert_eq!(game.object_view(winter).zone, Zone::Battlefield);
    }
}

#[test]
fn winter_power_counts_owned_creature_and_planeswalker_cards_live() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let winter = put(&mut game, player, "Winter, Tormented Loner", Zone::Battlefield);
        let bear = put(&mut game, player, "Bear Cub", Zone::Graveyard);
        put(&mut game, player, "Ajani, Caller of the Pride", Zone::Graveyard);
        put(&mut game, player, "Forest", Zone::Graveyard);
        put(&mut game, player.other(), "Bear Cub", Zone::Graveyard);
        put(&mut game, player.other(), "Bear Cub", Zone::Graveyard);
        assert_eq!(stats(&game, winter), (2, 3));
        game.move_object(bear, Zone::Exile);
        assert_eq!(stats(&game, winter), (1, 3));
        game.change_controller(game.object_ref(winter), player.other());
        assert_eq!(stats(&game, winter), (2, 3));
        super::super::contracts::branch_contract(game);
    }
}
