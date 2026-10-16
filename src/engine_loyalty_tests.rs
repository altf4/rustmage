//! The walker objects below are explicit future-card fixtures. Their printed
//! abilities remain unsupported and submitted decks continue to reject them.
use super::activations::counter_payments;
use super::tests::{act, cast, scenario, spawn};
use super::*;
fn named(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    spawn(g, p, Card::from_name(name).unwrap(), zone)
}
fn walker(g: &mut Game, p: Player, name: &str) -> CardId {
    let card = Card::from_name(name).unwrap();
    assert!(card.is_planeswalker());
    assert!(!card.is_supported());
    let id = spawn(g, p, card, Zone::Hand);
    g.move_object(id, Zone::Battlefield);
    assert_eq!(g.object_view(id).counters.loyalty, card.starting_loyalty());
    id
}
fn funded(g: &mut Game, name: &str) -> CardId {
    let id = named(g, Player::P0, name, Zone::Hand);
    let (mut mana, generic) = g.state.objects[id as usize].card.cost();
    mana[5] += generic;
    g.state.players[0].mana = mana;
    id
}
fn replay(g: &mut Game, a: Action) {
    let saved = g.snapshot();
    let actor = g.actor().unwrap();
    let rev = g.revision();
    let mut fork = g.clone();
    act(g, a.clone());
    act(&mut fork, a.clone());
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
    for _ in 0..1000 {
        if g.state.stack.is_empty()
            && g.state.resolving.is_none()
            && g.state.announcing.is_none()
            && g.state.ordering_triggers.is_empty()
            && g.state.pending_triggers.is_empty()
        {
            return;
        }
        let actions = g.legal_actions();
        let action = actions[crate::policy::smoke_index(g, &actions)].clone();
        assert_ne!(action, Action::Concede);
        replay(g, action);
    }
    panic!("fixture didn't settle");
}
#[test]
fn damage_to_walkers_removes_loyalty_and_zero_loyalty_sba_ignores_indestructible() {
    let mut g = scenario();
    let w = walker(&mut g, Player::P1, "Chandra, Torch of Defiance");
    g.state.objects[w as usize].gained_keywords = Keywords::INDESTRUCTIBLE;
    let shock = funded(&mut g, "Shock");
    cast(&mut g, shock, Some(Target::Card(w)));
    passes(&mut g);
    assert_eq!(g.object_view(w).zone, Zone::Battlefield);
    assert_eq!(g.object_view(w).counters.loyalty, 2);
    assert_eq!(g.object_view(w).damage, 0);
    assert!(g.object_view(w).damaged_this_turn);
    let bolt = funded(&mut g, "Lightning Bolt");
    cast(&mut g, bolt, Some(Target::Card(w)));
    passes(&mut g);
    assert_eq!(g.object_view(w).zone, Zone::Graveyard);
    assert_eq!(g.object_view(w).counters.loyalty, 0);
    assert_eq!(g.object_view(w).damage, 0);
    assert!(!g.object_view(w).damaged_this_turn);
}
#[test]
fn loyalty_entry_and_zone_resets_are_separate_from_turn_and_cleanup() {
    let mut g = scenario();
    let w = walker(&mut g, Player::P1, "Vivien Reid");
    assert_eq!(g.object_view(w).counters.loyalty, 5);
    g.state.objects[w as usize]
        .counters
        .add(CounterKind::Loyalty, 3);
    g.begin_turn();
    assert_eq!(g.object_view(w).counters.loyalty, 8);
    g.finish_cleanup();
    assert_eq!(g.object_view(w).counters.loyalty, 8);
    g.move_object(w, Zone::Hand);
    assert_eq!(g.object_view(w).counters.loyalty, 0);
    g.move_object(w, Zone::Battlefield);
    assert_eq!(g.object_view(w).counters.loyalty, 5);
    let bear = named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    g.state.objects[bear as usize].counters.loyalty = 7;
    g.state_based_actions();
    assert_eq!(g.object_view(bear).zone, Zone::Battlefield);
    assert_eq!((g.stats(bear).power, g.stats(bear).toughness), (2, 2));
    assert!(g.object_view(bear).modified);
}
#[test]
fn marked_damage_deathtouch_lifelink_and_player_damage_events_are_distinct() {
    let mut g = scenario();
    let w = walker(&mut g, Player::P1, "Vivien Reid");
    let source = named(&mut g, Player::P0, "Vampire Nighthawk", Zone::Battlefield);
    g.apply_damage_batch(&[DamageEvent {
        source: g.damage_source(source),
        target: Target::Card(w),
        amount: 1,
        combat: false,
    }]);
    g.state_based_actions();
    assert_eq!(g.object_view(w).counters.loyalty, 4);
    assert_eq!(g.object_view(w).zone, Zone::Battlefield);
    assert_eq!(g.object_view(w).damage, 0);
    assert!(!g.state.objects[w as usize].deathtouch_since_sba);
    assert_eq!(g.state.players[0].life, 21);
    assert_eq!(g.state.players[1].life, 20);
    assert!(g.object_view(source).dealt_damage_this_turn);
    assert!(g.object_view(w).damaged_this_turn);
}
#[test]
fn huge_damage_removes_only_available_counters_without_narrowing_the_event() {
    let mut g = scenario();
    let w = walker(&mut g, Player::P1, "Vivien Reid");
    let source = named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    g.state.objects[w as usize].counters.loyalty = u64::MAX;
    g.apply_damage_batch(&[DamageEvent {
        source: g.damage_source(source),
        target: Target::Card(w),
        amount: i128::from(u64::MAX) - 1,
        combat: true,
    }]);
    assert_eq!(g.object_view(w).counters.loyalty, 1);
    g.apply_damage_batch(&[DamageEvent {
        source: g.damage_source(source),
        target: Target::Card(w),
        amount: i128::MAX,
        combat: false,
    }]);
    assert_eq!(g.object_view(w).counters.loyalty, 0);
    g.state_based_actions();
    assert_eq!(g.object_view(w).zone, Zone::Graveyard);
}
#[test]
fn group_damage_is_simultaneous_for_creatures_and_walkers_and_does_not_target_hexproof() {
    let mut g = scenario();
    let own = walker(&mut g, Player::P0, "Vivien Reid");
    let enemy = walker(&mut g, Player::P1, "Vivien Reid");
    let bear = named(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
    for id in [own, enemy, bear] {
        g.state.objects[id as usize].gained_keywords = Keywords::HEXPROOF;
    }
    let source = funded(&mut g, "Dragonback Assault");
    cast(&mut g, source, None);
    passes(&mut g);
    passes(&mut g);
    assert_eq!(g.object_view(own).counters.loyalty, 2);
    assert_eq!(g.object_view(enemy).counters.loyalty, 2);
    assert_eq!(g.object_view(own).damage, 0);
    assert_eq!(g.object_view(enemy).damage, 0);
    assert_eq!(g.object_view(bear).zone, Zone::Graveyard);
    let land = named(&mut g, Player::P0, "Forest", Zone::Hand);
    g.move_object(land, Zone::Battlefield);
    g.prepare_triggers();
    settle(&mut g);
    let dragons: Vec<_> = g
        .state
        .objects
        .iter()
        .enumerate()
        .filter(|(_, o)| {
            o.zone == Zone::Battlefield && o.card.is_token() && o.card.has_subtype("Dragon")
        })
        .collect();
    assert_eq!(dragons.len(), 1);
    let id = dragons[0].0 as CardId;
    assert_eq!((g.stats(id).power, g.stats(id).toughness), (4, 4));
    assert!(g.keywords(id).contains(Keywords::FLYING));
}
#[test]
fn every_new_modal_mode_has_exact_controller_sets_and_draw_or_damage_programs() {
    for name in ["Splatter Technique", "Fulminous Forte"] {
        for mode in 0..2 {
            let mut g = scenario();
            let own = walker(&mut g, Player::P0, "Vivien Reid");
            let enemy = walker(&mut g, Player::P1, "Vivien Reid");
            let bear = named(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
            let spell = funded(&mut g, name);
            replay(&mut g, Action::AnnounceCast { card: spell });
            replay(&mut g, Action::ChooseMode { mode });
            if name == "Fulminous Forte" && mode == 1 {
                replay(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(enemy),
                    },
                );
            }
            let pay = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::PayCosts { .. }))
                .unwrap();
            replay(&mut g, pay);
            settle(&mut g);
            if name == "Splatter Technique" {
                if mode == 0 {
                    assert_eq!(g.state.players[0].cards_drawn_this_turn, 4);
                    assert_eq!(g.object_view(own).counters.loyalty, 5);
                    assert_eq!(g.object_view(enemy).counters.loyalty, 5);
                } else {
                    assert_eq!(g.object_view(own).counters.loyalty, 1);
                    assert_eq!(g.object_view(enemy).counters.loyalty, 1);
                    assert_eq!(g.object_view(bear).zone, Zone::Graveyard);
                }
            } else {
                assert_eq!(g.object_view(own).counters.loyalty, 5);
                if mode == 0 {
                    assert_eq!(g.object_view(enemy).counters.loyalty, 4);
                    assert_eq!(g.object_view(bear).damage, 1);
                } else {
                    assert_eq!(g.object_view(enemy).zone, Zone::Graveyard);
                    assert_eq!(g.object_view(bear).damage, 0);
                }
            }
        }
    }
}
#[test]
fn targeted_damage_and_discard_choices_defer_zero_loyalty_sbas_until_resolution_finishes() {
    let mut g = scenario();
    let w = walker(&mut g, Player::P1, "Chandra, Torch of Defiance");
    let spell = funded(&mut g, "Broadside Barrage");
    cast(&mut g, spell, Some(Target::Card(w)));
    passes(&mut g);
    assert!(matches!(g.state.pending, Pending::EffectDiscard { .. }));
    assert_eq!(g.object_view(w).zone, Zone::Battlefield);
    assert_eq!(g.object_view(w).counters.loyalty, 0);
    let json = serde_json::to_value(g.observe(Player::P0)).unwrap();
    assert_eq!(json["schema"], 41);
    settle(&mut g);
    assert_eq!(g.object_view(w).zone, Zone::Graveyard);
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 1);
    let mut g = scenario();
    let w = walker(&mut g, Player::P1, "Chandra, Torch of Defiance");
    let spell = funded(&mut g, "Sear");
    cast(&mut g, spell, Some(Target::Card(w)));
    passes(&mut g);
    assert_eq!(g.object_view(w).zone, Zone::Graveyard);
}
#[test]
fn loyalty_is_in_all_kind_counter_doubling_and_any_kind_payment_space() {
    let mut g = scenario();
    let w = walker(&mut g, Player::P0, "The Aetherspark");
    let z = named(
        &mut g,
        Player::P0,
        "Zimone, Paradox Sculptor",
        Zone::Battlefield,
    );
    g.state.players[0].mana = [0, 1, 0, 0, 1, 0];
    let action = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a,Action::AnnounceActivate {card,..}if *card==z))
        .unwrap();
    replay(&mut g, action);
    replay(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(w),
        },
    );
    replay(&mut g, Action::FinishTargets);
    let action = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    replay(&mut g, action);
    passes(&mut g);
    assert_eq!(g.object_view(w).counters.loyalty, 8);
    let mut cost = Card::from_name("Burdened Stoneback").unwrap().activations()[0].cost;
    cost.remove = Some((None, 2));
    let available = Counters {
        plus_one: 1,
        minus_one: 1,
        stun: 1,
        loyalty: 1,
    };
    let mut payments = Vec::new();
    counter_payments(cost, available, |paid| {
        payments.push(paid);
        false
    });
    assert_eq!(payments.len(), 6);
    assert_eq!(payments.iter().filter(|p| p.loyalty == 1).count(), 3);
    assert!(payments.iter().all(|p| p.total() == 2));
    cost.remove = Some((Some(CounterKind::Loyalty), 1));
    payments.clear();
    counter_payments(cost, available, |paid| {
        payments.push(paid);
        false
    });
    assert_eq!(
        payments,
        vec![Counters {
            loyalty: 1,
            ..Counters::default()
        }]
    );
}
#[test]
fn loyalty_doubling_capacity_failure_restores_all_earlier_counter_writes() {
    let mut g = scenario();
    let w = walker(&mut g, Player::P0, "The Aetherspark");
    g.state.objects[w as usize].counters.loyalty = u64::MAX / 2 + 1;
    g.state.objects[w as usize].counters.plus_one = 1;
    let z = named(
        &mut g,
        Player::P0,
        "Zimone, Paradox Sculptor",
        Zone::Battlefield,
    );
    g.state.players[0].mana = [0, 1, 0, 0, 1, 0];
    let a = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a,Action::AnnounceActivate {card,..}if *card==z))
        .unwrap();
    act(&mut g, a);
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(w),
        },
    );
    act(&mut g, Action::FinishTargets);
    let a = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    act(&mut g, a);
    act(&mut g, Action::Pass);
    let saved = g.snapshot();
    let revision = g.revision();
    for _ in 0..2 {
        assert_eq!(
            g.apply(g.actor().unwrap(), Action::Pass),
            Err(Error::CapacityExceeded(crate::CapacityExceeded {
                resource: "counter representation exhausted"
            }))
        );
        assert_eq!(g.snapshot(), saved);
        assert_eq!(g.revision(), revision);
        assert_eq!(g.object_view(w).counters.plus_one, 1);
    }
}
#[test]
fn loyalty_wire_defaults_are_exact_and_zero_counters_are_compact() {
    let zero = serde_json::to_value(Counters::default()).unwrap();
    assert!(zero.get("loyalty").is_none());
    assert_eq!(
        serde_json::from_value::<Counters>(zero).unwrap(),
        Counters::default()
    );
    let value = Counters {
        loyalty: u64::MAX,
        ..Counters::default()
    };
    let json = serde_json::to_string(&value).unwrap();
    assert_eq!(serde_json::from_str::<Counters>(&json).unwrap(), value);
    assert_eq!(value.total(), u128::from(u64::MAX));
}
#[test]
fn c_abi_observes_exact_loyalty_and_replays_real_spell_damage_on_future_walker_fixture() {
    use std::ffi::{CStr, CString};
    struct Handle(*mut crate::protocol::Session);
    impl Drop for Handle {
        fn drop(&mut self) {
            unsafe { crate::ffi::rm_free(self.0) }
        }
    }
    fn request(handle: &Handle, text: &str) -> serde_json::Value {
        let input = CString::new(text).unwrap();
        let output = unsafe { crate::ffi::rm_request(handle.0, input.as_ptr()) };
        assert!(!output.is_null());
        let result: serde_json::Value =
            serde_json::from_str(unsafe { CStr::from_ptr(output) }.to_str().unwrap()).unwrap();
        unsafe { crate::ffi::rm_string_free(output) };
        assert!(result.get("error").is_none_or(|e| e.is_null()), "{result}");
        result
    }
    let mut g = scenario();
    let w = walker(&mut g, Player::P1, "Chandra, Torch of Defiance");
    let spell = funded(&mut g, "Sear");
    let a=g.legal_actions().into_iter().find(|a|matches!(a,Action::Cast {card,target:Some(Target::Card(t)),..}if *card==spell&&*t==w)).unwrap();
    replay(&mut g, a);
    let handle = Handle(Box::into_raw(Box::new(
        crate::protocol::Session::from_game(g),
    )));
    let before = request(&handle, r#"{"op":"observe","viewer":"P0"}"#);
    let old = before["observation"]["objects"]
        .as_array()
        .unwrap()
        .iter()
        .find(|o| o["id"] == w)
        .unwrap();
    assert_eq!(old["counters"]["loyalty"], 4);
    let saved = request(&handle, r#"{"op":"snapshot"}"#)["snapshot"]
        .as_u64()
        .unwrap();
    let fork = Handle(unsafe { crate::ffi::rm_clone(handle.0) });
    assert!(!fork.0.is_null());
    fn resolve(h: &Handle) -> serde_json::Value {
        for _ in 0..2 {
            let obs = request(h, r#"{"op":"observe","viewer":"P0"}"#)["observation"].clone();
            let actor = obs["actor"].clone();
            let viewer = actor.clone();
            let current = request(
                h,
                &serde_json::json!({"op":"observe","viewer":viewer}).to_string(),
            );
            let obs = &current["observation"];
            let index = obs["actions"]
                .as_array()
                .unwrap()
                .iter()
                .position(|a| a["kind"] == "Pass")
                .unwrap();
            request(h,&serde_json::json!({"op":"step","actor":actor,"viewer":actor,"revision":obs["revision"],"index":index}).to_string());
        }
        request(h, r#"{"op":"observe","viewer":"P0"}"#)
    }
    let actual = resolve(&handle);
    let expected = resolve(&fork);
    assert_eq!(actual, expected);
    let now = actual["observation"]["objects"]
        .as_array()
        .unwrap()
        .iter()
        .find(|o| o["id"] == w)
        .unwrap();
    assert_eq!(now["zone"], "Graveyard");
    assert!(now["counters"].get("loyalty").is_none());
    assert_eq!(now["damage"], 0);
    request(
        &handle,
        &serde_json::json!({"op":"restore","snapshot":saved}).to_string(),
    );
    let restored = request(&handle, r#"{"op":"observe","viewer":"P0"}"#);
    assert_eq!(
        restored["observation"]["objects"],
        before["observation"]["objects"]
    );
}
#[test]
fn actual_any_counter_activation_accepts_loyalty_payment_and_rejects_forgery_atomically() {
    let mut g = scenario();
    let source = named(&mut g, Player::P0, "Burdened Stoneback", Zone::Battlefield);
    let target = named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    g.state.objects[source as usize].counters.loyalty = 1;
    g.state.players[0].mana = [1, 0, 0, 0, 0, 1];
    let action = g.legal_actions().into_iter().find(|a| matches!(a, Action::BeginActivate { card, counter_payment, .. } if *card == source && counter_payment.loyalty == 1)).unwrap();
    let mut bad = action.clone();
    if let Action::BeginActivate {
        counter_payment, ..
    } = &mut bad
    {
        counter_payment.loyalty = 2;
    }
    let before = g.snapshot();
    let revision = g.revision();
    assert_eq!(g.apply(Player::P0, bad), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), revision);
    replay(&mut g, action);
    replay(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    assert_eq!(g.object_view(source).counters.loyalty, 0);
    passes(&mut g);
    assert!(g.keywords(target).contains(Keywords::INDESTRUCTIBLE));
    assert_eq!(g.object_view(source).zone, Zone::Battlefield);
}
