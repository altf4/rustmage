//! Frozen target/sacrifice/death power and controlled scaled amass outcomes.
use super::*;
use crate::cards::{AnimationCreatureTypes, AnimationDuration, AnimationSpec};
const FIXTURES: &[&str] = &[
    "Azog, Moria's Ruin",
    "Rhovanion Rampager",
    "Bear Cub",
    "Forest",
    "Goldvein Pick",
    "Giant Growth",
    "Murder",
    "Rest in Peace",
    "Doubling Season",
    "Elvish Archdruid",
    "Adaptive Automaton",
    "Banishing Light",
    "@token/goblin_army",
    "Doc Samson, Super Psychiatrist",
    "Ant-Man, Colony Commander",
    "Goblin Plate Mail",
];
fn fixture(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    assert!(FIXTURES.contains(&name), "unlisted fixture {name}");
    put(g, p, name, zone)
}
fn quiet(g: &Game) -> bool {
    g.state.pending == Pending::Priority
        && g.state.stack.is_empty()
        && g.state.pending_triggers.is_empty()
        && g.state.ordering_triggers.is_empty()
        && g.state.resolving.is_none()
}
fn until(g: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..64 {
        if ready(g) {
            return;
        }
        choose(g, |a| {
            matches!(
                a,
                Action::Pass | Action::PutTrigger { .. } | Action::ChooseReplacement { .. }
            )
        });
    }
    panic!("azog/rampager decision budget {:?}", g.state.pending);
}
fn finish(g: &mut Game) {
    for _ in 0..96 {
        if quiet(g) {
            return;
        }
        choose(g, |a| {
            matches!(
                a,
                Action::Pass
                    | Action::PutTrigger { .. }
                    | Action::ChooseArmy { .. }
                    | Action::ChooseReplacement { .. }
            )
        });
    }
    panic!("azog/rampager finish budget {:?}", g.state.pending);
}
fn paid(g: &mut Game, p: Player, name: &str, mana: Mana, target: Option<Target>) -> CardId {
    let id = fixture(g, p, name, Zone::Hand);
    g.state.players[p.index()].mana = mana;
    cast(g, id, target);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    id
}
fn source(g: &mut Game, p: Player, name: &str) -> CardId {
    paid(g, p, name, [0, 0, 1, 0, 0, 2], None)
}
fn bind(g: &mut Game, id: CardId) {
    act(
        g,
        Action::ChooseTarget {
            target: Target::Card(id),
        },
    );
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
}
fn azog(g: &mut Game, p: Player) -> CardId {
    let id = source(g, p, "Azog, Moria's Ruin");
    until(g, |g| g.legal_actions().contains(&Action::FinishTargets));
    id
}
fn rampager(g: &mut Game, p: Player) -> CardId {
    let id = source(g, p, "Rhovanion Rampager");
    finish(g);
    assert_eq!(stats(g, id), (3, 2));
    assert!(g.has_subtype(id, "Wolf"));
    id
}
fn attack(g: &mut Game, id: CardId) {
    g.state.objects[id as usize].summoning_sick = false;
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    act(g, Action::Attack { card: id });
    act(g, Action::FinishAttackers);
}
fn types(g: &mut Game, id: CardId, names: &'static [&'static str]) {
    let recipient = g.object_ref(id);
    let timestamp = g.next_role_timestamp();
    g.state
        .roles
        .get_or_insert_with(Default::default)
        .creature_types
        .push(subtypes::CreatureTypesEffectView {
            recipient,
            source: recipient,
            types: names,
            additional: false,
            timestamp,
        });
}
fn army(g: &mut Game, p: Player) -> CardId {
    let id = fixture(g, p, "Bear Cub", Zone::Battlefield);
    types(g, id, &["Army"]);
    id
}
fn armies(g: &Game, p: Player) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(id, o)| {
            (o.zone == Zone::Battlefield
                && o.controller == p
                && g.is_creature(id as CardId)
                && g.has_subtype(id as CardId, "Army"))
            .then_some(id as CardId)
        })
        .collect()
}
#[test]
fn hob_azog_rampager_azog_paid_optional_zero_and_other_creature_target_legality() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let bear = fixture(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let land = fixture(&mut g, p, "Forest", Zone::Battlefield);
        let id = azog(&mut g, p);
        assert_eq!(stats(&g, id), (1, 3));
        assert!(g.object_view(id).is_legendary());
        assert!(g.has_subtype(id, "Goblin") && g.has_subtype(id, "Soldier"));
        for target in [
            Target::Card(id),
            Target::Card(land),
            Target::Player(p.other()),
        ] {
            reject(&mut g, Action::ChooseTarget { target });
        }
        act(&mut g, Action::FinishTargets);
        finish(&mut g);
        assert_eq!(g.object_view(bear).zone, Zone::Battlefield);
        assert!(armies(&g, p).is_empty() && armies(&g, p.other()).is_empty());
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
    }
}
#[test]
fn hob_azog_rampager_azog_resolution_controller_owner_failed_destroy_and_negative_zero() {
    for p in [Player::P0, Player::P1] {
        for mode in 0..9 {
            let mut g = scenario(p);
            let target = fixture(
                &mut g,
                if mode == 1 || mode == 2 || mode == 4 {
                    p.other()
                } else {
                    p
                },
                "Bear Cub",
                Zone::Battlefield,
            );
            if mode == 2 {
                g.record_control(g.object_ref(target), p, None);
            }
            if mode == 5 {
                g.state.objects[target as usize].gained_keywords = Keywords::INDESTRUCTIBLE;
            }
            if mode == 6 {
                fixture(&mut g, p, "Rest in Peace", Zone::Battlefield);
            }
            if mode == 7 {
                g.state.objects[target as usize].growth.power = -5;
            }
            if mode == 8 {
                g.state.objects[target as usize].growth.power = -2;
            }
            let existing = (mode == 8).then(|| army(&mut g, p));
            let source = azog(&mut g, p);
            bind(&mut g, target);
            if mode == 3 {
                g.record_control(g.object_ref(target), p.other(), None);
            }
            if mode == 4 {
                g.record_control(g.object_ref(target), p, None);
            }
            let receiver = if mode == 1 || mode == 3 { p.other() } else { p };
            finish(&mut g);
            assert_eq!(
                g.object_view(target).zone,
                if mode == 5 {
                    Zone::Battlefield
                } else if mode == 6 {
                    Zone::Exile
                } else {
                    Zone::Graveyard
                },
                "mode{mode}"
            );
            assert_eq!(
                g.state.players[p.index()].cards_drawn_this_turn,
                u64::from(receiver == p),
                "mode{mode}"
            );
            assert_eq!(stats(&g, source), (1, 3));
            assert!(armies(&g, receiver.other()).is_empty());
            if mode == 7 {
                assert!(armies(&g, receiver).is_empty());
                assert_eq!(
                    g.state
                        .objects
                        .iter()
                        .filter(|o| o.card == Card::GoblinArmyToken
                            && o.owner == p
                            && o.zone == Zone::Outside)
                        .count(),
                    1,
                    "negative Amass0 still creates then loses 0/0 Army"
                );
            } else {
                let a = armies(&g, receiver);
                assert_eq!(a.len(), 1);
                let id = a[0];
                assert_eq!(
                    g.counter_count(id, CounterKind::PlusOne),
                    if mode == 8 { 0 } else { 2 }
                );
                assert!(g.has_subtype(id, "Goblin"));
                if let Some(existing) = existing {
                    assert_eq!(id, existing);
                    assert_eq!(stats(&g, id), (2, 2));
                } else {
                    assert!(g.object_view(id).is_token());
                    assert_eq!(g.object_view(id).colors, 4);
                }
            }
        }
    }
}
#[test]
fn hob_azog_rampager_azog_live_power_before_destroy_paid_response_and_departed_source() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let target = fixture(&mut g, p, "Bear Cub", Zone::Battlefield);
        let source = azog(&mut g, p);
        bind(&mut g, target);
        let trigger = g.state.stack.last().unwrap().id;
        paid(
            &mut g,
            p,
            "Giant Growth",
            [0, 0, 0, 0, 1, 0],
            Some(Target::Card(target)),
        );
        until(&mut g, |g| {
            g.state.stack.last().is_some_and(|s| s.id == trigger)
        });
        assert_eq!(stats(&g, target), (5, 5));
        g.move_object(source, Zone::Hand);
        finish(&mut g);
        assert_eq!(g.object_view(target).zone, Zone::Graveyard);
        assert_eq!(g.counter_count(armies(&g, p)[0], CounterKind::PlusOne), 5);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
    }
}
#[test]
fn hob_azog_rampager_azog_stale_or_noncreature_target_fizzles_all_later_actions() {
    for p in [Player::P0, Player::P1] {
        for mode in 0..3 {
            let mut g = scenario(p);
            let target = fixture(&mut g, p, "Bear Cub", Zone::Battlefield);
            azog(&mut g, p);
            bind(&mut g, target);
            if mode < 2 {
                g.move_object(target, Zone::Hand);
                if mode == 1 {
                    g.move_object(target, Zone::Battlefield);
                }
            } else {
                let mut recipe = CopyRecipe::new(Card::from_name("Bear Cub").unwrap());
                recipe.overrides = Some(CopyOverrides {
                    kinds: Some(16),
                    ..CopyOverrides::NONE
                });
                g.apply_temporary_copy(g.object_ref(target), recipe, CopyDeadline::EndOfTurn);
            }
            finish(&mut g);
            assert!(armies(&g, p).is_empty() && armies(&g, p.other()).is_empty());
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
        }
    }
}
#[test]
fn hob_azog_rampager_target_controller_owns_public_army_choice_raw_index_restore() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let own = army(&mut g, p);
        let a = army(&mut g, p.other());
        let b = army(&mut g, p.other());
        let target = fixture(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        azog(&mut g, p);
        bind(&mut g, target);
        until(&mut g, |g| {
            g.legal_actions().contains(&Action::ChooseArmy {
                object: g.object_ref(a),
            })
        });
        assert_eq!(g.actor(), Some(p.other()));
        assert!(g.observe(p).effect_choice.is_some());
        let object = g.object_ref(own);
        reject(&mut g, Action::ChooseArmy { object });
        contracts::branch_contract(g.clone());
        let saved = g.snapshot();
        let mut raw = g.clone();
        let mut indexed = g.clone();
        drop(g);
        let action = Action::ChooseArmy {
            object: raw.object_ref(b),
        };
        let actor = raw.actor().unwrap();
        let idx = indexed
            .legal_actions()
            .iter()
            .position(|a| a == &action)
            .unwrap();
        act(&mut raw, action.clone());
        indexed.apply_index(actor, indexed.revision(), idx).unwrap();
        finish(&mut raw);
        finish(&mut indexed);
        assert_eq!(raw.snapshot(), indexed.snapshot());
        indexed.restore(&saved).unwrap();
        act(&mut indexed, action);
        finish(&mut indexed);
        assert_eq!(raw.snapshot(), indexed.snapshot());
        assert_eq!(raw.counter_count(b, CounterKind::PlusOne), 2);
        assert_eq!(raw.counter_count(a, CounterKind::PlusOne), 0);
        assert_eq!(raw.counter_count(own, CounterKind::PlusOne), 0);
        assert!(raw.has_subtype(b, "Goblin"));
        assert!(!raw.has_subtype(a, "Goblin"));
    }
}
#[test]
fn hob_azog_rampager_target_controller_token_and_counter_replacements_resume_before_draw() {
    for p in [Player::P0, Player::P1] {
        for existing in [false, true] {
            let mut g = scenario(p);
            let receiver = if existing { p } else { p.other() };
            let pre = existing.then(|| army(&mut g, receiver));
            fixture(&mut g, receiver, "Doubling Season", Zone::Battlefield);
            fixture(&mut g, receiver, "Doubling Season", Zone::Battlefield);
            let target = fixture(&mut g, receiver, "Bear Cub", Zone::Battlefield);
            azog(&mut g, p);
            bind(&mut g, target);
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::Replacement { .. })
            });
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
            contracts::branch_contract(g.clone());
            finish(&mut g);
            let a = armies(&g, receiver);
            assert_eq!(a.len(), 1);
            assert_eq!(g.counter_count(a[0], CounterKind::PlusOne), 8);
            if let Some(pre) = pre {
                assert_eq!(a[0], pre);
            } else {
                assert_eq!(
                    g.state
                        .objects
                        .iter()
                        .filter(|o| o.card == Card::GoblinArmyToken && o.owner == receiver)
                        .count(),
                    4
                );
            }
            assert_eq!(
                g.state.players[p.index()].cards_drawn_this_turn,
                u64::from(existing)
            );
        }
    }
}
#[test]
fn hob_azog_rampager_scaled_65536_and_counter_overflow_are_bounded_and_transactional() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let a = army(&mut g, p.other());
        let b = army(&mut g, p.other());
        let target = fixture(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.state.objects[target as usize].growth.power = 65534;
        azog(&mut g, p);
        bind(&mut g, target);
        until(&mut g, |g| {
            g.legal_actions().contains(&Action::ChooseArmy {
                object: g.object_ref(a),
            })
        });
        assert_eq!(
            serde_json::to_value(g.observe(p.other()).effect_choice.unwrap().kind).unwrap(),
            serde_json::json!({"AmassGoblins":65536})
        );
        let object = g.object_ref(b);
        act(&mut g, Action::ChooseArmy { object });
        finish(&mut g);
        assert_eq!(g.counter_count(b, CounterKind::PlusOne), 65536);
        let mut g = scenario(p);
        let a = army(&mut g, p.other());
        g.set_counters(a, Counters::of(CounterKind::PlusOne, u64::MAX));
        let target = fixture(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        azog(&mut g, p);
        bind(&mut g, target);
        act(&mut g, Action::Pass);
        let saved = g.snapshot();
        let rev = g.revision();
        let actor = g.actor().unwrap();
        let views = [
            serde_json::to_value(g.observe(p)).unwrap(),
            serde_json::to_value(g.observe(p.other())).unwrap(),
        ];
        assert!(matches!(
            g.apply(actor, Action::Pass),
            Err(Error::CapacityExceeded(_))
        ));
        assert_eq!(g.snapshot(), saved);
        assert_eq!(g.revision(), rev);
        assert_eq!(serde_json::to_value(g.observe(p)).unwrap(), views[0]);
        assert_eq!(
            serde_json::to_value(g.observe(p.other())).unwrap(),
            views[1]
        );
        assert_eq!(g.counter_count(a, CounterKind::PlusOne), u64::MAX);
        assert_eq!(g.object_view(target).zone, Zone::Battlefield);
    }
}
#[test]
fn hob_azog_rampager_rampager_actual_attack_optional_sacrifice_filters_power_and_incarnation() {
    for p in [Player::P0, Player::P1] {
        for mode in 0..8 {
            let mut g = scenario(p);
            let source = rampager(&mut g, p);
            let offering = fixture(
                &mut g,
                if mode == 5 { p.other() } else { p },
                "Bear Cub",
                Zone::Battlefield,
            );
            if mode == 5 {
                g.record_control(g.object_ref(offering), p, None);
            }
            let enemy = fixture(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let artifact = fixture(&mut g, p, "Goldvein Pick", Zone::Battlefield);
            if mode == 2 {
                g.state.objects[offering as usize].growth = Stats {
                    power: 3,
                    toughness: 1,
                };
            }
            if mode == 3 {
                g.state.objects[offering as usize].growth.power = -4;
            }
            if mode == 4 {
                fixture(&mut g, p, "Rest in Peace", Zone::Battlefield);
            }
            attack(&mut g, source);
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::EffectChoice { .. })
            });
            for id in [source, enemy, artifact] {
                let object = g.object_ref(id);
                reject(&mut g, Action::Sacrifice { object });
            }
            if mode == 0 {
                act(&mut g, Action::DeclineEffectPayment);
            } else {
                if mode == 6 {
                    g.move_object(source, Zone::Hand);
                    g.move_object(source, Zone::Battlefield);
                }
                if mode == 7 {
                    g.record_control(g.object_ref(source), p.other(), None);
                }
                let object = g.object_ref(offering);
                act(&mut g, Action::Sacrifice { object });
            }
            finish(&mut g);
            assert_eq!(
                g.counter_count(source, CounterKind::PlusOne),
                match mode {
                    0 | 3 | 6 => 0,
                    2 => 5,
                    _ => 2,
                },
                "mode{mode}"
            );
            assert_eq!(
                g.object_view(offering).zone,
                if mode == 0 {
                    Zone::Battlefield
                } else if mode == 4 {
                    Zone::Exile
                } else {
                    Zone::Graveyard
                }
            );
        }
        let mut g = scenario(p);
        let source = rampager(&mut g, p);
        attack(&mut g, source);
        finish(&mut g);
        assert_eq!(g.counter_count(source, CounterKind::PlusOne), 0);
        let mut g = scenario(p);
        let source = rampager(&mut g, p);
        let bear = fixture(&mut g, p, "Bear Cub", Zone::Battlefield);
        attack(&mut g, bear);
        finish(&mut g);
        assert_eq!(g.counter_count(source, CounterKind::PlusOne), 0);
    }
}
fn animated_prison(g: &mut Game, p: Player) -> CardId {
    let automaton = fixture(g, p.other(), "Adaptive Automaton", Zone::Battlefield);
    let prison = paid(g, p, "Banishing Light", [1, 0, 0, 0, 0, 2], None);
    until(g, |g| {
        g.legal_actions().contains(&Action::ChooseTarget {
            target: Target::Card(automaton),
        })
    });
    bind(g, automaton);
    finish(g);
    assert_eq!(g.object_view(automaton).zone, Zone::Exile);
    g.change_object_form(
        g.object_ref(prison),
        AnimationSpec {
            artifact: false,
            creature_types: AnimationCreatureTypes::Unchanged,
            base: Some((5, 5)),
            keywords: Keywords::NONE,
            duration: AnimationDuration::EndOfTurn,
        },
        characteristic_layers::AnimationTypeMode::AddCreature,
    );
    prison
}
#[test]
fn hob_azog_rampager_sacrificed_power_owned_across_linked_return_suspended_entry() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let prison = animated_prison(&mut g, p);
        let source = rampager(&mut g, p);
        attack(&mut g, source);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectChoice { .. })
        });
        let object = g.object_ref(prison);
        act(&mut g, Action::Sacrifice { object });
        assert!(g.observe(g.actor().unwrap()).entry_value.is_some());
        assert_eq!(g.counter_count(source, CounterKind::PlusOne), 0);
        assert_eq!(
            g.state
                .resolving
                .as_ref()
                .unwrap()
                .sacrificed_power
                .as_deref(),
            Some(&5)
        );
        contracts::branch_contract(g.clone());
        choose(&mut g, |a| {
            matches!(
                a,
                Action::ChooseEntryValue {
                    value: EntryChoiceValue::CreatureType(_)
                }
            )
        });
        finish(&mut g);
        assert_eq!(g.counter_count(source, CounterKind::PlusOne), 5);
        assert_eq!(stats(&g, source), (8, 7));
    }
}
#[test]
fn hob_azog_rampager_target_facts_survive_destroy_linked_return_suspended_entry() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let prison = animated_prison(&mut g, p);
        azog(&mut g, p);
        bind(&mut g, prison);
        until(&mut g, |g| {
            g.observe(g.actor().unwrap()).entry_value.is_some()
        });
        assert!(armies(&g, p).is_empty());
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
        contracts::branch_contract(g.clone());
        choose(&mut g, |a| {
            matches!(
                a,
                Action::ChooseEntryValue {
                    value: EntryChoiceValue::CreatureType(_)
                }
            )
        });
        finish(&mut g);
        assert_eq!(g.counter_count(armies(&g, p)[0], CounterKind::PlusOne), 5);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
    }
}
#[test]
fn hob_azog_rampager_rampager_death_last_known_power_counters_modifiers_reentry_and_replacement() {
    for p in [Player::P0, Player::P1] {
        for mode in 0..5 {
            let mut g = scenario(p);
            let source = rampager(&mut g, p);
            if mode == 0 || mode == 3 {
                g.add_counter(source, CounterKind::PlusOne, 2);
                g.state.objects[source as usize].growth = Stats {
                    power: 3,
                    toughness: 1,
                };
            }
            if mode == 1 {
                g.state.objects[source as usize].growth.power = -5;
            }
            if mode == 2 {
                fixture(&mut g, p, "Rest in Peace", Zone::Battlefield);
            }
            if mode == 3 {
                g.state.objects[source as usize].owner = p.other();
            }
            if mode == 4 {
                types(&mut g, source, &["Elf"]);
                let lord = fixture(&mut g, p, "Elvish Archdruid", Zone::Battlefield);
                let observers = g.death_trigger_sources();
                let events = g.move_objects_raw(&[lord, source], Zone::Graveyard);
                g.finish_zone_events(&events, &observers);
            } else {
                paid(
                    &mut g,
                    p,
                    "Murder",
                    [0, 0, 2, 0, 0, 1],
                    Some(Target::Card(source)),
                );
                until(&mut g, |g| g.object_view(source).zone != Zone::Battlefield);
                if mode == 0 {
                    g.move_object(source, Zone::Battlefield);
                    g.state.objects[source as usize].growth.power = 50;
                }
            }
            finish(&mut g);
            if mode == 1 {
                assert!(armies(&g, p).is_empty());
                assert_eq!(
                    g.state
                        .objects
                        .iter()
                        .filter(|o| o.card == Card::GoblinArmyToken && o.owner == p)
                        .count(),
                    1
                );
            } else if mode == 2 {
                assert_eq!(g.object_view(source).zone, Zone::Exile);
                assert!(armies(&g, p).is_empty());
                assert!(
                    !g.state
                        .objects
                        .iter()
                        .any(|o| o.card == Card::GoblinArmyToken)
                );
            } else {
                let a = armies(&g, p);
                assert_eq!(a.len(), 1);
                assert_eq!(
                    g.counter_count(a[0], CounterKind::PlusOne),
                    if mode == 4 { 4 } else { 8 }
                );
                assert!(armies(&g, p.other()).is_empty());
            }
        }
    }
}
#[test]
fn hob_azog_rampager_rampager_sacrifice_counter_replacements_then_death_existing_army() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = rampager(&mut g, p);
        let offering = fixture(&mut g, p, "Bear Cub", Zone::Battlefield);
        let a = army(&mut g, p);
        fixture(&mut g, p, "Doubling Season", Zone::Battlefield);
        fixture(&mut g, p, "Doubling Season", Zone::Battlefield);
        attack(&mut g, source);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectChoice { .. })
        });
        let object = g.object_ref(offering);
        act(&mut g, Action::Sacrifice { object });
        assert!(matches!(g.state.pending, Pending::Replacement { .. }));
        finish(&mut g);
        assert_eq!(g.counter_count(source, CounterKind::PlusOne), 8);
        g.move_object(source, Zone::Graveyard);
        finish(&mut g);
        assert_eq!(g.counter_count(a, CounterKind::PlusOne), 44);
        assert!(g.has_subtype(a, "Goblin"));
    }
}

