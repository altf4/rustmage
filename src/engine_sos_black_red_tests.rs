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

fn copied_prepared_spell(g: &mut Game, p: Player, front: &str, targets: &[Target]) -> CardId {
    let definition = Card::from_name(front).unwrap().prepared_spell().unwrap();
    let copy = g.allocate_card_copy(definition, p);
    g.grant_exile_play_permission(p, copy, false);
    act(g, Action::AnnounceExile { card: copy.card });
    for &target in targets {
        act(g, Action::ChooseTarget { target });
    }
    pay(g);
    copy.card
}

#[test]
fn sos_prepared_black_spell_faces_counters_return_drain_draw_and_private_tutor() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        copied_prepared_spell(&mut g, p, "Adventurous Eater", &[Target::Card(bear)]);
        settle(&mut g);
        assert_eq!(g.counter_count(bear, CounterKind::PlusOne), 1);
        assert_eq!(g.state.players[p.index()].life, 21);
        let grave = put(&mut g, p, "Bear Cub", Zone::Graveyard);
        copied_prepared_spell(&mut g, p, "Cheerful Osteomancer", &[Target::Card(grave)]);
        settle(&mut g);
        assert_eq!(g.object_view(grave).zone, Zone::Hand);
        copied_prepared_spell(&mut g, p, "Leech Collector", &[]);
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 18);
        let hand = g.hand_count(p.other());
        copied_prepared_spell(
            &mut g,
            p,
            "Scheming Silvertongue",
            &[Target::Player(p.other())],
        );
        settle(&mut g);
        assert_eq!(g.hand_count(p.other()), hand + 2);
        assert_eq!(g.state.players[p.other().index()].life, 16);
        let secret = put(&mut g, p, "Murder", Zone::Library);
        copied_prepared_spell(&mut g, p, "Emeritus of Woe", &[]);
        until_choice(&mut g);
        reject(&mut g, Action::FinishSearch);
        act(&mut g, Action::ChooseSearch { card: secret });
        assert!(g.observe(p.other()).objects.iter().all(|o| o.id != secret));
        act(&mut g, Action::FinishSearch);
        settle(&mut g);
        assert_eq!(g.object_view(secret).zone, Zone::Hand);
        assert!(g.observe(p.other()).objects.iter().all(|o| o.id != secret));
    }
}

#[test]
fn sos_prepared_red_spell_faces_mana_damage_treasure_spirits_and_nonbasic_removal() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let mana: u32 = g.state.players[p.index()].mana.iter().sum();
        copied_prepared_spell(&mut g, p, "Blazing Firesinger", &[]);
        settle(&mut g);
        assert_eq!(
            g.state.players[p.index()].mana.iter().sum::<u32>(),
            mana + 2
        );
        copied_prepared_spell(
            &mut g,
            p,
            "Emeritus of Conflict",
            &[Target::Player(p.other())],
        );
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 17);
        copied_prepared_spell(&mut g, p, "Goblin Glasswright", &[]);
        settle(&mut g);
        assert_eq!(
            g.state
                .objects
                .iter()
                .filter(|o| o.zone == Zone::Battlefield
                    && o.controller == p
                    && o.card.has_subtype("Treasure"))
                .count(),
            1
        );
        copied_prepared_spell(&mut g, p, "Strife Scholar", &[]);
        settle(&mut g);
        let spirit = Card::from_name("@token/sos-spirit").unwrap();
        assert_eq!(
            g.state
                .objects
                .iter()
                .filter(|o| o.zone == Zone::Battlefield && o.controller == p && o.card == spirit)
                .count(),
            2
        );
        let land = put(&mut g, p.other(), "Gruul Guildgate", Zone::Battlefield);
        let basic = put(&mut g, p.other(), "Mountain", Zone::Battlefield);
        let definition = Card::from_name("Maelstrom Artisan")
            .unwrap()
            .prepared_spell()
            .unwrap();
        let copy = g.allocate_card_copy(definition, p);
        g.grant_exile_play_permission(p, copy, false);
        act(&mut g, Action::AnnounceExile { card: copy.card });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(basic),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(land),
            },
        );
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(land).zone, Zone::Graveyard);
        assert_eq!(g.object_view(basic).zone, Zone::Battlefield);
    }
}

#[test]
fn sos_prepared_venomous_words_and_striking_palette_expire_and_copy_once() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        copied_prepared_spell(&mut g, p, "Scathing Shadelock", &[Target::Card(bear)]);
        settle(&mut g);
        assert_eq!(stats(&g, bear), (4, 2));
        assert!(g.keywords(bear).contains(Keywords::DEATHTOUCH));
        g.finish_cleanup();
        assert_eq!(stats(&g, bear), (2, 2));
        assert!(!g.keywords(bear).contains(Keywords::DEATHTOUCH));
        let mut g = scenario(p);
        copied_prepared_spell(&mut g, p, "Pigment Wrangler", &[]);
        settle(&mut g);
        let shock = put(&mut g, p, "Shock", Zone::Hand);
        cast(&mut g, shock, Some(Target::Player(p.other())));
        until_choice(&mut g);
        act(&mut g, Action::FinishStackTargets);
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 16);
        let shock = put(&mut g, p, "Shock", Zone::Hand);
        cast(&mut g, shock, Some(Target::Player(p.other())));
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 14);
    }
}

#[test]
fn sos_reanimate_opponent_card_control_and_frozen_mana_value_across_entry_choice() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let mare = put(&mut g, p.other(), "Diamond Mare", Zone::Graveyard);
        copied_prepared_spell(&mut g, p, "Grave Researcher", &[Target::Card(mare)]);
        until_choice(&mut g);
        assert_eq!(
            g.state.players[p.index()].life,
            20,
            "entry choice precedes Reanimate's life loss"
        );
        assert_eq!(g.actor(), Some(p));
        let choice = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::ChooseEntryValue { .. }))
            .unwrap();
        act(&mut g, choice);
        settle(&mut g);
        assert_eq!(g.object_view(mare).zone, Zone::Battlefield);
        assert_eq!(g.object_view(mare).controller, p);
        assert_eq!(g.object_view(mare).owner, p.other());
        assert_eq!(g.state.players[p.index()].life, 18);
        assert_eq!(g.state.players[p.other().index()].life, 20);
    }
}

