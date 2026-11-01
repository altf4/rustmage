//! Focused printed outcomes; shared engine invariants live in core tests.
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
    {
        let p = Player::P0;
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
    {
        let p = Player::P0;
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
    {
        let p = Player::P0;
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
    {
        let p = Player::P0;
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
    {
        let p = Player::P0;
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
    {
        let p = Player::P0;
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
fn sos_prepared_red_spell_faces_mana_damage_treasure_spirits_and_nonbasic_removal() {
    {
        let p = Player::P0;
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
    {
        let p = Player::P0;
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
    {
        let p = Player::P0;
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
    {
        let p = Player::P0;
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
fn sos_conflict_prepares_on_third_cast_only_and_leech_on_first_life_event_only() {
    {
        let p = Player::P0;
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
    {
        let p = Player::P0;
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
fn sos_professor_cannot_block_is_a_removable_printed_ability() {
    {
        let p = Player::P0;
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
    {
        let p = Player::P0;
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
fn sos_duel_tactics_damage_block_restriction_survives_ability_removal_flashback_and_cleanup() {
    {
        let p = Player::P0;
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
fn sos_heated_argument_optional_own_grave_exile_damage_controller_and_no_fuel() {
    {
        let p = Player::P0;
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
    {
        let p = Player::P0;
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
    {
        let p = Player::P0;
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
fn sos_end_hunt_opponent_chooses_current_highest_ties_ignoring_permanent_protection() {
    {
        let p = Player::P0;
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
    {
        let p = Player::P0;
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

fn announce_steal(g: &mut Game, spell: CardId, mode: u16, targets: &[Target]) {
    act(g, Action::AnnounceCast { card: spell });
    act(g, Action::ChooseMode { mode });
    for &target in targets {
        act(g, Action::ChooseTarget { target });
    }
    pay(g);
}

#[test]
fn sos_steal_both_uses_newly_discarded_instants_and_zero_hand_choice() {
    {
        let p = Player::P0;
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

#[test]
fn sos_pox_nonpositive_life_and_zero_quotas_do_not_gain_life_or_request_choices() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        put(&mut g, p, "Herald of Eternal Dawn", Zone::Battlefield);
        g.state.players[p.index()].life = -3;
        let spell = put(&mut g, p, "Pox Plague", Zone::Hand);
        cast(&mut g, spell, None);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, -3);
        assert_eq!(g.state.players[p.index()].life_gained_this_turn, 0);
        assert_eq!(g.state.players[p.other().index()].life, 10);
        assert!(g.outcome().is_none());
    }
}

#[test]
fn sos_tablet_milled_spell_play_and_both_mana_abilities_respect_restrictions() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let milled = put(&mut g, p, "Shock", Zone::Library);
        let tablet = put(&mut g, p, "Tablet of Discovery", Zone::Hand);
        cast(&mut g, tablet, None);
        settle(&mut g);
        assert_eq!(g.object_view(milled).zone, Zone::Graveyard);
        assert!(g.has_graveyard_cast(p, milled, GraveyardCastKind::Play));
        g.state.players[p.index()].mana = [0; 6];
        act(
            &mut g,
            Action::ActivateMana {
                card: tablet,
                ability: 1,
                payment: [0; 6],
                counter_payment: Counters::default(),
                production: [0, 0, 0, 2, 0, 0],
            },
        );
        let creature = put(&mut g, p, "Dragonlord's Servant", Zone::Hand);
        reject(&mut g, Action::AnnounceCast { card: creature });
        cast(&mut g, milled, Some(Target::Player(p.other())));
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 18);
        assert_eq!(g.state.players[p.index()].mana[3], 1);
        assert!(!g.has_graveyard_cast(p, milled, GraveyardCastKind::Play));
        g.untap_objects(&[tablet], untap_rules::UntapCause::Effect);
        act(
            &mut g,
            Action::ActivateMana {
                card: tablet,
                ability: 0,
                payment: [0; 6],
                counter_payment: Counters::default(),
                production: [0, 0, 0, 1, 0, 0],
            },
        );
        // The restricted red cannot fund this creature; ordinary red plus
        // colorless mana from the fixture can pay its full normal cost.
        g.state.players[p.index()].mana[5] = 1;
        let creature = put(&mut g, p, "Dragonlord's Servant", Zone::Hand);
        cast(&mut g, creature, None);
        settle(&mut g);
        assert_eq!(g.object_view(creature).zone, Zone::Battlefield);
    }
}

fn announce_rubble_mana(g: &mut Game, source: CardId) {
    act(
        g,
        Action::AnnounceMana {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
            production: [0, 0, 0, 1, 0, 0],
        },
    );
}

#[test]
fn sos_rubble_virtual_exile_fuel_is_consumed_once_and_token_graves_are_not_fuel() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let a = put(&mut g, p, "Rubble Rouser", Zone::Battlefield);
        let b = put(&mut g, p, "Rubble Rouser", Zone::Battlefield);
        let fuel = put(&mut g, p, "Forest", Zone::Graveyard);
        let spell = put(&mut g, p, "Lightning Strike", Zone::Hand);
        g.state.players[p.index()].mana = [0; 6];
        reject(&mut g, Action::AnnounceCast { card: spell });
        let second = put(&mut g, p, "Island", Zone::Graveyard);
        act(&mut g, Action::AnnounceCast { card: spell });
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        for (source, fuel) in [(a, fuel), (b, second)] {
            announce_rubble_mana(&mut g, source);
            let object = g.object_ref(fuel);
            act(&mut g, Action::ChooseAdditionalCost { object });
            pay(&mut g);
        }
        pay(&mut g);
        while matches!(g.state.pending, Pending::OrderTriggers { .. }) {
            choose(&mut g, |a| matches!(a, Action::PutTrigger { .. }));
        }
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 15);
        assert_eq!(g.object_view(fuel).zone, Zone::Exile);
        assert_eq!(g.object_view(second).zone, Zone::Exile);

        let mut g = scenario(p);
        let source = put(&mut g, p, "Rubble Rouser", Zone::Battlefield);
        put(&mut g, p, "@token/sos-pest", Zone::Graveyard);
        let spell = put(&mut g, p, "Shock", Zone::Hand);
        g.state.players[p.index()].mana = [0; 6];
        reject(&mut g, Action::AnnounceCast { card: spell });
        reject(
            &mut g,
            Action::AnnounceMana {
                card: source,
                ability: 0,
                counter_payment: Counters::default(),
                production: [0, 0, 0, 1, 0, 0],
            },
        );
    }
}

#[test]
fn sos_rubble_can_fund_using_a_physical_card_sacrificed_by_an_earlier_mana_step() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let source = put(&mut g, p, "Rubble Rouser", Zone::Battlefield);
        let pizza = put(&mut g, p, "Omni-Cheese Pizza", Zone::Battlefield);
        let spell = put(&mut g, p, "Lightning Strike", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        act(&mut g, Action::AnnounceCast { card: spell });
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        act(
            &mut g,
            Action::ActivateMana {
                card: pizza,
                ability: 0,
                payment: [0, 0, 0, 0, 0, 1],
                counter_payment: Counters::default(),
                production: [0, 0, 0, 1, 0, 0],
            },
        );
        assert_eq!(g.object_view(pizza).zone, Zone::Graveyard);
        announce_rubble_mana(&mut g, source);
        let object = g.object_ref(pizza);
        act(&mut g, Action::ChooseAdditionalCost { object });
        pay(&mut g);
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(pizza).zone, Zone::Exile);
        assert_eq!(g.state.players[p.other().index()].life, 16);
    }
}

fn activate_ral(g: &mut Game, source: CardId, ability: u16, targets: &[Target]) {
    act(
        g,
        Action::AnnounceActivate {
            card: source,
            ability,
            counter_payment: Counters::default(),
        },
    );
    for &target in targets {
        act(g, Action::ChooseTarget { target });
    }
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
    pay(g);
}

fn coin_heads(seed: u64) -> u64 {
    let mut rng = Rng(seed);
    (0..5).filter(|_| rng.below(2) == 1).count() as u64
}

#[test]
fn sos_ral_coins_owned_replay_and_actual_turn_skips_omit_upkeep_and_turn_numbers() {
    let zero = (0..1000).find(|&seed| coin_heads(seed) == 0).unwrap();
    let five = (0..1000).find(|&seed| coin_heads(seed) == 5).unwrap();
    {
        let p = Player::P0;
        for seed in [zero, five, 20261023] {
            let mut g = scenario(p);
            let source = put(&mut g, p, "Ral Zarek, Guest Lecturer", Zone::Battlefield);
            g.state.objects[source as usize].counters.loyalty = 7;
            put(&mut g, p.other(), "Phyrexian Arena", Zone::Battlefield);
            g.state.rng = Rng(seed);
            let heads = coin_heads(seed);
            activate_ral(&mut g, source, 3, &[Target::Player(p.other())]);
            assert_eq!(
                g.object_view(source).zone,
                Zone::Graveyard,
                "loyalty cost removes the source before the ability resolves"
            );
            act(&mut g, Action::Pass);
            big_helpers::checked(&mut g, Action::Pass);
            assert_eq!(g.observe(p).players[p.other().index()].turns_to_skip, heads);
            assert_eq!(
                g.observe(p.other()).players[p.other().index()].turns_to_skip,
                heads
            );
            for remaining in (0..heads).rev() {
                let turn = g.state.turn;
                g.finish_cleanup();
                assert_eq!(g.state.active, p);
                assert_eq!(g.state.turn, turn + 1);
                assert_eq!(
                    g.observe(p).players[p.other().index()].turns_to_skip,
                    remaining
                );
                assert!(
                    g.state.pending_triggers.is_empty(),
                    "skipped upkeep never queues Arena"
                );
            }
            g.finish_cleanup();
            assert_eq!(g.state.active, p.other());
            assert_eq!(
                g.state.pending_triggers.len(),
                1,
                "the actual upkeep queues Arena"
            );
            assert!(g.state.turn_skips.is_none());
        }
    }
}

#[test]
fn sos_memory_vessel_waits_for_activator_actual_turn_across_skips_for_both_grantees() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let vessel = put(&mut g, p, "Memory Vessel", Zone::Battlefield);
        activate_ral(&mut g, vessel, 0, &[]);
        settle(&mut g);
        assert!(g.hand_play_prohibited());
        assert_eq!(g.state.exile_permissions.len(), 14);
        assert!(
            g.state
                .exile_permissions
                .iter()
                .all(|v| v.expiry == ExilePlayExpiry::StartOfNextTurn(p))
        );
        let mut skips = [0; 2];
        skips[p.index()] = 1;
        g.state.turn_skips = Some(Box::new(skips));
        g.finish_cleanup();
        assert_eq!(g.state.active, p.other());
        assert!(g.hand_play_prohibited());
        assert_eq!(g.state.exile_permissions.len(), 14);
        g.finish_cleanup();
        assert_eq!(
            g.state.active,
            p.other(),
            "activator's next candidate turn is skipped"
        );
        assert!(g.hand_play_prohibited());
        assert_eq!(g.state.exile_permissions.len(), 14);
        g.finish_cleanup();
        assert_eq!(g.state.active, p);
        assert!(!g.hand_play_prohibited());
        assert!(g.state.exile_permissions.is_empty());
    }
}

fn announce_sparks(g: &mut Game, spell: CardId, mode: u16, targets: &[CardId]) {
    act(g, Action::AnnounceCast { card: spell });
    act(g, Action::ChooseMode { mode });
    for &card in targets {
        act(
            g,
            Action::ChooseTarget {
                target: Target::Spell(card),
            },
        );
    }
    pay(g);
}

#[test]
fn sos_sparks_instant_copy_retargets_and_neither_copy_counts_as_cast() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let shock = put(&mut g, p, "Shock", Zone::Hand);
        cast(&mut g, shock, Some(Target::Player(p.other())));
        let sparks = put(&mut g, p, "Choreographed Sparks", Zone::Hand);
        announce_sparks(&mut g, sparks, 0, &[shock]);
        until_choice(&mut g);
        act(
            &mut g,
            Action::SetStackTarget {
                index: 0,
                target: Target::Player(p),
            },
        );
        big_helpers::checked(&mut g, Action::FinishStackTargets);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, 18);
        assert_eq!(g.state.players[p.other().index()].life, 18);
        assert_eq!(g.state.players[p.index()].instant_sorcery_cast_this_turn, 2);
    }
}

#[test]
fn sos_sparks_creature_copy_keeps_x_and_entry_replacements_but_is_not_token_creation() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        put(&mut g, p, "Doubling Season", Zone::Battlefield);
        let trudge = put(&mut g, p, "Slumbering Trudge", Zone::Hand);
        act(&mut g, Action::AnnounceCast { card: trudge });
        act(&mut g, Action::ChooseX { value: 1 });
        pay(&mut g);
        let sparks = put(&mut g, p, "Choreographed Sparks", Zone::Hand);
        announce_sparks(&mut g, sparks, 1, &[trudge]);
        act(&mut g, Action::Pass);
        act(&mut g, Action::Pass);
        act(&mut g, Action::Pass);
        big_helpers::checked(&mut g, Action::Pass);
        settle(&mut g);
        let creatures = g
            .state
            .objects
            .iter()
            .enumerate()
            .filter(|(_, o)| {
                o.zone == Zone::Battlefield
                    && o.card == Card::from_name("Slumbering Trudge").unwrap()
            })
            .map(|(id, _)| id as CardId)
            .collect::<Vec<_>>();
        assert_eq!(
            creatures.len(),
            2,
            "a resolving copied permanent spell is not token creation"
        );
        assert_eq!(
            creatures
                .iter()
                .filter(|&&id| g.state.objects[id as usize].is_token())
                .count(),
            1
        );
        for id in creatures {
            assert_eq!(stats(&g, id), (6, 6));
            assert_eq!(g.counter_count(id, CounterKind::Stun), 4);
            assert!(g.object_view(id).tapped);
        }
    }
}

#[test]
fn sos_sparks_targets_only_controlled_spells_and_survives_one_invalid_target() {
    {
        let p = Player::P0;
        let mut g = scenario(p.other());
        let foreign = put(&mut g, p.other(), "Bear Cub", Zone::Hand);
        cast(&mut g, foreign, None);
        act(&mut g, Action::Pass);
        let own = put(&mut g, p, "Spectral Sailor", Zone::Hand);
        cast(&mut g, own, None);
        let sparks = put(&mut g, p, "Choreographed Sparks", Zone::Hand);
        act(&mut g, Action::AnnounceCast { card: sparks });
        act(&mut g, Action::ChooseMode { mode: 1 });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Spell(foreign),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Spell(own),
            },
        );
        pay(&mut g);
        settle(&mut g);
        assert_eq!(
            g.state
                .objects
                .iter()
                .filter(|o| o.zone == Zone::Battlefield && o.is_token())
                .count(),
            1
        );

        let mut g = scenario(p);
        let bear = put(&mut g, p, "Bear Cub", Zone::Hand);
        cast(&mut g, bear, None);
        let shock = put(&mut g, p, "Shock", Zone::Hand);
        cast(&mut g, shock, Some(Target::Player(p.other())));
        let sparks = put(&mut g, p, "Choreographed Sparks", Zone::Hand);
        announce_sparks(&mut g, sparks, 2, &[shock, bear]);
        act(&mut g, Action::Pass);
        let negate = put(&mut g, p.other(), "Negate", Zone::Hand);
        cast(&mut g, negate, Some(Target::Spell(shock)));
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 20);
        assert_eq!(
            g.state
                .objects
                .iter()
                .filter(|o| o.zone == Zone::Battlefield && o.is_token())
                .count(),
            1
        );
    }
}

fn choose_cast_clause(g: &mut Game, kind: CastCostClause) {
    let clause = g
        .state
        .announcing
        .as_ref()
        .unwrap()
        .cast_costs
        .as_ref()
        .unwrap()
        .clauses
        .iter()
        .position(|r| !r.paid && r.clause == kind)
        .unwrap() as u16;
    act(g, Action::ChooseCastCost { clause });
}
fn pay_casualty(g: &mut Game, victim: CardId) {
    choose_cast_clause(g, CastCostClause::Casualty(0));
    let object = g.object_ref(victim);
    act(g, Action::PayCasualty { object });
}
fn casualty_shock(g: &mut Game, shock: CardId, victim: Option<CardId>, target: Player) {
    act(g, Action::AnnounceCast { card: shock });
    act(
        g,
        Action::ChooseCasualty {
            pay: victim.is_some(),
        },
    );
    act(
        g,
        Action::ChooseTarget {
            target: Target::Player(target),
        },
    );
    pay(g);
    if let Some(victim) = victim {
        pay_casualty(g, victim);
    }
}

#[test]
fn sos_silverquill_can_pay_itself_but_loses_the_on_cast_ability_before_triggering() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let silver = put(&mut g, p, "Silverquill, the Disputant", Zone::Battlefield);
        let shock = put(&mut g, p, "Shock", Zone::Hand);
        casualty_shock(&mut g, shock, Some(silver), p.other());
        settle(&mut g);
        assert_eq!(g.object_view(silver).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.other().index()].life, 18);
        assert!(g.state.stack_copy_data.is_none());
    }
}

#[test]
fn sos_silverquill_additional_sacrifices_are_disjoint_and_copies_do_not_pay_again() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        put(&mut g, p, "Silverquill, the Disputant", Zone::Battlefield);
        let first = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let second = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let spell = put(&mut g, p, "Corrupted Conviction", Zone::Hand);
        act(&mut g, Action::AnnounceCast { card: spell });
        act(&mut g, Action::ChooseCasualty { pay: true });
        pay(&mut g);
        pay_casualty(&mut g, first);
        choose_cast_clause(&mut g, CastCostClause::Additional);
        let object = g.object_ref(first);
        reject(&mut g, Action::ChooseAdditionalCost { object });
        let object = g.object_ref(second);
        act(&mut g, Action::ChooseAdditionalCost { object });
        settle(&mut g);
        assert_eq!(g.object_view(first).zone, Zone::Graveyard);
        assert_eq!(g.object_view(second).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 4);
        assert_eq!(g.state.players[p.index()].instant_sorcery_cast_this_turn, 1);
    }
}
#[test]
fn sos_silverquill_zero_power_foreign_creatures_and_cannot_copy_spell() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        put(&mut g, p, "Silverquill, the Disputant", Zone::Battlefield);
        let wall = put(&mut g, p, "Rune-Sealed Wall", Zone::Battlefield);
        let foreign = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let shock = put(&mut g, p, "Shock", Zone::Hand);
        act(&mut g, Action::AnnounceCast { card: shock });
        act(&mut g, Action::ChooseCasualty { pay: true });
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        pay(&mut g);
        choose_cast_clause(&mut g, CastCostClause::Casualty(0));
        for id in [wall, foreign] {
            let object = g.object_ref(id);
            reject(&mut g, Action::PayCasualty { object });
        }
        let object = g.object_ref(bear);
        act(&mut g, Action::PayCasualty { object });
        until_choice(&mut g);
        act(&mut g, Action::FinishStackTargets);
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 16);
        let mut g = scenario(p);
        put(&mut g, p, "Silverquill, the Disputant", Zone::Battlefield);
        let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let shock = put(&mut g, p, "Shock", Zone::Hand);
        casualty_shock(&mut g, shock, None, p.other());
        let sparks = put(&mut g, p, "Choreographed Sparks", Zone::Hand);
        act(&mut g, Action::AnnounceCast { card: sparks });
        act(&mut g, Action::ChooseMode { mode: 0 });
        act(&mut g, Action::ChooseCasualty { pay: true });
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Spell(shock),
            },
        );
        pay(&mut g);
        pay_casualty(&mut g, bear);
        until_choice(&mut g);
        assert_eq!(
            g.observe(p)
                .retarget
                .as_ref()
                .unwrap()
                .item
                .source
                .unwrap()
                .card,
            Card::Shock
        );
        act(&mut g, Action::FinishStackTargets);
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 16);
    }
}
#[test]
fn sos_silverquill_virtual_power_rejects_mana_sacrifice_that_removes_last_eligible_creature() {
    {
        let p = Player::P0;
        for prefunded in [false, true] {
            let mut g = scenario(p);
            let silver = put(&mut g, p, "Silverquill, the Disputant", Zone::Battlefield);
            g.state.objects[silver as usize].growth.power = -4;
            let wall = put(&mut g, p, "Rune-Sealed Wall", Zone::Battlefield);
            let arsenal = put(&mut g, p, "Machinist's Arsenal", Zone::Battlefield);
            let pizza = put(&mut g, p, "Omni-Cheese Pizza", Zone::Battlefield);
            act(
                &mut g,
                Action::AnnounceActivate {
                    card: arsenal,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(wall),
                },
            );
            pay(&mut g);
            settle(&mut g);
            g.set_counters(wall, Counters::of(CounterKind::MinusOne, 5));
            assert_eq!(stats(&g, wall).0, 1);
            g.state.players[p.index()].mana = [0, 0, 0, u32::from(prefunded), 0, 1];
            let action=g.legal_actions().into_iter().find(|a|matches!(a,Action::ActivateMana{card,ability:0,production,..}if *card==pizza && production[3]==1)).unwrap();
            let shock = put(&mut g, p, "Shock", Zone::Hand);
            act(&mut g, Action::AnnounceCast { card: shock });
            if !prefunded {
                reject(&mut g, Action::ChooseCasualty { pay: true });
                act(&mut g, Action::ChooseCasualty { pay: false });
            } else {
                act(&mut g, Action::ChooseCasualty { pay: true });
            }
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Player(p.other()),
                },
            );
            if prefunded {
                reject(&mut g, action);
            } else {
                act(&mut g, action);
            }
            pay(&mut g);
            if prefunded {
                pay_casualty(&mut g, wall);
                until_choice(&mut g);
                act(&mut g, Action::FinishStackTargets);
            }
            settle(&mut g);
            assert_eq!(
                g.object_view(wall).zone,
                if prefunded {
                    Zone::Graveyard
                } else {
                    Zone::Battlefield
                }
            );
            assert_eq!(
                g.state.players[p.other().index()].life,
                if prefunded { 16 } else { 18 }
            );
        }
    }
}
#[test]
fn sos_silverquill_casualty_supplies_forage_and_mana_cannot_reopen_between_clauses() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        put(&mut g, p, "Silverquill, the Disputant", Zone::Battlefield);
        let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let target = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let a = put(&mut g, p, "Forest", Zone::Graveyard);
        let b = put(&mut g, p, "Forest", Zone::Graveyard);
        let mountain = put(&mut g, p, "Mountain", Zone::Battlefield);
        g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 1];
        let spell = put(&mut g, p, "Feed the Cycle", Zone::Hand);
        act(&mut g, Action::AnnounceCast { card: spell });
        act(
            &mut g,
            Action::ChooseForageRoute {
                route: ForageRoute::ExileThree,
            },
        );
        reject(&mut g, Action::ChooseCasualty { pay: false });
        act(&mut g, Action::ChooseCasualty { pay: true });
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        pay(&mut g);
        let base = g
            .state
            .announcing
            .as_ref()
            .unwrap()
            .cast_costs
            .as_ref()
            .unwrap()
            .clauses
            .iter()
            .position(|r| r.clause == CastCostClause::Additional)
            .unwrap() as u16;
        reject(&mut g, Action::ChooseCastCost { clause: base });
        reject(&mut g, Action::TapMana { card: mountain });
        pay_casualty(&mut g, bear);
        choose_cast_clause(&mut g, CastCostClause::Additional);
        for card in [a, b, bear] {
            let object = g.object_ref(card);
            act(&mut g, Action::ChooseForage { object });
        }
        until_choice(&mut g);
        act(&mut g, Action::FinishStackTargets);
        settle(&mut g);
        assert_eq!(g.object_view(bear).zone, Zone::Exile);
        assert_eq!(g.object_view(target).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].instant_sorcery_cast_this_turn, 1);
    }
}
#[test]
fn sos_silverquill_casualty_supplies_evidence_before_the_spell_becomes_cast() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let silver = put(&mut g, p, "Silverquill, the Disputant", Zone::Battlefield);
        let victim = put(&mut g, p, "Rune-Scarred Demon", Zone::Battlefield);
        let target = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let spell = put(&mut g, p, "Bite Down on Crime", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 1];
        act(&mut g, Action::AnnounceCast { card: spell });
        act(&mut g, Action::ChooseCollectEvidence { collect: true });
        act(&mut g, Action::ChooseCasualty { pay: true });
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(silver),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        pay(&mut g);
        pay_casualty(&mut g, victim);
        assert_eq!(g.state.players[p.index()].instant_sorcery_cast_this_turn, 0);
        choose_cast_clause(&mut g, CastCostClause::Additional);
        let object = g.object_ref(victim);
        big_helpers::checked(&mut g, Action::ChooseEvidence { object });
        big_helpers::checked(&mut g, Action::FinishEvidence);
        until_choice(&mut g);
        act(&mut g, Action::FinishStackTargets);
        settle(&mut g);
        assert_eq!(g.object_view(victim).zone, Zone::Exile);
        assert_eq!(g.object_view(target).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].instant_sorcery_cast_this_turn, 1);
    }
}
#[test]
fn sos_silverquill_tap_cost_must_precede_sacrificing_a_tapped_member() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let silver = put(&mut g, p, "Silverquill, the Disputant", Zone::Battlefield);
        let a = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let b = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let spell = put(&mut g, p, "Group Project", Zone::Graveyard);
        act(&mut g, Action::AnnounceFlashback { card: spell });
        act(&mut g, Action::ChooseCasualty { pay: true });
        pay(&mut g);
        let casualty = g
            .state
            .announcing
            .as_ref()
            .unwrap()
            .cast_costs
            .as_ref()
            .unwrap()
            .clauses
            .iter()
            .position(|r| r.clause == CastCostClause::Casualty(0))
            .unwrap() as u16;
        reject(&mut g, Action::ChooseCastCost { clause: casualty });
        choose_cast_clause(&mut g, CastCostClause::Additional);
        for card in [silver, a, b] {
            let object = g.object_ref(card);
            act(&mut g, Action::ChooseAdditionalCost { object });
        }
        assert!(g.object_view(a).tapped);
        pay_casualty(&mut g, a);
        settle(&mut g);
        assert_eq!(g.object_view(a).zone, Zone::Graveyard);
        assert_eq!(g.object_view(spell).zone, Zone::Exile);
        assert_eq!(
            g.state
                .objects
                .iter()
                .filter(|o| o.zone == Zone::Battlefield && o.card == Card::SosSpiritToken)
                .count(),
            2
        );
    }
}

