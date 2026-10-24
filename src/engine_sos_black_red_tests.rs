//! Whole-card SOS rules tests using the bounded assurance scenario helpers.
use super::*;

fn until_choice(g: &mut Game) {
    for _ in 0..12 {
        if g.state.pending != Pending::Priority {
            return;
        }
        act(g, Action::Pass);
    }
    panic!("expected a suspended resolution choice");
}

#[test]
fn sos_dissection_independent_optional_targets_and_cleanup() {
    for p in [Player::P0, Player::P1] {
        for (plus, minus) in [(false, false), (true, false), (false, true), (true, true)] {
            let mut g = scenario(p);
            let a = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let b = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let spell = put(&mut g, p, "Dissection Practice", Zone::Hand);
            act(&mut g, Action::AnnounceCast { card: spell });
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Player(p),
                },
            );
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Player(p.other()),
                },
            );
            if plus {
                act(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(a),
                    },
                );
            } else {
                act(&mut g, Action::FinishTargets);
            }
            if minus {
                act(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(b),
                    },
                );
            } else {
                act(&mut g, Action::FinishTargets);
            }
            pay(&mut g);
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].life, 21);
            assert_eq!(g.state.players[p.other().index()].life, 19);
            assert_eq!(stats(&g, a), if plus { (3, 3) } else { (2, 2) });
            assert_eq!(stats(&g, b), if minus { (1, 1) } else { (2, 2) });
            g.finish_cleanup();
            assert_eq!(stats(&g, a), (2, 2));
            assert_eq!(stats(&g, b), (2, 2));
        }
        let mut g = scenario(p);
        let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let spell = put(&mut g, p, "Dissection Practice", Zone::Hand);
        announce(
            &mut g,
            spell,
            &[
                Target::Player(p.other()),
                Target::Card(bear),
                Target::Card(bear),
            ],
        );
        settle(&mut g);
        assert_eq!(
            stats(&g, bear),
            (2, 2),
            "one creature may occupy both occurrences of target"
        );
    }
}

#[test]
fn sos_dissection_partially_invalid_targets_preserve_other_effects() {
    let p = Player::P0;
    let mut g = scenario(p);
    let a = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let b = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
    let spell = put(&mut g, p, "Dissection Practice", Zone::Hand);
    let bounce = put(&mut g, p.other(), "Unsummon", Zone::Hand);
    announce(
        &mut g,
        spell,
        &[Target::Player(p.other()), Target::Card(a), Target::Card(b)],
    );
    act(&mut g, Action::Pass);
    cast(&mut g, bounce, Some(Target::Card(a)));
    settle(&mut g);
    assert_eq!(g.object_view(a).zone, Zone::Hand);
    assert_eq!(stats(&g, b), (1, 1));
    assert_eq!(g.state.players[p.index()].life, 21);
    assert_eq!(g.state.players[p.other().index()].life, 19);
}

#[test]
fn sos_flashback_grants_exact_card_full_cost_exile_and_expiry() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let shock = put(&mut g, p, "Shock", Zone::Graveyard);
        let other = put(&mut g, p.other(), "Shock", Zone::Graveyard);
        let bear = put(&mut g, p, "Bear Cub", Zone::Graveyard);
        let spell = put(&mut g, p, "Flashback", Zone::Hand);
        act(&mut g, Action::AnnounceCast { card: spell });
        for illegal in [other, bear] {
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(illegal),
                },
            );
        }
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(shock),
            },
        );
        pay(&mut g);
        settle(&mut g);
        assert!(g.has_graveyard_cast(p, shock, GraveyardCastKind::Flashback));
        let saved_mana: u32 = g.state.players[p.index()].mana.iter().sum();
        act(
            &mut g,
            Action::AnnounceGrantedFlashback {
                card: shock,
                flash_surcharge: false,
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        pay(&mut g);
        assert_eq!(
            saved_mana - g.state.players[p.index()].mana.iter().sum::<u32>(),
            1
        );
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 18);
        assert_eq!(g.object_view(shock).zone, Zone::Exile);
        assert!(!g.has_graveyard_cast(p, shock, GraveyardCastKind::Flashback));

        let retained = put(&mut g, p, "Shock", Zone::Graveyard);
        let grant = put(&mut g, p, "Flashback", Zone::Hand);
        cast(&mut g, grant, Some(Target::Card(retained)));
        settle(&mut g);
        assert!(g.has_graveyard_cast(p, retained, GraveyardCastKind::Flashback));
        g.finish_cleanup();
        assert!(!g.has_graveyard_cast(p, retained, GraveyardCastKind::Flashback));
    }
}

