use super::*;

// Model the resolving copy effect: retargeting is an owned continuation of
// a resolving ability, rather than a stand-alone engine operation.
fn copy_ability_in_resolution(g: &mut Game, copier: &SpellView) {
    assert!(g.state.resolving.is_none());
    g.state.resolving = Some(Resolution {
        chosen_creature_type: None,
        target_counter_removal: None,
        linked_copy: None,
        cascade: None,
        target_card_facts: None,
        immediate_blink_return: None,
        connive: None,
        explore: None,
        evidence: None,
        item: copier.clone(),
        mana_payment: None,
        forage: None,
        target_library: None,
        program: &[],
        pc: 0,
        target_group: 0,
        moved_count: None,
        remembered_draws: None,
        remembered_creatures: Vec::new(),
        sacrificed_power: None,
        discard_after: None,
        optional_wheel: None,
        proliferate: None,
        discards: [0; 2],
        selected_discards: [Vec::new(), Vec::new()],
        milled: Vec::new(),
        milled_selection: None,
        library: None,
        selection: None,
        return_attachment: None,
        search: None,
        search_result: None,
        optional_searches: None,
        choice: None,
    });
    assert!(g.copy_target_ability(copier, 0));
}

fn chill(g: &mut Game, player: Player, loyalty: u64) -> CardId {
    let id = put(g, player, "Bear Cub", Zone::Battlefield);
    g.state.objects[id as usize].card = Card::from_name("Chandra, Chill of Compliance").unwrap();
    g.state.objects[id as usize].counters.loyalty = loyalty;
    g.register_battlefield_sources(id);
    id
}
fn activate(g: &mut Game, id: CardId, ability: u16) {
    act(
        g,
        Action::AnnounceActivate {
            card: id,
            ability,
            counter_payment: Counters::default(),
        },
    );
    pay(g);
}

#[test]
fn chill_surveil_returns_only_noncreature_nonland_actually_moved_to_graveyard() {
    for player in [Player::P0, Player::P1] {
        for (name, replacement, expected) in [
            ("Shock", false, Zone::Hand),
            ("Bear Cub", false, Zone::Graveyard),
            ("Forest", false, Zone::Graveyard),
            ("Shock", true, Zone::Exile),
        ] {
            let mut g = scenario(player);
            let source = chill(&mut g, player, 3);
            if replacement {
                put(&mut g, player, "Rest in Peace", Zone::Battlefield);
            }
            let card = put(&mut g, player, name, Zone::Library);
            activate(&mut g, source, 0);
            g.move_object(source, Zone::Graveyard);
            g.resolve_top();
            assert!(matches!(g.state.pending, Pending::EffectLibrary { .. }));
            let saved = g.snapshot();
            for _ in 0..2 {
                act(
                    &mut g,
                    Action::ArrangeLibrary {
                        card,
                        destination: LibraryDestination::Graveyard,
                    },
                );
                assert_eq!(g.object_view(card).zone, expected);
                g.restore(&saved).unwrap();
            }
        }
    }
}

#[test]
fn chill_restricted_blue_funds_noncreature_spell_but_not_creature() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let source = chill(&mut g, player, 3);
        g.state.players[player.index()].mana = [0; 6];
        activate(&mut g, source, 1);
        g.resolve_top();
        assert_eq!(g.state.players[player.index()].mana[1], 1);
        assert!(
            g.mana_lots(player)
                .iter()
                .any(|lot| lot.quality.restriction == ManaRestrictionView::NoncreatureSpell)
        );
        let creature = put(&mut g, player, "Spectral Sailor", Zone::Hand);
        reject(&mut g, Action::AnnounceCast { card: creature });
        let spell = put(&mut g, player, "Opt", Zone::Hand);
        let saved = g.snapshot();
        for _ in 0..2 {
            act(&mut g, Action::AnnounceCast { card: spell });
            pay(&mut g);
            assert_eq!(g.state.players[player.index()].mana[1], 0);
            g.restore(&saved).unwrap();
        }
    }
}