#[test]
fn sos_graveyard_leave_consumers_create_spirits_and_attack_boost() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let excavator = put(&mut g, p, "Garrison Excavator", Zone::Battlefield);
        assert!(g.keywords(excavator).contains(Keywords::MENACE));
        let history = put(&mut g, p, "Living History", Zone::Hand);
        cast(&mut g, history, None);
        settle(&mut g);
        let spirit = Card::from_name("@token/sos-spirit").unwrap();
        let attacker = g
            .state
            .objects
            .iter()
            .enumerate()
            .find_map(|(i, o)| {
                (o.zone == Zone::Battlefield && o.card == spirit).then_some(i as CardId)
            })
            .unwrap();
        let grave = put(&mut g, p, "Bear Cub", Zone::Graveyard);
        copied_prepared_spell(&mut g, p, "Cheerful Osteomancer", &[Target::Card(grave)]);
        settle(&mut g);
        assert_eq!(
            g.state
                .objects
                .iter()
                .filter(|o| o.zone == Zone::Battlefield && o.card == spirit && o.controller == p)
                .count(),
            2
        );
        g.state.objects[attacker as usize].summoning_sick = false;
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        act(&mut g, Action::Attack { card: attacker });
        act(&mut g, Action::FinishAttackers);
        assert!(g.legal_actions().contains(&Action::ChooseTarget {
            target: Target::Card(attacker)
        }));
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(attacker),
            },
        );
        settle(&mut g);
        assert_eq!(stats(&g, attacker), (4, 2));
        g.finish_cleanup();
        assert_eq!(stats(&g, attacker), (2, 2));
    }
}

fn associated_prepared_copy(g: &Game, source: CardId) -> ObjectRef {
    g.observe(g.state.objects[source as usize].controller)
        .preparations
        .iter()
        .find(|row| row.permanent == g.object_ref(source))
        .unwrap()
        .spell_copy
}
fn cast_associated_prepared(g: &mut Game, source: CardId, targets: &[Target]) -> CardId {
    let copy = associated_prepared_copy(g, source);
    act(g, Action::AnnounceExile { card: copy.card });
    assert!(
        g.is_prepared(g.object_ref(source)),
        "announcement alone does not unprepare"
    );
    for &target in targets {
        act(g, Action::ChooseTarget { target });
    }
    pay(g);
    assert!(!g.is_prepared(g.object_ref(source)));
    copy.card
}

#[test]
fn sos_prepared_fronts_whole_bodies_keywords_entry_preparation_and_paid_copy_lifecycle() {
    let cases = [
        ("Adventurous Eater", 3, 2, true, Keywords::NONE),
        ("Blazing Firesinger", 2, 3, true, Keywords::NONE),
        ("Cheerful Osteomancer", 4, 2, true, Keywords::NONE),
        ("Emeritus of Conflict", 2, 2, false, Keywords::FIRST_STRIKE),
        ("Emeritus of Woe", 5, 4, true, Keywords::NONE),
        ("Goblin Glasswright", 2, 2, true, Keywords::NONE),
        ("Grave Researcher", 3, 3, false, Keywords::NONE),
        ("Leech Collector", 2, 2, false, Keywords::NONE),
        ("Maelstrom Artisan", 3, 2, true, Keywords::HASTE),
        ("Pigment Wrangler", 4, 4, true, Keywords::FLYING),
        ("Scathing Shadelock", 4, 6, false, Keywords::NONE),
        (
            "Scheming Silvertongue",
            1,
            3,
            false,
            Keywords::FLYING.union(Keywords::LIFELINK),
        ),
        ("Strife Scholar", 3, 2, true, Keywords::NONE),
    ];
    for p in [Player::P0, Player::P1] {
        for (name, power, toughness, prepared, keywords) in cases {
            let mut g = scenario(p);
            let source = put(&mut g, p, name, Zone::Hand);
            cast(&mut g, source, None);
            settle(&mut g);
            assert_eq!(stats(&g, source), (power, toughness), "{name}");
            assert_eq!(g.keywords(source), keywords, "{name}");
            assert_eq!(g.is_prepared(g.object_ref(source)), prepared, "{name}");
            assert_eq!(g.object_view(source).prepared, prepared, "{name}");
        }
        let mut g = scenario(p);
        let source = put(&mut g, p, "Adventurous Eater", Zone::Hand);
        cast(&mut g, source, None);
        settle(&mut g);
        let copy = cast_associated_prepared(&mut g, source, &[Target::Card(source)]);
        settle(&mut g);
        assert_eq!(stats(&g, source), (4, 3));
        assert_eq!(g.state.players[p.index()].life, 21);
        assert_eq!(g.object_view(copy).zone, Zone::Outside);
        assert_eq!(g.object_view(source).zone, Zone::Battlefield);
        let bounce = put(&mut g, p, "Unsummon", Zone::Hand);
        cast(&mut g, bounce, Some(Target::Card(source)));
        settle(&mut g);
        cast(&mut g, source, None);
        settle(&mut g);
        assert_eq!(stats(&g, source), (3, 2));
        assert!(g.is_prepared(g.object_ref(source)));
    }
}

#[test]
fn sos_conflict_prepares_on_third_cast_only_and_leech_on_first_life_event_only() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let conflict = put(&mut g, p, "Emeritus of Conflict", Zone::Hand);
        cast(&mut g, conflict, None);
        settle(&mut g);
        for ordinal in [2, 3] {
            let spell = put(&mut g, p, "Shock", Zone::Hand);
            cast(&mut g, spell, Some(Target::Player(p.other())));
            settle(&mut g);
            assert_eq!(g.is_prepared(g.object_ref(conflict)), ordinal == 3);
        }
        cast_associated_prepared(&mut g, conflict, &[Target::Player(p.other())]);
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 13);
        assert!(
            !g.is_prepared(g.object_ref(conflict)),
            "fourth spell does not reprepare"
        );

        let mut g = scenario(p);
        let leech = put(&mut g, p, "Leech Collector", Zone::Hand);
        cast(&mut g, leech, None);
        settle(&mut g);
        let angel = put(&mut g, p, "Inspiring Overseer", Zone::Hand);
        cast(&mut g, angel, None);
        settle(&mut g);
        assert!(g.is_prepared(g.object_ref(leech)));
        cast_associated_prepared(&mut g, leech, &[]);
        settle(&mut g);
        let angel = put(&mut g, p, "Inspiring Overseer", Zone::Hand);
        cast(&mut g, angel, None);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, 22);
        assert!(!g.is_prepared(g.object_ref(leech)));
    }
}