fn silverquill_exile_return_setup(p: Player, name: &str) -> (Game, CardId, CardId, CardId) {
    let mut g = scenario(p);
    let silver = put(&mut g, p, "Silverquill, the Disputant", Zone::Battlefield);
    g.state.objects[silver as usize].growth.power = -4;
    let skeleton = put(&mut g, p, "Reassembling Skeleton", Zone::Battlefield);
    g.state.objects[skeleton as usize].growth.power = -1;
    let returning = put(&mut g, p, name, Zone::Battlefield);
    g.record_control(g.object_ref(returning), p.other(), None);
    let prison = put(&mut g, p, "Banishing Light", Zone::Hand);
    cast(&mut g, prison, None);
    until_choice(&mut g);
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(returning),
        },
    );
    settle(&mut g);
    assert_eq!(g.object_view(returning).zone, Zone::Exile);
    assert_eq!(stats(&g, skeleton).0, 0);
    (g, prison, skeleton, returning)
}
#[test]
fn sos_silverquill_cost_returned_lord_changes_later_casualty_power() {
    {
        let p = Player::P0;
        let (mut g, prison, skeleton, baron) = silverquill_exile_return_setup(p, "Death Baron");
        let target = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let spell = put(&mut g, p, "Candy Grapple", Zone::Hand);
        act(&mut g, Action::AnnounceCast { card: spell });
        let sacrifice = Some(g.object_ref(prison));
        act(&mut g, Action::ChooseBargain { sacrifice });
        act(&mut g, Action::ChooseCasualty { pay: true });
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        pay(&mut g);
        choose_cast_clause(&mut g, CastCostClause::Additional);
        assert_eq!(g.object_view(baron).zone, Zone::Battlefield);
        assert_eq!(stats(&g, skeleton).0, 1);
        pay_casualty(&mut g, skeleton);
        until_choice(&mut g);
        act(&mut g, Action::FinishStackTargets);
        settle(&mut g);
        assert_eq!(g.object_view(skeleton).zone, Zone::Graveyard);
        assert_eq!(g.object_view(prison).zone, Zone::Graveyard);
        assert_eq!(g.object_view(target).zone, Zone::Graveyard);
    }
}
#[test]
fn sos_silverquill_returned_banner_choice_cannot_strand_declared_casualty() {
    {
        let p = Player::P0;
        let (mut g, prison, skeleton, banner) =
            silverquill_exile_return_setup(p, "Banner of Kinship");
        let target = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let spell = put(&mut g, p, "Candy Grapple", Zone::Hand);
        act(&mut g, Action::AnnounceCast { card: spell });
        let sacrifice = Some(g.object_ref(prison));
        act(&mut g, Action::ChooseBargain { sacrifice });
        act(&mut g, Action::ChooseCasualty { pay: true });
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        pay(&mut g);
        choose_cast_clause(&mut g, CastCostClause::Additional);
        assert!(matches!(g.state.pending, Pending::EntryValue { .. }));
        let shark =
            EntryChoiceValue::CreatureType(crate::cards::CreatureType::from_name("Shark").unwrap());
        reject(&mut g, Action::ChooseEntryValue { value: shark });
        let skeleton_type = EntryChoiceValue::CreatureType(
            crate::cards::CreatureType::from_name("Skeleton").unwrap(),
        );
        super::big_helpers::checked(
            &mut g,
            Action::ChooseEntryValue {
                value: skeleton_type,
            },
        );
        assert_eq!(g.object_view(banner).zone, Zone::Battlefield);
        assert_eq!(stats(&g, skeleton).0, 1);
        pay_casualty(&mut g, skeleton);
        until_choice(&mut g);
        act(&mut g, Action::FinishStackTargets);
        settle(&mut g);
        assert_eq!(g.object_view(skeleton).zone, Zone::Graveyard);
    }
}