#[test]
fn chill_variable_loyalty_cost_keeps_x_after_source_leaves() {
    for player in [Player::P0, Player::P1] {
        for amount in [0, 1, 3] {
            let mut g = scenario(player);
            let source = chill(&mut g, player, 3);
            let target = put(&mut g, player.other(), "Gilded Lotus", Zone::Battlefield);
            let source_snapshot = g.snapshot();
            act(
                &mut g,
                Action::AnnounceActivate {
                    card: source,
                    ability: 2,
                    counter_payment: Counters::default(),
                },
            );
            reject(&mut g, Action::ChooseSourceCounterAmount { amount: 4 });
            act(&mut g, Action::ChooseSourceCounterAmount { amount });
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(target),
                },
            );
            if g.legal_actions().contains(&Action::FinishTargets) {
                act(&mut g, Action::FinishTargets);
            }
            pay(&mut g);
            let item = g.state.stack.last().unwrap();
            assert_eq!(item.x, Some(amount as u32));
            assert_eq!(item.loyalty_counters_paid, amount);
            if amount != 3 {
                g.move_object(source, Zone::Graveyard);
            }
            let saved = g.snapshot();
            for _ in 0..2 {
                g.resolve_top();
                assert!(g.object_view(target).tapped);
                assert_eq!(g.counter_count(target, CounterKind::Stun), amount);
                g.restore(&saved).unwrap();
            }
            g.restore(&source_snapshot).unwrap();
        }
    }
}

#[test]
fn chill_emblem_draws_after_source_departure_on_own_cast() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let source = chill(&mut g, player, 7);
        activate(&mut g, source, 3);
        g.resolve_top();
        g.move_object(source, Zone::Graveyard);
        assert!(
            g.state
                .objects
                .iter()
                .any(|o| o.zone == Zone::Command && o.card == Card::test_index(5466))
        );
        let creature = put(&mut g, player, "Bear Cub", Zone::Hand);
        let draws = g.state.players[player.index()].cards_drawn_this_turn;
        cast(&mut g, creature, None);
        g.prepare_triggers();
        g.resolve_top();
        assert_eq!(
            g.state.players[player.index()].cards_drawn_this_turn,
            draws + 1
        );
    }
}

#[test]
fn chill_copied_x_activation_preserves_paid_amount_and_target_incarnation() {
    for player in [Player::P0, Player::P1] {
        for reenter in [false, true] {
            let mut g = scenario(player);
            let source = chill(&mut g, player, 3);
            let target = put(&mut g, player.other(), "Gilded Lotus", Zone::Battlefield);
            act(
                &mut g,
                Action::AnnounceActivate {
                    card: source,
                    ability: 2,
                    counter_payment: Counters::default(),
                },
            );
            act(&mut g, Action::ChooseSourceCounterAmount { amount: 2 });
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(target),
                },
            );
            pay(&mut g);
            let original = g.state.stack.last().unwrap().clone();
            let mut copier = original.clone();
            copier.targets = vec![BoundTarget {
                group: 0,
                target: Target::Stack(original.id),
                object: None,
            }];
            copy_ability_in_resolution(&mut g, &copier);
            if g.legal_actions().contains(&Action::FinishStackTargets) {
                act(&mut g, Action::FinishStackTargets);
            }
            let copied = g.state.stack.last().unwrap();
            assert!(copied.is_copy);
            assert_eq!(copied.x, Some(2));
            g.move_object(source, Zone::Graveyard);
            if reenter {
                g.move_object(target, Zone::Exile);
                g.move_object(target, Zone::Battlefield);
            }
            let saved = g.snapshot();
            for _ in 0..2 {
                settle(&mut g);
                assert_eq!(
                    g.counter_count(target, CounterKind::Stun),
                    if reenter { 0 } else { 4 }
                );
                assert_eq!(g.object_view(target).tapped, !reenter);
                g.restore(&saved).unwrap();
            }
        }
    }
}

