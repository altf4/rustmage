use super::tests::{act, pass_pair, scenario, spawn};
use super::*;
fn named(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    spawn(g, p, Card::from_name(name).unwrap(), zone)
}
fn permanent(g: &mut Game, name: &str) -> CardId {
    named(g, Player::P0, name, Zone::Battlefield)
}
fn enter(g: &mut Game, step: Step) {
    g.begin_step(step);
    g.give_priority();
    g.state_based_actions();
    g.prepare_triggers();
}
fn replay(g: &mut Game, action: Action) {
    let before = g.snapshot();
    let mut fork = g.clone();
    act(&mut fork, action.clone());
    act(g, action.clone());
    assert_eq!(g.snapshot(), fork.snapshot());
    let after = g.snapshot();
    g.restore(&before).unwrap();
    act(g, action);
    assert_eq!(g.snapshot(), after);
}
fn rejected(g: &mut Game, action: Action) {
    let before = g.snapshot();
    let revision = g.revision;
    assert_eq!(
        g.apply(g.actor().unwrap(), action),
        Err(Error::IllegalAction)
    );
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision, revision);
}
fn choose(g: &mut Game, id: CardId) {
    replay(
        g,
        Action::ChooseTarget {
            target: Target::Card(id),
        },
    );
}

#[test]
fn upkeep_occurs_after_untap_before_draw_and_only_on_the_controllers_turn() {
    let mut g = scenario();
    let arena = permanent(&mut g, "Phyrexian Arena");
    let library = g.state.players[0].library.len();
    g.begin_turn();
    g.prepare_triggers();
    assert_eq!(g.state.step, Step::Upkeep);
    assert_eq!(g.state.stack.len(), 1);
    assert_eq!(g.state.stack[0].card, arena);
    assert_eq!(g.state.players[0].library.len(), library);
    replay(&mut g, Action::Pass);
    replay(&mut g, Action::Pass);
    assert_eq!(g.state.players[0].life, 19);
    assert_eq!(g.state.players[0].library.len(), library - 1);
    assert!(g.state.stack.is_empty());
    pass_pair(&mut g);
    assert_eq!(g.state.step, Step::Draw);
    assert_eq!(g.state.players[0].library.len(), library - 2);
    assert!(g.state.stack.is_empty());
    g.state.active = Player::P1;
    g.begin_turn();
    g.prepare_triggers();
    assert!(g.state.stack.is_empty());
}

#[test]
fn beginning_combat_target_and_optional_effect_suspend_before_attackers_and_replay() {
    let mut g = scenario();
    permanent(&mut g, "Battle-Rattle Shaman");
    let bear = permanent(&mut g, "Grizzly Bears");
    pass_pair(&mut g);
    assert_eq!(g.state.step, Step::BeginCombat);
    assert_eq!(g.observe(Player::P0).decision, "targets");
    rejected(&mut g, Action::Pass);
    rejected(&mut g, Action::FinishTargets);
    choose(&mut g, bear);
    assert_eq!(g.state.stack.len(), 1);
    pass_pair(&mut g);
    assert_eq!(g.observe(Player::P0).decision, "optional_effect");
    let before = g.snapshot();
    replay(&mut g, Action::ChooseOptional { accept: true });
    assert_eq!(g.stats(bear).power, 4);
    g.restore(&before).unwrap();
    act(&mut g, Action::ChooseOptional { accept: false });
    assert_eq!(g.stats(bear).power, 2);
    pass_pair(&mut g);
    assert_eq!(g.state.pending, Pending::Attackers);
    assert!(g.state.pending_triggers.is_empty());
}

#[test]
fn entering_during_an_existing_step_never_retroactively_triggers() {
    let mut g = scenario();
    enter(&mut g, Step::Upkeep);
    let arena = named(&mut g, Player::P0, "Phyrexian Arena", Zone::Hand);
    g.move_object(arena, Zone::Battlefield);
    g.prepare_triggers();
    assert!(g.state.stack.is_empty());
    pass_pair(&mut g);
    assert_eq!(g.state.step, Step::Draw);
    assert!(g.state.stack.is_empty());
    // A genuinely later upkeep does trigger, and restoring the entry repeats it.
    let before = g.snapshot();
    enter(&mut g, Step::Upkeep);
    let after = g.snapshot();
    assert_eq!(g.state.stack.len(), 1);
    g.restore(&before).unwrap();
    enter(&mut g, Step::Upkeep);
    assert_eq!(g.snapshot(), after);
}

