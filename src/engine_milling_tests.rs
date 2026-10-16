use super::tests::{act, cast, pass_pair, scenario, spawn};
use super::*;
fn named(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    let card = Card::from_name(name).unwrap();
    assert!(card.is_supported(), "{name}");
    spawn(g, p, card, zone)
}
fn replay(g: &mut Game, action: Action) {
    let before = g.snapshot();
    let mut fork = g.clone();
    act(&mut fork, action.clone());
    act(g, action.clone());
    let after = g.snapshot();
    assert_eq!(after, fork.snapshot());
    g.restore(&before);
    act(g, action);
    assert_eq!(g.snapshot(), after);
}
fn play_spell(g: &mut Game, name: &str, target: Option<Target>) -> CardId {
    let card = named(g, Player::P0, name, Zone::Hand);
    g.state.players[0].mana = [30; 6];
    cast(g, card, target);
    pass_pair(g);
    card
}
fn enter(g: &mut Game, name: &str) -> CardId {
    let id = named(g, Player::P0, name, Zone::Hand);
    g.move_object(id, Zone::Battlefield);
    g.prepare_triggers();
    pass_pair(g);
    id
}
fn reject(g: &mut Game, a: Action) {
    let before = g.snapshot();
    assert!(g.apply(g.actor().unwrap(), a).is_err());
    assert_eq!(g.snapshot(), before);
}
// A synthetic immutable program exercises generic instruction composition; this
// helper does not admit a partial card as a complete Standard definition.
fn program(g: &mut Game, code: &'static [Instruction]) {
    let source = named(g, Player::P0, "Helpful Hunter", Zone::Hand);
    g.move_object(source, Zone::Battlefield);
    g.prepare_triggers();
    let item = g.state.stack.pop().unwrap();
    g.state.resolving = Some(Resolution {
        connive: None,
        item,
        program: code,
        pc: 0,
        target_group: 0,
        discards: [0; 2],
        mana_payment: None,
        selected_discards: [Vec::new(), Vec::new()],
        milled: Vec::new(),
        library: None,
        search: None,
        choice: None,
    });
    g.run_program();
}

#[test]
fn fixed_mill_moves_top_cards_as_new_public_objects_without_enter_or_death_events() {
    let mut g = scenario();
    named(&mut g, Player::P0, "Midnight Reaper", Zone::Battlefield);
    let hunter = named(&mut g, Player::P0, "Helpful Hunter", Zone::Library);
    let crow = named(&mut g, Player::P0, "Crow of Dark Tidings", Zone::Library);
    let refs = [g.object_ref(hunter), g.object_ref(crow)];
    let n = g.state.players[0].library.len();
    enter(&mut g, "Scarblade Scout");
    assert_eq!(g.state.players[0].library.len(), n - 2);
    for r in refs {
        let o = g.object_view(r.card);
        assert_eq!(o.zone, Zone::Graveyard);
        assert_eq!(o.reference.generation, r.generation + 1);
        assert!(
            g.observe(Player::P1)
                .objects
                .iter()
                .any(|o| o.reference == g.object_ref(r.card))
        );
    }
    assert!(g.state.pending_triggers.is_empty() && g.state.stack.is_empty());
    assert!(!g.state.players[0].attempted_empty_draw);
}

#[test]
fn mandatory_mill_short_and_empty_libraries_do_not_lose_the_game() {
    for remaining in 0..=2 {
        let mut g = scenario();
        let ids = g.state.players[0].library.clone();
        for &id in &ids[remaining..] {
            g.move_object(id, Zone::Exile);
        }
        enter(&mut g, "Shinra Reinforcements");
        assert!(g.state.players[0].library.is_empty());
        assert_eq!(g.state.players[0].life, 23);
        assert!(!g.state.players[0].attempted_empty_draw);
        assert!(g.outcome().is_none());
    }
}

#[test]
fn optional_mill_requires_entire_quantity_and_decline_is_atomic_replayable() {
    for remaining in 0..=2 {
        let mut g = scenario();
        let ids = g.state.players[0].library.clone();
        for &id in &ids[remaining..] {
            g.move_object(id, Zone::Exile);
        }
        enter(&mut g, "Deathcap Marionette");
        if remaining < 2 {
            assert!(g.state.resolving.is_none());
            assert_eq!(g.state.players[0].library.len(), remaining);
        } else {
            assert_eq!(g.observe(Player::P0).decision, "optional_effect");
            let before = g.snapshot();
            replay(&mut g, Action::ChooseOptional { accept: false });
            assert_eq!(g.state.players[0].library.len(), 2);
            g.restore(&before);
            replay(&mut g, Action::ChooseOptional { accept: true });
            assert!(g.state.players[0].library.is_empty());
        }
    }
}

