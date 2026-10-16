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
fn resolve(g: &mut Game) {
    place(g);
    while !g.state.stack.is_empty() {
        assert_eq!(g.state.pending, Pending::Priority);
        replay(g, Action::Pass);
        replay(g, Action::Pass);
        place(g);
    }
}
fn events(g: &Game, id: CardId) -> Vec<DrawTriggerView> {
    g.state
        .stack
        .iter()
        .chain(&g.state.pending_triggers)
        .chain(&g.state.ordering_triggers)
        .filter(|s| s.card == id)
        .filter_map(|s| s.draw_event)
        .collect()
}
fn cycle(g: &mut Game, id: CardId) {
    let a=g.legal_actions().into_iter().find(|a|matches!(a,Action::Activate {card,ability,..} if *card==id && g.state.objects[id as usize].card.activations()[*ability as usize].cycling)).unwrap();
    replay(g, a);
}
fn activate(g: &mut Game, id: CardId) {
    let a = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a,Action::Activate {card,..} if *card==id))
        .unwrap();
    replay(g, a);
}

#[test]
fn multi_card_effects_generate_individual_ordinals_and_defer_triggers_until_complete() {
    let mut g = scenario();
    let each = named(&mut g, Player::P0, "Clinquant Skymage", Zone::Battlefield);
    let second = named(&mut g, Player::P0, "Erudite Wizard", Zone::Battlefield);
    let spell = named(&mut g, Player::P0, "Divination", Zone::Hand);
    g.state.players[0].mana = [10; 6];
    cast(&mut g, spell, None);
    pass_pair(&mut g);
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 2);
    assert_eq!(
        events(&g, each)
            .iter()
            .map(|e| e.ordinal)
            .collect::<Vec<_>>(),
        vec![1, 2]
    );
    assert_eq!(
        events(&g, second),
        vec![DrawTriggerView {
            player: Player::P0,
            ordinal: 2
        }]
    );
    assert_eq!(g.object_view(each).counters.plus_one, 0);
    assert!(g.state.resolving.is_none());
    resolve(&mut g);
    assert_eq!(g.object_view(each).counters.plus_one, 2);
    assert_eq!(g.object_view(second).counters.plus_one, 1);
    g.draw(Player::P0);
    resolve(&mut g);
    assert_eq!(g.object_view(each).counters.plus_one, 3);
    assert_eq!(g.object_view(second).counters.plus_one, 1);
}

#[test]
fn late_entering_observers_use_global_history_without_retroactive_triggers() {
    let mut g = scenario();
    g.draw(Player::P0);
    let first = named(&mut g, Player::P0, "Atlantean Cavalry", Zone::Battlefield);
    g.draw(Player::P0);
    place(&mut g);
    assert_eq!(events(&g, first)[0].ordinal, 2);
    let late = named(
        &mut g,
        Player::P0,
        "Lakeshore Apothecary",
        Zone::Battlefield,
    );
    g.draw(Player::P0);
    assert!(events(&g, late).is_empty());
    resolve(&mut g);
    assert_eq!(g.object_view(first).counters.plus_one, 1);
    assert_eq!(g.object_view(late).counters.plus_one, 0);
}

#[test]
fn draw_history_resets_for_both_players_at_every_turn_boundary() {
    let mut g = scenario();
    let a = named(&mut g, Player::P0, "Erudite Wizard", Zone::Battlefield);
    let b = named(&mut g, Player::P1, "Erudite Wizard", Zone::Battlefield);
    g.draw_many(Player::P0, 3);
    g.draw_many(Player::P1, 2);
    resolve(&mut g);
    assert_eq!(g.object_view(a).counters.plus_one, 1);
    assert_eq!(g.object_view(b).counters.plus_one, 1);
    g.state.active = Player::P1;
    g.begin_turn();
    assert!(g.state.players.iter().all(|p| p.cards_drawn_this_turn == 0));
    g.draw_many(Player::P0, 2);
    g.draw_many(Player::P1, 2);
    resolve(&mut g);
    assert_eq!(g.object_view(a).counters.plus_one, 2);
    assert_eq!(g.object_view(b).counters.plus_one, 2);
    let view = g.observe(Player::P0);
    assert_eq!(view.players[0].cards_drawn_this_turn, 2);
    assert_eq!(view.players[1].cards_drawn_this_turn, 2);
}

