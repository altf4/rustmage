//! Actual activated-creature events and exact delayed blink continuations.
use super::*;
use crate::cards::{AnimationCreatureTypes, AnimationDuration, AnimationSpec};
fn hand(g: &Game, p: Player) -> usize {
    g.state
        .objects
        .iter()
        .filter(|o| o.zone == Zone::Hand && o.owner == p)
        .count()
}
const FIXTURES: &[&str] = &[
    "Elrond, Moon-Reader",
    "Forest",
    "Bear Cub",
    "Llanowar Elves",
    "Goldvein Pick",
    "Witness Protection",
    "Disenchant",
    "Banishing Light",
    "Adaptive Automaton",
    "Authority of the Consuls",
    "Territory Forge",
    "Prophetic Prism",
    "Rubble Rouser",
    "Candy Grapple",
    "Silverquill, the Disputant",
    "Chitin Gravestalker",
    "Troop of Ponies",
    "@token/hob-dwarf",
    "Lotus Ring",
    "Fíli the Pathfinder",
    "Axgard Cavalry",
];
fn fixture(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    assert!(FIXTURES.contains(&name), "unlisted {name}");
    put(g, p, name, zone)
}
fn quiet(g: &Game) -> bool {
    g.state.pending == Pending::Priority
        && g.state.stack.is_empty()
        && g.state.pending_triggers.is_empty()
        && g.state.ordering_triggers.is_empty()
        && g.state.resolving.is_none()
}
fn until(g: &mut Game, done: impl Fn(&Game) -> bool) {
    for _ in 0..96 {
        if done(g) {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("Elrond budget {:?}", g.state.pending)
}
pub(super) fn finish(g: &mut Game) {
    until(g, quiet)
}
fn paid(g: &mut Game, p: Player, name: &str, mana: Mana, target: Option<Target>) -> CardId {
    let id = fixture(g, p, name, Zone::Hand);
    g.state.players[p.index()].mana = mana;
    cast(g, id, target);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    id
}
fn elrond(g: &mut Game, p: Player) -> CardId {
    let id = paid(g, p, "Elrond, Moon-Reader", [0, 1, 0, 0, 0, 2], None);
    finish(g);
    assert_eq!(stats(g, id), (3, 3));
    assert!(g.has_subtype(id, "Elf"));
    assert!(g.has_subtype(id, "Noble"));
    id
}
fn activate(g: &mut Game, id: CardId, targets: &[CardId]) {
    let p = g.state.objects[id as usize].controller;
    g.state.players[p.index()].mana = [0, 2, 0, 0, 0, 5];
    act(
        g,
        Action::AnnounceActivate {
            card: id,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    for &id in targets {
        act(
            g,
            Action::ChooseTarget {
                target: Target::Card(id),
            },
        );
    }
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
    pay(g);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
}
pub(super) fn mana(g: &mut Game, id: CardId) {
    choose(
        g,
        |a| matches!(a,Action::ActivateMana{card,..}|Action::TapMana{card}|Action::TapGrantedMana{card,..} if *card==id),
    );
}
fn end(g: &mut Game) {
    g.begin_step(Step::End);
    g.give_priority();
    g.prepare_triggers();
}
#[test]
fn elrond_paid_own_zero_target_activation_draws_once_even_if_countered_or_source_gone() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let id = elrond(&mut g, p);
        let before = hand(&g, p);
        activate(&mut g, id, &[]);
        assert_eq!(g.state.triggered_this_turn.len(), 1);
        assert_eq!(g.state.stack.len(), 2);
        let ability = g
            .state
            .stack
            .iter()
            .position(|x| matches!(x.kind, StackKind::ActivatedAbility { .. }))
            .unwrap();
        assert!(g.counter_stack_item(g.state.stack[ability].id, false));
        g.move_object(id, Zone::Graveyard);
        finish(&mut g);
        assert_eq!(hand(&g, p), before + 1);
    }
}
#[test]
fn elrond_direct_mana_once_suppression_control_and_new_incarnation() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let id = elrond(&mut g, p);
        let a = fixture(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        let b = fixture(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        let before = hand(&g, p);
        mana(&mut g, a);
        finish(&mut g);
        assert_eq!(hand(&g, p), before + 1);
        mana(&mut g, b);
        finish(&mut g);
        assert_eq!(hand(&g, p), before + 1);
        let aura = paid(
            &mut g,
            p,
            "Witness Protection",
            [0, 1, 0, 0, 0, 0],
            Some(Target::Card(id)),
        );
        finish(&mut g);
        g.state.objects[a as usize].tapped = false;
        mana(&mut g, a);
        finish(&mut g);
        g.move_object(aura, Zone::Graveyard);
        g.state.objects[a as usize].tapped = false;
        mana(&mut g, a);
        finish(&mut g);
        assert_eq!(hand(&g, p), before + 1); // the fixture adds and then casts its Aura
        let q = p.other();
        g.record_control(g.object_ref(id), q, None);
        g.refresh_controllers();
        g.state.priority = q;
        let c = fixture(&mut g, q, "Llanowar Elves", Zone::Battlefield);
        let theirs = hand(&g, q);
        mana(&mut g, c);
        finish(&mut g);
        assert_eq!(hand(&g, q), theirs);
        g.move_object(id, Zone::Exile);
        g.move_object(id, Zone::Battlefield);
        g.state.priority = p;
        g.state.objects[a as usize].tapped = false;
        mana(&mut g, a);
        finish(&mut g);
        assert_eq!(hand(&g, p), before + 2);
        g.begin_turn();
        g.state.active = p;
        g.state.priority = p;
        g.state.pending = Pending::Priority;
        g.state.step = Step::Main1;
        g.state.objects[a as usize].tapped = false;
        g.state.objects[a as usize].summoning_sick = false;
        let n = hand(&g, p);
        mana(&mut g, a);
        finish(&mut g);
        assert_eq!(hand(&g, p), n + 1);
    }
}
#[test]
fn elrond_noncreature_ability_and_opponent_creature_excluded_then_current_creature_qualifies() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let id = elrond(&mut g, p);
        let forest = fixture(&mut g, p, "Forest", Zone::Battlefield);
        let before = hand(&g, p);
        mana(&mut g, forest);
        finish(&mut g);
        assert_eq!(hand(&g, p), before);
        let q = p.other();
        let elf = fixture(&mut g, q, "Llanowar Elves", Zone::Battlefield);
        g.state.priority = q;
        mana(&mut g, elf);
        finish(&mut g);
        assert_eq!(hand(&g, p), before);
        g.state.priority = p;
        // An actual creature's nonmana equip is an activation, regardless of its printed card type.
        let pick = fixture(&mut g, p, "Goldvein Pick", Zone::Battlefield);
        g.change_object_form(
            g.object_ref(pick),
            AnimationSpec {
                artifact: false,
                base: Some((2, 2)),
                creature_types: AnimationCreatureTypes::Unchanged,
                keywords: Keywords::NONE,
                duration: AnimationDuration::EndOfTurn,
            },
            characteristic_layers::AnimationTypeMode::AddCreature,
        );
        choose(
            &mut g,
            |a| matches!(a,Action::Activate{card,target:Some(Target::Card(t)),..}if *card==pick&&*t==id),
        );
        finish(&mut g);
        assert_eq!(hand(&g, p), before + 1);
    }
}
#[test]
fn elrond_paid_blink_zero_one_two_and_source_departure_owner_return() {
    for p in [Player::P0, Player::P1] {
        for n in 0..=2 {
            let mut g = scenario(p);
            let id = elrond(&mut g, p);
            let a = fixture(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            g.record_control(g.object_ref(a), p, None);
            g.refresh_controllers();
            let b = fixture(&mut g, p, "Goldvein Pick", Zone::Battlefield);
            let ids = [a, b];
            let old = [g.object_ref(a), g.object_ref(b)];
            activate(&mut g, id, &ids[..n]);
            g.move_object(id, Zone::Graveyard);
            finish(&mut g);
            for (i, &target) in ids.iter().enumerate() {
                assert_eq!(
                    g.object_view(target).zone,
                    if i < n {
                        Zone::Exile
                    } else {
                        Zone::Battlefield
                    }
                );
            }
            end(&mut g);
            finish(&mut g);
            for i in 0..n {
                assert_eq!(g.object_view(ids[i]).zone, Zone::Battlefield);
                assert_ne!(g.object_ref(ids[i]), old[i]);
                assert_eq!(
                    g.object_view(ids[i]).controller,
                    if i == 0 { p.other() } else { p }
                );
            }
        }
    }
}
#[test]
fn elrond_target_legality_failed_payment_partial_and_all_stale() {
    for p in [Player::P0, Player::P1] {
        for stale in 0..=2 {
            let mut g = scenario(p);
            let id = elrond(&mut g, p);
            let land = fixture(&mut g, p, "Forest", Zone::Battlefield);
            let foe = fixture(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let a = fixture(&mut g, p, "Bear Cub", Zone::Battlefield);
            let b = fixture(&mut g, p, "Goldvein Pick", Zone::Battlefield);
            g.state.players[p.index()].mana = [0, 2, 0, 0, 0, 5];
            act(
                &mut g,
                Action::AnnounceActivate {
                    card: id,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            for illegal in [id, land, foe] {
                reject(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(illegal),
                    },
                );
            }
            for target in [a, b] {
                act(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(target),
                    },
                );
            }
            reject(&mut g, Action::PayCosts { payment: [0; 6] });
            assert!(g.state.triggered_this_turn.is_empty());
            pay(&mut g);
            for target in [a, b].into_iter().take(stale) {
                g.move_object(target, Zone::Hand);
            }
            finish(&mut g);
            assert_eq!(
                g.object_view(a).zone,
                if stale > 0 { Zone::Hand } else { Zone::Exile }
            );
            assert_eq!(
                g.object_view(b).zone,
                if stale > 1 { Zone::Hand } else { Zone::Exile }
            );
            end(&mut g);
            finish(&mut g);
            assert_eq!(
                g.object_view(a).zone,
                if stale > 0 {
                    Zone::Hand
                } else {
                    Zone::Battlefield
                }
            );
            assert_eq!(
                g.object_view(b).zone,
                if stale > 1 {
                    Zone::Hand
                } else {
                    Zone::Battlefield
                }
            );
        }
    }
}
#[test]
fn elrond_batched_exile_before_linked_entry_choice_and_owned_restore() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let id = elrond(&mut g, p);
        let automaton = fixture(&mut g, p.other(), "Adaptive Automaton", Zone::Battlefield);
        let prison = paid(&mut g, p, "Banishing Light", [1, 0, 0, 0, 0, 2], None);
        until(&mut g, |g| {
            g.legal_actions().contains(&Action::ChooseTarget {
                target: Target::Card(automaton),
            })
        });
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(automaton),
            },
        );
        finish(&mut g);
        let second = fixture(&mut g, p, "Bear Cub", Zone::Battlefield);
        activate(&mut g, id, &[prison, second]);
        until(&mut g, |g| {
            g.actor()
                .is_some_and(|p| g.observe(p).entry_value.is_some())
        });
        assert_eq!(g.object_view(prison).zone, Zone::Exile);
        assert_eq!(g.object_view(second).zone, Zone::Exile);
        assert_eq!(g.state.delayed_returns.as_ref().unwrap().len(), 1);
        assert_eq!(
            g.state.delayed_returns.as_ref().unwrap()[0]
                .view
                .returning
                .additional_returning
                .as_ref()
                .unwrap()
                .len(),
            1
        );
        contracts::branch_contract(g.clone());
        let saved = g.snapshot();
        let mut raw = g.clone();
        let mut indexed = g.clone();
        drop(g);
        let action = raw
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::ChooseEntryValue { .. }))
            .unwrap();
        let actor = raw.actor().unwrap();
        let ix = indexed
            .legal_actions()
            .iter()
            .position(|a| *a == action)
            .unwrap();
        act(&mut raw, action);
        indexed.apply_index(actor, indexed.revision(), ix).unwrap();
        finish(&mut raw);
        finish(&mut indexed);
        assert_eq!(raw.snapshot(), indexed.snapshot());
        raw.restore(&saved).unwrap();
        choose(&mut raw, |a| matches!(a, Action::ChooseEntryValue { .. }));
        finish(&mut raw);
        end(&mut raw);
        until(&mut raw, |g| {
            quiet(g) || matches!(g.state.pending, Pending::Targets { .. })
        });
        if matches!(raw.state.pending, Pending::Targets { .. }) {
            act(
                &mut raw,
                Action::ChooseTarget {
                    target: Target::Card(automaton),
                },
            );
        }
        finish(&mut raw);
        assert_eq!(raw.object_view(second).zone, Zone::Battlefield);
        assert_eq!(raw.object_view(prison).zone, Zone::Battlefield);
    }
}
#[test]
fn elrond_two_observers_independent_limits_and_copied_activation_is_not_activated() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let a = elrond(&mut g, p);
        let mut recipe = g.copiable_recipe(g.object_ref(a));
        recipe.overrides = Some(CopyOverrides {
            legendary: Some(false),
            ..CopyOverrides::NONE
        });
        g.set_copy_recipe(g.object_ref(a), recipe);
        let b = elrond(&mut g, p);
        let before = hand(&g, p);
        activate(&mut g, b, &[]);
        assert_eq!(
            g.state
                .triggered_this_turn
                .iter()
                .filter(|u| u.source.card == a || u.source.card == b)
                .count(),
            2
        );
        let mut copied = g
            .state
            .stack
            .iter()
            .find(|s| matches!(s.kind, StackKind::ActivatedAbility { .. }))
            .unwrap()
            .clone();
        copied.is_copy = true;
        copied.id = g.next_stack_id();
        g.push_stack_item(copied);
        finish(&mut g);
        assert_eq!(hand(&g, p), before + 2);
    }
}
#[test]
fn elrond_borrowed_ability_uses_actual_source_current_creature() {
    for p in [Player::P0, Player::P1] {
        for borrowed_name in ["Prophetic Prism", "Llanowar Elves"] {
            for creature in [false, true] {
                let mut g = scenario(p);
                elrond(&mut g, p);
                let blueprint = fixture(&mut g, p.other(), borrowed_name, Zone::Battlefield);
                if borrowed_name == "Llanowar Elves" {
                    // Forge can exile artifacts/lands. The temporary artifact type
                    // makes this creature blueprint a legal actual paid ETB target.
                    g.change_object_form(
                        g.object_ref(blueprint),
                        AnimationSpec {
                            artifact: true,
                            base: None,
                            creature_types: AnimationCreatureTypes::Unchanged,
                            keywords: Keywords::NONE,
                            duration: AnimationDuration::EndOfTurn,
                        },
                        characteristic_layers::AnimationTypeMode::PreserveCardTypes,
                    );
                }
                g.state.players[p.index()].mana = [8; 6];
                let forge = crate::engine::territory_forge_tests::forge(&mut g, p, blueprint);
                let acquired = g
                    .borrowed_ability_views()
                    .into_iter()
                    .find(|a| a.source.card == forge)
                    .unwrap();
                if creature {
                    g.change_object_form(
                        g.object_ref(forge),
                        AnimationSpec {
                            artifact: false,
                            base: Some((3, 3)),
                            creature_types: AnimationCreatureTypes::Unchanged,
                            keywords: Keywords::NONE,
                            duration: AnimationDuration::EndOfTurn,
                        },
                        characteristic_layers::AnimationTypeMode::AddCreature,
                    );
                }
                g.state.objects[forge as usize].summoning_sick = false;
                let before = hand(&g, p);
                g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
                let production = if borrowed_name == "Prophetic Prism" {
                    [0, 1, 0, 0, 0, 0]
                } else {
                    [0, 0, 0, 0, 1, 0]
                };
                act(
                    &mut g,
                    Action::AnnounceBorrowedMana {
                        card: forge,
                        exiled: acquired.exiled,
                        ability: acquired.ability,
                        counter_payment: Counters::default(),
                        production,
                    },
                );
                if g.state.announcing.is_some() {
                    pay(&mut g);
                }
                finish(&mut g);
                assert_eq!(hand(&g, p), before + usize::from(creature));
            }
        }
    }
}
#[test]
fn elrond_return_replacement_and_exact_exile_generation() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = elrond(&mut g, p);
        fixture(
            &mut g,
            p.other(),
            "Authority of the Consuls",
            Zone::Battlefield,
        );
        let a = fixture(&mut g, p, "Bear Cub", Zone::Battlefield);
        let b = fixture(&mut g, p, "Bear Cub", Zone::Battlefield);
        activate(&mut g, source, &[a, b]);
        finish(&mut g);
        g.move_object(a, Zone::Hand);
        g.move_object(a, Zone::Exile);
        end(&mut g);
        finish(&mut g);
        assert_eq!(g.object_view(a).zone, Zone::Exile);
        assert_eq!(g.object_view(b).zone, Zone::Battlefield);
        assert!(g.object_view(b).tapped);
    }
}
#[test]
fn elrond_creature_card_cycling_excluded_and_self_sacrifice_uses_departure_facts() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        elrond(&mut g, p);
        let cycling = fixture(&mut g, p, "Chitin Gravestalker", Zone::Hand);
        let before = g.state.players[p.index()].cards_drawn_this_turn;
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        choose(
            &mut g,
            |a| matches!(a,Action::Activate{card,..}if *card==cycling),
        );
        finish(&mut g);
        assert_eq!(g.object_view(cycling).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, before + 1);
        assert!(g.state.triggered_this_turn.is_empty());
        let pony = fixture(&mut g, p, "Troop of Ponies", Zone::Battlefield);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        choose(
            &mut g,
            |a| matches!(a,Action::Activate{card,..}if *card==pony),
        );
        assert_eq!(g.object_view(pony).zone, Zone::Graveyard);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectSearch { .. })
        });
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, before + 2);
        contracts::branch_contract(g.clone());
    }
}
#[test]
fn elrond_token_blink_never_returns_and_activation_after_end_waits_next_beginning() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let id = elrond(&mut g, p);
        g.create_tokens(p, Card::from_name("@token/hob-dwarf").unwrap(), 1, false);
        finish(&mut g);
        let token = g
            .state
            .objects
            .iter()
            .enumerate()
            .find_map(|(i, o)| (o.zone == Zone::Battlefield && o.is_token()).then_some(i as CardId))
            .unwrap();
        let bear = fixture(&mut g, p, "Bear Cub", Zone::Battlefield);
        end(&mut g);
        activate(&mut g, id, &[token, bear]);
        finish(&mut g);
        assert!(!g.on_battlefield(token));
        assert_eq!(g.object_view(bear).zone, Zone::Exile);
        assert_eq!(g.state.delayed_returns.as_ref().unwrap().len(), 1);
        g.begin_step(Step::Main1);
        g.give_priority();
        finish(&mut g);
        assert_eq!(g.object_view(bear).zone, Zone::Exile);
        end(&mut g);
        finish(&mut g);
        assert_eq!(g.object_view(bear).zone, Zone::Battlefield);
        assert!(!g.on_battlefield(token));
    }
}
#[test]
fn elrond_borrowed_nonmana_uses_creature_forge_not_equipment_definition() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let id = elrond(&mut g, p);
        let equipment = fixture(&mut g, p.other(), "Goldvein Pick", Zone::Battlefield);
        g.state.players[p.index()].mana = [8; 6];
        let forge = crate::engine::territory_forge_tests::forge(&mut g, p, equipment);
        let acquired = g
            .borrowed_ability_views()
            .into_iter()
            .find(|a| a.source.card == forge && a.ability == 0)
            .unwrap();
        g.change_object_form(
            g.object_ref(forge),
            AnimationSpec {
                artifact: false,
                base: Some((3, 3)),
                creature_types: AnimationCreatureTypes::Unchanged,
                keywords: Keywords::NONE,
                duration: AnimationDuration::EndOfTurn,
            },
            characteristic_layers::AnimationTypeMode::AddCreature,
        );
        let before = hand(&g, p);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        act(
            &mut g,
            Action::AnnounceBorrowedAbility {
                card: forge,
                exiled: acquired.exiled,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(id),
            },
        );
        pay(&mut g);
        finish(&mut g);
        assert_eq!(hand(&g, p), before + 1);
    }
}
#[test]
fn elrond_granted_mana_sacrificed_creature_qualifies_not_artifact_grantor() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        elrond(&mut g, p);
        let bear = fixture(&mut g, p, "Bear Cub", Zone::Battlefield);
        let ring = fixture(&mut g, p, "Lotus Ring", Zone::Battlefield);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
        choose(
            &mut g,
            |a| matches!(a,Action::Activate{card,target:Some(Target::Card(t)),..}if *card==ring&&*t==bear),
        );
        finish(&mut g);
        assert!(g.state.triggered_this_turn.is_empty());
        let before = hand(&g, p);
        let grantor = g.object_ref(ring);
        act(
            &mut g,
            Action::TapGrantedMana {
                card: bear,
                grantor,
                production: [0, 3, 0, 0, 0, 0],
            },
        );
        assert_eq!(g.object_view(bear).zone, Zone::Graveyard);
        finish(&mut g);
        assert_eq!(hand(&g, p), before + 1);
        assert_eq!(g.state.players[p.index()].mana, [0, 3, 0, 0, 0, 0]);
    }
}
#[test]
fn elrond_resolution_rechecks_current_target_control() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let id = elrond(&mut g, p);
        let a = fixture(&mut g, p, "Bear Cub", Zone::Battlefield);
        let b = fixture(&mut g, p, "Goldvein Pick", Zone::Battlefield);
        activate(&mut g, id, &[a, b]);
        g.record_control(g.object_ref(a), p.other(), None);
        g.refresh_controllers();
        finish(&mut g);
        assert_eq!(g.object_view(a).zone, Zone::Battlefield);
        assert_eq!(g.object_view(a).controller, p.other());
        assert_eq!(g.object_view(b).zone, Zone::Exile);
        assert_eq!(g.state.delayed_returns.as_ref().unwrap().len(), 1);
        end(&mut g);
        finish(&mut g);
        assert_eq!(g.object_view(b).zone, Zone::Battlefield);
    }
}
#[test]
fn elrond_countered_draw_trigger_still_consumes_exact_once_limit() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let id = elrond(&mut g, p);
        let before = hand(&g, p);
        activate(&mut g, id, &[]);
        let trigger = g
            .state
            .stack
            .iter()
            .find(|s| matches!(s.kind, StackKind::TriggeredAbility { .. }))
            .unwrap()
            .id;
        assert!(g.counter_stack_item(trigger, false));
        finish(&mut g);
        assert_eq!(hand(&g, p), before);
        activate(&mut g, id, &[]);
        finish(&mut g);
        assert_eq!(hand(&g, p), before);
        assert_eq!(
            g.state
                .triggered_this_turn
                .iter()
                .filter(|u| u.source.card == id)
                .count(),
            1
        );
    }
}
#[test]
fn elrond_one_delayed_trigger_counter_leaves_entire_group_exiled() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = elrond(&mut g, p);
        let a = fixture(&mut g, p, "Bear Cub", Zone::Battlefield);
        let b = fixture(&mut g, p, "Goldvein Pick", Zone::Battlefield);
        activate(&mut g, source, &[a, b]);
        finish(&mut g);
        g.move_object(source, Zone::Graveyard);
        let rows = &g.observe(p).delayed_returns;
        assert_eq!(rows.len(), 1);
        assert_eq!(
            g.observe(p).delayed_returns,
            g.observe(p.other()).delayed_returns
        );
        let json = serde_json::to_value(&rows[0]).unwrap();
        assert_eq!(json["returning"]["object"]["card"], a);
        assert_eq!(
            json["returning"]["additional_returning"][0]["object"]["card"],
            b
        );
        assert!(
            json["returning"]["additional_returning"][0]
                .get("additional_returning")
                .is_none()
        );
        end(&mut g);
        assert_eq!(
            g.state
                .stack
                .iter()
                .filter(|s| s.kind == StackKind::DelayedReturn)
                .count(),
            1
        );
        let item = g.state.stack.last().unwrap();
        assert_eq!(
            item.return_link
                .as_ref()
                .unwrap()
                .additional_returning
                .as_ref()
                .unwrap()
                .len(),
            1
        );
        contracts::branch_contract(g.clone());
        let trigger = item.id;
        assert!(g.counter_stack_item(trigger, false));
        finish(&mut g);
        for id in [a, b] {
            assert_eq!(g.object_view(id).zone, Zone::Exile);
        }
        end(&mut g);
        finish(&mut g);
        for id in [a, b] {
            assert_eq!(g.object_view(id).zone, Zone::Exile);
        }
    }
}
#[test]
fn elrond_returned_observer_sees_other_simultaneous_returned_dwarf() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = elrond(&mut g, p);
        // The other Dwarf is first: a serial return would enter it BEFORE Fíli
        // exists, losing that trigger. One batch must collect both Fíli entry events.
        let dwarf = fixture(&mut g, p, "Axgard Cavalry", Zone::Battlefield);
        let fili = fixture(&mut g, p, "Fíli the Pathfinder", Zone::Battlefield);
        activate(&mut g, source, &[dwarf, fili]);
        finish(&mut g);
        end(&mut g);
        assert_eq!(g.state.stack.len(), 1);
        assert_eq!(g.state.stack[0].kind, StackKind::DelayedReturn);
        finish(&mut g);
        let tokens = g
            .state
            .objects
            .iter()
            .filter(|o| {
                o.zone == Zone::Battlefield
                    && o.controller == p
                    && o.is_token()
                    && o.card == Card::from_name("@token/hob-dwarf").unwrap()
            })
            .count();
        assert_eq!(
            tokens, 2,
            "Fíli sees itself AND the simultaneously returned other nontoken Dwarf"
        );
    }
}
#[test]
fn elrond_independent_activations_keep_separate_delayed_groups() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = elrond(&mut g, p);
        let a = fixture(&mut g, p, "Bear Cub", Zone::Battlefield);
        let b = fixture(&mut g, p, "Goldvein Pick", Zone::Battlefield);
        let c = fixture(&mut g, p, "Bear Cub", Zone::Battlefield);
        activate(&mut g, source, &[a, b]);
        finish(&mut g);
        activate(&mut g, source, &[c]);
        finish(&mut g);
        let rows = &g.observe(p).delayed_returns;
        assert_eq!(rows.len(), 2);
        assert!(rows[1].returning.additional_returning.is_none());
        assert!(
            serde_json::to_value(&rows[1]).unwrap()["returning"]
                .get("additional_returning")
                .is_none(),
            "legacy singleton shape"
        );
        end(&mut g);
        until(&mut g, |g| g.state.stack.len() == 2);
        assert!(
            g.state
                .stack
                .iter()
                .all(|s| s.kind == StackKind::DelayedReturn)
        );
        let first = g
            .state
            .stack
            .iter()
            .find(|s| {
                s.return_link
                    .as_ref()
                    .unwrap()
                    .additional_returning
                    .is_some()
            })
            .unwrap()
            .id;
        assert!(g.counter_stack_item(first, false));
        finish(&mut g);
        assert_eq!(g.object_view(a).zone, Zone::Exile);
        assert_eq!(g.object_view(b).zone, Zone::Exile);
        assert_eq!(g.object_view(c).zone, Zone::Battlefield);
    }
}
#[test]
fn elrond_grouped_return_entry_choice_owns_all_moves_until_batch_commit() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = elrond(&mut g, p);
        let automaton = fixture(&mut g, p, "Adaptive Automaton", Zone::Battlefield);
        let bear = fixture(&mut g, p, "Bear Cub", Zone::Battlefield);
        activate(&mut g, source, &[automaton, bear]);
        finish(&mut g);
        end(&mut g);
        until(&mut g, |g| {
            g.actor()
                .is_some_and(|p| g.observe(p).entry_value.is_some())
        });
        assert_eq!(g.object_view(automaton).zone, Zone::Exile);
        assert_eq!(g.object_view(bear).zone, Zone::Exile);
        contracts::branch_contract(g.clone());
        let saved = g.snapshot();
        let mut raw = g.clone();
        let mut indexed = g.clone();
        drop(g);
        let action = raw
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::ChooseEntryValue { .. }))
            .unwrap();
        let actor = raw.actor().unwrap();
        let index = indexed
            .legal_actions()
            .iter()
            .position(|a| *a == action)
            .unwrap();
        act(&mut raw, action);
        indexed
            .apply_index(actor, indexed.revision(), index)
            .unwrap();
        finish(&mut raw);
        finish(&mut indexed);
        assert_eq!(raw.snapshot(), indexed.snapshot());
        for id in [automaton, bear] {
            assert_eq!(raw.object_view(id).zone, Zone::Battlefield);
        }
        raw.restore(&saved).unwrap();
        assert_eq!(raw.object_view(bear).zone, Zone::Exile);
        choose(&mut raw, |a| matches!(a, Action::ChooseEntryValue { .. }));
        finish(&mut raw);
        for id in [automaton, bear] {
            assert_eq!(raw.object_view(id).zone, Zone::Battlefield);
        }
    }
}