#[test]
fn intervening_if_is_required_at_the_event_and_rechecked_before_resolution() {
    let mut g = scenario();
    permanent(&mut g, "Flight-Deck Coordinator");
    let a = permanent(&mut g, "Grizzly Bears");
    let b = permanent(&mut g, "Grizzly Bears");
    g.state.objects[a as usize].tapped = true;
    enter(&mut g, Step::End);
    assert!(g.state.stack.is_empty());
    g.state.objects[b as usize].tapped = true;
    g.prepare_triggers();
    assert!(g.state.stack.is_empty()); // A later true condition is insufficient.
    enter(&mut g, Step::End);
    assert_eq!(g.state.stack.len(), 1);
    let before = g.snapshot();
    g.state.objects[a as usize].untap();
    pass_pair(&mut g);
    assert_eq!(g.state.players[0].life, 20);
    g.restore(&before).unwrap();
    pass_pair(&mut g);
    assert_eq!(g.state.players[0].life, 22);
}

#[test]
fn simultaneous_end_triggers_can_change_each_others_intervening_condition() {
    let mut g = scenario();
    let coordinator = permanent(&mut g, "Flight-Deck Coordinator");
    let veteran = permanent(&mut g, "Veteran Beastrider");
    let bear = permanent(&mut g, "Grizzly Bears");
    g.state.objects[veteran as usize].tapped = true;
    g.state.objects[bear as usize].tapped = true;
    enter(&mut g, Step::End);
    assert_eq!(g.state.ordering_triggers.len(), 2);
    let before = g.snapshot();
    let bottom = g
        .state
        .ordering_triggers
        .iter()
        .find(|t| t.card == coordinator)
        .unwrap()
        .id;
    replay(&mut g, Action::PutTrigger { id: bottom });
    assert_eq!(g.state.stack.last().unwrap().card, veteran);
    pass_pair(&mut g);
    pass_pair(&mut g);
    assert_eq!(g.state.players[0].life, 20);
    assert!(!g.object_view(veteran).tapped && !g.object_view(bear).tapped);
    g.restore(&before).unwrap();
    let bottom = g
        .state
        .ordering_triggers
        .iter()
        .find(|t| t.card == veteran)
        .unwrap()
        .id;
    act(&mut g, Action::PutTrigger { id: bottom });
    pass_pair(&mut g);
    pass_pair(&mut g);
    assert_eq!(g.state.players[0].life, 22);
}

#[test]
fn formidable_uses_current_derived_total_power_and_cannot_resolve_below_threshold() {
    let mut g = scenario();
    let surrak = permanent(&mut g, "Surrak, the Hunt Caller");
    let bear = permanent(&mut g, "Grizzly Bears");
    enter(&mut g, Step::BeginCombat);
    assert!(g.state.stack.is_empty()); // 5 + 2 = 7.
    g.state.objects[bear as usize].counters.plus_one = 1;
    enter(&mut g, Step::BeginCombat);
    choose(&mut g, bear);
    let before = g.snapshot();
    g.state.objects[bear as usize].counters.plus_one = 0;
    pass_pair(&mut g);
    assert!(!g.object_view(bear).keywords.contains(Keywords::HASTE));
    g.restore(&before).unwrap();
    pass_pair(&mut g);
    assert!(g.object_view(bear).keywords.contains(Keywords::HASTE));
    assert_eq!(g.object_view(surrak).zone, Zone::Battlefield);
}

#[test]
fn upkeep_land_condition_counts_only_own_current_lands_and_creates_complete_token() {
    let mut g = scenario();
    permanent(&mut g, "Dragonmaster Outcast");
    let mut lands = Vec::new();
    for _ in 0..5 {
        lands.push(spawn(&mut g, Player::P0, Card::Forest, Zone::Battlefield));
    }
    for _ in 0..6 {
        spawn(&mut g, Player::P1, Card::Forest, Zone::Battlefield);
    }
    enter(&mut g, Step::Upkeep);
    assert!(g.state.stack.is_empty());
    lands.push(spawn(&mut g, Player::P0, Card::Forest, Zone::Battlefield));
    enter(&mut g, Step::Upkeep);
    assert_eq!(g.state.stack.len(), 1);
    let before = g.snapshot();
    g.move_object(lands[0], Zone::Hand);
    pass_pair(&mut g);
    assert!(!g.state.objects.iter().any(|o| o.card.is_token()));
    g.restore(&before).unwrap();
    pass_pair(&mut g);
    let dragon = g
        .state
        .objects
        .iter()
        .enumerate()
        .find(|(_, o)| o.zone == Zone::Battlefield && o.card.is_token())
        .unwrap()
        .0 as CardId;
    let token = g.object_view(dragon);
    assert_eq!((token.power, token.toughness), (5, 5));
    assert!(token.keywords.contains(Keywords::FLYING));
    assert_eq!(token.owner, Player::P0);
    assert!(token.summoning_sick);
}