#[test]
fn chill_large_loyalty_x_survives_copy_observation_and_restore() {
    let amount = u64::from(u32::MAX) + 17;
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let source = chill(&mut g, player, amount + 1);
        let target = put(&mut g, player.other(), "Gilded Lotus", Zone::Battlefield);
        let initial = g.snapshot();
        act(
            &mut g,
            Action::AnnounceActivate {
                card: source,
                ability: 2,
                counter_payment: Counters::default(),
            },
        );
        assert_eq!(g.source_counter_choice().unwrap().maximum, amount + 1);
        reject(
            &mut g,
            Action::ChooseSourceCounterAmount { amount: amount + 2 },
        );
        act(&mut g, Action::ChooseSourceCounterAmount { amount });
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            act(&mut g, Action::FinishTargets);
        }
        pay(&mut g);
        let original = g.state.stack.last().unwrap().clone();
        assert_eq!(original.loyalty_x, Some(amount));
        assert_eq!(original.loyalty_counters_paid, amount);
        for viewer in [player, player.other()] {
            let json = serde_json::to_value(g.observe(viewer)).unwrap();
            assert!(
                json.to_string()
                    .contains(&format!("\"loyalty_x\":{amount}"))
            );
        }
        let mut copier = original.clone();
        copier.targets = vec![BoundTarget {
            group: 0,
            target: Target::Stack(original.id),
            object: None,
        }];
        copy_ability_in_resolution(&mut g, &copier);
        if g.legal_actions().contains(&Action::FinishStackTargets) {
            act(&mut g, Action::FinishStackTargets);
        }
        assert_eq!(g.state.stack.last().unwrap().loyalty_x, Some(amount));
        g.move_object(source, Zone::Graveyard);
        let saved = g.snapshot();
        for _ in 0..2 {
            settle(&mut g);
            assert_eq!(g.counter_count(target, CounterKind::Stun), amount * 2);
            g.restore(&saved).unwrap();
        }
        g.restore(&initial).unwrap();
        assert_eq!(g.counter_count(source, CounterKind::Loyalty), amount + 1);
        assert_eq!(g.counter_count(target, CounterKind::Stun), 0);
    }
}

#[test]
fn chill_surveil_keeps_top_and_handles_empty_library() {
    for player in [Player::P0, Player::P1] {
        for empty in [false, true] {
            let mut g = scenario(player);
            let source = chill(&mut g, player, 3);
            if empty {
                let library = g.state.players[player.index()].library.clone();
                for id in library {
                    g.move_object(id, Zone::Exile);
                }
            }
            let top = (!empty).then(|| put(&mut g, player, "Shock", Zone::Library));
            activate(&mut g, source, 0);
            g.resolve_top();
            if let Some(card) = top {
                assert!(matches!(g.state.pending, Pending::EffectLibrary { .. }));
                let saved = g.snapshot();
                for _ in 0..2 {
                    act(
                        &mut g,
                        Action::ArrangeLibrary {
                            card,
                            destination: LibraryDestination::Top,
                        },
                    );
                    assert_eq!(g.object_view(card).zone, Zone::Library);
                    assert_eq!(g.state.players[player.index()].library.last(), Some(&card));
                    g.restore(&saved).unwrap();
                }
            } else {
                assert!(!matches!(g.state.pending, Pending::EffectLibrary { .. }));
            }
        }
    }
}

#[test]
fn chill_restricted_blue_cannot_fund_face_down_creature_and_restores() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let source = chill(&mut g, player, 3);
        g.state.players[player.index()].mana = [0; 6];
        activate(&mut g, source, 1);
        g.resolve_top();
        // Face-down disguise is a creature spell, even when its cost is generic.
        g.state.players[player.index()].mana[5] = 2;
        let disguise = put(&mut g, player, "Bubble Smuggler", Zone::Hand);
        let saved = g.snapshot();
        reject(&mut g, Action::AnnounceDisguise { card: disguise });
        assert!(
            g.mana_lots(player)
                .iter()
                .any(|lot| lot.quality.restriction == ManaRestrictionView::NoncreatureSpell)
        );
        g.restore(&saved).unwrap();
        reject(&mut g, Action::AnnounceDisguise { card: disguise });
    }
}

