//! Dathon's Saga cast branch and a live enchantment counter choice.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn drain_until(game: &mut Game, predicate: impl Fn(&Game) -> bool) {
    for _ in 0..48 {
        if predicate(game) {
            return;
        }
        choose(game, |action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. })
        });
    }
    panic!("Dathon trigger did not resolve");
}

#[test]
fn an_enchantment_cast_draws_and_only_a_saga_cast_damages_before_its_spell() {
    let mut game = scenario(Player::P0);
    let dathon = put(
        &mut game,
        Player::P0,
        "Dathon, His Heart Open",
        Zone::Battlefield,
    );
    assert_eq!(stats(&game, dathon), (3, 3));
    let enchantment = put(&mut game, Player::P0, "Rest in Peace", Zone::Hand);
    cast(&mut game, enchantment, None);
    drain_until(&mut game, |g| g.state.players[0].cards_drawn_this_turn == 1);
    assert_eq!(game.state.players[1].life, 20);
    assert_eq!(game.object_view(enchantment).zone, Zone::Stack);
    settle(&mut game);

    let saga = put(&mut game, Player::P0, "The Naked Time", Zone::Hand);
    cast(&mut game, saga, None);
    drain_until(&mut game, |g| g.state.players[0].cards_drawn_this_turn == 2);
    assert_eq!(game.state.players[1].life, 18);
    assert_eq!(game.object_view(saga).zone, Zone::Stack);
}

#[test]
fn sorcery_activation_chooses_a_counter_already_on_own_enchantment() {
    let mut game = scenario(Player::P0);
    let dathon = put(
        &mut game,
        Player::P0,
        "Dathon, His Heart Open",
        Zone::Battlefield,
    );
    let enchantment = put(&mut game, Player::P0, "Rest in Peace", Zone::Battlefield);
    let opposing = put(&mut game, Player::P1, "Rest in Peace", Zone::Battlefield);
    let artifact = put(&mut game, Player::P0, "Short Bow", Zone::Battlefield);
    game.state.objects[enchantment as usize].counters.lore = 1;
    game.state.objects[enchantment as usize].counters.stun = 1;
    let activation = Action::AnnounceActivate {
        card: dathon,
        ability: 0,
        counter_payment: Counters::default(),
    };
    game.state.step = Step::End;
    reject(&mut game, activation.clone());
    game.state.step = Step::Main1;
    act(&mut game, activation);
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(opposing),
        },
    );
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(artifact),
        },
    );
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(enchantment),
        },
    );
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(&mut game, Action::FinishTargets);
    }
    pay(&mut game);
    drain_until(&mut game, |g| {
        matches!(
            g.state.pending,
            Pending::TargetExistingCounterKindToPut { .. }
        )
    });
    reject(
        &mut game,
        Action::ChooseTargetExistingCounterKindToPut {
            counter: CounterKind::Charge,
        },
    );
    act(
        &mut game,
        Action::ChooseTargetExistingCounterKindToPut {
            counter: CounterKind::Stun,
        },
    );
    settle(&mut game);
    assert_eq!(game.counter_count(enchantment, CounterKind::Lore), 1);
    assert_eq!(game.counter_count(enchantment, CounterKind::Stun), 2);
}
