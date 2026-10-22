//! Whole-card ECL rules scenarios using paid actions and owned indexed branches.
use super::ability_timeline_tests::{checked, settle, setup};
use super::tests::{cast, spawn};
use super::*;

fn mare(g: &mut Game, p: Player) -> CardId {
    let id = spawn(g, p, Card::from_name("Loch Mare").unwrap(), Zone::Hand);
    g.state.players[p.index()].mana[1] = 20;
    cast(g, id, None);
    settle(g);
    assert_eq!(g.object_view(id).counters.minus_one, 3);
    id
}
fn activate(g: &mut Game, source: CardId, ability_index: u16, target_id: Option<CardId>) {
    let action = g.legal_actions().into_iter().find(|a| matches!(a,
        Action::Activate {card, ability, target: None, ..} | Action::BeginActivate {card, ability, ..}
        if *card == source && *ability == ability_index))
        .expect("paid activation available");
    checked(g, action);
    if let Some(id) = target_id {
        checked(
            g,
            Action::ChooseTarget {
                target: Target::Card(id),
            },
        );
    }
}

#[test]
fn loch_mare_paid_draw_tap_stun_and_no_counter_legality() {
    for player in [Player::P0, Player::P1] {
        let mut g = setup(player);
        let source = mare(&mut g, player);
        let target = spawn(
            &mut g,
            player.other(),
            Card::GrizzlyBears,
            Zone::Battlefield,
        );
        let hand = g
            .state
            .objects
            .iter()
            .filter(|o| o.owner == player && o.zone == Zone::Hand)
            .count();
        activate(&mut g, source, 0, None);
        assert_eq!(g.object_view(source).counters.minus_one, 2);
        settle(&mut g);
        assert_eq!(
            g.state
                .objects
                .iter()
                .filter(|o| o.owner == player && o.zone == Zone::Hand)
                .count(),
            hand + 1
        );
        activate(&mut g, source, 1, Some(target));
        assert_eq!(g.object_view(source).counters.minus_one, 0);
        settle(&mut g);
        assert!(g.object_view(target).tapped);
        assert_eq!(g.object_view(target).counters.stun, 1);
        assert!(
            !g.legal_actions().iter().any(|a| matches!(a,
            Action::Activate { card, .. } | Action::BeginActivate { card, .. } if *card == source))
        );
    }
}

#[test]
fn loch_mare_mixed_counter_payment_atomicity_and_departed_source() {
    let mut g = setup(Player::P0);
    let source = mare(&mut g, Player::P0);
    g.set_counters(
        source,
        Counters::of(CounterKind::Stun, 1).with(CounterKind::Soul, 1),
    );
    let target = spawn(&mut g, Player::P1, Card::GrizzlyBears, Zone::Battlefield);
    let action = g
        .legal_actions()
        .into_iter()
        .find(|a| {
            matches!(a,
        Action::BeginActivate {card, ability: 1, counter_payment, ..}
        if *card == source && counter_payment.get(CounterKind::Stun) == 1
            && counter_payment.get(CounterKind::Soul) == 1)
        })
        .unwrap();
    let mut forged = action.clone();
    if let Action::BeginActivate {
        counter_payment, ..
    } = &mut forged
    {
        *counter_payment = Counters::of(CounterKind::Page, 2);
    }
    let before = g.snapshot();
    assert_eq!(g.apply(Player::P0, forged), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), before);
    checked(&mut g, action);
    checked(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    g.move_object(source, Zone::Graveyard);
    settle(&mut g);
    assert!(g.object_view(target).tapped);
    assert_eq!(g.object_view(target).counters.stun, 1);
}

#[test]
fn loch_mare_invalid_target_keeps_paid_cost_and_does_not_stun_reentry() {
    let mut g = setup(Player::P0);
    let source = mare(&mut g, Player::P0);
    let target = spawn(&mut g, Player::P1, Card::GrizzlyBears, Zone::Battlefield);
    activate(&mut g, source, 1, Some(target));
    g.move_object(target, Zone::Hand);
    g.move_object(target, Zone::Battlefield);
    settle(&mut g);
    assert!(!g.object_view(target).tapped);
    assert_eq!(g.object_view(target).counters.stun, 0);
    assert_eq!(g.object_view(source).counters.minus_one, 1);
}
