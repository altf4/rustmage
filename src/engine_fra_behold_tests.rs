//! Exact, non-targeting witnesses shared by behold consumers.
use super::*;

#[test]
fn behold_witnesses_respect_control_hidden_hand_and_simultaneous_entry() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let own = put(&mut g, p, "Jace, Reality Sculptor", Zone::Battlefield);
        let hand = put(&mut g, p, "Jace, Reality Sculptor", Zone::Hand);
        let enemy_hand = put(&mut g, p.other(), "Jace, Reality Sculptor", Zone::Hand);
        let enemy = put(
            &mut g,
            p.other(),
            "Jace, Reality Sculptor",
            Zone::Battlefield,
        );
        let plain = put(&mut g, p, "Hill Giant", Zone::Battlefield);
        let witnesses = |g: &Game, excluded: &[ObjectRef]| {
            hand_choices::behold_candidates(g, p, "Jace", excluded)
        };
        let before = g.snapshot();
        let candidates = witnesses(&g, &[]);
        assert!(candidates.contains(&g.object_ref(own)));
        assert!(candidates.contains(&g.object_ref(hand)));
        assert!(!candidates.contains(&g.object_ref(enemy_hand)));
        assert!(!candidates.contains(&g.object_ref(enemy)));
        assert!(!candidates.contains(&g.object_ref(plain)));
        assert!(!witnesses(&g, &[g.object_ref(own)]).contains(&g.object_ref(own)));
        assert_eq!(g.snapshot(), before);
        g.change_controller(g.object_ref(enemy), p);
        assert!(witnesses(&g, &[]).contains(&g.object_ref(enemy)));
        g.move_object(own, Zone::Graveyard);
        assert!(!witnesses(&g, &[]).contains(&g.object_ref(own)));
        super::super::contracts::branch_contract(g);
    }
}

#[test]
fn behold_cost_candidates_are_non_consuming_and_respect_live_control() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Shock", Zone::Stack);
        let permanent = put(&mut g, p, "Jace, Reality Sculptor", Zone::Battlefield);
        let hand = put(&mut g, p, "Jace, Reality Sculptor", Zone::Hand);
        let opponent_hand = put(&mut g, p.other(), "Jace, Reality Sculptor", Zone::Hand);
        let cost = AdditionalCost {
            kind: EffectChoiceKind::Behold,
            filter: SacrificeFilter {
                keywords: Keywords::NONE,
                source_only: false,
                kinds: 0,
                nonland: false,
                other: false,
                tokens: None,
                artifact_enchantment_or_token: false,
                subtype: Some("Jace"),
            },
        };
        let before = g.snapshot();
        for id in [permanent, hand] {
            let reference = g.object_ref(id);
            assert!(g.additional_candidate(cost, p, g.object_ref(source), reference));
            assert!(g.can_fund(FundingGoal {
                actor: p,
                required: [0; 6],
                generic: 0,
                reserve: Some((reference, cost)),
                source: None,
            }));
        }
        assert!(!g.additional_candidate(
            cost,
            p,
            g.object_ref(source),
            g.object_ref(opponent_hand)
        ));
        assert_eq!(g.snapshot(), before);
        g.change_controller(g.object_ref(permanent), p.other());
        assert!(!g.additional_candidate(cost, p, g.object_ref(source), g.object_ref(permanent)));
    }
}

