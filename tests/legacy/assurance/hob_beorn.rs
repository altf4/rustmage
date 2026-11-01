//! Printed outcomes and shared permanent self-grant/type contracts for both Beorns.
use super::*;
fn finish(g: &mut Game) {
    for _ in 0..64 {
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.resolving.is_none()
            && g.state.pending_triggers.is_empty()
            && g.state.ordering_triggers.is_empty()
        {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("Beorn resolution budget {:?}", g.state.pending);
}
fn until(g: &mut Game, predicate: impl Fn(&Action) -> bool) {
    for _ in 0..32 {
        if g.legal_actions().iter().any(&predicate) {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("Beorn choice budget {:?}", g.state.pending);
}
fn paid(g: &mut Game, p: Player, name: &str, mana: Mana) -> CardId {
    let id = put(g, p, name, Zone::Hand);
    g.state.players[p.index()].mana = mana;
    cast(g, id, None);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    finish(g);
    id
}
fn fierce(g: &mut Game, p: Player) -> CardId {
    paid(g, p, "Beorn the Fierce", [0, 0, 0, 0, 2, 3])
}
fn hospitality(g: &mut Game, p: Player) -> CardId {
    paid(g, p, "Beorn's Hospitality", [0, 0, 0, 0, 1, 1])
}
fn combat(g: &mut Game, p: Player) {
    g.state.active = p;
    g.begin_step(Step::BeginCombat);
    g.give_priority();
    g.prepare_triggers();
}
fn select(g: &mut Game, id: Option<CardId>) {
    until(g, |a| {
        matches!(a, Action::ChooseTarget { .. } | Action::FinishTargets)
    });
    if let Some(id) = id {
        act(
            g,
            Action::ChooseTarget {
                target: Target::Card(id),
            },
        );
    }
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
    if g.legal_actions()
        .iter()
        .any(|a| matches!(a, Action::PayCosts { .. }))
    {
        pay(g);
    }
}
fn start(g: &mut Game, p: Player, id: CardId) {
    g.state.priority = p;
    g.state.players[p.index()].mana = [0, 0, 0, 0, 2, 5];
    act(
        g,
        Action::AnnounceActivate {
            card: id,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    pay(g);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
}
fn lands(g: &mut Game, p: Player, n: usize) -> Vec<CardId> {
    (0..n)
        .map(|_| put(g, p, "Forest", Zone::Battlefield))
        .collect()
}
fn enter(g: &mut Game, p: Player) -> CardId {
    let id = put(g, p, "Forest", Zone::Hand);
    g.move_object(id, Zone::Battlefield);
    g.give_priority();
    g.prepare_triggers();
    id
}
#[test]
fn beorn_fierce_paid_body_other_own_bear_bonus_and_current_controller() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let goblin = put(&mut g, p, "Goblin Piker", Zone::Battlefield);
        let b = fierce(&mut g, p);
        assert_eq!(stats(&g, b), (6, 6));
        assert!(g.keywords(b).contains(Keywords::TRAMPLE));
        assert_eq!(stats(&g, own), (4, 4));
        assert_eq!(stats(&g, enemy), (2, 2));
        assert_eq!(stats(&g, goblin), (2, 1));
        g.state.objects[enemy as usize].controller = p;
        assert_eq!(stats(&g, enemy), (4, 4));
        g.move_object(b, Zone::Hand);
        assert_eq!(stats(&g, own), (2, 2));
        assert_eq!(stats(&g, enemy), (2, 2));
    }
}
#[test]
fn beorn_fierce_combat_counter_adds_bear_before_current_threshold_draw_and_persists() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let b = fierce(&mut g, p);
        put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let goblin = put(&mut g, p, "Goblin Piker", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        combat(&mut g, p);
        until(&mut g, |a| {
            matches!(a, Action::ChooseTarget { .. } | Action::FinishTargets)
        });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        contracts::branch_contract(g.clone());
        select(&mut g, Some(goblin));
        finish(&mut g);
        assert_eq!(g.counter_count(goblin, CounterKind::Trample), 1);
        assert!(g.keywords(goblin).contains(Keywords::TRAMPLE));
        assert!(g.has_subtype(goblin, "Goblin") && g.has_subtype(goblin, "Bear"));
        assert_eq!(stats(&g, goblin), (4, 3));
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 2);
        let hand: Vec<_> = g
            .state
            .objects
            .iter()
            .enumerate()
            .filter(|(_, o)| o.zone == Zone::Hand && o.owner == p)
            .map(|(i, _)| i as CardId)
            .collect();
        for id in hand {
            assert!(g.observe(p).objects.iter().any(|o| o.id == id));
            assert!(g.observe(p.other()).objects.iter().all(|o| o.id != id));
        }
        g.move_object(b, Zone::Graveyard);
        g.finish_cleanup();
        assert!(g.has_subtype(goblin, "Bear") && g.has_subtype(goblin, "Goblin"));
        assert!(g.keywords(goblin).contains(Keywords::TRAMPLE));
        assert_eq!(stats(&g, goblin), (2, 1));
        g.move_object(goblin, Zone::Hand);
        g.move_object(goblin, Zone::Battlefield);
        assert!(!g.has_subtype(goblin, "Bear"));
        assert_eq!(g.counter_count(goblin, CounterKind::Trample), 0);
    }
}
#[test]
fn beorn_fierce_zero_targets_still_draws_with_three_current_bears_only_own_combat() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        fierce(&mut g, p);
        g.create_tokens(p, Card::from_name("@token/hob-bear").unwrap(), 2, false);
        finish(&mut g);
        combat(&mut g, p.other());
        finish(&mut g);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
        combat(&mut g, p);
        select(&mut g, None);
        finish(&mut g);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 2);
        assert!(
            g.state
                .objects
                .iter()
                .enumerate()
                .all(|(id, _)| g.counter_count(id as CardId, CounterKind::Trample) == 0)
        );
    }
}
#[test]
fn beorn_fierce_threshold_is_resolution_only_and_illegal_chosen_target_fizzles_everything() {
    for p in [Player::P0, Player::P1] {
        for stale in [false, true] {
            let mut g = scenario(p);
            fierce(&mut g, p);
            let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let target = put(&mut g, p, "Goblin Piker", Zone::Battlefield);
            combat(&mut g, p);
            select(&mut g, Some(target));
            if stale {
                g.move_object(target, Zone::Hand);
                g.move_object(target, Zone::Battlefield);
                put(&mut g, p, "Bear Cub", Zone::Battlefield);
            } else {
                g.move_object(bear, Zone::Hand);
            }
            finish(&mut g);
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
            assert_eq!(
                g.counter_count(target, CounterKind::Trample),
                u64::from(!stale)
            );
            assert_eq!(g.has_subtype(target, "Bear"), !stale);
        }
    }
}
#[test]
fn beorn_fierce_queued_combat_survives_source_departure_and_uses_remaining_bears() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let b = fierce(&mut g, p);
        for _ in 0..3 {
            put(&mut g, p, "Bear Cub", Zone::Battlefield);
        }
        combat(&mut g, p);
        select(&mut g, None);
        g.move_object(b, Zone::Hand);
        finish(&mut g);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 2);
    }
}
#[test]
fn beorn_hospitality_landfall_is_mandatory_only_controlled_creature_and_exact_target() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let h = hospitality(&mut g, p);
        let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        enter(&mut g, p.other());
        finish(&mut g);
        assert_eq!(g.counter_count(bear, CounterKind::PlusOne), 0);
        enter(&mut g, p);
        until(&mut g, |a| matches!(a, Action::ChooseTarget { .. }));
        reject(&mut g, Action::FinishTargets);
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(h),
            },
        );
        select(&mut g, Some(bear));
        finish(&mut g);
        assert_eq!(g.counter_count(bear, CounterKind::PlusOne), 1);
        enter(&mut g, p);
        select(&mut g, Some(bear));
        g.move_object(bear, Zone::Hand);
        g.move_object(bear, Zone::Battlefield);
        finish(&mut g);
        assert_eq!(g.counter_count(bear, CounterKind::PlusOne), 0);
    }
}
#[test]
fn beorn_hospitality_paid_permanent_bear_and_live_land_stats_keep_other_types_and_counters() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let own = lands(&mut g, p, 3);
        lands(&mut g, p.other(), 2);
        let h = hospitality(&mut g, p);
        for &land in &own {
            g.state.objects[land as usize].tapped = true;
        }
        let action = Action::AnnounceActivate {
            card: h,
            ability: 0,
            counter_payment: Counters::default(),
        };
        g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 6];
        reject(&mut g, action.clone());
        g.change_object_form(
            g.object_ref(h),
            crate::cards::AnimationSpec {
                artifact: true,
                creature_types: crate::cards::AnimationCreatureTypes::Named(&["Elf"]),
                base: Some((2, 2)),
                keywords: Keywords::NONE,
                duration: crate::cards::AnimationDuration::Permanent,
            },
            characteristic_layers::AnimationTypeMode::AddCreature,
        );
        g.state.active = p.other();
        start(&mut g, p, h);
        finish(&mut g);
        assert_eq!(g.current_kinds(h), 49);
        assert!(g.has_subtype(h, "Elf"));
        assert!(g.has_subtype(h, "Bear"));
        assert_eq!(stats(&g, h), (3, 3));
        assert!(!g.object_view(h).tapped);
        assert_eq!(g.object_view(h).colors, 16);
        g.state.active = p;
        enter(&mut g, p);
        select(&mut g, Some(h));
        finish(&mut g);
        assert_eq!(stats(&g, h), (5, 5));
        g.move_object(own[0], Zone::Graveyard);
        assert_eq!(stats(&g, h), (4, 4));
        g.state.objects[h as usize].controller = p.other();
        assert_eq!(stats(&g, h), (3, 3));
        g.state.priority = p.other();
        g.state.active = p.other();
        start(&mut g, p.other(), h);
        finish(&mut g);
        assert_eq!(stats(&g, h), (3, 3));
        for _ in 0..2 {
            g.finish_cleanup();
            assert_eq!(stats(&g, h), (3, 3));
            assert!(g.has_subtype(h, "Bear"));
        }
        assert!(
            g.state
                .self_granted_abilities
                .as_ref()
                .unwrap()
                .iter()
                .all(|row| row.expires_turn.is_none())
        );
        contracts::branch_contract(g.clone());
    }
}
#[test]
fn beorn_hospitality_layer7b_order_is_live_and_separate_from_counter_bonus() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        lands(&mut g, p, 3);
        let h = hospitality(&mut g, p);
        start(&mut g, p, h);
        finish(&mut g);
        g.add_counter(h, CounterKind::PlusOne, 2);
        assert_eq!(stats(&g, h), (5, 5));
        let item = g.activation_item(p, h, 0);
        g.set_base_stats(&item, Subject::Source, Some(1), Some(4));
        assert_eq!(stats(&g, h), (3, 6));
        start(&mut g, p, h);
        finish(&mut g);
        assert_eq!(
            stats(&g, h),
            (5, 5),
            "later acquired setter beats prior7b setter on both axes"
        );
        let first = g.self_granted_ability_views().unwrap()[0].timestamp;
        let item = g.activation_item(p, h, 0);
        g.set_base_stats(&item, Subject::Source, Some(7), Some(1));
        assert_eq!(stats(&g, h), (9, 3));
        assert_eq!(g.self_granted_ability_views().unwrap()[0].timestamp, first);
    }
}
#[test]
fn beorn_hospitality_ability_loss_new_acquisition_and_departure_are_exact() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        lands(&mut g, p, 3);
        let h = hospitality(&mut g, p);
        start(&mut g, p, h);
        finish(&mut g);
        g.add_counter(h, CounterKind::PlusOne, 2);
        let old = g.self_granted_ability_views().unwrap()[0].timestamp;
        start(&mut g, p, h);
        let object = g.object_ref(h);
        g.blank_and_grant_haste(object, object);
        assert_eq!(stats(&g, h), (2, 2));
        assert!(!g.self_granted_ability_views().unwrap()[0].available);
        finish(&mut g);
        assert_eq!(stats(&g, h), (5, 5));
        let views = g.self_granted_ability_views().unwrap();
        assert_eq!(views[0].timestamp, old);
        assert!(!views[0].available);
        assert!(views[1].available);
        g.move_object(h, Zone::Hand);
        assert!(g.state.self_granted_abilities.is_none());
        g.state.active = p;
        g.state.priority = p;
        g.state.step = Step::Main1;
        g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 1];
        cast(&mut g, h, None);
        finish(&mut g);
        assert_ne!(g.object_ref(h), object);
        assert_eq!(g.current_kinds(h), 32);
        assert!(!g.has_subtype(h, "Bear"));
        start(&mut g, p, h);
        g.move_object(h, Zone::Hand);
        finish(&mut g);
        assert!(g.state.self_granted_abilities.is_none());
    }
}
#[test]
fn beorn_hospitality_wire_distinguishes_permanent_and_old_temporary_lifetimes() {
    let p = Player::P0;
    let mut g = scenario(p);
    lands(&mut g, p, 3);
    let h = hospitality(&mut g, p);
    start(&mut g, p, h);
    finish(&mut g);
    let item = g.activation_item(p, h, 0);
    g.grant_self_ability(&item, SelfGrantedAbilityKind::ControllerHandPower);
    let wire = serde_json::to_value(g.observe(p)).unwrap();
    let rows = wire["self_granted_abilities"].as_array().unwrap();
    assert_eq!(rows[0]["kind"], "ControllerLandStats");
    assert!(rows[0].get("expires_turn").is_none());
    assert_eq!(rows[1]["expires_turn"], g.state.turn);
    let acquired = rows[0]["timestamp"].clone();
    g.finish_cleanup();
    let wire = serde_json::to_value(g.observe(p)).unwrap();
    let rows = wire["self_granted_abilities"].as_array().unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["timestamp"], acquired);
}
#[test]
fn beorn_hospitality_zero_lands_dies_as_zero_toughness_and_forgets_grant() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let h = hospitality(&mut g, p);
        start(&mut g, p, h);
        finish(&mut g);
        assert_eq!(g.object_view(h).zone, Zone::Graveyard);
        assert!(g.state.self_granted_abilities.is_none());
        assert!(!g.has_subtype(h, "Bear"));
    }
}
#[test]
fn beorn_hospitality_landfall_no_owned_target_and_queued_source_departure() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let h = hospitality(&mut g, p);
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        enter(&mut g, p);
        finish(&mut g);
        assert_eq!(g.counter_count(enemy, CounterKind::PlusOne), 0);
        let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        enter(&mut g, p);
        select(&mut g, Some(bear));
        g.move_object(h, Zone::Hand);
        finish(&mut g);
        assert_eq!(g.counter_count(bear, CounterKind::PlusOne), 1);
    }
}
#[test]
fn beorn_hospitality_self_granted_layer7b_overrides_actual_cda_layer7a() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        lands(&mut g, p, 3);
        let h = hospitality(&mut g, p);
        start(&mut g, p, h);
        // Copy changes retain the exact object and queued original activation.
        // The copied definition's real CDA gives0/1 with empty graveyards.
        g.apply_temporary_copy(
            g.object_ref(h),
            CopyRecipe::new(Card::from_name("Tarmogoyf").unwrap()),
            CopyDeadline::EndOfTurn,
        );
        assert_eq!(stats(&g, h), (0, 1));
        finish(&mut g);
        assert_eq!(stats(&g, h), (3, 3));
        assert!(g.has_subtype(h, "Lhurgoyf") && g.has_subtype(h, "Bear"));
        g.add_counter(h, CounterKind::PlusOne, 1);
        assert_eq!(stats(&g, h), (4, 4));
        g.finish_cleanup();
        assert_eq!(
            g.object_view(h).card,
            Card::from_name("Beorn's Hospitality").unwrap()
        );
        assert_eq!(stats(&g, h), (4, 4));
        assert!(g.has_subtype(h, "Bear"));
    }
}