#[test]
fn sos_woe_requires_two_deaths_and_own_end_step_and_silvertongue_two_life_main2() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let woe = put(&mut g, p, "Emeritus of Woe", Zone::Hand);
        cast(&mut g, woe, None);
        settle(&mut g);
        cast_associated_prepared(&mut g, woe, &[]);
        until_choice(&mut g);
        choose(&mut g, |a| matches!(a, Action::ChooseSearch { .. }));
        act(&mut g, Action::FinishSearch);
        settle(&mut g);
        for (index, owner) in [p, p.other()].into_iter().enumerate() {
            let bear = put(&mut g, owner, "Bear Cub", Zone::Battlefield);
            let murder = put(&mut g, p, "Murder", Zone::Hand);
            cast(&mut g, murder, Some(Target::Card(bear)));
            settle(&mut g);
            assert_eq!(g.state.creatures_died_this_turn, index as u64 + 1);
            begin_paradigm_main(&mut g, p, Step::End);
            settle(&mut g);
            assert_eq!(g.is_prepared(g.object_ref(woe)), index == 1);
        }

        let mut g = scenario(p);
        let silver = put(&mut g, p, "Scheming Silvertongue", Zone::Hand);
        cast(&mut g, silver, None);
        settle(&mut g);
        for count in [1, 2] {
            let angel = put(&mut g, p, "Inspiring Overseer", Zone::Hand);
            cast(&mut g, angel, None);
            settle(&mut g);
            begin_paradigm_main(&mut g, p, Step::Main2);
            settle(&mut g);
            assert_eq!(g.is_prepared(g.object_ref(silver)), count == 2);
        }
    }
}

#[test]
fn sos_researcher_surveillance_precedes_threshold_and_shadelock_own_main1_only() {
    for p in [Player::P0, Player::P1] {
        for third_creature in [false, true] {
            let mut g = scenario(p);
            let researcher = put(&mut g, p, "Grave Researcher", Zone::Hand);
            cast(&mut g, researcher, None);
            settle(&mut g);
            put(&mut g, p, "Bear Cub", Zone::Graveyard);
            put(&mut g, p, "Bear Cub", Zone::Graveyard);
            let top = put(
                &mut g,
                p,
                if third_creature { "Bear Cub" } else { "Forest" },
                Zone::Library,
            );
            begin_paradigm_main(&mut g, p, Step::Upkeep);
            until_choice(&mut g);
            assert!(!g.is_prepared(g.object_ref(researcher)));
            act(
                &mut g,
                Action::ArrangeLibrary {
                    card: top,
                    destination: LibraryDestination::Graveyard,
                },
            );
            settle(&mut g);
            assert_eq!(g.is_prepared(g.object_ref(researcher)), third_creature);
        }
        let mut g = scenario(p);
        let source = put(&mut g, p, "Scathing Shadelock", Zone::Hand);
        cast(&mut g, source, None);
        settle(&mut g);
        begin_paradigm_main(&mut g, p.other(), Step::Main1);
        settle(&mut g);
        assert!(!g.is_prepared(g.object_ref(source)));
        begin_paradigm_main(&mut g, p, Step::Main2);
        settle(&mut g);
        assert!(!g.is_prepared(g.object_ref(source)));
        begin_paradigm_main(&mut g, p, Step::Main1);
        settle(&mut g);
        assert!(g.is_prepared(g.object_ref(source)));
    }
}

#[test]
fn sos_ancestral_anger_counts_only_own_named_grave_cards_on_resolution_and_fizzles_draw() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        put(&mut g, p, "Ancestral Anger", Zone::Graveyard);
        put(&mut g, p.other(), "Ancestral Anger", Zone::Graveyard);
        put(&mut g, p, "Shock", Zone::Graveyard);
        let hand = g.hand_count(p);
        for (cast_count, expected) in [(1, 4), (2, 7)] {
            let spell = put(&mut g, p, "Ancestral Anger", Zone::Hand);
            cast(&mut g, spell, Some(Target::Card(bear)));
            settle(&mut g);
            assert_eq!(stats(&g, bear), (expected, 2));
            assert_eq!(g.hand_count(p), hand + cast_count);
            assert!(g.keywords(bear).contains(Keywords::TRAMPLE));
        }
        g.finish_cleanup();
        assert_eq!(stats(&g, bear), (2, 2));
        assert!(!g.keywords(bear).contains(Keywords::TRAMPLE));
        let mut g = scenario(p);
        let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let spell = put(&mut g, p, "Ancestral Anger", Zone::Hand);
        let bounce = put(&mut g, p.other(), "Unsummon", Zone::Hand);
        cast(&mut g, spell, Some(Target::Card(bear)));
        act(&mut g, Action::Pass);
        cast(&mut g, bounce, Some(Target::Card(bear)));
        let library = g.state.players[p.index()].library.len();
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].library.len(), library);
    }
}

#[test]
fn sos_arnyn_frozen_power_or_toughness_death_and_self_departure() {
    for p in [Player::P0, Player::P1] {
        for (name, owner, qualifies) in [
            ("Dragonlord's Servant", p, true),
            ("Goblin Piker", p, true),
            ("Bear Cub", p, false),
            ("Dragonlord's Servant", p.other(), false),
        ] {
            let mut g = scenario(p);
            let arnyn = put(&mut g, p, "Arnyn, Deathbloom Botanist", Zone::Battlefield);
            assert!(g.keywords(arnyn).contains(Keywords::DEATHTOUCH));
            let victim = put(&mut g, owner, name, Zone::Battlefield);
            let murder = put(&mut g, p, "Murder", Zone::Hand);
            cast(&mut g, murder, Some(Target::Card(victim)));
            if qualifies {
                until_choice(&mut g);
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
            }
            settle(&mut g);
            assert_eq!(
                g.state.players[p.index()].life,
                if qualifies { 22 } else { 20 }
            );
            assert_eq!(
                g.state.players[p.other().index()].life,
                if qualifies { 18 } else { 20 }
            );
        }
        let mut g = scenario(p);
        let arnyn = put(&mut g, p, "Arnyn, Deathbloom Botanist", Zone::Battlefield);
        let shrink = put(&mut g, p, "Fleeting Distraction", Zone::Hand);
        cast(&mut g, shrink, Some(Target::Card(arnyn)));
        settle(&mut g);
        assert_eq!(stats(&g, arnyn), (1, 2));
        let murder = put(&mut g, p, "Murder", Zone::Hand);
        cast(&mut g, murder, Some(Target::Card(arnyn)));
        until_choice(&mut g);
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        settle(&mut g);
        assert_eq!(g.object_view(arnyn).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].life, 22);
        assert_eq!(g.state.players[p.other().index()].life, 18);
    }
}

