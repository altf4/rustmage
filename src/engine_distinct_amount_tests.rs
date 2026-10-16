use super::tests::{act, cast, scenario, spawn};
use super::*;

fn named(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    let c = Card::from_name(name).unwrap();
    assert!(c.is_supported(), "{name}");
    spawn(g, p, c, zone)
}
fn funded(g: &mut Game, name: &str) -> CardId {
    let id = named(g, Player::P0, name, Zone::Hand);
    g.state.players[0].mana = [20; 6];
    id
}
fn replay(g: &mut Game, a: Action) {
    let before = g.snapshot();
    let rev = g.revision();
    let p = g.actor().unwrap();
    let mut fork = g.clone();
    act(&mut fork, a.clone());
    act(g, a.clone());
    assert_eq!(fork.snapshot(), g.snapshot());
    let after = g.snapshot();
    g.restore(&before).unwrap();
    assert_eq!(g.apply_index(p, rev, 0), Err(Error::StaleRevision));
    act(g, a);
    assert_eq!(g.snapshot(), after);
}
fn passes(g: &mut Game) {
    replay(g, Action::Pass);
    replay(g, Action::Pass);
}
fn trigger(g: &mut Game, targets: &[Target]) {
    g.prepare_triggers();
    let mut n = 0;
    for _ in 0..20 {
        if g.state.pending == Pending::Priority {
            return;
        }
        let actions = g.legal_actions();
        let a = if actions
            .iter()
            .any(|a| matches!(a, Action::ChooseTarget { .. }))
            && n < targets.len()
        {
            let a = Action::ChooseTarget { target: targets[n] };
            n += 1;
            a
        } else {
            actions
                .into_iter()
                .find(|a| matches!(a, Action::PutTrigger { .. } | Action::FinishTargets))
                .unwrap()
        };
        replay(g, a);
    }
    panic!("trigger announcement stalled");
}
fn enter(g: &mut Game, name: &str) -> CardId {
    let id = named(g, Player::P0, name, Zone::Hand);
    g.move_object(id, Zone::Battlefield);
    id
}
fn illegal(g: &mut Game, a: Action) {
    let before = g.snapshot();
    let rev = g.revision();
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), rev);
}

const OWN_BATTLEFIELD: CountFilter = CountFilter {
    zone: CountZone::Battlefield,
    relation: 0,
    kind: 0,
    exclude: 0,
    subtype: None,
    basic: false,
    other: false,
    cards_only: false,
};
fn colors(g: &Game) -> i128 {
    g.stat_value(
        StatValue::Colors(OWN_BATTLEFIELD),
        g.object_ref(0),
        Player::P0,
    )
}
fn card_types(g: &Game, p: Player) -> i128 {
    g.stat_value(
        StatValue::CardTypes(CountFilter {
            zone: CountZone::Graveyard,
            cards_only: true,
            ..OWN_BATTLEFIELD
        }),
        g.object_ref(0),
        p,
    )
}
fn settle(g: &mut Game) {
    g.prepare_triggers();
    for _ in 0..1000 {
        if g.state.stack.is_empty()
            && g.state.resolving.is_none()
            && g.state.announcing.is_none()
            && g.state.pending_triggers.is_empty()
            && g.state.ordering_triggers.is_empty()
        {
            return;
        }
        let actions = g.legal_actions();
        let a = actions[crate::policy::smoke_index(g, &actions)].clone();
        replay(g, a);
    }
    panic!("resolution stalled");
}