#[test]
fn optional_mill_unavailable_skips_only_mill_and_continues() {
    static CODE: &[Instruction] = &[
        Instruction::MayMill(100),
        Instruction::Mill(PlayerSet::Controller, MillAmount::Fixed(100)),
        Instruction::GainLife(PlayerSet::Controller, 4),
    ];
    let mut g = scenario();
    program(&mut g, CODE);
    assert_eq!(g.state.players[0].life, 24);
    assert_eq!(g.state.players[0].library.len(), 20);
    assert!(g.state.resolving.is_none());
}

#[test]
fn recovery_uses_exact_milled_batch_and_public_hand_knowledge() {
    let mut g = scenario();
    let old = named(&mut g, Player::P0, "Venomized Cat", Zone::Graveyard);
    let creature = named(&mut g, Player::P0, "Venomized Cat", Zone::Library);
    let instant = named(&mut g, Player::P0, "Giant Growth", Zone::Library);
    let aura = named(&mut g, Player::P0, "Dead Weight", Zone::Library);
    let land = named(&mut g, Player::P0, "Swamp", Zone::Library);
    let spell = play_spell(&mut g, "Midnight Tilling", None);
    let view = g.observe(Player::P0).recovery_choice.unwrap();
    assert_eq!(view.filter, RecoveryFilter::Permanent);
    assert_eq!(
        view.milled.iter().map(|r| r.card).collect::<Vec<_>>(),
        vec![land, aura, instant, creature]
    );
    assert_eq!(
        view.candidates.iter().map(|r| r.card).collect::<Vec<_>>(),
        vec![land, aura, creature]
    );
    assert_eq!(g.object_view(spell).zone, Zone::Stack);
    assert_eq!(
        g.observe(Player::P1).recovery_choice.unwrap().candidates,
        view.candidates
    );
    assert!(g.observe(Player::P1).actions.is_empty());
    for id in [old, instant] {
        let object = g.object_ref(id);
        reject(
            &mut g,
            Action::RecoverMilled {
                object: Some(object),
            },
        );
    }
    reject(&mut g, Action::Pass);
    reject(
        &mut g,
        Action::RecoverMilled {
            object: Some(ObjectRef {
                card: u32::MAX,
                generation: 0,
            }),
        },
    );
    let object = g.object_ref(creature);
    let snapshot = g.snapshot();
    assert!(
        g.apply(
            Player::P1,
            Action::RecoverMilled {
                object: Some(object)
            }
        )
        .is_err()
    );
    assert_eq!(g.snapshot(), snapshot);
    replay(
        &mut g,
        Action::RecoverMilled {
            object: Some(object),
        },
    );
    assert_eq!(g.object_view(creature).zone, Zone::Hand);
    assert!(g.state.known_hand.contains(&g.object_ref(creature)));
    assert!(
        g.observe(Player::P1)
            .objects
            .iter()
            .any(|o| o.id == creature)
    );
    assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
    g.state.players[0].mana = [10; 6];
    cast(&mut g, creature, None);
    assert!(!g.state.known_hand.iter().any(|r| r.card == creature));
}

#[test]
fn recovery_cannot_follow_blinked_milled_incarnation_or_foreign_card() {
    let mut g = scenario();
    play_spell(&mut g, "Midnight Tilling", None);
    let old = g.observe(Player::P0).recovery_choice.unwrap().candidates[0];
    g.move_object(old.card, Zone::Exile);
    g.move_object(old.card, Zone::Graveyard);
    let foreign = named(&mut g, Player::P1, "Forest", Zone::Graveyard);
    let candidates = g.observe(Player::P0).recovery_choice.unwrap().candidates;
    assert!(!candidates.iter().any(|r| r.card == old.card));
    reject(&mut g, Action::RecoverMilled { object: Some(old) });
    let fresh = g.object_ref(old.card);
    reject(
        &mut g,
        Action::RecoverMilled {
            object: Some(fresh),
        },
    );
    let foreign = g.object_ref(foreign);
    reject(
        &mut g,
        Action::RecoverMilled {
            object: Some(foreign),
        },
    );
    replay(&mut g, Action::RecoverMilled { object: None });
}

#[test]
fn recovery_decline_and_no_candidates_preserve_following_effects() {
    let mut g = scenario();
    let spell = play_spell(&mut g, "Rapid Rescue", None);
    assert_eq!(g.state.players[0].life, 20);
    replay(&mut g, Action::RecoverMilled { object: None });
    assert_eq!(g.state.players[0].life, 22);
    assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
    named(&mut g, Player::P0, "Giant Growth", Zone::Library);
    named(&mut g, Player::P0, "Giant Growth", Zone::Library);
    play_spell(&mut g, "Rapid Rescue", None);
    assert!(g.state.resolving.is_none());
    assert_eq!(g.state.players[0].life, 24);
}

