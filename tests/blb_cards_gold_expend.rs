//! Three Expend payoffs observe actual paid spell mana independently.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn settle_triggers(game: &mut Game) {
    for _ in 0..32 {
        if game.state.stack.is_empty() && game.state.pending == Pending::Priority {
            return;
        }
        choose(game, |action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. })
        });
    }
    panic!("threshold triggers did not settle");
}

#[test]
fn muerra_splits_raccoon_mana_and_expend_thresholds_apply_the_distinct_rewards() {
    let player = Player::P0;
    let mut game = scenario(player);
    put(
        &mut game,
        player,
        "Muerra, Trash Tactician",
        Zone::Battlefield,
    );
    let wander = put(&mut game, player, "Wandertale Mentor", Zone::Battlefield);
    let bruiser = put(&mut game, player, "Junkblade Bruiser", Zone::Battlefield);
    put(&mut game, player, "Firdoch Core", Zone::Battlefield);
    put(
        &mut game,
        player.other(),
        "Wandertale Mentor",
        Zone::Battlefield,
    );
    game.state.players[player.index()].mana = [0; 6];
    game.begin_step(Step::Main1);
    game.give_priority();
    game.prepare_triggers();
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    let split = [0, 0, 0, 2, 2, 0];
    act(&mut game, Action::ChooseMana { production: split });
    assert_eq!(game.state.players[player.index()].mana, split);

    for (index, amount) in [(3, 1), (4, 1)] {
        let mut branch = game.clone();
        let mut production = [0; 6];
        production[index] = amount;
        act(
            &mut branch,
            Action::ActivateMana {
                card: wander,
                ability: 0,
                payment: [0; 6],
                counter_payment: Counters::default(),
                production,
            },
        );
        assert_eq!(branch.state.players[player.index()].mana[index], 3);
        assert_eq!(branch.state.players[player.index()].life, 20);
        assert_eq!(branch.counter_count(wander, CounterKind::PlusOne), 0);
    }

    let first_exile = put(&mut game, player, "Forest", Zone::Library);
    let second_exile = put(&mut game, player, "Shock", Zone::Library);
    game.state.players[player.index()].mana = [8; 6];
    let first = put(&mut game, player, "Hill Giant", Zone::Hand);
    cast(&mut game, first, None);
    settle_triggers(&mut game);
    assert_eq!(game.state.players[player.index()].life, 23);
    assert_eq!(game.counter_count(wander, CounterKind::PlusOne), 1);
    assert_eq!(stats(&game, bruiser), (6, 6));
    assert!(game.keywords(bruiser).contains(Keywords::TRAMPLE));
    assert_eq!(game.object_view(first_exile).zone, Zone::Library);

    let second = put(&mut game, player, "Hill Giant", Zone::Hand);
    cast(&mut game, second, None);
    settle_triggers(&mut game);
    assert_eq!(game.state.players[player.index()].life, 23);
    assert_eq!(game.counter_count(wander, CounterKind::PlusOne), 1);
    assert_eq!(stats(&game, bruiser), (6, 6));
    for id in [first_exile, second_exile] {
        assert_eq!(game.object_view(id).zone, Zone::Exile);
        assert!(
            game.observe(player)
                .exile_permissions
                .iter()
                .any(|p| p.object.card == id)
        );
    }
    game.finish_cleanup();
    assert_eq!(stats(&game, bruiser), (4, 5));
    assert_eq!(game.counter_count(wander, CounterKind::PlusOne), 1);
}

#[test]
fn junkblade_bruiser_can_be_cast_for_each_printed_hybrid_combination() {
    let player = Player::P1;
    for (red, green) in [(2, 0), (1, 1), (0, 2)] {
        let mut game = scenario(player);
        let card = put(&mut game, player, "Junkblade Bruiser", Zone::Hand);
        game.state.players[player.index()].mana = [0, 0, 0, red, green, 3];
        cast(&mut game, card, None);
        settle_triggers(&mut game);
        assert_eq!(game.object_view(card).zone, Zone::Battlefield);
        assert_eq!(stats(&game, card), (4, 5));
        assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    }
}