#[test]
fn sos_silverquill_casualty_can_return_a_creature_before_convoking_it() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let silver = put(&mut g, p, "Silverquill, the Disputant", Zone::Battlefield);
        let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.record_control(g.object_ref(bear), p.other(), None);
        let chariot = put(&mut g, p, "Detention Chariot", Zone::Hand);
        cast(&mut g, chariot, None);
        until_choice(&mut g);
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bear),
            },
        );
        settle(&mut g);
        assert_eq!(g.object_view(bear).zone, Zone::Exile);
        act(
            &mut g,
            Action::AnnounceActivate {
                card: chariot,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        let object = g.object_ref(silver);
        act(&mut g, Action::ChooseCrewer { object });
        act(&mut g, Action::FinishCrew);
        settle(&mut g);
        assert!(g.object_view(silver).tapped);
        g.state.objects[chariot as usize].tapped = true;
        let target = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let spell = put(&mut g, p, "Vote Out", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 2];
        act(&mut g, Action::AnnounceCast { card: spell });
        act(&mut g, Action::ChooseCasualty { pay: true });
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        super::big_helpers::checked(&mut g, Action::BeginConvoke);
        let costs = g
            .state
            .announcing
            .as_ref()
            .unwrap()
            .cast_costs
            .as_ref()
            .unwrap();
        assert!(costs.mana_closed);
        assert!(!costs.mana_paid);
        pay_casualty(&mut g, chariot);
        assert_eq!(g.object_view(bear).zone, Zone::Battlefield);
        assert!(!g.object_view(bear).tapped);
        choose_cast_clause(&mut g, CastCostClause::Mana);
        let object = g.object_ref(bear);
        super::big_helpers::checked(
            &mut g,
            Action::ChooseConvoker {
                object,
                contribution: ConvokeContribution::Generic,
            },
        );
        assert!(!g.object_view(bear).tapped);
        pay(&mut g);
        assert!(g.object_view(bear).tapped);
        until_choice(&mut g);
        act(&mut g, Action::FinishStackTargets);
        settle(&mut g);
        assert_eq!(g.object_view(chariot).zone, Zone::Graveyard);
        assert_eq!(g.object_view(target).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}

#[test]
fn sos_silverquill_nested_mana_keeps_the_original_casualty_obligation() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let silver = put(&mut g, p, "Silverquill, the Disputant", Zone::Battlefield);
        g.state.objects[silver as usize].growth.power = -4;
        let rubble = put(&mut g, p, "Rubble Rouser", Zone::Battlefield);
        g.state.objects[rubble as usize].growth.power = -1;
        let goyf = put(&mut g, p, "Tarmogoyf", Zone::Battlefield);
        g.state.objects[goyf as usize].growth.power = -1;
        let bear = put(&mut g, p, "Bear Cub", Zone::Graveyard);
        let forest = put(&mut g, p, "Forest", Zone::Graveyard);
        put(&mut g, p, "Forest", Zone::Graveyard);
        assert_eq!(stats(&g, goyf).0, 1);
        g.state.players[p.index()].mana = [0; 6];
        let shock = put(&mut g, p, "Shock", Zone::Hand);
        act(&mut g, Action::AnnounceCast { card: shock });
        act(&mut g, Action::ChooseCasualty { pay: true });
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        announce_rubble_mana(&mut g, rubble);
        let object = g.object_ref(bear);
        reject(&mut g, Action::ChooseAdditionalCost { object });
        let object = g.object_ref(forest);
        super::big_helpers::checked(&mut g, Action::ChooseAdditionalCost { object });
        pay(&mut g);
        assert_eq!(stats(&g, goyf).0, 1);
        pay(&mut g);
        pay_casualty(&mut g, goyf);
        while matches!(g.state.pending, Pending::OrderTriggers { .. }) {
            choose(&mut g, |a| matches!(a, Action::PutTrigger { .. }));
        }
        until_choice(&mut g);
        act(&mut g, Action::FinishStackTargets);
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 15);
        assert_eq!(g.object_view(bear).zone, Zone::Graveyard);
        assert_eq!(g.object_view(forest).zone, Zone::Exile);
    }
}