#[test]
fn countersculpt_both_cost_routes_counter_and_empower() {
    for p in [Player::P0, Player::P1] {
        for route in [0, 1] {
            for hand_witness in [false, true] {
                let mut g = scenario(p.other());
                let target = put(&mut g, p.other(), "Bear Cub", Zone::Hand);
                let spell = put(&mut g, p, "Countersculpt", Zone::Hand);
                let witness = put(
                    &mut g,
                    p,
                    "Jace, Reality Sculptor",
                    if hand_witness {
                        Zone::Hand
                    } else {
                        Zone::Battlefield
                    },
                );
                if !hand_witness {
                    g.add_counter(witness, CounterKind::Loyalty, 2);
                }
                cast(&mut g, target, None);
                g.state.priority = p;
                g.state.players[p.index()].mana = [0, 2, 0, 0, 0, 1];
                act(&mut g, Action::AnnounceCast { card: spell });
                act(&mut g, Action::ChooseSpellCost { cost: route });
                act(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Spell(target),
                    },
                );
                if route == 0 {
                    let reference = g.object_ref(witness);
                    act(&mut g, Action::ChooseAdditionalCost { object: reference });
                }
                pay(&mut g);
                if route == 0 && hand_witness {
                    assert!(
                        g.observe(p.other())
                            .known_hand
                            .contains(&g.object_ref(witness))
                    );
                }
                assert_eq!(
                    g.object_view(witness).zone,
                    if hand_witness {
                        Zone::Hand
                    } else {
                        Zone::Battlefield
                    }
                );
                super::super::contracts::branch_contract(g.clone());
                settle(&mut g);
                assert_eq!(g.object_view(target).zone, Zone::Graveyard);
                assert!(g.state.objects.iter().enumerate().any(|(id, o)| o.zone
                    == Zone::Battlefield
                    && o.controller == p
                    && o.is_token()
                    && g.has_subtype(id as CardId, "Jace")
                    && g.counter_count(id as CardId, CounterKind::Loyalty) == 1));
            }
        }
    }
}

#[test]
fn sanctum_behold_or_decline_preserves_hand_and_branches() {
    for p in [Player::P0, Player::P1] {
        for hand in [false, true] {
            for behold in [false, true] {
                let mut g = scenario(p);
                let witness = put(
                    &mut g,
                    p,
                    "Jace, Reality Sculptor",
                    if hand { Zone::Hand } else { Zone::Battlefield },
                );
                if !hand {
                    g.add_counter(witness, CounterKind::Loyalty, 2);
                }
                let sanctum = put(&mut g, p, "Theorist's Sanctum", Zone::Hand);
                act(&mut g, Action::PlayLand { card: sanctum });
                assert!(matches!(g.state.pending, Pending::EntryBehold { .. }));
                for viewer in [p, p.other()] {
                    assert!(g.observe(viewer).objects.iter().any(|o| o.id == sanctum
                        && o.card == Card::from_name("Theorist's Sanctum").unwrap()));
                }
                let witness_ref = g.object_ref(witness);
                let before = g.snapshot();
                let selection = Action::ChooseEntryBehold {
                    object: behold.then_some(witness_ref),
                };
                act(&mut g, selection.clone());
                assert_eq!(g.object_view(sanctum).zone, Zone::Battlefield);
                assert_eq!(g.object_view(sanctum).tapped, !behold);
                assert_eq!(
                    g.object_view(witness).zone,
                    if hand { Zone::Hand } else { Zone::Battlefield }
                );
                if hand {
                    assert_eq!(
                        g.observe(p.other()).known_hand.contains(&witness_ref),
                        behold
                    );
                }
                super::super::contracts::branch_contract(g.clone());
                g.restore(&before).unwrap();
                act(&mut g, selection);
                assert_eq!(g.object_view(sanctum).tapped, !behold);
            }
        }
        let mut g = scenario(p);
        let sanctum = put(&mut g, p, "Theorist's Sanctum", Zone::Hand);
        act(&mut g, Action::PlayLand { card: sanctum });
        assert!(g.object_view(sanctum).tapped);
    }
}

