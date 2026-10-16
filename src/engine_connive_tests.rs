use super::tests::{act, cast, pass_pair, scenario, spawn};
use super::*;
fn named(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    let c = Card::from_name(name).unwrap();
    assert!(c.is_supported(), "{name}");
    spawn(g, p, c, zone)
}
fn replay(g: &mut Game, a: Action) {
    let before = g.snapshot();
    let mut fork = g.clone();
    act(&mut fork, a.clone());
    act(g, a.clone());
    let after = g.snapshot();
    assert_eq!(after, fork.snapshot());
    g.restore(&before).unwrap();
    act(g, a);
    assert_eq!(after, g.snapshot());
}
fn reject(g: &mut Game, a: Action) {
    let before = g.snapshot();
    let rev = g.revision;
    assert!(g.apply(g.actor().unwrap(), a).is_err());
    assert_eq!(before, g.snapshot());
    assert_eq!(rev, g.revision);
}
fn place(g: &mut Game) {
    g.prepare_triggers();
    while let Some(a) = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PutTrigger { .. }))
    {
        replay(g, a);
    }
}
fn settle(g: &mut Game) {
    for _ in 0..100 {
        place(g);
        if g.state.stack.is_empty() && g.state.resolving.is_none() {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::Discard { .. } | Action::Pass | Action::ChooseConniver { .. }
                )
            })
            .expect("fixture choice");
        replay(g, a);
    }
    panic!("fixture did not settle");
}
fn enter(g: &mut Game, name: &str) -> CardId {
    let id = named(g, Player::P0, name, Zone::Hand);
    g.move_object(id, Zone::Battlefield);
    place(g);
    pass_pair(g);
    id
}
fn pending_count(g: &Game, id: CardId) -> usize {
    g.state
        .pending_triggers
        .iter()
        .chain(&g.state.ordering_triggers)
        .chain(&g.state.stack)
        .filter(|s| s.card == id && matches!(s.kind, StackKind::TriggeredAbility { .. }))
        .count()
}
// Synthetic immutable IR exercises fixed N and multiple connivers; it enables no partial cards.
fn program(g: &mut Game, source: CardId, code: &'static [Instruction], targets: &[CardId]) {
    g.queue_trigger(&g.object_view(source), 0);
    let mut item = g.state.pending_triggers.pop().unwrap();
    item.targets = targets
        .iter()
        .map(|&id| g.bind_target(Target::Card(id)))
        .collect();
    g.state.resolving = Some(Resolution {
        connive: None,
        item,
        program: code,
        pc: 0,
        target_group: 0,
        moved_count: None,
        discards: [0; 2],
        selected_discards: [Vec::new(), Vec::new()],
        milled: Vec::new(),
        library: None,
        search: None,
        choice: None,
        mana_payment: None,
    });
    g.run_program();
}
fn announce(g: &mut Game, card: CardId) {
    g.state.players[0].mana = [10; 6];
    replay(g, Action::AnnounceCast { card });
}
fn pay(g: &mut Game) {
    let a = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    replay(g, a);
}

#[test]
fn source_connive_draws_then_privately_discards_and_only_nonlands_add_counters() {
    for nonland in [false, true] {
        let mut g = scenario();
        let discard = named(
            &mut g,
            Player::P0,
            if nonland { "Shock" } else { "Forest" },
            Zone::Hand,
        );
        let source = enter(&mut g, "Red Room Recruit");
        assert_eq!(
            g.state.pending,
            Pending::EffectDiscard {
                player: Player::P0,
                remaining: 1
            }
        );
        assert_eq!(g.state.players[0].cards_drawn_this_turn, 1);
        assert_eq!(g.object_view(source).counters.plus_one, 0);
        let public = g.observe(Player::P1);
        assert_eq!(public.schema, 50);
        assert_eq!(
            public
                .connive_choice
                .as_ref()
                .unwrap()
                .current
                .unwrap()
                .object,
            g.object_ref(source)
        );
        assert!(public.actions.is_empty());
        assert!(public.objects.iter().all(|o| o.id != discard));
        let library = g.state.players[0].library[0];
        reject(&mut g, Action::Discard { card: library });
        reject(&mut g, Action::Pass);
        replay(&mut g, Action::Discard { card: discard });
        assert_eq!(g.object_view(discard).zone, Zone::Graveyard);
        assert_eq!(g.object_view(source).counters.plus_one, u64::from(nonland));
        assert!(g.state.resolving.is_none());
    }
}