#[test]
fn sos_magmablood_all_twobrid_payments_colorless_entry_and_cast_color_snapshot() {
    for p in [Player::P0, Player::P1] {
        for (cost, payment, counters) in [
            (0, [1, 1, 1, 1, 1, 1], 5),
            (0, [0, 0, 0, 0, 0, 6], 0),
            (1, [1, 1, 1, 1, 1, 0], 5),
            (2, [1, 1, 0, 2, 0, 0], 3),
            (3, [0, 0, 0, 3, 0, 0], 1),
        ] {
            let mut g = scenario(p);
            let source = put(&mut g, p, "Magmablood Archaic", Zone::Hand);
            assert_eq!(g.state.objects[source as usize].card.fixed_mana_value(), 6);
            act(&mut g, Action::AnnounceCast { card: source });
            act(&mut g, Action::ChooseSpellCost { cost });
            act(&mut g, Action::PayCosts { payment });
            settle(&mut g);
            assert_eq!(g.counter_count(source, CounterKind::PlusOne), counters);
            assert_eq!(
                stats(&g, source),
                (2 + i128::from(counters), 2 + i128::from(counters))
            );
            assert!(
                g.keywords(source)
                    .contains(Keywords::REACH.union(Keywords::TRAMPLE))
            );
        }
        let mut g = scenario(p);
        let source = put(&mut g, p, "Magmablood Archaic", Zone::Graveyard);
        copied_prepared_spell(&mut g, p, "Grave Researcher", &[Target::Card(source)]);
        settle(&mut g);
        assert_eq!(g.counter_count(source, CounterKind::PlusOne), 0);
        assert_eq!(g.state.players[p.index()].life, 14);
        let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let spell = put(&mut g, p, "Divination", Zone::Hand);
        act(&mut g, Action::AnnounceCast { card: spell });
        act(
            &mut g,
            Action::PayCosts {
                payment: [1, 1, 1, 0, 0, 0],
            },
        );
        settle(&mut g);
        assert_eq!(stats(&g, source), (5, 2));
        assert_eq!(stats(&g, bear), (5, 2));
        assert_eq!(stats(&g, enemy), (2, 2));
        let late = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        assert_eq!(stats(&g, late), (2, 2));
        g.finish_cleanup();
        assert_eq!(stats(&g, source), (2, 2));
        assert_eq!(stats(&g, bear), (2, 2));
    }
}

#[test]
fn sos_maestro_counts_before_live_power_mana_uses_payment_and_departure_lki() {
    for p in [Player::P0, Player::P1] {
        for bounce in [false, true] {
            let mut g = scenario(p);
            let source = put(&mut g, p, "Molten-Core Maestro", Zone::Battlefield);
            let bear = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let spell = put(&mut g, p, "Artistic Process", Zone::Hand);
            act(&mut g, Action::AnnounceCast { card: spell });
            act(&mut g, Action::ChooseMode { mode: 0 });
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(bear),
                },
            );
            pay(&mut g);
            let red = g.state.players[p.index()].mana[3];
            if bounce {
                let unsummon = put(&mut g, p.other(), "Unsummon", Zone::Hand);
                act(&mut g, Action::Pass);
                cast(&mut g, unsummon, Some(Target::Card(source)));
            }
            settle(&mut g);
            assert_eq!(
                g.state.players[p.index()].mana[3],
                red + if bounce { 2 } else { 3 }
            );
            assert_eq!(
                g.object_view(source).zone,
                if bounce {
                    Zone::Hand
                } else {
                    Zone::Battlefield
                }
            );
            assert_eq!(
                g.counter_count(source, CounterKind::PlusOne),
                if bounce { 0 } else { 1 }
            );
        }
        let mut g = scenario(p);
        let source = put(&mut g, p, "Molten-Core Maestro", Zone::Battlefield);
        put(&mut g, p, "Omniscience", Zone::Battlefield);
        let spell = put(&mut g, p, "Artistic Process", Zone::Hand);
        let bear = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let mana = g.state.players[p.index()].mana;
        act(
            &mut g,
            Action::AnnounceFree {
                card: spell,
                face: CastFace::Normal,
                flash_surcharge: false,
            },
        );
        act(&mut g, Action::ChooseMode { mode: 0 });
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bear),
            },
        );
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.counter_count(source, CounterKind::PlusOne), 1);
        assert_eq!(
            g.state.players[p.index()].mana,
            mana,
            "high mana value alone is not five mana spent"
        );
    }
}

#[test]
fn sos_ordinary_resolutions_do_not_allocate_paradigm_history() {
    let mut g = scenario(Player::P0);
    let bear = put(&mut g, Player::P0, "Bear Cub", Zone::Hand);
    cast(&mut g, bear, None);
    settle(&mut g);
    let shock = put(&mut g, Player::P0, "Shock", Zone::Hand);
    cast(&mut g, shock, Some(Target::Player(Player::P1)));
    settle(&mut g);
    assert!(g.state.paradigm.is_none());
    assert!(g.observe(Player::P0).resolved_spell_names.is_empty());
}

#[test]
fn sos_professor_graveyard_additional_cost_legality_reentry_and_attack_drain() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Postmortem Professor", Zone::Graveyard);
        let fuel = put(&mut g, p, "Shock", Zone::Graveyard);
        let wrong_type = put(&mut g, p, "Bear Cub", Zone::Graveyard);
        let wrong_owner = put(&mut g, p.other(), "Shock", Zone::Graveyard);
        act(
            &mut g,
            Action::AnnounceActivate {
                card: source,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        for id in [wrong_type, wrong_owner] {
            let object = g.object_ref(id);
            reject(&mut g, Action::ChooseAdditionalCost { object });
        }
        let object = g.object_ref(fuel);
        act(&mut g, Action::ChooseAdditionalCost { object });
        pay(&mut g);
        assert_eq!(g.object_view(fuel).zone, Zone::Exile);
        assert_eq!(g.object_view(source).zone, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Battlefield);
        assert!(g.object_view(source).summoning_sick);
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        reject(&mut g, Action::Attack { card: source });
        g.state.objects[source as usize].summoning_sick = false;
        act(&mut g, Action::Attack { card: source });
        act(&mut g, Action::FinishAttackers);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, 21);
        assert_eq!(g.state.players[p.other().index()].life, 19);
    }
}

