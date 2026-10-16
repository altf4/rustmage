use super::tests::{act, cast, scenario, spawn};
use super::*;
fn professor(g: &mut Game, p: Player) -> CardId {
    let c = Card::from_name("Professor Dellian Fel").unwrap();
    assert!(c.is_supported());
    let id = spawn(g, p, c, Zone::Hand);
    g.move_object(id, Zone::Battlefield);
    id
}
fn activation(g: &Game, id: CardId, index: u16, target: Option<Target>) -> Action {
    g.legal_actions().into_iter().find(|a|matches!(a,Action::Activate{card,ability,target:t,..}if *card==id&&*ability==index&&*t==target)).unwrap()
}
fn has(g: &Game, id: CardId) -> bool {
    g.legal_actions().iter().any(|a|matches!(a,Action::Activate{card,..}|Action::BeginActivate{card,..}|Action::AnnounceActivate{card,..}if *card==id))
}
fn replay(g: &mut Game, a: Action) {
    let saved = g.snapshot();
    let rev = g.revision();
    let actor = g.actor().unwrap();
    let mut fork = g.clone();
    act(&mut fork, a.clone());
    act(g, a.clone());
    let expected = g.snapshot();
    assert_eq!(fork.snapshot(), expected);
    g.restore(&saved).unwrap();
    assert_eq!(g.apply_index(actor, rev, 0), Err(Error::StaleRevision));
    act(g, a);
    assert_eq!(g.snapshot(), expected);
}
fn passes(g: &mut Game) {
    replay(g, Action::Pass);
    replay(g, Action::Pass);
}
fn settle(g: &mut Game) {
    for _ in 0..200 {
        if g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty()
            && g.state.ordering_triggers.is_empty()
            && g.state.announcing.is_none()
            && g.state.resolving.is_none()
        {
            return;
        }
        let actions = g.legal_actions();
        let a = actions[crate::policy::smoke_index(g, &actions)].clone();
        assert_ne!(a, Action::Concede);
        replay(g, a);
    }
    panic!("did not settle");
}
fn reject(g: &mut Game, a: Action) {
    let saved = g.snapshot();
    let rev = g.revision();
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), saved);
    assert_eq!(g.revision(), rev);
}
fn emblem(g: &mut Game, p: Player) -> CardId {
    let factory = Card::catalogue().find(|c| c.is_emblem()).unwrap();
    let id = g.state.objects.len() as CardId;
    g.create_emblem(p, factory);
    id
}

#[test]
fn loyalty_is_paid_before_resolution_and_limit_is_shared_across_all_abilities() {
    for index in [0, 1, 2] {
        let mut g = scenario();
        let w = professor(&mut g, Player::P0);
        let b = spawn(&mut g, Player::P1, Card::GrizzlyBears, Zone::Battlefield);
        // Loyalty does not use the tap symbol, so sickness/tapping do not restrict it.
        g.state.objects[w as usize].tapped = true;
        let a = activation(
            &g,
            w,
            index,
            if index == 2 {
                Some(Target::Card(b))
            } else {
                None
            },
        );
        replay(&mut g, a);
        assert_eq!(
            g.object_view(w).counters.loyalty,
            match index {
                0 => 7,
                1 => 5,
                _ => 2,
            }
        );
        assert_eq!(g.state.players[0].life, 20);
        assert_eq!(g.object_view(w).used_abilities, 1u64 << index);
        assert!(!has(&g, w));
        passes(&mut g);
        assert!(!has(&g, w));
        match index {
            0 => assert_eq!(g.state.players[0].life, 23),
            1 => {
                assert_eq!(g.state.players[0].life, 19);
                assert_eq!(g.hand_count(Player::P0), 1);
            }
            _ => assert_eq!(g.object_view(b).zone, Zone::Graveyard),
        }
        // Zone incarnation changes reset the shared limit and printed loyalty.
        g.move_object(w, Zone::Hand);
        g.move_object(w, Zone::Battlefield);
        assert_eq!(g.object_view(w).used_abilities, 0);
        assert_eq!(g.object_view(w).counters.loyalty, 5);
        assert!(has(&g, w));
    }
}

#[test]
fn loyalty_requires_own_main_empty_stack_priority_and_resets_next_turn() {
    let mut g = scenario();
    let w = professor(&mut g, Player::P0);
    let a = activation(&g, w, 0, None);
    g.state.step = Step::Upkeep;
    assert!(!has(&g, w));
    reject(&mut g, a.clone());
    g.state.step = Step::Main1;
    g.state.active = Player::P1;
    assert!(!has(&g, w));
    reject(&mut g, a.clone());
    g.state.active = Player::P0;
    let shock = spawn(&mut g, Player::P0, Card::Shock, Zone::Hand);
    g.state.players[0].mana[3] = 1;
    cast(&mut g, shock, Some(Target::Player(Player::P1)));
    assert!(!has(&g, w));
    reject(&mut g, a.clone());
    passes(&mut g);
    replay(&mut g, a);
    passes(&mut g);
    assert!(!has(&g, w));
    g.begin_turn();
    g.state.step = Step::Main1;
    g.give_priority();
    assert!(has(&g, w));
    assert_eq!(g.object_view(w).counters.loyalty, 7);
}

