//! Vicious Verse and Omit Variables preparation identities and card outcomes.
use super::*;

fn announce_extra(g: &mut Game, action: Action, target: Option<Target>) {
    act(g, action);
    let mut target = target;
    for _ in 0..20 {
        if g.state.announcing.is_none() {
            return;
        }
        if let Some(wanted) = target {
            let select = Action::ChooseTarget { target: wanted };
            if g.legal_actions().contains(&select) {
                act(g, select);
                target = None;
                continue;
            }
        }
        choose(g, |a| {
            matches!(
                a,
                Action::ChooseSpellCost { .. } | Action::FinishTargets | Action::PayCosts { .. }
            )
        });
    }
    panic!("prepared spell announcement did not complete");
}

fn settle_extra(g: &mut Game) {
    for _ in 0..96 {
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.resolving.is_none()
            && g.state.pending_triggers.is_empty()
        {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("prepared spell did not resolve");
}

fn prepare_extra(g: &mut Game, p: Player, name: &str, on_entry: bool) -> (CardId, ObjectRef) {
    let source = put(g, p, name, Zone::Hand);
    announce_extra(g, Action::AnnounceCast { card: source }, None);
    settle_extra(g);
    if !on_entry {
        assert!(!g.is_prepared(g.object_ref(source)));
        g.begin_step(Step::Upkeep);
        g.prepare_triggers();
        settle_extra(g);
        // Return to the main phase for the prepared sorcery.
        g.state.step = Step::Main1;
        g.state.priority = p;
    }
    let permanent = g.object_ref(source);
    let copy = g
        .state
        .preparations
        .as_ref()
        .unwrap()
        .iter()
        .find(|row| row.permanent == permanent)
        .unwrap()
        .spell_copy;
    (source, copy)
}

#[test]
fn fra_verse_and_omit_prepared_faces_cast_for_each_hybrid_color() {
    for p in [Player::P0, Player::P1] {
        for alternative in [false, true] {
            for (name, index, on_entry, verse) in [
                ("Hallway Heckler // Vicious Verse", 5447, true, true),
                ("Stingerquill Voxmancer // Vicious Verse", 5448, false, true),
                ("Whiplash Wordsmith // Vicious Verse", 5449, true, true),
                ("Paradox Shaper // Omit Variables", 5450, false, false),
                ("Theorix Metamage // Omit Variables", 5451, true, false),
                ("Void Extrapolator // Omit Variables", 5452, true, false),
            ] {
                let mut g = scenario(p);
                let (source, copy) = prepare_extra(&mut g, p, name, on_entry);
                assert_eq!(
                    g.state.objects[copy.card as usize].card,
                    Card::test_index(index)
                );
                let before = g.state.players[p.index()].library.len();
                let life = g.state.players[p.other().index()].life;
                g.state.players[p.index()].mana = if verse {
                    if alternative {
                        [0, 0, 0, 1, 0, 0]
                    } else {
                        [0, 0, 1, 0, 0, 0]
                    }
                } else if alternative {
                    [0, 0, 1, 0, 0, 0]
                } else {
                    [0, 1, 0, 0, 0, 0]
                };
                super::super::contracts::branch_contract(g.clone());
                announce_extra(
                    &mut g,
                    Action::AnnounceExile { card: copy.card },
                    verse.then_some(Target::Player(p.other())),
                );
                assert!(!g.is_prepared(g.object_ref(source)));
                settle_extra(&mut g);
                assert_eq!(g.state.objects[copy.card as usize].zone, Zone::Outside);
                assert_eq!(
                    g.state.players[p.other().index()].life,
                    life - i128::from(verse)
                );
                assert_eq!(
                    g.state.players[p.index()].library.len(),
                    before - if verse { 0 } else { 3 }
                );
                if name.starts_with("Whiplash") {
                    assert!(g.keywords(source).contains(Keywords::FLYING));
                    assert!(g.keywords(source).contains(Keywords::HASTE));
                }
            }
        }
    }
}

#[test]
fn fra_upkeep_preparation_repeats_only_after_unpreparing() {
    for p in [Player::P0, Player::P1] {
        for (name, verse) in [
            ("Stingerquill Voxmancer // Vicious Verse", true),
            ("Paradox Shaper // Omit Variables", false),
        ] {
            let mut g = scenario(p);
            let (source, first) = prepare_extra(&mut g, p, name, false);
            g.begin_step(Step::Upkeep);
            g.prepare_triggers();
            assert!(g.state.pending_triggers.is_empty() && g.state.stack.is_empty());
            assert!(g.is_prepared(g.object_ref(source)));
            g.state.step = Step::Main1;
            g.state.priority = p;
            announce_extra(
                &mut g,
                Action::AnnounceExile { card: first.card },
                verse.then_some(Target::Player(p.other())),
            );
            settle_extra(&mut g);
            assert!(!g.is_prepared(g.object_ref(source)));
            g.begin_step(Step::Upkeep);
            g.prepare_triggers();
            settle_extra(&mut g);
            assert!(g.is_prepared(g.object_ref(source)));
            let second = g
                .state
                .preparations
                .as_ref()
                .unwrap()
                .iter()
                .find(|row| row.permanent == g.object_ref(source))
                .unwrap()
                .spell_copy;
            assert_ne!(first, second);
        }
    }
}

#[test]
fn fra_omit_creatures_threshold_tracks_own_current_graveyard() {
    for p in [Player::P0, Player::P1] {
        for (name, before, after, flying) in [
            ("Theorix Metamage // Omit Variables", (2, 3), (3, 3), true),
            ("Void Extrapolator // Omit Variables", (2, 2), (3, 3), false),
        ] {
            let mut g = scenario(p);
            let source = put(&mut g, p, name, Zone::Battlefield);
            for _ in 0..6 {
                put(&mut g, p, "Forest", Zone::Graveyard);
            }
            put(&mut g, p.other(), "Forest", Zone::Graveyard);
            assert_eq!(stats(&g, source), before);
            assert!(!g.keywords(source).contains(Keywords::FLYING));
            let seventh = put(&mut g, p, "Forest", Zone::Graveyard);
            assert_eq!(stats(&g, source), after);
            assert_eq!(g.keywords(source).contains(Keywords::FLYING), flying);
            g.move_object(seventh, Zone::Exile);
            assert_eq!(stats(&g, source), before);
            assert!(!g.keywords(source).contains(Keywords::FLYING));
        }
    }
}

#[test]
fn fra_paradox_shaper_puts_own_graveyard_card_on_bottom() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(
            &mut g,
            p,
            "Paradox Shaper // Omit Variables",
            Zone::Battlefield,
        );
        let own = put(&mut g, p, "Bear Cub", Zone::Graveyard);
        let other = put(&mut g, p.other(), "Bear Cub", Zone::Graveyard);
        act(
            &mut g,
            Action::AnnounceActivate {
                card: source,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(other),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(own),
            },
        );
        pay(&mut g);
        settle_extra(&mut g);
        assert_eq!(g.state.objects[own as usize].zone, Zone::Library);
        assert_eq!(g.state.players[p.index()].library.first(), Some(&own));
    }
}

