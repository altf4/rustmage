//! Future-walker fixtures exercise the combat kernel only. Their missing printed
//! abilities are not executed and submitted decks still reject those cards.
use super::tests::{act, scenario, spawn};
use super::*;

fn walker(g: &mut Game, owner: Player, name: &str) -> CardId {
    let card = Card::from_name(name).unwrap();
    assert!(!card.is_supported());
    let id = spawn(g, owner, card, Zone::Hand);
    g.move_object(id, Zone::Battlefield);
    id
}
fn bear(g: &mut Game, owner: Player) -> CardId {
    spawn(g, owner, Card::GrizzlyBears, Zone::Battlefield)
}
fn begin(g: &mut Game) {
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
}
fn attack(g: &Game, card: CardId, defender: CardId) -> Action {
    Action::AttackPlaneswalker {
        card,
        defender: g.object_ref(defender),
    }
}
fn replay(g: &mut Game, a: Action) {
    let before = g.snapshot();
    let revision = g.revision();
    let actor = g.actor().unwrap();
    let mut fork = g.clone();
    act(&mut fork, a.clone());
    act(g, a.clone());
    let after = g.snapshot();
    assert_eq!(after, fork.snapshot());
    g.restore(&before).unwrap();
    assert_eq!(g.apply_index(actor, revision, 0), Err(Error::StaleRevision));
    act(g, a);
    assert_eq!(after, g.snapshot());
}
fn passes(g: &mut Game) {
    replay(g, Action::Pass);
    replay(g, Action::Pass);
}
fn declare(g: &mut Game) {
    replay(g, Action::FinishAttackers);
    passes(g);
    assert_eq!(g.state.pending, Pending::Blockers);
}
fn combat(g: &mut Game) {
    replay(g, Action::FinishBlockers);
    passes(g);
}
fn reject(g: &mut Game, action: Action) {
    let before = g.snapshot();
    let revision = g.revision();
    assert_eq!(g.apply(Player::P0, action), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), revision);
}

#[test]
fn declaration_is_generation_bound_not_targeted_and_private_until_committed() {
    let mut g = scenario();
    let a = bear(&mut g, Player::P0);
    let w = walker(&mut g, Player::P1, "Chandra, Torch of Defiance");
    g.state.objects[w as usize].gained_keywords = Keywords::HEXPROOF;
    let own = walker(&mut g, Player::P0, "Vivien Reid");
    let other = bear(&mut g, Player::P1);
    begin(&mut g);
    let action = attack(&g, a, w);
    assert!(g.legal_actions().contains(&action));
    reject(
        &mut g,
        Action::AttackPlaneswalker {
            card: a,
            defender: ObjectRef {
                card: w,
                generation: u64::MAX,
            },
        },
    );
    for bad in [
        g.object_ref(own),
        g.object_ref(other),
        ObjectRef {
            card: CardId::MAX,
            generation: 0,
        },
    ] {
        reject(
            &mut g,
            Action::AttackPlaneswalker {
                card: a,
                defender: bad,
            },
        );
    }
    let before = g.snapshot();
    replay(&mut g, action.clone());
    assert!(g.observe(Player::P1).attack_declarations.is_empty());
    let selected = g.observe(Player::P0);
    assert_eq!(selected.attackers, vec![a]);
    assert_eq!(
        selected.attack_declarations[0].defender,
        Some(CombatDefender::Planeswalker(g.object_ref(w)))
    );
    assert!(!g.object_view(a).attacking);
    reject(&mut g, Action::Attack { card: a });
    reject(&mut g, action.clone());
    replay(&mut g, Action::FinishAttackers);
    assert!(g.object_view(a).attacking);
    assert!(g.object_view(a).tapped);
    assert_eq!(
        g.observe(Player::P1).attack_declarations,
        selected.attack_declarations
    );
    assert!(g.state.stack.is_empty() && g.state.pending_triggers.is_empty());
    assert!(g.state.players[0].attacked_this_turn);
    let after = g.snapshot();
    g.restore(&before).unwrap();
    act(&mut g, action);
    act(&mut g, Action::FinishAttackers);
    assert_eq!(g.snapshot(), after);
}

#[test]
fn creature_attack_constraints_apply_to_both_defender_choices() {
    let mut g = scenario();
    let a = bear(&mut g, Player::P0);
    let w = walker(&mut g, Player::P1, "Chandra, Torch of Defiance");
    begin(&mut g);
    for (tapped, sick, flags, legal) in [
        (true, false, Keywords::NONE, false),
        (false, true, Keywords::NONE, false),
        (false, true, Keywords::HASTE, true),
        (false, false, Keywords::DEFENDER, false),
        (false, false, Keywords::VIGILANCE, true),
    ] {
        let o = &mut g.state.objects[a as usize];
        o.tapped = tapped;
        o.summoning_sick = sick;
        o.gained_keywords = flags;
        assert_eq!(
            g.legal_actions().contains(&Action::Attack { card: a }),
            legal
        );
        assert_eq!(g.legal_actions().contains(&attack(&g, a, w)), legal);
    }
    let action = attack(&g, a, w);
    replay(&mut g, action);
    replay(&mut g, Action::FinishAttackers);
    assert!(!g.object_view(a).tapped);
}