#[test]
fn source_lki_still_draws_and_discards_after_departure_without_growing_new_objects() {
    for destination in [Zone::Graveyard, Zone::Hand, Zone::Battlefield] {
        let mut g = scenario();
        let source = named(&mut g, Player::P0, "Red Room Recruit", Zone::Hand);
        let discard = named(&mut g, Player::P0, "Shock", Zone::Hand);
        g.move_object(source, Zone::Battlefield);
        place(&mut g);
        let original = g.object_ref(source);
        g.move_object(source, Zone::Graveyard);
        if destination != Zone::Graveyard {
            g.move_object(source, destination);
        }
        // A synthetic re-entry adds its own trigger; isolate the original captured one.
        g.state.pending_triggers.clear();
        pass_pair(&mut g);
        assert_eq!(g.state.players[0].cards_drawn_this_turn, 1);
        assert_eq!(
            g.observe(Player::P0)
                .connive_choice
                .unwrap()
                .current
                .unwrap()
                .object,
            original
        );
        replay(&mut g, Action::Discard { card: discard });
        assert_eq!(g.object_view(source).counters.plus_one, 0);
        assert_eq!(g.object_view(source).zone, destination);
    }
}

#[test]
fn source_reincarnation_during_private_choice_cannot_receive_the_original_counter() {
    let mut g = scenario();
    let discard = named(&mut g, Player::P0, "Shock", Zone::Hand);
    let source = enter(&mut g, "Red Room Recruit");
    let original = g.object_ref(source);
    g.move_object(source, Zone::Graveyard);
    g.move_object(source, Zone::Battlefield);
    assert_ne!(g.object_ref(source), original);
    g.state.pending_triggers.clear();
    replay(&mut g, Action::Discard { card: discard });
    assert_eq!(g.object_view(source).counters.plus_one, 0);
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 1);
}

#[test]
fn connive_defers_draw_and_discard_triggers_until_after_its_counter() {
    let mut g = scenario();
    let bird = named(&mut g, Player::P0, "Clinquant Skymage", Zone::Battlefield);
    let mako = named(&mut g, Player::P0, "Marauding Mako", Zone::Battlefield);
    g.draw(Player::P0);
    settle(&mut g);
    let discard = named(&mut g, Player::P0, "Shock", Zone::Hand);
    let source = enter(&mut g, "Madame Masque");
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 2);
    assert_eq!(g.object_view(bird).counters.plus_one, 1);
    assert_eq!(pending_count(&g, source), 1);
    assert!(g.state.stack.is_empty());
    replay(&mut g, Action::Discard { card: discard });
    assert_eq!(g.object_view(source).counters.plus_one, 1);
    assert_eq!(g.object_view(bird).counters.plus_one, 1);
    assert_eq!(g.object_view(mako).counters.plus_one, 0);
    settle(&mut g);
    assert_eq!(g.object_view(bird).counters.plus_one, 2);
    assert_eq!(g.object_view(mako).counters.plus_one, 1);
    assert!(g.state.objects.iter().any(|o| o.zone == Zone::Battlefield
        && o.card.token_info().is_some_and(|i| i.subtypes == "Villain")
        && o.card.keywords().contains(Keywords::MENACE)));
}