fn insects(g: &Game, player: Player) -> usize {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter(|(id, o)| {
            o.zone == Zone::Battlefield
                && o.controller == player
                && o.is_token()
                && g.has_subtype(*id as CardId, "Insect")
        })
        .count()
}
#[test]
fn hob_azog_rampager_actual_amassing_player_places_counters_doc_replacement_history_triggers_and_owned_choice()
 {
    for p in [Player::P0, Player::P1] {
        for army_count in 0..3 {
            let mut g = scenario(p);
            for player in [p, p.other()] {
                fixture(
                    &mut g,
                    player,
                    "Doc Samson, Super Psychiatrist",
                    Zone::Battlefield,
                );
                fixture(
                    &mut g,
                    player,
                    "Ant-Man, Colony Commander",
                    Zone::Battlefield,
                );
            }
            let existing: Vec<_> = (0..army_count).map(|_| army(&mut g, p.other())).collect();
            let target = fixture(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let source = azog(&mut g, p);
            bind(&mut g, target);
            if army_count == 2 {
                until(&mut g, |g| {
                    g.legal_actions().contains(&Action::ChooseArmy {
                        object: g.object_ref(existing[1]),
                    })
                });
                assert_eq!(g.actor(), Some(p.other()));
                assert_eq!(g.state.resolving.as_ref().unwrap().item.controller, p);
                assert_eq!(g.state.resolving.as_ref().unwrap().item.source.unwrap().id, source);
                contracts::branch_contract(g.clone());
                let saved = g.snapshot();
                let mut indexed = g.clone();
                let action = Action::ChooseArmy {
                    object: g.object_ref(existing[1]),
                };
                let index = indexed
                    .legal_actions()
                    .iter()
                    .position(|a| a == &action)
                    .unwrap();
                indexed
                    .apply_index(p.other(), indexed.revision(), index)
                    .unwrap();
                act(&mut g, action.clone());
                finish(&mut g);
                finish(&mut indexed);
                assert_eq!(g.snapshot(), indexed.snapshot());
                indexed.restore(&saved).unwrap();
                act(&mut indexed, action);
                finish(&mut indexed);
                assert_eq!(g.snapshot(), indexed.snapshot());
            } else {
                finish(&mut g);
            }
            let recipient = if army_count == 0 {
                armies(&g, p.other())[0]
            } else {
                existing[army_count - 1]
            };
            assert_eq!(
                g.counter_count(recipient, CounterKind::PlusOne),
                3,
                "target controller's actual Doc Samson adds one counter"
            );
            assert_eq!(
                insects(&g, p.other()),
                1,
                "amassing player's Ant-Man triggers"
            );
            assert_eq!(
                insects(&g, p),
                0,
                "Azog's controller did not put the counters"
            );
            let object = g.object_ref(recipient);
            for viewer in [p, p.other()] {
                let rows = g.observe(viewer).counter_placement_history;
                assert!(
                    rows.iter()
                        .any(|r| r.object == object && r.placed_by == p.other() && r.plus_one)
                );
                assert!(!rows.iter().any(|r| r.object == object && r.placed_by == p));
            }
            assert!(g.put_counter_on_source_this_turn(object, p.other()));
            assert!(!g.put_counter_on_source_this_turn(object, p));
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
            if army_count == 2 {
                assert_eq!(g.counter_count(existing[0], CounterKind::PlusOne), 0);
            }
        }
    }
}
#[test]
fn hob_azog_rampager_legacy_fixed_amass_keeps_original_placer_and_controller() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let recipient = army(&mut g, p);
        for player in [p, p.other()] {
            fixture(
                &mut g,
                player,
                "Doc Samson, Super Psychiatrist",
                Zone::Battlefield,
            );
            fixture(
                &mut g,
                player,
                "Ant-Man, Colony Commander",
                Zone::Battlefield,
            );
        }
        paid(&mut g, p, "Goblin Plate Mail", [0, 0, 1, 0, 0, 1], None);
        finish(&mut g);
        assert_eq!(g.counter_count(recipient, CounterKind::PlusOne), 2);
        assert_eq!(insects(&g, p), 1);
        assert_eq!(insects(&g, p.other()), 0);
        let object = g.object_ref(recipient);
        assert!(g.put_counter_on_source_this_turn(object, p));
        assert!(!g.put_counter_on_source_this_turn(object, p.other()));
    }
}