#[test]
fn sos_silverquill_life_payment_order_uses_projected_layered_power() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let silver = put(&mut g, p, "Silverquill, the Disputant", Zone::Battlefield);
        g.state.objects[silver as usize].growth.power = -4;
        let wall = put(&mut g, p, "Rune-Sealed Wall", Zone::Battlefield);
        g.state.objects[wall as usize].growth.power = -1;
        let sword = put(&mut g, p, "Aettir and Priwen", Zone::Battlefield);
        act(
            &mut g,
            Action::AnnounceActivate {
                card: sword,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(wall),
            },
        );
        pay(&mut g);
        settle(&mut g);
        g.state.players[p.index()].life = 2;
        assert_eq!(stats(&g, wall).0, 1);
        let shock = put(&mut g, p, "Shock", Zone::Hand);
        act(&mut g, Action::AnnounceCast { card: shock });
        g.state
            .announcing
            .as_mut()
            .unwrap()
            .add_cast_cost_clause(CastCostClause::Life(1));
        act(&mut g, Action::ChooseCasualty { pay: true });
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        pay(&mut g);
        let index = g
            .state
            .announcing
            .as_ref()
            .unwrap()
            .cast_costs
            .as_ref()
            .unwrap()
            .clauses
            .iter()
            .position(|r| r.clause == CastCostClause::Life(1))
            .unwrap() as u16;
        reject(&mut g, Action::ChooseCastCost { clause: index });
        pay_casualty(&mut g, wall);
        choose_cast_clause(&mut g, CastCostClause::Life(1));
        until_choice(&mut g);
        act(&mut g, Action::FinishStackTargets);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, 1);
        assert_eq!(g.state.players[p.other().index()].life, 16);
    }
}