#[test]
fn sos_mica_optional_artifact_payment_copy_retarget_and_ward() {
    for p in [Player::P0, Player::P1] {
        for sacrifice in [false, true] {
            let mut g = scenario(p);
            let mica = put(&mut g, p, "Mica, Reader of Ruins", Zone::Battlefield);
            let artifact = put(&mut g, p, "Goldvein Pick", Zone::Battlefield);
            let shock = put(&mut g, p, "Shock", Zone::Hand);
            cast(&mut g, shock, Some(Target::Player(p.other())));
            until_choice(&mut g);
            if sacrifice {
                let object = g.object_ref(artifact);
                act(&mut g, Action::Sacrifice { object });
                assert_eq!(g.object_view(artifact).zone, Zone::Graveyard);
                act(
                    &mut g,
                    Action::SetStackTarget {
                        index: 0,
                        target: Target::Player(p),
                    },
                );
                act(&mut g, Action::FinishStackTargets);
            } else {
                act(&mut g, Action::DeclineEffectPayment);
            }
            settle(&mut g);
            assert_eq!(g.state.players[p.other().index()].life, 18);
            assert_eq!(
                g.state.players[p.index()].life,
                if sacrifice { 18 } else { 20 }
            );
            assert_eq!(g.object_view(mica).zone, Zone::Battlefield);
        }
        for pay_life in [false, true] {
            let mut g = scenario(p);
            let mica = put(&mut g, p, "Mica, Reader of Ruins", Zone::Battlefield);
            let murder = put(&mut g, p.other(), "Murder", Zone::Hand);
            act(&mut g, Action::Pass);
            cast(&mut g, murder, Some(Target::Card(mica)));
            until_choice(&mut g);
            act(
                &mut g,
                if pay_life {
                    Action::PayResolutionLife
                } else {
                    Action::DeclineResolutionLife
                },
            );
            settle(&mut g);
            assert_eq!(
                g.state.players[p.other().index()].life,
                if pay_life { 17 } else { 20 }
            );
            assert_eq!(
                g.object_view(mica).zone,
                if pay_life {
                    Zone::Graveyard
                } else {
                    Zone::Battlefield
                }
            );
        }
    }
}

#[test]
fn sos_artistic_process_modes_opponent_only_damage_and_temporary_haste() {
    for p in [Player::P0, Player::P1] {
        for mode in 0..3 {
            let mut g = scenario(p);
            let friendly = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let opposing = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let spell = put(&mut g, p, "Artistic Process", Zone::Hand);
            act(&mut g, Action::AnnounceCast { card: spell });
            act(&mut g, Action::ChooseMode { mode });
            if mode == 0 {
                act(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(opposing),
                    },
                );
            }
            pay(&mut g);
            settle(&mut g);
            assert_eq!(g.object_view(friendly).zone, Zone::Battlefield);
            assert_eq!(g.object_view(friendly).damage, 0);
            if mode < 2 {
                assert_eq!(g.object_view(opposing).zone, Zone::Graveyard);
            } else {
                let elemental = g
                    .state
                    .objects
                    .iter()
                    .enumerate()
                    .find_map(|(i, o)| {
                        (o.zone == Zone::Battlefield
                            && o.card == Card::from_name("@token/sos-elemental").unwrap())
                        .then_some(i as CardId)
                    })
                    .unwrap();
                assert_eq!(stats(&g, elemental), (3, 3));
                assert_eq!(g.object_view(elemental).controller, p);
                assert!(
                    g.keywords(elemental)
                        .contains(Keywords::FLYING.union(Keywords::HASTE))
                );
                g.finish_cleanup();
                assert!(g.keywords(elemental).contains(Keywords::FLYING));
                assert!(!g.keywords(elemental).contains(Keywords::HASTE));
            }
        }
    }
}