#[test]
fn fra_hallway_heckler_tap_and_discard_cost_then_draw() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(
            &mut g,
            p,
            "Hallway Heckler // Vicious Verse",
            Zone::Battlefield,
        );
        let action = Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        };
        reject(&mut g, action.clone());
        assert!(!g.state.objects[source as usize].tapped);
        let discarded = put(&mut g, p, "Bear Cub", Zone::Hand);
        let hand = g.hand_count(p);
        act(&mut g, action);
        let object = g.object_ref(discarded);
        act(&mut g, Action::ChooseAdditionalCost { object });
        pay(&mut g);
        assert!(g.state.objects[source as usize].tapped);
        assert_eq!(g.state.objects[discarded as usize].zone, Zone::Graveyard);
        settle_extra(&mut g);
        assert_eq!(g.hand_count(p), hand);
    }
}

#[test]
fn fra_whiplash_noncombat_damage_tracks_opponent_and_current_controller() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(
            &mut g,
            p,
            "Whiplash Wordsmith // Vicious Verse",
            Zone::Battlefield,
        );
        assert!(!g.keywords(source).contains(Keywords::FLYING));
        let self_damage = put(&mut g, p, "Shock", Zone::Hand);
        cast(&mut g, self_damage, Some(Target::Player(p)));
        settle_extra(&mut g);
        assert!(!g.keywords(source).contains(Keywords::FLYING));
        g.change_controller(g.object_ref(source), p.other());
        assert!(g.keywords(source).contains(Keywords::FLYING));
        g.change_controller(g.object_ref(source), p);
        assert!(!g.keywords(source).contains(Keywords::FLYING));
        let opponent_damage = put(&mut g, p, "Shock", Zone::Hand);
        cast(&mut g, opponent_damage, Some(Target::Player(p.other())));
        settle_extra(&mut g);
        assert!(g.keywords(source).contains(Keywords::FLYING));
        assert!(g.keywords(source).contains(Keywords::HASTE));
        let aura = put(&mut g, p.other(), "Witness Protection", Zone::Battlefield);
        g.state.objects[aura as usize].attached_to = Some(g.object_ref(source));
        assert!(!g.keywords(source).contains(Keywords::FLYING));
        g.move_object(aura, Zone::Graveyard);
        assert!(g.keywords(source).contains(Keywords::FLYING));
        g.finish_cleanup();
        assert!(!g.keywords(source).contains(Keywords::FLYING));
        assert!(!g.keywords(source).contains(Keywords::HASTE));
    }
}

