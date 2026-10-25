//! Connive replacement scenarios. Earlier-layer fixtures and copied packets
//! are bounded production-state probes, not additional card admissions.
use super::assurance::big_helpers::{cast, checked, named, settle, setup};
use super::*;
const LEADER: &str = "Leader, Super-Genius";
fn views(g: &Game) -> [serde_json::Value; 2] {
    [Player::P0, Player::P1].map(|p| {
        let mut v = g.observe(p);
        v.revision = 0;
        serde_json::to_value(v).unwrap()
    })
}
fn reject(g: &mut Game, a: Action) {
    let before = (g.snapshot(), g.revision(), g.state_bytes(), views(g));
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(
        (g.snapshot(), g.revision(), g.state_bytes(), views(g)),
        before
    );
}
fn orphan(g: Game, a: Action) -> Game {
    let actor = g.actor().unwrap();
    let revision = g.revision();
    let index = g.legal_actions().iter().position(|v| *v == a).unwrap();
    let old = g.snapshot();
    let oldviews = views(&g);
    let (mut raw, mut indexed) = (g.clone(), g.clone());
    drop(g);
    raw.apply(actor, a.clone()).unwrap();
    indexed.apply_index(actor, revision, index).unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(views(&raw), views(&indexed));
    let after = raw.snapshot();
    raw.restore(&old).unwrap();
    assert_eq!(views(&raw), oldviews);
    assert_eq!(
        raw.apply_index(actor, revision, index),
        Err(Error::StaleRevision)
    );
    raw.apply(actor, a).unwrap();
    assert_eq!(raw.snapshot(), after);
    indexed
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
    let c = Card::from_name(name).unwrap();
    assert!(c.is_supported());
    let id = named(g, p, name, Zone::Hand);
    let (mut m, n) = c.cost();
    m[5] += n;
    g.state.players[p.index()].mana = m;
    cast(g, id, None, targets);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    id
}
fn leader(g: &mut Game, p: Player) -> CardId {
    let id = paid(g, p, LEADER, &[]);
    settle(g);
    id
}
fn until(g: &mut Game, stop: impl Fn(&Game) -> bool) {
    g.prepare_triggers();
    for _ in 0..180 {
        if stop(g) {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::Pass
                        | Action::FinishTargets
                        | Action::PayCosts { .. }
                        | Action::PutTrigger { .. }
                )
            })
            .unwrap_or_else(|| panic!("stalled {:?}", g.state.pending));
        checked(g, a);
    }
    panic!("connive stalled {:?}", g.state.pending)
}
fn combat(g: &mut Game, p: Player, target: CardId) {
    main(g, p);
    g.begin_step(Step::BeginCombat);
    until(g, |g| matches!(g.state.pending, Pending::Targets { .. }));
    checked(
        g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
}
fn discard(g: &mut Game, p: Player, id: CardId) {
    until(g, |g| {
        matches!(g.state.pending, Pending::EffectDiscard { .. })
    });
    assert_eq!(g.actor(), Some(p));
    checked(g, Action::Discard { card: id });
    settle(g);
}
fn activate(g: &mut Game, p: Player, id: CardId) {
    main(g, p);
    checked(
        g,
        Action::AnnounceActivate {
            card: id,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    for _ in 0..8 {
        if g.state.announcing.is_none() {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. } | Action::FinishTargets))
            .unwrap();
        checked(g, a);
    }
    panic!("activation not paid")
}
fn moved(g: &mut Game, ids: &[CardId], to: Zone) {
    let before = g.death_trigger_sources();
    let ev = g.move_objects_raw(ids, to);
    g.finish_zone_events(&ev, &before);
}
fn blank(g: &mut Game, p: Player, id: CardId) {
    let a = named(g, p, "Stop Cold", Zone::Battlefield);
    g.state.objects[a as usize].attached_to = Some(g.object_ref(id));
}
fn moon(g: &mut Game, p: Player, id: CardId) {
    let a = named(g, p, "Imprisoned in the Moon", Zone::Battlefield);
    g.state.objects[a as usize].attached_to = Some(g.object_ref(id));
    assert!(!g.is_creature(id));
}
fn entry(g: &mut Game, p: Player) -> (CardId, CardId) {
    let b = paid(g, p, "Baron Strucker, HYDRA Overlord", &[]);
    settle(g);
    let v = paid(g, p, "Aerial Doombot", &[]);
    until(g, |g| g.on_battlefield(v));
    (b, v)
}
fn accept(g: &mut Game) {
    until(g, |g| {
        matches!(g.state.pending, Pending::EffectOptional { .. })
    });
    checked(g, Action::ChooseOptional { accept: true });
}
fn packet(g: &Game, id: CardId) -> &SpellView {
    g.state
        .stack
        .iter()
        .chain(&g.state.pending_triggers)
        .chain(&g.state.ordering_triggers)
        .find(|v| v.card == id)
        .unwrap()
}
#[test]
fn msh_leader_body_cost_combat_self_nonland_and_land_discard() {
    for p in [Player::P0, Player::P1] {
        for nonland in [false, true] {
            let mut g = setup(p);
            let s = leader(&mut g, p);
            let v = g.object_view(s);
            assert_eq!((v.power, v.toughness), (1, 3));
            assert!(v.card.is_legendary());
            for t in ["Gamma", "Scientist", "Villain"] {
                assert!(v.has_subtype(t));
            }
            let d = named(
                &mut g,
                p,
                if nonland { "Shock" } else { "Forest" },
                Zone::Hand,
            );
            let h = g.hand_count(p);
            combat(&mut g, p, s);
            discard(&mut g, p, d);
            assert_eq!(g.hand_count(p), h + 1);
            assert_eq!(g.counter_count(s, CounterKind::PlusOne), u64::from(nonland));
        }
    }
}
#[test]
fn msh_leader_combat_only_own_turn_and_own_creature_target() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let s = leader(&mut g, p);
        main(&mut g, p.other());
        g.begin_step(Step::BeginCombat);
        assert!(!g.state.pending_triggers.iter().any(|x| x.card == s));
        let theirs = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let land = named(&mut g, p, "Forest", Zone::Battlefield);
        main(&mut g, p);
        g.begin_step(Step::BeginCombat);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::Targets { .. })
        });
        for id in [theirs, land] {
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(id),
                },
            );
        }
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(s),
            },
        );
        settle(&mut g);
    }
}
#[test]
fn msh_leader_source_gone_does_not_replace_queued_combat_connive() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let s = leader(&mut g, p);
        let b = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let d = named(&mut g, p, "Shock", Zone::Hand);
        combat(&mut g, p, b);
        let h = g.hand_count(p);
        moved(&mut g, &[s], Zone::Exile);
        discard(&mut g, p, d);
        assert_eq!(g.hand_count(p), h);
        assert_eq!(g.counter_count(b, CounterKind::PlusOne), 1);
    }
}
#[test]
fn msh_leader_target_departure_reentry_never_rebinds() {
    for p in [Player::P0, Player::P1] {
        for reenter in [false, true] {
            let mut g = setup(p);
            leader(&mut g, p);
            let b = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let r = g.object_ref(b);
            combat(&mut g, p, b);
            moved(&mut g, &[b], Zone::Exile);
            if reenter {
                moved(&mut g, &[b], Zone::Battlefield);
                assert_ne!(g.object_ref(b), r);
            }
            let h = g.hand_count(p);
            settle(&mut g);
            assert_eq!(g.hand_count(p), h);
            assert_eq!(g.counter_count(b, CounterKind::PlusOne), 0);
        }
    }
}
#[test]
fn msh_leader_target_control_or_type_change_fizzles() {
    for p in [Player::P0, Player::P1] {
        for kind in [false, true] {
            let mut g = setup(p);
            leader(&mut g, p);
            let b = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            combat(&mut g, p, b);
            if kind {
                moon(&mut g, p, b)
            } else {
                g.change_controller(g.object_ref(b), p.other());
            }
            let h = [g.hand_count(p), g.hand_count(p.other())];
            settle(&mut g);
            assert_eq!([g.hand_count(p), g.hand_count(p.other())], h);
        }
    }
}
#[test]
fn msh_leader_source_connive_live_control_uses_new_controller_and_counter_placer() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        leader(&mut g, p.other());
        let s = paid(&mut g, p, "M.O.D.O.K.", &[]);
        settle(&mut g);
        let d = named(&mut g, p.other(), "Shock", Zone::Hand);
        let h = g.hand_count(p.other());
        activate(&mut g, p, s);
        g.change_controller(g.object_ref(s), p.other());
        discard(&mut g, p.other(), d);
        assert_eq!(g.hand_count(p.other()), h + 1);
        assert!(g.put_counter_on_source_this_turn(g.object_ref(s), p.other()));
        assert!(!g.put_counter_on_source_this_turn(g.object_ref(s), p));
    }
}
#[test]
fn msh_leader_source_connive_latest_departure_kind_controller_and_incarnation() {
    for p in [Player::P0, Player::P1] {
        for noncreature in [false, true] {
            for reenter in [false, true] {
                let mut g = setup(p);
                leader(&mut g, p.other());
                let s = paid(&mut g, p, "M.O.D.O.K.", &[]);
                settle(&mut g);
                let d = named(&mut g, p.other(), "Shock", Zone::Hand);
                activate(&mut g, p, s);
                let old = g.object_ref(s);
                g.change_controller(old, p.other());
                if noncreature {
                    moon(&mut g, p, s);
                }
                moved(&mut g, &[s], Zone::Exile);
                if reenter {
                    moved(&mut g, &[s], Zone::Battlefield);
                    assert_ne!(g.object_ref(s), old);
                }
                let h = g.hand_count(p.other());
                discard(&mut g, p.other(), d);
                assert_eq!(g.hand_count(p.other()), h + usize::from(!noncreature));
                assert_eq!(g.counter_count(s, CounterKind::PlusOne), 0);
            }
        }
    }
}
#[test]
fn msh_leader_entering_conniver_latest_departure_kind_and_copy_payload() {
    for p in [Player::P0, Player::P1] {
        for noncreature in [false, true] {
            let mut g = setup(p);
            leader(&mut g, p.other());
            let (b, v) = entry(&mut g, p);
            let old = g.object_ref(v);
            g.change_controller(old, p.other());
            if noncreature {
                moon(&mut g, p, v);
            }
            let mut copy = packet(&g, b).clone();
            copy.id = g.next_stack_id();
            copy.is_copy = true;
            g.state.pending_triggers.push(copy);
            moved(&mut g, &[v], Zone::Exile);
            let original = packet(&g, b).clone();
            assert_eq!(
                original.event_object.as_ref().unwrap().conniver_creature,
                Some(!noncreature)
            );
            assert!(
                g.state
                    .pending_triggers
                    .iter()
                    .filter(|v| v.card == b)
                    .all(|v| v.event_object.as_ref().unwrap().conniver_creature
                        == Some(!noncreature))
            );
            let d = named(&mut g, p.other(), "Shock", Zone::Hand);
            let h = g.hand_count(p.other());
            accept(&mut g);
            discard(&mut g, p.other(), d);
            assert_eq!(g.hand_count(p.other()), h + usize::from(!noncreature));
            assert_eq!(g.counter_count(v, CounterKind::PlusOne), 0);
            assert_eq!(original.event_object.as_ref().unwrap().object, old);
        }
    }
}
#[test]
fn msh_leader_entering_noncreature_kindred_still_connives_without_replacement() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        leader(&mut g, p);
        let b = paid(&mut g, p, "Baron Strucker, HYDRA Overlord", &[]);
        settle(&mut g);
        // Immutable gated Kindred metadata plus a real earlier-layer subtype row.
        let v = named(&mut g, p, "Hill Giant", Zone::Graveyard);
        g.state.objects[v as usize].card = Card::from_name("Boggart Mischief").unwrap();
        let ev = g.move_objects_raw(&[v], Zone::Battlefield);
        let r = g.object_ref(v);
        let timestamp = g.next_role_timestamp();
        g.state.roles.get_or_insert_default().creature_types.push(
            subtypes::CreatureTypesEffectView {
                recipient: r,
                source: r,
                types: &["Villain"],
                additional: false,
                timestamp,
            },
        );
        g.finish_zone_events(&ev, &[]);
        assert!(!g.is_creature(v));
        assert_eq!(
            packet(&g, b)
                .event_object
                .as_ref()
                .unwrap()
                .conniver_creature,
            None
        );
        let d = named(&mut g, p, "Shock", Zone::Hand);
        let h = g.hand_count(p);
        accept(&mut g);
        discard(&mut g, p, d);
        assert_eq!(g.hand_count(p), h);
        assert_eq!(g.counter_count(v, CounterKind::PlusOne), 1);
    }
}
#[test]
fn msh_leader_live_noncreature_entrant_loses_creature_before_connive() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        leader(&mut g, p);
        let (_, v) = entry(&mut g, p);
        moon(&mut g, p, v);
        let d = named(&mut g, p, "Shock", Zone::Hand);
        let h = g.hand_count(p);
        accept(&mut g);
        discard(&mut g, p, d);
        assert_eq!(g.hand_count(p), h);
        assert_eq!(g.counter_count(v, CounterKind::PlusOne), 1);
    }
}
#[test]
fn msh_leader_multiple_replacements_choose_once_owned_orphan_restore() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let s = leader(&mut g, p);
        // Production nonlegendary copy executor, not a card/query override.
        g.copy_controlled_nontoken_creatures_nonlegendary(p);
        let copies: Vec<_> = g
            .state
            .objects
            .iter()
            .enumerate()
            .filter(|(_, o)| {
                o.zone == Zone::Battlefield
                    && o.is_token()
                    && o.card
                        .replacement_effects()
                        .contains(&crate::cards::ReplacementSpec::ConniveDraw)
            })
            .map(|(i, _)| i as CardId)
            .collect();
        assert_eq!(copies.len(), 1);
        let c = copies[0];
        named(&mut g, p, "Lorehold, the Historian", Zone::Battlefield);
        let m = paid(&mut g, p, "M.O.D.O.K.", &[]);
        settle(&mut g);
        let d = named(&mut g, p, "Shock", Zone::Hand);
        let h = g.hand_count(p);
        activate(&mut g, p, m);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::Replacement { .. })
        });
        let view = g.observe(p).replacement_choice.unwrap();
        assert_eq!(view.kind, "Connive");
        assert_eq!(view.effects.len(), 2);
        assert_eq!(view.object, Some(g.object_ref(m)));
        let wrong = g.object_ref(m);
        reject(
            &mut g,
            Action::ChooseReplacement {
                source: wrong,
                ability: 0,
            },
        );
        let choose = Action::ChooseReplacement {
            source: g.object_ref(c),
            ability: 0,
        };
        g = orphan(g, choose);
        assert!(matches!(g.state.pending, Pending::MiracleReveal { .. }));
        g = orphan(g, Action::DeclineMiracle);
        discard(&mut g, p, d);
        assert_eq!(g.hand_count(p), h + 2);
        assert_eq!(g.counter_count(m, CounterKind::PlusOne), 1);
        assert!(g.on_battlefield(s));
    }
}
#[test]
fn msh_leader_first_extra_draw_miracle_pause_before_underlying_draw_and_private_choices() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let s = leader(&mut g, p);
        named(&mut g, p, "Lorehold, the Historian", Zone::Battlefield);
        let first = named(&mut g, p, "Shock", Zone::Library);
        let h = g.hand_count(p);
        let n = g.state.players[p.index()].library.len();
        combat(&mut g, p, s);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::MiracleReveal { .. })
        });
        assert_eq!(g.hand_count(p), h + 1);
        assert_eq!(g.state.players[p.index()].library.len(), n - 1);
        assert!(g.observe(p).miracle.is_some());
        assert!(g.observe(p.other()).miracle.is_none());
        assert_eq!(g.state.resolving.as_ref().unwrap().discards[p.index()], 0);
        reject(&mut g, Action::Discard { card: first });
        g = orphan(g, Action::DeclineMiracle);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectDiscard { .. })
        });
        assert_eq!(g.hand_count(p), h + 2);
        discard(&mut g, p, first);
        assert_eq!(g.hand_count(p), h + 1);
    }
}
#[test]
fn msh_leader_suppressed_static_and_combat_trigger_are_unavailable() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let s = leader(&mut g, p);
        blank(&mut g, p, s);
        let m = paid(&mut g, p, "M.O.D.O.K.", &[]);
        settle(&mut g);
        let d = named(&mut g, p, "Shock", Zone::Hand);
        let h = g.hand_count(p);
        activate(&mut g, p, m);
        discard(&mut g, p, d);
        assert_eq!(g.hand_count(p), h);
        main(&mut g, p);
        g.begin_step(Step::BeginCombat);
        assert!(!g.state.pending_triggers.iter().any(|x| x.card == s));
    }
}
#[test]
fn msh_leader_opponent_provider_does_not_replace_our_connive() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        leader(&mut g, p.other());
        let m = paid(&mut g, p, "M.O.D.O.K.", &[]);
        settle(&mut g);
        let d = named(&mut g, p, "Shock", Zone::Hand);
        let h = g.hand_count(p);
        activate(&mut g, p, m);
        discard(&mut g, p, d);
        assert_eq!(g.hand_count(p), h);
    }
}
#[test]
fn msh_leader_counter_replacement_and_opponent_discard_preserve_placer() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        leader(&mut g, p.other());
        named(&mut g, p.other(), "Doubling Season", Zone::Battlefield);
        let m = paid(&mut g, p, "M.O.D.O.K.", &[]);
        settle(&mut g);
        let d = named(&mut g, p.other(), "Wilt-Leaf Liege", Zone::Hand);
        activate(&mut g, p, m);
        g.change_controller(g.object_ref(m), p.other());
        let h = g.hand_count(p.other());
        discard(&mut g, p.other(), d);
        assert_eq!(g.hand_count(p.other()), h + 1);
        assert_eq!(g.object_view(d).zone, Zone::Battlefield);
        assert_eq!(g.counter_count(m, CounterKind::PlusOne), 2);
        assert!(g.put_counter_on_source_this_turn(g.object_ref(m), p.other()));
        assert!(!g.put_counter_on_source_this_turn(g.object_ref(m), p));
    }
}
#[test]
fn msh_leader_stratagem_selected_group_only_never_connives_moved_opponent_target() {
    for p in [Player::P0, Player::P1] {
        for select in [false, true] {
            let mut g = setup(p);
            leader(&mut g, p);
            let victim = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
            let own = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let d = named(&mut g, p, "Shock", Zone::Hand);
            let before = [g.hand_count(p), g.hand_count(p.other())];
            let targets = if select {
                vec![Target::Card(victim), Target::Card(own)]
            } else {
                vec![Target::Card(victim)]
            };
            paid(&mut g, p, "Trickster's Stratagem", &targets);
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::TargetLibrary { .. })
            });
            let a = g
                .legal_actions()
                .into_iter()
                .find(|a| {
                    matches!(
                        a,
                        Action::ChooseLibraryDestination {
                            destination: LibraryDestination::SecondFromTop,
                            ..
                        }
                    )
                })
                .unwrap();
            checked(&mut g, a);
            if select {
                discard(&mut g, p, d);
            } else {
                settle(&mut g);
            }
            assert_eq!(g.hand_count(p.other()), before[1]);
            assert_eq!(g.hand_count(p), before[0] + usize::from(select));
            assert_eq!(g.object_view(victim).zone, Zone::Library);
            assert_eq!(
                g.counter_count(own, CounterKind::PlusOne),
                u64::from(select)
            );
        }
    }
}
#[test]
fn msh_leader_hidden_discard_counterfactual_atomic_orphan_restore() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let s = leader(&mut g, p);
        let d = named(&mut g, p, "Shock", Zone::Hand);
        let foreign = named(&mut g, p.other(), "Giant Growth", Zone::Hand);
        combat(&mut g, p, s);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectDiscard { .. })
        });
        let mut hidden = g.clone();
        hidden.state.objects[d as usize].card = Card::from_name("Giant Growth").unwrap();
        assert_eq!(
            serde_json::to_value(g.observe(p.other())).unwrap(),
            serde_json::to_value(hidden.observe(p.other())).unwrap()
        );
        reject(&mut g, Action::Discard { card: foreign });
        let choice = Action::Discard { card: d };
        g = orphan(g, choice);
        settle(&mut g);
        assert_eq!(g.counter_count(s, CounterKind::PlusOne), 1);
    }
}