#[test]
fn survival_checks_current_tapped_state_then_departure_lki_and_never_a_returned_incarnation() {
    let mut g = scenario();
    let survivor = permanent(&mut g, "Cautious Survivor");
    enter(&mut g, Step::Main2);
    assert!(g.state.stack.is_empty());
    g.state.objects[survivor as usize].tapped = true;
    enter(&mut g, Step::Main2);
    assert_eq!(g.state.stack.len(), 1);
    let before = g.snapshot();
    g.state.objects[survivor as usize].untap();
    pass_pair(&mut g);
    assert_eq!(g.state.players[0].life, 20);
    g.restore(&before).unwrap();
    g.move_object(survivor, Zone::Hand);
    assert!(g.state.stack[0].source_tapped);
    g.move_object(survivor, Zone::Battlefield);
    assert!(!g.object_view(survivor).tapped);
    pass_pair(&mut g);
    assert_eq!(g.state.players[0].life, 22);
    g.restore(&before).unwrap();
    g.state.objects[survivor as usize].untap();
    g.move_object(survivor, Zone::Graveyard);
    assert!(!g.state.stack[0].source_tapped);
    pass_pair(&mut g);
    assert_eq!(g.state.players[0].life, 20);
}

#[test]
fn survival_lki_is_updated_even_during_its_target_announcement() {
    let mut g = scenario();
    let storyteller = permanent(&mut g, "Shrewd Storyteller");
    let bear = permanent(&mut g, "Grizzly Bears");
    g.state.objects[storyteller as usize].tapped = true;
    enter(&mut g, Step::Main2);
    assert_eq!(g.observe(Player::P0).decision, "targets");
    g.move_object(storyteller, Zone::Graveyard);
    assert!(g.state.announcing.as_ref().unwrap().item.source_tapped);
    choose(&mut g, bear);
    pass_pair(&mut g);
    assert_eq!(g.object_view(bear).counters.plus_one, 1);
}

#[test]
fn raid_records_committed_attacks_past_end_of_combat_and_resets_each_turn() {
    let mut g = scenario();
    permanent(&mut g, "Searslicer Goblin");
    let bear = permanent(&mut g, "Grizzly Bears");
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    replay(&mut g, Action::Attack { card: bear });
    assert!(!g.state.players[0].attacked_this_turn);
    replay(&mut g, Action::FinishAttackers);
    assert!(g.observe(Player::P1).players[0].attacked_this_turn);
    g.move_object(bear, Zone::Graveyard);
    g.state.step = Step::EndCombat;
    g.advance_step();
    assert!(!g.state.attack_declared);
    assert!(g.state.players[0].attacked_this_turn);
    pass_pair(&mut g);
    assert_eq!(g.state.step, Step::End);
    assert_eq!(g.state.stack.len(), 1);
    pass_pair(&mut g);
    assert!(
        g.state
            .objects
            .iter()
            .any(|o| o.zone == Zone::Battlefield && o.card.is_token())
    );
    g.state.active = Player::P1;
    g.begin_turn();
    assert!(!g.state.players[0].attacked_this_turn && !g.state.players[1].attacked_this_turn);
}

#[test]
fn empty_attack_declaration_does_not_enable_raid() {
    let mut g = scenario();
    permanent(&mut g, "Searslicer Goblin");
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    act(&mut g, Action::FinishAttackers);
    assert!(!g.state.players[0].attacked_this_turn);
    enter(&mut g, Step::End);
    assert!(g.state.stack.is_empty());
}

#[test]
fn upkeep_life_loss_and_token_creation_finish_before_state_based_loss() {
    let mut g = scenario();
    permanent(&mut g, "Bitterbloom Bearer");
    g.state.players[0].life = 1;
    enter(&mut g, Step::Upkeep);
    let before = g.snapshot();
    pass_pair(&mut g);
    assert_eq!(g.outcome(), Some(Outcome::Winner(Player::P1)));
    let token = g
        .state
        .objects
        .iter()
        .find(|o| o.zone == Zone::Battlefield && o.card.is_token())
        .unwrap();
    assert_eq!(token.card.token_info().unwrap().colors, 6); // blue and black
    assert!(token.keywords().contains(Keywords::FLYING));
    let after = g.snapshot();
    g.restore(&before).unwrap();
    pass_pair(&mut g);
    assert_eq!(g.snapshot(), after);
}