#[test]
fn scientists_keep_basic_landcycling_and_cast_entry_connive_as_separate_paths() {
    let mut g = scenario();
    let source = named(&mut g, Player::P0, "A.I.M. Scientists", Zone::Hand);
    g.state.players[0].mana = [10; 6];
    let a = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a,Action::Activate {card,..} if *card==source))
        .unwrap();
    replay(&mut g, a);
    pass_pair(&mut g);
    assert!(g.observe(Player::P0).search_choice.is_some());
    assert!(g.observe(Player::P0).connive_choice.is_none());
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 0);
    replay(&mut g, Action::FinishSearch);
    let cast_source = named(&mut g, Player::P0, "A.I.M. Scientists", Zone::Hand);
    cast(&mut g, cast_source, None);
    pass_pair(&mut g);
    pass_pair(&mut g);
    assert!(g.observe(Player::P0).connive_choice.is_some());
    let discard = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::Discard { .. }))
        .unwrap();
    replay(&mut g, discard);
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 1);
}

#[test]
fn flash_henchmen_attack_trigger_and_named_kang_attack_use_connive() {
    for name in ["Doc Ock's Henchmen", "Kang, Temporal Tyrant"] {
        let mut g = scenario();
        let discard = named(&mut g, Player::P0, "Shock", Zone::Hand);
        let source = named(&mut g, Player::P0, name, Zone::Hand);
        g.state.players[0].mana = [10; 6];
        if name == "Doc Ock's Henchmen" {
            g.state.step = Step::Upkeep;
        }
        cast(&mut g, source, None);
        pass_pair(&mut g);
        g.begin_turn();
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        if name == "Kang, Temporal Tyrant" {
            g.draw(Player::P0);
        }
        replay(&mut g, Action::Attack { card: source });
        replay(&mut g, Action::FinishAttackers);
        pass_pair(&mut g);
        assert!(g.observe(Player::P0).connive_choice.is_some());
        replay(&mut g, Action::Discard { card: discard });
        settle(&mut g);
        assert_eq!(g.object_view(source).counters.plus_one, 1);
        if name == "Kang, Temporal Tyrant" {
            assert_eq!(g.state.players[0].life, 21);
            assert_eq!(g.state.players[1].life, 19);
        }
    }
}

#[test]
fn subtype_enter_observers_require_another_owned_villain_and_keep_explicit_source_binding() {
    for (owner, name, triggered) in [
        (Player::P0, "Red Room Recruit", true),
        (Player::P1, "Red Room Recruit", false),
        (Player::P0, "Grizzly Bears", false),
    ] {
        let mut g = scenario();
        let prowler = named(
            &mut g,
            Player::P0,
            "Prowler, Clawed Thief",
            Zone::Battlefield,
        );
        let yellow = named(
            &mut g,
            Player::P0,
            "Yellowjacket, Heartless Marauder",
            Zone::Battlefield,
        );
        let entrant = named(&mut g, owner, name, Zone::Hand);
        g.move_object(entrant, Zone::Battlefield);
        assert_eq!(pending_count(&g, prowler), usize::from(triggered));
        assert_eq!(pending_count(&g, yellow), usize::from(triggered));
        settle(&mut g);
        assert_eq!(
            g.state.objects[yellow as usize].growth.power,
            i128::from(triggered)
        );
        assert_eq!(g.keywords(yellow).contains(Keywords::LIFELINK), triggered);
    }
    let mut g = scenario();
    let source = named(&mut g, Player::P0, "Prowler, Clawed Thief", Zone::Hand);
    g.move_object(source, Zone::Battlefield);
    assert_eq!(pending_count(&g, source), 0);
}

#[test]
fn independent_player_draw_and_optional_conniver_target_groups_preserve_player_ownership() {
    for choose in [false, true] {
        let mut g = scenario();
        let target = named(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
        let discard = named(&mut g, Player::P0, "Shock", Zone::Hand);
        let spell = named(&mut g, Player::P0, "Unstable Experiment", Zone::Hand);
        announce(&mut g, spell);
        replay(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(Player::P1),
            },
        );
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(Player::P0),
            },
        );
        if choose {
            replay(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(target),
                },
            );
        } else {
            replay(&mut g, Action::FinishTargets);
        }
        pay(&mut g);
        pass_pair(&mut g);
        assert_eq!(g.state.players[1].cards_drawn_this_turn, 1);
        assert_eq!(g.state.players[0].cards_drawn_this_turn, u64::from(choose));
        if choose {
            assert_eq!(g.actor(), Some(Player::P0));
            replay(&mut g, Action::Discard { card: discard });
        }
        assert_eq!(g.object_view(target).counters.plus_one, u64::from(choose));
        assert!(g.state.resolving.is_none());
    }
}

