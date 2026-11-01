//! Full printed outcomes plus frozen event and resolved layer-7b contracts.
use super::*;
pub(super) fn quiet(g: &Game) -> bool {
    g.state.pending == Pending::Priority
        && g.state.stack.is_empty()
        && g.state.pending_triggers.is_empty()
        && g.state.resolving.is_none()
}
pub(super) fn drive(g: &mut Game, preferred: Option<CardId>) {
    for _ in 0..96 {
        if quiet(g) {
            return;
        }
        let a = g.legal_actions();
        let action = preferred
            .and_then(|id| {
                a.iter().find(|a| match a {
                    Action::ChooseTarget { target } => *target == Target::Card(id),
                    Action::ChooseArmy { object } => object.card == id,
                    _ => false,
                })
            })
            .or_else(|| {
                a.iter()
                    .find(|a| matches!(a, Action::ChooseReplacement { .. }))
            })
            .or_else(|| {
                a.iter().find(|a| {
                    matches!(
                        a,
                        Action::Pass
                            | Action::PutTrigger { .. }
                            | Action::FinishTargets
                            | Action::ChooseArmy { .. }
                    )
                })
            })
            .unwrap_or_else(|| panic!("explicit choice {:?}", g.state.pending))
            .clone();
        act(g, action);
    }
    panic!("crooked/galion resolution budget");
}
fn until(g: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..48 {
        if ready(g) {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("crooked/galion decision budget {:?}", g.state.pending);
}
pub(super) fn armies(g: &Game, p: Player) -> Vec<CardId> {
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
pub(super) fn types(g: &mut Game, id: CardId, names: &'static [&'static str]) {
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
fn paid(g: &mut Game, p: Player, name: &str, target: Option<Target>) -> CardId {
    let id = put(g, p, name, Zone::Hand);
    g.state.players[p.index()].mana = [12; 6];
    cast(g, id, target);
    drive(g, None);
    id
}
fn galion(g: &mut Game, p: Player) -> CardId {
    let id = put(g, p, "Galion, Elvenking's Butler", Zone::Hand);
    g.state.players[p.index()].mana = [0, 0, 0, 0, 2, 2];
    cast(g, id, None);
    drive(g, None);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    assert_eq!(stats(g, id), (4, 4));
    assert!(g.has_subtype(id, "Elf") && g.has_subtype(id, "Advisor"));
    assert!(g.object_view(id).is_legendary());
    id
}
fn attack(g: &mut Game, source: CardId) {
    g.state.objects[source as usize].summoning_sick = false;
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    act(g, Action::Attack { card: source });
    act(g, Action::FinishAttackers);
    until(g, |g| g.legal_actions().contains(&Action::FinishTargets));
}
fn cleanup(g: &mut Game) {
    let turn = g.state.turn;
    for _ in 0..96 {
        if g.state.turn != turn {
            return;
        }
        choose(g, |a| {
            matches!(
                a,
                Action::Pass | Action::FinishAttackers | Action::FinishBlockers
            )
        });
    }
    panic!("cleanup budget");
}
#[test]
fn hob_crooked_galion_along_paid_etb_return_causes_amass_and_public_hand_knowledge() {
    for p in [Player::P0, Player::P1] {
        for stale in [false, true] {
            let mut g = scenario(p);
            let own = put(&mut g, p, "Bear Cub", Zone::Graveyard);
            let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Graveyard);
            let land = put(&mut g, p, "Forest", Zone::Graveyard);
            let source = put(&mut g, p, "Along the Crooked Way", Zone::Hand);
            g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 2];
            cast(&mut g, source, None);
            until(&mut g, |g| {
                g.legal_actions().contains(&Action::ChooseTarget {
                    target: Target::Card(own),
                })
            });
            for illegal in [enemy, land] {
                reject(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(illegal),
                    },
                );
            }
            bind(&mut g, own);
            if stale {
                g.move_object(own, Zone::Exile);
                drive(&mut g, None);
            }
            drive(&mut g, None);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert_eq!(g.object_view(source).zone, Zone::Battlefield);
            assert_eq!(
                g.object_view(own).zone,
                if stale { Zone::Exile } else { Zone::Hand }
            );
            assert_eq!(armies(&g, p).len(), 1);
            assert_eq!(
                g.counter_count(armies(&g, p)[0], CounterKind::PlusOne),
                1,
                "stale ETB target still left graveyard via actual response"
            );
            if !stale {
                assert!(g.observe(p.other()).known_hand.contains(&g.object_ref(own)));
            }
        }
        let mut g = scenario(p);
        let source = put(&mut g, p, "Along the Crooked Way", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 2];
        cast(&mut g, source, None);
        drive(&mut g, None);
        assert!(armies(&g, p).is_empty());
        assert_eq!(g.object_view(source).zone, Zone::Battlefield);
    }
}
#[test]
fn hob_crooked_galion_along_actual_reanimation_and_paid_graveyard_cast_each_trigger() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p, "Along the Crooked Way", Zone::Battlefield);
        let body = put(&mut g, p, "Bear Cub", Zone::Graveyard);
        paid(&mut g, p, "Zombify", Some(Target::Card(body)));
        assert_eq!(g.object_view(body).zone, Zone::Battlefield);
        let army = armies(&g, p)[0];
        assert_eq!(g.counter_count(army, CounterKind::PlusOne), 1);
        let casted = put(&mut g, p, "Bear Cub", Zone::Graveyard);
        g.grant_graveyard_permission(p, g.object_ref(casted), GraveyardCastKind::Cast);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 1];
        cast(&mut g, casted, None);
        drive(&mut g, None);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(g.object_view(casted).zone, Zone::Battlefield);
        assert_eq!(g.counter_count(army, CounterKind::PlusOne), 2);
    }
}
#[test]
fn hob_crooked_galion_along_per_card_batches_before_owner_and_card_exclusions() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Along the Crooked Way", Zone::Battlefield);
        let own = put(&mut g, p, "Bear Cub", Zone::Graveyard);
        g.state.objects[own as usize].controller = p.other();
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Graveyard);
        g.state.objects[enemy as usize].controller = p;
        let land = put(&mut g, p, "Forest", Zone::Graveyard);
        let token = put(&mut g, p, "@token/goblin_army", Zone::Graveyard);
        let events = g.move_objects_raw(&[own, enemy, land, token], Zone::Exile);
        g.finish_zone_events(&events, &[]);
        assert_eq!(
            g.state
                .pending_triggers
                .iter()
                .filter(|s| s.source.is_some_and(|identity| identity.id == source))
                .count(),
            1
        );
        drive(&mut g, None);
        let army = armies(&g, p)[0];
        assert_eq!(g.counter_count(army, CounterKind::PlusOne), 1);
        for destination in [Zone::Hand, Zone::Library, Zone::Exile] {
            let a = put(&mut g, p, "Bear Cub", Zone::Graveyard);
            let b = put(&mut g, p, "Llanowar Elves", Zone::Graveyard);
            let events = g.move_objects_raw(&[a, b], destination);
            g.finish_zone_events(&events, &[]);
            assert_eq!(
                g.state
                    .pending_triggers
                    .iter()
                    .filter(|s| s.source.is_some_and(|identity| identity.id == source))
                    .count(),
                2
            );
            drive(&mut g, None);
        }
        assert_eq!(g.counter_count(army, CounterKind::PlusOne), 7);
    }
}
#[test]
fn hob_crooked_galion_along_pre_event_observer_and_multiplier_survive_simultaneous_departure() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        g.state.players[p.index()].enduring_story = true;
        let provider = put(&mut g, p, "Bifur, Melodic Rider", Zone::Battlefield);
        let source = put(
            &mut g,
            p.other(),
            "Along the Crooked Way",
            Zone::Battlefield,
        );
        g.record_control(g.object_ref(source), p, None);
        types(&mut g, source, &["Dwarf"]);
        assert_eq!(g.object_view(source).additional_triggers, 1);
        let a = put(&mut g, p, "Bear Cub", Zone::Graveyard);
        let b = put(&mut g, p, "Llanowar Elves", Zone::Graveyard);
        let events = g.move_objects_raw(&[provider, source, a, b], Zone::Exile);
        g.finish_zone_events(&events, &[]);
        let packets: Vec<_> = g
            .state
            .pending_triggers
            .iter()
            .filter(|s| s.source.is_some_and(|identity| identity.id == source))
            .collect();
        assert_eq!(packets.len(), 2);
        assert!(
            packets
                .iter()
                .all(|s| s.additional_triggers == 1 && s.controller == p)
        );
        super::contracts::branch_contract(g.clone());
        drive(&mut g, None);
        assert_eq!(g.counter_count(armies(&g, p)[0], CounterKind::PlusOne), 4);
        let mut g = scenario(p);
        let source = put(&mut g, p, "Along the Crooked Way", Zone::Graveyard);
        let body = put(&mut g, p, "Bear Cub", Zone::Graveyard);
        let events = g.move_objects_raw(&[source, body], Zone::Battlefield);
        g.finish_zone_events(&events, &[]);
        drive(&mut g, None);
        assert!(
            armies(&g, p).is_empty(),
            "new observer cannot look back at its simultaneous entry"
        );
    }
}
#[test]
fn hob_crooked_galion_along_menace_snapshots_union_current_permanents_and_expires() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Along the Crooked Way", Zone::Battlefield);
        let goblin = put(&mut g, p, "Goblin Piker", Zone::Battlefield);
        let orc = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        types(&mut g, orc, &["Orc"]);
        let dual = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        types(&mut g, dual, &["Goblin", "Orc"]);
        let rock = put(&mut g, p, "Heraldic Banner", Zone::Battlefield);
        types(&mut g, rock, &["Goblin"]);
        let enemy = put(&mut g, p.other(), "Goblin Piker", Zone::Battlefield);
        let ordinary = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 1];
        act(
            &mut g,
            Action::AnnounceActivate {
                card: source,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut g);
        let before_resolution = put(&mut g, p, "Goblin Piker", Zone::Battlefield);
        g.move_object(source, Zone::Exile);
        drive(&mut g, None);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        for id in [goblin, orc, dual, rock, before_resolution] {
            assert!(g.keywords(id).contains(Keywords::MENACE));
        }
        assert!(!g.is_creature(rock));
        for id in [enemy, ordinary] {
            assert!(!g.keywords(id).contains(Keywords::MENACE));
        }
        let late = put(&mut g, p, "Goblin Piker", Zone::Battlefield);
        assert!(!g.keywords(late).contains(Keywords::MENACE));
        assert_eq!(
            g.state
                .roles
                .as_ref()
                .unwrap()
                .keyword_effects
                .iter()
                .filter(
                    |e| e.recipient == g.object_ref(dual) && e.keywords.contains(Keywords::MENACE)
                )
                .count(),
            1
        );
        types(&mut g, orc, &["Citizen"]);
        g.record_control(g.object_ref(orc), p.other(), None);
        assert!(g.keywords(orc).contains(Keywords::MENACE));
        cleanup(&mut g);
        for id in [goblin, orc, dual, rock, before_resolution] {
            assert!(!g.keywords(id).contains(Keywords::MENACE));
        }
    }
}
#[test]
fn hob_crooked_galion_galion_paid_attack_optional_other_targets_and_decline() {
    for p in [Player::P0, Player::P1] {
        for decline in [false, true] {
            let mut g = scenario(p);
            let source = galion(&mut g, p);
            let body = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            attack(&mut g, source);
            for id in [source, enemy] {
                reject(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(id),
                    },
                );
            }
            if decline {
                act(&mut g, Action::FinishTargets);
            } else {
                bind(&mut g, body);
            }
            drive(&mut g, None);
            assert_eq!(stats(&g, body), if decline { (2, 2) } else { (4, 4) });
            if !decline {
                cleanup(&mut g);
                assert_eq!(stats(&g, body), (2, 2));
            }
        }
        let mut g = scenario(p);
        let source = galion(&mut g, p);
        let other = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        act(&mut g, Action::Attack { card: other });
        act(&mut g, Action::FinishAttackers);
        assert!(
            g.state
                .pending_triggers
                .iter()
                .all(|s| s.source.is_none_or(|identity| identity.id != source))
        );
        assert!(
            g.state
                .stack
                .iter()
                .all(|s| s.source.is_none_or(|identity| identity.id != source))
        );
    }
}
#[test]
fn hob_crooked_galion_galion_samples_resolution_axes_then_fixed_base_layer_and_modifiers() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = galion(&mut g, p);
        g.add_counter(source, CounterKind::PlusOne, 1);
        let body = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.add_counter(body, CounterKind::PlusOne, 2);
        paid(&mut g, p, "Witness Protection", Some(Target::Card(body)));
        paid(&mut g, p, "Giant Growth", Some(Target::Card(body)));
        assert_eq!(stats(&g, body), (6, 6));
        attack(&mut g, source);
        bind(&mut g, body);
        let trigger = g.state.stack.last().unwrap().id;
        let growth = put(&mut g, p, "Giant Growth", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 0];
        cast(&mut g, growth, Some(Target::Card(source)));
        until(&mut g, |g| {
            g.state.stack.last().is_some_and(|s| s.id == trigger)
        });
        assert_eq!(stats(&g, source), (8, 8));
        drive(&mut g, None);
        assert_eq!(stats(&g, body), (13, 13));
        paid(&mut g, p, "Giant Growth", Some(Target::Card(source)));
        assert_eq!(stats(&g, source), (11, 11));
        assert_eq!(stats(&g, body), (13, 13));
        // The later Aura is a sorcery-speed base setter. Reach main phase two
        // through normal combat choices while both temporary growth effects last.
        for _ in 0..32 {
            if g.state.step == Step::Main2 && quiet(&g) {
                break;
            }
            choose(&mut g, |a| {
                matches!(a, Action::Pass | Action::FinishBlockers)
            });
        }
        assert_eq!(g.state.step, Step::Main2);
        paid(&mut g, p, "Witness Protection", Some(Target::Card(body)));
        assert_eq!(stats(&g, body), (6, 6));
    }
}
#[test]
fn hob_crooked_galion_galion_signed_departure_lki_incarnation_simultaneous_and_stale_targets() {
    for p in [Player::P0, Player::P1] {
        for mode in 0..9 {
            let mut g = scenario(p);
            let source = galion(&mut g, p);
            let body = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let lord = if mode == 2 || mode == 5 {
                Some(put(&mut g, p, "Elvish Archdruid", Zone::Battlefield))
            } else {
                None
            };
            if mode == 1 {
                g.state.objects[source as usize].growth.power = -6;
            }
            if mode >= 6 {
                g.add_counter(body, CounterKind::PlusOne, 5);
            }
            attack(&mut g, source);
            bind(&mut g, body);
            if mode == 0 {
                g.state.objects[source as usize].growth = Stats {
                    power: 3,
                    toughness: 1,
                };
                g.move_object(source, Zone::Exile);
                g.move_object(source, Zone::Battlefield);
            }
            if mode == 2 {
                let events = g.move_objects_raw(&[lord.unwrap(), source], Zone::Exile);
                g.finish_zone_events(&events, &[]);
            }
            if mode == 3 {
                g.move_object(body, Zone::Hand);
                g.move_object(body, Zone::Battlefield);
            }
            if mode == 4 {
                g.record_control(g.object_ref(body), p.other(), None);
            }
            if mode == 5 {
                g.record_control(g.object_ref(source), p.other(), None);
            }
            if mode == 6 {
                g.state.objects[source as usize].growth = Stats {
                    power: -6,
                    toughness: -6,
                };
                g.move_object(source, Zone::Exile);
            }
            if mode >= 7 {
                // A layer-one override retains the printed recipe but removes
                // creature type. Neither live nor last-known values have P/T.
                let mut recipe =
                    CopyRecipe::new(Card::from_name("Galion, Elvenking's Butler").unwrap());
                recipe.overrides = Some(CopyOverrides {
                    kinds: Some(16),
                    ..CopyOverrides::NONE
                });
                g.apply_temporary_copy(g.object_ref(source), recipe, CopyDeadline::EndOfTurn);
                assert!(!g.is_creature(source));
                if mode == 8 {
                    g.move_object(source, Zone::Exile);
                }
            }
            drive(&mut g, None);
            assert_eq!(
                stats(&g, body),
                match mode {
                    0 => (7, 5),
                    1 => (-2, 4),
                    2 => (5, 5),
                    3 | 4 => (2, 2),
                    6 => (3, 3),
                    7 | 8 => (5, 5),
                    _ => (4, 4),
                },
                "mode{mode}"
            );
        }
    }
}
#[test]
fn hob_crooked_galion_galion_copied_trigger_samples_independently_with_owned_retarget() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let echo = put(&mut g, p, "Echo, Perceptive Prodigy", Zone::Battlefield);
        let source = galion(&mut g, p);
        let a = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let b = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        attack(&mut g, source);
        bind(&mut g, a);
        let original = g.state.stack.last().unwrap().id;
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
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
        until(&mut g, |g| g.observe(p).retarget.is_some());
        super::contracts::branch_contract(g.clone());
        let saved = g.snapshot();
        let mut raw = g.clone();
        let mut indexed = g.clone();
        drop(g);
        let action = Action::SetStackTarget {
            index: 0,
            target: Target::Card(b),
        };
        let index = indexed
            .legal_actions()
            .iter()
            .position(|a| *a == action)
            .unwrap();
        act(&mut raw, action.clone());
        indexed.apply_index(p, indexed.revision(), index).unwrap();
        assert_eq!(raw.snapshot(), indexed.snapshot());
        indexed.restore(&saved).unwrap();
        act(&mut indexed, action);
        assert_eq!(raw.snapshot(), indexed.snapshot());
        drop(indexed);
        act(&mut raw, Action::FinishStackTargets);
        raw.state.objects[source as usize].growth = Stats {
            power: 3,
            toughness: 1,
        };
        until(&mut raw, |g| {
            g.state.stack.last().is_some_and(|s| s.id == original)
        });
        assert_eq!(stats(&raw, b), (7, 5));
        raw.state.objects[source as usize].growth = Stats {
            power: 5,
            toughness: 2,
        };
        drive(&mut raw, None);
        assert_eq!(stats(&raw, a), (9, 6));
        assert_eq!(stats(&raw, b), (7, 5));
    }
}