#[test]
fn group_untap_respects_stun_and_simultaneous_condition_rechecks() {
    let mut g = scenario();
    permanent(&mut g, "Veteran Beastrider");
    let a = permanent(&mut g, "Grizzly Bears");
    let b = permanent(&mut g, "Grizzly Bears");
    g.state.objects[a as usize].tapped = true;
    g.state.objects[a as usize].counters.stun = 1;
    g.state.objects[b as usize].tapped = true;
    enter(&mut g, Step::End);
    pass_pair(&mut g);
    assert!(g.object_view(a).tapped);
    assert_eq!(g.object_view(a).counters.stun, 0);
    assert!(!g.object_view(b).tapped);
}

#[test]
fn every_admitted_step_source_is_indexed_removed_and_restored_by_zone_changes() {
    let mut g = scenario();
    for card in Card::ALL.into_iter().filter(|c| !c.is_token()) {
        let id = spawn(&mut g, Player::P0, card, Zone::Hand);
        if matches!(card.entry_tapped(), EntryTapped::PayLife(_)) {
            g.stage_entries(vec![g.entry_move(id, Zone::Battlefield, false)], None);
            g.advance_entries();
            g.choose_entry_life(false);
            g.advance_entries();
        } else {
            g.move_object(id, Zone::Battlefield);
        }
        let expected = card
            .triggers()
            .iter()
            .any(|a| matches!(a.condition.event(), TriggerCondition::AtStep { .. }));
        assert_eq!(
            g.state.step_sources.contains(&id),
            expected,
            "{}",
            card.name()
        );
        let before = g.snapshot();
        g.move_object(id, Zone::Exile);
        assert!(!g.state.step_sources.contains(&id));
        g.restore(&before).unwrap();
        assert_eq!(g.state.step_sources.contains(&id), expected);
        g.move_object(id, Zone::Exile);
    }
    assert!(g.state.step_sources.is_empty());
}

#[test]
fn predicate_queries_distinguish_own_board_hand_derived_stats_and_turn_history() {
    let mut g = scenario();
    let a = permanent(&mut g, "Grizzly Bears");
    let b = permanent(&mut g, "Grizzly Bears");
    g.state.objects[a as usize].growth.power = 3;
    g.state.objects[b as usize].counters.plus_one = 1;
    g.state.objects[a as usize].tapped = true;
    named(&mut g, Player::P1, "Shivan Dragon", Zone::Battlefield);
    named(&mut g, Player::P1, "Shivan Dragon", Zone::Battlefield);
    for _ in 0..3 {
        spawn(&mut g, Player::P0, Card::Forest, Zone::Battlefield);
    }
    for _ in 0..4 {
        spawn(&mut g, Player::P1, Card::Forest, Zone::Battlefield);
    }
    for _ in 0..2 {
        spawn(&mut g, Player::P0, Card::Forest, Zone::Hand);
    }
    g.state.players[0].spells_cast_this_turn = 3;
    let source = g.object_ref(a);
    for (value, expected) in [
        (StepValue::Life, 20),
        (StepValue::Hand, 2),
        (StepValue::Creatures, 2),
        (StepValue::TappedCreatures, 1),
        (StepValue::Lands, 3),
        (StepValue::GreatestCreaturePower, 5),
        (StepValue::TotalCreaturePower, 8),
        (StepValue::TotalCreatureToughness, 5),
        (StepValue::SpellsCast, 3),
    ] {
        assert!(g.step_guard(
            StepGuard::Threshold {
                value,
                minimum: Some(expected),
                maximum: Some(expected)
            },
            Player::P0,
            source,
            false
        ));
        assert!(!g.step_guard(
            StepGuard::Threshold {
                value,
                minimum: Some(expected + 1),
                maximum: None
            },
            Player::P0,
            source,
            false
        ));
        assert!(!g.step_guard(
            StepGuard::Threshold {
                value,
                minimum: None,
                maximum: Some(expected - 1)
            },
            Player::P0,
            source,
            false
        ));
    }
    g.move_object(a, Zone::Graveyard);
    g.move_object(b, Zone::Graveyard);
    assert!(!g.step_guard(
        StepGuard::Threshold {
            value: StepValue::GreatestCreaturePower,
            minimum: Some(0),
            maximum: None
        },
        Player::P0,
        source,
        false
    ));
    assert!(g.step_guard(
        StepGuard::Threshold {
            value: StepValue::TotalCreaturePower,
            minimum: Some(0),
            maximum: Some(0)
        },
        Player::P0,
        source,
        false
    ));
}