#[test]
fn distinct_colors_union_hybrid_multicolor_tokens_and_ignore_mana_identity() {
    let mut g = scenario();
    named(&mut g, Player::P0, "Plains", Zone::Battlefield);
    named(&mut g, Player::P0, "Three Tree Mascot", Zone::Battlefield);
    assert_eq!(colors(&g), 0);
    named(&mut g, Player::P0, "Glister Bairn", Zone::Battlefield); // Both hybrid colors.
    named(&mut g, Player::P0, "Burrowguard Mentor", Zone::Battlefield); // White and green.
    named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield); // Duplicate green.
    named(&mut g, Player::P1, "Cat-Gator", Zone::Battlefield);
    named(&mut g, Player::P0, "Explosive Prodigy", Zone::Hand);
    named(&mut g, Player::P0, "Shimmercreep", Zone::Graveyard);
    assert_eq!(colors(&g), 3);
    let treasure = Card::catalogue()
        .find(|c| c.is_token() && c.has_subtype("Treasure"))
        .unwrap();
    g.create_tokens(Player::P0, treasure, 2, false);
    assert_eq!(colors(&g), 3);
    let goblin = Card::catalogue()
        .find(|c| c.is_token() && c.has_subtype("Goblin") && c.colors() == 8)
        .unwrap();
    g.create_tokens(Player::P0, goblin, 2, false);
    assert_eq!(colors(&g), 4);
    named(&mut g, Player::P0, "Cat-Gator", Zone::Battlefield);
    assert_eq!(colors(&g), 5);
}

#[test]
fn owned_card_types_are_a_union_include_kindred_and_exclude_graveyard_tokens() {
    let mut g = scenario();
    for name in [
        "Three Tree Mascot",
        "Clachan Festival",
        "Plains",
        "Shock",
        "Rumbling Rockslide",
        "Ajani, Caller of the Pride",
    ] {
        named(&mut g, Player::P0, name, Zone::Graveyard);
    }
    assert_eq!(card_types(&g, Player::P0), 8);
    assert_eq!(card_types(&g, Player::P1), 0);
    named(&mut g, Player::P1, "Shock", Zone::Graveyard);
    assert_eq!(card_types(&g, Player::P1), 1);
    let token = Card::catalogue()
        .find(|c| c.is_token() && c.is_creature())
        .unwrap();
    spawn(&mut g, Player::P1, token, Zone::Graveyard); // Before token SBA; it still isn't a card.
    assert_eq!(card_types(&g, Player::P1), 1);
    named(&mut g, Player::P0, "Cat-Gator", Zone::Graveyard);
    assert_eq!(card_types(&g, Player::P0), 8);
}

#[test]
fn explosive_prodigy_counts_live_colors_after_source_leaves_and_preserves_target_legality() {
    let mut g = scenario();
    let victim = named(&mut g, Player::P1, "Shivan Dragon", Zone::Battlefield);
    named(&mut g, Player::P0, "Burrowguard Mentor", Zone::Battlefield);
    let source = enter(&mut g, "Explosive Prodigy");
    g.prepare_triggers();
    illegal(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(source),
        },
    );
    trigger(&mut g, &[Target::Card(victim)]);
    g.move_object(source, Zone::Hand);
    named(&mut g, Player::P0, "Shinestriker", Zone::Battlefield);
    passes(&mut g);
    assert_eq!(g.object_view(victim).damage, 3); // W,G,U; departed red doesn't count.
    assert_eq!(g.state.players[1].life, 20);
}

#[test]
fn luminollusk_gain_counts_itself_and_new_colors_but_is_one_event() {
    let mut g = scenario();
    let pride = named(&mut g, Player::P0, "Ajani's Pridemate", Zone::Battlefield);
    let source = enter(&mut g, "Luminollusk");
    trigger(&mut g, &[]);
    assert!(g.keywords(source).contains(Keywords::DEATHTOUCH));
    named(&mut g, Player::P0, "Glister Bairn", Zone::Battlefield);
    passes(&mut g);
    assert_eq!(g.state.players[0].life, 23);
    assert_eq!(g.state.players[0].life_gained_this_turn, 3);
    assert_eq!(g.state.players[0].life_gain_events_this_turn, 1);
    settle(&mut g);
    assert_eq!(g.object_view(pride).counters.plus_one, 1);
}