#[test]
fn msh_leader_current_provider_controller_and_queued_trigger_captured_controller() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let s = leader(&mut g, p);
        let b = named(&mut g, p, "Hill Giant", Zone::Battlefield);
        let d = named(&mut g, p, "Shock", Zone::Hand);
        combat(&mut g, p, b);
        let h = g.hand_count(p);
        g.change_controller(g.object_ref(s), p.other());
        discard(&mut g, p, d);
        assert_eq!(g.hand_count(p), h);
        assert_eq!(g.counter_count(b, CounterKind::PlusOne), 1);
    }
}
#[test]
fn msh_leader_counterfactual_current_creature_kind_beats_printed_noncreature() {
    for p in [Player::P0, Player::P1] {
        for departed in [false, true] {
            let mut g = setup(p);
            leader(&mut g, p);
            let b = paid(&mut g, p, "Baron Strucker, HYDRA Overlord", &[]);
            settle(&mut g);
            let v = named(&mut g, p, "Hill Giant", Zone::Graveyard);
            g.state.objects[v as usize].card = Card::from_name("Boggart Mischief").unwrap();
            let ev = g.move_objects_raw(&[v], Zone::Battlefield);
            let r = g.object_ref(v);
            let timestamp = g.next_role_timestamp();
            g.state.roles.get_or_insert_default().creature_types.push(
                subtypes::CreatureTypesEffectView {
                    recipient: r,
                    source: r,
                    types: &["Villain"],
                    additional: false,
                    timestamp,
                },
            );
            g.finish_zone_events(&ev, &[]);
            assert!(!g.is_creature(v));
            // Ordinary earlier-layer animation after the entry trigger has queued.
            let item = g.cast_preview(p, v, Zone::Battlefield, r, 0, [0; 6]);
            g.animate_source(
                &item,
                crate::cards::AnimationSpec {
                    artifact: false,
                    creature_types: crate::cards::AnimationCreatureTypes::Unchanged,
                    base: Some((4, 4)),
                    keywords: Keywords::NONE,
                    duration: crate::cards::AnimationDuration::Permanent,
                },
            );
            assert!(g.is_creature(v));
            if departed {
                moved(&mut g, &[v], Zone::Exile);
                assert_eq!(
                    packet(&g, b)
                        .event_object
                        .as_ref()
                        .unwrap()
                        .conniver_creature,
                    Some(true)
                );
            }
            let d = named(&mut g, p, "Shock", Zone::Hand);
            let h = g.hand_count(p);
            accept(&mut g);
            discard(&mut g, p, d);
            assert_eq!(g.hand_count(p), h + 1);
            assert_eq!(
                g.counter_count(v, CounterKind::PlusOne),
                u64::from(!departed)
            );
        }
    }
}
#[test]
fn msh_leader_creature_mana_cost_requires_two_blue_and_no_false_activation() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        main(&mut g, p);
        let s = named(&mut g, p, LEADER, Zone::Hand);
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 3];
        reject(&mut g, Action::AnnounceCast { card: s });
        g.state.players[p.index()].mana = [0, 2, 0, 0, 0, 2];
        cast(&mut g, s, None, &[]);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        settle(&mut g);
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: s,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
    }
}

#[test]
fn msh_leader_extra_draw_empty_or_short_library_still_discards_then_loses() {
    for p in [Player::P0, Player::P1] {
        for count in [0, 1] {
            let mut g = setup(p);
            let s = leader(&mut g, p);
            let ids = g.state.players[p.index()].library.clone();
            moved(&mut g, &ids, Zone::Outside);
            for _ in 0..count {
                named(&mut g, p, "Forest", Zone::Library);
            }
            let d = named(&mut g, p, "Shock", Zone::Hand);
            combat(&mut g, p, s);
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::EffectDiscard { .. })
            });
            assert!(g.state.players[p.index()].attempted_empty_draw);
            assert_eq!(g.state.players[p.index()].library.len(), 0);
            checked(&mut g, Action::Discard { card: d });
            settle(&mut g);
            assert_eq!(g.counter_count(s, CounterKind::PlusOne), 1);
            assert_eq!(g.object_view(d).zone, Zone::Graveyard);
            assert_eq!(g.outcome(), Some(Outcome::Winner(p.other())));
        }
    }
}