#[test]
fn invalid_optional_creature_target_does_not_prevent_the_independent_player_draw() {
    let mut g = scenario();
    let target = named(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let spell = named(&mut g, Player::P0, "Unstable Experiment", Zone::Hand);
    announce(&mut g, spell);
    replay(
        &mut g,
        Action::ChooseTarget {
            target: Target::Player(Player::P1),
        },
    );
    replay(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    pay(&mut g);
    g.move_object(target, Zone::Graveyard);
    g.move_object(target, Zone::Battlefield);
    pass_pair(&mut g);
    assert_eq!(g.state.players[1].cards_drawn_this_turn, 1);
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 0);
    assert!(g.state.resolving.is_none());
    assert_eq!(g.object_view(target).counters.plus_one, 0);
}

#[test]
fn targeted_connive_uses_the_permanents_controller_instead_of_the_ability_controller() {
    static CODE: &[Instruction] = &[Instruction::Connive {
        subject: Subject::Targets,
        amount: 1,
    }];
    let mut g = scenario();
    let source = named(&mut g, Player::P0, "Red Room Recruit", Zone::Battlefield);
    let target = named(&mut g, Player::P1, "Grizzly Bears", Zone::Battlefield);
    let discard = named(&mut g, Player::P1, "Shock", Zone::Hand);
    program(&mut g, source, CODE, &[target]);
    assert_eq!(g.actor(), Some(Player::P1));
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 0);
    assert_eq!(g.state.players[1].cards_drawn_this_turn, 1);
    assert!(g.observe(Player::P0).actions.is_empty());
    let before = g.snapshot();
    assert_eq!(
        g.apply(Player::P0, Action::Discard { card: discard }),
        Err(Error::WrongActor)
    );
    assert_eq!(g.snapshot(), before);
    replay(&mut g, Action::Discard { card: discard });
    assert_eq!(g.object_view(target).counters.plus_one, 1);
}

#[test]
fn fixed_connive_n_counts_only_nonland_discards_and_waits_for_all_selections() {
    static CODE: &[Instruction] = &[Instruction::Connive {
        subject: Subject::Source,
        amount: 3,
    }];
    let mut g = scenario();
    let source = named(&mut g, Player::P0, "Red Room Recruit", Zone::Battlefield);
    let mako = named(&mut g, Player::P0, "Marauding Mako", Zone::Battlefield);
    let a = named(&mut g, Player::P0, "Shock", Zone::Hand);
    let b = named(&mut g, Player::P0, "Giant Growth", Zone::Hand);
    let c = named(&mut g, Player::P0, "Forest", Zone::Hand);
    program(&mut g, source, CODE, &[]);
    assert_eq!(g.observe(Player::P0).connive_choice.unwrap().amount, 3);
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 3);
    replay(&mut g, Action::Discard { card: a });
    assert_eq!(g.object_view(a).zone, Zone::Hand);
    assert_eq!(g.object_view(source).counters.plus_one, 0);
    reject(&mut g, Action::Discard { card: a });
    replay(&mut g, Action::Discard { card: b });
    assert_eq!(g.object_view(source).counters.plus_one, 0);
    replay(&mut g, Action::Discard { card: c });
    assert_eq!(g.object_view(source).counters.plus_one, 2);
    assert_eq!(g.state.stack[0].discard_event.unwrap().amount, 3);
    assert_eq!(g.state.stack[0].card, mako);
    settle(&mut g);
    assert_eq!(g.object_view(mako).counters.plus_one, 3);
}