#[test]
fn sba_waits_for_recovery_and_later_life_gain() {
    let mut g = scenario();
    let spell = named(&mut g, Player::P0, "Something Worth Saving", Zone::Hand);
    g.state.players[0].mana = [20; 6];
    cast(&mut g, spell, None);
    // Synthetic state forces a resolution-time SBA boundary. Returning the
    // milled card is not a new priority window before the following life gain.
    g.state.players[0].life = 0;
    g.resolve_top();
    assert_eq!(g.observe(Player::P0).decision, "recover_milled");
    assert!(g.outcome().is_none());
    let object = g.observe(Player::P0).recovery_choice.unwrap().candidates[0];
    replay(
        &mut g,
        Action::RecoverMilled {
            object: Some(object),
        },
    );
    assert_eq!(g.state.players[0].life, 1);
    assert!(g.outcome().is_none());
}

#[test]
fn each_player_mill_freezes_both_libraries_and_replaces_previous_batch() {
    static CODE: &[Instruction] = &[
        Instruction::Mill(PlayerSet::Each, MillAmount::Fixed(2)),
        Instruction::RecoverMilled(RecoveryFilter::Permanent),
        Instruction::Mill(PlayerSet::Controller, MillAmount::Fixed(1)),
        Instruction::RecoverMilled(RecoveryFilter::Land),
    ];
    let mut g = scenario();
    g.state.active = Player::P1;
    program(&mut g, CODE);
    let choice = g.observe(Player::P0).recovery_choice.unwrap();
    assert_eq!(choice.milled.len(), 4);
    assert_eq!(choice.candidates.len(), 2);
    assert!(
        choice
            .candidates
            .iter()
            .all(|r| g.object_view(r.card).owner == Player::P0)
    );
    assert_eq!(g.state.players[0].library.len(), 18);
    assert_eq!(g.state.players[1].library.len(), 18);
    let first = choice.candidates[0];
    replay(&mut g, Action::RecoverMilled { object: None });
    let next = g.observe(Player::P0).recovery_choice.unwrap();
    assert_eq!(next.milled.len(), 1);
    assert!(!next.milled.contains(&first));
    reject(
        &mut g,
        Action::RecoverMilled {
            object: Some(first),
        },
    );
    replay(
        &mut g,
        Action::RecoverMilled {
            object: Some(next.candidates[0]),
        },
    );
}

#[test]
fn targeted_mill_uses_selected_player_and_life_at_resolution() {
    for target in [Player::P0, Player::P1] {
        let mut g = scenario();
        let spell = named(&mut g, Player::P0, "Space-Time Anomaly", Zone::Hand);
        g.state.players[0].mana = [20; 6];
        cast(&mut g, spell, Some(Target::Player(target)));
        g.state.players[0].life = 7;
        pass_pair(&mut g);
        assert_eq!(g.state.players[target.index()].library.len(), 13);
        assert_eq!(g.state.players[target.other().index()].library.len(), 20);
    }
}

#[test]
fn dynamic_mill_nonpositive_life_and_extreme_life_are_bounded_by_library() {
    for life in [-3, 0, i128::MAX] {
        let mut g = scenario();
        let spell = named(&mut g, Player::P0, "Space-Time Anomaly", Zone::Hand);
        g.state.players[0].mana = [20; 6];
        cast(&mut g, spell, Some(Target::Player(Player::P1)));
        g.state.players[0].life = life;
        g.resolve_top();
        assert_eq!(
            g.state.players[1].library.len(),
            if life > 0 { 0 } else { 20 }
        );
        assert!(!g.state.players[1].attempted_empty_draw);
    }
}

#[test]
fn entry_death_mill_does_not_trigger_from_other_zones_and_survives_source_departure() {
    let mut g = scenario();
    let crow = enter(&mut g, "Crow of Dark Tidings");
    assert_eq!(g.state.players[0].library.len(), 18);
    let old = g.object_ref(crow);
    g.move_object(crow, Zone::Graveyard);
    g.prepare_triggers();
    assert_eq!(g.state.stack[0].object, old);
    pass_pair(&mut g);
    assert_eq!(g.state.players[0].library.len(), 16);
    g.move_object(crow, Zone::Exile);
    g.move_object(crow, Zone::Graveyard);
    assert!(g.state.pending_triggers.is_empty());
}