#[test]
fn sos_silverquill_mana_counter_cost_cannot_remove_the_last_casualty_power() {
    {
        let p = Player::P0;
        for prefunded in [false, true] {
            let mut g = scenario(p);
            let silver = put(&mut g, p, "Silverquill, the Disputant", Zone::Battlefield);
            g.state.objects[silver as usize].growth.power = -4;
            let ramos = put(&mut g, p, "Ramos, Dragon Engine", Zone::Battlefield);
            g.state.objects[ramos as usize].growth.power = -4;
            g.set_counters(ramos, Counters::of(CounterKind::PlusOne, 5));
            let action = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a,Action::ActivateMana{card,..}if *card==ramos))
                .unwrap();
            let shock = put(&mut g, p, "Shock", Zone::Hand);
            g.state.players[p.index()].mana = [0, 0, 0, u32::from(prefunded), 0, 0];
            act(&mut g, Action::AnnounceCast { card: shock });
            if !prefunded {
                reject(&mut g, Action::ChooseCasualty { pay: true });
                act(&mut g, Action::ChooseCasualty { pay: false });
            } else {
                act(&mut g, Action::ChooseCasualty { pay: true });
            }
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Player(p.other()),
                },
            );
            if prefunded {
                reject(&mut g, action);
            } else {
                act(&mut g, action);
            }
            pay(&mut g);
            if prefunded {
                pay_casualty(&mut g, ramos);
                until_choice(&mut g);
                act(&mut g, Action::FinishStackTargets);
            }
            settle(&mut g);
            assert_eq!(
                g.state.players[p.other().index()].life,
                if prefunded { 16 } else { 18 }
            );
            assert_eq!(
                g.object_view(ramos).zone,
                if prefunded {
                    Zone::Graveyard
                } else {
                    Zone::Battlefield
                }
            );
        }
    }
}

