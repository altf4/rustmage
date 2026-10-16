use super::tests::{act, scenario, spawn};
use super::*;
const NAMES: [&str; 10] = [
    "Coruscation Mage",
    "Finch Formation",
    "Flowerfoot Swordmaster",
    "Intrepid Rabbit",
    "Iridescent Vinelasher",
    "Rust-Shield Rampager",
    "Splash Lasher",
    "Starscape Cleric",
    "Steampath Charger",
    "Tender Wildguide",
];
fn named(g: &mut Game, p: Player, n: &str, z: Zone) -> CardId {
    let c = Card::from_name(n).unwrap();
    assert!(c.is_supported(), "{n}");
    spawn(g, p, c, z)
}
fn setup(p: Player) -> Game {
    let mut g = scenario();
    g.state.active = p;
    g.state.priority = p;
    for s in &mut g.state.players {
        s.mana = [30; 6];
    }
    g
}
fn replay(g: &mut Game, a: Action) {
    let s = g.snapshot();
    let r = g.revision();
    let p = g.actor().unwrap();
    let mut f = g.clone();
    act(&mut f, a.clone());
    act(g, a.clone());
    let e = g.snapshot();
    assert_eq!(e, f.snapshot());
    g.restore(&s).unwrap();
    assert_eq!(g.apply_index(p, r, 0), Err(Error::StaleRevision));
    act(g, a);
    assert_eq!(g.snapshot(), e);
}
fn choose(g: &mut Game, target: Option<Target>) {
    let actions = g.legal_actions();
    let a = target
        .and_then(|t| {
            actions
                .iter()
                .find(|a| **a == Action::ChooseTarget { target: t })
        })
        .or_else(|| {
            actions.iter().find(|a| {
                matches!(
                    a,
                    Action::ChooseOffspring { pay: false }
                        | Action::ChooseSpellCost { .. }
                        | Action::PayCosts { .. }
                        | Action::PutTrigger { .. }
                        | Action::ChooseTarget { .. }
                        | Action::FinishTargets
                        | Action::ArrangeLibrary { .. }
                        | Action::Discard { .. }
                        | Action::ChooseOptional { accept: true }
                        | Action::Pass
                )
            })
        })
        .expect("continuation")
        .clone();
    replay(g, a);
}
fn settle(g: &mut Game, target: Option<Target>) {
    g.prepare_triggers();
    for _ in 0..200 {
        if g.outcome().is_some()
            || g.state.pending == Pending::Priority
                && g.state.stack.is_empty()
                && g.state.announcing.is_none()
        {
            return;
        }
        choose(g, target);
    }
    panic!("unsettled");
}
fn cast(g: &mut Game, id: CardId, pay: bool, target: Option<Target>) {
    replay(g, Action::AnnounceCast { card: id });
    if matches!(g.state.pending, Pending::Offspring { .. }) {
        replay(g, Action::ChooseOffspring { pay });
    }
    while g
        .state
        .announcing
        .as_ref()
        .is_some_and(|a| a.item.kind == StackKind::Spell)
    {
        choose(g, target);
    }
    settle(g, target);
}
fn copies(g: &Game, p: Player, n: &str) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter(|(_, o)| {
            o.owner == p && o.zone == Zone::Battlefield && o.card.is_token() && o.card.name() == n
        })
        .map(|(i, _)| i as CardId)
        .collect()
}
#[test]
fn ten_complete_bodies_cast_paid_and_unpaid_in_both_seats_with_exact_copiable_values() {
    for p in [Player::P0, Player::P1] {
        for n in NAMES {
            for pay in [false, true] {
                let mut g = setup(p);
                let parent = named(&mut g, p, n, Zone::Hand);
                cast(&mut g, parent, pay, None);
                let c = Card::from_name(n).unwrap();
                let token = c.offspring_copy().unwrap();
                assert!(token.is_token());
                assert_eq!(token.name(), n);
                assert_eq!(
                    serde_json::from_str::<Card>(&serde_json::to_string(&token).unwrap()).unwrap(),
                    token
                );
                assert_eq!(token.info().mana_cost, c.info().mana_cost);
                assert_eq!(token.fixed_mana_value(), c.fixed_mana_value());
                assert_eq!(token.colors(), c.colors());
                assert_eq!(token.info().type_line, c.info().type_line);
                assert_eq!(token.keywords(), c.keywords());
                assert_eq!(token.effect(), Effect::Creature(1, 1));
                assert_eq!(token.triggers(), c.triggers());
                assert_eq!(token.activations(), c.activations());
                assert_eq!(token.offspring_cost(), c.offspring_cost());
                assert_eq!(
                    Card::from_name(token.token_info().unwrap().factory_identity),
                    Some(token)
                );
                assert_eq!(Card::from_name(n), Some(c));
                assert_eq!(copies(&g, p, n).len(), usize::from(pay), "{n}");
            }
        }
    }
}
#[test]
fn optional_choice_precedes_payment_and_cannot_bypass_funding_or_actor_checks() {
    let mut g = setup(Player::P0);
    let parent = named(&mut g, Player::P0, NAMES[0], Zone::Hand);
    g.state.players[0].mana = [0, 0, 0, 1, 0, 1];
    replay(&mut g, Action::AnnounceCast { card: parent });
    assert_eq!(g.observe(Player::P0).decision, "offspring");
    assert!(g.observe(Player::P0).payment_request.is_none());
    let saved = g.snapshot();
    let r = g.revision();
    assert_eq!(
        g.apply(Player::P0, Action::ChooseOffspring { pay: true }),
        Err(Error::IllegalAction)
    );
    assert_eq!(
        g.apply(Player::P1, Action::ChooseOffspring { pay: false }),
        Err(Error::WrongActor)
    );
    assert_eq!(g.snapshot(), saved);
    assert_eq!(g.revision(), r);
    replay(&mut g, Action::ChooseOffspring { pay: false });
    let a = &g.state.announcing.as_ref().unwrap().item;
    assert_eq!(a.cast.unwrap().offspring, Some(false));
    assert_eq!(
        g.observe(Player::P0)
            .payment_request
            .as_ref()
            .unwrap()
            .generic,
        1
    );
    while g.state.announcing.is_some() {
        choose(&mut g, None);
    }
    settle(&mut g, None);
    assert!(copies(&g, Player::P0, NAMES[0]).is_empty());
}
#[test]
fn colored_surcharge_and_generic_reductions_apply_to_combined_cost() {
    for n in [NAMES[6], NAMES[7]] {
        let mut g = setup(Player::P0);
        let id = named(&mut g, Player::P0, n, Zone::Hand);
        replay(&mut g, Action::AnnounceCast { card: id });
        replay(&mut g, Action::ChooseOffspring { pay: true });
        let request = g.observe(Player::P0).payment_request.unwrap();
        let c = Card::from_name(n).unwrap();
        let extra = c.offspring_cost().unwrap();
        let (base, generic) = c.cost();
        assert_eq!(request.generic, generic + extra.generic);
        for (i, required) in request.required.iter().enumerate() {
            assert_eq!(*required, base[i] + extra.colored[i]);
        }
    }
    let mut g = setup(Player::P0);
    named(&mut g, Player::P0, "Ballyrush Banneret", Zone::Battlefield);
    named(&mut g, Player::P0, "Ballyrush Banneret", Zone::Battlefield);
    let id = named(&mut g, Player::P0, NAMES[2], Zone::Hand);
    g.state.players[0].mana = [1, 0, 0, 0, 0, 0];
    cast(&mut g, id, true, None);
    assert_eq!(g.state.players[0].mana, [0; 6]);
    assert_eq!(copies(&g, Player::P0, NAMES[2]).len(), 1);
}
#[test]
fn countered_paid_spell_never_creates_a_copy() {
    let mut g = setup(Player::P0);
    let id = named(&mut g, Player::P0, NAMES[0], Zone::Hand);
    replay(&mut g, Action::AnnounceCast { card: id });
    replay(&mut g, Action::ChooseOffspring { pay: true });
    while g.state.announcing.is_some() {
        choose(&mut g, None);
    }
    replay(&mut g, Action::Pass);
    let counter = named(&mut g, Player::P1, "Counterspell", Zone::Hand);
    cast(&mut g, counter, false, Some(Target::Spell(id)));
    assert_eq!(g.object_view(id).zone, Zone::Graveyard);
    assert!(copies(&g, Player::P0, NAMES[0]).is_empty());
}
#[test]
fn suppression_blocks_offspring_and_other_entry_effects_but_not_payment() {
    for orb in ["Torpor Orb", "Doorkeeper Thrull"] {
        let mut g = setup(Player::P0);
        named(&mut g, Player::P1, orb, Zone::Battlefield);
        let id = named(&mut g, Player::P0, NAMES[3], Zone::Hand);
        let mana = g.state.players[0].mana.iter().sum::<u32>();
        cast(&mut g, id, true, None);
        assert_eq!(g.object_view(id).power, 3);
        assert_eq!(g.state.players[0].mana.iter().sum::<u32>(), mana - 4);
        assert!(copies(&g, Player::P0, NAMES[3]).is_empty());
    }
}
#[test]
fn queued_copy_survives_source_departure_and_reentry_without_reinheriting_payment() {
    let mut g = setup(Player::P0);
    let id = named(&mut g, Player::P0, NAMES[0], Zone::Hand);
    replay(&mut g, Action::AnnounceCast { card: id });
    replay(&mut g, Action::ChooseOffspring { pay: true });
    while g.state.announcing.is_some() {
        choose(&mut g, None);
    }
    replay(&mut g, Action::Pass);
    replay(&mut g, Action::Pass);
    assert_eq!(g.state.stack.len(), 1);
    let old = g.object_ref(id);
    g.move_object(id, Zone::Exile);
    g.move_object(id, Zone::Battlefield);
    assert_ne!(old, g.object_ref(id));
    assert_eq!(g.state.stack[0].object, old);
    settle(&mut g, None);
    assert_eq!(copies(&g, Player::P0, NAMES[0]).len(), 1);
    let copy = copies(&g, Player::P0, NAMES[0])[0];
    assert!(g.state.stack.is_empty());
    assert_eq!(g.object_view(copy).counters.plus_one, 0);
}
#[test]
fn copy_ignores_counters_growth_and_tapped_status_but_obeys_entry_replacements() {
    let mut g = setup(Player::P0);
    named(
        &mut g,
        Player::P1,
        "Authority of the Consuls",
        Zone::Battlefield,
    );
    let id = named(&mut g, Player::P0, NAMES[0], Zone::Hand);
    replay(&mut g, Action::AnnounceCast { card: id });
    replay(&mut g, Action::ChooseOffspring { pay: true });
    while g.state.announcing.is_some() {
        choose(&mut g, None);
    }
    replay(&mut g, Action::Pass);
    replay(&mut g, Action::Pass);
    g.state.objects[id as usize].counters.plus_one = 5;
    g.state.objects[id as usize].growth.power = 10;
    settle(&mut g, None);
    let token = copies(&g, Player::P0, NAMES[0])[0];
    let v = g.object_view(token);
    assert_eq!((v.power, v.toughness), (1, 1));
    assert!(v.tapped);
    assert_eq!(v.counters.plus_one, 0);
}
#[test]
fn copied_cast_landfall_and_life_observers_have_full_programs() {
    for i in [0, 4, 7] {
        let mut g = setup(Player::P0);
        let id = named(&mut g, Player::P0, NAMES[i], Zone::Hand);
        cast(&mut g, id, true, None);
        if i == 0 {
            let spell = named(&mut g, Player::P0, "Opt", Zone::Hand);
            cast(&mut g, spell, false, None);
        } else if i == 4 {
            let land = named(&mut g, Player::P0, "Forest", Zone::Hand);
            replay(&mut g, Action::PlayLand { card: land });
            settle(&mut g, Some(Target::Player(Player::P1)));
        } else {
            g.gain_life(Player::P0, 2);
            settle(&mut g, None);
        }
        assert_eq!(g.state.players[1].life, 18, "{}", NAMES[i]);
    }
}
#[test]
fn copied_death_trigger_survives_token_ceasing_and_slot_reuse() {
    let mut g = setup(Player::P0);
    let id = named(&mut g, Player::P0, NAMES[8], Zone::Hand);
    cast(&mut g, id, true, None);
    let token = copies(&g, Player::P0, NAMES[8])[0];
    let old = g.object_ref(token);
    g.move_object(token, Zone::Graveyard);
    g.state_based_actions();
    let other = Card::from_name(NAMES[0]).unwrap().offspring_copy().unwrap();
    g.create_tokens(Player::P0, other, 1, false);
    assert_ne!(g.object_ref(token), old);
    settle(&mut g, Some(Target::Player(Player::P1)));
    assert_eq!(g.state.players[1].life, 19);
    assert!(
        g.object_view(token).card != Card::from_name(NAMES[8]).unwrap().offspring_copy().unwrap()
    );
}
#[test]
fn tender_copy_retains_both_activations_and_summoning_sickness() {
    let mut g = setup(Player::P0);
    let id = named(&mut g, Player::P0, NAMES[9], Zone::Hand);
    cast(&mut g, id, true, None);
    let token = copies(&g, Player::P0, NAMES[9])[0];
    assert!(!g.legal_actions().iter().any(
        |a| matches!(a,Action::ActivateMana{card,..}|Action::Activate{card,..}if *card==token)
    ));
    g.state.objects[token as usize].summoning_sick = false;
    g.state.players[0].mana = [0; 6];
    let a=g.legal_actions().into_iter().find(|a|matches!(a,Action::ActivateMana{card,production,..}if *card==token&&production[1]==1)).unwrap();
    replay(&mut g, a);
    assert_eq!(g.state.players[0].mana[1], 1);
    g.state.objects[token as usize].untap();
    let a = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a,Action::Activate{card,ability:1,..}if *card==token))
        .unwrap();
    replay(&mut g, a);
    settle(&mut g, None);
    assert_eq!(g.object_view(token).counters.plus_one, 1);
    assert_eq!(g.object_view(id).counters.plus_one, 0);
}
#[test]
fn flowerfoot_copy_has_independent_once_per_incarnation_history() {
    let mut g = setup(Player::P0);
    let id = named(&mut g, Player::P0, NAMES[2], Zone::Hand);
    cast(&mut g, id, true, None);
    let token = copies(&g, Player::P0, NAMES[2])[0];
    for target in [id, id, token] {
        let spell = named(&mut g, Player::P0, "Giant Growth", Zone::Hand);
        cast(&mut g, spell, false, Some(Target::Card(target)));
    }
    assert_eq!(g.state.triggered_this_turn.len(), 2);
    assert_eq!(g.object_view(id).power, 9);
    assert_eq!(g.object_view(token).power, 6);
}
#[test]
fn failed_offspring_event_allocation_restores_paid_spell_and_parent_entry() {
    let mut g = setup(Player::P0);
    let id = named(&mut g, Player::P0, NAMES[0], Zone::Hand);
    replay(&mut g, Action::AnnounceCast { card: id });
    replay(&mut g, Action::ChooseOffspring { pay: true });
    while g.state.announcing.is_some() {
        choose(&mut g, None);
    }
    replay(&mut g, Action::Pass);
    g.state.next_stack_id = u64::MAX;
    let saved = g.snapshot();
    let rev = g.revision();
    assert!(matches!(
        g.apply(g.actor().unwrap(), Action::Pass),
        Err(Error::CapacityExceeded(_))
    ));
    assert_eq!(g.snapshot(), saved);
    assert_eq!(g.revision(), rev);
    assert_eq!(g.object_view(id).zone, Zone::Stack);
}