#[test]
fn attackers_choose_separate_players_and_walkers_then_damage_is_simultaneous() {
    for active in [Player::P0, Player::P1] {
        let mut g = scenario();
        g.state.active = active;
        let a = bear(&mut g, active);
        let b = bear(&mut g, active);
        let c = bear(&mut g, active);
        let w = walker(&mut g, active.other(), "Chandra, Torch of Defiance");
        let v = walker(&mut g, active.other(), "Vivien Reid");
        begin(&mut g);
        replay(&mut g, Action::Attack { card: a });
        let action = attack(&g, b, w);
        replay(&mut g, action);
        let action = attack(&g, c, v);
        replay(&mut g, action);
        declare(&mut g);
        combat(&mut g);
        assert_eq!(g.state.players[active.other().index()].life, 18);
        assert_eq!(g.object_view(w).counters.loyalty, 2);
        assert_eq!(g.object_view(v).counters.loyalty, 3);
        assert_eq!(g.object_view(w).damage, 0);
        assert!(g.object_view(w).damaged_this_turn);
        passes(&mut g);
        passes(&mut g);
        assert_eq!(g.state.step, Step::Main2);
        assert!(g.observe(Player::P0).attack_declarations.is_empty());
    }
}

#[test]
fn blockers_defend_a_walker_and_can_still_fight_after_that_defender_leaves() {
    for removed in [false, true] {
        let mut g = scenario();
        let a = bear(&mut g, Player::P0);
        let b = bear(&mut g, Player::P1);
        let w = walker(&mut g, Player::P1, "Chandra, Torch of Defiance");
        begin(&mut g);
        let action = attack(&g, a, w);
        replay(&mut g, action);
        declare(&mut g);
        if removed {
            g.move_object(w, Zone::Hand);
            assert!(g.object_view(a).attacking);
        }
        replay(
            &mut g,
            Action::Block {
                blocker: b,
                attacker: a,
            },
        );
        combat(&mut g);
        assert_eq!(g.object_view(a).zone, Zone::Graveyard);
        assert_eq!(g.object_view(b).zone, Zone::Graveyard);
        assert_eq!(
            g.object_view(w).counters.loyalty,
            if removed { 0 } else { 4 }
        );
        assert_eq!(g.state.players[1].life, 20);
    }
}

#[test]
fn ordinary_trample_spills_to_the_walker_and_never_its_controller() {
    let mut g = scenario();
    let a = bear(&mut g, Player::P0);
    let b = bear(&mut g, Player::P1);
    g.state.objects[a as usize].growth = Stats {
        power: 8,
        toughness: 8,
    };
    g.state.objects[a as usize].gained_keywords = Keywords::TRAMPLE;
    let w = walker(&mut g, Player::P1, "Chandra, Torch of Defiance");
    begin(&mut g);
    let action = attack(&g, a, w);
    replay(&mut g, action);
    declare(&mut g);
    replay(
        &mut g,
        Action::Block {
            blocker: b,
            attacker: a,
        },
    );
    combat(&mut g);
    let bounds = g.damage_assignment_range().unwrap();
    assert_eq!((bounds.minimum, bounds.maximum), (2, 10));
    replay(
        &mut g,
        Action::AssignDamage {
            target: Target::Card(b),
            amount: 2,
        },
    );
    assert_eq!(g.object_view(b).zone, Zone::Graveyard);
    assert_eq!(g.object_view(w).zone, Zone::Graveyard);
    assert_eq!(g.state.players[1].life, 20);
    assert_eq!(g.object_view(a).damage, 2);
}