#[test]
fn sos_send_pest_opponent_discard_and_attack_life_not_death_life() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let hidden = put(&mut g, p.other(), "Shock", Zone::Hand);
        let spell = put(&mut g, p, "Send in the Pest", Zone::Hand);
        cast(&mut g, spell, None);
        until_choice(&mut g);
        assert_eq!(g.actor(), Some(p.other()));
        act(&mut g, Action::Discard { card: hidden });
        settle(&mut g);
        let pest = g
            .state
            .objects
            .iter()
            .enumerate()
            .find_map(|(i, o)| {
                (o.zone == Zone::Battlefield
                    && o.card == Card::from_name("@token/sos-pest").unwrap())
                .then_some(i as CardId)
            })
            .unwrap();
        assert_eq!(stats(&g, pest), (1, 1));
        assert_eq!(g.object_view(pest).controller, p);
        assert_eq!(g.state.players[p.index()].life, 20);
        g.state.objects[pest as usize].summoning_sick = false;
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        act(&mut g, Action::Attack { card: pest });
        act(&mut g, Action::FinishAttackers);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, 21);
    }
}

fn begin_paradigm_main(g: &mut Game, active: Player, step: Step) {
    g.state.active = active;
    g.state.priority = active;
    g.state.pending = Pending::Priority;
    g.begin_step(step);
    g.prepare_triggers();
}

#[test]
fn sos_paradigm_first_resolution_source_independence_optional_copy_and_cast_count() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let original = put(&mut g, p, "Decorum Dissertation", Zone::Hand);
        let library = g.state.players[p.other().index()].library.len();
        cast(&mut g, original, Some(Target::Player(p.other())));
        settle(&mut g);
        assert_eq!(
            g.state.players[p.other().index()].library.len(),
            library - 2
        );
        assert_eq!(g.state.players[p.other().index()].life, 18);
        assert_eq!(g.object_view(original).zone, Zone::Exile);
        assert_eq!(g.observe(p).paradigms.len(), 1);
        let second = put(&mut g, p, "Decorum Dissertation", Zone::Hand);
        cast(&mut g, second, Some(Target::Player(p)));
        settle(&mut g);
        assert_eq!(
            g.observe(p).paradigms.len(),
            1,
            "second same-name resolution does not multiply the delayed ability"
        );
        assert_eq!(g.object_view(second).zone, Zone::Exile);
        // The delayed ability is independent of both the old card and its incarnation.
        g.move_object(original, Zone::Hand);
        begin_paradigm_main(&mut g, p.other(), Step::Main1);
        assert!(g.state.stack.is_empty());
        begin_paradigm_main(&mut g, p, Step::Main2);
        assert!(g.state.stack.is_empty());
        begin_paradigm_main(&mut g, p, Step::Main1);
        until_choice(&mut g);
        let unused = g.state.resolution_free_casts.as_ref().unwrap().remaining[0];
        assert_eq!(g.object_view(unused.card).zone, Zone::Exile);
        // One representative suspended private-owned root checks raw/index replay,
        // stale revision and restore, instead of multiplying that cost per action.
        if p == Player::P0 {
            super::big_helpers::checked(&mut g, Action::FinishResolutionCasts);
        } else {
            act(&mut g, Action::FinishResolutionCasts);
        }
        settle(&mut g);
        assert_eq!(g.object_view(unused.card).zone, Zone::Outside);
        assert_eq!(g.observe(p.other()).paradigms.len(), 1);
        begin_paradigm_main(&mut g, p, Step::Main1);
        until_choice(&mut g);
        let copy = g.state.resolution_free_casts.as_ref().unwrap().remaining[0];
        let mana = g.state.players[p.index()].mana;
        let casts = g.state.players[p.index()].spells_cast_this_turn;
        act(
            &mut g,
            Action::CastExiledDuringResolution {
                object: copy,
                face: CastFace::Normal,
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        pay(&mut g);
        assert_eq!(g.state.players[p.index()].mana, mana);
        assert_eq!(g.state.players[p.index()].spells_cast_this_turn, casts + 1);
        act(&mut g, Action::FinishResolutionCasts);
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 16);
        assert_eq!(g.observe(p).paradigms.len(), 1);
        assert_eq!(g.object_view(copy.card).zone, Zone::Outside);
    }
}