#[test]
fn offspring_and_residual_entries_offer_apnap_order_and_each_copy_enters_once() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        named(
            &mut g,
            p.other(),
            "Authority of the Consuls",
            Zone::Battlefield,
        );
        let source = named(&mut g, p, NAMES[3], Zone::Hand);
        replay(&mut g, Action::AnnounceCast { card: source });
        replay(&mut g, Action::ChooseOffspring { pay: true });
        while g.state.announcing.is_some() {
            choose(&mut g, None);
        }
        replay(&mut g, Action::Pass);
        replay(&mut g, Action::Pass);
        assert!(matches!(g.state.pending,Pending::OrderTriggers{player}if player==p));
        assert_eq!(
            g.state
                .ordering_triggers
                .iter()
                .filter(|s| s.controller == p)
                .count(),
            2
        );
        assert_eq!(
            g.state
                .ordering_triggers
                .iter()
                .filter(|s| s.controller == p.other())
                .count(),
            1
        );
        assert!(
            g.state
                .ordering_triggers
                .iter()
                .all(|s| s.entering_object == Some(g.object_ref(source)))
        );
        settle(&mut g, Some(Target::Card(source)));
        assert_eq!(copies(&g, p, NAMES[3]).len(), 1);
        assert_eq!(g.state.players[p.other().index()].life, 22);
        assert_eq!(g.object_view(source).power, 5);
    }
}