#[test]
fn sos_professor_cannot_block_is_a_removable_printed_ability() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p.other());
        let source = put(&mut g, p, "Postmortem Professor", Zone::Battlefield);
        let attacker = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        assert!(!g.can_block_pair(source, attacker));
        let protection = put(&mut g, p.other(), "Witness Protection", Zone::Hand);
        cast(&mut g, protection, Some(Target::Card(source)));
        settle(&mut g);
        assert!(g.can_block_pair(source, attacker));
    }
}

#[test]
fn sos_tragedy_ward_discard_payment_decline_and_empty_hand() {
    for p in [Player::P0, Player::P1] {
        for (have_card, discard) in [(true, true), (true, false), (false, false)] {
            let mut g = scenario(p);
            let source = put(&mut g, p, "Tragedy Feaster", Zone::Battlefield);
            let fuel = have_card.then(|| put(&mut g, p.other(), "Shock", Zone::Hand));
            let murder = put(&mut g, p.other(), "Murder", Zone::Hand);
            act(&mut g, Action::Pass);
            cast(&mut g, murder, Some(Target::Card(source)));
            if let Some(fuel) = fuel {
                until_choice(&mut g);
                assert_eq!(g.actor(), Some(p.other()));
                assert!(
                    g.observe(p).effect_choice.is_none(),
                    "ward discard identities remain private"
                );
                act(
                    &mut g,
                    if discard {
                        Action::Discard { card: fuel }
                    } else {
                        Action::DeclineEffectPayment
                    },
                );
            }
            settle(&mut g);
            assert_eq!(
                g.object_view(source).zone,
                if discard {
                    Zone::Graveyard
                } else {
                    Zone::Battlefield
                }
            );
            if let Some(fuel) = fuel {
                assert_eq!(
                    g.object_view(fuel).zone,
                    if discard { Zone::Graveyard } else { Zone::Hand }
                );
            }
        }
    }
}

#[test]
fn sos_tragedy_checks_life_gain_when_end_trigger_resolves_and_can_sacrifice_self() {
    for p in [Player::P0, Player::P1] {
        for gain_in_response in [false, true] {
            let mut g = scenario(p);
            let source = put(&mut g, p, "Tragedy Feaster", Zone::Battlefield);
            assert_eq!(stats(&g, source), (7, 6));
            assert!(g.keywords(source).contains(Keywords::TRAMPLE));
            begin_paradigm_main(&mut g, p, Step::End);
            if gain_in_response {
                let spell = put(&mut g, p, "Dissection Practice", Zone::Hand);
                act(&mut g, Action::AnnounceCast { card: spell });
                act(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Player(p.other()),
                    },
                );
                act(&mut g, Action::FinishTargets);
                act(&mut g, Action::FinishTargets);
                pay(&mut g);
                settle(&mut g);
                assert_eq!(g.object_view(source).zone, Zone::Battlefield);
            } else {
                until_choice(&mut g);
                reject(&mut g, Action::DeclineEffectPayment);
                let object = g.object_ref(source);
                act(&mut g, Action::Sacrifice { object });
                settle(&mut g);
                assert_eq!(g.object_view(source).zone, Zone::Graveyard);
            }
        }
    }
}

#[test]
fn sos_duel_tactics_damage_block_restriction_survives_ability_removal_flashback_and_cleanup() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let blocker = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let attacker = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let growth = put(&mut g, p, "Giant Growth", Zone::Hand);
        cast(&mut g, growth, Some(Target::Card(blocker)));
        settle(&mut g);
        let spell = put(&mut g, p, "Duel Tactics", Zone::Hand);
        cast(&mut g, spell, Some(Target::Card(blocker)));
        settle(&mut g);
        assert_eq!(g.object_view(blocker).damage, 1);
        assert!(!g.can_block_pair(blocker, attacker));
        let protection = put(&mut g, p, "Witness Protection", Zone::Hand);
        cast(&mut g, protection, Some(Target::Card(blocker)));
        settle(&mut g);
        assert!(
            !g.can_block_pair(blocker, attacker),
            "resolution restriction is not an ability grant"
        );
        g.finish_cleanup();
        assert_eq!(g.object_view(blocker).damage, 0);
        assert!(!g.keywords(blocker).contains(Keywords::CANT_BLOCK));
        let mut g = scenario(p);
        let attacker = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let spell = put(&mut g, p, "Duel Tactics", Zone::Graveyard);
        let target = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        act(&mut g, Action::AnnounceFlashback { card: spell });
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(spell).zone, Zone::Exile);
        assert_eq!(g.object_view(target).damage, 1);
        assert!(!g.can_block_pair(target, attacker));
    }
}

fn gain_one_with_dissection(g: &mut Game, p: Player) {
    let spell = put(g, p, "Dissection Practice", Zone::Hand);
    act(g, Action::AnnounceCast { card: spell });
    act(
        g,
        Action::ChooseTarget {
            target: Target::Player(p.other()),
        },
    );
    while g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
    pay(g);
    settle(g);
}

#[test]
fn sos_foolish_fate_destroy_then_infusion_uses_target_controller_after_departure_or_failed_destroy()
{
    for p in [Player::P0, Player::P1] {
        for (gained, indestructible, stolen) in [
            (false, false, false),
            (true, false, false),
            (true, true, false),
            (true, false, true),
        ] {
            let mut g = scenario(p);
            let target = put(
                &mut g,
                p.other(),
                "Bear Cub",
                if stolen {
                    Zone::Graveyard
                } else {
                    Zone::Battlefield
                },
            );
            if stolen {
                copied_prepared_spell(&mut g, p, "Grave Researcher", &[Target::Card(target)]);
                settle(&mut g);
                assert_eq!(g.object_view(target).controller, p);
            }
            if gained {
                gain_one_with_dissection(&mut g, p);
            }
            let life = [g.state.players[0].life, g.state.players[1].life];
            let controller = g.object_view(target).controller;
            let spell = put(&mut g, p, "Foolish Fate", Zone::Hand);
            cast(&mut g, spell, Some(Target::Card(target)));
            if indestructible {
                let will = put(&mut g, p.other(), "Adamant Will", Zone::Hand);
                act(&mut g, Action::Pass);
                cast(&mut g, will, Some(Target::Card(target)));
            }
            settle(&mut g);
            assert_eq!(
                g.object_view(target).zone,
                if indestructible {
                    Zone::Battlefield
                } else {
                    Zone::Graveyard
                }
            );
            for player in [p, p.other()] {
                assert_eq!(
                    g.state.players[player.index()].life,
                    life[player.index()] - if gained && player == controller { 3 } else { 0 }
                );
            }
        }
    }
}