#[test]
fn connive_zero_has_no_draw_discard_or_choice_and_resumes_the_following_instruction() {
    static CODE: &[Instruction] = &[
        Instruction::Connive {
            subject: Subject::Source,
            amount: 0,
        },
        Instruction::GainLife(PlayerSet::Controller, 1),
    ];
    let mut g = scenario();
    let source = named(&mut g, Player::P0, "Red Room Recruit", Zone::Battlefield);
    program(&mut g, source, CODE, &[]);
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 0);
    assert_eq!(g.state.players[0].life, 21);
    assert_eq!(g.object_view(source).counters.plus_one, 0);
    assert!(g.observe(Player::P0).connive_choice.is_none());
}

#[test]
fn short_hands_and_failed_draws_finish_connive_before_state_based_loss() {
    static CODE: &[Instruction] = &[Instruction::Connive {
        subject: Subject::Source,
        amount: 3,
    }];
    let mut g = scenario();
    let source = named(&mut g, Player::P0, "Red Room Recruit", Zone::Battlefield);
    let ids = g.state.players[0].library.clone();
    for id in ids {
        g.move_object(id, Zone::Graveyard);
    }
    let a = named(&mut g, Player::P0, "Shock", Zone::Hand);
    let b = named(&mut g, Player::P0, "Giant Growth", Zone::Library);
    program(&mut g, source, CODE, &[]);
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 1);
    assert!(g.outcome().is_none());
    replay(&mut g, Action::Discard { card: a });
    assert!(g.outcome().is_none());
    replay(&mut g, Action::Discard { card: b });
    assert_eq!(g.object_view(source).counters.plus_one, 2);
    assert!(g.state.resolving.is_none());
    assert_eq!(g.outcome(), Some(Outcome::Winner(Player::P1)));
}

#[test]
fn multiple_connivers_offer_apnap_order_and_finish_each_private_operation_before_the_next() {
    static CODE: &[Instruction] = &[Instruction::Connive {
        subject: Subject::Targets,
        amount: 1,
    }];
    let mut g = scenario();
    let source = named(&mut g, Player::P0, "Red Room Recruit", Zone::Battlefield);
    let a = named(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let b = named(&mut g, Player::P0, "Hill Giant", Zone::Battlefield);
    let c = named(&mut g, Player::P1, "Grizzly Bears", Zone::Battlefield);
    let da = named(&mut g, Player::P0, "Shock", Zone::Hand);
    let db = named(&mut g, Player::P0, "Giant Growth", Zone::Hand);
    program(&mut g, source, CODE, &[c, a, b]);
    assert_eq!(
        g.state.pending,
        Pending::ConniveOrder { player: Player::P0 }
    );
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 0);
    let ctx = g
        .state
        .resolving
        .as_ref()
        .unwrap()
        .connive
        .as_ref()
        .unwrap();
    let expected = ctx.bytes();
    let mut stripped = g.clone();
    stripped.state.resolving.as_mut().unwrap().connive = None;
    assert_eq!(g.state_bytes() - stripped.state_bytes(), expected);
    let a_ref = g.object_ref(a);
    let b_ref = g.object_ref(b);
    let c_ref = g.object_ref(c);
    reject(&mut g, Action::ChooseConniver { object: c_ref });
    let saved = g.snapshot();
    assert_eq!(
        g.apply(Player::P1, Action::ChooseConniver { object: b_ref }),
        Err(Error::WrongActor)
    );
    assert_eq!(saved, g.snapshot());
    replay(&mut g, Action::ChooseConniver { object: b_ref });
    assert_eq!(
        g.observe(Player::P0)
            .connive_choice
            .unwrap()
            .current
            .unwrap()
            .object,
        b_ref
    );
    replay(&mut g, Action::Discard { card: db });
    assert_eq!(g.object_view(b).counters.plus_one, 1);
    assert_eq!(
        g.observe(Player::P0)
            .connive_choice
            .unwrap()
            .current
            .unwrap()
            .object,
        a_ref
    );
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 2);
    replay(&mut g, Action::Discard { card: da });
    assert_eq!(g.object_view(a).counters.plus_one, 1);
    assert_eq!(g.actor(), Some(Player::P1));
    assert_eq!(g.state.players[1].cards_drawn_this_turn, 1);
    let discard = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::Discard { .. }))
        .unwrap();
    replay(&mut g, discard);
    assert!(g.state.resolving.is_none());
    assert_eq!(g.object_view(c).counters.plus_one, 0);
}