#[test]
fn negative_cost_checks_exact_loyalty_and_zero_source_dies_before_ability_resolves() {
    let mut g = scenario();
    let w = professor(&mut g, Player::P0);
    let b = spawn(&mut g, Player::P1, Card::GrizzlyBears, Zone::Battlefield);
    assert!(
        !g.legal_actions()
            .iter()
            .any(|a| matches!(a,Action::Activate{card,ability:3,..}if *card==w))
    );
    g.state.objects[w as usize].counters.loyalty = 2;
    assert!(
        !g.legal_actions()
            .iter()
            .any(|a| matches!(a,Action::Activate{card,ability:2,..}if *card==w))
    );
    g.state.objects[w as usize].counters.loyalty = 3;
    let a = activation(&g, w, 2, Some(Target::Card(b)));
    replay(&mut g, a);
    assert_eq!(g.object_view(w).zone, Zone::Graveyard);
    assert_eq!(g.object_view(b).zone, Zone::Battlefield);
    assert_eq!(g.state.stack.len(), 1);
    passes(&mut g);
    assert_eq!(g.object_view(b).zone, Zone::Graveyard);
}

#[test]
fn targeted_fizzle_does_not_refund_loyalty_and_forged_counter_payment_is_atomic() {
    let mut g = scenario();
    let w = professor(&mut g, Player::P0);
    let b = spawn(&mut g, Player::P1, Card::GrizzlyBears, Zone::Battlefield);
    let a = activation(&g, w, 2, Some(Target::Card(b)));
    let mut forged = a.clone();
    if let Action::Activate {
        counter_payment, ..
    } = &mut forged
    {
        counter_payment.loyalty = 1;
    }
    reject(&mut g, forged);
    replay(&mut g, a);
    g.move_object(b, Zone::Hand);
    passes(&mut g);
    assert_eq!(g.object_view(w).counters.loyalty, 2);
    assert!(!has(&g, w));
    assert_eq!(g.object_view(b).zone, Zone::Hand);
}

#[test]
fn positive_loyalty_overflow_restores_all_state_costs_revision_and_checkpoints() {
    let mut g = scenario();
    let w = professor(&mut g, Player::P0);
    g.state.objects[w as usize].counters.loyalty = u64::MAX;
    let a = activation(&g, w, 0, None);
    let saved = g.snapshot();
    let rev = g.revision();
    assert!(matches!(
        g.apply(Player::P0, a.clone()),
        Err(Error::CapacityExceeded(_))
    ));
    assert_eq!(g.snapshot(), saved);
    assert_eq!(g.revision(), rev);
    let mut fork = g.clone();
    assert!(matches!(
        fork.apply(Player::P0, a),
        Err(Error::CapacityExceeded(_))
    ));
    assert_eq!(fork.snapshot(), saved);
    g.restore(&saved).unwrap();
    assert_eq!(g.snapshot(), saved);
}

#[test]
fn loyalty_mana_is_a_stack_ability_and_never_a_mana_funding_resource() {
    let mut ability = Card::from_name("Professor Dellian Fel")
        .unwrap()
        .activations()[0];
    ability.program = &[Instruction::AddMana(ManaProduction::Fixed([
        0, 0, 0, 2, 0, 0,
    ]))];
    assert!(ability.mana_production().is_none());
    ability.cost.loyalty = None;
    assert!(ability.mana_production().is_some());
}

#[test]
fn ultimate_creates_a_persistent_noncard_nonpermanent_emblem_after_its_source_dies() {
    let mut g = scenario();
    let w = professor(&mut g, Player::P0);
    g.state.objects[w as usize].counters.loyalty = 6;
    let a = activation(&g, w, 3, None);
    replay(&mut g, a);
    assert_eq!(g.object_view(w).zone, Zone::Graveyard);
    assert!(!g.state.objects.iter().any(|o| o.zone == Zone::Command));
    passes(&mut g);
    let id = g
        .state
        .objects
        .iter()
        .position(|o| o.zone == Zone::Command)
        .unwrap() as CardId;
    let view = g.object_view(id);
    assert!(view.emblem.is_some());
    assert!(view.token.is_none());
    assert!(!view.card.is_token());
    assert!(view.card.is_emblem());
    assert_eq!(view.card.kind_mask(), 0);
    assert_eq!(view.card.colors(), 0);
    assert!(view.card.info().mana_cost.is_empty());
    assert!(view.card.info().type_line.is_empty());
    assert_eq!(view.owner, Player::P0);
    assert!(g.state.life_sources.contains(&id));
    let source = spawn(&mut g, Player::P0, Card::LightningBolt, Zone::Hand);
    g.state.players[0].mana[3] = 1;
    assert!(!g.legal_actions().iter().any(
        |a| matches!(a,Action::Cast{card,target:Some(Target::Card(t)),..}if *card==source&&*t==id)
    ));
    g.move_object(source, Zone::Graveyard);
    let before = g.snapshot();
    assert!(!g.move_object(id, Zone::Graveyard).moved);
    assert_eq!(g.snapshot(), before);
    g.gain_life(Player::P0, 3);
    settle(&mut g);
    assert_eq!(g.state.players[1].life, 17);
    assert_eq!(g.object_view(id).zone, Zone::Command);
}

