use super::*;
pub(super) fn drive(g: &mut Game) -> usize {
    let mut scries = 0;
    for _ in 0..96 {
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty()
            && g.state.resolving.is_none()
        {
            return scries;
        }
        let actions = g.legal_actions();
        let action = if let Some(a) = actions.iter().find(|a| {
            matches!(
                a,
                Action::ArrangeLibrary {
                    destination: LibraryDestination::Bottom,
                    ..
                }
            )
        }) {
            let p = g.actor().unwrap();
            let view = g.observe(p).library_choice.unwrap();
            assert_eq!(view.remaining.len(), 1);
            assert!(g.observe(p.other()).library_choice.is_none());
            scries += 1;
            a.clone()
        } else {
            actions
                .into_iter()
                .find(|a| {
                    matches!(
                        a,
                        Action::PutTrigger { .. } | Action::FinishTargets | Action::Pass
                    )
                })
                .unwrap_or_else(|| panic!("explicit choice {:?}", g.state.pending))
        };
        act(g, action);
    }
    panic!("death/return resolution budget");
}
fn until(g: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..32 {
        if ready(g) {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PutTrigger { .. } | Action::Pass))
            .unwrap();
        act(g, a);
    }
    panic!("missing expected choice");
}
fn spell(g: &mut Game, p: Player, name: &str, mana: Mana, target: Option<Target>) -> CardId {
    let id = put(g, p, name, Zone::Hand);
    g.state.players[p.index()].mana = mana;
    cast(g, id, target);
    id
}
fn bee(g: &mut Game, p: Player) -> CardId {
    let id = spell(g, p, "Great Fierce Bee", [0, 0, 1, 0, 0, 2], None);
    settle(g);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    assert_eq!(stats(g, id), (2, 2));
    assert!(g.keywords(id).contains(Keywords::FLYING));
    id
}
fn staff(g: &mut Game, p: Player, body: CardId) -> CardId {
    let id = put(g, p, "Wizard's Staff", Zone::Hand);
    g.state.players[p.index()].mana = [12; 6];
    cast(g, id, None);
    settle(g);
    act(
        g,
        Action::AnnounceActivate {
            card: id,
            ability: 1,
            counter_payment: Counters::default(),
        },
    );
    act(
        g,
        Action::ChooseTarget {
            target: Target::Card(body),
        },
    );
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
    pay(g);
    settle(g);
    id
}
#[test]
fn hob_death_return_bee_batch_other_any_controller_tokens_and_distinct_deaths() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = bee(&mut g, p);
        let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        put(&mut g, p, "@token/hob-wolf", Zone::Battlefield);
        spell(&mut g, p, "Day of Judgment", [2, 0, 0, 0, 0, 2], None);
        assert_eq!(drive(&mut g), 1);
        for id in [source, own, enemy] {
            assert_eq!(g.object_view(id).zone, Zone::Graveyard);
        }
        let mut g = scenario(p);
        let source = bee(&mut g, p);
        for owner in [p, p.other()] {
            let body = put(&mut g, owner, "Bear Cub", Zone::Battlefield);
            spell(
                &mut g,
                p,
                "Murder",
                [0, 0, 2, 0, 0, 1],
                Some(Target::Card(body)),
            );
            assert_eq!(drive(&mut g), 1);
        }
        spell(
            &mut g,
            p,
            "Murder",
            [0, 0, 2, 0, 0, 1],
            Some(Target::Card(source)),
        );
        assert_eq!(drive(&mut g), 0, "self alone is excluded");
        let mut g = scenario(p);
        bee(&mut g, p);
        put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        put(&mut g, p, "Rest in Peace", Zone::Battlefield);
        spell(&mut g, p, "Day of Judgment", [2, 0, 0, 0, 0, 2], None);
        assert_eq!(drive(&mut g), 0, "exiled creatures did not die");
    }
}
#[test]
fn hob_death_return_bee_staff_uses_frozen_predeath_multiplier() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = bee(&mut g, p);
        let staff = staff(&mut g, p, source);
        put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        put(&mut g, p, "Bear Cub", Zone::Battlefield);
        assert_eq!(g.object_view(source).additional_triggers, 1);
        spell(&mut g, p, "Day of Judgment", [2, 0, 0, 0, 0, 2], None);
        assert_eq!(
            drive(&mut g),
            2,
            "one batch trigger plus one extra instance, not one per victim"
        );
        assert_eq!(g.object_view(source).zone, Zone::Graveyard);
        assert_eq!(g.object_view(staff).attached_to, None);
    }
}
fn nurture(g: &mut Game, p: Player, blue: bool) -> CardId {
    let mana = if blue {
        [0, 1, 0, 0, 0, 2]
    } else {
        [0, 0, 0, 0, 1, 2]
    };
    let id = spell(g, p, "Mirkwood Nurturer", mana, None);
    until(g, |g| g.legal_actions().contains(&Action::FinishTargets));
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    id
}
fn target(g: &mut Game, id: CardId) {
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
#[test]
fn hob_death_return_nurturer_both_hybrid_costs_decline_lands_stolen_tokens_and_legality() {
    for p in [Player::P0, Player::P1] {
        for blue in [false, true] {
            for mode in 0..4 {
                let mut g = scenario(p);
                let own = put(&mut g, p, "Forest", Zone::Battlefield);
                let stolen = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
                g.record_control(g.object_ref(stolen), p, None);
                let token = put(&mut g, p, "@token/hob-wolf", Zone::Battlefield);
                let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
                let source = nurture(&mut g, p, blue);
                reject(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(source),
                    },
                );
                reject(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(enemy),
                    },
                );
                let selected = match mode {
                    0 => None,
                    1 => Some(own),
                    2 => Some(stolen),
                    _ => Some(token),
                };
                if let Some(id) = selected {
                    target(&mut g, id);
                } else {
                    act(&mut g, Action::FinishTargets);
                }
                assert_eq!(drive(&mut g), 0);
                assert_eq!(
                    g.counter_count(source, CounterKind::PlusOne),
                    u64::from(mode != 0)
                );
                assert_eq!(stats(&g, source), if mode == 0 { (3, 2) } else { (4, 3) });
                if mode == 1 {
                    assert_eq!(g.object_view(own).zone, Zone::Hand);
                }
                if mode == 2 {
                    let view = g.object_view(stolen);
                    assert_eq!(view.zone, Zone::Hand);
                    assert_eq!(view.owner, p.other());
                }
                if mode == 3 {
                    assert_eq!(g.object_view(token).zone, Zone::Outside);
                }
            }
        }
    }
}
#[test]
fn hob_death_return_nurturer_stale_targets_source_incarnation_and_counter_replacement() {
    for p in [Player::P0, Player::P1] {
        for mode in 0..4 {
            let mut g = scenario(p);
            let land = put(&mut g, p, "Forest", Zone::Battlefield);
            if mode == 3 {
                put(&mut g, p, "Doubling Season", Zone::Battlefield);
            }
            let source = nurture(&mut g, p, false);
            target(&mut g, land);
            if mode == 0 {
                g.move_object(land, Zone::Hand);
                g.move_object(land, Zone::Battlefield);
            }
            if mode == 1 {
                g.move_object(source, Zone::Exile);
            }
            if mode == 2 {
                g.move_object(source, Zone::Exile);
                g.move_object(source, Zone::Battlefield);
                g.prepare_triggers();
            }
            if mode == 2 {
                // Explicitly decline the new incarnation's independent entry trigger.
                until(&mut g, |g| {
                    g.legal_actions().contains(&Action::FinishTargets)
                });
                act(&mut g, Action::FinishTargets);
            }
            drive(&mut g);
            assert_eq!(
                g.counter_count(source, CounterKind::PlusOne),
                if mode == 3 { 2 } else { 0 }
            );
            assert_eq!(
                g.object_view(land).zone,
                if mode == 0 {
                    Zone::Battlefield
                } else {
                    Zone::Hand
                }
            );
        }
    }
}
#[test]
fn hob_death_return_nurturer_linked_return_entry_choice_preserves_owned_continuation() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let automaton = put(&mut g, p.other(), "Adaptive Automaton", Zone::Battlefield);
        let prison = spell(&mut g, p, "Banishing Light", [1, 0, 0, 0, 0, 2], None);
        until(&mut g, |g| {
            g.legal_actions().contains(&Action::ChooseTarget {
                target: Target::Card(automaton),
            })
        });
        target(&mut g, automaton);
        settle(&mut g);
        assert_eq!(g.object_view(automaton).zone, Zone::Exile);
        let source = nurture(&mut g, p, true);
        target(&mut g, prison);
        until(&mut g, |g| {
            g.observe(g.actor().unwrap()).entry_value.is_some()
        });
        assert_eq!(g.object_view(prison).zone, Zone::Hand);
        assert_eq!(g.counter_count(source, CounterKind::PlusOne), 0);
        super::contracts::branch_contract(g.clone());
        let saved = g.snapshot();
        let mut raw = g.clone();
        let mut indexed = g.clone();
        drop(g);
        let action = raw
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::ChooseEntryValue {
                        value: EntryChoiceValue::CreatureType(_)
                    }
                )
            })
            .unwrap();
        let actor = raw.actor().unwrap();
        let index = indexed
            .legal_actions()
            .iter()
            .position(|a| *a == action)
            .unwrap();
        act(&mut raw, action.clone());
        indexed
            .apply_index(actor, indexed.revision(), index)
            .unwrap();
        drive(&mut raw);
        drive(&mut indexed);
        assert_eq!(raw.snapshot(), indexed.snapshot());
        assert_eq!(raw.counter_count(source, CounterKind::PlusOne), 1);
        assert_eq!(raw.object_view(automaton).zone, Zone::Battlefield);
        indexed.restore(&saved).unwrap();
        act(&mut indexed, action);
        drive(&mut indexed);
        assert_eq!(raw.snapshot(), indexed.snapshot());
    }
}
#[test]
fn hob_death_return_nurturer_copied_instances_keep_targets_and_success_independent() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let a = put(&mut g, p, "Forest", Zone::Battlefield);
        let b = put(&mut g, p, "Island", Zone::Battlefield);
        let echo = put(&mut g, p, "Echo, Perceptive Prodigy", Zone::Battlefield);
        let source = nurture(&mut g, p, false);
        target(&mut g, a);
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
        act(
            &mut g,
            Action::SetStackTarget {
                index: 0,
                target: Target::Card(b),
            },
        );
        act(&mut g, Action::FinishStackTargets);
        let copied = g.state.stack.iter().find(|s| s.is_copy).unwrap().id;
        until(&mut g, |g| !g.state.stack.iter().any(|s| s.id == copied));
        assert_eq!(g.object_view(b).zone, Zone::Hand);
        assert_eq!(g.counter_count(source, CounterKind::PlusOne), 1);
        g.move_object(a, Zone::Exile);
        drive(&mut g);
        assert_eq!(
            g.counter_count(source, CounterKind::PlusOne),
            1,
            "fizzled original does not inherit copy's success"
        );
    }
}
