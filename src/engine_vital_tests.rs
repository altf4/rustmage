use super::tests::{act, cast, pass_pair, scenario, spawn};
use super::*;
fn named(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    let card = Card::from_name(name).unwrap();
    assert!(card.is_supported(), "{name}");
    spawn(g, p, card, zone)
}
fn permanent(g: &mut Game, name: &str) -> CardId {
    named(g, Player::P0, name, Zone::Battlefield)
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
        replay(g, Action::Pass);
        replay(g, Action::Pass);
        order(g);
    }
}
fn count(g: &Game, source: CardId) -> usize {
    g.state
        .pending_triggers
        .iter()
        .filter(|t| t.card == source)
        .count()
}
fn rat_token() -> Card {
    Card::catalogue()
        .find(|c| c.token_info().is_some_and(|t| t.subtypes == "Rat"))
        .unwrap()
}

#[test]
fn explicit_gain_counts_events_not_amount_and_orders_owned_effects() {
    let mut g = scenario();
    let cat = permanent(&mut g, "Ajani's Pridemate");
    let priest = permanent(&mut g, "Marauding Blight-Priest");
    let other = named(&mut g, Player::P1, "Ajani's Pridemate", Zone::Battlefield);
    g.gain_life(Player::P0, 0);
    assert_eq!(g.state.players[0].life_gain_events_this_turn, 0);
    assert!(g.state.pending_triggers.is_empty());
    g.gain_life(Player::P0, 7);
    assert_eq!(count(&g, cat), 1);
    assert_eq!(count(&g, priest), 1);
    assert_eq!(count(&g, other), 0);
    assert_eq!(
        g.observe(Player::P1).players[0].life_gain_events_this_turn,
        1
    );
    resolve_all(&mut g);
    assert_eq!(g.state.players[0].life, 27);
    assert_eq!(g.state.players[1].life, 19);
    assert_eq!(g.object_view(cat).counters.plus_one, 1);
    assert_eq!(g.object_view(other).counters.plus_one, 0);
}

#[test]
fn first_gain_history_precedes_sources_resets_both_players_and_obeys_own_turn() {
    let mut g = scenario();
    g.gain_life(Player::P0, 1); // No observer on the battlefield yet.
    let angel = permanent(&mut g, "Vanguard Seraph");
    let collector = permanent(&mut g, "Cat Collector");
    g.gain_life(Player::P0, 2);
    assert!(g.state.pending_triggers.is_empty());
    g.state.active = Player::P1;
    g.begin_turn();
    assert_eq!(g.state.players[0].life_gain_events_this_turn, 0);
    assert_eq!(g.state.players[1].life_gain_events_this_turn, 0);
    g.gain_life(Player::P0, 3);
    assert_eq!(count(&g, angel), 1);
    assert_eq!(count(&g, collector), 0);
    g.state.pending_triggers.clear();
    g.gain_life(Player::P0, 1);
    assert!(g.state.pending_triggers.is_empty());
    g.state.active = Player::P0;
    g.begin_turn();
    g.gain_life(Player::P0, 4);
    assert_eq!(count(&g, angel), 1);
    assert_eq!(count(&g, collector), 1);
    // Source removal/return cannot reset the player's event history.
    g.move_object(angel, Zone::Exile);
    g.move_object(angel, Zone::Battlefield);
    g.gain_life(Player::P0, 1);
    assert_eq!(count(&g, angel), 1);
}

#[test]
fn simultaneous_lifelink_is_one_gain_per_source_and_observers_see_it_before_sbas() {
    let mut g = scenario();
    let cat = permanent(&mut g, "Ajani's Pridemate");
    let collector = permanent(&mut g, "Cat Collector");
    let a = permanent(&mut g, "Vampire Nighthawk");
    let b = permanent(&mut g, "Brightblade Stoat");
    let source_a = g.damage_source(a);
    let source_b = g.damage_source(b);
    let gains = g.apply_damage_batch(&[
        DamageEvent {
            source: source_a,
            target: Target::Player(Player::P1),
            amount: 2,
            combat: true,
        },
        DamageEvent {
            source: source_a,
            target: Target::Card(cat),
            amount: 1,
            combat: true,
        },
        DamageEvent {
            source: source_b,
            target: Target::Player(Player::P1),
            amount: 3,
            combat: true,
        },
    ]);
    assert_eq!(gains.len(), 2);
    assert_eq!(g.state.players[0].life, 26);
    assert_eq!(g.state.players[0].life_gain_events_this_turn, 2);
    assert_eq!(count(&g, cat), 2);
    assert_eq!(count(&g, collector), 1);
    // Lethal deathtouch damage doesn't remove the observer before gain dispatch.
    g.state_based_actions();
    assert_eq!(g.object_view(cat).zone, Zone::Graveyard);
    resolve_all(&mut g);
    assert_eq!(g.object_view(cat).counters.plus_one, 0);
    assert_eq!(
        g.state
            .objects
            .iter()
            .filter(|o| o.zone == Zone::Battlefield
                && o.card.token_info().is_some_and(|t| t.subtypes == "Cat"))
            .count(),
        1
    );
}