#[test]
fn multiple_emblems_capture_separate_exact_life_events_and_survive_source_turns() {
    let mut g = scenario();
    let a = emblem(&mut g, Player::P0);
    let b = emblem(&mut g, Player::P0);
    assert_ne!(a, b);
    g.state.players[1].life = 100;
    g.gain_life(Player::P1, 2);
    g.gain_life(Player::P0, 0);
    g.gain_life(Player::P0, -1);
    assert!(g.state.pending_triggers.is_empty());
    g.gain_life(Player::P0, 7);
    g.gain_life(Player::P0, 11);
    assert_eq!(g.state.pending_triggers.len(), 4);
    let events: Vec<_> = g
        .state
        .pending_triggers
        .iter()
        .map(|i| i.life_event.unwrap().amount)
        .collect();
    assert_eq!(events, vec![7, 7, 11, 11]);
    g.prepare_triggers();
    settle(&mut g);
    assert_eq!(g.state.players[1].life, 66);
    g.begin_turn();
    g.state.step = Step::Main1;
    g.give_priority();
    g.gain_life(Player::P0, 1);
    settle(&mut g);
    assert_eq!(g.state.players[1].life, 64);
    assert_eq!(g.object_view(a).zone, Zone::Command);
    assert_eq!(g.object_view(b).zone, Zone::Command);
}

#[test]
fn wide_life_event_amount_is_public_captured_and_replayable_through_target_announcement() {
    let mut g = scenario();
    let e = emblem(&mut g, Player::P0);
    let amount = 1i128 << 80;
    g.state.players[1].life = 1i128 << 100;
    g.gain_life(Player::P0, amount);
    g.prepare_triggers();
    let view = g.observe(Player::P0);
    let json = serde_json::to_value(&view).unwrap();
    assert_eq!(json["schema"], 40);
    let trigger = view
        .waiting_triggers
        .first()
        .or(view.announcing.as_ref())
        .unwrap();
    assert_eq!(trigger.card, e);
    assert_eq!(
        trigger.life_event.unwrap(),
        LifeTriggerView {
            player: Player::P0,
            amount
        }
    );
    assert_eq!(
        serde_json::to_value(trigger).unwrap()["life_event"]["amount"].to_string(),
        amount.to_string()
    );
    settle(&mut g);
    assert_eq!(g.state.players[1].life, (1i128 << 100) - amount);
}

#[test]
fn staged_targets_precede_loyalty_cost_and_missing_unpaid_source_cannot_commit() {
    let mut g = scenario();
    let w = professor(&mut g, Player::P0);
    let b = spawn(&mut g, Player::P1, Card::GrizzlyBears, Zone::Battlefield);
    let forest = spawn(&mut g, Player::P0, Card::Forest, Zone::Battlefield);
    replay(
        &mut g,
        Action::AnnounceActivate {
            card: w,
            ability: 2,
            counter_payment: Counters::default(),
        },
    );
    assert_eq!(g.object_view(w).counters.loyalty, 5);
    assert_eq!(g.object_view(w).used_abilities, 0);
    replay(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(b),
        },
    );
    assert!(matches!(g.state.pending, Pending::PayCosts { .. }));
    assert_eq!(g.object_view(w).counters.loyalty, 5);
    assert!(
        !g.legal_actions()
            .contains(&Action::TapMana { card: forest })
    );
    replay(&mut g, Action::PayCosts { payment: [0; 6] });
    assert_eq!(g.object_view(w).counters.loyalty, 2);
    passes(&mut g);
    assert_eq!(g.object_view(b).zone, Zone::Graveyard);
    let mut g = scenario();
    let w = professor(&mut g, Player::P0);
    replay(
        &mut g,
        Action::AnnounceActivate {
            card: w,
            ability: 1,
            counter_payment: Counters::default(),
        },
    );
    g.move_object(w, Zone::Hand);
    reject(&mut g, Action::PayCosts { payment: [0; 6] });
    assert!(g.state.stack.is_empty());
}

#[test]
fn capacity_during_staged_loyalty_payment_restores_the_pending_announcement() {
    let mut g = scenario();
    let w = professor(&mut g, Player::P0);
    g.state.objects[w as usize].counters.loyalty = u64::MAX;
    replay(
        &mut g,
        Action::AnnounceActivate {
            card: w,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    let before = g.snapshot();
    let rev = g.revision();
    assert!(matches!(
        g.apply(Player::P0, Action::PayCosts { payment: [0; 6] }),
        Err(Error::CapacityExceeded(_))
    ));
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), rev);
    assert!(g.state.announcing.is_some());
    assert!(g.state.stack.is_empty());
    assert_eq!(g.object_view(w).used_abilities, 0);
}
