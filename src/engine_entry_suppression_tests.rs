use super::tests::{act, scenario, spawn};
use super::*;
const NAMES: [&str; 3] = ["Torpor Orb", "Doorkeeper Thrull", "Karn, Argent Defender"];
fn named(g: &mut Game, p: Player, n: &str, z: Zone) -> CardId {
    let c = Card::from_name(n).unwrap();
    assert!(c.is_supported(), "{n}");
    spawn(g, p, c, z)
}
fn setup(p: Player) -> Game {
    let mut g = scenario();
    g.state.active = p;
    g.state.priority = p;
    for s in &mut g.state.players {
        s.mana = [20; 6];
    }
    g
}
fn replay(g: &mut Game, a: Action) {
    let s = g.snapshot();
    let r = g.revision();
    let p = g.actor().unwrap();
    let mut fork = g.clone();
    act(&mut fork, a.clone());
    act(g, a.clone());
    let expected = g.snapshot();
    assert_eq!(fork.snapshot(), expected);
    g.restore(&s).unwrap();
    assert_eq!(g.apply_index(p, r, 0), Err(Error::StaleRevision));
    act(g, a);
    assert_eq!(g.snapshot(), expected);
}
fn choose(g: &mut Game) {
    let a = g
        .legal_actions()
        .into_iter()
        .find(|a| {
            matches!(
                a,
                Action::ChooseSpellCost { .. }
                    | Action::PayCosts { .. }
                    | Action::Pass
                    | Action::PutTrigger { .. }
                    | Action::Discard { .. }
                    | Action::ChooseTarget {
                        target: Target::Player(_)
                    }
                    | Action::FinishTargets
                    | Action::ArrangeLibrary { .. }
            )
        })
        .expect("fixture choice");
    replay(g, a);
}
fn settle(g: &mut Game) {
    g.prepare_triggers();
    for _ in 0..160 {
        if g.outcome().is_some() || g.state.stack.is_empty() && g.state.pending == Pending::Priority
        {
            return;
        }
        choose(g);
    }
    panic!("unsettled");
}
fn cast(g: &mut Game, id: CardId) {
    replay(g, Action::AnnounceCast { card: id });
    while g.observe(g.actor().unwrap()).announcing.is_some() {
        choose(g);
    }
    settle(g);
}
fn enter(g: &mut Game, ids: &[CardId]) {
    let events = g.move_objects_raw(ids, Zone::Battlefield);
    g.finish_zone_events(&events, &[]);
}
#[test]
fn all_three_whole_bodies_cast_and_suppress_enemy_entry_for_both_seats() {
    for p in [Player::P0, Player::P1] {
        for (i, n) in NAMES.iter().enumerate() {
            let mut g = setup(p);
            let source = named(&mut g, p, n, Zone::Hand);
            cast(&mut g, source);
            let v = g.object_view(source);
            assert_eq!(v.zone, Zone::Battlefield);
            assert_eq!(
                v.card.entry_trigger_suppression(),
                if i == 0 { 1 } else { 17 }
            );
            if i == 1 {
                assert_eq!((v.power, v.toughness), (1, 2));
                assert_eq!(v.keywords, Keywords::FLASH.union(Keywords::FLYING));
            }
            if i == 2 {
                assert_eq!((v.power, v.toughness), (1, 3));
                assert!(v.card.info().legendary);
            }
            // Transfer priority/active seat only as fixture setup; all casts are legal actions.
            g.state.active = p.other();
            g.state.priority = p.other();
            let hunter = named(&mut g, p.other(), "Helpful Hunter", Zone::Hand);
            let library = g.state.players[p.other().index()].library.len();
            cast(&mut g, hunter);
            assert_eq!(g.state.players[p.other().index()].library.len(), library);
        }
    }
}
#[test]
fn artifact_entry_uses_union_and_blocks_all_observers_including_self_once() {
    for p in [Player::P0, Player::P1] {
        for (i, n) in NAMES.iter().enumerate() {
            let mut g = setup(p);
            named(&mut g, p.other(), n, Zone::Battlefield);
            named(&mut g, p, "Pactdoll Terror", Zone::Battlefield);
            let artifact = named(&mut g, p, "Mephitic Draught", Zone::Hand);
            let before = g.state.players[p.index()].library.len();
            cast(&mut g, artifact);
            assert_eq!(
                g.state.players[p.index()].library.len(),
                before - usize::from(i == 0)
            );
            // Torpor allows both artifact-entry triggers: draw/lose1 and gain1/drain1.
            assert_eq!(g.state.players[p.index()].life, 20);
            assert_eq!(
                g.state.players[p.other().index()].life,
                if i == 0 { 19 } else { 20 }
            );
            let doll = named(&mut g, p, "Pactdoll Terror", Zone::Hand);
            let serial = g.state.next_stack_id;
            enter(&mut g, &[doll]);
            assert!(g.state.pending_triggers.is_empty());
            assert_eq!(g.state.next_stack_id, serial);
        }
    }
}
#[test]
fn simultaneous_suppressor_and_entrants_use_post_batch_sources_in_both_orders() {
    for n in NAMES {
        for reverse in [false, true] {
            let mut g = setup(Player::P0);
            named(&mut g, Player::P0, "Pactdoll Terror", Zone::Battlefield);
            let s = named(&mut g, Player::P1, n, Zone::Hand);
            let h = named(&mut g, Player::P0, "Helpful Hunter", Zone::Hand);
            let d = named(&mut g, Player::P0, "Pactdoll Terror", Zone::Hand);
            let mut batch = vec![s, h, d];
            if reverse {
                batch.reverse();
            }
            enter(&mut g, &batch);
            // Orb itself is an artifact owned by P1, so P0's Pactdoll ignores it.
            assert!(g.state.pending_triggers.is_empty(), "{n}");
            assert_eq!(g.state.next_stack_id, 0);
            assert!(g.state.static_sources.contains(&s));
        }
    }
}
#[test]
fn mixed_entry_batch_preserves_land_and_enchantment_triggers() {
    for n in NAMES {
        let mut g = setup(Player::P0);
        let s = named(&mut g, Player::P1, n, Zone::Hand);
        let h = named(&mut g, Player::P0, "Helpful Hunter", Zone::Hand);
        let land = named(&mut g, Player::P0, "Tranquil Cove", Zone::Hand);
        let vigil = named(&mut g, Player::P0, "Hopeful Vigil", Zone::Hand);
        enter(&mut g, &[h, land, s, vigil]);
        assert_eq!(g.state.pending_triggers.len(), 2);
        assert!(
            g.state
                .pending_triggers
                .iter()
                .all(|t| [land, vigil].contains(&t.object.card))
        );
        assert!(g.object_view(land).tapped);
        settle(&mut g);
        assert_eq!(g.state.players[0].life, 21);
        assert_eq!(
            g.state
                .objects
                .iter()
                .filter(|v| v.zone == Zone::Battlefield
                    && v.card.token_info().is_some_and(|t| t.subtypes == "Knight"))
                .count(),
            1
        );
    }
}
#[test]
fn existing_queued_entry_trigger_survives_real_flash_response() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let h = named(&mut g, p, "Helpful Hunter", Zone::Hand);
        replay(&mut g, Action::AnnounceCast { card: h });
        while g.observe(g.actor().unwrap()).announcing.is_some() {
            choose(&mut g);
        }
        replay(&mut g, Action::Pass);
        replay(&mut g, Action::Pass);
        assert!(g.state.stack.iter().any(|t| t.object.card == h && matches!(t.kind, StackKind::TriggeredAbility { .. })));
        replay(&mut g, Action::Pass);
        let thrull = named(&mut g, p.other(), NAMES[1], Zone::Hand);
        let before = g.state.players[p.index()].library.len();
        cast(&mut g, thrull);
        assert_eq!(g.state.players[p.index()].library.len(), before - 1);
    }
}
#[test]
fn removal_reentry_and_clone_preserve_only_live_source_incarnations() {
    for n in NAMES {
        let mut g = setup(Player::P0);
        let source = named(&mut g, Player::P1, n, Zone::Battlefield);
        let old = g.object_ref(source);
        let saved = g.snapshot();
        let mut fork = g.clone();
        let events = fork.move_objects_raw(&[source], Zone::Exile);
        fork.finish_zone_events(&events, &[]);
        assert!(!fork.state.static_sources.contains(&source));
        let h = named(&mut fork, Player::P0, "Helpful Hunter", Zone::Hand);
        enter(&mut fork, &[h]);
        assert_eq!(fork.state.pending_triggers.len(), 1);
        fork.state.pending_triggers.clear();
        enter(&mut fork, &[source]);
        assert_ne!(fork.object_ref(source), old);
        assert_eq!(
            fork.state
                .static_sources
                .iter()
                .filter(|&&id| id == source)
                .count(),
            1
        );
        let h2 = named(&mut fork, Player::P0, "Helpful Hunter", Zone::Hand);
        enter(&mut fork, &[h2]);
        assert!(fork.state.pending_triggers.is_empty());
        assert_eq!(g.snapshot(), saved);
        let h3 = named(&mut g, Player::P0, "Helpful Hunter", Zone::Hand);
        enter(&mut g, &[h3]);
        assert!(g.state.pending_triggers.is_empty());
        g.restore(&saved).unwrap();
        assert_eq!(g.snapshot(), saved);
    }
}
#[test]
fn suppression_does_not_change_entry_counters_or_tapped_replacements() {
    for n in NAMES {
        let mut g = setup(Player::P0);
        named(&mut g, Player::P1, n, Zone::Battlefield);
        let giant = named(&mut g, Player::P0, "Burdened Stoneback", Zone::Hand);
        cast(&mut g, giant);
        let v = g.object_view(giant);
        assert_eq!(v.counters.minus_one, 2);
        assert_eq!((v.power, v.toughness), (2, 2));
        let ghoul = named(&mut g, Player::P0, "Diregraf Ghoul", Zone::Hand);
        cast(&mut g, ghoul);
        assert!(g.object_view(ghoul).tapped);
    }
}
#[test]
fn death_half_and_other_death_triggers_still_fire_under_suppression() {
    for n in NAMES {
        let mut g = setup(Player::P0);
        named(&mut g, Player::P1, n, Zone::Battlefield);
        let d = named(&mut g, Player::P0, "Mephitic Draught", Zone::Battlefield);
        let f = named(&mut g, Player::P0, "Fear of Lost Teeth", Zone::Battlefield);
        let observers = g.death_trigger_sources();
        let events = g.move_objects_raw(&[d, f], Zone::Graveyard);
        g.finish_zone_events(&events, &observers);
        assert_eq!(g.state.pending_triggers.len(), 2);
        let before = g.state.players[0].library.len();
        settle(&mut g);
        assert_eq!(g.state.players[0].library.len(), before - 1);
        assert!(g.state.stack.is_empty());
    }
}
#[test]
fn suppressed_entry_half_does_not_disable_attack_or_step_triggers() {
    for n in NAMES {
        let mut g = setup(Player::P0);
        named(&mut g, Player::P1, n, Zone::Battlefield);
        let bird = named(&mut g, Player::P0, "Dreadwing Scavenger", Zone::Hand);
        enter(&mut g, &[bird]);
        assert!(g.state.pending_triggers.is_empty());
        g.state.objects[bird as usize].summoning_sick = false;
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        let before = g.state.players[0].library.len();
        replay(&mut g, Action::Attack { card: bird });
        replay(&mut g, Action::FinishAttackers);
        settle(&mut g);
        assert_eq!(g.state.players[0].library.len(), before - 1);
        let giant = named(&mut g, Player::P0, "Ravenous Giant", Zone::Battlefield);
        g.begin_step(Step::Upkeep);
        settle(&mut g);
        assert_eq!(g.state.players[0].life, 19);
        assert_eq!(g.object_view(giant).damage, 0);
    }
}
#[test]
fn suppressors_outside_battlefield_do_not_apply() {
    for zone in [Zone::Hand, Zone::Graveyard, Zone::Exile, Zone::Library] {
        let mut g = setup(Player::P0);
        for n in NAMES {
            named(&mut g, Player::P1, n, zone);
        }
        let hunter = named(&mut g, Player::P0, "Helpful Hunter", Zone::Hand);
        enter(&mut g, &[hunter]);
        assert_eq!(g.state.pending_triggers.len(), 1);
        assert!(g.state.static_sources.is_empty());
    }
}
#[test]
fn creature_and_artifact_token_batches_obey_distinct_suppression_masks() {
    let treasure = Card::catalogue()
        .find(|c| c.token_info().is_some_and(|t| t.subtypes == "Treasure"))
        .unwrap();
    let knight = Card::catalogue()
        .find(|c| c.token_info().is_some_and(|t| t.subtypes == "Knight"))
        .unwrap();
    for (i, n) in NAMES.iter().enumerate() {
        let mut g = setup(Player::P0);
        named(&mut g, Player::P1, n, Zone::Battlefield);
        named(&mut g, Player::P0, "Pactdoll Terror", Zone::Battlefield);
        named(&mut g, Player::P0, "EPF Point Squad", Zone::Battlefield);
        g.create_tokens(Player::P0, treasure, 2, false);
        assert_eq!(g.state.pending_triggers.len(), if i == 0 { 2 } else { 0 });
        g.state.pending_triggers.clear();
        g.create_tokens(Player::P0, knight, 2, false);
        assert!(g.state.pending_triggers.is_empty());
    }
}
#[test]
fn blocked_trigger_does_not_allocate_stack_identity_and_unblocked_overflow_is_atomic() {
    for blocked in [false, true] {
        let mut g = setup(Player::P0);
        if blocked {
            named(&mut g, Player::P1, NAMES[0], Zone::Battlefield);
        }
        let h = named(&mut g, Player::P0, "Helpful Hunter", Zone::Hand);
        replay(&mut g, Action::AnnounceCast { card: h });
        while g.observe(g.actor().unwrap()).announcing.is_some() {
            choose(&mut g);
        }
        replay(&mut g, Action::Pass);
        g.state.next_stack_id = u64::MAX;
        let snapshot = g.snapshot();
        let revision = g.revision();
        let result = g.apply(g.actor().unwrap(), Action::Pass);
        if blocked {
            assert!(result.is_ok());
            assert_eq!(g.object_view(h).zone, Zone::Battlefield);
            assert!(g.state.stack.is_empty());
        } else {
            assert!(matches!(result, Err(Error::CapacityExceeded(_))));
            assert_eq!(g.snapshot(), snapshot);
            assert_eq!(g.revision(), revision);
        }
    }
}

