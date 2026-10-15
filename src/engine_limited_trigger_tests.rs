use super::tests::{act, cast, pass_pair, scenario, spawn};
use super::*;

fn named(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    let c = Card::from_name(name).unwrap();
    assert!(c.is_supported(), "{name}");
    spawn(g, p, c, zone)
}
fn replay(g: &mut Game, a: Action) {
    let saved = g.snapshot();
    let mut fork = g.clone();
    act(&mut fork, a.clone());
    act(g, a.clone());
    let expected = g.snapshot();
    assert_eq!(fork.snapshot(), expected);
    g.restore(&saved);
    act(g, a);
    assert_eq!(g.snapshot(), expected);
}
fn settle(g: &mut Game) {
    for _ in 0..100 {
        g.prepare_triggers();
        if g.state.pending == Pending::Priority && g.state.stack.is_empty() {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::PutTrigger { .. } | Action::Pass | Action::Discard { .. }
                )
            })
            .expect("fixture needs explicit targets");
        replay(g, a);
    }
    panic!("fixture did not settle");
}
fn count(g: &Game, id: CardId) -> usize {
    g.state
        .pending_triggers
        .iter()
        .chain(&g.state.ordering_triggers)
        .chain(&g.state.stack)
        .filter(|t| t.card == id)
        .count()
}
fn enter(g: &mut Game, p: Player, name: &str) -> CardId {
    let id = named(g, p, name, Zone::Hand);
    g.move_object(id, Zone::Battlefield);
    id
}

#[test]
fn octobots_consume_on_first_event_even_before_resolution_and_reset_each_turn() {
    let mut g = scenario();
    let a = named(&mut g, Player::P0, "Flying Octobot", Zone::Battlefield);
    let b = named(&mut g, Player::P0, "Flying Octobot", Zone::Battlefield);
    enter(&mut g, Player::P1, "Common Crook");
    enter(&mut g, Player::P0, "Youthful Valkyrie");
    assert!(g.state.triggered_this_turn.is_empty());
    enter(&mut g, Player::P0, "Common Crook");
    enter(&mut g, Player::P0, "Common Crook");
    assert_eq!(count(&g, a), 1);
    assert_eq!(count(&g, b), 1);
    assert_eq!(g.state.triggered_this_turn.len(), 2);
    settle(&mut g);
    assert_eq!(g.object_view(a).counters.plus_one, 1);
    assert_eq!(g.object_view(b).counters.plus_one, 1);
    let before = g.snapshot();
    g.state.active = Player::P1;
    g.begin_turn();
    assert!(g.state.triggered_this_turn.is_empty());
    enter(&mut g, Player::P0, "Common Crook");
    settle(&mut g);
    assert_eq!(g.object_view(a).counters.plus_one, 2);
    g.restore(&before);
    enter(&mut g, Player::P0, "Common Crook");
    assert_eq!(count(&g, a), 0);
}

#[test]
fn source_reentry_and_pending_old_trigger_keep_independent_incarnations() {
    let mut g = scenario();
    let a = named(&mut g, Player::P0, "Flying Octobot", Zone::Battlefield);
    let old = g.object_ref(a);
    enter(&mut g, Player::P0, "Common Crook");
    g.move_object(a, Zone::Hand);
    g.move_object(a, Zone::Battlefield);
    let new = g.object_ref(a);
    assert_ne!(old, new);
    // Its own entry does not satisfy 'another'.
    assert_eq!(g.state.triggered_this_turn.len(), 1);
    enter(&mut g, Player::P0, "Common Crook");
    assert_eq!(g.state.triggered_this_turn.len(), 2);
    assert_eq!(count(&g, a), 2);
    assert_eq!(
        g.state
            .triggered_this_turn
            .iter()
            .map(|u| u.source)
            .collect::<Vec<_>>(),
        vec![old, new]
    );
    settle(&mut g);
    assert_eq!(g.object_view(a).counters.plus_one, 1);
}

