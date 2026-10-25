//! Paid Plan producers, committed graveyard batches and bounded immediate free casts.
use super::assurance::big_helpers::{cast, checked, named, settle, setup};
use super::*;
const ROBOT: &str = "Robot Domination";
const DOOM: &str = "Doom Reigns Supreme";
fn views(g: &Game) -> [serde_json::Value; 2] {
    [Player::P0, Player::P1].map(|p| {
        let mut v = g.observe(p);
        v.revision = 0;
        serde_json::to_value(v).unwrap()
    })
}
fn reject(g: &mut Game, a: Action) {
    let old = (g.snapshot(), g.revision(), g.state_bytes(), views(g));
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!((g.snapshot(), g.revision(), g.state_bytes(), views(g)), old);
}
fn orphan(g: Game, a: Action) -> Game {
    let actor = g.actor().unwrap();
    let rev = g.revision();
    let ix = g.legal_actions().iter().position(|v| *v == a).unwrap();
    let before = g.snapshot();
    let v = views(&g);
    let (mut raw, mut indexed) = (g.clone(), g.clone());
    drop(g);
    raw.apply(actor, a.clone()).unwrap();
    indexed.apply_index(actor, rev, ix).unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(views(&raw), views(&indexed));
    let after = raw.snapshot();
    raw.restore(&before).unwrap();
    assert_eq!(views(&raw), v);
    assert_eq!(raw.apply_index(actor, rev, ix), Err(Error::StaleRevision));
    raw.apply(actor, a).unwrap();
    assert_eq!(raw.snapshot(), after);
    indexed
}
fn step(g: &mut Game) {
    let aa = g.legal_actions();
    let a = aa
        .iter()
        .find(|a| {
            matches!(
                a,
                Action::Pass | Action::FinishTargets | Action::PayCosts { .. }
            )
        })
        .or_else(|| aa.iter().find(|a| !matches!(a, Action::Concede)))
        .unwrap()
        .clone();
    checked(g, a);
}
fn until(g: &mut Game, p: impl Fn(&Game) -> bool) {
    g.prepare_triggers();
    for _ in 0..180 {
        if p(g) {
            return;
        }
        assert!(g.outcome().is_none());
        step(g);
    }
    panic!("plan continuation stalled {:?}", g.state.pending);
}
fn main(g: &mut Game, p: Player) {
    g.state.active = p;
    g.state.priority = p;
    g.state.step = Step::Main1;
    g.state.pending = Pending::Priority;
    g.state.passes = 0;
}
fn paid(g: &mut Game, p: Player, name: &str, targets: &[Target]) -> CardId {
    main(g, p);
    let d = Card::from_name(name).unwrap();
    assert!(d.is_supported());
    let id = named(g, p, name, Zone::Hand);
    let (mut m, n) = d.cost();
    m[5] += n;
    g.state.players[p.index()].mana = m;
    cast(g, id, None, targets);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    id
}
fn producer(g: &mut Game, p: Player, name: &str) -> CardId {
    let id = paid(g, p, name, &[]);
    settle(g);
    id
}
fn batch(g: &mut Game, ids: &[CardId], to: Zone) {
    let before = g.death_trigger_sources();
    let ev = g.move_objects_raw(ids, to);
    g.finish_zone_events(&ev, &before);
}
fn triggers(g: &Game, s: CardId, ability: u16) -> usize {
    g.state
        .pending_triggers
        .iter()
        .chain(&g.state.ordering_triggers)
        .chain(&g.state.stack)
        .filter(|v| v.card == s && v.kind == StackKind::TriggeredAbility { ability })
        .count()
}
fn tokens(g: &Game, p: Player) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(i, o)| {
            (o.zone == Zone::Battlefield
                && o.controller == p
                && o.is_token()
                && o.card == Card::from_name("@token/msh-robot-villain").unwrap())
            .then_some(i as CardId)
        })
        .collect()
}
fn crossing(g: &mut Game, s: CardId, n: u64) {
    g.effect_counter(s, CounterKind::Plan, n, replacements::ResumeMode::Pending);
}
fn child(g: &Game, s: CardId) -> bool {
    g.state.announcing.as_ref().is_some_and(|a| {
        a.item.card == s && a.item.kind == StackKind::TriggeredAbility { ability: 2 }
    })
}
fn offer(g: &mut Game, p: Player, s: CardId) {
    crossing(g, s, 5);
    until(g, |g| child(g, s));
    assert_eq!(g.object_view(s).zone, Zone::Graveyard);
    checked(
        g,
        Action::ChooseTarget {
            target: Target::Player(p.other()),
        },
    );
    until(g, |g| {
        matches!(g.state.pending, Pending::ResolutionCasts { .. })
    });
}
fn quota(g: &Game) -> u16 {
    g.state
        .resolution_free_casts
        .as_ref()
        .unwrap()
        .casts_remaining
        .unwrap()
}
fn action(g: &Game, id: CardId) -> Action {
    Action::CastExiledDuringResolution {
        object: g.object_ref(id),
        face: CastFace::Normal,
    }
}
fn commit(g: &mut Game, id: CardId, target: Option<Target>) {
    let selected = action(g, id);
    checked(g, selected);
    if let Some(target) = target {
        checked(g, Action::ChooseTarget { target });
    }
    until(g, |g| g.state.announcing.is_none());
    assert!(matches!(g.state.pending, Pending::ResolutionCasts { .. }));
}
fn clear_library(g: &mut Game, p: Player) {
    let ids = g.state.players[p.index()].library.clone();
    batch(g, &ids, Zone::Outside);
}

