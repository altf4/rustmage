//! Whole-card private entitlements and live Goblin combat/mana rules.
mod support;
use rustmage::Card;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;
fn idle(g: &Game) -> bool {
    g.state.pending == Pending::Priority
        && g.state.stack.is_empty()
        && g.state.pending_triggers.is_empty()
        && g.state.ordering_triggers.is_empty()
        && g.state.resolving.is_none()
}
fn drain(g: &mut Game) {
    for _ in 0..80 {
        g.prepare_triggers();
        if idle(g) {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("bounded Gonti/Howlsquad resolution");
}
fn main(g: &mut Game, p: Player) {
    g.state.active = p;
    g.state.priority = p;
    g.state.step = Step::Main2;
    g.state.pending = Pending::Priority;
    g.state.passes = 0;
}
fn treasures(g: &Game, p: Player) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter(|(i, o)| {
            o.zone == Zone::Battlefield
                && o.controller == p
                && o.is_token()
                && g.has_subtype(*i as CardId, "Treasure")
        })
        .map(|(i, _)| i as CardId)
        .collect()
}
fn combat_trigger(g: &mut Game, observer: CardId, attackers: &[CardId]) {
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    g.state.priority = g.state.active;
    g.state.passes = 0;
    for &card in attackers {
        act(g, Action::Attack { card });
    }
    act(g, Action::FinishAttackers);
    for _ in 0..48 {
        if g.state
            .stack
            .iter()
            .chain(&g.state.pending_triggers)
            .chain(&g.state.ordering_triggers)
            .any(|s| {
                s.source.is_some_and(|x| x.id == observer)
                    && matches!(s.kind, StackKind::TriggeredAbility { .. })
            })
        {
            return;
        }
        choose(g, |a| {
            matches!(
                a,
                Action::Pass
                    | Action::FinishBlockers
                    | Action::AssignDamage {
                        target: Target::Player(_),
                        ..
                    }
                    | Action::PutTrigger { .. }
            )
        });
    }
    panic!("no combat trigger");
}
fn exile_cast(g: &mut Game, id: CardId, target: Target) {
    act(g, Action::AnnounceExile { card: id });
    act(g, Action::ChooseTarget { target });
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
    pay(g);
}
#[test]
fn gonti_foreign_spell_treasure_belongs_to_actual_caster_even_on_an_opponents_turn_or_countered_spell()
 {
    for observer in [Player::P0, Player::P1] {
        for caster in [Player::P0, Player::P1] {
            for foreign in [false, true] {
                let mut g = scenario(caster);
                put(&mut g, observer, "Gonti, Night Minister", Zone::Battlefield);
                let turtle = put(&mut g, caster.other(), "Aegis Turtle", Zone::Battlefield);
                g.state.active = caster.other();
                g.state.priority = caster;
                let spell = put(
                    &mut g,
                    if foreign { caster.other() } else { caster },
                    "Shock",
                    if foreign { Zone::Exile } else { Zone::Hand },
                );
                if foreign {
                    let object = g.object_ref(spell);
                    g.grant_exile_play_permission(caster, object, false);
                    exile_cast(&mut g, spell, Target::Card(turtle));
                    let stack = g
                        .state
                        .stack
                        .iter()
                        .find(|s| {
                            s.source.is_some_and(|x| x.id == spell)
                                && matches!(s.kind, StackKind::Spell)
                        })
                        .unwrap()
                        .id;
                    assert!(g.counter_stack_item(stack, false));
                } else {
                    cast(&mut g, spell, Some(Target::Card(turtle)));
                }
                drain(&mut g);
                let tokens = treasures(&g, caster);
                assert_eq!(tokens.len(), usize::from(foreign));
                assert!(treasures(&g, caster.other()).is_empty());
                if foreign {
                    let token = tokens[0];
                    assert_eq!(g.object_view(token).colors, 0);
                    assert!(!g.object_view(token).tapped);
                    assert_eq!(g.object_view(token).owner, caster);
                    g.state.players[caster.index()].mana = [0; 6];
                    g.state.priority = caster;
                    choose(
                        &mut g,
                        |a| matches!(a,Action::ActivateMana{card,production,..}if *card==token&&*production==[0,0,0,1,0,0]),
                    );
                    assert_eq!(g.state.players[caster.index()].mana, [0, 0, 0, 1, 0, 0]);
                    assert_eq!(g.state.objects[token as usize].zone, Zone::Outside);
                }
            }
        }
    }
}
#[test]
fn gonti_actual_combat_freezes_damage_dealer_controller_and_face_down_permission_survives_both_sources()
 {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Gonti, Night Minister", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 2, 0, 0, 2];
        cast(&mut g, source, None);
        drain(&mut g);
        assert_eq!(stats(&g, source), (3, 4));
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        let dealer = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.state.objects[dealer as usize].controller = p;
        let top = put(&mut g, p.other(), "Shock", Zone::Library);
        combat_trigger(&mut g, source, &[dealer]);
        let packet = g
            .state
            .stack
            .iter()
            .chain(&g.state.pending_triggers)
            .chain(&g.state.ordering_triggers)
            .find(|s| s.source.is_some_and(|x| x.id == source))
            .unwrap();
        assert_eq!(packet.event_player, Some(p));
        g.change_controller(g.object_ref(dealer), p.other());
        g.move_object(dealer, Zone::Graveyard);
        g.move_object(source, Zone::Graveyard);
        drain(&mut g);
        assert_eq!(g.object_view(top).zone, Zone::Exile);
        assert!(
            g.observe(p)
                .objects
                .iter()
                .any(|o| o.id == top && o.card == Card::Shock)
        );
        assert!(!g.observe(p.other()).objects.iter().any(|o| o.id == top));
        let row = g
            .state
            .exile_permissions
            .iter()
            .find(|r| r.object == g.object_ref(top))
            .unwrap();
        assert_eq!(row.player, p);
        assert_eq!(row.expiry, ExilePlayExpiry::WhileExiled);
        assert!(row.spend_any_mana);
        g.finish_cleanup();
        main(&mut g, p);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        exile_cast(&mut g, top, Target::Player(p.other()));
        drain(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(g.object_view(top).zone, Zone::Graveyard);
        assert_eq!(g.object_view(top).owner, p.other());
        assert!(g.observe(p.other()).objects.iter().any(|o| o.id == top));
    }
}
#[test]
fn gonti_each_damaging_creature_exiles_a_card_and_land_permissions_keep_normal_timing_and_shared_limit()
 {
    let p = Player::P0;
    let mut g = scenario(p);
    let source = put(&mut g, p, "Gonti, Night Minister", Zone::Battlefield);
    let a = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let b = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let first = put(&mut g, p.other(), "Mountain", Zone::Library);
    let second = put(&mut g, p.other(), "Forest", Zone::Library);
    combat_trigger(&mut g, source, &[a, b]);
    drain(&mut g);
    for id in [first, second] {
        assert_eq!(g.object_view(id).zone, Zone::Exile);
        assert!(!g.observe(p.other()).objects.iter().any(|o| o.id == id));
    }
    reject(&mut g, Action::PlayLand { card: first });
    main(&mut g, p);
    act(&mut g, Action::PlayLand { card: first });
    assert_eq!(g.object_view(first).controller, p);
    assert_eq!(g.object_view(first).owner, p.other());
    assert!(
        treasures(&g, p).is_empty(),
        "playing a foreign land is not casting a spell"
    );
    reject(&mut g, Action::PlayLand { card: second });
    g.move_object(second, Zone::Graveyard);
    g.move_object(second, Zone::Exile);
    g.state.land_played = false;
    reject(&mut g, Action::PlayLand { card: second });
}
#[test]
fn gonti_noncombat_damage_and_combat_damage_to_its_controller_never_exile() {
    let p = Player::P0;
    let mut g = scenario(p);
    put(&mut g, p, "Gonti, Night Minister", Zone::Battlefield);
    let top = put(&mut g, p.other(), "Shock", Zone::Library);
    let shock = put(&mut g, p, "Shock", Zone::Hand);
    cast(&mut g, shock, Some(Target::Player(p.other())));
    drain(&mut g);
    assert!(g.state.exile_permissions.is_empty());
    assert_eq!(g.object_view(top).zone, Zone::Library);
    let attacker = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
    g.state.active = p.other();
    g.state.priority = p.other();
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    act(&mut g, Action::Attack { card: attacker });
    act(&mut g, Action::FinishAttackers);
    for _ in 0..48 {
        if g.state.step == Step::EndCombat && idle(&g) {
            break;
        }
        choose(&mut g, |a| {
            matches!(
                a,
                Action::Pass
                    | Action::FinishBlockers
                    | Action::AssignDamage { .. }
                    | Action::PutTrigger { .. }
            )
        });
    }
    assert_eq!(g.state.players[p.index()].life, 18);
    assert!(g.state.exile_permissions.is_empty());
}
#[test]
fn gonti_prevented_exile_keeps_private_look_only_and_owns_replacement_result_across_raw_index_restore()
 {
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|v| v.set(None));
        }
    }
    let _reset = Reset;
    let p = Player::P1;
    let mut g = scenario(p);
    let source = put(&mut g, p, "Gonti, Night Minister", Zone::Battlefield);
    let dealer = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let top = put(&mut g, p.other(), "Shock", Zone::Library);
    let exact = g.object_ref(top);
    combat_trigger(&mut g, source, &[dealer]);
    g.prepare_triggers();
    zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|v| v.set(Some(exact)));
    g.state.zone_rules.get_or_insert_default();
    g.state.passes = 1;
    g.state.priority = p.other();
    let saved = g.snapshot();
    let mut raw = g.clone();
    let mut indexed = g.clone();
    let i = indexed
        .legal_actions()
        .iter()
        .position(|a| *a == Action::Pass)
        .unwrap();
    act(&mut raw, Action::Pass);
    indexed
        .apply_index(p.other(), indexed.revision(), i)
        .unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    drain(&mut raw);
    assert_eq!(raw.object_view(top).zone, Zone::Library);
    assert_eq!(
        raw.observe(p).known_library_top[p.other().index()]
            .unwrap()
            .object,
        exact
    );
    assert!(raw.observe(p.other()).known_library_top[p.other().index()].is_none());
    assert!(raw.public_library_top(p.other()).is_none());
    assert!(raw.state.exile_permissions.is_empty());
    indexed.restore(&saved).unwrap();
    drop(g);
    act(&mut indexed, Action::Pass);
    drain(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
    zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|v| v.set(None));
    indexed.shuffle_library_cards(p.other());
    assert!(indexed.observe(p).known_library_top[p.other().index()].is_none());
}
fn created_goblins(g: &Game) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter(|(i, o)| {
            o.zone == Zone::Battlefield && o.is_token() && g.has_subtype(*i as CardId, "Goblin")
        })
        .map(|(i, _)| i as CardId)
        .collect()
}
fn begin_combat(g: &mut Game) {
    g.begin_step(Step::BeginCombat);
    g.give_priority();
    act(g, Action::Pass);
    drain(g);
}
fn attackers(g: &mut Game) {
    for _ in 0..12 {
        if g.state.pending == Pending::Attackers {
            return;
        }
        act(g, Action::Pass);
    }
    panic!("no attackers");
}
#[test]
fn howlsquad_actual_entry_starts_speed_and_live_other_goblin_haste_is_controller_relative() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let own = put(&mut g, p, "Goblin Piker", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Goblin Piker", Zone::Battlefield);
        let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let source = put(&mut g, p, "Howlsquad Heavy", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 2];
        cast(&mut g, source, None);
        drain(&mut g);
        assert_eq!(stats(&g, source), (2, 3));
        assert_eq!(g.state.players[p.index()].speed, 1);
        assert_eq!(g.state.players[p.other().index()].speed, 0);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert!(g.keywords(own).contains(Keywords::HASTE));
        for id in [source, enemy, bear] {
            assert!(!g.keywords(id).contains(Keywords::HASTE));
        }
        let late = put(&mut g, p, "Goblin Piker", Zone::Battlefield);
        assert!(g.keywords(late).contains(Keywords::HASTE));
        g.change_controller(g.object_ref(source), p.other());
        assert!(!g.keywords(own).contains(Keywords::HASTE));
        assert!(g.keywords(enemy).contains(Keywords::HASTE));
        assert!(!g.keywords(source).contains(Keywords::HASTE));
        g.move_object(source, Zone::Hand);
        assert!(!g.keywords(enemy).contains(Keywords::HASTE));
        assert_eq!(g.state.players[p.index()].speed, 1);
    }
}
#[test]
fn howlsquad_own_combat_creates_one_red_goblin_that_must_attack_this_combat_and_requirement_expires()
 {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Howlsquad Heavy", Zone::Battlefield);
        begin_combat(&mut g);
        let tokens = created_goblins(&g);
        assert_eq!(tokens.len(), 1);
        let token = tokens[0];
        assert_eq!(stats(&g, token), (1, 1));
        assert_eq!(g.object_view(token).colors, 8);
        assert!(!g.object_view(token).tapped);
        assert!(g.keywords(token).contains(Keywords::HASTE));
        assert_eq!(
            g.observe(p).must_attack_this_combat,
            vec![g.object_ref(token)]
        );
        attackers(&mut g);
        reject(&mut g, Action::FinishAttackers);
        act(&mut g, Action::Attack { card: token });
        act(&mut g, Action::FinishAttackers);
        g.move_object(source, Zone::Graveyard);
        for _ in 0..64 {
            if g.state.step == Step::Main2 && idle(&g) {
                break;
            }
            choose(&mut g, |a| {
                matches!(
                    a,
                    Action::Pass
                        | Action::FinishBlockers
                        | Action::AssignDamage { .. }
                        | Action::PutTrigger { .. }
                )
            });
        }
        assert!(g.observe(p).must_attack_this_combat.is_empty());
        assert!(!g.attacks_each_combat(token));
    }
    let p = Player::P0;
    let mut g = scenario(p);
    put(&mut g, p, "Howlsquad Heavy", Zone::Battlefield);
    g.state.active = p.other();
    g.state.priority = p.other();
    begin_combat(&mut g);
    assert!(created_goblins(&g).is_empty());
}
#[test]
fn howlsquad_queued_creation_survives_source_but_without_haste_summoning_sick_token_is_not_forced_to_attack()
 {
    for before in [false, true] {
        let p = Player::P0;
        let mut g = scenario(p);
        let source = put(&mut g, p, "Howlsquad Heavy", Zone::Battlefield);
        g.begin_step(Step::BeginCombat);
        g.give_priority();
        act(&mut g, Action::Pass);
        if before {
            g.move_object(source, Zone::Graveyard);
        }
        drain(&mut g);
        let token = created_goblins(&g)[0];
        if !before {
            g.move_object(source, Zone::Graveyard);
        }
        assert!(g.attacks_each_combat(token));
        assert!(!g.keywords(token).contains(Keywords::HASTE));
        attackers(&mut g);
        reject(&mut g, Action::Attack { card: token });
        act(&mut g, Action::FinishAttackers);
    }
}
#[test]
fn howlsquad_max_speed_tap_mana_counts_current_owned_goblins_including_itself_and_funds_real_red_spell()
 {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Howlsquad Heavy", Zone::Battlefield);
        let other = put(&mut g, p, "Goblin Piker", Zone::Battlefield);
        put(&mut g, p.other(), "Goblin Piker", Zone::Battlefield);
        put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let spell = put(&mut g, p, "Lightning Strike", Zone::Hand);
        g.state.players[p.index()].mana = [0; 6];
        let mana = Action::ActivateMana {
            card: source,
            ability: 0,
            payment: [0; 6],
            counter_payment: Counters::default(),
            production: [0, 0, 0, 2, 0, 0],
        };
        g.state.players[p.index()].speed = 3;
        reject(&mut g, mana.clone());
        reject(&mut g, Action::AnnounceCast { card: spell });
        g.state.players[p.index()].speed = 4;
        g.state.objects[source as usize].summoning_sick = true;
        reject(&mut g, mana.clone());
        g.state.objects[source as usize].summoning_sick = false;
        g.state.objects[source as usize].tapped = true;
        reject(&mut g, mana.clone());
        g.state.objects[source as usize].tapped = false;
        assert!(
            g.legal_actions()
                .contains(&Action::AnnounceCast { card: spell })
        );
        act(&mut g, Action::AnnounceCast { card: spell });
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        act(&mut g, mana);
        assert!(g.object_view(source).tapped);
        pay(&mut g);
        drain(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(g.state.players[p.other().index()].life, 17);
        g.state.objects[source as usize].tapped = false;
        g.change_controller(g.object_ref(other), p.other());
        choose(
            &mut g,
            |a| matches!(a,Action::ActivateMana{card,production,..}if *card==source&&*production==[0,0,0,1,0,0]),
        );
        assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 1, 0, 0]);
    }
}
