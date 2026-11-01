//! Paid rules outcomes for the shared land/Saga cohort.
use super::*;
fn card(index: usize) -> Card {
    Card::catalogue().nth(index).unwrap()
}
fn step(g: &mut Game, step: Step) {
    g.begin_step(step);
    g.give_priority();
    g.prepare_triggers();
}
fn finish(g: &mut Game) {
    for _ in 0..64 {
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.resolving.is_none()
            && g.state.pending_triggers.is_empty()
            && g.state.ordering_triggers.is_empty()
        {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("land/Saga resolution budget {:?}", g.state.pending);
}
fn tokens(g: &Game, p: Player, identity: usize) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter(|(_, o)| {
            o.zone == Zone::Battlefield && o.controller == p && o.card == card(identity)
        })
        .map(|(id, _)| id as CardId)
        .collect()
}
fn start(g: &mut Game, id: CardId) {
    act(
        g,
        Action::AnnounceActivate {
            card: id,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
}
fn saga(g: &mut Game, p: Player) -> CardId {
    let id = put(g, p, "The Misty Mountains Cold", Zone::Hand);
    g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 2];
    cast(g, id, None);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    id
}
fn until_trigger(g: &mut Game) {
    for _ in 0..12 {
        if g.state
            .stack
            .last()
            .is_some_and(|s| matches!(s.kind, StackKind::TriggeredAbility { .. }))
        {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("chapter never stacked");
}
#[test]
fn lonely_actual_land_entry_equipment_control_and_intrinsic_red_mana() {
    for p in [Player::P0, Player::P1] {
        for (own, token, ready) in [
            (false, false, false),
            (true, false, true),
            (true, true, true),
        ] {
            let mut g = scenario(p);
            let owner = if own { p } else { p.other() };
            if token {
                g.create_tokens(owner, card(5406), 1, false);
                finish(&mut g);
            } else {
                put(&mut g, owner, "Goldvein Pick", Zone::Battlefield);
            }
            let id = put(&mut g, p, "The Lonely Mountain", Zone::Hand);
            act(&mut g, Action::PlayLand { card: id });
            assert_eq!(g.object_view(id).tapped, !ready);
            assert!(g.has_subtype(id, "Mountain"));
            g.state.players[p.index()].mana = [0; 6];
            if ready {
                act(&mut g, Action::TapMana { card: id });
                assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 1, 0, 0]);
            } else {
                reject(&mut g, Action::TapMana { card: id });
            }
        }
        let mut g = scenario(p);
        put(&mut g, p, "Thrór's Map", Zone::Battlefield);
        let id = put(&mut g, p, "The Lonely Mountain", Zone::Hand);
        act(&mut g, Action::PlayLand { card: id });
        assert!(
            g.object_view(id).tapped,
            "an ordinary artifact is not Equipment"
        );
    }
}
#[test]
fn lonely_paid_activation_live_equipment_count_generic_floor_and_exact_dwarf() {
    for p in [Player::P0, Player::P1] {
        for n in [0, 1, 3, 4, 6] {
            let mut g = scenario(p);
            let id = put(&mut g, p, "The Lonely Mountain", Zone::Battlefield);
            for _ in 0..n {
                put(&mut g, p, "Goldvein Pick", Zone::Battlefield);
            }
            put(&mut g, p.other(), "Goldvein Pick", Zone::Battlefield);
            let generic = 4u32.saturating_sub(n);
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, generic + 1];
            reject(
                &mut g,
                Action::AnnounceActivate {
                    card: id,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            g.state.players[p.index()].mana = [0, 0, 0, 1, 0, generic];
            start(&mut g, id);
            if n == 1 && p == Player::P0 {
                contracts::branch_contract(g.clone());
            }
            reject(
                &mut g,
                Action::PayCosts {
                    payment: [0, 0, 0, 0, 0, generic + 1],
                },
            );
            pay(&mut g);
            assert!(g.object_view(id).tapped);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            finish(&mut g);
            let dwarves = tokens(&g, p, 5407);
            assert_eq!(dwarves.len(), 1);
            let dwarf = g.object_view(dwarves[0]);
            assert_eq!(stats(&g, dwarves[0]), (2, 2));
            assert_eq!(dwarf.colors, 8);
            assert!(dwarf.has_subtype("Dwarf"));
            assert_eq!(dwarf.keywords, Keywords::NONE);
            assert!(!dwarf.tapped);
        }
    }
}
#[test]
fn lonely_tap_sorcery_stack_and_current_control_restrictions() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let id = put(&mut g, p, "The Lonely Mountain", Zone::Battlefield);
        let action = Action::AnnounceActivate {
            card: id,
            ability: 0,
            counter_payment: Counters::default(),
        };
        g.state.objects[id as usize].tapped = true;
        reject(&mut g, action.clone());
        g.state.objects[id as usize].tapped = false;
        g.state.active = p.other();
        reject(&mut g, action.clone());
        g.state.active = p;
        g.state.step = Step::BeginCombat;
        reject(&mut g, action.clone());
        g.state.step = Step::Main1;
        let shock = put(&mut g, p, "Shock", Zone::Hand);
        cast(&mut g, shock, Some(Target::Player(p.other())));
        reject(&mut g, action.clone());
        finish(&mut g);
        g.state.priority = p;
        g.state.objects[id as usize].controller = p.other();
        reject(&mut g, action);
    }
}
#[test]
fn lonely_announced_reduction_locks_before_equipment_sacrificed_for_mana() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let id = put(&mut g, p, "The Lonely Mountain", Zone::Battlefield);
        let e = put(&mut g, p, "Goldvein Pick", Zone::Battlefield);
        let object = g.object_ref(e);
        g.change_object_form(
            object,
            crate::cards::AnimationSpec {
                artifact: true,
                creature_types: crate::cards::AnimationCreatureTypes::Named(&["Goblin"]),
                base: Some((2, 2)),
                keywords: Keywords::NONE,
                duration: crate::cards::AnimationDuration::EndOfTurn,
            },
            characteristic_layers::AnimationTypeMode::AddCreature,
        );
        assert!(g.has_subtype(e, "Goblin") && g.has_subtype(e, "Equipment"));
        let bolg = put(&mut g, p, "Bolg's Company", Zone::Battlefield);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
        start(&mut g, id);
        act(
            &mut g,
            Action::AnnounceMana {
                card: bolg,
                ability: 0,
                counter_payment: Counters::default(),
                production: [0, 0, 1, 1, 0, 0],
            },
        );
        act(&mut g, Action::ChooseAdditionalCost { object });
        pay(&mut g);
        assert_eq!(g.object_view(e).zone, Zone::Graveyard);
        contracts::branch_contract(g.clone());
        pay(&mut g);
        finish(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0, 0, 1, 0, 0, 0]);
        assert_eq!(tokens(&g, p, 5407).len(), 1);
    }
}
#[test]
fn lonely_borrowed_ability_counts_equipment_of_actual_activator() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let land = put(&mut g, p.other(), "The Lonely Mountain", Zone::Battlefield);
        let forge = crate::engine::territory_forge_tests::forge(&mut g, p, land);
        for _ in 0..4 {
            put(&mut g, p, "Goldvein Pick", Zone::Battlefield);
        }
        let ability = g
            .borrowed_ability_views()
            .into_iter()
            .find(|a| a.source.card == forge && a.ability == 0)
            .unwrap();
        g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 0];
        act(
            &mut g,
            Action::AnnounceBorrowedAbility {
                card: forge,
                exiled: ability.exiled,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut g);
        finish(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert!(g.object_view(forge).tapped);
        assert_eq!(tokens(&g, p, 5407).len(), 1);
    }
}
#[test]
fn misty_paid_chapters_one_through_four_and_exact_dragon_same_resolution() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let id = saga(&mut g, p);
        finish(&mut g);
        assert_eq!(g.counter_count(id, CounterKind::Lore), 1);
        for chapter in 1..=4 {
            if chapter > 1 {
                step(&mut g, Step::Main1);
                finish(&mut g);
            }
            assert_eq!(tokens(&g, p, 5302).len(), chapter);
            assert_eq!(
                g.object_view(id).zone,
                if chapter == 4 {
                    Zone::Graveyard
                } else {
                    Zone::Battlefield
                }
            );
            assert_eq!(tokens(&g, p, 5416).len(), usize::from(chapter == 4));
        }
        let dragon = tokens(&g, p, 5416)[0];
        assert_eq!(stats(&g, dragon), (6, 6));
        assert_eq!(g.object_view(dragon).colors, 8);
        assert!(g.has_subtype(dragon, "Dragon"));
        assert!(g.object_view(dragon).keywords.contains(Keywords::FLYING));
        assert!(!g.object_view(dragon).tapped);
    }
}
#[test]
fn misty_spent_treasures_final_chapter_sba_never_creates_dragon() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let id = saga(&mut g, p);
        finish(&mut g);
        for chapter in 1..=4 {
            if chapter > 1 {
                step(&mut g, Step::Main1);
                finish(&mut g);
            }
            assert_eq!(tokens(&g, p, 5302).len(), 1);
            let treasure = tokens(&g, p, 5302)[0];
            choose(
                &mut g,
                |a| matches!(a,Action::ActivateMana{card,..}if *card==treasure),
            );
            assert!(tokens(&g, p, 5416).is_empty());
        }
        assert_eq!(g.object_view(id).zone, Zone::Graveyard);
    }
}
#[test]
fn misty_current_treasure_count_includes_tokens_and_ignores_opponent() {
    for p in [Player::P0, Player::P1] {
        for own in [false, true] {
            let mut g = scenario(p);
            g.create_tokens(if own { p } else { p.other() }, card(5302), 3, false);
            finish(&mut g);
            let id = saga(&mut g, p);
            finish(&mut g);
            assert_eq!(tokens(&g, p, 5416).len(), usize::from(own));
            assert_eq!(
                g.object_view(id).zone,
                if own {
                    Zone::Graveyard
                } else {
                    Zone::Battlefield
                }
            );
        }
    }
}
#[test]
fn misty_departed_or_reentered_or_uncontrolled_source_never_pays_old_chapter() {
    for p in [Player::P0, Player::P1] {
        for mode in 0..3 {
            let mut g = scenario(p);
            g.create_tokens(p, card(5302), 3, false);
            finish(&mut g);
            let id = saga(&mut g, p);
            until_trigger(&mut g);
            let old = g.object_ref(id);
            if mode == 2 {
                g.state.objects[id as usize].controller = p.other();
            } else {
                g.move_object(id, Zone::Hand);
                if mode == 1 {
                    g.move_object(id, Zone::Battlefield);
                }
            }
            // Entry reuses the physical card but has a new exact incarnation.
            if mode == 1 {
                assert_ne!(g.object_ref(id), old);
                g.state.pending_triggers.clear();
                g.state.ordering_triggers.clear();
            }
            finish(&mut g);
            assert_eq!(tokens(&g, p, 5302).len(), 4);
            assert!(tokens(&g, p, 5416).is_empty());
            assert_eq!(
                g.object_view(id).zone,
                if mode == 0 {
                    Zone::Hand
                } else {
                    Zone::Battlefield
                }
            );
        }
    }
}
#[test]
fn misty_sacrifice_replaced_by_exile_still_completes_without_reflexive_trigger() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p.other(), "Rest in Peace", Zone::Battlefield);
        g.create_tokens(p, card(5302), 3, false);
        finish(&mut g);
        let id = saga(&mut g, p);
        until_trigger(&mut g);
        contracts::branch_contract(g.clone());
        finish(&mut g);
        assert_eq!(g.object_view(id).zone, Zone::Exile);
        assert_eq!(tokens(&g, p, 5302).len(), 4);
        assert_eq!(tokens(&g, p, 5416).len(), 1);
        assert!(g.state.pending_triggers.is_empty());
        assert!(g.state.stack.is_empty());
    }
}
#[test]
fn lonely_current_copy_types_and_control_change_entry_and_reduction() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let e = put(&mut g, p.other(), "Goldvein Pick", Zone::Battlefield);
        g.state.objects[e as usize].controller = p;
        let id = put(&mut g, p, "The Lonely Mountain", Zone::Hand);
        act(&mut g, Action::PlayLand { card: id });
        assert!(!g.object_view(id).tapped);
        g.apply_temporary_copy(
            g.object_ref(e),
            CopyRecipe::new(Card::from_name("Bear Cub").unwrap()),
            CopyDeadline::EndOfTurn,
        );
        assert!(!g.has_subtype(e, "Equipment"));
        g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 3];
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: id,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 4];
        start(&mut g, id);
        pay(&mut g);
        finish(&mut g);
        assert_eq!(tokens(&g, p, 5407).len(), 1);
        let second = put(&mut g, p, "The Lonely Mountain", Zone::Hand);
        g.state.land_played = false;
        act(&mut g, Action::PlayLand { card: second });
        assert!(g.object_view(second).tapped);
    }
}
#[test]
fn misty_sacrifice_competing_replacements_pause_replay_and_owned_continuation() {
    for effect in [
        ZoneReplacementKind::Exile,
        ZoneReplacementKind::RevealShuffle,
    ] {
        let p = Player::P0;
        let mut g = scenario(p);
        put(&mut g, p.other(), "Rest in Peace", Zone::Battlefield);
        g.create_tokens(p, card(5302), 3, false);
        finish(&mut g);
        let id = saga(&mut g, p);
        until_trigger(&mut g);
        // An in-place copy retains this exact source incarnation and the queued
        // chapter, while adding the Colossus's actual self-shuffle replacement.
        g.apply_temporary_copy(
            g.object_ref(id),
            CopyRecipe::new(Card::from_name("Darksteel Colossus").unwrap()),
            CopyDeadline::EndOfTurn,
        );
        for _ in 0..8 {
            if g.legal_actions()
                .iter()
                .any(|a| matches!(a, Action::ChooseZoneReplacement { .. }))
            {
                break;
            }
            choose(&mut g, |a| matches!(a, Action::Pass));
        }
        assert!(
            g.legal_actions()
                .contains(&Action::ChooseZoneReplacement { effect })
        );
        assert!(tokens(&g, p, 5416).is_empty());
        contracts::branch_contract(g.clone());
        act(&mut g, Action::ChooseZoneReplacement { effect });
        finish(&mut g);
        assert_eq!(
            g.object_view(id).zone,
            if effect == ZoneReplacementKind::Exile {
                Zone::Exile
            } else {
                Zone::Library
            }
        );
        assert_eq!(
            tokens(&g, p, 5302).len(),
            4,
            "replay must not duplicate Treasure"
        );
        assert_eq!(tokens(&g, p, 5416).len(), 1);
    }
}