#[test]
fn source_departure_does_not_reset_ordinals_or_modify_a_later_incarnation() {
    let mut g = scenario();
    let card = named(&mut g, Player::P0, "Ravenhill Flock", Zone::Battlefield);
    g.draw(Player::P0);
    place(&mut g);
    let original = g.state.stack[0].object;
    g.move_object(card, Zone::Graveyard);
    g.move_object(card, Zone::Battlefield);
    assert_ne!(g.object_ref(card), original);
    resolve(&mut g);
    assert_eq!(g.object_view(card).counters.plus_one, 0);
    g.draw(Player::P0);
    resolve(&mut g);
    assert_eq!(g.object_view(card).counters.plus_one, 1);
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 2);
}

#[test]
fn drawing_player_binding_survives_source_departure_and_other_players_draws() {
    let mut g = scenario();
    let crawler = named(&mut g, Player::P0, "Scrawling Crawler", Zone::Battlefield);
    g.draw(Player::P1);
    place(&mut g);
    assert_eq!(
        events(&g, crawler),
        vec![DrawTriggerView {
            player: Player::P1,
            ordinal: 1
        }]
    );
    g.move_object(crawler, Zone::Graveyard);
    g.draw(Player::P0);
    resolve(&mut g);
    assert_eq!(g.state.players[1].life, 19);
    assert_eq!(g.state.players[0].life, 20);
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 1);
}

#[test]
fn upkeep_draws_count_before_the_normal_draw_step() {
    let mut g = scenario();
    named(&mut g, Player::P0, "Scrawling Crawler", Zone::Battlefield);
    let mystic = named(&mut g, Player::P0, "Mischievous Mystic", Zone::Battlefield);
    g.begin_turn();
    resolve(&mut g);
    assert_eq!(g.state.step, Step::Upkeep);
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 1);
    assert!(events(&g, mystic).is_empty());
    pass_pair(&mut g);
    assert_eq!(g.state.step, Step::Draw);
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 2);
    assert_eq!(events(&g, mystic)[0].ordinal, 2);
    resolve(&mut g);
    let tokens: Vec<_> = g
        .state
        .objects
        .iter()
        .filter(|o| o.zone == Zone::Battlefield && o.card.is_token())
        .collect();
    assert_eq!(tokens.len(), 1);
    assert!(tokens[0].card.keywords().contains(Keywords::FLYING));
}

#[test]
fn failed_draws_do_not_increment_counts_or_emit_success_events() {
    let mut g = scenario();
    let card = named(&mut g, Player::P0, "Clinquant Skymage", Zone::Battlefield);
    let second = named(&mut g, Player::P0, "Erudite Wizard", Zone::Battlefield);
    let ids = g.state.players[0].library.clone();
    for &id in &ids[1..] {
        g.move_object(id, Zone::Graveyard);
    }
    g.draw_many(Player::P0, 3);
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 1);
    assert_eq!(events(&g, card).len(), 1);
    assert!(events(&g, second).is_empty());
    assert!(g.state.players[0].attempted_empty_draw);
    g.state_based_actions();
    assert_eq!(g.outcome(), Some(Outcome::Winner(Player::P1)));
}

#[test]
fn library_to_hand_movement_and_typecycling_searches_are_not_draws() {
    let mut g = scenario();
    let observer = named(&mut g, Player::P0, "Clinquant Skymage", Zone::Battlefield);
    let id = g.state.players[0].library[0];
    g.move_object(id, Zone::Hand);
    let source = named(&mut g, Player::P0, "Kree Sentinel", Zone::Hand);
    g.state.players[0].mana = [2, 0, 0, 0, 0, 0];
    cycle(&mut g, source);
    pass_pair(&mut g);
    let a = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::ChooseSearch { .. }))
        .unwrap();
    replay(&mut g, a);
    replay(&mut g, Action::FinishSearch);
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 0);
    assert!(events(&g, observer).is_empty());
    assert_eq!(g.object_view(observer).counters.plus_one, 0);
}