#[test]
fn shimmercreep_drains_frozen_colors_without_damage_or_lifelink() {
    let mut g = scenario();
    named(&mut g, Player::P0, "Burrowguard Mentor", Zone::Battlefield);
    let source = enter(&mut g, "Shimmercreep");
    trigger(&mut g, &[]);
    g.state.objects[source as usize].gained_keywords = Keywords::LIFELINK;
    assert!(g.keywords(source).contains(Keywords::MENACE));
    passes(&mut g);
    assert_eq!(g.state.players[1].life, 17);
    assert_eq!(g.state.players[0].life, 23);
    assert_eq!(g.state.players[0].life_gain_events_this_turn, 1);
    assert_eq!(g.state.players[0].life_gained_this_turn, 3);
    assert!(!g.object_view(source).dealt_damage_this_turn);
}

#[test]
fn shimmercreep_gain_failure_rolls_back_earlier_opponent_loss() {
    let mut g = scenario();
    enter(&mut g, "Shimmercreep");
    trigger(&mut g, &[]);
    g.state.players[0].life = i128::MAX;
    replay(&mut g, Action::Pass);
    let before = g.snapshot();
    let rev = g.revision();
    assert!(matches!(
        g.apply(g.actor().unwrap(), Action::Pass),
        Err(Error::CapacityExceeded(_))
    ));
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), rev);
    assert_eq!(g.state.players[1].life, 20);
}

#[test]
fn shinestriker_draws_one_per_color_with_individual_ordinals_and_private_cards() {
    let mut g = scenario();
    let each = named(&mut g, Player::P0, "Clinquant Skymage", Zone::Battlefield);
    let second = named(&mut g, Player::P0, "Erudite Wizard", Zone::Battlefield);
    named(&mut g, Player::P0, "Burrowguard Mentor", Zone::Battlefield);
    enter(&mut g, "Shinestriker");
    trigger(&mut g, &[]);
    let library = g.state.players[0].library.len();
    passes(&mut g);
    assert_eq!(g.state.players[0].library.len(), library - 3);
    assert_eq!(g.hand_count(Player::P0), 3);
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 3);
    assert_eq!(
        g.observe(Player::P1)
            .objects
            .iter()
            .filter(|o| o.owner == Player::P0 && o.zone == Zone::Hand)
            .count(),
        0
    );
    let events: Vec<_> = g
        .state
        .stack
        .iter()
        .chain(&g.state.ordering_triggers)
        .chain(&g.state.pending_triggers)
        .filter(|s| s.card == each)
        .map(|s| s.draw_event.unwrap().ordinal)
        .collect();
    assert_eq!(events, vec![1, 2, 3]);
    settle(&mut g);
    assert_eq!(g.object_view(each).counters.plus_one, 3);
    assert_eq!(g.object_view(second).counters.plus_one, 1);
}

#[test]
fn shinestriker_empty_draws_are_bounded_and_loss_occurs_after_successful_draws() {
    let mut g = scenario();
    named(&mut g, Player::P0, "Burrowguard Mentor", Zone::Battlefield);
    let remove: Vec<_> = g.state.players[0].library.iter().copied().skip(1).collect();
    for id in remove {
        g.move_object(id, Zone::Exile);
    }
    enter(&mut g, "Shinestriker");
    trigger(&mut g, &[]);
    passes(&mut g);
    assert_eq!(g.hand_count(Player::P0), 1);
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 1);
    assert_eq!(g.outcome(), Some(Outcome::Winner(Player::P1)));
}

#[test]
fn glister_bairn_fixes_bonus_at_resolution_rejects_self_and_expires_at_cleanup() {
    let mut g = scenario();
    let source = named(&mut g, Player::P0, "Glister Bairn", Zone::Battlefield);
    let bear = named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    let enemy = named(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
    g.begin_step(Step::BeginCombat);
    g.give_priority();
    g.prepare_triggers();
    illegal(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(source),
        },
    );
    illegal(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(enemy),
        },
    );
    trigger(&mut g, &[Target::Card(bear)]);
    named(&mut g, Player::P0, "Explosive Prodigy", Zone::Battlefield);
    passes(&mut g);
    assert_eq!(
        (g.object_view(bear).power, g.object_view(bear).toughness),
        (5, 5)
    );
    named(&mut g, Player::P0, "Shimmercreep", Zone::Battlefield);
    assert_eq!(g.object_view(bear).power, 5);
    g.finish_cleanup();
    assert_eq!(
        (g.object_view(bear).power, g.object_view(bear).toughness),
        (2, 2)
    );
}