#[test]
fn sanctum_tap_replacements_compose_in_chosen_order() {
    for p in [Player::P0, Player::P1] {
        for intrinsic_first in [false, true] {
            let mut g = scenario(p);
            let spellunking = put(&mut g, p, "Spelunking", Zone::Battlefield);
            let witness = put(&mut g, p, "Jace, Reality Sculptor", Zone::Hand);
            let sanctum = put(&mut g, p, "Theorist's Sanctum", Zone::Hand);
            act(&mut g, Action::PlayLand { card: sanctum });
            assert!(matches!(g.state.pending, Pending::EntryReplacement { .. }));
            let source = if intrinsic_first {
                g.object_ref(sanctum)
            } else {
                g.object_ref(spelunking)
            };
            act(
                &mut g,
                Action::ChooseEntryReplacement {
                    source,
                    intrinsic: intrinsic_first,
                },
            );
            let witness_ref = g.object_ref(witness);
            if intrinsic_first {
                assert!(matches!(g.state.pending, Pending::EntryBehold { .. }));
                act(
                    &mut g,
                    Action::ChooseEntryBehold {
                        object: Some(witness_ref),
                    },
                );
            }
            // Declining the intrinsic replacement is overridden only when the
            // untapped replacement is applied after it.
            if !intrinsic_first {
                assert!(matches!(g.state.pending, Pending::EntryBehold { .. }));
                act(&mut g, Action::ChooseEntryBehold { object: None });
                assert!(g.object_view(sanctum).tapped);
            } else {
                assert!(!g.object_view(sanctum).tapped);
            }
        }
    }
}

#[test]
fn sanctum_adds_blue_and_its_other_ability_empowers() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let witness = put(&mut g, p, "Jace, Reality Sculptor", Zone::Hand);
        let land = put(&mut g, p, "Theorist's Sanctum", Zone::Hand);
        act(&mut g, Action::PlayLand { card: land });
        let reference = g.object_ref(witness);
        act(
            &mut g,
            Action::ChooseEntryBehold {
                object: Some(reference),
            },
        );
        g.state.players[p.index()].mana = [0; 6];
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::TapMana { card } if *card == land))
            .expect("blue mana ability");
        act(&mut g, action);
        assert_eq!(g.state.players[p.index()].mana[1], 1);
        g.untap_objects(&[land], untap_rules::UntapCause::Effect);
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 2];
        act(
            &mut g,
            Action::AnnounceActivate {
                card: land,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut g);
        settle(&mut g);
        assert!(
            g.state
                .objects
                .iter()
                .enumerate()
                .any(|(id, o)| o.zone == Zone::Battlefield
                    && o.controller == p
                    && o.is_token()
                    && g.has_subtype(id as CardId, "Jace")
                    && g.counter_count(id as CardId, CounterKind::Loyalty) == 2)
        );
    }
}

#[test]
fn sanctum_ability_removal_suppresses_blue_and_restores_it() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let land = put(&mut g, p, "Theorist's Sanctum", Zone::Battlefield);
        let aura = put(
            &mut g,
            p.other(),
            "Imprisoned in the Moon",
            Zone::Battlefield,
        );
        let reference = g.object_ref(land);
        g.state.objects[aura as usize].attached_to = Some(reference);
        assert!(g.has_subtype(land, "Island"));
        assert!(!g.legal_actions().contains(&Action::TapMana { card: land }));
        assert!(g.legal_actions().iter().any(|a| matches!(a, Action::TapGrantedMana { card, production, .. } if *card == land && *production == [0, 0, 0, 0, 0, 1])));
        let checkpoint = g.snapshot();
        super::super::contracts::branch_contract(g.clone());
        g.move_object(aura, Zone::Graveyard);
        assert!(g.has_subtype(land, "Island"));
        assert!(g.legal_actions().contains(&Action::TapMana { card: land }));
        g.restore(&checkpoint).unwrap();
        assert!(!g.legal_actions().contains(&Action::TapMana { card: land }));
    }
}