#[test]
fn cycling_draws_and_clue_sacrifice_feed_the_same_successful_draw_hook() {
    let mut g = scenario();
    named(&mut g, Player::P0, "Mischievous Mystic", Zone::Battlefield);
    for _ in 0..2 {
        let card = named(&mut g, Player::P0, "Lightshield Parry", Zone::Hand);
        g.state.players[0].mana = [2, 0, 0, 0, 0, 0];
        cycle(&mut g, card);
        resolve(&mut g);
    }
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 2);
    assert_eq!(
        g.state
            .objects
            .iter()
            .filter(|o| o.zone == Zone::Battlefield && o.card.is_token())
            .count(),
        1
    );
    let seeker = named(&mut g, Player::P0, "Knowledge Seeker", Zone::Battlefield);
    g.move_object(seeker, Zone::Graveyard);
    resolve(&mut g);
    let clue = g
        .state
        .objects
        .iter()
        .enumerate()
        .find(|(_, o)| {
            o.zone == Zone::Battlefield && o.card.token_info().is_some_and(|i| i.subtypes == "Clue")
        })
        .map(|(i, _)| i as CardId)
        .unwrap();
    g.state.players[0].mana = [2, 0, 0, 0, 0, 0];
    activate(&mut g, clue);
    resolve(&mut g);
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 3);
}