#[test]
fn fra_vicious_verse_rejects_self_and_fizzles_for_hexproof_opponent() {
    for p in [Player::P0, Player::P1] {
        for name in [
            "Hallway Heckler // Vicious Verse",
            "Stingerquill Voxmancer // Vicious Verse",
            "Whiplash Wordsmith // Vicious Verse",
        ] {
            let mut g = scenario(p);
            let (source, copy) = prepare_extra(&mut g, p, name, !name.starts_with("Stingerquill"));
            g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 0];
            act(&mut g, Action::AnnounceExile { card: copy.card });
            for _ in 0..16 {
                let actions = g.legal_actions();
                let target = Action::ChooseTarget {
                    target: Target::Player(p.other()),
                };
                if actions.contains(&target) {
                    reject(
                        &mut g,
                        Action::ChooseTarget {
                            target: Target::Player(p),
                        },
                    );
                    act(&mut g, target);
                    break;
                }
                choose(&mut g, |a| matches!(a, Action::ChooseSpellCost { .. }));
            }
            pay(&mut g);
            assert!(!g.is_prepared(g.object_ref(source)));
            let life = g.state.players[p.other().index()].life;
            g.state.players[p.other().index()].hexproof_until_end_turn = true;
            let saved = g.snapshot();
            super::super::contracts::branch_contract(g.clone());
            for replay in 0..2 {
                if replay == 1 {
                    g.restore(&saved).unwrap();
                }
                settle_extra(&mut g);
                assert_eq!(g.state.players[p.other().index()].life, life);
                assert!(!g.observe(p).players[p.other().index()].noncombat_damage_this_turn);
                assert_eq!(g.object_view(copy.card).zone, Zone::Outside);
            }
        }
    }
}

#[test]
fn fra_omit_variables_handles_short_library_and_graveyard_replacement() {
    for p in [Player::P0, Player::P1] {
        for replacement in [false, true] {
            let mut g = scenario(p);
            let (source, copy) =
                prepare_extra(&mut g, p, "Void Extrapolator // Omit Variables", true);
            while g.state.players[p.index()].library.len() > 2 {
                let card = *g.state.players[p.index()].library.last().unwrap();
                g.move_object(card, Zone::Exile);
            }
            let selected = g.state.players[p.index()].library.clone();
            assert_eq!(selected.len(), 2);
            if replacement {
                put(&mut g, p, "Rest in Peace", Zone::Battlefield);
            }
            g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 0];
            announce_extra(&mut g, Action::AnnounceExile { card: copy.card }, None);
            assert!(!g.is_prepared(g.object_ref(source)));
            let saved = g.snapshot();
            super::super::contracts::branch_contract(g.clone());
            for replay in 0..2 {
                if replay == 1 {
                    g.restore(&saved).unwrap();
                }
                settle_extra(&mut g);
                assert!(g.state.players[p.index()].library.is_empty());
                for id in &selected {
                    assert_eq!(
                        g.object_view(*id).zone,
                        if replacement {
                            Zone::Exile
                        } else {
                            Zone::Graveyard
                        }
                    );
                }
            }
        }
    }
}