#[test]
fn glister_stale_target_generation_fizzles_and_opponents_turn_has_no_trigger() {
    let mut g = scenario();
    named(&mut g, Player::P0, "Glister Bairn", Zone::Battlefield);
    let bear = named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    g.begin_step(Step::BeginCombat);
    g.give_priority();
    trigger(&mut g, &[Target::Card(bear)]);
    g.move_object(bear, Zone::Hand);
    g.move_object(bear, Zone::Battlefield);
    passes(&mut g);
    assert_eq!(g.object_view(bear).power, 2);
    g.state.active = Player::P1;
    g.begin_step(Step::BeginCombat);
    g.give_priority();
    g.prepare_triggers();
    assert!(g.state.stack.is_empty());
}

#[test]
fn squawkroaster_characteristic_power_applies_in_every_zone_and_tracks_color_union() {
    let mut g = scenario();
    let source = named(&mut g, Player::P0, "Squawkroaster", Zone::Hand);
    assert_eq!(g.object_view(source).power, 0);
    named(&mut g, Player::P0, "Burrowguard Mentor", Zone::Battlefield);
    assert_eq!(g.object_view(source).power, 2);
    g.move_object(source, Zone::Battlefield);
    assert_eq!(g.object_view(source).power, 3);
    assert!(g.keywords(source).contains(Keywords::DOUBLE_STRIKE));
    g.state.objects[source as usize].counters.plus_one = 2;
    assert_eq!(g.object_view(source).power, 5);
    g.move_object(source, Zone::Graveyard);
    assert_eq!(g.object_view(source).power, 2);
    g.move_object(source, Zone::Exile);
    assert_eq!(g.object_view(source).power, 2);
}

#[test]
fn peer_discards_complete_hand_privately_before_counting_own_graveyard_types() {
    let mut g = scenario();
    named(&mut g, Player::P0, "Rumbling Rockslide", Zone::Graveyard);
    named(&mut g, Player::P1, "Clachan Festival", Zone::Graveyard); // Opponent's two types don't count.
    let a = named(&mut g, Player::P0, "Three Tree Mascot", Zone::Hand);
    let b = named(&mut g, Player::P0, "Plains", Zone::Hand);
    let spell = funded(&mut g, "Peer Past the Veil");
    cast(&mut g, spell, None);
    passes(&mut g);
    assert_eq!(g.observe(Player::P0).decision, "effect_discard");
    assert_eq!(g.hand_count(Player::P0), 2);
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 0);
    replay(&mut g, Action::Discard { card: a });
    illegal(&mut g, Action::Discard { card: a });
    assert_eq!(g.object_view(a).zone, Zone::Hand);
    assert!(
        g.observe(Player::P1)
            .objects
            .iter()
            .all(|o| o.id != a && o.id != b)
    );
    assert!(g.observe(Player::P1).effect_discards.is_empty());
    replay(&mut g, Action::Discard { card: b });
    assert_eq!(g.object_view(a).zone, Zone::Graveyard);
    assert_eq!(g.object_view(b).zone, Zone::Graveyard);
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 4); // Sorcery, artifact, creature, land.
    assert_eq!(g.hand_count(Player::P0), 4);
    assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
    assert_eq!(card_types(&g, Player::P0), 5); // Only now does Peer add instant.
}

#[test]
fn peer_empty_hand_and_graveyard_draw_zero_and_never_counts_itself_on_stack() {
    let mut g = scenario();
    let spell = funded(&mut g, "Peer Past the Veil");
    cast(&mut g, spell, None);
    passes(&mut g);
    assert_eq!(g.hand_count(Player::P0), 0);
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 0);
    assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
    assert_eq!(card_types(&g, Player::P0), 1);
}