#[test]
fn msh_graveyard_plans_robot_paid_three_real_removals_draw_life_threshold_and_recipe() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let s = producer(&mut g, p, ROBOT);
        let h = g.hand_count(p);
        for n in 1..=3 {
            let b = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            paid(&mut g, p.other(), "Murder", &[Target::Card(b)]);
            settle(&mut g);
            assert_eq!(g.hand_count(p), h + n);
            assert_eq!(g.state.players[p.index()].life, 20 - n as i128);
            if n < 3 {
                assert_eq!(g.counter_count(s, CounterKind::Plan), n as u64);
            }
        }
        assert_eq!(g.object_view(s).zone, Zone::Graveyard);
        let ts = tokens(&g, p);
        assert_eq!(ts.len(), 3);
        for id in ts {
            let v = g.object_view(id);
            assert_eq!((v.power, v.toughness, v.kinds, v.colors), (2, 2, 17, 0));
            assert!(v.has_subtype("Robot") && v.has_subtype("Villain"));
            assert!(!v.tapped);
            assert!(v.summoning_sick);
        }
    }
}
#[test]
fn msh_graveyard_plans_robot_from_anywhere_is_one_trigger_per_committed_batch() {
    for p in [Player::P0, Player::P1] {
        for zone in [
            Zone::Hand,
            Zone::Library,
            Zone::Battlefield,
            Zone::Exile,
            Zone::Stack,
        ] {
            let mut g = setup(p);
            let s = producer(&mut g, p, ROBOT);
            let a = named(&mut g, p, "Bear Cub", zone);
            let b = named(&mut g, p, "Hill Giant", zone);
            let h = g.hand_count(p);
            batch(&mut g, &[a, b], Zone::Graveyard);
            assert_eq!(triggers(&g, s, 0), 1);
            settle(&mut g);
            assert_eq!(g.hand_count(p), h + 1 - usize::from(zone == Zone::Hand) * 2);
            assert_eq!(g.counter_count(s, CounterKind::Plan), 1);
            assert_eq!(g.state.players[p.index()].life, 19);
        }
    }
}
#[test]
fn msh_graveyard_plans_robot_uses_owner_final_card_type_not_dying_creature() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let s = producer(&mut g, p, ROBOT);
        let land = named(&mut g, p, "Forest", Zone::Battlefield);
        let object = g.object_ref(land);
        let timestamp = g.next_role_timestamp();
        g.state.roles.get_or_insert_default().animations.push(
            characteristic_layers::AnimationView {
                object,
                timestamp,
                artifact: false,
                preserve_card_types: false,
                colors: None,
                creature_types: None,
                all_creature_types: false,
                base: Some((3, 3)),
                keywords: Keywords::NONE,
                expires_turn: None,
            },
        );
        assert!(g.is_creature(land));
        batch(&mut g, &[land], Zone::Graveyard);
        assert_eq!(triggers(&g, s, 0), 0);
        let b = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let moon = named(
            &mut g,
            p.other(),
            "Imprisoned in the Moon",
            Zone::Battlefield,
        );
        g.state.objects[moon as usize].attached_to = Some(g.object_ref(b));
        assert!(!g.is_creature(b));
        batch(&mut g, &[b], Zone::Graveyard);
        assert_eq!(triggers(&g, s, 0), 1);
        settle(&mut g);
        assert_eq!(g.counter_count(s, CounterKind::Plan), 1);
        let foreign = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.change_controller(g.object_ref(foreign), p);
        batch(&mut g, &[foreign], Zone::Graveyard);
        assert_eq!(triggers(&g, s, 0), 0);
    }
}
#[test]
fn msh_graveyard_plans_robot_tokens_noncreatures_and_unchanged_graveyard_do_not_trigger() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let s = producer(&mut g, p, ROBOT);
        let t = named(&mut g, p, "@token/msh-robot-villain", Zone::Battlefield);
        let land = named(&mut g, p, "Forest", Zone::Hand);
        let old = named(&mut g, p, "Bear Cub", Zone::Graveyard);
        let prior = g.object_ref(old);
        batch(&mut g, &[t, land], Zone::Graveyard);
        assert_eq!(g.object_ref(old), prior);
        assert_eq!(triggers(&g, s, 0), 0);
        assert_eq!(g.counter_count(s, CounterKind::Plan), 0);
    }
}
#[test]
fn msh_graveyard_plans_robot_postevent_observer_does_not_see_its_simultaneous_departure() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let s = producer(&mut g, p, ROBOT);
        let b = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let h = g.hand_count(p);
        batch(&mut g, &[s, b], Zone::Graveyard);
        assert_eq!(triggers(&g, s, 0), 0);
        settle(&mut g);
        assert_eq!(g.hand_count(p), h);
        assert_eq!(g.state.players[p.index()].life, 20);
    }
}
#[test]
fn msh_graveyard_plans_robot_actual_replacement_destination_is_not_graveyard() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let s = producer(&mut g, p, ROBOT);
        named(&mut g, p, "Rest in Peace", Zone::Battlefield);
        let b = named(&mut g, p, "Bear Cub", Zone::Hand);
        batch(&mut g, &[b], Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.object_view(b).zone, Zone::Exile);
        assert_eq!(g.counter_count(s, CounterKind::Plan), 0);
    }
}
#[test]
fn msh_graveyard_plans_robot_payoff_creates_even_if_sacrifice_fails() {
    for p in [Player::P0, Player::P1] {
        for mode in 0..3 {
            let mut g = setup(p);
            let s = producer(&mut g, p, ROBOT);
            crossing(&mut g, s, 3);
            until(&mut g, |g| {
                g.state
                    .stack
                    .iter()
                    .any(|a| a.card == s && a.kind == StackKind::TriggeredAbility { ability: 1 })
            });
            let mut fresh = None;
            match mode {
                0 => {
                    g.move_object(s, Zone::Exile);
                }
                1 => {
                    g.change_controller(g.object_ref(s), p.other());
                }
                _ => {
                    g.move_object(s, Zone::Exile);
                    g.move_object(s, Zone::Battlefield);
                    fresh = Some(g.object_ref(s));
                }
            }
            settle(&mut g);
            assert_eq!(tokens(&g, p).len(), 3);
            assert_eq!(tokens(&g, p.other()).len(), 0);
            if let Some(fresh) = fresh {
                assert_eq!(g.object_ref(s), fresh);
                assert!(g.on_battlefield(s));
            }
        }
    }
}
#[test]
fn msh_graveyard_plans_robot_counter_and_token_replacements_cross_historically() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let s = producer(&mut g, p, ROBOT);
        named(&mut g, p, "Doubling Season", Zone::Battlefield);
        for n in 1..=2 {
            let b = named(&mut g, p, "Bear Cub", Zone::Hand);
            batch(&mut g, &[b], Zone::Graveyard);
            settle(&mut g);
            if n == 1 {
                assert_eq!(g.counter_count(s, CounterKind::Plan), 2);
            }
        }
        assert_eq!(g.object_view(s).zone, Zone::Graveyard);
        assert_eq!(tokens(&g, p).len(), 6);
    }
}
#[test]
fn msh_graveyard_plans_robot_draw_pause_preserves_later_life_and_counter_order() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let s = producer(&mut g, p, ROBOT);
        named(&mut g, p, "Lorehold, the Historian", Zone::Battlefield);
        let b = named(&mut g, p, "Bear Cub", Zone::Hand);
        batch(&mut g, &[b], Zone::Graveyard);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::MiracleReveal { .. })
        });
        assert_eq!(g.state.players[p.index()].life, 20);
        assert_eq!(g.counter_count(s, CounterKind::Plan), 0);
        assert!(g.observe(p.other()).miracle.is_none());
        g = orphan(g, Action::DeclineMiracle);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, 19);
        assert_eq!(g.counter_count(s, CounterKind::Plan), 1);
    }
}
#[test]
fn msh_graveyard_plans_robot_queued_trigger_uses_captured_player_after_control_change() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let s = producer(&mut g, p, ROBOT);
        let b = named(&mut g, p, "Bear Cub", Zone::Hand);
        batch(&mut g, &[b], Zone::Graveyard);
        let h = [g.hand_count(p), g.hand_count(p.other())];
        g.change_controller(g.object_ref(s), p.other());
        settle(&mut g);
        assert_eq!([g.hand_count(p), g.hand_count(p.other())], [h[0] + 1, h[1]]);
        assert_eq!(g.state.players[p.index()].life, 19);
        assert_eq!(g.state.players[p.other().index()].life, 20);
        assert_eq!(g.counter_count(s, CounterKind::Plan), 1);
    }
}
#[test]
fn msh_graveyard_plans_doom_paid_five_villain_entries_then_separate_opponent_target() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let s = producer(&mut g, p, DOOM);
        for n in 1..=5 {
            paid(&mut g, p, "Aerial Doombot", &[]);
            if n < 5 {
                settle(&mut g);
                assert_eq!(g.counter_count(s, CounterKind::Plan), n);
            } else {
                until(&mut g, |g| child(g, s));
            }
        }
        assert_eq!(g.state.players[p.index()].life, 25);
        assert_eq!(g.state.players[p.other().index()].life, 15);
        assert_eq!(g.object_view(s).zone, Zone::Graveyard);
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p),
            },
        );
        g = orphan(
            g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::ResolutionCasts { .. })
        });
        assert_eq!(quota(&g), 2);
        checked(&mut g, Action::FinishResolutionCasts);
        settle(&mut g);
    }
}
#[test]
fn msh_graveyard_plans_doom_each_entry_not_batch_nonvillain_and_other_controller_excluded() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let s = producer(&mut g, p, DOOM);
        let aa = [
            named(&mut g, p, "Aerial Doombot", Zone::Outside),
            named(&mut g, p, "Aerial Doombot", Zone::Outside),
            named(&mut g, p.other(), "Aerial Doombot", Zone::Outside),
            named(&mut g, p, "Bear Cub", Zone::Outside),
        ];
        batch(&mut g, &aa, Zone::Battlefield);
        settle(&mut g);
        assert_eq!(g.counter_count(s, CounterKind::Plan), 2);
        assert_eq!(g.state.players[p.index()].life, 22);
        assert_eq!(g.state.players[p.other().index()].life, 18);
    }
}
#[test]
fn msh_graveyard_plans_doom_fifth_crossing_failed_sacrifice_has_no_child() {
    for p in [Player::P0, Player::P1] {
        for blink in [false, true] {
            let mut g = setup(p);
            let s = producer(&mut g, p, DOOM);
            crossing(&mut g, s, 5);
            until(&mut g, |g| {
                g.state
                    .stack
                    .iter()
                    .any(|a| a.card == s && a.kind == StackKind::TriggeredAbility { ability: 1 })
            });
            if blink {
                g.move_object(s, Zone::Exile);
                g.move_object(s, Zone::Battlefield);
            } else {
                g.change_controller(g.object_ref(s), p.other());
            }
            settle(&mut g);
            assert!(g.state.resolution_free_casts.is_none());
            assert!(g.on_battlefield(s));
            assert_eq!(triggers(&g, s, 2), 0);
        }
    }
}
#[test]
fn msh_graveyard_plans_doom_zero_one_two_free_casts_exact_quota_owner_and_order() {
    for p in [Player::P0, Player::P1] {
        for amount in 0..=2 {
            let mut g = setup(p);
            let s = producer(&mut g, p, DOOM);
            clear_library(&mut g, p.other());
            let a = named(&mut g, p.other(), "Bear Cub", Zone::Library);
            let b = named(&mut g, p.other(), "Gilded Lotus", Zone::Library);
            let c = named(&mut g, p.other(), "Hill Giant", Zone::Library);
            offer(&mut g, p, s);
            let mana = g.state.players[p.index()].mana;
            for viewer in [p, p.other()] {
                assert_eq!(
                    g.observe(viewer)
                        .resolution_free_casts
                        .unwrap()
                        .casts_remaining,
                    Some(2)
                );
            }
            if amount >= 1 {
                let selected = action(&g, a);
                g = orphan(g, selected);
                until(&mut g, |g| g.state.announcing.is_none());
                assert_eq!(quota(&g), 1);
            }
            if amount == 2 {
                commit(&mut g, b, None);
                assert_eq!(quota(&g), 0);
                let selected = action(&g, c);
                reject(&mut g, selected);
            }
            checked(&mut g, Action::FinishResolutionCasts);
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].mana, mana);
            assert_eq!(g.on_battlefield(a), amount >= 1);
            assert_eq!(g.on_battlefield(b), amount == 2);
            if amount >= 1 {
                assert_eq!(g.object_view(a).controller, p);
                assert_eq!(g.object_view(a).owner, p.other());
            }
            assert_eq!(g.object_view(c).zone, Zone::Exile);
        }
    }
}
#[test]
fn msh_graveyard_plans_doom_top_five_short_library_and_no_lands_or_later_permission() {
    for p in [Player::P0, Player::P1] {
        for len in [0usize, 2, 7] {
            let mut g = setup(p);
            let s = producer(&mut g, p, DOOM);
            clear_library(&mut g, p.other());
            let ids: Vec<_> = (0..len)
                .map(|_| named(&mut g, p.other(), "Forest", Zone::Library))
                .collect();
            offer(&mut g, p, s);
            assert_eq!(
                g.state
                    .resolution_free_casts
                    .as_ref()
                    .unwrap()
                    .remaining
                    .len(),
                len.min(5)
            );
            assert_eq!(
                g.state.players[p.other().index()].library.len(),
                len.saturating_sub(5)
            );
            for id in ids.iter().rev().take(5) {
                let selected = action(&g, *id);
                reject(&mut g, selected);
                reject(&mut g, Action::PlayLand { card: *id });
            }
            checked(&mut g, Action::FinishResolutionCasts);
            settle(&mut g);
            for id in ids {
                if g.object_view(id).zone == Zone::Exile {
                    assert!(!g.can_play_exiled(p, id));
                }
            }
        }
    }
}
#[test]
fn msh_graveyard_plans_doom_exact_cast_object_target_failures_and_success_only_count() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let s = producer(&mut g, p, DOOM);
        clear_library(&mut g, p.other());
        let a = named(&mut g, p.other(), "Shock", Zone::Library);
        offer(&mut g, p, s);
        let old = g.object_ref(a);
        let selected = action(&g, a);
        checked(&mut g, selected);
        assert_eq!(quota(&g), 2);
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(s),
            },
        );
        assert_eq!(quota(&g), 2);
        g = orphan(
            g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        until(&mut g, |g| g.state.announcing.is_none());
        assert_eq!(quota(&g), 1);
        assert!(
            !serde_json::to_string(&g.observe(p))
                .unwrap()
                .contains("pending_cast")
        );
        reject(
            &mut g,
            Action::CastExiledDuringResolution {
                object: old,
                face: CastFace::Normal,
            },
        );
        checked(&mut g, Action::FinishResolutionCasts);
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 18);
    }
}
#[test]
fn msh_graveyard_plans_doom_free_cast_preserves_mandatory_additional_cost() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let s = producer(&mut g, p, DOOM);
        clear_library(&mut g, p.other());
        let t = named(&mut g, p.other(), "Titania, Rugged Rumbler", Zone::Library);
        offer(&mut g, p, s);
        g.state.players[p.index()].mana = [0; 6];
        let selected = action(&g, t);
        reject(&mut g, selected);
        assert_eq!(quota(&g), 2);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        commit(&mut g, t, None);
        assert_eq!(quota(&g), 1);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        checked(&mut g, Action::FinishResolutionCasts);
        settle(&mut g);
        assert!(g.on_battlefield(t));
    }
}
#[test]
fn msh_graveyard_plans_doom_free_x_is_zero_without_an_alternative_cost() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let s = producer(&mut g, p, DOOM);
        clear_library(&mut g, p.other());
        let x = named(&mut g, p.other(), "Wildwood Scourge", Zone::Library);
        offer(&mut g, p, s);
        let selected = action(&g, x);
        checked(&mut g, selected);
        assert_eq!(g.state.announcing.as_ref().unwrap().item.x, Some(0));
        until(&mut g, |g| g.state.announcing.is_none());
        assert_eq!(quota(&g), 1);
        checked(&mut g, Action::FinishResolutionCasts);
        settle(&mut g);
        assert_eq!(g.object_view(x).zone, Zone::Graveyard);
    }
}
#[test]
fn msh_graveyard_plans_doom_unknown_top_is_private_until_exact_exile_and_old_refs_reject() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let s = producer(&mut g, p, DOOM);
        let a = named(&mut g, p.other(), "Bear Cub", Zone::Library);
        let mut alt = g.clone();
        alt.state.objects[a as usize].card = Card::from_name("Gilded Lotus").unwrap();
        assert_eq!(views(&g), views(&alt));
        assert_eq!(g.legal_actions(), alt.legal_actions());
        offer(&mut g, p, s);
        assert!(g.observe(p).objects.iter().any(|v| v.reference.card == a));
        assert!(
            g.observe(p.other())
                .objects
                .iter()
                .any(|v| v.reference.card == a)
        );
        let old = g.object_ref(a);
        g.move_object(a, Zone::Hand);
        g.move_object(a, Zone::Exile);
        reject(
            &mut g,
            Action::CastExiledDuringResolution {
                object: old,
                face: CastFace::Normal,
            },
        );
        let selected = action(&g, a);
        reject(&mut g, selected);
        assert_eq!(quota(&g), 2);
    }
}
#[test]
fn msh_graveyard_plans_doom_opponent_target_becoming_illegal_exiles_nothing() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let s = producer(&mut g, p, DOOM);
        crossing(&mut g, s, 5);
        until(&mut g, |g| child(g, s));
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        until(&mut g, |g| {
            g.state
                .stack
                .iter()
                .any(|a| a.card == s && a.kind == StackKind::TriggeredAbility { ability: 2 })
        });
        let n = g.state.players[p.other().index()].library.len();
        named(&mut g, p.other(), "Crystal Barricade", Zone::Battlefield);
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].library.len(), n);
        assert!(g.state.resolution_free_casts.is_none());
    }
}

