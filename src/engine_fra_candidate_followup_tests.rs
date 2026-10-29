use super::*;

#[test]
fn craterclaw_and_pia_reject_unfunded_actions_and_pia_token_has_printed_characteristics() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let colossus = put(&mut g, p, "Craterclaw Colossus", Zone::Hand);
        g.state.players[p.index()].mana = [0; 6];
        reject(&mut g, Action::AnnounceCast { card: colossus });
        assert_eq!(g.object_view(colossus).zone, Zone::Hand);

        let pia = put(&mut g, p, "Pia, Determined Rebuilder", Zone::Battlefield);
        put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let activation = Action::AnnounceActivate {
            card: pia,
            ability: 0,
            counter_payment: Counters::default(),
        };
        g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 4];
        reject(&mut g, activation);
        assert_eq!(g.object_view(pia).zone, Zone::Battlefield);

        let mut h = scenario(p);
        let pia = put(&mut h, p, "Pia, Determined Rebuilder", Zone::Hand);
        cast(&mut h, pia, None);
        settle(&mut h);
        let thopters = fra_thopters(&h, p);
        assert_eq!(thopters.len(), 1);
        let thopter = thopters[0];
        let view = h.object_view(thopter);
        assert_eq!(view.owner, p);
        assert_eq!(view.controller, p);
        assert!(view.card.is_artifact());
        assert!(view.card.is_creature());
        assert!(h.has_subtype(thopter, "Thopter"));
        assert_eq!(view.colors, 0);
        assert_eq!(stats(&h, thopter), (1, 1));
        assert!(h.keywords(thopter).contains(Keywords::FLYING));
    }
}

#[test]
fn gallia_countered_creature_can_attack_and_control_change_updates_haste() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let gallia = put(&mut g, p, "Gallia, the Merrymaker", Zone::Battlefield);
        let ally = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.state.objects[ally as usize].summoning_sick = true;
        g.add_counter(ally, CounterKind::PlusOne, 1);
        assert!(g.keywords(ally).contains(Keywords::HASTE));
        let source = g.object_ref(gallia);
        g.change_controller(source, p.other());
        assert!(!g.keywords(ally).contains(Keywords::HASTE));
        g.change_controller(source, p);
        assert!(g.keywords(ally).contains(Keywords::HASTE));
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        act(&mut g, Action::Attack { card: ally });
        act(&mut g, Action::FinishAttackers);
        assert!(g.state.attackers.iter().any(|a| a.card == ally));
    }
}

#[test]
fn gallia_ability_does_not_affect_reentered_target_or_new_hexproof() {
    for p in [Player::P0, Player::P1] {
        for reenter in [false, true] {
            let mut g = scenario(p);
            let gallia = put(&mut g, p, "Gallia, the Merrymaker", Zone::Battlefield);
            let target = put(&mut g, p.other(), "Bear Cub", Zone::Hand);
            g.move_object(target, Zone::Battlefield);
            g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 1];
            act(
                &mut g,
                Action::AnnounceActivate {
                    card: gallia,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(target),
                },
            );
            pay(&mut g);
            if reenter {
                g.move_object(target, Zone::Exile);
                g.move_object(target, Zone::Battlefield);
            } else {
                g.state.objects[target as usize].gained_keywords = Keywords::HEXPROOF;
            }
            settle(&mut g);
            assert_eq!(g.counter_count(target, CounterKind::PlusOne), 0);
        }
    }
}