#[test]
fn elvish_archivist_limits_its_two_abilities_independently_across_batches() {
    let mut g = scenario();
    let a = named(&mut g, Player::P0, "Elvish Archivist", Zone::Battlefield);
    let token = Card::from_name("Mechan Assembler").unwrap().triggers()[0]
        .program
        .iter()
        .find_map(|i| {
            if let Instruction::CreateTokens { token, .. } = i {
                Some(*token)
            } else {
                None
            }
        })
        .unwrap();
    g.create_tokens(Player::P1, token, 3, false);
    assert!(g.state.triggered_this_turn.is_empty());
    g.create_tokens(Player::P0, token, 3, false);
    assert_eq!(count(&g, a), 1);
    enter(&mut g, Player::P0, "Fear of Surveillance");
    enter(&mut g, Player::P0, "Fear of Surveillance");
    assert_eq!(count(&g, a), 2);
    assert_eq!(
        g.state
            .triggered_this_turn
            .iter()
            .map(|u| u.ability)
            .collect::<Vec<_>>(),
        vec![0, 1]
    );
    settle(&mut g);
    assert_eq!(g.object_view(a).counters.plus_one, 2);
    assert_eq!(g.state.players[0].library.len(), 19);
}

#[test]
fn mechan_assembler_generated_artifact_cannot_retrigger_same_ability() {
    let mut g = scenario();
    let a = named(&mut g, Player::P0, "Mechan Assembler", Zone::Battlefield);
    let b = named(&mut g, Player::P0, "Mechan Assembler", Zone::Battlefield);
    enter(&mut g, Player::P0, "Flying Octobot");
    assert_eq!(count(&g, a), 1);
    assert_eq!(count(&g, b), 1);
    settle(&mut g);
    assert_eq!(
        g.state
            .objects
            .iter()
            .filter(|o| o.zone == Zone::Battlefield && o.card.is_token())
            .count(),
        2
    );
    assert_eq!(g.state.triggered_this_turn.len(), 2);
}

#[test]
fn homicide_observers_look_back_across_simultaneous_deaths_and_keep_individual_limits() {
    let mut g = scenario();
    let a = named(
        &mut g,
        Player::P0,
        "Homicide Investigator",
        Zone::Battlefield,
    );
    let b = named(
        &mut g,
        Player::P0,
        "Homicide Investigator",
        Zone::Battlefield,
    );
    let bear = spawn(&mut g, Player::P0, Card::GrizzlyBears, Zone::Battlefield);
    let enemy = spawn(&mut g, Player::P1, Card::GrizzlyBears, Zone::Battlefield);
    g.move_object(enemy, Zone::Graveyard);
    assert!(g.state.triggered_this_turn.is_empty());
    let sources = g.death_trigger_sources();
    let events = g.move_objects_raw(&[a, b, bear], Zone::Graveyard);
    g.finish_zone_events(&events, &sources);
    assert_eq!(count(&g, a), 1);
    assert_eq!(count(&g, b), 1);
    assert_eq!(g.state.triggered_this_turn.len(), 2);
    settle(&mut g);
    assert_eq!(
        g.state
            .objects
            .iter()
            .filter(|o| o.zone == Zone::Battlefield && o.card.is_token())
            .count(),
        2
    );
    // New incarnation gets a fresh limit even in the same turn.
    g.move_object(a, Zone::Battlefield);
    g.move_object(a, Zone::Graveyard);
    assert_eq!(count(&g, a), 1);
    settle(&mut g);
}

#[test]
fn token_deaths_do_not_consume_homicide_limit() {
    let mut g = scenario();
    let a = named(
        &mut g,
        Player::P0,
        "Homicide Investigator",
        Zone::Battlefield,
    );
    let maker = Card::from_name("Mechan Assembler").unwrap();
    let token = maker.triggers()[0]
        .program
        .iter()
        .find_map(|i| {
            if let Instruction::CreateTokens { token, .. } = i {
                Some(*token)
            } else {
                None
            }
        })
        .unwrap();
    g.create_tokens(Player::P0, token, 1, false);
    let id = g
        .state
        .objects
        .iter()
        .position(|o| o.card == token)
        .unwrap() as CardId;
    g.move_object(id, Zone::Graveyard);
    assert_eq!(count(&g, a), 0);
    let bear = spawn(&mut g, Player::P0, Card::GrizzlyBears, Zone::Battlefield);
    g.move_object(bear, Zone::Graveyard);
    assert_eq!(count(&g, a), 1);
}