#[test]
fn frontline_rush_both_modes_create_real_tokens_or_give_live_creature_count_bonus() {
    for mode in [0, 1] {
        let mut g = scenario();
        let bear = named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
        let spell = funded(&mut g, "Frontline Rush");
        replay(&mut g, Action::AnnounceCast { card: spell });
        replay(&mut g, Action::ChooseMode { mode });
        if mode == 1 {
            replay(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(bear),
                },
            );
        }
        let pay = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        replay(&mut g, pay);
        named(&mut g, Player::P0, "Three Tree Mascot", Zone::Battlefield);
        passes(&mut g);
        if mode == 0 {
            assert_eq!(
                g.state
                    .objects
                    .iter()
                    .filter(|o| o.zone == Zone::Battlefield
                        && o.card.is_token()
                        && o.card.has_subtype("Goblin"))
                    .count(),
                2
            );
        }
        assert_eq!(g.object_view(bear).power, if mode == 0 { 2 } else { 4 });
    }
}

#[test]
fn peer_draw_count_capacity_restores_last_private_discard_and_whole_batch() {
    let mut g = scenario();
    let a = named(&mut g, Player::P0, "Three Tree Mascot", Zone::Hand);
    let b = named(&mut g, Player::P0, "Plains", Zone::Hand);
    let spell = funded(&mut g, "Peer Past the Veil");
    cast(&mut g, spell, None);
    passes(&mut g);
    replay(&mut g, Action::Discard { card: a });
    g.state.players[0].cards_drawn_this_turn = u64::MAX;
    let before = g.snapshot();
    let rev = g.revision();
    let views = [
        serde_json::to_value(g.observe(Player::P0)).unwrap(),
        serde_json::to_value(g.observe(Player::P1)).unwrap(),
    ];
    for _ in 0..2 {
        assert!(matches!(
            g.apply(g.actor().unwrap(), Action::Discard { card: b }),
            Err(Error::CapacityExceeded(_))
        ));
        assert_eq!(g.snapshot(), before);
        assert_eq!(g.revision(), rev);
        assert_eq!(
            serde_json::to_value(g.observe(Player::P0)).unwrap(),
            views[0]
        );
        assert_eq!(
            serde_json::to_value(g.observe(Player::P1)).unwrap(),
            views[1]
        );
    }
    assert_eq!(g.object_view(a).zone, Zone::Hand);
    assert_eq!(g.object_view(b).zone, Zone::Hand);
    assert_eq!(g.object_view(spell).zone, Zone::Stack);
}

#[test]
fn glister_derived_power_overflow_restores_resolution_and_temporary_growth() {
    let mut g = scenario();
    named(&mut g, Player::P0, "Glister Bairn", Zone::Battlefield);
    let bear = named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    g.state.objects[bear as usize].growth = Stats {
        power: i128::MAX - 3,
        toughness: 0,
    };
    g.begin_step(Step::BeginCombat);
    g.give_priority();
    trigger(&mut g, &[Target::Card(bear)]);
    replay(&mut g, Action::Pass);
    let before = g.snapshot();
    let rev = g.revision();
    assert!(matches!(
        g.apply(g.actor().unwrap(), Action::Pass),
        Err(Error::CapacityExceeded(_))
    ));
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), rev);
    assert_eq!(g.object_view(bear).power, i128::MAX - 1);
}

#[test]
fn departed_last_color_makes_all_four_vivid_entry_effects_zero() {
    for name in [
        "Explosive Prodigy",
        "Luminollusk",
        "Shimmercreep",
        "Shinestriker",
    ] {
        let mut g = scenario();
        let enemy = named(&mut g, Player::P1, "Shivan Dragon", Zone::Battlefield);
        let source = enter(&mut g, name);
        let target = [Target::Card(enemy)];
        trigger(
            &mut g,
            if name == "Explosive Prodigy" {
                &target
            } else {
                &[]
            },
        );
        g.move_object(source, Zone::Hand);
        passes(&mut g);
        assert_eq!(g.state.players[0].life, 20);
        assert_eq!(g.state.players[1].life, 20);
        assert_eq!(g.state.players[0].life_gain_events_this_turn, 0);
        assert_eq!(g.state.players[0].cards_drawn_this_turn, 0);
        assert_eq!(g.object_view(enemy).damage, 0);
        assert!(g.state.stack.is_empty());
    }
}