#[test]
fn departed_walker_does_not_receive_trample_or_redirect_to_player_or_new_incarnation() {
    for blocked in [false, true] {
        let mut g = scenario();
        let a = bear(&mut g, Player::P0);
        let b = bear(&mut g, Player::P1);
        g.state.objects[a as usize].growth = Stats {
            power: 8,
            toughness: 8,
        };
        g.state.objects[a as usize].gained_keywords = Keywords::TRAMPLE;
        let w = walker(&mut g, Player::P1, "Chandra, Torch of Defiance");
        begin(&mut g);
        let action = attack(&g, a, w);
        replay(&mut g, action);
        declare(&mut g);
        if blocked {
            replay(
                &mut g,
                Action::Block {
                    blocker: b,
                    attacker: a,
                },
            );
        }
        g.move_object(w, Zone::Hand);
        g.move_object(w, Zone::Battlefield);
        assert_eq!(g.observe(Player::P1).attack_declarations[0].defender, None);
        assert!(g.object_view(a).attacking);
        combat(&mut g);
        assert_eq!(g.object_view(w).counters.loyalty, 4);
        assert_eq!(g.state.players[1].life, 20);
        assert!(g.damage_assignment_range().is_none());
        if blocked {
            assert_eq!(g.object_view(b).zone, Zone::Graveyard);
        }
    }
}

#[test]
fn double_strike_kills_a_walker_in_first_strike_and_does_not_hit_player_in_normal_step() {
    let mut g = scenario();
    let a = bear(&mut g, Player::P0);
    g.state.objects[a as usize].growth = Stats {
        power: 2,
        toughness: 2,
    };
    g.state.objects[a as usize].gained_keywords = Keywords::DOUBLE_STRIKE
        .union(Keywords::TRAMPLE)
        .union(Keywords::LIFELINK);
    let w = walker(&mut g, Player::P1, "Chandra, Torch of Defiance");
    begin(&mut g);
    let action = attack(&g, a, w);
    replay(&mut g, action);
    declare(&mut g);
    combat(&mut g);
    assert_eq!(g.state.step, Step::FirstStrikeDamage);
    assert_eq!(g.object_view(w).zone, Zone::Graveyard);
    assert_eq!(g.state.players[0].life, 24);
    assert_eq!(g.observe(Player::P0).attack_declarations[0].defender, None);
    assert!(g.object_view(a).attacking);
    passes(&mut g);
    assert_eq!(g.state.step, Step::CombatDamage);
    assert_eq!(g.state.players[0].life, 24);
    assert_eq!(g.state.players[1].life, 20);
}

#[test]
fn normal_attacker_still_fights_blocker_after_another_attacker_kills_defender_in_first_strike() {
    let mut g = scenario();
    let a = bear(&mut g, Player::P0);
    let c = bear(&mut g, Player::P0);
    let b = bear(&mut g, Player::P1);
    g.state.objects[a as usize].growth = Stats {
        power: 2,
        toughness: 2,
    };
    g.state.objects[a as usize].gained_keywords = Keywords::FIRST_STRIKE;
    let w = walker(&mut g, Player::P1, "Chandra, Torch of Defiance");
    begin(&mut g);
    for id in [a, c] {
        let action = attack(&g, id, w);
        replay(&mut g, action);
    }
    declare(&mut g);
    replay(
        &mut g,
        Action::Block {
            blocker: b,
            attacker: c,
        },
    );
    combat(&mut g);
    assert_eq!(g.object_view(w).zone, Zone::Graveyard);
    assert_eq!(g.object_view(b).zone, Zone::Battlefield);
    passes(&mut g);
    assert_eq!(g.object_view(b).zone, Zone::Graveyard);
    assert_eq!(g.object_view(c).zone, Zone::Graveyard);
    assert_eq!(g.state.players[1].life, 20);
}

#[test]
fn damage_counter_facts_include_loyalty_in_last_known_zone_transition_data() {
    let mut g = scenario();
    let a = bear(&mut g, Player::P0);
    g.state.objects[a as usize].counters.loyalty = 1;
    let current = g.damage_facts(a);
    assert!(current.counter && current.modified && !current.plus_one);
    let moved = g.move_object(a, Zone::Hand);
    let lki = DamageFacts::from_view(&moved.before);
    assert_eq!(current, lki);
    assert!(!g.damage_facts(a).counter);
}

#[test]
fn exact_wide_trample_assignment_and_walker_damage_survive_every_search_boundary() {
    let mut g = scenario();
    let a = bear(&mut g, Player::P0);
    let b = bear(&mut g, Player::P1);
    let power = i128::from(u64::MAX) + 100;
    g.state.objects[a as usize].growth = Stats {
        power: power - 2,
        toughness: 10,
    };
    g.state.objects[a as usize].gained_keywords = Keywords::TRAMPLE;
    let w = walker(&mut g, Player::P1, "Chandra, Torch of Defiance");
    g.state.objects[w as usize].counters.loyalty = u64::MAX;
    begin(&mut g);
    let action = attack(&g, a, w);
    replay(&mut g, action);
    declare(&mut g);
    replay(
        &mut g,
        Action::Block {
            blocker: b,
            attacker: a,
        },
    );
    combat(&mut g);
    let range = g.damage_assignment_range().unwrap();
    assert_eq!((range.minimum, range.maximum), (2, power));
    let amount = power - 3;
    assert!(!g.legal_actions().contains(&Action::AssignDamage {
        target: Target::Card(b),
        amount
    }));
    let encoded = serde_json::to_string(&Action::AssignDamage {
        target: Target::Card(b),
        amount,
    })
    .unwrap();
    let action: Action = serde_json::from_str(&encoded).unwrap();
    replay(&mut g, action);
    assert_eq!(g.object_view(w).counters.loyalty, u64::MAX - 3);
    assert_eq!(g.object_view(w).damage, 0);
    assert_eq!(g.state.players[1].life, 20);
}

