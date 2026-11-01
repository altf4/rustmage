//! Focused printed outcomes; shared engine invariants live in core tests.
//! Frozen life-loss milling and resolution-time graveyard thresholds.
use super::*;
fn until(g: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..64 {
        if ready(g) {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("Master decision budget: {:?}", g.state.pending);
}
fn drive(g: &mut Game) {
    until(g, |g| {
        g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty()
            && g.state.resolving.is_none()
    });
}
fn master(g: &mut Game, p: Player) -> CardId {
    let id = put(g, p, "The Master of Lake-town", Zone::Hand);
    g.state.players[p.index()].mana = [0, 0, 2, 0, 0, 1];
    cast(g, id, None);
    drive(g);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    assert_eq!(stats(g, id), (3, 2));

    id
}
fn shock(g: &mut Game, p: Player, target: Target) {
    let id = put(g, p, "Shock", Zone::Hand);
    g.state.priority = p;
    g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 0];
    cast(g, id, Some(target));
}
#[test]
fn master_paid_body_mills_actual_damage_for_either_player_without_targeting() {
    {
        let p = Player::P0;
        for target in [p, p.other()] {
            let mut g = scenario(p);
            master(&mut g, p);
            let top: Vec<_> = g.state.players[target.index()]
                .library
                .iter()
                .rev()
                .take(2)
                .copied()
                .collect();
            shock(&mut g, p, Target::Player(target));
            drive(&mut g);
            assert_eq!(g.state.players[target.index()].life, 18);
            assert_eq!(g.state.players[target.index()].library.len(), 6);
            assert!(
                top.iter()
                    .all(|&id| g.object_view(id).zone == Zone::Graveyard)
            );
            assert_eq!(g.state.players[target.other().index()].library.len(), 8);
        }
    }
}

#[test]
fn master_threshold_is_rechecked_at_resolution_and_replaced_death_does_not_trigger() {
    let p = Player::P0;
    let mut g = scenario(p);
    let id = master(&mut g, p);
    shock(&mut g, p, Target::Card(id));
    until(&mut g, |g| g.object_view(id).zone == Zone::Graveyard);
    for _ in 0..7 {
        put(&mut g, p.other(), "Forest", Zone::Graveyard);
    }
    g.move_object(id, Zone::Exile);
    drive(&mut g);
    assert_eq!(g.hand_count(p), 1);
    let mut g = scenario(p);
    let id = master(&mut g, p);
    put(&mut g, p, "Rest in Peace", Zone::Battlefield);
    shock(&mut g, p, Target::Card(id));
    drive(&mut g);
    assert_eq!(g.object_view(id).zone, Zone::Exile);
    assert_eq!(g.hand_count(p), 0);
    assert_eq!(g.state.players[p.index()].library.len(), 8);
}

#[test]
fn master_deathtouch_kills_a_larger_blocker_in_actual_combat() {
    let p = Player::P0;
    let mut g = scenario(p);
    let id = master(&mut g, p);
    let blocker = put(&mut g, p.other(), "Gigantic Big Bear", Zone::Battlefield);
    g.state.objects[id as usize].summoning_sick = false;
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    act(&mut g, Action::Attack { card: id });
    act(&mut g, Action::FinishAttackers);
    until(&mut g, |g| g.state.pending == Pending::Blockers);
    act(
        &mut g,
        Action::Block {
            blocker,
            attacker: id,
        },
    );
    act(&mut g, Action::FinishBlockers);
    until(&mut g, |g| g.state.step == Step::Main2);
    assert_eq!(g.object_view(blocker).zone, Zone::Graveyard);
    assert_eq!(g.object_view(id).zone, Zone::Graveyard);
}