#[test]
fn blocked_entry_does_not_consume_once_per_turn_use() {
    for n in NAMES {
        let mut g = setup(Player::P0);
        let source = named(&mut g, Player::P1, n, Zone::Battlefield);
        let observer = named(&mut g, Player::P0, "Mechan Assembler", Zone::Battlefield);
        let first = named(&mut g, Player::P0, "Three Tree Mascot", Zone::Hand);
        enter(&mut g, &[first]);
        assert!(g.state.pending_triggers.is_empty());
        assert!(g.state.triggered_this_turn.is_empty());
        let events = g.move_objects_raw(&[source], Zone::Exile);
        g.finish_zone_events(&events, &[]);
        let second = named(&mut g, Player::P0, "Three Tree Mascot", Zone::Hand);
        enter(&mut g, &[second]);
        assert_eq!(g.state.pending_triggers.len(), 1);
        assert_eq!(g.state.pending_triggers[0].object.card, observer);
        assert_eq!(g.state.triggered_this_turn.len(), 1);
        let third = named(&mut g, Player::P0, "Three Tree Mascot", Zone::Hand);
        enter(&mut g, &[third]);
        assert_eq!(g.state.pending_triggers.len(), 1);
    }
}
#[test]
fn cast_triggers_and_other_indexed_continuous_effects_are_preserved() {
    for n in NAMES {
        let mut g = setup(Player::P0);
        named(&mut g, Player::P1, n, Zone::Battlefield);
        let agent = named(&mut g, Player::P0, "Agent of Atlas", Zone::Battlefield);
        let base = g.object_view(agent);
        let draught = named(&mut g, Player::P0, "Mephitic Draught", Zone::Hand);
        cast(&mut g, draught);
        let boosted = g.object_view(agent);
        assert_eq!(
            (boosted.power, boosted.toughness),
            (base.power + 1, base.toughness + 1)
        );
        named(
            &mut g,
            Player::P0,
            "Arahbo, the First Fang",
            Zone::Battlefield,
        );
        let hunter = named(&mut g, Player::P0, "Helpful Hunter", Zone::Hand);
        let before = g.state.players[0].library.len();
        cast(&mut g, hunter);
        assert_eq!(g.state.players[0].library.len(), before);
        let cat = g.object_view(hunter);
        assert_eq!((cat.power, cat.toughness), (2, 2));
        assert_eq!(g.state.static_sources.len(), 2);
    }
}
