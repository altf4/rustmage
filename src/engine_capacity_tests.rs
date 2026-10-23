use super::tests::{act, cast, scenario, spawn};
use super::*;
fn named(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    spawn(g, p, Card::from_name(name).unwrap(), zone)
}
fn fail(g: &mut Game, action: Action, resource: &'static str) {
    let saved = g.snapshot();
    let revision = g.revision();
    let actor = g.actor().unwrap();
    let views = [
        serde_json::to_value(g.observe(Player::P0)).unwrap(),
        serde_json::to_value(g.observe(Player::P1)).unwrap(),
    ];
    for _ in 0..2 {
        assert_eq!(
            g.apply(actor, action.clone()),
            Err(Error::CapacityExceeded(crate::CapacityExceeded {
                resource
            }))
        );
        assert_eq!(g.snapshot(), saved);
        assert_eq!(g.revision(), revision);
        assert_eq!(
            serde_json::to_value(g.observe(Player::P0)).unwrap(),
            views[0]
        );
        assert_eq!(
            serde_json::to_value(g.observe(Player::P1)).unwrap(),
            views[1]
        );
    }
    let mut fork = g.clone();
    assert_eq!(
        fork.apply(actor, action),
        Err(Error::CapacityExceeded(crate::CapacityExceeded {
            resource
        }))
    );
    assert_eq!(fork.snapshot(), saved);
}
fn pay(g: &mut Game) {
    let action = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    act(g, action);
}
#[test]
fn mana_exhaustion_rolls_back_tap_payment_and_both_public_views() {
    let mut g = scenario();
    let land = named(&mut g, Player::P0, "Forest", Zone::Battlefield);
    g.state.players[0].mana[4] = u32::MAX;
    fail(
        &mut g,
        Action::TapMana { card: land },
        "mana representation exhausted",
    );
    assert!(!g.object_view(land).tapped);
    assert!(g.outcome().is_none());
    act(&mut g, Action::Pass);
}
#[test]
fn later_counter_field_failure_restores_earlier_writes_and_suspended_resolution() {
    let mut g = scenario();
    let z = named(
        &mut g,
        Player::P0,
        "Zimone, Paradox Sculptor",
        Zone::Battlefield,
    );
    let artifact = named(&mut g, Player::P0, "Mox Jasper", Zone::Battlefield);
    g.set_counters(
        artifact,
        Counters {
            lore: 0,
            loyalty: 0,
            plus_one: 1,
            minus_one: 0,
            stun: u64::MAX,
            ..Counters::default()
        },
    );
    g.state.players[0].mana = [0, 1, 0, 0, 1, 0];
    let action = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a,Action::AnnounceActivate{card,..}if *card==z))
        .unwrap();
    act(&mut g, action);
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(artifact),
        },
    );
    act(&mut g, Action::FinishTargets);
    pay(&mut g);
    act(&mut g, Action::Pass);
    fail(&mut g, Action::Pass, "counter representation exhausted");
    assert_eq!(g.object_view(artifact).counters.plus_one, 1);
    assert!(g.object_view(z).tapped);
    assert_eq!(g.state.stack.len(), 1);
}
#[test]
fn failed_spell_identity_and_generation_changes_restore_hand_mana_and_rng() {
    for generation in [false, true] {
        let mut g = scenario();
        let spell = named(&mut g, Player::P0, "Shock", Zone::Hand);
        g.state.players[0].mana[3] = 1;
        if generation {
            g.state.objects[spell as usize].generation = u64::MAX;
        } else {
            g.state.next_stack_id = u64::MAX;
        }
        fail(
            &mut g,
            Action::Cast {
                card: spell,
                target: Some(Target::Player(Player::P1)),
                payment: [0, 0, 0, 1, 0, 0],
            },
            if generation {
                "object generation exhausted"
            } else {
                "stack identity exhausted"
            },
        );
        assert_eq!(g.object_view(spell).zone, Zone::Hand);
        assert_eq!(g.state.players[0].mana[3], 1);
    }
}
#[test]
fn event_exhaustion_restores_completed_cast_and_draw_side_effects() {
    let mut g = scenario();
    let spell = named(&mut g, Player::P0, "Shock", Zone::Hand);
    g.state.players[0].mana[3] = 1;
    g.state.players[0].spells_cast_this_turn = u64::MAX;
    fail(
        &mut g,
        Action::Cast {
            card: spell,
            target: Some(Target::Player(Player::P1)),
            payment: [0, 0, 0, 1, 0, 0],
        },
        "spell count exhausted",
    );
    g.state.players[0].spells_cast_this_turn = 0;
    let draw = named(&mut g, Player::P0, "Divination", Zone::Hand);
    g.state.players[0].mana = [0, 1, 0, 0, 0, 2];
    cast(&mut g, draw, None);
    g.state.players[0].cards_drawn_this_turn = u64::MAX;
    act(&mut g, Action::Pass);
    fail(&mut g, Action::Pass, "draw ordinal exhausted");
}
#[test]
fn life_exhaustion_restores_batch_damage_and_lifelink_events() {
    let mut g = scenario();
    let hawk = named(&mut g, Player::P0, "Vampire Nighthawk", Zone::Battlefield);
    g.state.players[0].life = i128::MAX;
    g.state.attackers.push(Attacker {
        defender: Some(CombatDefender::Player(Player::P1)),
        card: hawk,
        blocked: false,
    });
    g.state.step = Step::Blockers;
    g.give_priority();
    act(&mut g, Action::Pass);
    fail(&mut g, Action::Pass, "life total exhausted");
    assert_eq!(g.state.players[1].life, 20);
}
#[test]
fn revision_and_turn_exhaustion_leave_everything_unchanged_and_restore_is_fallible() {
    let mut g = scenario();
    let saved = g.snapshot();
    g.revision = u64::MAX;
    fail(&mut g, Action::Pass, "revision exhausted");
    let before = g.snapshot();
    assert_eq!(
        g.restore(&saved),
        Err(Error::CapacityExceeded(crate::CapacityExceeded {
            resource: "revision exhausted"
        }))
    );
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), u64::MAX);
    let mut g = scenario();
    g.state.turn = u32::MAX;
    g.state.step = Step::Cleanup;
    g.state.passes = 1;
    fail(&mut g, Action::Pass, "turn count exhausted");
}
#[test]
fn observations_of_failed_funding_do_not_overflow_counterfactual_mana() {
    let mut g = scenario();
    let elf = named(&mut g, Player::P0, "Llanowar Elves", Zone::Battlefield);
    let factory = Card::catalogue()
        .find(|c| c.token_info().is_some_and(|t| t.name == "Treasure Token"))
        .unwrap();
    let token = spawn(&mut g, Player::P0, factory, Zone::Battlefield);
    g.state.players[0].mana[4] = u32::MAX;
    let spec = AdditionalCost {
        kind: EffectChoiceKind::Sacrifice,
        filter: SacrificeFilter {
            artifact_enchantment_or_token: false,
            keywords: Keywords::NONE,
            source_only: false,
            kinds: 16,
            nonland: false,
            other: false,
            tokens: None,
            subtype: None,
        },
    };
    let saved = g.snapshot();
    assert!(!g.can_fund(FundingGoal {
        actor: Player::P0,
        required: [0, 0, 0, 1, 0, 0],
        generic: 0,
        source: None,
        reserve: Some((g.object_ref(token), spec))
    }));
    assert_eq!(saved, g.snapshot());
    assert!(!g.object_view(elf).tapped);
    assert!(!g.observe(Player::P0).actions.is_empty());
}
#[test]
fn snapshots_and_branches_share_buffers_until_a_mutation_detaches_them() {
    let mut g = scenario();
    let land = named(&mut g, Player::P0, "Forest", Zone::Battlefield);
    let saved = g.snapshot();
    let mut fork = g.clone();
    assert!(g.state.objects.ptr_eq(&saved.0.objects));
    assert!(g.state.objects.ptr_eq(&fork.state.objects));
    assert!(
        g.state.players[0]
            .library
            .ptr_eq(&fork.state.players[0].library)
    );
    act(&mut fork, Action::TapMana { card: land });
    assert!(!g.state.objects.ptr_eq(&fork.state.objects));
    assert!(g.state.objects.ptr_eq(&saved.0.objects));
    assert!(!g.object_view(land).tapped);
    assert!(fork.object_view(land).tapped);
    let expected = fork.snapshot();
    act(&mut g, Action::TapMana { card: land });
    assert_eq!(g.snapshot(), expected);
    g.restore(&saved).unwrap();
    assert_eq!(g.snapshot(), saved);
}
#[test]
fn unrepresentable_derived_stats_cannot_escape_at_an_optional_resolution_boundary() {
    let mut g = scenario();
    let bear = named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    g.state.objects[bear as usize].growth = Stats {
        power: i128::MAX - 5,
        toughness: i128::MAX - 5,
    };
    let spell = named(&mut g, Player::P0, "Shock", Zone::Hand);
    let mut item = g.cast_item(Player::P0, spell, [0; 6]);
    item.targets.push(BoundTarget {
        target: Target::Card(bear),
        object: Some(g.object_ref(bear)),
        group: 0,
    });
    // Synthetic IR validates a suspended transaction, not a printed admission.
    g.state.resolving = Some(Resolution {
        wheel: None,
        continuation: None,
        chosen_creature_type: None,
        chosen_color: None,
        explore: None,
        evidence: None,
        item,
        program: &[
            Instruction::May { skip: 2 },
            Instruction::Grow {
                subject: Subject::Targets,
                power: 4,
                toughness: 4,
            },
            Instruction::May { skip: 0 },
        ],
        pc: 0,
        target_group: 0,
        moved_count: None,
        connive: None,
        mana_payment: None,
        forage: None,
        discards: [0; 2],
        selected_discards: [Vec::new(), Vec::new()],
        milled: Vec::new(),
        milled_selection: None,
        library: None,
        selection: None,
        target_library: None,
        search: None,
        optional_searches: None,
        choice: None,
    });
    g.run_program();
    assert_eq!(g.observe(Player::P0).decision, "optional_effect");
    fail(
        &mut g,
        Action::ChooseOptional { accept: true },
        "power representation exhausted",
    );
    act(&mut g, Action::ChooseOptional { accept: false });
    assert!(g.state.resolving.is_none());
    assert_eq!(g.object_view(bear).power, i128::MAX - 3);
}
#[test]
fn non_capacity_invariant_panics_are_not_mislabeled_as_numeric_errors() {
    let mut g = scenario();
    g.state.pending = Pending::Damage;
    g.state.damage_choices.clear();
    let saved = g.snapshot();
    let revision = g.revision();
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        g.apply(
            Player::P0,
            Action::AssignDamage {
                target: Target::Player(Player::P1),
                amount: 0,
            },
        )
    }));
    let payload = result.expect_err("invariant panic remains a bug");
    assert!(!payload.is::<CapacitySignal>());
    assert_eq!(g.snapshot(), saved);
    assert_eq!(g.revision(), revision);
}
#[test]
fn c_abi_reports_capacity_metadata_and_preserves_observation_snapshot_and_clone_calls() {
    use std::ffi::{CStr, CString};
    let mut g = scenario();
    let land = named(&mut g, Player::P0, "Forest", Zone::Battlefield);
    g.state.players[0].mana[4] = u32::MAX;
    let before = serde_json::to_value(g.observe(Player::P0)).unwrap();
    let revision = g.revision();
    struct Handle(*mut crate::protocol::Session);
    impl Drop for Handle {
        fn drop(&mut self) {
            unsafe { crate::ffi::rm_free(self.0) }
        }
    }
    fn request(handle: &Handle, input: &str) -> serde_json::Value {
        let input = CString::new(input).unwrap();
        unsafe {
            let pointer = crate::ffi::rm_request(handle.0, input.as_ptr());
            assert!(!pointer.is_null());
            let text = CStr::from_ptr(pointer).to_string_lossy().into_owned();
            crate::ffi::rm_string_free(pointer);
            serde_json::from_str(&text).unwrap()
        }
    }
    let handle = Handle(Box::into_raw(Box::new(
        crate::protocol::Session::from_game(g),
    )));
    let text=serde_json::json!({"op":"act","actor":"P0","revision":revision,"action":{"kind":"TapMana","card":land},"viewer":"P0"}).to_string();
    for _ in 0..2 {
        let error = request(&handle, &text);
        assert_eq!(
            error["capacity"]["resource"],
            "mana representation exhausted"
        );
        assert_eq!(error["schema"], 2);
        assert_eq!(error["revision"], revision);
        assert_eq!(
            request(&handle, r#"{"op":"observe","viewer":"P0"}"#)["observation"],
            before
        );
    }
    let fork = Handle(unsafe { crate::ffi::rm_clone(handle.0) });
    assert!(!fork.0.is_null());
    assert_eq!(
        request(&fork, r#"{"op":"observe","viewer":"P0"}"#)["observation"],
        before
    );
    let saved = request(&handle, r#"{"op":"snapshot"}"#)["snapshot"]
        .as_u64()
        .unwrap();
    let result = request(
        &handle,
        &format!(r#"{{"op":"restore","snapshot":{saved}}}"#),
    );
    assert!(result["error"].is_null());
    assert!(result["revision"].as_u64().unwrap() > revision);
}
