use super::tests::{act, pass_pair, scenario, spawn};
use super::*;
fn named(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    let card = Card::from_name(name).unwrap();
    assert!(card.is_supported(), "{name}");
    spawn(g, p, card, zone)
}
fn permanent(g: &mut Game, p: Player, name: &str) -> CardId {
    named(g, p, name, Zone::Battlefield)
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
fn hit(g: &Game, source: CardId, player: Player, amount: i128, combat: bool) -> DamageEvent {
    DamageEvent {
        source: g.damage_source(source),
        target: Target::Player(player),
        amount,
        combat,
    }
}
fn count(g: &Game, source: CardId) -> usize {
    g.state
        .pending_triggers
        .iter()
        .filter(|s| s.card == source)
        .count()
}
fn order(g: &mut Game) {
    g.prepare_triggers();
    while let Some(a) = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PutTrigger { .. }))
    {
        replay(g, a);
    }
}
fn resolve_all(g: &mut Game) {
    order(g);
    while !g.state.stack.is_empty() {
        assert_eq!(g.observe(g.actor().unwrap()).decision, "priority");
        replay(g, Action::Pass);
        replay(g, Action::Pass);
        order(g);
    }
}
fn combat(g: &mut Game, attacker: CardId, blockers: &[CardId]) {
    g.state.attackers.push(Attacker {
        defender: Some(CombatDefender::Player(Player::P1)),
        card: attacker,
        blocked: !blockers.is_empty(),
    });
    for &blocker in blockers {
        g.state.blocks.push((blocker, attacker));
        g.state.blocking.push(blocker);
    }
    g.state.step = Step::Blockers;
    g.give_priority();
}
fn attach(g: &mut Game, p: Player, name: &str, target: CardId) -> CardId {
    let id = permanent(g, p, name);
    g.state.objects[id as usize].attached_to = Some(g.object_ref(target));
    id
}

#[test]
fn actual_combat_draw_uses_captured_amount_after_source_power_changes_or_leaves() {
    let mut g = scenario();
    let fear = permanent(&mut g, Player::P0, "Fear of Failed Tests");
    combat(&mut g, fear, &[]);
    pass_pair(&mut g);
    let event = g.observe(Player::P1).stack[0].damage_event.unwrap();
    assert_eq!(event.amount, 2);
    assert!(event.combat);
    assert_eq!(event.player, Player::P1);
    let old = g.object_ref(fear);
    assert_eq!(event.dealer, Some(old));
    g.state.objects[fear as usize].growth = Stats {
        power: 100,
        toughness: 0,
    };
    g.move_object(fear, Zone::Graveyard);
    let before = g.snapshot();
    pass_pair(&mut g);
    assert_eq!(g.state.players[0].library.len(), 18);
    let after = g.snapshot();
    g.restore(&before).unwrap();
    pass_pair(&mut g);
    assert_eq!(after, g.snapshot());
    assert!(g.state.damage_sources.is_empty());
}

#[test]
fn noncombat_opponent_hit_differs_from_combat_player_hit_and_life_loss() {
    let mut g = scenario();
    let otter = permanent(&mut g, Player::P0, "Thieving Otter");
    let fear = permanent(&mut g, Player::P0, "Fear of Failed Tests");
    for (id, p, n, c) in [
        (otter, Player::P0, 2, false),
        (otter, Player::P1, 0, false),
        (fear, Player::P1, 2, false),
        (fear, Player::P1, -1, true),
    ] {
        let event = hit(&g, id, p, n, c);
        g.apply_damage_batch(&[event]);
    }
    assert!(g.state.pending_triggers.is_empty());
    g.state.players[1].life -= 1; // Life loss is not damage.
    assert!(g.state.pending_triggers.is_empty());
    let event = hit(&g, otter, Player::P1, 3, false);
    g.apply_damage_batch(&[event]);
    assert_eq!(count(&g, otter), 1);
    assert_eq!(count(&g, fear), 0);
    assert!(!g.state.pending_triggers[0].damage_event.unwrap().combat);
    resolve_all(&mut g);
    assert_eq!(g.state.players[0].library.len(), 19);
}

