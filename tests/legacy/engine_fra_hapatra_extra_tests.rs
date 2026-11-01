use super::*;

fn enter_and_choose(g: &mut Game, p: Player, name: &str, target: Option<CardId>) -> CardId {
    let source = put(g, p, name, Zone::Hand);
    cast(g, source, None);
    let mut selected = false;
    for _ in 0..48 {
        g.prepare_triggers();
        let actions = g.legal_actions();
        if let Some(id) = target
            && !selected
            && actions.contains(&Action::ChooseTarget {
                target: Target::Card(id),
            })
        {
            act(
                g,
                Action::ChooseTarget {
                    target: Target::Card(id),
                },
            );
            selected = true;
        } else if actions.contains(&Action::FinishTargets) {
            act(g, Action::FinishTargets);
            return source;
        } else if selected && g.state.pending == Pending::Priority {
            return source;
        } else if let Some(action) = actions
            .into_iter()
            .find(|a| matches!(a, Action::Pass | Action::PutTrigger { .. }))
        {
            act(g, action);
        } else {
            panic!("unexpected Hapatra choice {:?}", g.state.pending);
        }
    }
    panic!("Hapatra target selection did not finish")
}

#[test]
fn hapatra_entry_optional_target_and_changed_control() {
    for p in [Player::P0, Player::P1] {
        for name in ["Hapatra, the Desert Fang", "Hapatra, the Desert Frost"] {
            for choice in 0..3 {
                let mut g = scenario(p);
                let victim = put(&mut g, p.other(), "Gigantosaurus", Zone::Battlefield);
                put(&mut g, p, "Bear Cub", Zone::Graveyard);
                let source = enter_and_choose(&mut g, p, name, (choice != 0).then_some(victim));
                if choice == 2 {
                    let victim_ref = g.object_ref(victim);
                    g.change_controller(victim_ref, p);
                }
                g.move_object(source, Zone::Exile);
                let before = g.snapshot();
                super::super::contracts::branch_contract(g.clone());
                for replay in 0..2 {
                    if replay == 1 {
                        g.restore(&before).unwrap();
                    }
                    settle(&mut g);
                    let applied = choice == 1;
                    if name == "Hapatra, the Desert Fang" {
                        assert_eq!(
                            g.counter_count(victim, CounterKind::MinusOne),
                            if applied { 2 } else { 0 }
                        );
                    } else {
                        assert_eq!(
                            g.counter_count(victim, CounterKind::Stun),
                            u64::from(applied)
                        );
                        assert_eq!(g.object_view(victim).tapped, applied);
                    }
                }
            }
        }
    }
}

#[test]
fn hapatra_empty_graveyard_and_counter_replacements() {
    for p in [Player::P0, Player::P1] {
        for name in ["Hapatra, the Desert Fang", "Hapatra, the Desert Frost"] {
            for double in [false, true] {
                let mut g = scenario(p);
                let victim = put(&mut g, p.other(), "Gigantosaurus", Zone::Battlefield);
                if double {
                    put(&mut g, p.other(), "Doubling Season", Zone::Battlefield);
                }
                let source = enter_and_choose(&mut g, p, name, Some(victim));
                let snapshot = g.snapshot();
                super::super::contracts::branch_contract(g.clone());
                for replay in 0..2 {
                    if replay == 1 {
                        g.restore(&snapshot).unwrap();
                    }
                    settle(&mut g);
                    assert_eq!(g.object_view(source).zone, Zone::Battlefield);
                    if name == "Hapatra, the Desert Fang" {
                        assert_eq!(g.counter_count(victim, CounterKind::MinusOne), 0);
                    } else {
                        assert!(g.object_view(victim).tapped);
                        assert_eq!(
                            g.counter_count(victim, CounterKind::Stun),
                            if double { 2 } else { 1 }
                        );
                    }
                }
            }
        }
        let mut g = scenario(p);
        let victim = put(&mut g, p.other(), "Gigantosaurus", Zone::Battlefield);
        put(&mut g, p, "Bear Cub", Zone::Graveyard);
        put(&mut g, p.other(), "Doubling Season", Zone::Battlefield);
        enter_and_choose(&mut g, p, "Hapatra, the Desert Fang", Some(victim));
        settle(&mut g);
        assert_eq!(g.counter_count(victim, CounterKind::MinusOne), 4);
    }
}

#[test]
fn hapatra_frost_untap_survives_source_departure() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let victim = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.state.objects[victim as usize].tapped = true;
        let source = put(&mut g, p, "Hapatra, the Desert Frost", Zone::Battlefield);
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 2];
        act(
            &mut g,
            Action::AnnounceActivate {
                card: source,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(victim),
            },
        );
        pay(&mut g);
        g.move_object(source, Zone::Exile);
        let snapshot = g.snapshot();
        super::super::contracts::branch_contract(g.clone());
        for replay in 0..2 {
            if replay == 1 {
                g.restore(&snapshot).unwrap();
            }
            settle(&mut g);
            assert!(!g.object_view(victim).tapped);
        }
    }
}