#[test]
fn sos_heated_argument_optional_own_grave_exile_damage_controller_and_no_fuel() {
    for p in [Player::P0, Player::P1] {
        for (has_fuel, pay_exile, stolen) in [
            (true, true, false),
            (true, false, false),
            (false, false, false),
            (true, true, true),
        ] {
            let mut g = scenario(p);
            let target = put(
                &mut g,
                p.other(),
                "Bear Cub",
                if stolen {
                    Zone::Graveyard
                } else {
                    Zone::Battlefield
                },
            );
            if stolen {
                copied_prepared_spell(&mut g, p, "Grave Researcher", &[Target::Card(target)]);
                settle(&mut g);
            }
            let fuel = has_fuel.then(|| put(&mut g, p, "Forest", Zone::Graveyard));
            let foreign = put(&mut g, p.other(), "Forest", Zone::Graveyard);
            let controller = g.object_view(target).controller;
            let life = [g.state.players[0].life, g.state.players[1].life];
            let spell = put(&mut g, p, "Heated Argument", Zone::Hand);
            cast(&mut g, spell, Some(Target::Card(target)));
            if let Some(fuel) = fuel {
                until_choice(&mut g);
                assert_eq!(
                    g.object_view(target).zone,
                    Zone::Battlefield,
                    "SBAs wait until the whole spell finishes"
                );
                assert_eq!(g.object_view(target).damage, 6);
                let object = g.object_ref(foreign);
                reject(&mut g, Action::Sacrifice { object });
                if pay_exile {
                    let object = g.object_ref(fuel);
                    act(&mut g, Action::Sacrifice { object });
                } else {
                    act(&mut g, Action::DeclineEffectPayment);
                }
            }
            settle(&mut g);
            assert_eq!(g.object_view(target).zone, Zone::Graveyard);
            if let Some(fuel) = fuel {
                assert_eq!(
                    g.object_view(fuel).zone,
                    if pay_exile {
                        Zone::Exile
                    } else {
                        Zone::Graveyard
                    }
                );
            }
            for player in [p, p.other()] {
                assert_eq!(
                    g.state.players[player.index()].life,
                    life[player.index()]
                        - if pay_exile && player == controller {
                            2
                        } else {
                            0
                        }
                );
            }
        }
    }
}

#[test]
fn sos_foolish_and_heated_all_invalid_target_skip_life_loss_and_resolution_payment() {
    for p in [Player::P0, Player::P1] {
        for name in ["Foolish Fate", "Heated Argument"] {
            let mut g = scenario(p);
            gain_one_with_dissection(&mut g, p);
            let target = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let fuel = put(&mut g, p, "Forest", Zone::Graveyard);
            let spell = put(&mut g, p, name, Zone::Hand);
            let bounce = put(&mut g, p.other(), "Unsummon", Zone::Hand);
            let life = [g.state.players[0].life, g.state.players[1].life];
            cast(&mut g, spell, Some(Target::Card(target)));
            act(&mut g, Action::Pass);
            cast(&mut g, bounce, Some(Target::Card(target)));
            settle(&mut g);
            assert_eq!(g.object_view(target).zone, Zone::Hand);
            assert_eq!(g.object_view(fuel).zone, Zone::Graveyard);
            assert_eq!([g.state.players[0].life, g.state.players[1].life], life);
        }
    }
}

fn announce_burrog(g: &mut Game, spell: CardId, source: CardId, target: Option<CardId>) {
    act(g, Action::AnnounceCast { card: spell });
    act(
        g,
        Action::ChooseTarget {
            target: Target::Card(source),
        },
    );
    if let Some(target) = target {
        act(
            g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
    } else if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
    pay(g);
}

#[test]
fn sos_burrog_first_cast_optional_damage_then_second_cast_of_same_physical_card() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let sphinx = put(&mut g, p, "Sphinx of Forgotten Lore", Zone::Battlefield);
        let source = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let target = put(&mut g, p.other(), "Dragonlord's Servant", Zone::Battlefield);
        let spell = put(&mut g, p, "Burrog Barrage", Zone::Hand);
        announce_burrog(&mut g, spell, source, Some(target));
        settle(&mut g);
        assert_eq!(stats(&g, source), (2, 2));
        assert_eq!(g.object_view(target).damage, 2);
        assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        act(&mut g, Action::Attack { card: sphinx });
        act(&mut g, Action::FinishAttackers);
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(spell),
            },
        );
        settle(&mut g);
        assert!(g.has_graveyard_cast(p, spell, GraveyardCastKind::Flashback));
        act(
            &mut g,
            Action::AnnounceGrantedFlashback {
                card: spell,
                flash_surcharge: false,
            },
        );
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(source),
            },
        );
        act(&mut g, Action::FinishTargets);
        pay(&mut g);
        settle(&mut g);
        assert_eq!(
            stats(&g, source),
            (3, 2),
            "the first cast of this physical card is another spell"
        );
        assert_eq!(
            g.object_view(target).damage,
            2,
            "second cast declined its optional target"
        );
        assert_eq!(g.object_view(spell).zone, Zone::Exile);
        g.finish_cleanup();
        assert_eq!(stats(&g, source), (2, 2));
    }
}