#[test]
fn sos_countered_paradigm_never_registers_but_countered_copy_preserves_recurrence() {
    let p = Player::P0;
    let mut g = scenario(p);
    let original = put(&mut g, p, "Decorum Dissertation", Zone::Hand);
    let negate = put(&mut g, p.other(), "Negate", Zone::Hand);
    cast(&mut g, original, Some(Target::Player(p)));
    act(&mut g, Action::Pass);
    cast(&mut g, negate, Some(Target::Spell(original)));
    settle(&mut g);
    assert!(g.observe(p).paradigms.is_empty());
    assert!(
        !g.observe(p)
            .resolved_spell_names
            .iter()
            .any(|row| row.name == "Decorum Dissertation")
    );
    assert_eq!(g.object_view(original).zone, Zone::Graveyard);
    let second = put(&mut g, p, "Decorum Dissertation", Zone::Hand);
    cast(&mut g, second, Some(Target::Player(p)));
    settle(&mut g);
    begin_paradigm_main(&mut g, p, Step::Main1);
    until_choice(&mut g);
    let copy = g.state.resolution_free_casts.as_ref().unwrap().remaining[0];
    act(
        &mut g,
        Action::CastExiledDuringResolution {
            object: copy,
            face: CastFace::Normal,
        },
    );
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Player(p),
        },
    );
    pay(&mut g);
    act(&mut g, Action::FinishResolutionCasts);
    let negate = put(&mut g, p.other(), "Negate", Zone::Hand);
    act(&mut g, Action::Pass);
    cast(&mut g, negate, Some(Target::Spell(copy.card)));
    settle(&mut g);
    assert_eq!(g.observe(p).paradigms.len(), 1);
    begin_paradigm_main(&mut g, p, Step::Main1);
    until_choice(&mut g);
    assert_eq!(
        g.state
            .resolution_free_casts
            .as_ref()
            .unwrap()
            .remaining
            .len(),
        1
    );
}

#[test]
fn sos_capstone_exiles_to_total_threshold_then_casts_only_exiled_spells_free() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let retained = put(&mut g, p, "Murder", Zone::Library);
        let threshold = put(&mut g, p, "Murder", Zone::Library);
        let bear = put(&mut g, p, "Bear Cub", Zone::Library);
        let land = put(&mut g, p, "Mountain", Zone::Library);
        let spell = put(&mut g, p, "Improvisation Capstone", Zone::Hand);
        cast(&mut g, spell, None);
        until_choice(&mut g);
        assert_eq!(g.object_view(retained).zone, Zone::Library);
        for id in [threshold, bear, land] {
            assert_eq!(g.object_view(id).zone, Zone::Exile);
        }
        reject(&mut g, Action::PlayLand { card: land });
        let object = g.object_ref(bear);
        let mana = g.state.players[p.index()].mana;
        act(
            &mut g,
            Action::CastExiledDuringResolution {
                object,
                face: CastFace::Normal,
            },
        );
        pay(&mut g);
        assert_eq!(g.state.players[p.index()].mana, mana);
        act(&mut g, Action::FinishResolutionCasts);
        settle(&mut g);
        assert_eq!(g.object_view(bear).zone, Zone::Battlefield);
        assert_eq!(g.object_view(threshold).zone, Zone::Exile);
        assert_eq!(g.object_view(spell).zone, Zone::Exile);
        assert_eq!(g.observe(p).paradigms.len(), 1);
    }
}