#[test]
fn forge_borrows_sanctum_printed_and_land_type_mana_abilities() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let sanctum = put(&mut g, p.other(), "Theorist's Sanctum", Zone::Battlefield);
        let forge = put(&mut g, p, "Territory Forge", Zone::Hand);
        cast(&mut g, forge, None);
        let target = Action::ChooseTarget {
            target: Target::Card(sanctum),
        };
        let mut chose = false;
        for _ in 0..64 {
            g.prepare_triggers();
            let actions = g.legal_actions();
            if !chose && actions.contains(&target) {
                act(&mut g, target.clone());
                chose = true;
            } else if actions.contains(&Action::FinishTargets) {
                act(&mut g, Action::FinishTargets);
            } else if chose
                && g.state.pending == Pending::Priority
                && g.state.stack.is_empty()
                && g.state.pending_triggers.is_empty()
            {
                break;
            } else if let Some(action) = actions
                .into_iter()
                .find(|a| matches!(a, Action::Pass | Action::PutTrigger { .. }))
            {
                act(&mut g, action);
            } else {
                panic!("unexpected Forge state {:?}", g.state.pending);
            }
        }
        assert!(chose);
        assert_eq!(g.object_view(sanctum).zone, Zone::Exile);
        let abilities = g.borrowed_ability_views();
        assert!(
            abilities
                .iter()
                .any(|a| a.source.card == forge && a.ability == 0)
        );
        assert!(
            abilities
                .iter()
                .any(|a| a.source.card == forge && a.ability == u16::MAX - 2)
        );
        super::super::contracts::branch_contract(g);
    }
}

#[test]
fn dual_land_type_mana_and_lone_mountain_borrowing() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let land = put(&mut g, p, "Hedge Maze", Zone::Battlefield);
        let actions = g.legal_actions();
        assert!(actions.contains(&Action::TapMana { card: land }));
        assert!(actions.iter().any(|a| matches!(a, Action::TapGrantedMana { card, production, .. } if *card == land && *production == [0, 0, 0, 0, 1, 0])));
        assert!(
            !actions
                .iter()
                .any(|a| matches!(a, Action::ActivateMana { card, .. } if *card == land))
        );
        super::super::contracts::branch_contract(g);
    }
}

#[test]
fn chosen_forest_produces_green_for_real_cast_with_or_without_ability_removal() {
    for p in [Player::P0, Player::P1] {
        for removal in [false, true] {
            let mut g = scenario(p);
            let land = put(&mut g, p, "Multiversal Passage", Zone::Hand);
            let creature = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            if removal {
                let aura = put(&mut g, p.other(), "Fresh Start", Zone::Battlefield);
                g.state.objects[aura as usize].attached_to = Some(g.object_ref(creature));
            }
            act(&mut g, Action::PlayLand { card: land });
            act(
                &mut g,
                Action::ChooseEntryValue {
                    value: EntryChoiceValue::BasicLandType(crate::cards::BasicLandType::Forest),
                },
            );
            act(&mut g, Action::ChooseEntryLife { pay: true });
            g.state.players[p.index()].mana = [0; 6];
            assert!(g.can_fund(FundingGoal {
                actor: p,
                required: [0, 0, 0, 0, 1, 0],
                generic: 0,
                reserve: None,
                source: None
            }));
            let growth = put(&mut g, p, "Giant Growth", Zone::Hand);
            act(&mut g, Action::AnnounceCast { card: growth });
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(creature),
                },
            );
            let tap = g.legal_actions().into_iter().find(|a| matches!(a, Action::TapGrantedMana { card, production, .. } if *card == land && *production == [0, 0, 0, 0, 1, 0])).expect("chosen Forest mana");
            act(&mut g, tap);
            pay(&mut g);
            assert_eq!(g.object_view(growth).zone, Zone::Stack);
        }
    }
}