#[test]
fn chill_copied_minus_x_retargets_for_new_controller_and_shares_loyalty_use() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let source = chill(&mut g, player, 5);
        let first = put(&mut g, player, "Gilded Lotus", Zone::Battlefield);
        let second = put(&mut g, player.other(), "Gilded Lotus", Zone::Battlefield);
        act(
            &mut g,
            Action::AnnounceActivate {
                card: source,
                ability: 2,
                counter_payment: Counters::default(),
            },
        );
        act(&mut g, Action::ChooseSourceCounterAmount { amount: 2 });
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(first),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            act(&mut g, Action::FinishTargets);
        }
        pay(&mut g);
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: source,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        let original = g.state.stack.last().unwrap().clone();
        let mut copier = original.clone();
        copier.controller = player.other();
        copier.targets = vec![BoundTarget {
            group: 0,
            target: Target::Stack(original.id),
            object: None,
        }];
        copy_ability_in_resolution(&mut g, &copier);
        act(
            &mut g,
            Action::SetStackTarget {
                index: 0,
                target: Target::Card(second),
            },
        );
        act(&mut g, Action::FinishStackTargets);
        assert_eq!(g.state.stack.last().unwrap().controller, player.other());
        let snapshot = g.snapshot();
        for _ in 0..2 {
            settle(&mut g);
            assert_eq!(g.counter_count(first, CounterKind::Stun), 2);
            assert_eq!(g.counter_count(second, CounterKind::Stun), 2);
            g.restore(&snapshot).unwrap();
        }
    }
}

#[test]
fn chill_stun_counters_honor_doubling_season() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let source = chill(&mut g, player, 4);
        let target = put(&mut g, player, "Gilded Lotus", Zone::Battlefield);
        put(&mut g, player, "Doubling Season", Zone::Battlefield);
        act(
            &mut g,
            Action::AnnounceActivate {
                card: source,
                ability: 2,
                counter_payment: Counters::default(),
            },
        );
        act(&mut g, Action::ChooseSourceCounterAmount { amount: 2 });
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            act(&mut g, Action::FinishTargets);
        }
        pay(&mut g);
        let snapshot = g.snapshot();
        for _ in 0..2 {
            g.resolve_top();
            assert_eq!(g.counter_count(target, CounterKind::Stun), 4);
            g.restore(&snapshot).unwrap();
        }
    }
}

#[test]
fn chill_surveil_does_not_return_artifact_creature() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let source = chill(&mut g, player, 3);
        let card = put(&mut g, player, "Adaptive Automaton", Zone::Library);
        activate(&mut g, source, 0);
        g.resolve_top();
        let snapshot = g.snapshot();
        for _ in 0..2 {
            act(
                &mut g,
                Action::ArrangeLibrary {
                    card,
                    destination: LibraryDestination::Graveyard,
                },
            );
            assert_eq!(g.object_view(card).zone, Zone::Graveyard);
            g.restore(&snapshot).unwrap();
        }
    }
}

#[test]
fn chill_restricted_blue_pays_selected_adventure_face() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let source = chill(&mut g, player, 3);
        g.state.players[player.index()].mana = [0; 6];
        activate(&mut g, source, 1);
        g.resolve_top();
        g.state.players[player.index()].mana[5] = 3;
        let card = put(
            &mut g,
            player,
            "Lake-town Mariners // Gone Fishing",
            Zone::Hand,
        );
        let first = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        let second = put(&mut g, player, "Forest", Zone::Battlefield);
        let saved = g.snapshot();
        for _ in 0..2 {
            act(&mut g, Action::AnnounceAdventure { card });
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(first),
                },
            );
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(second),
                },
            );
            if g.legal_actions().contains(&Action::FinishTargets) {
                act(&mut g, Action::FinishTargets);
            }
            pay(&mut g);
            assert_eq!(g.state.players[player.index()].mana, [0; 6]);
            assert_eq!(
                g.state.stack.last().unwrap().cast.unwrap().face,
                CastFace::Adventure
            );
            g.restore(&saved).unwrap();
        }
    }
}

