//! Declared attackers qualify using counters present at declaration.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const WATCHER: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::ControlledCreatureAttacksWithCounter(CounterKind::PlusOne),
        targets: TargetPlan::NONE,
        program: &[Instruction::DoubleCounterKind {
            subject: Subject::Entering,
            kind: CounterKind::PlusOne,
        }],
    }],
    ..CardRules::EMPTY
};

fn declare(game: &mut Game, attackers: &[CardId]) {
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    game.state.priority = game.state.active;
    for &card in attackers {
        act(game, Action::Attack { card });
    }
}

fn order_triggers(game: &mut Game) {
    while matches!(game.state.pending, Pending::OrderTriggers { .. }) {
        choose(game, |action| matches!(action, Action::PutTrigger { .. }));
    }
}

#[test]
fn each_eligible_attacker_is_exact_and_doubles_live_counters_after_source_departure() {
    let _rules = fixture_rules::activate(&[(Card::HillGiant, &WATCHER)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let observer = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let first = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let second = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let late = put(&mut game, player, "Llanowar Elves", Zone::Battlefield);
    let stays_home = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    for card in [first, second, stays_home] {
        game.add_counter(card, CounterKind::PlusOne, 1);
    }
    for card in [first, late] {
        game.add_counter(card, CounterKind::Shield, 1);
    }
    declare(&mut game, &[first, second, late]);

    let snapshot = game.snapshot();
    let finish = Action::FinishAttackers;
    let index = game
        .legal_actions()
        .iter()
        .position(|action| *action == finish)
        .unwrap();
    let mut indexed = game.clone();
    indexed.apply_index(player, game.revision(), index).unwrap();
    act(&mut game, finish);
    assert_eq!(game.snapshot(), indexed.snapshot());
    for branch in [&mut game, &mut indexed] {
        order_triggers(branch);
        assert_eq!(branch.state.stack.len(), 2);
        let watched = branch.object_ref(observer);
        let first_attacker = branch.object_ref(first);
        let second_attacker = branch.object_ref(second);
        let observed: Vec<_> = branch
            .state
            .stack
            .iter()
            .map(|item| {
                assert_eq!(item.source().unwrap().object, watched);
                assert!(item.targets.is_empty());
                item.entering_object.unwrap()
            })
            .collect();
        assert!(observed.contains(&first_attacker) && observed.contains(&second_attacker));

        branch.add_counter(first, CounterKind::PlusOne, 2);
        branch.add_counter(late, CounterKind::PlusOne, 1);
        branch.move_object(observer, Zone::Graveyard);
        branch.move_object(second, Zone::Exile);
        branch.move_object(second, Zone::Battlefield);
        branch.add_counter(second, CounterKind::PlusOne, 1);
        branch.record_control(first_attacker, player.other(), None);
        assert_eq!(branch.object_view(first).controller, player.other());
        settle(branch);

        assert_eq!(branch.counter_count(first, CounterKind::PlusOne), 6);
        assert_eq!(branch.counter_count(first, CounterKind::Shield), 1);
        for card in [second, late, stays_home] {
            assert_eq!(branch.counter_count(card, CounterKind::PlusOne), 1);
        }
        assert_eq!(branch.counter_count(late, CounterKind::Shield), 1);
    }
    assert_eq!(game.snapshot(), indexed.snapshot());
    indexed.restore(&snapshot).unwrap();
    assert_eq!(indexed.snapshot(), snapshot);
}

#[test]
fn a_qualified_observer_can_double_its_own_counters_but_enemy_observers_do_not_trigger() {
    let _rules = fixture_rules::activate(&[(Card::HillGiant, &WATCHER)]);
    let player = Player::P1;
    let mut game = scenario(player);
    let own = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let opponent = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    for card in [own, opponent] {
        game.add_counter(card, CounterKind::PlusOne, 1);
    }
    declare(&mut game, &[own]);
    act(&mut game, Action::FinishAttackers);
    assert_eq!(game.state.stack.len(), 1);
    assert_eq!(game.state.stack[0].source().unwrap().id, own);
    settle(&mut game);
    assert_eq!(game.counter_count(own, CounterKind::PlusOne), 2);
    assert_eq!(game.counter_count(opponent, CounterKind::PlusOne), 1);
}