#[test]
fn conniver_choice_retains_old_incarnations_and_rejects_a_replacement_reference() {
    static CODE: &[Instruction] = &[Instruction::Connive {
        subject: Subject::Targets,
        amount: 1,
    }];
    let mut g = scenario();
    let source = named(&mut g, Player::P0, "Red Room Recruit", Zone::Battlefield);
    let a = named(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let b = named(&mut g, Player::P0, "Hill Giant", Zone::Battlefield);
    let discard = named(&mut g, Player::P0, "Shock", Zone::Hand);
    program(&mut g, source, CODE, &[a, b]);
    let original = g.object_ref(a);
    g.move_object(a, Zone::Graveyard);
    g.move_object(a, Zone::Battlefield);
    let replacement = g.object_ref(a);
    reject(
        &mut g,
        Action::ChooseConniver {
            object: replacement,
        },
    );
    replay(&mut g, Action::ChooseConniver { object: original });
    replay(&mut g, Action::Discard { card: discard });
    assert_eq!(g.object_view(a).counters.plus_one, 0);
    settle(&mut g);
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 2);
}

#[test]
fn mechanical_mobster_keeps_optional_exile_and_required_conniver_as_independent_groups() {
    for exile in [false, true] {
        let mut g = scenario();
        let grave = named(&mut g, Player::P1, "Forest", Zone::Graveyard);
        let target = named(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
        let discard = named(&mut g, Player::P0, "Shock", Zone::Hand);
        let source = named(&mut g, Player::P0, "Mechanical Mobster", Zone::Hand);
        g.move_object(source, Zone::Battlefield);
        place(&mut g);
        if exile {
            replay(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(grave),
                },
            );
        } else {
            replay(&mut g, Action::FinishTargets);
        }
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(grave),
            },
        );
        replay(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        pass_pair(&mut g);
        assert_eq!(
            g.object_view(grave).zone,
            if exile { Zone::Exile } else { Zone::Graveyard }
        );
        replay(&mut g, Action::Discard { card: discard });
        assert_eq!(g.object_view(target).counters.plus_one, 1);
    }
}

#[test]
fn optional_exile_tail_life_or_draw_runs_when_omitted_but_fizzles_if_the_only_target_leaves() {
    for selected in [false, true] {
        let mut g = scenario();
        let grave = named(&mut g, Player::P1, "Forest", Zone::Graveyard);
        let source = named(&mut g, Player::P0, "Wreck Remover", Zone::Hand);
        g.move_object(source, Zone::Battlefield);
        place(&mut g);
        if selected {
            replay(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(grave),
                },
            );
            g.move_object(grave, Zone::Hand);
        } else {
            replay(&mut g, Action::FinishTargets);
        }
        pass_pair(&mut g);
        assert_eq!(g.state.players[0].life, if selected { 20 } else { 21 });
    }
    let mut g = scenario();
    named(&mut g, Player::P1, "Forest", Zone::Graveyard);
    let spell = named(&mut g, Player::P0, "Heritage Reclamation", Zone::Hand);
    announce(&mut g, spell);
    replay(&mut g, Action::ChooseMode { mode: 2 });
    replay(&mut g, Action::FinishTargets);
    pay(&mut g);
    pass_pair(&mut g);
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 1);
}

#[test]
fn explicit_named_attack_normalization_preserves_other_creature_filter_and_first_strike() {
    let mut g = scenario();
    let source = named(
        &mut g,
        Player::P0,
        "Syr Alin, the Lion's Claw",
        Zone::Battlefield,
    );
    let other = named(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    replay(&mut g, Action::Attack { card: source });
    replay(&mut g, Action::FinishAttackers);
    settle(&mut g);
    assert_eq!(g.state.objects[source as usize].growth, Stats::default());
    assert_eq!(
        g.state.objects[other as usize].growth,
        Stats {
            power: 1,
            toughness: 1
        }
    );
    assert!(g.keywords(source).contains(Keywords::FIRST_STRIKE));
}