#[test]
fn hostile_investigator_own_entry_discard_fires_once_and_waits_for_resolution() {
    let mut g = scenario();
    let source = named(&mut g, Player::P0, "Hostile Investigator", Zone::Hand);
    let hand = spawn(&mut g, Player::P1, Card::Forest, Zone::Hand);
    g.state.players[0].mana = [10; 6];
    cast(&mut g, source, None);
    pass_pair(&mut g);
    assert!(matches!(g.state.pending, Pending::Targets { .. }));
    replay(
        &mut g,
        Action::ChooseTarget {
            target: Target::Player(Player::P1),
        },
    );
    pass_pair(&mut g);
    assert_eq!(
        g.state.pending,
        Pending::EffectDiscard {
            player: Player::P1,
            remaining: 1
        }
    );
    replay(&mut g, Action::Discard { card: hand });
    assert_eq!(count(&g, source), 1);
    settle(&mut g);
    let own = spawn(&mut g, Player::P0, Card::Forest, Zone::Hand);
    let other = spawn(&mut g, Player::P1, Card::Forest, Zone::Hand);
    g.discard_batch(&[own, other], None);
    assert_eq!(count(&g, source), 0);
    assert_eq!(g.state.triggered_this_turn.len(), 1);
    assert_eq!(
        g.state
            .objects
            .iter()
            .filter(|o| o.zone == Zone::Battlefield && o.card.is_token())
            .count(),
        1
    );
}

#[test]
fn bloombrute_triggers_on_successful_life_gain_only_and_keeps_its_activation() {
    let mut g = scenario();
    let a = named(&mut g, Player::P0, "Bloombrute", Zone::Battlefield);
    g.gain_life(Player::P0, 0);
    g.gain_life(Player::P1, 2);
    assert!(g.state.triggered_this_turn.is_empty());
    g.gain_life(Player::P0, 2);
    g.gain_life(Player::P0, 2);
    assert_eq!(count(&g, a), 1);
    settle(&mut g);
    assert_eq!(g.state.players[0].library.len(), 19);
    g.state.players[0].mana = [10; 6];
    let action = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a,Action::Activate{card,target:Some(Target::Card(t)),..}if *card==*t))
        .unwrap();
    replay(&mut g, action);
    pass_pair(&mut g);
    assert!(
        g.keywords(a)
            .contains(Keywords::TRAMPLE.union(Keywords::LIFELINK))
    );
}

#[test]
fn subtype_entry_events_use_exact_subtype_owner_and_another() {
    for (observer, entrant, other) in [
        ("Avatar Enthusiasts", "Avatar Enthusiasts", true),
        ("Youthful Valkyrie", "Youthful Valkyrie", true),
        ("Obyra, Dreaming Duelist", "Il Mheg Pixie", false),
    ] {
        let mut g = scenario();
        let a = named(&mut g, Player::P0, observer, Zone::Hand);
        g.move_object(a, Zone::Battlefield);
        assert_eq!(count(&g, a), 0);
        enter(&mut g, Player::P1, entrant);
        assert_eq!(count(&g, a), 0);
        enter(&mut g, Player::P0, "Common Crook");
        assert_eq!(count(&g, a), 0);
        enter(&mut g, Player::P0, entrant);
        assert_eq!(count(&g, a), 1);
        settle(&mut g);
        if other {
            assert_eq!(g.object_view(a).counters.plus_one, 1);
        } else {
            assert_eq!(g.state.players[1].life, 19);
        }
        assert!(g.state.triggered_this_turn.is_empty());
    }
}

#[test]
fn artifact_entry_observers_include_self_and_repeat_without_limits() {
    for name in ["Donatello, Way with Machines", "Tidus, Blitzball Star"] {
        let mut g = scenario();
        let a = named(&mut g, Player::P0, name, Zone::Battlefield);
        enter(&mut g, Player::P1, "Flying Octobot");
        assert_eq!(count(&g, a), 0);
        enter(&mut g, Player::P0, "Flying Octobot");
        enter(&mut g, Player::P0, "Friendly Teddy");
        settle(&mut g);
        assert_eq!(g.object_view(a).counters.plus_one, 2);
        assert!(g.state.triggered_this_turn.is_empty());
    }
    let mut g = scenario();
    let a = named(&mut g, Player::P0, "Fateful Discovery", Zone::Battlefield);
    enter(&mut g, Player::P0, "Flying Octobot");
    enter(&mut g, Player::P0, "Friendly Teddy");
    assert_eq!(count(&g, a), 2);
    settle(&mut g);
    assert_eq!(g.state.players[0].library.len(), 18);
}