#[test]
fn sos_silverquill_returned_aura_attachment_uses_projected_layers() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let silver = put(&mut g, p, "Silverquill, the Disputant", Zone::Battlefield);
        g.state.objects[silver as usize].growth.power = -4;
        let victim = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.state.objects[victim as usize].growth.power = -1;
        let other = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let aura = put(&mut g, p, "Witness Protection", Zone::Hand);
        cast(&mut g, aura, Some(Target::Card(other)));
        settle(&mut g);
        g.record_control(g.object_ref(aura), p.other(), None);
        let prison = put(&mut g, p, "Banishing Light", Zone::Hand);
        cast(&mut g, prison, None);
        until_choice(&mut g);
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(aura),
            },
        );
        settle(&mut g);
        assert_eq!(g.object_view(aura).zone, Zone::Exile);
        let spell = put(&mut g, p, "Candy Grapple", Zone::Hand);
        act(&mut g, Action::AnnounceCast { card: spell });
        let sacrifice = Some(g.object_ref(prison));
        act(&mut g, Action::ChooseBargain { sacrifice });
        act(&mut g, Action::ChooseCasualty { pay: true });
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(other),
            },
        );
        pay(&mut g);
        choose_cast_clause(&mut g, CastCostClause::Additional);
        assert!(matches!(g.state.pending, Pending::AuraEntry { .. }));
        let object = g.object_ref(victim);
        reject(&mut g, Action::ChooseAuraAttachment { object });
        let object = g.object_ref(other);
        super::big_helpers::checked(&mut g, Action::ChooseAuraAttachment { object });
        assert_eq!(stats(&g, victim).0, 1);
        assert_eq!(stats(&g, other), (1, 1));
        pay_casualty(&mut g, victim);
        until_choice(&mut g);
        act(&mut g, Action::FinishStackTargets);
        settle(&mut g);
        assert_eq!(g.object_view(victim).zone, Zone::Graveyard);
        assert_eq!(g.object_view(other).zone, Zone::Graveyard);
    }
}