#[test]
fn sos_burrog_uncast_copy_counts_original_and_controller_change_preserves_caster() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p, "Mica, Reader of Ruins", Zone::Battlefield);
        let artifact = put(&mut g, p, "Goldvein Pick", Zone::Battlefield);
        let source = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let spell = put(&mut g, p, "Burrog Barrage", Zone::Hand);
        announce_burrog(&mut g, spell, source, None);
        until_choice(&mut g);
        let object = g.object_ref(artifact);
        act(&mut g, Action::Sacrifice { object });
        act(&mut g, Action::FinishStackTargets);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].instant_sorcery_cast_this_turn, 1);
        assert_eq!(
            stats(&g, source),
            (3, 2),
            "only the uncast copy counts the original as another spell"
        );

        let mut g = scenario(p);
        let source = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let spell = put(&mut g, p, "Burrog Barrage", Zone::Hand);
        announce_burrog(&mut g, spell, source, None);
        // Model a spell-control effect: its existing targets remain valid relative
        // to its new controller, and that player's history must not exclude this cast.
        g.state.objects[source as usize].controller = p.other();
        g.state.stack.last_mut().unwrap().controller = p.other();
        g.state.players[p.other().index()].instant_sorcery_cast_this_turn = 1;
        assert_eq!(g.state.stack.last().unwrap().cast.unwrap().caster, p);
        settle(&mut g);
        assert_eq!(stats(&g, source), (3, 2));
    }
}

#[test]
fn sos_moseo_entry_pest_end_step_dynamic_life_limit_and_departed_source() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Moseo, Vein's New Dean", Zone::Hand);
        cast(&mut g, source, None);
        settle(&mut g);
        assert_eq!(stats(&g, source), (2, 1));
        assert!(g.keywords(source).contains(Keywords::FLYING));
        assert_eq!(
            g.state
                .objects
                .iter()
                .filter(|o| o.zone == Zone::Battlefield
                    && o.card == Card::SosPestToken
                    && o.controller == p)
                .count(),
            1
        );
        let small = put(&mut g, p, "Bear Cub", Zone::Graveyard);
        let large = put(&mut g, p, "Hill Giant", Zone::Graveyard);
        let foreign = put(&mut g, p.other(), "Bear Cub", Zone::Graveyard);
        begin_paradigm_main(&mut g, p.other(), Step::End);
        assert!(
            g.state.stack.is_empty(),
            "opponent's end step does not trigger"
        );
        begin_paradigm_main(&mut g, p, Step::Main2);
        gain_one_with_dissection(&mut g, p);
        gain_one_with_dissection(&mut g, p);
        begin_paradigm_main(&mut g, p, Step::End);
        for illegal in [large, foreign] {
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(illegal),
                },
            );
        }
        big_helpers::checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(small),
            },
        );
        let murder = put(&mut g, p.other(), "Murder", Zone::Hand);
        if g.actor() != Some(p.other()) {
            act(&mut g, Action::Pass);
        }
        cast(&mut g, murder, Some(Target::Card(source)));
        settle(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Graveyard);
        assert_eq!(g.object_view(small).zone, Zone::Battlefield);
        assert_eq!(g.object_view(large).zone, Zone::Graveyard);
        assert_eq!(g.object_view(small).controller, p);

        let mut g = scenario(p);
        put(&mut g, p, "Moseo, Vein's New Dean", Zone::Battlefield);
        let small = put(&mut g, p, "Bear Cub", Zone::Graveyard);
        begin_paradigm_main(&mut g, p, Step::End);
        assert!(g.state.stack.is_empty(), "no life gain means no trigger");
        begin_paradigm_main(&mut g, p, Step::Main2);
        gain_one_with_dissection(&mut g, p);
        gain_one_with_dissection(&mut g, p);
        begin_paradigm_main(&mut g, p, Step::End);
        act(&mut g, Action::FinishTargets);
        settle(&mut g);
        assert_eq!(g.object_view(small).zone, Zone::Graveyard);
    }
}

#[test]
fn sos_end_hunt_opponent_chooses_current_highest_ties_ignoring_permanent_protection() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let low = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let high = put(&mut g, p.other(), "Progenitus", Zone::Battlefield);
        let spell = put(&mut g, p, "End of the Hunt", Zone::Hand);
        cast(&mut g, spell, Some(Target::Player(p.other())));
        until_choice(&mut g);
        assert_eq!(g.actor(), Some(p.other()));
        let object = g.object_ref(low);
        reject(&mut g, Action::Sacrifice { object });
        let object = g.object_ref(high);
        big_helpers::checked(&mut g, Action::Sacrifice { object });
        settle(&mut g);
        assert_eq!(g.object_view(high).zone, Zone::Exile);
        assert_eq!(g.object_view(low).zone, Zone::Battlefield);

        let mut g = scenario(p);
        let a = put(&mut g, p.other(), "Shivan Dragon", Zone::Battlefield);
        let b = put(&mut g, p.other(), "Shivan Dragon", Zone::Battlefield);
        let walker = put(&mut g, p.other(), "Chandra, Flameshaper", Zone::Battlefield);
        g.state.objects[walker as usize].counters.loyalty = 6;
        let spell = put(&mut g, p, "End of the Hunt", Zone::Hand);
        cast(&mut g, spell, Some(Target::Player(p.other())));
        until_choice(&mut g);
        for lower in [a, b] {
            let object = g.object_ref(lower);
            reject(&mut g, Action::Sacrifice { object });
        }
        let object = g.object_ref(walker);
        act(&mut g, Action::Sacrifice { object });
        settle(&mut g);
        assert_eq!(g.object_view(walker).zone, Zone::Exile);
        let spell = put(&mut g, p, "End of the Hunt", Zone::Hand);
        cast(&mut g, spell, Some(Target::Player(p.other())));
        until_choice(&mut g);
        let object = g.object_ref(b);
        act(&mut g, Action::Sacrifice { object });
        settle(&mut g);
        assert_eq!(g.object_view(a).zone, Zone::Battlefield);
        assert_eq!(g.object_view(b).zone, Zone::Exile);

        let mut g = scenario(p);
        let spell = put(&mut g, p, "End of the Hunt", Zone::Hand);
        cast(&mut g, spell, Some(Target::Player(p.other())));
        settle(&mut g);
        assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
    }
}