#[test]
fn trample_hit_context_excludes_damage_assigned_to_blockers() {
    let mut g = scenario();
    let fear = permanent(&mut g, Player::P0, "Fear of Failed Tests");
    g.state.objects[fear as usize].growth = Stats {
        power: 3,
        toughness: 0,
    };
    g.state.objects[fear as usize].gained_keywords = Keywords::TRAMPLE;
    let bear = permanent(&mut g, Player::P1, "Grizzly Bears");
    combat(&mut g, fear, &[bear]);
    pass_pair(&mut g);
    replay(
        &mut g,
        Action::AssignDamage {
            target: Target::Card(bear),
            amount: 2,
        },
    );
    let event = g.state.stack[0].damage_event.unwrap();
    assert_eq!(event.amount, 3);
    pass_pair(&mut g);
    assert_eq!(g.state.players[0].library.len(), 17);
}

#[test]
fn double_strike_dispatches_and_resolves_between_damage_steps() {
    let mut g = scenario();
    let fear = permanent(&mut g, Player::P0, "Fear of Failed Tests");
    g.state.objects[fear as usize].gained_keywords = Keywords::DOUBLE_STRIKE;
    combat(&mut g, fear, &[]);
    pass_pair(&mut g);
    assert_eq!(g.state.step, Step::FirstStrikeDamage);
    assert_eq!(g.state.stack.len(), 1);
    resolve_all(&mut g);
    assert_eq!(g.state.players[0].library.len(), 18);
    replay(&mut g, Action::Pass);
    replay(&mut g, Action::Pass);
    assert_eq!(g.state.step, Step::CombatDamage);
    assert_eq!(g.state.stack[0].damage_event.unwrap().amount, 2);
    resolve_all(&mut g);
    assert_eq!(g.state.players[0].library.len(), 16);
}

#[test]
fn trample_source_that_dies_still_creates_food_and_damage_to_creatures_does_not() {
    let mut g = scenario();
    let eager = permanent(&mut g, Player::P0, "Eager Trufflesnout");
    let bear = permanent(&mut g, Player::P1, "Grizzly Bears");
    g.state.objects[eager as usize].growth = Stats {
        power: 5,
        toughness: -1,
    };
    combat(&mut g, eager, &[bear]);
    pass_pair(&mut g);
    replay(
        &mut g,
        Action::AssignDamage {
            target: Target::Card(bear),
            amount: 2,
        },
    );
    assert_eq!(g.object_view(eager).zone, Zone::Graveyard);
    assert_eq!(g.state.stack.len(), 1);
    assert_eq!(g.state.stack[0].card, eager);
    resolve_all(&mut g);
    assert_eq!(
        g.observe(Player::P0)
            .objects
            .iter()
            .filter(|o| o.zone == Zone::Battlefield
                && o.card.token_info().is_some_and(|t| t.subtypes == "Food"))
            .count(),
        1
    );
}

#[test]
fn equipment_hit_trigger_uses_equipment_controller_and_survives_detachment() {
    let mut g = scenario();
    let bear = permanent(&mut g, Player::P1, "Grizzly Bears");
    let pick = attach(&mut g, Player::P0, "Goldvein Pick", bear);
    let event = hit(&g, bear, Player::P0, 3, true);
    g.apply_damage_batch(&[event]);
    assert_eq!(count(&g, pick), 1);
    assert_eq!(g.state.pending_triggers[0].controller, Player::P0);
    let captured = g.state.pending_triggers[0].damage_event;
    g.move_object(pick, Zone::Graveyard);
    assert_eq!(g.state.pending_triggers[0].damage_event, captured);
    resolve_all(&mut g);
    let treasures: Vec<_> = g
        .observe(Player::P0)
        .objects
        .into_iter()
        .filter(|o| {
            o.zone == Zone::Battlefield
                && o.card
                    .token_info()
                    .is_some_and(|t| t.subtypes == "Treasure")
        })
        .collect();
    assert_eq!(treasures.len(), 1);
    assert_eq!(treasures[0].owner, Player::P0);
}