#[test]
fn public_limit_ledger_is_owned_accounted_and_restore_invalidates_actions() {
    let mut g = scenario();
    let a = named(&mut g, Player::P0, "Flying Octobot", Zone::Battlefield);
    let hidden = spawn(&mut g, Player::P0, Card::Forest, Zone::Hand);
    let blank = g.snapshot();
    let bytes = g.state_bytes();
    assert!(g.queue_trigger(&g.object_view(a), 0));
    assert!(!g.queue_trigger(&g.object_view(a), 0));
    assert_eq!(
        g.state_bytes() - bytes,
        std::mem::size_of::<TriggerUse>() + std::mem::size_of::<SpellView>()
    );
    for p in [Player::P0, Player::P1] {
        let o = g.observe(p);
        assert_eq!(o.schema, 32);
        assert_eq!(
            o.triggered_this_turn,
            vec![TriggerUse {
                source: g.object_ref(a),
                ability: 0
            }]
        );
        if p == Player::P1 {
            assert!(o.objects.iter().all(|o| o.id != hidden));
        }
    }
    let used = g.snapshot();
    let revision = g.revision;
    g.restore(&blank);
    assert!(g.state.triggered_this_turn.is_empty());
    assert_eq!(
        g.apply_index(Player::P0, revision, 0),
        Err(Error::StaleRevision)
    );
    let mut fork = g.clone();
    assert!(fork.queue_trigger(&fork.object_view(a), 0));
    assert_eq!(fork.snapshot(), used);
}

fn settle_with_target(g: &mut Game, target: Target) {
    for _ in 0..100 {
        g.prepare_triggers();
        if g.state.pending == Pending::Priority && g.state.stack.is_empty() {
            return;
        }
        let actions = g.legal_actions();
        let a = actions
            .iter()
            .find(|a| matches!(a,Action::ChooseTarget{target:t}if *t==target))
            .or_else(|| {
                actions
                    .iter()
                    .find(|a| matches!(a, Action::PutTrigger { .. } | Action::Pass))
            })
            .unwrap()
            .clone();
        replay(g, a);
    }
    panic!("target fixture did not settle");
}

#[test]
fn plains_subtype_and_landfall_are_distinct_triggers_and_targeting_is_enforced() {
    let mut g = scenario();
    let a = named(
        &mut g,
        Player::P0,
        "Koth of the Homestead",
        Zone::Battlefield,
    );
    let b = spawn(&mut g, Player::P0, Card::GrizzlyBears, Zone::Battlefield);
    enter(&mut g, Player::P1, "Plains");
    assert_eq!(count(&g, a), 0);
    enter(&mut g, Player::P0, "Forest");
    assert_eq!(count(&g, a), 1);
    settle(&mut g);
    assert_eq!(g.state.players[0].life, 21);
    enter(&mut g, Player::P0, "Plains");
    assert_eq!(count(&g, a), 2);
    settle_with_target(&mut g, Target::Card(b));
    assert_eq!(g.state.players[0].life, 22);
    assert_eq!(g.object_view(b).counters.plus_one, 1);
}

#[test]
fn enchantment_entry_target_ownership_and_artifact_entry_target_counters() {
    let mut g = scenario();
    let a = named(&mut g, Player::P0, "Rimefur Reindeer", Zone::Battlefield);
    let own = spawn(&mut g, Player::P0, Card::GrizzlyBears, Zone::Battlefield);
    let enemy = spawn(&mut g, Player::P1, Card::GrizzlyBears, Zone::Battlefield);
    enter(&mut g, Player::P1, "Fateful Discovery");
    assert_eq!(count(&g, a), 0);
    enter(&mut g, Player::P0, "Fear of Surveillance");
    g.prepare_triggers();
    let before = g.snapshot();
    let revision = g.revision;
    assert_eq!(
        g.apply(
            Player::P0,
            Action::ChooseTarget {
                target: Target::Card(own)
            }
        ),
        Err(Error::IllegalAction)
    );
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision, revision);
    settle_with_target(&mut g, Target::Card(enemy));
    assert!(g.object_view(enemy).tapped);
    let m = named(
        &mut g,
        Player::P0,
        "Mm'menon, Uthros Exile",
        Zone::Battlefield,
    );
    enter(&mut g, Player::P1, "Friendly Teddy");
    assert_eq!(count(&g, m), 0);
    enter(&mut g, Player::P0, "Friendly Teddy");
    assert_eq!(count(&g, m), 1);
    settle_with_target(&mut g, Target::Card(enemy));
    assert_eq!(g.object_view(enemy).counters.plus_one, 1);
}