#[test]
fn entry_death_surveil_and_tokens_have_complete_shared_condition_semantics() {
    for name in [
        "Lys Alana Informant",
        "Thawbringer",
        "Wary Thespian",
        "Wary Watchdog",
    ] {
        let mut g = scenario();
        let source = enter(&mut g, name);
        let card = g.observe(Player::P0).library_choice.unwrap().remaining[0].id;
        replay(
            &mut g,
            Action::ArrangeLibrary {
                card,
                destination: LibraryDestination::Top,
            },
        );
        g.move_object(source, Zone::Graveyard);
        g.prepare_triggers();
        pass_pair(&mut g);
        assert!(g.observe(Player::P0).library_choice.unwrap().surveil);
    }
    let mut g = scenario();
    let source = enter(&mut g, "Vinereap Mentor");
    g.move_object(source, Zone::Graveyard);
    g.prepare_triggers();
    pass_pair(&mut g);
    assert_eq!(
        g.observe(Player::P0)
            .objects
            .iter()
            .filter(|o| o.zone == Zone::Battlefield
                && o.card.token_info().is_some_and(|t| t.subtypes == "Food"))
            .count(),
        2
    );
}

#[test]
fn attack_mill_only_on_declared_attack_and_targeted_entry_continues_life_gain() {
    let mut g = scenario();
    let source = named(&mut g, Player::P0, "Mysterio's Phantasm", Zone::Battlefield);
    g.state.objects[source as usize].summoning_sick = false;
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    act(&mut g, Action::Attack { card: source });
    replay(&mut g, Action::FinishAttackers);
    g.prepare_triggers();
    pass_pair(&mut g);
    assert_eq!(g.state.players[0].library.len(), 19);
    let watcher = named(&mut g, Player::P0, "Watcher of the Wayside", Zone::Hand);
    g.move_object(watcher, Zone::Battlefield);
    g.prepare_triggers();
    assert_eq!(g.observe(Player::P0).decision, "targets");
    replay(
        &mut g,
        Action::ChooseTarget {
            target: Target::Player(Player::P1),
        },
    );
    pass_pair(&mut g);
    assert_eq!(g.state.players[1].library.len(), 18);
    assert_eq!(g.state.players[0].life, 22);
}

#[test]
fn mill_recovery_storage_is_owned_and_accounted() {
    let mut g = scenario();
    play_spell(&mut g, "Midnight Tilling", None);
    let bytes = g.state_bytes();
    let mut fork = g.clone();
    fork.state.resolving.as_mut().unwrap().milled.clear();
    assert_eq!(
        bytes - fork.state_bytes(),
        4 * std::mem::size_of::<ObjectRef>()
    );
    assert_eq!(g.state.resolving.as_ref().unwrap().milled.len(), 4);
    let snapshot = g.snapshot();
    replay(&mut fork, Action::RecoverMilled { object: None });
    assert_eq!(g.snapshot(), snapshot);
}

#[test]
fn recovery_life_gain_triggers_wait_for_completed_resolution() {
    let mut g = scenario();
    let cat = named(&mut g, Player::P0, "Ajani's Pridemate", Zone::Battlefield);
    let spell = play_spell(&mut g, "Rapid Rescue", None);
    assert!(g.state.pending_triggers.is_empty());
    assert!(g.state.stack.is_empty());
    replay(&mut g, Action::RecoverMilled { object: None });
    assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
    assert!(g.state.resolving.is_none());
    assert_eq!(g.state.stack.len(), 1);
    assert_eq!(g.state.stack[0].card, cat);
    pass_pair(&mut g);
    assert_eq!(g.object_view(cat).counters.plus_one, 1);
}

#[test]
fn countered_mill_spell_never_mills_or_recovers_and_sorcery_timing_is_enforced() {
    let mut g = scenario();
    let source = named(&mut g, Player::P0, "Midnight Tilling", Zone::Hand);
    g.state.players[0].mana = [10; 6];
    g.state.players[1].mana = [10; 6];
    cast(&mut g, source, None);
    act(&mut g, Action::Pass);
    let counter = named(&mut g, Player::P1, "Counterspell", Zone::Hand);
    cast(&mut g, counter, Some(Target::Spell(source)));
    pass_pair(&mut g);
    assert_eq!(g.state.players[0].library.len(), 20);
    assert!(g.state.stack.is_empty() && g.state.resolving.is_none());
    let sorcery = named(&mut g, Player::P0, "Space-Time Anomaly", Zone::Hand);
    g.state.step = Step::BeginCombat;
    assert!(!g.legal_actions().iter().any(|a| matches!(a,
        Action::AnnounceCast {card} | Action::BeginCast {card,..} | Action::Cast {card,..} if *card == sorcery)));
}