#[test]
fn fra_upkeep_preparation_rechecks_designation_and_exact_source() {
    for p in [Player::P0, Player::P1] {
        for name in [
            "Stingerquill Voxmancer // Vicious Verse",
            "Paradox Shaper // Omit Variables",
        ] {
            for reenter in [false, true] {
                let mut g = scenario(p);
                let source = put(&mut g, p, name, Zone::Battlefield);
                let original = g.object_ref(source);
                g.begin_step(Step::Upkeep);
                g.prepare_triggers();
                assert!(!g.state.stack.is_empty() || !g.state.pending_triggers.is_empty());
                let manually_prepared = if reenter {
                    g.move_object(source, Zone::Exile);
                    g.move_object(source, Zone::Battlefield);
                    assert_ne!(g.object_ref(source), original);
                    None
                } else {
                    g.prepare_permanent(original);
                    Some(
                        g.state
                            .preparations
                            .as_ref()
                            .unwrap()
                            .iter()
                            .find(|row| row.permanent == original)
                            .unwrap()
                            .spell_copy,
                    )
                };
                super::super::contracts::branch_contract(g.clone());
                settle_extra(&mut g);
                if let Some(expected) = manually_prepared {
                    assert_eq!(
                        g.state
                            .preparations
                            .as_ref()
                            .unwrap()
                            .iter()
                            .find(|row| row.permanent == original)
                            .unwrap()
                            .spell_copy,
                        expected
                    );
                } else {
                    assert!(!g.is_prepared(g.object_ref(source)));
                }
            }
            let mut g = scenario(p.other());
            let source = put(&mut g, p, name, Zone::Battlefield);
            g.begin_step(Step::Upkeep);
            g.prepare_triggers();
            assert!(g.state.pending_triggers.is_empty() && g.state.stack.is_empty());
            assert!(!g.is_prepared(g.object_ref(source)));
        }
    }
}

#[test]
fn fra_paradox_unfunded_and_stale_graveyard_target() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(
            &mut g,
            p,
            "Paradox Shaper // Omit Variables",
            Zone::Battlefield,
        );
        let target = put(&mut g, p, "Bear Cub", Zone::Graveyard);
        let action = Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        };
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        reject(&mut g, action.clone());
        assert_eq!(g.object_view(target).zone, Zone::Graveyard);
        g.state.players[p.index()].mana[5] = 2;
        act(&mut g, action);
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        pay(&mut g);
        let before = g.object_ref(target);
        g.move_object(target, Zone::Exile);
        g.move_object(target, Zone::Graveyard);
        assert_ne!(before, g.object_ref(target));
        let saved = g.snapshot();
        super::super::contracts::branch_contract(g.clone());
        for replay in 0..2 {
            if replay == 1 {
                g.restore(&saved).unwrap();
            }
            settle_extra(&mut g);
            assert_eq!(g.object_view(target).zone, Zone::Graveyard);
        }
    }
}

#[test]
fn fra_hallway_discard_replacement_preserves_paid_tap_and_draw() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p, "Rest in Peace", Zone::Battlefield);
        let source = put(
            &mut g,
            p,
            "Hallway Heckler // Vicious Verse",
            Zone::Battlefield,
        );
        let discard = put(&mut g, p, "Bear Cub", Zone::Hand);
        let hand = g.hand_count(p);
        act(
            &mut g,
            Action::AnnounceActivate {
                card: source,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        let object = g.object_ref(discard);
        act(&mut g, Action::ChooseAdditionalCost { object });
        pay(&mut g);
        assert!(g.object_view(source).tapped);
        assert_eq!(g.object_view(discard).zone, Zone::Exile);
        super::super::contracts::branch_contract(g.clone());
        settle_extra(&mut g);
        assert_eq!(g.hand_count(p), hand);
    }
}