#[test]
fn entrust_sacrifice_can_be_replaced_by_exile_and_search_can_find_nothing() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p, "Rest in Peace", Zone::Battlefield);
        let walker = put(&mut g, p, "Liliana, Dreadhorde General", Zone::Battlefield);
        g.add_counter(walker, CounterKind::Loyalty, 5);
        put(&mut g, p, "Bear Cub", Zone::Library);
        let spell = put(&mut g, p, "Entrust the Spark", Zone::Hand);
        cast(&mut g, spell, None);
        let mut sacrificed = false;
        let mut finished = false;
        for _ in 0..64 {
            g.prepare_triggers();
            let actions = g.legal_actions();
            if !sacrificed && actions.contains(&Action::DeclineEffectPayment) {
                let object = g.object_ref(walker);
                act(&mut g, Action::Sacrifice { object });
                sacrificed = true;
            } else if actions.contains(&Action::FinishSearch) {
                assert!(g.observe(p.other()).search_choice.is_none());
                act(&mut g, Action::FinishSearch);
                finished = true;
            } else if sacrificed
                && g.state.pending == Pending::Priority
                && g.state.stack.is_empty()
                && g.state.pending_triggers.is_empty()
                && g.state.resolving.is_none()
            {
                break;
            } else if let Some(action) = actions
                .into_iter()
                .find(|a| matches!(a, Action::Pass | Action::PutTrigger { .. }))
            {
                act(&mut g, action);
            } else {
                panic!("unexpected Entrust choice {:?}", g.state.pending);
            }
        }
        assert!(sacrificed && finished);
        assert_eq!(g.object_view(walker).zone, Zone::Exile);
    }
}

#[test]
fn darklight_phoenix_counts_simultaneous_deaths_but_not_exiled_creatures() {
    for p in [Player::P0, Player::P1] {
        for replaced in [false, true] {
            let mut g = scenario(p);
            let phoenix = put(&mut g, p, "Darklight Phoenix", Zone::Graveyard);
            if replaced {
                put(&mut g, p, "Rest in Peace", Zone::Battlefield);
            }
            let a = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let b = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let observers = g.death_trigger_sources();
            let events = g.move_objects_raw(&[a, b], Zone::Graveyard);
            g.finish_zone_events(&events, &observers);
            g.begin_step(Step::BeginCombat);
            g.prepare_triggers();
            super::super::contracts::branch_contract(g.clone());
            settle(&mut g);
            assert_eq!(
                g.object_view(a).zone,
                if replaced {
                    Zone::Exile
                } else {
                    Zone::Graveyard
                }
            );
            assert_eq!(
                g.object_view(b).zone,
                if replaced {
                    Zone::Exile
                } else {
                    Zone::Graveyard
                }
            );
            assert_eq!(
                g.object_view(phoenix).zone,
                if replaced {
                    Zone::Graveyard
                } else {
                    Zone::Battlefield
                }
            );
        }
    }
}

#[test]
fn flickering_hound_trigger_survives_source_but_rechecks_target() {
    for p in [Player::P0, Player::P1] {
        for invalid in [false, true] {
            let mut g = scenario(p);
            let hound = put(&mut g, p, "Flickering Hound", Zone::Battlefield);
            let target = put(&mut g, p, "Hill Giant", Zone::Battlefield);
            let spell = put(&mut g, p, "Bear Cub", Zone::Hand);
            let old = g.object_ref(target);
            cast(&mut g, spell, None);
            let mut selected = false;
            for _ in 0..64 {
                g.prepare_triggers();
                let actions = g.legal_actions();
                let choice = Action::ChooseTarget {
                    target: Target::Card(target),
                };
                if !selected && actions.contains(&choice) {
                    act(&mut g, choice);
                    selected = true;
                } else if selected && actions.contains(&Action::FinishTargets) {
                    act(&mut g, Action::FinishTargets);
                    break;
                } else if let Some(a) = actions
                    .into_iter()
                    .find(|a| matches!(a, Action::Pass | Action::PutTrigger { .. }))
                {
                    act(&mut g, a);
                } else {
                    panic!("unexpected Hound choice {:?}", g.state.pending);
                }
            }
            assert!(selected);
            g.move_object(hound, Zone::Graveyard);
            if invalid {
                g.change_controller(old, p.other());
            }
            super::super::contracts::branch_contract(g.clone());
            settle(&mut g);
            assert_eq!(g.object_view(target).zone, Zone::Battlefield);
            assert_eq!(g.object_ref(target) != old, !invalid);
        }
    }
}