#[test]
fn sos_silverquill_control_aura_must_be_sacrificed_after_its_stolen_casualty_victim() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let silver = put(&mut g, p, "Silverquill, the Disputant", Zone::Battlefield);
        g.state.objects[silver as usize].growth.power = -4;
        let stolen = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let confiscate = put(&mut g, p, "Confiscate", Zone::Hand);
        cast(&mut g, confiscate, Some(Target::Card(stolen)));
        settle(&mut g);
        assert_eq!(g.object_view(stolen).controller, p);
        let target = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let spell = put(&mut g, p, "Candy Grapple", Zone::Hand);
        act(&mut g, Action::AnnounceCast { card: spell });
        let sacrifice = Some(g.object_ref(confiscate));
        act(&mut g, Action::ChooseBargain { sacrifice });
        act(&mut g, Action::ChooseCasualty { pay: true });
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        pay(&mut g);
        let additional = g
            .state
            .announcing
            .as_ref()
            .unwrap()
            .cast_costs
            .as_ref()
            .unwrap()
            .clauses
            .iter()
            .position(|r| r.clause == CastCostClause::Additional)
            .unwrap() as u16;
        reject(&mut g, Action::ChooseCastCost { clause: additional });
        choose_cast_clause(&mut g, CastCostClause::Casualty(0));
        let object = g.object_ref(stolen);
        super::big_helpers::checked(&mut g, Action::PayCasualty { object });
        assert_eq!(g.object_view(stolen).zone, Zone::Graveyard);
        assert_eq!(g.object_view(stolen).owner, p.other());
        choose_cast_clause(&mut g, CastCostClause::Additional);
        until_choice(&mut g);
        act(&mut g, Action::FinishStackTargets);
        settle(&mut g);
        assert_eq!(g.object_view(confiscate).zone, Zone::Graveyard);
        assert_eq!(g.object_view(target).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].spells_cast_this_turn, 2);
    }
}