#[test]
fn modified_filter_counts_counters_own_auras_and_any_equipment() {
    for (name, owner, counter, wanted) in [
        (None, Player::P0, None, false),
        (None, Player::P0, Some(CounterKind::PlusOne), true),
        (None, Player::P0, Some(CounterKind::MinusOne), true),
        (None, Player::P0, Some(CounterKind::Stun), true),
        (Some("Frantic Strength"), Player::P0, None, true),
        (Some("Frantic Strength"), Player::P1, None, false),
        (Some("Goldvein Pick"), Player::P1, None, true),
    ] {
        let mut g = scenario();
        let spider = permanent(&mut g, Player::P0, "SP//dr, Piloted by Peni");
        let bear = permanent(&mut g, Player::P0, "Grizzly Bears");
        if let Some(counter) = counter {
            g.state.objects[bear as usize].counters.add(counter, 1);
        }
        if let Some(name) = name {
            attach(&mut g, owner, name, bear);
        }
        let event = hit(&g, bear, Player::P1, 1, true);
        g.apply_damage_batch(&[event]);
        assert_eq!(
            count(&g, spider),
            usize::from(wanted),
            "{name:?} {owner:?} {counter:?}"
        );
    }
}

#[test]
fn modified_attachment_filter_requires_current_incarnation_and_matching_owner() {
    let mut g = scenario();
    let spider = permanent(&mut g, Player::P0, "SP//dr, Piloted by Peni");
    let bear = permanent(&mut g, Player::P0, "Grizzly Bears");
    let pick = attach(&mut g, Player::P0, "Goldvein Pick", bear);
    g.move_object(bear, Zone::Exile);
    g.move_object(bear, Zone::Battlefield);
    let event = hit(&g, bear, Player::P1, 2, true);
    g.apply_damage_batch(&[event]);
    assert_eq!(count(&g, spider), 0);
    assert_eq!(count(&g, pick), 0);
    g.state.objects[bear as usize].counters.plus_one = 1;
    let enemy = permanent(&mut g, Player::P1, "Grizzly Bears");
    g.state.objects[enemy as usize].counters.plus_one = 1;
    let event = hit(&g, enemy, Player::P0, 2, true);
    g.apply_damage_batch(&[event]);
    assert_eq!(count(&g, spider), 0);
}

#[test]
fn counter_observers_distinguish_plus_one_and_group_per_player() {
    let mut g = scenario();
    let haliya = permanent(&mut g, Player::P0, "Haliya, Ascendant Cadet");
    let tomb = permanent(&mut g, Player::P0, "Yathan Tombguard");
    let a = permanent(&mut g, Player::P0, "Grizzly Bears");
    let b = permanent(&mut g, Player::P0, "Grizzly Bears");
    g.state.objects[a as usize].counters.plus_one = 1;
    g.state.objects[b as usize].counters.plus_one = 2;
    let events = [
        hit(&g, a, Player::P1, 1, true),
        hit(&g, a, Player::P1, 2, true),
        hit(&g, b, Player::P1, 4, true),
        hit(&g, b, Player::P0, 1, true),
    ];
    g.apply_damage_batch(&events);
    assert_eq!(count(&g, haliya), 2);
    assert_eq!(count(&g, tomb), 3);
    let aggregate = g
        .state
        .pending_triggers
        .iter()
        .find(|t| t.card == haliya && t.damage_event.unwrap().player == Player::P1)
        .unwrap()
        .damage_event
        .unwrap();
    assert_eq!(aggregate.amount, 7);
    assert_eq!(aggregate.dealer, None);
    assert!(
        g.state
            .pending_triggers
            .iter()
            .filter(|t| t.card == tomb)
            .all(|t| t.damage_event.unwrap().dealer.is_some())
    );
    g.state.pending_triggers.clear();
    g.state.objects[a as usize].counters = Counters {
        stun: 1,
        ..Counters::default()
    };
    let event = hit(&g, a, Player::P1, 1, true);
    g.apply_damage_batch(&[event]);
    assert_eq!(count(&g, haliya), 0);
    assert_eq!(count(&g, tomb), 1);
}