#[test]
fn targeted_draw_trigger_grants_lifelink_and_counters_or_is_countered_by_ward() {
    for ward in [false, true] {
        let mut g = scenario();
        let bard = named(&mut g, Player::P0, "Bard the Bowman", Zone::Battlefield);
        let target = named(
            &mut g,
            if ward { Player::P1 } else { Player::P0 },
            if ward {
                "Hulking Raptor"
            } else {
                "Grizzly Bears"
            },
            Zone::Battlefield,
        );
        g.draw_many(Player::P0, 2);
        place(&mut g);
        assert!(matches!(g.state.pending, Pending::Targets { .. }));
        assert_eq!(g.state.announcing.as_ref().unwrap().item.card, bard);
        replay(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        if ward {
            pass_pair(&mut g);
            replay(&mut g, Action::DeclineResolutionMana);
        } else {
            resolve(&mut g);
        }
        assert_eq!(g.object_view(target).counters.plus_one, u64::from(!ward));
        assert_eq!(g.keywords(target).contains(Keywords::LIFELINK), !ward);
        g.state.step = Step::End;
        pass_pair(&mut g);
        assert!(!g.keywords(target).contains(Keywords::LIFELINK));
        assert_eq!(g.object_view(target).counters.plus_one, u64::from(!ward));
    }
}

#[test]
fn two_player_draw_activation_requires_two_distinct_targets_and_uses_captured_opponent() {
    let mut g = scenario();
    let splendor = named(&mut g, Player::P0, "Gleaming Splendor", Zone::Battlefield);
    g.draw(Player::P1);
    g.state.players[0].mana = [3, 0, 0, 0, 0, 0];
    replay(
        &mut g,
        Action::AnnounceActivate {
            card: splendor,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    replay(
        &mut g,
        Action::ChooseTarget {
            target: Target::Player(Player::P0),
        },
    );
    reject(&mut g, Action::FinishTargets);
    reject(
        &mut g,
        Action::ChooseTarget {
            target: Target::Player(Player::P0),
        },
    );
    replay(
        &mut g,
        Action::ChooseTarget {
            target: Target::Player(Player::P1),
        },
    );
    replay(
        &mut g,
        Action::PayCosts {
            payment: [3, 0, 0, 0, 0, 0],
        },
    );
    pass_pair(&mut g);
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 1);
    assert_eq!(g.state.players[1].cards_drawn_this_turn, 2);
    assert_eq!(
        events(&g, splendor),
        vec![DrawTriggerView {
            player: Player::P1,
            ordinal: 2
        }]
    );
    resolve(&mut g);
    assert_eq!(
        g.state
            .objects
            .iter()
            .filter(|o| o.zone == Zone::Battlefield
                && o.card
                    .token_info()
                    .is_some_and(|i| i.subtypes == "Treasure"))
            .count(),
        1
    );
}

#[test]
fn targeted_counter_trigger_fizzles_on_old_object_while_draw_history_remains() {
    let mut g = scenario();
    named(&mut g, Player::P0, "Roxxon Brutes", Zone::Battlefield);
    let target = named(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
    g.draw_many(Player::P0, 2);
    place(&mut g);
    replay(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    g.move_object(target, Zone::Graveyard);
    g.move_object(target, Zone::Battlefield);
    resolve(&mut g);
    assert_eq!(g.object_view(target).counters.plus_one, 0);
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 2);
}

#[test]
fn investigate_and_second_draw_temporary_source_modifiers_remain_separate() {
    let mut g = scenario();
    let elephant = named(&mut g, Player::P0, "Loxodon Eavesdropper", Zone::Hand);
    g.move_object(elephant, Zone::Battlefield);
    resolve(&mut g);
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 0);
    g.draw_many(Player::P0, 2);
    resolve(&mut g);
    assert_eq!(
        g.state.objects[elephant as usize].growth,
        Stats {
            power: 1,
            toughness: 1
        }
    );
    assert!(g.keywords(elephant).contains(Keywords::VIGILANCE));
    g.state.step = Step::End;
    pass_pair(&mut g);
    assert_eq!(g.state.objects[elephant as usize].growth, Stats::default());
    assert!(!g.keywords(elephant).contains(Keywords::VIGILANCE));
}

#[test]
fn source_upkeep_tap_and_second_draw_untap_honor_stun_replacement() {
    for stun in [false, true] {
        let mut g = scenario();
        let seal = named(&mut g, Player::P0, "Tiger-Seal", Zone::Battlefield);
        g.begin_turn();
        resolve(&mut g);
        assert!(g.object_view(seal).tapped);
        if stun {
            g.state.objects[seal as usize].counters.stun = 1;
        }
        g.draw_many(Player::P0, 2);
        resolve(&mut g);
        assert_eq!(g.object_view(seal).tapped, stun);
        assert_eq!(g.object_view(seal).counters.stun, 0);
    }
}

#[test]
fn draw_source_growth_and_unblockability_expire_in_cleanup() {
    let mut g = scenario();
    let otter = named(&mut g, Player::P0, "Otter-Penguin", Zone::Battlefield);
    g.draw_many(Player::P0, 2);
    resolve(&mut g);
    assert_eq!(
        g.state.objects[otter as usize].growth,
        Stats {
            power: 1,
            toughness: 2
        }
    );
    assert!(g.keywords(otter).contains(Keywords::CANT_BE_BLOCKED));
    g.state.step = Step::End;
    pass_pair(&mut g);
    assert_eq!(g.state.objects[otter as usize].growth, Stats::default());
    assert!(!g.keywords(otter).contains(Keywords::CANT_BE_BLOCKED));
}

#[test]
fn whole_targeted_counter_and_keyword_spells_keep_permanent_counters_after_cleanup() {
    for (name, keyword) in [
        ("Lightfoot Technique", Keywords::FLYING),
        ("Maximum Overdrive", Keywords::DEATHTOUCH),
        ("Take Up the Shield", Keywords::LIFELINK),
    ] {
        let mut g = scenario();
        let target = named(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
        let spell = named(&mut g, Player::P0, name, Zone::Hand);
        g.state.players[0].mana = [10; 6];
        cast(&mut g, spell, Some(Target::Card(target)));
        resolve(&mut g);
        assert_eq!(g.object_view(target).counters.plus_one, 1);
        assert!(g.keywords(target).contains(keyword));
        assert!(g.keywords(target).contains(Keywords::INDESTRUCTIBLE));
        g.state.step = Step::End;
        pass_pair(&mut g);
        assert_eq!(g.object_view(target).counters.plus_one, 1);
        assert!(!g.keywords(target).contains(keyword));
        assert!(!g.keywords(target).contains(Keywords::INDESTRUCTIBLE));
    }
}

#[test]
fn shared_draws_queue_opponent_observers_in_apnap_order_with_correct_event_players() {
    let mut g = scenario();
    let a = named(&mut g, Player::P0, "Scrawling Crawler", Zone::Battlefield);
    let b = named(&mut g, Player::P1, "Scrawling Crawler", Zone::Battlefield);
    g.state.active = Player::P1;
    g.begin_turn();
    place(&mut g);
    assert_eq!(g.state.stack[0].card, b);
    pass_pair(&mut g);
    assert_eq!(g.state.stack.len(), 2);
    assert_eq!(g.state.stack[0].controller, Player::P1);
    assert_eq!(g.state.stack[0].draw_event.unwrap().player, Player::P0);
    assert_eq!(g.state.stack[1].controller, Player::P0);
    assert_eq!(g.state.stack[1].draw_event.unwrap().player, Player::P1);
    assert_eq!(events(&g, a).len(), 1);
    resolve(&mut g);
    assert_eq!(g.state.players[0].life, 19);
    assert_eq!(g.state.players[1].life, 19);
}

#[test]
fn public_draw_context_contains_no_drawn_card_identity_and_owned_replay_preserves_counts() {
    let mut g = scenario();
    let card = named(&mut g, Player::P0, "Clinquant Skymage", Zone::Battlefield);
    let top = *g.state.players[0].library.last().unwrap();
    g.draw_many(Player::P0, 2);
    place(&mut g);
    let public = g.observe(Player::P1);
    assert_eq!(public.schema, 51);
    assert_eq!(public.players[0].cards_drawn_this_turn, 2);
    assert!(public.objects.iter().all(|o| o.id != top));
    let event = serde_json::to_value(events(&g, card)[0]).unwrap();
    assert_eq!(event.as_object().unwrap().len(), 2);
    assert!(event.get("player").is_some());
    assert!(event.get("ordinal").is_some());
    let before = g.observe(Player::P0);
    let saved = g.snapshot();
    let a = g.legal_actions()[1].clone();
    replay(&mut g, a);
    g.restore(&saved).unwrap();
    assert_eq!(
        g.apply_index(Player::P0, before.revision, 1),
        Err(Error::StaleRevision)
    );
    resolve(&mut g);
    assert_eq!(g.object_view(card).counters.plus_one, 2);
}

#[test]
fn opening_hands_and_mulligans_do_not_count_toward_the_first_game_turn() {
    let mut g = Game::new(Config {
        decks: [vec![Card::Forest; 60], vec![Card::Forest; 60]],
        seed: 3,
        starting_player: Player::P0,
        deck_rules: DeckRules::Standard,
    })
    .unwrap();
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 7);
    replay(&mut g, Action::Mulligan);
    replay(&mut g, Action::Keep);
    let bottom = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::Bottom { .. }))
        .unwrap();
    replay(&mut g, bottom);
    replay(&mut g, Action::Keep);
    assert_eq!(g.state.step, Step::Upkeep);
    assert!(g.state.players.iter().all(|p| p.cards_drawn_this_turn == 0));
    pass_pair(&mut g);
    assert_eq!(g.state.step, Step::Main1);
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 0);
}