#[test]
fn added_basic_land_mana_obeys_blank_and_grant_timestamps() {
    for p in [Player::P0, Player::P1] {
        for later in [false, true] {
            let mut g = scenario(p);
            let land = put(&mut g, p, "Meticulous Commons", Zone::Battlefield);
            let source = g.object_ref(land);
            let add = |game: &mut Game| {
                let item = game.cast_preview(p, land, Zone::Battlefield, source, 0, [0; 6]);
                game.add_basic_land_type(
                    &item,
                    Subject::Source,
                    crate::cards::BasicLandType::Island,
                );
            };
            if !later {
                add(&mut g);
            }
            let aura = put(
                &mut g,
                p.other(),
                "Imprisoned in the Moon",
                Zone::Battlefield,
            );
            g.state.objects[aura as usize].attached_to = Some(source);
            if later {
                add(&mut g);
            }
            assert!(g.has_subtype(land, "Island"));
            g.state.players[p.index()].mana = [0; 6];
            assert_eq!(
                g.can_fund(FundingGoal {
                    actor: p,
                    required: [0, 1, 0, 0, 0, 0],
                    generic: 0,
                    reserve: None,
                    source: None,
                }),
                later
            );
            let action = Action::TapGrantedMana {
                card: land,
                grantor: source,
                production: [0, 1, 0, 0, 0, 0],
            };
            assert_eq!(g.legal_actions().contains(&action), later);
            let before = g.snapshot();
            if later {
                g.apply(p, action.clone()).unwrap();
                assert_eq!(g.state.players[p.index()].mana[1], 1);
                g.restore(&before).unwrap();
                assert_eq!(g.snapshot(), before);
                g.apply(p, action).unwrap();
                assert_eq!(g.state.players[p.index()].mana[1], 1);
            }
        }
    }
}

#[test]
fn adding_an_already_present_basic_type_does_not_restore_a_removed_ability() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let land = put(&mut g, p, "Island", Zone::Battlefield);
        let source = g.object_ref(land);
        let aura = put(
            &mut g,
            p.other(),
            "Imprisoned in the Moon",
            Zone::Battlefield,
        );
        g.state.objects[aura as usize].attached_to = Some(source);
        let timestamp = g.next_role_timestamp();
        g.state
            .roles
            .get_or_insert_with(Default::default)
            .basic_land_types
            .push(subtypes::BasicLandTypesEffectView {
                recipient: source,
                source,
                kind: crate::cards::BasicLandType::Island,
                timestamp,
            });
        assert!(g.has_subtype(land, "Island"));
        assert!(!g.legal_actions().contains(&Action::TapMana { card: land }));
        assert!(!g.legal_actions().contains(&Action::TapGrantedMana {
            card: land,
            grantor: source,
            production: [0, 1, 0, 0, 0, 0]
        }));
    }
}

#[test]
fn added_basic_land_type_is_forgotten_on_zone_change_and_restored_by_snapshot() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let land = put(&mut g, p, "Meticulous Commons", Zone::Battlefield);
        let source = g.object_ref(land);
        let timestamp = g.next_role_timestamp();
        g.state
            .roles
            .get_or_insert_with(Default::default)
            .basic_land_types
            .push(subtypes::BasicLandTypesEffectView {
                recipient: source,
                source,
                kind: crate::cards::BasicLandType::Forest,
                timestamp,
            });
        let saved = g.snapshot();
        assert!(g.has_subtype(land, "Forest"));
        g.move_object(land, Zone::Graveyard);
        g.move_object(land, Zone::Battlefield);
        assert!(!g.has_subtype(land, "Forest"));
        assert!(
            g.state
                .roles
                .as_ref()
                .unwrap()
                .basic_land_types
                .iter()
                .all(|row| row.recipient != source)
        );
        g.restore(&saved).unwrap();
        assert!(g.has_subtype(land, "Forest"));
    }
}