#[test]
fn chill_restricted_blue_pays_prepared_noncreature_face() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let prepared = put(
            &mut g,
            player,
            "Theorix Metamage // Omit Variables",
            Zone::Hand,
        );
        act(&mut g, Action::AnnounceCast { card: prepared });
        choose(&mut g, |a| matches!(a, Action::ChooseSpellCost { .. }));
        pay(&mut g);
        settle(&mut g);
        let permanent = g.object_ref(prepared);
        let copy = g
            .state
            .preparations
            .as_ref()
            .unwrap()
            .iter()
            .find(|row| row.permanent == permanent)
            .unwrap()
            .spell_copy;
        let source = chill(&mut g, player, 3);
        g.state.players[player.index()].mana = [0; 6];
        activate(&mut g, source, 1);
        g.resolve_top();
        assert_eq!(g.state.players[player.index()].mana[1], 1);
        assert!(
            g.mana_lots(player)
                .iter()
                .any(|lot| lot.quality.restriction == ManaRestrictionView::NoncreatureSpell)
        );
        let saved = g.snapshot();
        for _ in 0..2 {
            act(&mut g, Action::AnnounceExile { card: copy.card });
            let announcement = g.state.announcing.as_ref().unwrap();
            assert_eq!(
                announcement.item.source.unwrap().card,
                Card::test_index(5451)
            );
            let blue = announcement.cost_options().next().unwrap();
            assert_eq!(blue.colored, [0, 1, 0, 0, 0, 0]);
            assert!(
                g.announcement_cost_fundable(announcement, blue),
                "prepared sorcery face must be fundable with restricted blue"
            );
            for _ in 0..20 {
                if g.state.announcing.is_none() {
                    break;
                }
                choose(&mut g, |a| {
                    matches!(
                        a,
                        Action::ChooseSpellCost { .. }
                            | Action::FinishTargets
                            | Action::PayCosts { .. }
                    )
                });
            }
            assert!(g.state.announcing.is_none());
            assert_eq!(g.state.players[player.index()].mana[1], 0);
            g.restore(&saved).unwrap();
        }
    }
}

#[test]
fn chill_loyalty_x_choice_rejects_wrong_actor_and_restores_before_choice() {
    for player in [Player::P0, Player::P1] {
        for amount in [0, u64::from(u32::MAX) + 17] {
            let mut g = scenario(player);
            let source = chill(&mut g, player, amount + 1);
            put(&mut g, player.other(), "Gilded Lotus", Zone::Battlefield);
            act(
                &mut g,
                Action::AnnounceActivate {
                    card: source,
                    ability: 2,
                    counter_payment: Counters::default(),
                },
            );
            let before = g.snapshot();
            let revision = g.revision();
            let action = Action::ChooseSourceCounterAmount { amount };
            assert_eq!(
                g.apply(player.other(), action.clone()),
                Err(Error::WrongActor)
            );
            assert_eq!(g.snapshot(), before);
            assert_eq!(g.revision(), revision);
            for _ in 0..2 {
                act(&mut g, action.clone());
                let item = &g.state.announcing.as_ref().unwrap().item;
                assert_eq!(item.loyalty_x, Some(amount));
                assert_eq!(item.x, u32::try_from(amount).ok());
                assert_eq!(g.counter_count(source, CounterKind::Loyalty), amount + 1);
                g.restore(&before).unwrap();
                assert_eq!(g.source_counter_choice().unwrap().maximum, amount + 1);
            }
        }
    }
}