#[test]
fn ally_subtype_batch_and_individual_creature_observers_keep_distinct_multiplicity() {
    let mut g = scenario();
    let invasion = permanent(&mut g, Player::P0, "Invasion Tactics");
    let ultimecia = permanent(&mut g, Player::P0, "Ultimecia, Temporal Threat");
    let ally = permanent(&mut g, Player::P0, "Kyoshi Warriors");
    let other = permanent(&mut g, Player::P0, "Water Tribe Captain");
    let bear = permanent(&mut g, Player::P0, "Grizzly Bears");
    let events = [
        hit(&g, ally, Player::P1, 1, true),
        hit(&g, other, Player::P1, 1, true),
        hit(&g, bear, Player::P1, 1, true),
    ];
    g.apply_damage_batch(&events);
    assert_eq!(count(&g, invasion), 1);
    assert_eq!(count(&g, ultimecia), 3);
    resolve_all(&mut g);
    assert_eq!(g.state.players[0].library.len(), 16);
}

#[test]
fn hero_observer_sees_noncombat_batches_and_does_not_grow_returned_incarnation() {
    let mut g = scenario();
    let thing = permanent(&mut g, Player::P0, "The Thing, Ben Grimm");
    let hero = permanent(&mut g, Player::P0, "SP//dr, Piloted by Peni");
    let other = permanent(&mut g, Player::P0, "Grizzly Bears");
    let events = [
        hit(&g, thing, Player::P1, 2, false),
        hit(&g, hero, Player::P1, 1, false),
        hit(&g, other, Player::P1, 1, false),
    ];
    g.apply_damage_batch(&events);
    assert_eq!(count(&g, thing), 1);
    g.move_object(thing, Zone::Exile);
    g.move_object(thing, Zone::Battlefield);
    resolve_all(&mut g);
    assert_eq!(g.object_view(thing).counters.plus_one, 0);
}

#[test]
fn dragon_mage_discard_choices_are_private_and_all_precede_draws() {
    let mut g = scenario();
    g.state.active = Player::P1;
    let mage = permanent(&mut g, Player::P0, "Dragon Mage");
    let p0 = [
        named(&mut g, Player::P0, "Shock", Zone::Hand),
        named(&mut g, Player::P0, "Forest", Zone::Hand),
    ];
    let p1 = [
        named(&mut g, Player::P1, "Shock", Zone::Hand),
        named(&mut g, Player::P1, "Island", Zone::Hand),
    ];
    let event = hit(&g, mage, Player::P1, 5, true);
    g.apply_damage_batch(&[event]);
    order(&mut g);
    pass_pair(&mut g);
    assert_eq!(g.actor(), Some(Player::P1));
    replay(&mut g, Action::Discard { card: p1[0] });
    assert!(g.observe(Player::P0).effect_discards.is_empty());
    assert_eq!(g.object_view(p1[0]).zone, Zone::Hand);
    replay(&mut g, Action::Discard { card: p1[1] });
    assert_eq!(g.actor(), Some(Player::P0));
    assert_eq!(g.state.players[0].library.len(), 20);
    assert_eq!(g.state.players[1].library.len(), 20);
    assert!(g.observe(Player::P0).effect_discards.is_empty());
    replay(&mut g, Action::Discard { card: p0[0] });
    let before = g.snapshot();
    assert!(
        g.apply(Player::P1, Action::Discard { card: p0[1] })
            .is_err()
    );
    assert_eq!(before, g.snapshot());
    replay(&mut g, Action::Discard { card: p0[1] });
    for id in p0.into_iter().chain(p1) {
        assert_eq!(g.object_view(id).zone, Zone::Graveyard);
    }
    for p in [Player::P0, Player::P1] {
        assert_eq!(g.hand_count(p), 7);
        assert_eq!(g.state.players[p.index()].library.len(), 13);
    }
    assert!(g.state.resolving.is_none());
}