#[test]
fn land_type_loss_and_regain_rebuilds_only_new_intrinsic_mana() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let land = put(&mut g, p, "Island", Zone::Battlefield);
        let source = g.object_ref(land);
        let animate = |game: &mut Game, food_only: bool, land_type: bool| {
            let timestamp = game.next_role_timestamp();
            game.state
                .roles
                .get_or_insert_with(Default::default)
                .animations
                .push(AnimationView {
                    object: source,
                    timestamp,
                    artifact: false,
                    land: land_type,
                    preserve_card_types: false,
                    food_only,
                    artifact_noncreature: false,
                    colors: None,
                    creature_types: None,
                    all_creature_types: false,
                    base: None,
                    keywords: Keywords::NONE,
                    expires_turn: None,
                });
        };
        animate(&mut g, false, true);
        let item = g.cast_preview(p, land, Zone::Battlefield, source, 0, [0; 6]);
        g.add_basic_land_type(&item, Subject::Source, crate::cards::BasicLandType::Forest);
        assert!(g.has_subtype(land, "Forest"));
        let aura = put(&mut g, p.other(), "Witness Protection", Zone::Battlefield);
        g.state.objects[aura as usize].attached_to = Some(source);
        animate(&mut g, false, true);
        assert!(!g.has_subtype(land, "Island"));
        assert!(!g.has_subtype(land, "Forest"));
        assert!(!g.legal_actions().contains(&Action::TapMana { card: land }));
        let item = g.cast_preview(p, land, Zone::Battlefield, source, 0, [0; 6]);
        g.add_basic_land_type(&item, Subject::Source, crate::cards::BasicLandType::Forest);
        assert!(!g.has_subtype(land, "Island"));
        assert!(g.has_subtype(land, "Forest"));
        let green = Action::TapGrantedMana {
            card: land,
            grantor: source,
            production: [0, 0, 0, 0, 1, 0],
        };
        assert!(g.legal_actions().contains(&green));
        g.state.players[p.index()].mana = [0; 6];
        let saved = g.snapshot();
        g.apply(p, green).unwrap();
        assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 0, 1, 0]);
        g.restore(&saved).unwrap();
        assert_eq!(g.snapshot(), saved);
    }
}

#[test]
fn chosen_basic_type_keeps_its_ability_removal_after_land_type_is_lost() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let land = put(&mut g, p, "Multiversal Passage", Zone::Hand);
        act(&mut g, Action::PlayLand { card: land });
        act(
            &mut g,
            Action::ChooseEntryValue {
                value: EntryChoiceValue::BasicLandType(crate::cards::BasicLandType::Forest),
            },
        );
        act(&mut g, Action::ChooseEntryLife { pay: true });
        let saved = g.snapshot();
        let source = g.object_ref(land);
        let timestamp = g.next_role_timestamp();
        g.state
            .roles
            .get_or_insert_with(Default::default)
            .animations
            .push(AnimationView {
                object: source,
                timestamp,
                artifact: false,
                land: false,
                preserve_card_types: false,
                food_only: true,
                artifact_noncreature: false,
                colors: None,
                creature_types: None,
                all_creature_types: false,
                base: None,
                keywords: Keywords::NONE,
                expires_turn: None,
            });
        assert_eq!(g.current_kinds(land) & 2, 0);
        assert!(!g.has_subtype(land, "Forest"));
        assert!(!ability_world::CharacteristicWorld::printed_abilities_available(&g, land));
        g.restore(&saved).unwrap();
        assert_eq!(g.snapshot(), saved);
    }
}

#[test]
fn unblanked_land_type_loss_and_regain_updates_fast_mana_path() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let land = put(&mut g, p, "Island", Zone::Battlefield);
        let source = g.object_ref(land);
        for (food_only, land_type) in [(true, false), (false, true)] {
            let timestamp = g.next_role_timestamp();
            g.state
                .roles
                .get_or_insert_with(Default::default)
                .animations
                .push(AnimationView {
                    object: source,
                    timestamp,
                    artifact: false,
                    land: land_type,
                    preserve_card_types: false,
                    food_only,
                    artifact_noncreature: false,
                    colors: None,
                    creature_types: None,
                    all_creature_types: false,
                    base: land_type.then_some((1, 1)),
                    keywords: Keywords::NONE,
                    expires_turn: None,
                });
        }
        assert!(!g.has_subtype(land, "Island"));
        assert!(!g.legal_actions().contains(&Action::TapMana { card: land }));
        let item = g.cast_preview(p, land, Zone::Battlefield, source, 0, [0; 6]);
        g.add_basic_land_type(&item, Subject::Source, crate::cards::BasicLandType::Forest);
        assert!(g.has_subtype(land, "Forest"));
        assert!(g.legal_actions().contains(&Action::TapMana { card: land }));
        g.state.players[p.index()].mana = [0; 6];
        let before = g.snapshot();
        g.apply(p, Action::TapMana { card: land }).unwrap();
        assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 0, 1, 0]);
        g.restore(&before).unwrap();
        assert_eq!(g.snapshot(), before);
    }
}
