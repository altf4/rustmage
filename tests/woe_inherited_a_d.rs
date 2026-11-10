//! Paid card-level proofs for inherited Wilds of Eldraine definitions.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

fn activate(card: CardId, ability: u16) -> Action {
    Action::AnnounceActivate {
        card,
        ability,
        counter_payment: Counters::default(),
    }
}

fn pass_twice(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn reach_attackers(game: &mut Game) {
    for _ in 0..16 {
        if game.state.pending == Pending::Attackers {
            return;
        }
        act(game, Action::Pass);
    }
    panic!("attackers did not begin");
}

fn finish_turn(game: &mut Game) {
    let turn = game.state.turn;
    for _ in 0..32 {
        if game.state.turn != turn {
            return;
        }
        let action = match game.state.pending {
            Pending::Attackers => Action::FinishAttackers,
            Pending::Blockers => Action::FinishBlockers,
            Pending::Priority => Action::Pass,
            ref pending => panic!("unexpected turn choice {pending:?}"),
        };
        act(game, action);
    }
    panic!("turn did not finish");
}

#[test]
fn groves_bounty_pays_positive_x_with_zero_targets_or_exact_counter_shares() {
    let player = Player::P0;
    let mut game = scenario(player);
    let otter = put(
        &mut game,
        player,
        "Elusive Otter // Grove's Bounty",
        Zone::Hand,
    );
    let first = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let second = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let foreign = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    let bounce = put(&mut game, player, "Unsummon", Zone::Hand);
    act(&mut game, Action::AnnounceAdventure { card: otter });
    act(&mut game, Action::ChooseX { value: 3 });
    let payment = Action::PayCosts {
        payment: [0, 0, 0, 0, 1, 3],
    };

    let mut empty = game.clone();
    act(&mut empty, Action::FinishTargets);
    act(&mut empty, payment.clone());
    assert_eq!(empty.state.stack.last().unwrap().x, Some(3));
    assert!(empty.state.stack.last().unwrap().targets.is_empty());
    settle(&mut empty);
    assert_eq!(empty.object_view(otter).zone, Zone::Exile);
    for recipient in [first, second] {
        assert_eq!(empty.counter_count(recipient, CounterKind::PlusOne), 0);
    }
    act(&mut empty, Action::AnnounceExile { card: otter });
    act(
        &mut empty,
        Action::PayCosts {
            payment: [0, 1, 0, 0, 0, 0],
        },
    );
    settle(&mut empty);
    assert_eq!(empty.object_view(otter).name, "Elusive Otter");
    assert_eq!(empty.object_view(otter).zone, Zone::Battlefield);
    assert_eq!(stats(&empty, otter), (1, 1));

    reject(&mut game, target(foreign));
    act(&mut game, target(first));
    act(&mut game, target(second));
    act(&mut game, Action::FinishTargets);
    reject(&mut game, Action::AllocateTargetAmount { amount: 0 });
    reject(&mut game, Action::AllocateTargetAmount { amount: 3 });
    act(&mut game, Action::AllocateTargetAmount { amount: 1 });
    act(&mut game, Action::AllocateTargetAmount { amount: 2 });
    act(&mut game, payment);

    let mut stale = game.clone();
    cast(&mut stale, bounce, Some(Target::Card(first)));
    settle(&mut stale);
    assert_eq!(stale.object_view(first).zone, Zone::Hand);
    assert_eq!(stale.counter_count(second, CounterKind::PlusOne), 2);
    assert_eq!(stale.object_view(otter).zone, Zone::Exile);

    settle(&mut game);
    assert_eq!(game.counter_count(first, CounterKind::PlusOne), 1);
    assert_eq!(game.counter_count(second, CounterKind::PlusOne), 2);
    assert_eq!(game.counter_count(foreign, CounterKind::PlusOne), 0);
    assert_eq!(game.object_view(otter).zone, Zone::Exile);
}

#[test]
fn elusive_otter_prowess_changes_which_creatures_can_block_it() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let otter = put(
        &mut game,
        player,
        "Elusive Otter // Grove's Bounty",
        Zone::Hand,
    );
    let own_creature = put(&mut game, player, "Bear Cub", Zone::Hand);
    let shock = put(&mut game, player, "Shock", Zone::Hand);
    let boots = put(&mut game, player, "Swiftfoot Boots", Zone::Battlefield);
    let small = put(&mut game, opponent, "Llanowar Elves", Zone::Battlefield);
    let equal = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    cast(&mut game, otter, None);
    settle(&mut game);
    assert_eq!(stats(&game, otter), (1, 1));
    cast(&mut game, own_creature, None);
    settle(&mut game);
    assert_eq!(stats(&game, otter), (1, 1));
    act(&mut game, activate(boots, 0));
    act(&mut game, target(otter));
    pay(&mut game);
    settle(&mut game);

    cast(&mut game, shock, Some(Target::Player(opponent)));
    pass_twice(&mut game);
    assert_eq!(stats(&game, otter), (2, 2));
    assert_eq!(game.state.players[opponent.index()].life, 20);
    settle(&mut game);

    let mut cleanup = game.clone();
    finish_turn(&mut cleanup);
    assert_eq!(stats(&cleanup, otter), (1, 1));

    reach_attackers(&mut game);
    act(&mut game, Action::Attack { card: otter });
    act(&mut game, Action::FinishAttackers);
    pass_twice(&mut game);
    reject(
        &mut game,
        Action::Block {
            blocker: small,
            attacker: otter,
        },
    );
    act(
        &mut game,
        Action::Block {
            blocker: equal,
            attacker: otter,
        },
    );
    act(&mut game, Action::FinishBlockers);
    assert!(game.state.blocks.contains(&(equal, otter)));
}