#[test]
fn fra_prepared_fronts_accept_each_printed_hybrid_cost() {
    for p in [Player::P0, Player::P1] {
        for (name, colored, generic) in [
            ("Hallway Heckler // Vicious Verse", [0, 0, 0, 1, 0, 0], 2),
            (
                "Stingerquill Voxmancer // Vicious Verse",
                [0, 0, 1, 0, 0, 0],
                0,
            ),
            (
                "Stingerquill Voxmancer // Vicious Verse",
                [0, 0, 0, 1, 0, 0],
                0,
            ),
            ("Whiplash Wordsmith // Vicious Verse", [0, 0, 1, 0, 0, 0], 3),
            ("Whiplash Wordsmith // Vicious Verse", [0, 0, 0, 1, 0, 0], 3),
            ("Paradox Shaper // Omit Variables", [0, 1, 0, 0, 0, 0], 1),
            ("Paradox Shaper // Omit Variables", [0, 0, 1, 0, 0, 0], 1),
            ("Theorix Metamage // Omit Variables", [0, 1, 0, 0, 0, 0], 2),
            ("Theorix Metamage // Omit Variables", [0, 0, 1, 0, 0, 0], 2),
            ("Void Extrapolator // Omit Variables", [0, 0, 1, 0, 0, 0], 1),
        ] {
            let mut g = scenario(p);
            let source = put(&mut g, p, name, Zone::Hand);
            g.state.players[p.index()].mana = colored;
            g.state.players[p.index()].mana[5] = generic;
            announce_extra(&mut g, Action::AnnounceCast { card: source }, None);
            settle_extra(&mut g);
            assert_eq!(g.object_view(source).zone, Zone::Battlefield);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        }
    }
}

#[test]
fn fra_prepared_verse_and_omit_countered_copies_leave_source_unprepared() {
    for p in [Player::P0, Player::P1] {
        for (name, verse) in [
            ("Hallway Heckler // Vicious Verse", true),
            ("Void Extrapolator // Omit Variables", false),
        ] {
            let mut g = scenario(p);
            let (source, copy) = prepare_extra(&mut g, p, name, true);
            let negate = put(&mut g, p.other(), "Negate", Zone::Hand);
            let life = g.state.players[p.other().index()].life;
            let library = g.state.players[p.index()].library.len();
            announce_extra(
                &mut g,
                Action::AnnounceExile { card: copy.card },
                verse.then_some(Target::Player(p.other())),
            );
            assert!(!g.is_prepared(g.object_ref(source)));
            act(&mut g, Action::Pass);
            cast(&mut g, negate, Some(Target::Spell(copy.card)));
            let saved = g.snapshot();
            super::super::contracts::branch_contract(g.clone());
            for replay in 0..2 {
                if replay == 1 {
                    g.restore(&saved).unwrap();
                }
                settle_extra(&mut g);
                assert_eq!(g.object_view(copy.card).zone, Zone::Outside);
                assert!(!g.is_prepared(g.object_ref(source)));
                assert_eq!(g.state.players[p.other().index()].life, life);
                assert_eq!(g.state.players[p.index()].library.len(), library);
            }
        }
    }
}

#[test]
fn fra_omit_variables_crosses_threshold_at_resolution() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let (theorix, copy) = prepare_extra(&mut g, p, "Theorix Metamage // Omit Variables", true);
        let void = put(
            &mut g,
            p,
            "Void Extrapolator // Omit Variables",
            Zone::Battlefield,
        );
        for _ in 0..4 {
            put(&mut g, p, "Forest", Zone::Graveyard);
        }
        assert_eq!(stats(&g, theorix), (2, 3));
        assert_eq!(stats(&g, void), (2, 2));
        assert!(!g.keywords(theorix).contains(Keywords::FLYING));
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 0];
        announce_extra(&mut g, Action::AnnounceExile { card: copy.card }, None);
        let saved = g.snapshot();
        super::super::contracts::branch_contract(g.clone());
        for replay in 0..2 {
            if replay == 1 {
                g.restore(&saved).unwrap();
            }
            settle_extra(&mut g);
            assert_eq!(stats(&g, theorix), (3, 3));
            assert_eq!(stats(&g, void), (3, 3));
            assert!(g.keywords(theorix).contains(Keywords::FLYING));
        }
    }
}