#[test]
fn dragon_mage_empty_hands_and_short_libraries_finish_both_draw_sequences() {
    let mut g = scenario();
    let mage = permanent(&mut g, Player::P0, "Dragon Mage");
    for p in [Player::P0, Player::P1] {
        let ids = g.state.players[p.index()].library.clone();
        for &id in &ids[3..] {
            g.move_object(id, Zone::Exile);
        }
    }
    let event = hit(&g, mage, Player::P1, 1, true);
    g.apply_damage_batch(&[event]);
    order(&mut g);
    pass_pair(&mut g);
    for p in [Player::P0, Player::P1] {
        assert_eq!(g.hand_count(p), 3);
        assert!(g.state.players[p.index()].attempted_empty_draw);
    }
    assert!(g.state.resolving.is_none());
    assert!(g.outcome().is_some());
}

#[test]
fn huge_damage_draw_only_attempts_existing_cards_and_one_empty_draw() {
    let mut g = scenario();
    let fear = permanent(&mut g, Player::P0, "Fear of Failed Tests");
    // A synthetic damage batch checks bounded work for a huge captured amount.
    let event = hit(&g, fear, Player::P1, i128::MAX, true);
    g.apply_damage_batch(&[event]);
    g.state.players[1].life = i128::MAX;
    order(&mut g);
    pass_pair(&mut g);
    assert_eq!(g.hand_count(Player::P0), 20);
    assert!(g.state.players[0].attempted_empty_draw);
}

#[test]
fn ultimecia_entry_taps_opponent_creatures_without_targeting() {
    let mut g = scenario();
    let enemy = permanent(&mut g, Player::P1, "Grizzly Bears");
    g.state.objects[enemy as usize].gained_keywords = Keywords::HEXPROOF;
    let own = permanent(&mut g, Player::P0, "Grizzly Bears");
    let land = permanent(&mut g, Player::P1, "Forest");
    let source = named(&mut g, Player::P0, "Ultimecia, Temporal Threat", Zone::Hand);
    g.move_object(source, Zone::Battlefield);
    order(&mut g);
    pass_pair(&mut g);
    assert!(g.object_view(enemy).tapped);
    assert!(!g.object_view(own).tapped);
    assert!(!g.object_view(land).tapped);
}

#[test]
fn end_step_sacrifice_uses_source_incarnation_and_does_not_sacrifice_a_returned_card() {
    for returned in [false, true] {
        let mut g = scenario();
        let ball = permanent(&mut g, Player::P0, "Ball Lightning");
        g.begin_step(Step::End);
        order(&mut g);
        if returned {
            g.move_object(ball, Zone::Exile);
            g.move_object(ball, Zone::Battlefield);
        }
        pass_pair(&mut g);
        if returned {
            assert_eq!(g.object_view(ball).zone, Zone::Battlefield);
        } else {
            let object = g.object_ref(ball);
            replay(&mut g, Action::Sacrifice { object });
            assert_eq!(g.object_view(ball).zone, Zone::Graveyard);
        }
    }
}

#[test]
fn april_discard_continuation_captures_hit_and_keeps_hands_private() {
    let mut g = scenario();
    let april = permanent(&mut g, Player::P0, "April, Reporter of the Weird");
    let event = hit(&g, april, Player::P1, 4, true);
    g.apply_damage_batch(&[event]);
    order(&mut g);
    pass_pair(&mut g);
    assert_eq!(g.hand_count(Player::P0), 4);
    assert_eq!(g.observe(Player::P0).decision, "effect_discard");
    assert!(
        g.observe(Player::P1)
            .objects
            .iter()
            .all(|o| o.zone != Zone::Hand)
    );
    assert_eq!(
        g.observe(Player::P1)
            .resolving
            .unwrap()
            .damage_event
            .unwrap()
            .amount,
        4
    );
    let card = g
        .observe(Player::P0)
        .objects
        .iter()
        .find(|o| o.zone == Zone::Hand)
        .unwrap()
        .id;
    replay(&mut g, Action::Discard { card });
    assert_eq!(g.hand_count(Player::P0), 3);
}