#[test]
fn sos_agony_colors_actual_excess_and_exact_next_turn_play_permission() {
    for p in [Player::P0, Player::P1] {
        for (prior, payment, exiled) in [
            (false, [1, 1, 1, 1, 1, 0], 2),
            (true, [1, 1, 1, 1, 1, 0], 4),
            (false, [0, 0, 0, 1, 0, 4], 0),
        ] {
            let mut g = scenario(p);
            let target = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
            if prior {
                let shock = put(&mut g, p, "Shock", Zone::Hand);
                cast(&mut g, shock, Some(Target::Card(target)));
                settle(&mut g);
            }
            let card = put(&mut g, p, "Bear Cub", Zone::Library);
            let spell = put(&mut g, p, "Archaic's Agony", Zone::Hand);
            let before = g.state.players[p.index()].library.len();
            act(&mut g, Action::AnnounceCast { card: spell });
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(target),
                },
            );
            act(&mut g, Action::PayCosts { payment });
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].library.len(), before - exiled);
            if exiled > 0 {
                assert_eq!(g.object_view(card).zone, Zone::Exile);
                assert!(g.can_play_exiled(p, card));
                assert!(!g.can_play_exiled(p.other(), card));
                g.finish_cleanup();
                assert!(g.can_play_exiled(p, card));
                g.finish_cleanup();
                assert!(g.can_play_exiled(p, card));
                g.state.players[p.index()].mana = [8; 6];
                begin_paradigm_main(&mut g, p, Step::Main1);
                act(&mut g, Action::AnnounceExile { card });
                pay(&mut g);
                settle(&mut g);
                assert_eq!(g.object_view(card).zone, Zone::Battlefield);
                g.finish_cleanup();
                assert!(g.state.exile_permissions.is_empty());
            }
        }
    }
}

#[test]
fn sos_joke_global_prevention_target_optional_fizzle_and_cleanup() {
    for p in [Player::P0, Player::P1] {
        for target_joke in [false, true] {
            let mut g = scenario(p);
            put(&mut g, p.other(), "Crystal Barricade", Zone::Battlefield);
            let target = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
            let joke = put(&mut g, p, "Impractical Joke", Zone::Hand);
            cast(&mut g, joke, target_joke.then_some(Target::Card(target)));
            settle(&mut g);
            assert!(g.observe(p).damage_prevention_prohibited);
            assert!(g.observe(p.other()).damage_prevention_prohibited);
            if !target_joke {
                let shock = put(&mut g, p, "Shock", Zone::Hand);
                cast(&mut g, shock, Some(Target::Card(target)));
                settle(&mut g);
                assert_eq!(g.object_view(target).damage, 2);
            } else {
                assert_eq!(g.object_view(target).zone, Zone::Graveyard);
            }
            g.finish_cleanup();
            assert!(!g.observe(p).damage_prevention_prohibited);
        }
        let mut g = scenario(p);
        let target = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let joke = put(&mut g, p, "Impractical Joke", Zone::Hand);
        cast(&mut g, joke, Some(Target::Card(target)));
        act(&mut g, Action::Pass);
        let bounce = put(&mut g, p.other(), "Unsummon", Zone::Hand);
        cast(&mut g, bounce, Some(Target::Card(target)));
        settle(&mut g);
        assert!(!g.observe(p).damage_prevention_prohibited);
    }
}

fn announce_steal(g: &mut Game, spell: CardId, mode: u16, targets: &[Target]) {
    act(g, Action::AnnounceCast { card: spell });
    act(g, Action::ChooseMode { mode });
    for &target in targets {
        act(g, Action::ChooseTarget { target });
    }
    pay(g);
}

#[test]
fn sos_steal_private_any_number_actual_discard_and_replacement_entry_draw() {
    for p in [Player::P0, Player::P1] {
        for (target_self, selected) in [(false, 0), (false, 2), (true, 2)] {
            let mut g = scenario(p);
            let recipient = if target_self { p } else { p.other() };
            let shock = put(&mut g, recipient, "Shock", Zone::Hand);
            let baloth = put(&mut g, recipient, "Wilt-Leaf Liege", Zone::Hand);
            let foreign = put(&mut g, recipient.other(), "Forest", Zone::Hand);
            let spell = put(&mut g, p, "Steal the Show", Zone::Hand);
            let library = g.state.players[recipient.index()].library.len();
            announce_steal(&mut g, spell, 0, &[Target::Player(recipient)]);
            until_choice(&mut g);
            assert_eq!(g.actor(), Some(recipient));
            assert!(g.observe(recipient.other()).effect_choice.is_none());
            reject(&mut g, Action::Discard { card: foreign });
            if selected > 0 {
                big_helpers::checked(&mut g, Action::Discard { card: shock });
                reject(&mut g, Action::Discard { card: shock });
                act(&mut g, Action::Discard { card: baloth });
                assert!(g.observe(recipient.other()).effect_choice.is_none());
            }
            act(&mut g, Action::DeclineEffectPayment);
            settle(&mut g);
            assert_eq!(
                g.state.players[recipient.index()].library.len(),
                library - selected
            );
            assert_eq!(
                g.state.players[recipient.index()].cards_discarded_this_turn,
                selected as u64
            );
            assert_eq!(
                g.object_view(shock).zone,
                if selected == 0 {
                    Zone::Hand
                } else {
                    Zone::Graveyard
                }
            );
            assert_eq!(
                g.object_view(baloth).zone,
                if selected == 0 {
                    Zone::Hand
                } else if target_self {
                    Zone::Graveyard
                } else {
                    Zone::Battlefield
                }
            );
            assert_eq!(g.state.players[recipient.index()].life, 20);
        }
    }
}

#[test]
fn sos_steal_both_uses_newly_discarded_instants_and_zero_hand_choice() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let discarded = put(&mut g, p, "Shock", Zone::Hand);
        put(&mut g, p, "Murder", Zone::Graveyard);
        put(&mut g, p.other(), "Shock", Zone::Graveyard);
        let target = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let spell = put(&mut g, p, "Steal the Show", Zone::Hand);
        announce_steal(&mut g, spell, 2, &[Target::Player(p), Target::Card(target)]);
        until_choice(&mut g);
        act(&mut g, Action::Discard { card: discarded });
        act(&mut g, Action::DeclineEffectPayment);
        settle(&mut g);
        assert_eq!(g.object_view(target).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);

        let mut g = scenario(p);
        let spell = put(&mut g, p, "Steal the Show", Zone::Hand);
        announce_steal(&mut g, spell, 0, &[Target::Player(p.other())]);
        until_choice(&mut g);
        assert!(g.observe(p.other()).effect_choice.is_some());
        assert!(g.observe(p).effect_choice.is_none());
        act(&mut g, Action::DeclineEffectPayment);
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].cards_drawn_this_turn, 0);

        let mut g = scenario(p);
        let target = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let spell = put(&mut g, p, "Steal the Show", Zone::Hand);
        put(&mut g, p, "Murder", Zone::Graveyard);
        announce_steal(&mut g, spell, 1, &[Target::Card(target)]);
        settle(&mut g);
        assert_eq!(g.object_view(target).damage, 1);
    }
}
