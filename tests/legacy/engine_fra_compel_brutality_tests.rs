use super::*;

fn announce_compel(g: &mut Game, spell: CardId, mode: u16, source: CardId, target: CardId) {
    act(g, Action::AnnounceCast { card: spell });
    act(g, Action::ChooseMode { mode });
    act(
        g,
        Action::ChooseTarget {
            target: Target::Card(source),
        },
    );
    act(
        g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
    pay(g);
}

#[test]
fn compel_creature_damage_uses_current_power_and_its_lifelink() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let target = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        // Keep the target alive so its marked damage remains observable.
        g.state.objects[target as usize].growth.toughness += 1;
        g.state.objects[source as usize].gained_keywords = Keywords::LIFELINK;
        let life = g.state.players[p.index()].life;
        let spell = put(&mut g, p, "Compel Brutality", Zone::Hand);
        announce_compel(&mut g, spell, 0, source, target);
        g.state.objects[source as usize].growth.power += 1;
        let saved = g.snapshot();
        super::super::contracts::branch_contract(g.clone());
        for replay in 0..2 {
            if replay == 1 {
                g.restore(&saved).unwrap();
            }
            settle(&mut g);
            assert_eq!(g.object_view(target).damage, 3);
            assert_eq!(g.state.players[p.index()].life, life + 3);
        }
    }
}

#[test]
fn compel_planeswalker_damage_uses_live_loyalty_and_prevention() {
    for p in [Player::P0, Player::P1] {
        for shielded in [false, true] {
            let mut g = scenario(p);
            let source = put(&mut g, p, "Ajani, Caller of the Pride", Zone::Battlefield);
            let target = put(
                &mut g,
                p.other(),
                "Ajani, Caller of the Pride",
                Zone::Battlefield,
            );
            g.state.objects[source as usize].counters.loyalty = 2;
            g.state.objects[target as usize].counters.loyalty = 6;
            if shielded {
                g.add_counter(target, CounterKind::Shield, 1);
            }
            let spell = put(&mut g, p, "Compel Brutality", Zone::Hand);
            announce_compel(&mut g, spell, 1, source, target);
            g.add_counter(source, CounterKind::Loyalty, 2);
            settle(&mut g);
            assert_eq!(
                g.counter_count(target, CounterKind::Loyalty),
                if shielded { 6 } else { 2 }
            );
        }
    }
}

#[test]
fn compel_needs_both_operands_to_remain_legal() {
    for p in [Player::P0, Player::P1] {
        for mode in 0..=1 {
            for change_source in [false, true] {
                let mut g = scenario(p);
                let source = put(
                    &mut g,
                    p,
                    if mode == 0 {
                        "Bear Cub"
                    } else {
                        "Ajani, Caller of the Pride"
                    },
                    Zone::Battlefield,
                );
                if mode == 1 {
                    g.state.objects[source as usize].counters.loyalty = 3;
                }
                let target = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
                let spell = put(&mut g, p, "Compel Brutality", Zone::Hand);
                announce_compel(&mut g, spell, mode, source, target);
                let changed = if change_source { source } else { target };
                let new_controller = if change_source { p.other() } else { p };
                g.change_controller(g.object_ref(changed), new_controller);
                settle(&mut g);
                assert_eq!(g.object_view(target).damage, 0);
                assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
            }
        }
    }
}

#[test]
fn compel_modes_require_correct_controllers_and_types() {
    for p in [Player::P0, Player::P1] {
        for mode in 0..=1 {
            let mut g = scenario(p);
            let own_creature = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let own_walker = put(&mut g, p, "Ajani, Caller of the Pride", Zone::Battlefield);
            g.state.objects[own_walker as usize].counters.loyalty = 4;
            let enemy = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
            let spell = put(&mut g, p, "Compel Brutality", Zone::Hand);
            act(&mut g, Action::AnnounceCast { card: spell });
            act(&mut g, Action::ChooseMode { mode });
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(enemy),
                },
            );
            let (source, wrong_type) = if mode == 0 {
                (own_creature, own_walker)
            } else {
                (own_walker, own_creature)
            };
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(wrong_type),
                },
            );
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(source),
                },
            );
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(own_creature),
                },
            );
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(enemy),
                },
            );
            if g.legal_actions().contains(&Action::FinishTargets) {
                act(&mut g, Action::FinishTargets);
            }
            pay(&mut g);
            settle(&mut g);
        }
    }
}