#[test]
fn double_strike_lifelink_dispatches_and_resolves_between_damage_steps() {
    let mut g = scenario();
    let reaver = permanent(&mut g, "Drogskol Reaver");
    let cat = permanent(&mut g, "Ajani's Pridemate");
    g.state.attackers.push(Attacker {
        defender: Some(CombatDefender::Player(Player::P1)),
        card: reaver,
        blocked: false,
    });
    g.state.step = Step::Blockers;
    g.give_priority();
    pass_pair(&mut g);
    assert_eq!(g.state.step, Step::FirstStrikeDamage);
    resolve_all(&mut g);
    assert_eq!(g.state.players[0].life, 23);
    assert_eq!(g.state.players[0].library.len(), 19);
    assert_eq!(g.object_view(cat).counters.plus_one, 1);
    pass_pair(&mut g);
    assert_eq!(g.state.step, Step::CombatDamage);
    resolve_all(&mut g);
    assert_eq!(g.state.players[0].life, 26);
    assert_eq!(g.state.players[1].life, 14);
    assert_eq!(g.state.players[0].library.len(), 18);
    assert_eq!(g.object_view(cat).counters.plus_one, 2);
    assert_eq!(g.state.players[0].life_gain_events_this_turn, 2);
}

#[test]
fn queued_source_effect_cannot_grow_returned_incarnation() {
    let mut g = scenario();
    let cat = permanent(&mut g, "Tigra, Feline Fury");
    g.gain_life(Player::P0, 2);
    let before = g.object_ref(cat);
    g.move_object(cat, Zone::Exile);
    g.move_object(cat, Zone::Battlefield);
    assert_ne!(g.object_ref(cat), before);
    resolve_all(&mut g);
    assert_eq!(g.object_view(cat).counters.plus_one, 0);
}

#[test]
fn simultaneous_deaths_use_pre_batch_sources_including_the_dying_observers() {
    let mut g = scenario();
    let venom = permanent(&mut g, "Agent Venom");
    let reaper = permanent(&mut g, "Midnight Reaper");
    let vermin = permanent(&mut g, "Voracious Vermin");
    let bear = permanent(&mut g, "Grizzly Bears");
    let token = spawn(&mut g, Player::P0, rat_token(), Zone::Battlefield);
    let hunter = named(&mut g, Player::P1, "High-Society Hunter", Zone::Battlefield);
    let sources = g.death_trigger_sources();
    let events = g.move_objects_raw(&[venom, reaper, vermin, bear, token], Zone::Graveyard);
    g.finish_zone_events(&events, &sources);
    assert_eq!(count(&g, venom), 3);
    assert_eq!(count(&g, reaper), 4);
    assert_eq!(count(&g, vermin), 4);
    assert_eq!(count(&g, hunter), 4);
    resolve_all(&mut g);
    assert_eq!(g.state.players[0].life, 13);
    assert_eq!(g.state.players[0].library.len(), 13);
    assert_eq!(g.state.players[1].library.len(), 16);
    assert_eq!(g.object_view(vermin).counters.plus_one, 0);
    assert_eq!(g.object_view(token).zone, Zone::Outside);
}

#[test]
fn death_filters_use_exact_subtype_ownership_other_and_real_creature_deaths() {
    let mut g = scenario();
    let boggart = permanent(&mut g, "Boggart Cursecrafter");
    let goblin = permanent(&mut g, "Raging Redcap");
    let enemy = named(&mut g, Player::P1, "Raging Redcap", Zone::Battlefield);
    let rat = permanent(&mut g, "Voracious Vermin");
    let forest = permanent(&mut g, "Forest");
    g.move_object(enemy, Zone::Graveyard);
    g.move_object(rat, Zone::Graveyard);
    g.move_object(forest, Zone::Graveyard);
    g.move_object(goblin, Zone::Exile);
    assert_eq!(count(&g, boggart), 0);
    g.move_object(goblin, Zone::Battlefield);
    g.move_object(goblin, Zone::Graveyard);
    assert_eq!(count(&g, boggart), 1);
    let token_card = Card::catalogue()
        .find(|c| c.token_info().is_some_and(|t| t.subtypes == "Goblin"))
        .unwrap();
    let token = spawn(&mut g, Player::P0, token_card, Zone::Battlefield);
    g.move_object(token, Zone::Graveyard);
    assert_eq!(count(&g, boggart), 2);
    g.move_object(boggart, Zone::Graveyard);
    assert_eq!(count(&g, boggart), 2); // "another" excludes itself.
    resolve_all(&mut g);
    assert_eq!(g.state.players[1].life, 18);
}

