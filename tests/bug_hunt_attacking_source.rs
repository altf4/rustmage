mod support;
use rustmage as _;
use rustmage_core::{
    cards::{CounterKind, Counters},
    engine::*,
};
use support::*;

fn hulk_combat_with_pending_enrage(blocker_name: &str) -> (Game, CardId) {
    let p = Player::P0;
    let mut game = scenario(p);
    let hulk = put(&mut game, p, "Bruce Banner", Zone::Hand);
    cast(&mut game, hulk, None);
    settle(&mut game);
    game.state.players[p.index()].mana = [0, 0, 0, 2, 2, 2];
    act(
        &mut game,
        Action::AnnounceActivate {
            card: hulk,
            ability: 1,
            counter_payment: Counters::default(),
        },
    );
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(&mut game);
    }
    settle(&mut game);
    assert_eq!(game.object_view(hulk).name, "The Incredible Hulk");
    assert_eq!(stats(&game, hulk), (8, 8));
    // Start combat on a later turn, when Hulk is able to attack.
    game.state.objects[hulk as usize].summoning_sick = false;
    let blocker = put(&mut game, p.other(), blocker_name, Zone::Battlefield);
    for _ in 0..8 {
        if game.state.pending == Pending::Attackers {
            break;
        }
        act(&mut game, Action::Pass);
    }
    act(&mut game, Action::Attack { card: hulk });
    act(&mut game, Action::FinishAttackers);
    for _ in 0..8 {
        if game.state.pending == Pending::Blockers {
            break;
        }
        act(&mut game, Action::Pass);
    }
    act(
        &mut game,
        Action::Block {
            blocker,
            attacker: hulk,
        },
    );
    act(&mut game, Action::FinishBlockers);
    for _ in 0..8 {
        if game.state.pending == Pending::Damage {
            break;
        }
        act(&mut game, Action::Pass);
    }
    let range = game.damage_assignment_range().unwrap();
    act(
        &mut game,
        Action::AssignDamage {
            target: range.target,
            amount: range.maximum,
        },
    );
    assert_eq!(game.object_view(blocker).zone, Zone::Graveyard);
    assert_eq!(
        game.state.stack.len(),
        1,
        "Hulk's enrage trigger went on the stack"
    );
    assert_eq!(game.state.stack[0].source.unwrap().id, hulk);
    (game, hulk)
}

#[test]
fn attacking_hulk_schedules_extra_combat_even_when_the_damage_kills_him() {
    let (mut game, hulk) = hulk_combat_with_pending_enrage("Vampire Nighthawk");
    let p = game.state.active;
    assert_eq!(game.object_view(hulk).zone, Zone::Graveyard);
    settle(&mut game);
    assert_eq!(game.state.step, Step::CombatDamage);
    assert_eq!(
        game.observe(p).additional_combats,
        1,
        "use Hulk's last-known attacking status to schedule another combat"
    );
}

#[test]
fn an_attacker_removed_from_combat_before_departure_uses_its_last_status() {
    let (mut game, hulk) = hulk_combat_with_pending_enrage("Skybeast Tracker");
    let p = game.state.active;
    assert!(game.object_view(hulk).attacking);
    let mut remains_attacking = game.clone();
    settle(&mut remains_attacking);
    assert_eq!(remains_attacking.observe(p).additional_combats, 1);
    assert_eq!(
        remains_attacking.counter_count(hulk, CounterKind::PlusOne),
        1
    );
    assert!(!remains_attacking.object_view(hulk).tapped);

    game.remove_from_combat(hulk, game.object_ref(hulk));
    assert!(!game.object_view(hulk).attacking);
    let mut departs = game.clone();
    departs.move_object(hulk, Zone::Graveyard);
    settle(&mut game);
    settle(&mut departs);
    assert_eq!(game.observe(p).additional_combats, 0);
    assert_eq!(departs.observe(p).additional_combats, 0);
}
