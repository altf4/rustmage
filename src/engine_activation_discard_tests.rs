use super::tests::{act, scenario, spawn};
use super::*;
fn named(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    let c = Card::from_name(name).unwrap();
    assert!(c.is_supported(), "{name}");
    spawn(g, p, c, zone)
}
fn replay(g: &mut Game, a: Action) {
    let s = g.snapshot();
    let r = g.revision();
    let p = g.actor().unwrap();
    let mut f = g.clone();
    act(&mut f, a.clone());
    act(g, a.clone());
    let expected = g.snapshot();
    assert_eq!(f.snapshot(), expected);
    g.restore(&s).unwrap();
    assert_eq!(g.apply_index(p, r, 0), Err(Error::StaleRevision));
    act(g, a);
    assert_eq!(g.snapshot(), expected);
}
fn reject(g: &mut Game, a: Action) {
    let s = g.snapshot();
    let r = g.revision();
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), s);
    assert_eq!(g.revision(), r);
}
fn available(g: &Game, card: CardId, ability: u16) -> bool {
    g.legal_actions()
        .iter()
        .any(|a| matches!(a,Action::AnnounceActivate{card:c,ability:n,..} if *c==card&&*n==ability))
}
fn announce(g: &mut Game, card: CardId, ability: u16) {
    replay(
        g,
        Action::AnnounceActivate {
            card,
            ability,
            counter_payment: Counters::default(),
        },
    );
}
fn choose(g: &mut Game, card: CardId) {
    replay(
        g,
        Action::ChooseAdditionalCost {
            object: g.object_ref(card),
        },
    );
}
fn pay(g: &mut Game) {
    let a = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    replay(g, a);
}
fn resolve(g: &mut Game) {
    while !g.state.stack.is_empty() || matches!(g.state.pending, Pending::OrderTriggers { .. }) {
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PutTrigger { .. } | Action::Pass))
            .unwrap();
        replay(g, a);
    }
}
#[test]
fn mandatory_activation_discard_cannot_bypass_choice_or_leak_the_hand() {
    let mut g = scenario();
    let source = named(&mut g, Player::P0, "Flamecache Gecko", Zone::Battlefield);
    let opponent = named(&mut g, Player::P1, "Forest", Zone::Hand);
    g.state.players[0].mana = [0, 0, 0, 2, 0, 0];
    assert!(!available(&g, source, 0));
    let discard = named(&mut g, Player::P0, "Forest", Zone::Hand);
    let s = g.snapshot();
    assert!(available(&g, source, 0));
    assert_eq!(g.snapshot(), s);
    for a in [
        Action::Activate {
            card: source,
            ability: 0,
            target: None,
            payment: [0, 0, 0, 2, 0, 0],
            counter_payment: Counters::default(),
        },
        Action::BeginActivate {
            card: source,
            ability: 0,
            payment: [0, 0, 0, 2, 0, 0],
            counter_payment: Counters::default(),
        },
    ] {
        reject(&mut g, a);
    }
    announce(&mut g, source, 0);
    assert_eq!(g.observe(Player::P0).decision, "additional_cost");
    assert!(g.observe(Player::P0).payment_request.is_none());
    assert!(g.observe(Player::P1).additional_cost.is_none());
    assert_eq!(
        g.observe(Player::P0).additional_cost.unwrap().candidates,
        vec![g.object_ref(discard)]
    );
    for object in [
        g.object_ref(source),
        g.object_ref(opponent),
        ObjectRef {
            card: discard,
            generation: g.object_ref(discard).generation + 1,
        },
        ObjectRef {
            card: u32::MAX,
            generation: 0,
        },
    ] {
        reject(&mut g, Action::ChooseAdditionalCost { object });
    }
    reject(
        &mut g,
        Action::PayCosts {
            payment: [0, 0, 0, 2, 0, 0],
        },
    );
    let s = g.snapshot();
    let r = g.revision();
    assert_eq!(
        g.apply(
            Player::P1,
            Action::ChooseAdditionalCost {
                object: g.object_ref(discard)
            }
        ),
        Err(Error::WrongActor)
    );
    assert_eq!(g.snapshot(), s);
    assert_eq!(g.revision(), r);
    choose(&mut g, discard);
    assert_eq!(g.object_view(discard).zone, Zone::Hand);
    assert!(g.observe(Player::P1).additional_cost.is_none());
    assert!(
        !g.observe(Player::P1)
            .objects
            .iter()
            .any(|o| o.id == discard)
    );
    assert!(g.state.stack.is_empty());
    assert_eq!(g.observe(Player::P0).payment_request.unwrap().generic, 1);
    let object = g.object_ref(discard);
    reject(&mut g, Action::ChooseAdditionalCost { object });
    pay(&mut g);
    assert_eq!(g.object_view(discard).zone, Zone::Graveyard);
    assert_eq!(g.state.stack.len(), 1);
    assert_eq!(g.state.players[0].spells_cast_this_turn, 0);
    resolve(&mut g);
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 1);
}
#[test]
fn funding_reserves_tap_source_and_exact_hand_incarnation() {
    let mut g = scenario();
    let volume = named(&mut g, Player::P0, "Murmuring Volume", Zone::Battlefield);
    let discard = named(&mut g, Player::P0, "Forest", Zone::Hand);
    named(&mut g, Player::P0, "Mountain", Zone::Battlefield);
    assert!(!available(&g, volume, 1)); // Volume cannot tap twice to fund its own ability.
    let second = named(&mut g, Player::P0, "Mountain", Zone::Battlefield);
    assert!(available(&g, volume, 1));
    announce(&mut g, volume, 1);
    choose(&mut g, discard);
    let goal = g.funding_goal(g.state.announcing.as_ref().unwrap());
    assert!(g.can_fund(goal.clone()));
    assert!(
        !g.legal_actions()
            .iter()
            .any(|a| matches!(a,Action::ActivateMana{card,..} if *card==volume))
    );
    let a = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a,Action::TapMana{card} if *card==second))
        .unwrap();
    replay(&mut g, a);
    let a = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::TapMana { .. }))
        .unwrap();
    replay(&mut g, a);
    let s = g.snapshot();
    g.move_object(discard, Zone::Graveyard);
    assert!(!g.can_fund(goal.clone()));
    g.move_object(discard, Zone::Hand);
    assert!(!g.can_fund(goal.clone()));
    g.restore(&s).unwrap();
    pay(&mut g);
    assert!(g.object_view(volume).tapped);
    resolve(&mut g);
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 1);
}
#[test]
fn haste_tap_cost_and_sorcery_once_restrictions_are_independent() {
    let mut g = scenario();
    let knight = named(
        &mut g,
        Player::P0,
        "Charging Strifeknight",
        Zone::Battlefield,
    );
    named(&mut g, Player::P0, "Forest", Zone::Hand);
    g.state.objects[knight as usize].summoning_sick = true;
    assert!(g.state.objects[knight as usize].summoning_sick);
    assert!(available(&g, knight, 0));
    announce(&mut g, knight, 0);
    let c = g.observe(Player::P0).additional_cost.unwrap().candidates[0].card;
    choose(&mut g, c);
    pay(&mut g);
    resolve(&mut g);
    assert!(g.object_view(knight).tapped);
    assert!(!available(&g, knight, 0));
    let gunner = named(&mut g, Player::P0, "Thunderhead Gunner", Zone::Battlefield);
    assert!(available(&g, gunner, 0));
    g.state.active = Player::P1;
    assert!(!available(&g, gunner, 0));
    g.state.active = Player::P0;
    announce(&mut g, gunner, 0);
    let c = g.observe(Player::P0).additional_cost.unwrap().candidates[0].card;
    choose(&mut g, c);
    pay(&mut g);
    resolve(&mut g);
    assert!(!available(&g, gunner, 0));
    g.begin_turn();
    g.state.step = Step::Main1;
    g.state.pending = Pending::Priority;
    g.state.priority = Player::P0;
    assert!(available(&g, gunner, 0));
}
#[test]
fn discard_trigger_waits_above_ability_and_is_not_a_spell_or_cycling_event() {
    let mut g = scenario();
    let source = named(&mut g, Player::P0, "Flamecache Gecko", Zone::Battlefield);
    let discard = named(&mut g, Player::P0, "Forest", Zone::Hand);
    let mako = named(&mut g, Player::P0, "Marauding Mako", Zone::Battlefield);
    named(&mut g, Player::P0, "Firebrand Archer", Zone::Battlefield);
    g.state.players[0].mana = [0, 0, 0, 2, 0, 0];
    announce(&mut g, source, 0);
    choose(&mut g, discard);
    assert!(g.state.pending_triggers.is_empty());
    pay(&mut g);
    assert_eq!(g.state.stack[0].card, source);
    let trigger = g.state.stack.iter().find(|s| s.card == mako).unwrap();
    assert_eq!(trigger.discard_event.unwrap().amount, 1);
    assert!(!trigger.discard_event.unwrap().cycling);
    assert_eq!(g.state.players[0].spells_cast_this_turn, 0);
    resolve(&mut g);
    assert_eq!(g.object_view(mako).counters.plus_one, 1);
    assert_eq!(g.state.players[1].life, 20);
}
#[test]
fn discard_and_self_sacrifice_are_both_paid_once_before_draw() {
    let mut g = scenario();
    let source = named(&mut g, Player::P0, "Masked Meower", Zone::Battlefield);
    let discard = named(&mut g, Player::P0, "Forest", Zone::Hand);
    announce(&mut g, source, 0);
    choose(&mut g, discard);
    pay(&mut g);
    assert_eq!(g.object_view(source).zone, Zone::Graveyard);
    assert_eq!(g.object_view(discard).zone, Zone::Graveyard);
    assert_eq!(g.state.creatures_died_this_turn, 1);
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 0);
    resolve(&mut g);
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 1);
}
#[test]
fn source_cost_entry_suspension_does_not_repeat_discard_or_source_payment() {
    let mut g = scenario();
    let source = named(&mut g, Player::P0, "Masked Meower", Zone::Battlefield);
    let discard = named(&mut g, Player::P0, "Forest", Zone::Hand);
    let held = named(&mut g, Player::P1, "Temple Garden", Zone::Exile);
    // Future exile-link IR: the printed Meower does not create this link.
    g.state.exile_links.push(ExileLink {
        source: g.object_ref(source),
        exiled: g.object_ref(held),
        return_zone: Zone::Battlefield,
    });
    announce(&mut g, source, 0);
    choose(&mut g, discard);
    pay(&mut g);
    assert_eq!(g.observe(Player::P1).decision, "entry_life");
    assert!(g.state.stack.is_empty());
    assert!(g.state.announcing.as_ref().unwrap().additional_paid);
    assert!(g.observe(Player::P1).payment_request.is_none());
    replay(&mut g, Action::ChooseEntryLife { pay: true });
    assert_eq!(g.state.players[1].life, 18);
    assert_eq!(g.state.creatures_died_this_turn, 1);
    assert_eq!(g.state.stack.len(), 1);
    assert!(g.state.announcing.is_none());
    resolve(&mut g);
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 1);
}
#[test]
fn source_boost_and_resolution_tap_are_not_activation_tap_costs() {
    for (name, keyword) in [
        ("Inner Demons Gangsters", Keywords::MENACE),
        ("Iron-Shield Elf", Keywords::INDESTRUCTIBLE),
    ] {
        let mut g = scenario();
        let source = named(&mut g, Player::P0, name, Zone::Battlefield);
        let discard = named(&mut g, Player::P0, "Forest", Zone::Hand);
        let other = named(&mut g, Player::P0, "Mountain", Zone::Hand);
        g.state.objects[source as usize].tapped = true;
        assert!(available(&g, source, 0)); // No {T} cost; even a tapped sick creature can activate.
        g.state.objects[source as usize].tapped = false;
        announce(&mut g, source, 0);
        choose(&mut g, discard);
        pay(&mut g);
        assert!(!g.object_view(source).tapped);
        assert!(!g.keywords(source).contains(keyword));
        resolve(&mut g);
        assert!(g.keywords(source).contains(keyword));
        if name == "Iron-Shield Elf" {
            assert!(g.object_view(source).tapped);
        } else {
            assert_eq!(g.stats(source).power, 4);
        }
        assert!(available(&g, source, 0));
        assert_eq!(g.object_view(other).zone, Zone::Hand);
        g.finish_cleanup();
        assert!(!g.keywords(source).contains(keyword));
    }
}
#[test]
fn flamecache_entry_uses_intervening_guard_and_adds_both_colors_on_resolution() {
    for lost in [false, true] {
        let mut g = scenario();
        if lost {
            g.lose_life(Player::P1, 1);
        }
        let source = named(&mut g, Player::P0, "Flamecache Gecko", Zone::Hand);
        g.move_object(source, Zone::Battlefield);
        g.prepare_triggers();
        assert_eq!(g.state.stack.len(), usize::from(lost));
        assert_eq!(g.state.players[0].mana, [0; 6]);
        resolve(&mut g);
        assert_eq!(
            g.state.players[0].mana,
            if lost { [0, 0, 1, 1, 0, 0] } else { [0; 6] }
        );
    }
}