#[test]
fn draw_facts_wait_through_private_rummage_and_coexist_with_discard_contexts() {
    let mut g = scenario();
    let bird = named(&mut g, Player::P0, "Clinquant Skymage", Zone::Battlefield);
    let mako = named(&mut g, Player::P0, "Marauding Mako", Zone::Battlefield);
    let spell = named(&mut g, Player::P0, "Ancestral Reminiscence", Zone::Hand);
    g.state.players[0].mana = [10; 6];
    cast(&mut g, spell, None);
    pass_pair(&mut g);
    assert!(matches!(
        g.state.pending,
        Pending::EffectDiscard { remaining: 1, .. }
    ));
    assert!(g.state.stack.is_empty());
    assert!(g.state.ordering_triggers.is_empty());
    assert_eq!(events(&g, bird).len(), 3);
    assert_eq!(g.object_view(bird).counters.plus_one, 0);
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 3);
    let hidden = g.observe(Player::P1);
    assert!(hidden.actions.is_empty());
    assert!(hidden.objects.iter().all(|o| o.zone != Zone::Hand));
    let a = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::Discard { .. }))
        .unwrap();
    replay(&mut g, a);
    assert!(g.state.resolving.is_none());
    assert!(events(&g, bird).iter().all(|e| e.player == Player::P0));
    let discard = g
        .state
        .ordering_triggers
        .iter()
        .find(|s| s.card == mako)
        .unwrap();
    assert!(discard.draw_event.is_none());
    assert_eq!(discard.discard_event.unwrap().amount, 1);
    resolve(&mut g);
    assert_eq!(g.object_view(bird).counters.plus_one, 3);
    assert_eq!(g.object_view(mako).counters.plus_one, 1);
}