#[test]
fn msh_graveyard_plans_robot_actual_paid_mill_batch_triggers_once_before_recovery() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let s = producer(&mut g, p, ROBOT);
        named(&mut g, p, "Bear Cub", Zone::Library);
        named(&mut g, p, "Hill Giant", Zone::Library);
        paid(&mut g, p, "Rapid Rescue", &[]);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::RecoverMilled { .. })
        });
        assert_eq!(triggers(&g, s, 0), 1);
        assert_eq!(g.counter_count(s, CounterKind::Plan), 0);
        g = orphan(g, Action::RecoverMilled { object: None });
        settle(&mut g);
        assert_eq!(g.counter_count(s, CounterKind::Plan), 1);
        assert_eq!(g.state.players[p.index()].life, 21);
    }
}
#[test]
fn msh_graveyard_plans_doom_noncreature_kindred_villain_entry_is_included() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let s = producer(&mut g, p, DOOM);
        // Only immutable Kindred metadata is used; no gated Boggart ability is cast.
        let id = named(&mut g, p, "Bear Cub", Zone::Outside);
        g.state.objects[id as usize].card = Card::from_name("Boggart Mischief").unwrap();
        let ev = g.move_objects_raw(&[id], Zone::Battlefield);
        let recipient = g.object_ref(id);
        let timestamp = g.next_role_timestamp();
        g.state.roles.get_or_insert_default().creature_types.push(
            subtypes::CreatureTypesEffectView {
                recipient,
                source: recipient,
                types: &["Villain"],
                additional: true,
                timestamp,
            },
        );
        assert_eq!(g.current_kinds(id) & 1, 0);
        assert_ne!(g.current_kinds(id) & 128, 0);
        assert!(g.has_subtype(id, "Villain"));
        g.finish_zone_events(&ev, &[]);
        settle(&mut g);
        assert_eq!(g.counter_count(s, CounterKind::Plan), 1);
        assert_eq!(g.state.players[p.index()].life, 21);
        assert_eq!(g.state.players[p.other().index()].life, 19);
    }
}
#[test]
fn msh_graveyard_plans_doom_copy_is_not_a_cast_and_probes_do_not_spend_quota() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let s = producer(&mut g, p, DOOM);
        clear_library(&mut g, p.other());
        let a = named(&mut g, p.other(), "Bear Cub", Zone::Library);
        offer(&mut g, p, s);
        let before = g.snapshot();
        let expected = g.legal_actions();
        assert_eq!(g.legal_actions(), expected);
        assert_eq!(g.snapshot(), before);
        assert_eq!(quota(&g), 2);
        commit(&mut g, a, None);
        let casts = g.state.players[p.index()].spells_cast_this_turn;
        let target = g.state.stack.last().unwrap().clone();
        let mut item = target.clone();
        item.targets = vec![BoundTarget {
            target: Target::Stack(target.id),
            object: Some(target.object),
            group: 0,
        }];
        // Bounded central copy executor; no additional cast or fake spell payment.
        g.copy_target_spell(&item, 0, false, crate::cards::CopyExceptions::NONE);
        assert!(g.state.stack.last().unwrap().is_copy);
        assert_eq!(quota(&g), 1);
        assert_eq!(g.state.players[p.index()].spells_cast_this_turn, casts);
        checked(&mut g, Action::FinishResolutionCasts);
        settle(&mut g);
    }
}
#[test]
fn msh_graveyard_plans_doom_old_child_keeps_caster_after_source_reentry_and_control_change() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let s = producer(&mut g, p, DOOM);
        crossing(&mut g, s, 5);
        until(&mut g, |g| child(g, s));
        g.move_object(s, Zone::Battlefield);
        g.change_controller(g.object_ref(s), p.other());
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::ResolutionCasts { .. })
        });
        assert_eq!(g.state.resolution_free_casts.as_ref().unwrap().player, p);
        assert_eq!(g.actor(), Some(p));
        g = orphan(g, Action::FinishResolutionCasts);
        settle(&mut g);
        assert!(g.on_battlefield(s));
        assert_eq!(g.object_view(s).controller, p.other());
    }
}