#[test]
fn echo_copies_targeted_creature_ability_through_legal_actions() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let officer = put(&mut g, player, "Raft Security Officer", Zone::Battlefield);
        let echo = put(
            &mut g,
            player,
            "Echo, Perceptive Prodigy",
            Zone::Battlefield,
        );
        let first = put(&mut g, player.other(), "Bear Cub", Zone::Battlefield);
        let second = put(&mut g, player.other(), "Hill Giant", Zone::Battlefield);
        act(
            &mut g,
            Action::AnnounceActivate {
                card: officer,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(first),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            act(&mut g, Action::FinishTargets);
        }
        pay(&mut g);
        let original = g.state.stack.last().unwrap().id;
        act(
            &mut g,
            Action::AnnounceActivate {
                card: echo,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Stack(original),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            act(&mut g, Action::FinishTargets);
        }
        pay(&mut g);
        act(&mut g, Action::Pass);
        act(&mut g, Action::Pass);
        assert!(matches!(g.state.pending, Pending::Retarget { player: p } if p == player));
        let saved = g.snapshot();
        for _ in 0..2 {
            act(
                &mut g,
                Action::SetStackTarget {
                    index: 0,
                    target: Target::Card(second),
                },
            );
            act(&mut g, Action::FinishStackTargets);
            let copied = g.state.stack.last().unwrap();
            assert!(copied.is_copy);
            assert_eq!(copied.controller, player);
            settle(&mut g);
            assert!(g.object_view(first).tapped);
            assert!(g.object_view(second).tapped);
            g.restore(&saved).unwrap();
        }
    }
}

#[test]
fn echo_legally_copies_animated_chandra_loyalty_ability() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let source = chill(&mut g, player, 3);
        let object = g.object_ref(source);
        let timestamp = g.next_role_timestamp();
        g.state
            .roles
            .get_or_insert_with(Default::default)
            .animations
            .push(AnimationView {
                object,
                timestamp,
                artifact: false,
                land: false,
                preserve_card_types: false,
                food_only: false,
                artifact_noncreature: false,
                enchantment_noncreature: false,
                colors: None,
                creature_types: None,
                all_creature_types: false,
                base: Some((3, 3)),
                keywords: Keywords::NONE,
                expires_turn: None,
            });
        assert_eq!(g.current_kinds(source) & (1 | 64), 1 | 64);
        let echo = put(
            &mut g,
            player,
            "Echo, Perceptive Prodigy",
            Zone::Battlefield,
        );
        let first = put(&mut g, player.other(), "Gilded Lotus", Zone::Battlefield);
        let second = put(&mut g, player.other(), "Bear Cub", Zone::Battlefield);
        act(
            &mut g,
            Action::AnnounceActivate {
                card: source,
                ability: 2,
                counter_payment: Counters::default(),
            },
        );
        act(&mut g, Action::ChooseSourceCounterAmount { amount: 2 });
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(first),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            act(&mut g, Action::FinishTargets);
        }
        pay(&mut g);
        let original = g.state.stack.last().unwrap().id;
        act(
            &mut g,
            Action::AnnounceActivate {
                card: echo,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Stack(original),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            act(&mut g, Action::FinishTargets);
        }
        pay(&mut g);
        act(&mut g, Action::Pass);
        act(&mut g, Action::Pass);
        assert!(matches!(g.state.pending, Pending::Retarget { player: p } if p == player));
        let saved = g.snapshot();
        for _ in 0..2 {
            act(
                &mut g,
                Action::SetStackTarget {
                    index: 0,
                    target: Target::Card(second),
                },
            );
            act(&mut g, Action::FinishStackTargets);
            let copied = g.state.stack.last().unwrap();
            assert!(copied.is_copy);
            assert_eq!(copied.loyalty_x, Some(2));
            settle(&mut g);
            assert_eq!(g.counter_count(first, CounterKind::Stun), 2);
            assert_eq!(g.counter_count(second, CounterKind::Stun), 2);
            g.restore(&saved).unwrap();
        }
    }
}