#[test]
fn death_trigger_retains_source_lifelink_and_life_gain_can_trigger_again() {
    let mut g = scenario();
    let reaper = permanent(&mut g, "Midnight Reaper");
    let priest = permanent(&mut g, "Marauding Blight-Priest");
    g.state.objects[reaper as usize].gained_keywords = Keywords::LIFELINK;
    g.move_object(reaper, Zone::Graveyard);
    assert_eq!(count(&g, reaper), 1);
    resolve_all(&mut g);
    assert_eq!(g.state.players[0].life, 20); // Damage and lifelink are both applied.
    assert_eq!(g.state.players[0].life_gain_events_this_turn, 1);
    assert_eq!(g.state.players[0].library.len(), 19);
    assert_eq!(g.state.players[1].life, 19);
    assert_eq!(g.state.stack.len(), 0);
    assert!(g.state.life_sources.contains(&priest));
}

#[test]
fn opposing_deaths_from_entry_debuff_queue_before_priority_with_exact_sources() {
    let mut g = scenario();
    named(&mut g, Player::P1, "Grizzly Bears", Zone::Battlefield);
    named(&mut g, Player::P1, "Raging Redcap", Zone::Battlefield);
    let sephiroth = named(&mut g, Player::P0, "Sephiroth, Planet's Heir", Zone::Hand);
    g.state.players[0].mana = [0, 1, 1, 0, 0, 4];
    cast(&mut g, sephiroth, None);
    resolve_all(&mut g);
    assert_eq!(g.object_view(sephiroth).counters.plus_one, 2);
    assert_eq!(g.stats(sephiroth).power, 6);
    assert_eq!(
        g.state
            .objects
            .iter()
            .filter(|o| o.zone == Zone::Graveyard && o.owner == Player::P1)
            .count(),
        2
    );
}

#[test]
fn sacrifice_during_attack_finishes_ability_before_death_trigger_draws() {
    let mut g = scenario();
    let hunter = permanent(&mut g, "High-Society Hunter");
    let bear = permanent(&mut g, "Grizzly Bears");
    g.attack_triggers(&g.object_view(hunter));
    order(&mut g);
    pass_pair(&mut g);
    assert_eq!(g.observe(Player::P0).decision, "effect_choice");
    let before = g.snapshot();
    let bad = Action::Sacrifice {
        object: g.object_ref(hunter),
    };
    assert_eq!(g.apply(Player::P0, bad), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), before);
    let object = g.object_ref(bear);
    replay(&mut g, Action::Sacrifice { object });
    assert_eq!(g.object_view(hunter).counters.plus_one, 1);
    assert_eq!(g.state.players[0].library.len(), 20);
    resolve_all(&mut g);
    assert_eq!(g.state.players[0].library.len(), 19);
}

#[test]
fn source_indices_follow_every_admitted_definition_and_round_trip_restore() {
    let mut g = scenario();
    for &card in &Card::ALL {
        let id = spawn(&mut g, Player::P0, card, Zone::Battlefield);
        let life = card.triggers().iter().any(|a| {
            matches!(
                a.condition.event(),
                TriggerCondition::LifeGained { .. } | TriggerCondition::LifeLost { .. }
            )
        });
        let death = card.triggers().iter().any(|a| {
            matches!(
                a.condition.event(),
                TriggerCondition::SelfDies
                    | TriggerCondition::SelfEntersOrDies
                    | TriggerCondition::CreatureDies { .. }
                    | TriggerCondition::PermanentToGraveyard { .. }
                    | TriggerCondition::CreatureDiesFiltered { .. }
                    | TriggerCondition::FoodsSacrificed
                    | TriggerCondition::Sacrificed { .. }
            )
        });
        assert_eq!(g.state.life_sources.contains(&id), life, "{}", card.name());
        assert_eq!(
            g.state.death_sources.contains(&id),
            death,
            "{}",
            card.name()
        );
        let before = g.snapshot();
        g.move_object(id, Zone::Exile);
        assert!(!g.state.life_sources.contains(&id));
        assert!(!g.state.death_sources.contains(&id));
        g.restore(&before).unwrap();
        assert_eq!(g.state.life_sources.contains(&id), life);
        assert_eq!(g.state.death_sources.contains(&id), death);
        g.move_object(id, Zone::Exile);
    }
    assert!(g.state.life_sources.is_empty() && g.state.death_sources.is_empty());
}