#[test]
fn real_c_abi_accepts_generation_bound_attack_and_owns_pending_defenders_across_clone_restore() {
    use std::ffi::{CStr, CString};
    struct Handle(*mut crate::protocol::Session);
    impl Drop for Handle {
        fn drop(&mut self) {
            unsafe { crate::ffi::rm_free(self.0) }
        }
    }
    fn request(h: &Handle, value: serde_json::Value) -> serde_json::Value {
        let input = CString::new(value.to_string()).unwrap();
        let output = unsafe { crate::ffi::rm_request(h.0, input.as_ptr()) };
        assert!(!output.is_null());
        let result =
            serde_json::from_str(unsafe { CStr::from_ptr(output) }.to_str().unwrap()).unwrap();
        unsafe { crate::ffi::rm_string_free(output) };
        result
    }
    fn observe(h: &Handle, p: &str) -> serde_json::Value {
        request(h, serde_json::json!({"op":"observe","viewer":p}))["observation"].clone()
    }
    fn step(h: &Handle, kind: &str) -> serde_json::Value {
        let actor = observe(h, "P0")["actor"].as_str().unwrap().to_owned();
        let obs = observe(h, &actor);
        let index = obs["actions"]
            .as_array()
            .unwrap()
            .iter()
            .position(|a| a["kind"] == kind)
            .unwrap();
        let result = request(
            h,
            serde_json::json!({"op":"step","actor":actor,"viewer":actor,"revision":obs["revision"],"index":index}),
        );
        assert!(result["error"].is_null(), "{result}");
        result["observation"].clone()
    }
    let mut g = scenario();
    let a = bear(&mut g, Player::P0);
    let w = walker(&mut g, Player::P1, "Chandra, Torch of Defiance");
    begin(&mut g);
    let reference = g.object_ref(w);
    let h = Handle(Box::into_raw(Box::new(
        crate::protocol::Session::from_game(g),
    )));
    let before = observe(&h, "P0");
    assert_eq!(before["schema"], 40);
    let saved = request(&h, serde_json::json!({"op":"snapshot"}))["snapshot"].clone();
    let mut action = serde_json::to_value(Action::AttackPlaneswalker {
        card: a,
        defender: reference,
    })
    .unwrap();
    action["defender"]["generation"] = serde_json::json!(u64::MAX);
    let forged = request(
        &h,
        serde_json::json!({"op":"act","actor":"P0","viewer":"P0","revision":before["revision"],"action":action}),
    );
    assert_eq!(forged["error"], "IllegalAction");
    assert_eq!(observe(&h, "P0"), before);
    let chosen = step(&h, "AttackPlaneswalker");
    assert_eq!(
        chosen["attack_declarations"][0]["defender"]["Planeswalker"],
        serde_json::to_value(reference).unwrap()
    );
    assert!(
        observe(&h, "P1")["attack_declarations"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    let fork = Handle(unsafe { crate::ffi::rm_clone(h.0) });
    assert!(!fork.0.is_null());
    fn finish(h: &Handle) -> serde_json::Value {
        for kind in [
            "FinishAttackers",
            "Pass",
            "Pass",
            "FinishBlockers",
            "Pass",
            "Pass",
        ] {
            step(h, kind);
        }
        observe(h, "P0")
    }
    let after = finish(&h);
    assert_eq!(after, finish(&fork));
    let walker = after["objects"]
        .as_array()
        .unwrap()
        .iter()
        .find(|o| o["id"] == w)
        .unwrap();
    assert_eq!(walker["counters"]["loyalty"], 2);
    assert_eq!(after["players"][1]["life"], 20);
    let restored = request(&h, serde_json::json!({"op":"restore","snapshot":saved}));
    assert!(restored["error"].is_null());
    let restored = observe(&h, "P0");
    assert_eq!(restored["objects"], before["objects"]);
    assert!(
        restored["attack_declarations"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    step(&h, "AttackPlaneswalker");
    let replayed = finish(&h);
    assert_eq!(replayed["objects"], after["objects"]);
    assert_eq!(
        replayed["attack_declarations"],
        after["attack_declarations"]
    );
}