#[test]
fn aura_entry_draws_for_lookout_only_when_aura_actually_enters() {
    let mut g = scenario();
    let a = named(&mut g, Player::P0, "Tanglespan Lookout", Zone::Battlefield);
    let bear = spawn(&mut g, Player::P0, Card::GrizzlyBears, Zone::Battlefield);
    let aura = named(&mut g, Player::P0, "Frantic Strength", Zone::Hand);
    g.state.players[0].mana = [10; 6];
    cast(&mut g, aura, Some(Target::Card(bear)));
    assert_eq!(count(&g, a), 0);
    pass_pair(&mut g);
    assert_eq!(count(&g, a), 1);
    settle(&mut g);
    assert_eq!(g.state.players[0].library.len(), 19);
    enter(&mut g, Player::P0, "Fateful Discovery");
    assert_eq!(count(&g, a), 0);
}

#[test]
fn tidus_named_attack_trigger_survives_source_departure_and_taps_only_an_opponent() {
    let mut g = scenario();
    let a = named(
        &mut g,
        Player::P0,
        "Tidus, Blitzball Star",
        Zone::Battlefield,
    );
    let enemy = spawn(&mut g, Player::P1, Card::GrizzlyBears, Zone::Battlefield);
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    g.state.objects[a as usize].summoning_sick = false;
    replay(&mut g, Action::Attack { card: a });
    assert_eq!(count(&g, a), 0);
    replay(&mut g, Action::FinishAttackers);
    assert!(
        g.legal_actions()
            .iter()
            .all(|a| !matches!(a,Action::ChooseTarget{target:Target::Card(t)}if *t!=enemy))
    );
    replay(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(enemy),
        },
    );
    g.move_object(a, Zone::Graveyard);
    settle(&mut g);
    assert!(g.object_view(enemy).tapped);
}

#[test]
fn empty_target_announcement_still_consumes_previously_triggered_limit() {
    let mut g = scenario();
    let a = named(&mut g, Player::P0, "Flying Octobot", Zone::Battlefield);
    let source = g.object_view(a);
    assert!(g.queue_trigger(&source, 0));
    let mut item = g.state.pending_triggers.pop().unwrap();
    // Synthetic target-plan fixture, not another admitted printed definition:
    // claim a limited trigger, then use a required-opponent target plan with no
    // candidates to exercise failed announcement rather than resolving it.
    item.source_card = Card::from_name("Rimefur Reindeer").unwrap();
    assert!(!g.start_announcement(item, true));
    assert!(g.state.stack.is_empty());
    assert!(!g.queue_trigger(&source, 0));
    g.state.active = Player::P1;
    g.begin_turn();
    assert!(g.queue_trigger(&source, 0));
}

#[test]
fn simultaneous_death_limits_keep_apnap_and_opponent_sources_independent() {
    let mut g = scenario();
    let a = named(
        &mut g,
        Player::P0,
        "Homicide Investigator",
        Zone::Battlefield,
    );
    let b = named(
        &mut g,
        Player::P1,
        "Homicide Investigator",
        Zone::Battlefield,
    );
    let sources = g.death_trigger_sources();
    let events = g.move_objects_raw(&[b, a], Zone::Graveyard);
    g.finish_zone_events(&events, &sources);
    g.prepare_triggers();
    assert_eq!(
        g.state
            .stack
            .iter()
            .map(|s| s.controller)
            .collect::<Vec<_>>(),
        vec![Player::P0, Player::P1]
    );
    settle(&mut g);
    assert_eq!(g.state.triggered_this_turn.len(), 2);
    for p in [Player::P0, Player::P1] {
        assert_eq!(
            g.state
                .objects
                .iter()
                .filter(|o| o.zone == Zone::Battlefield && o.owner == p && o.card.is_token())
                .count(),
            1
        );
    }
}

#[test]
fn simultaneous_multiplayer_discard_batch_claims_one_limit_per_investigator() {
    let mut g = scenario();
    let a = named(
        &mut g,
        Player::P0,
        "Hostile Investigator",
        Zone::Battlefield,
    );
    let b = named(
        &mut g,
        Player::P1,
        "Hostile Investigator",
        Zone::Battlefield,
    );
    let ids = [
        spawn(&mut g, Player::P0, Card::Forest, Zone::Hand),
        spawn(&mut g, Player::P0, Card::Forest, Zone::Hand),
        spawn(&mut g, Player::P1, Card::Forest, Zone::Hand),
    ];
    g.discard_batch(&ids, None);
    assert_eq!(count(&g, a), 1);
    assert_eq!(count(&g, b), 1);
    assert_eq!(g.state.triggered_this_turn.len(), 2);
    settle(&mut g);
    assert_eq!(
        g.state
            .objects
            .iter()
            .filter(|o| o.zone == Zone::Battlefield && o.card.is_token())
            .count(),
        2
    );
}