#[test]
fn damage_observer_index_matches_all_admissions_and_snapshot_restoration() {
    let mut g = scenario();
    for &card in &Card::ALL {
        let id = spawn(&mut g, Player::P0, card, Zone::Battlefield);
        let expected = card
            .triggers()
            .iter()
            .any(|t| matches!(t.condition.event(), TriggerCondition::PlayerDamaged { .. }));
        assert_eq!(
            g.state.damage_sources.contains(&id),
            expected,
            "{}",
            card.name()
        );
        let saved = g.snapshot();
        g.move_object(id, Zone::Exile);
        assert!(!g.state.damage_sources.contains(&id));
        g.restore(&saved).unwrap();
        assert_eq!(g.state.damage_sources.contains(&id), expected);
        g.move_object(id, Zone::Exile);
    }
}

#[test]
fn departed_hero_damage_uses_frozen_battlefield_facts_for_observers() {
    static CODE: &[Instruction] = &[Instruction::DamagePlayers(PlayerSet::Opponent, 3)];
    let mut g = scenario();
    let thing = permanent(&mut g, Player::P0, "The Thing, Ben Grimm");
    let hero = permanent(&mut g, Player::P0, "SP//dr, Piloted by Peni");
    // Synthetic immutable IR: no complete admitted SP//dr ability deals damage.
    // Exercise the engine path for a source removed before its ability resolves.
    let item = g.activation_item(Player::P0, hero, 0);
    g.state.resolving = Some(Resolution {
        connive: None,
        item,
        program: CODE,
        pc: 0,
        target_group: 0,
        moved_count: None,
        discards: [0; 2],
        mana_payment: None,
        selected_discards: [Vec::new(), Vec::new()],
        milled: Vec::new(),
        library: None,
        search: None,
        choice: None,
    });
    let aura = attach(&mut g, Player::P0, "Frantic Strength", hero);
    g.state.objects[hero as usize].counters.stun = 1;
    let old = g.object_ref(hero);
    g.move_object(hero, Zone::Graveyard);
    g.move_object(aura, Zone::Graveyard);
    let facts = g.state.resolving.as_ref().unwrap().item.source_facts;
    assert!(facts.battlefield && facts.counter && facts.modified);
    assert_eq!(facts.card.name(), "SP//dr, Piloted by Peni");
    let saved = g.snapshot();
    g.run_program();
    let after = g.snapshot();
    let mut fork = g.clone();
    fork.restore(&saved).unwrap();
    fork.run_program();
    assert_eq!(fork.snapshot(), after);
    assert_eq!(count(&g, thing), 1);
    assert_eq!(g.state.pending_triggers[0].damage_event.unwrap().amount, 3);
    // Aggregate event, so no single dealer is claimed. The old source did act;
    // the new graveyard object cannot gain the departed source's own triggers.
    assert_ne!(g.object_ref(hero), old);
    assert_eq!(count(&g, hero), 0);
    resolve_all(&mut g);
    assert_eq!(g.object_view(thing).counters.plus_one, 2);
}

#[test]
fn nonbattlefield_hero_card_is_not_a_creature_you_control() {
    let mut g = scenario();
    let thing = permanent(&mut g, Player::P0, "The Thing, Ben Grimm");
    let card = named(&mut g, Player::P0, "SP//dr, Piloted by Peni", Zone::Hand);
    let event = hit(&g, card, Player::P1, 2, false);
    g.apply_damage_batch(&[event]);
    assert_eq!(count(&g, thing), 0);
}

#[test]
fn enchantment_creatures_keep_both_types_and_complete_attack_or_death_triggers() {
    let mut g = scenario();
    let longneck = permanent(&mut g, Player::P0, "Grasping Longneck");
    let card = g.object_view(longneck).card;
    assert!(card.is_creature());
    assert_ne!(card.kind_mask() & 32, 0);
    g.move_object(longneck, Zone::Graveyard);
    resolve_all(&mut g);
    assert_eq!(g.state.players[0].life, 22);
    for name in ["Appendage Amalgam", "Fear of Surveillance"] {
        let id = permanent(&mut g, Player::P0, name);
        g.attack_triggers(&g.object_view(id));
        order(&mut g);
        pass_pair(&mut g);
        let card = g.observe(Player::P0).library_choice.unwrap().remaining[0].id;
        replay(
            &mut g,
            Action::ArrangeLibrary {
                card,
                destination: LibraryDestination::Top,
            },
        );
    }
}