#[test]
fn living_library_rejects_unfunded_and_can_sacrifice_to_exile() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p, "Rest in Peace", Zone::Battlefield);
        let source = put(&mut g, p, "Living Library", Zone::Battlefield);
        let target = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let activation = Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        };
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 5];
        reject(&mut g, activation.clone());
        assert_eq!(g.object_view(source).zone, Zone::Battlefield);
        g.state.players[p.index()].mana[5] = 6;
        act(&mut g, activation);
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        pay(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Exile);
        super::super::contracts::branch_contract(g.clone());
        settle(&mut g);
        assert_eq!(g.object_view(target).zone, Zone::Library);
    }
}

#[test]
fn return_to_light_realms_leaves_unattachable_aura_in_graveyard() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let aura = put(&mut g, p, "Pacifism", Zone::Graveyard);
        let artifact = put(&mut g, p, "Mox Jasper", Zone::Graveyard);
        let spell = put(&mut g, p, "Return to the Light Realms", Zone::Hand);
        g.state.players[p.index()].mana = [2, 0, 0, 0, 0, 7];
        cast(&mut g, spell, None);
        super::super::contracts::branch_contract(g.clone());
        settle(&mut g);
        assert_eq!(g.object_view(aura).zone, Zone::Graveyard);
        assert_eq!(g.object_view(artifact).zone, Zone::Battlefield);
    }
}

#[test]
fn saheeli_thopter_is_colorless_and_owned_by_trigger_controller() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Saheeli, Jewel of Avishkar", Zone::Battlefield);
        let spell = put(&mut g, p, "Shock", Zone::Hand);
        cast(&mut g, spell, Some(Target::Player(p.other())));
        g.move_object(source, Zone::Graveyard);
        settle(&mut g);
        let thopters = fra_thopters(&g, p);
        assert_eq!(thopters.len(), 1);
        let view = g.object_view(thopters[0]);
        assert_eq!(view.owner, p);
        assert_eq!(view.controller, p);
        assert_eq!(view.colors, 0);
        assert!(view.card.is_artifact() && view.card.is_creature());
        assert!(g.has_subtype(thopters[0], "Thopter"));
        assert_eq!(stats(&g, thopters[0]), (1, 1));
        assert!(g.keywords(thopters[0]).contains(Keywords::FLYING));
    }
}

#[test]
fn saheeli_cast_trigger_resolves_even_when_spell_is_countered() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p, "Saheeli, Jewel of Avishkar", Zone::Battlefield);
        let shock = put(&mut g, p, "Shock", Zone::Hand);
        let negate = put(&mut g, p.other(), "Negate", Zone::Hand);
        cast(&mut g, shock, Some(Target::Player(p.other())));
        g.prepare_triggers();
        act(&mut g, Action::Pass);
        cast(&mut g, negate, Some(Target::Spell(shock)));
        super::super::contracts::branch_contract(g.clone());
        settle(&mut g);
        assert_eq!(g.object_view(shock).zone, Zone::Graveyard);
        assert_eq!(fra_thopters(&g, p).len(), 1);
    }
}

#[test]
fn koth_landfall_damage_uses_its_sources_lifelink() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let koth = put(&mut g, p, "Koth, the Geomancer", Zone::Battlefield);
        g.state.objects[koth as usize].gained_keywords = Keywords::LIFELINK;
        let land = put(&mut g, p, "Mountain", Zone::Hand);
        let own_life = g.state.players[p.index()].life;
        let enemy_life = g.state.players[p.other().index()].life;
        g.state.players[p.index()].mana = [0; 6];
        g.move_object(land, Zone::Battlefield);
        g.prepare_triggers();
        super::super::contracts::branch_contract(g.clone());
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, own_life + 1);
        assert_eq!(g.state.players[p.other().index()].life, enemy_life - 1);
        assert_eq!(g.state.players[p.index()].mana[3], 1);
    }
}
