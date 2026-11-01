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
    assert!(g.keywords(id).contains(Keywords::DEATHTOUCH));
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
    for p in [Player::P0, Player::P1] {
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
fn master_actual_life_payment_and_unaffordable_equip_are_transactional() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        master(&mut g, p);
        let creature = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let equipment = put(
            &mut g,
            p,
            "My Precious // Allure of Power",
            Zone::Battlefield,
        );
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        act(
            &mut g,
            Action::AnnounceActivate {
                card: equipment,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(creature),
            },
        );
        pay(&mut g);
        drive(&mut g);
        assert_eq!(g.state.players[p.index()].life, 18);
        assert_eq!(g.state.players[p.index()].library.len(), 6);
        assert_eq!(
            g.object_view(equipment).attached_to,
            Some(g.object_ref(creature))
        );
        g.gain_life(p, 3);
        g.lose_life(p, 0);
        assert!(g.state.pending_triggers.is_empty());
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a,Action::Activate{card,..} if *card==equipment))
            .expect("ordinary paid equip available");
        g.state.players[p.index()].life = 1;
        reject(&mut g, action);
        let before = g.snapshot();
        assert!(
            !g.legal_actions()
                .iter()
                .any(|a| matches!(a,Action::Activate{card,..} if *card==equipment))
        );
        assert_eq!(g.snapshot(), before);
    }
}
#[test]
fn master_simultaneous_damage_freezes_player_amount_and_survives_source_departure_owned() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let id = master(&mut g, p);
        let source = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let events = [2, 3].map(|amount| DamageEvent {
            preventable: true,
            source: g.damage_source(source),
            target: Target::Player(p),
            amount,
            combat: false,
        });
        g.apply_damage_batch(&events);
        assert_eq!(g.state.pending_triggers.len(), 1);
        assert_eq!(
            g.state.pending_triggers[0].life_event,
            Some(LifeTriggerView {
                player: p,
                amount: 5
            })
        );
        contracts::branch_contract(g.clone());
        let saved = g.snapshot();
        let mut left = g.clone();
        let mut right = g.clone();
        drop(g);
        left.move_object(id, Zone::Exile);
        drive(&mut left);
        assert_eq!(left.state.players[p.index()].library.len(), 3);
        drive(&mut right);
        assert_eq!(right.state.players[p.index()].library.len(), 3);
        right.restore(&saved).unwrap();
        drive(&mut right);
        assert_eq!(right.state.players[p.index()].library.len(), 3);
        assert_eq!(right.state.players[p.other().index()].library.len(), 8);
    }
}
#[test]
fn master_death_draw_counts_current_zero_one_two_graveyards_and_is_private() {
    for p in [Player::P0, Player::P1] {
        for (own, other, want) in [(4, 6, 0), (5, 6, 1), (5, 7, 2)] {
            let mut g = scenario(p);
            let id = master(&mut g, p);
            for _ in 0..own {
                put(&mut g, p, "Forest", Zone::Graveyard);
            }
            for _ in 0..other {
                put(&mut g, p.other(), "Forest", Zone::Graveyard);
            }
            let top: Vec<_> = g.state.players[p.index()]
                .library
                .iter()
                .rev()
                .take(want)
                .copied()
                .collect();
            shock(&mut g, p, Target::Card(id));
            drive(&mut g);
            assert_eq!(g.object_view(id).zone, Zone::Graveyard);
            assert_eq!(g.hand_count(p), want);
            assert_eq!(g.state.players[p.index()].library.len(), 8 - want);
            assert!(top.iter().all(|&id| g.object_view(id).zone == Zone::Hand));
            assert!(
                top.iter()
                    .all(|&id| !g.observe(p.other()).objects.iter().any(|o| o.id == id))
            );
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
fn master_large_life_loss_mills_only_available_cards_and_replacements_are_respected() {
    for replacement in [false, true] {
        let p = Player::P0;
        let mut g = scenario(p);
        master(&mut g, p);
        if replacement {
            put(&mut g, p, "Rest in Peace", Zone::Battlefield);
        }
        g.state.players[p.other().index()].hexproof_until_end_turn = true;
        g.state.players[p.other().index()].life = 100_000;
        let ids = g.state.players[p.other().index()].library.to_vec();
        g.lose_life(p.other(), 65_536);
        drive(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 34_464);
        assert!(g.state.players[p.other().index()].library.is_empty());
        assert!(ids.iter().all(|&id| g.object_view(id).zone
            == if replacement {
                Zone::Exile
            } else {
                Zone::Graveyard
            }));
        assert!(g.outcome().is_none());
    }
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
