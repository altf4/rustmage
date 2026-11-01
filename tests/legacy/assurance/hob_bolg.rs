//! Real sacrifice, separate reflexive target and actual excess-damage amass.
use super::*;
fn quiet(g: &Game) -> bool {
    g.state.pending == Pending::Priority
        && g.state.stack.is_empty()
        && g.state.pending_triggers.is_empty()
        && g.state.ordering_triggers.is_empty()
        && g.state.resolving.is_none()
}
fn until(g: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..96 {
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
    panic!("Bolg decision budget {:?}", g.state.pending);
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
                    | Action::ChooseReplacement { .. }
                    | Action::ChooseArmy { .. }
            )
        });
    }
    panic!("Bolg finish budget {:?}", g.state.pending);
}
fn start(g: &mut Game, p: Player) -> CardId {
    let id = put(g, p, "Bolg of the North", Zone::Hand);
    g.state.players[p.index()].mana = [0, 0, 1, 1, 0, 3];
    cast(g, id, None);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    until(g, |g| {
        matches!(g.state.pending, Pending::EffectChoice { .. })
    });
    assert_eq!(stats(g, id), (5, 5));
    id
}
fn sacrifice(g: &mut Game, id: CardId) {
    let object = g.object_ref(id);
    act(g, Action::Sacrifice { object });
}
fn targets(g: &mut Game) {
    until(g, |g| matches!(g.state.pending, Pending::Targets { .. }));
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
fn armies(g: &Game, p: Player) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter(|(id, o)| {
            o.zone == Zone::Battlefield && o.controller == p && g.has_subtype(*id as CardId, "Army")
        })
        .map(|(id, _)| id as CardId)
        .collect()
}
#[test]
fn bolg_paid_optional_and_actual_sacrifice_before_target_then_excess() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let offering = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.state.objects[offering as usize].growth.power = 3;
        let victim = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let bolg = start(&mut g, p);
        let self_ref = g.object_ref(bolg);
        reject(&mut g, Action::Sacrifice { object: self_ref });
        contracts::branch_contract(g.clone());
        sacrifice(&mut g, offering);
        assert_eq!(g.object_view(offering).zone, Zone::Graveyard);
        targets(&mut g);
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bolg),
            },
        );
        bind(&mut g, victim);
        finish(&mut g);
        assert_eq!(g.object_view(victim).zone, Zone::Graveyard);
        let armies = armies(&g, p);
        assert_eq!(armies.len(), 1);
        assert_eq!(g.counter_count(armies[0], CounterKind::PlusOne), 2);
        assert!(g.has_subtype(armies[0], "Goblin"));
    }
}
#[test]
fn bolg_decline_and_damage_prevented_do_not_amass() {
    for p in [Player::P0, Player::P1] {
        for prevented in [false, true] {
            let mut g = scenario(p);
            let offering = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            g.state.objects[offering as usize].growth.power = 4;
            let victim = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
            if prevented {
                g.add_counter(victim, CounterKind::Shield, 1);
            }
            start(&mut g, p);
            if !prevented {
                act(&mut g, Action::DeclineEffectPayment);
            } else {
                sacrifice(&mut g, offering);
                targets(&mut g);
                bind(&mut g, victim);
            }
            finish(&mut g);
            assert!(armies(&g, p).is_empty());
            assert_eq!(g.object_view(victim).zone, Zone::Battlefield);
            assert_eq!(g.state.objects[victim as usize].damage, 0);
        }
    }
}
#[test]
fn bolg_replaced_sacrifice_and_frozen_power_survive_source_departure_and_target_reentry() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let offering = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.state.objects[offering as usize].growth.power = 4;
        let victim = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        put(&mut g, p, "Rest in Peace", Zone::Battlefield);
        let bolg = start(&mut g, p);
        sacrifice(&mut g, offering);
        assert_eq!(g.object_view(offering).zone, Zone::Exile);
        targets(&mut g);
        let packet = g
            .state
            .announcing
            .as_ref()
            .unwrap()
            .item
            .sacrificed_power
            .as_deref()
            .copied();
        assert_eq!(packet, Some(6));
        bind(&mut g, victim);
        let mut stale = g.clone();
        stale.move_object(victim, Zone::Hand);
        stale.move_object(victim, Zone::Battlefield);
        g.move_object(bolg, Zone::Graveyard);
        finish(&mut g);
        finish(&mut stale);
        assert_eq!(g.counter_count(armies(&g, p)[0], CounterKind::PlusOne), 3);
        assert!(armies(&stale, p).is_empty());
        assert_eq!(stale.state.objects[victim as usize].damage, 0);
    }
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
    let id = put(g, p, "Bear Cub", Zone::Battlefield);
    types(g, id, &["Army"]);
    id
}
#[test]
fn bolg_multi_army_choice_uses_controller_and_owned_restore() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let a = army(&mut g, p);
        let b = army(&mut g, p);
        let enemy = army(&mut g, p.other());
        let offering = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.state.objects[offering as usize].growth.power = 3;
        let victim = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        start(&mut g, p);
        sacrifice(&mut g, offering);
        targets(&mut g);
        bind(&mut g, victim);
        until(&mut g, |g| {
            g.legal_actions().contains(&Action::ChooseArmy {
                object: g.object_ref(b),
            })
        });
        assert_eq!(g.actor(), Some(p));
        let foreign = g.object_ref(enemy);
        reject(&mut g, Action::ChooseArmy { object: foreign });
        contracts::branch_contract(g.clone());
        let snapshot = g.snapshot();
        let mut raw = g.clone();
        let mut indexed = g.clone();
        drop(g);
        let action = Action::ChooseArmy {
            object: raw.object_ref(b),
        };
        let index = indexed
            .legal_actions()
            .iter()
            .position(|x| x == &action)
            .unwrap();
        act(&mut raw, action.clone());
        indexed.apply_index(p, indexed.revision(), index).unwrap();
        finish(&mut raw);
        finish(&mut indexed);
        assert_eq!(raw.snapshot(), indexed.snapshot());
        indexed.restore(&snapshot).unwrap();
        act(&mut indexed, action);
        finish(&mut indexed);
        assert_eq!(raw.snapshot(), indexed.snapshot());
        assert_eq!(raw.counter_count(a, CounterKind::PlusOne), 0);
        assert_eq!(raw.counter_count(b, CounterKind::PlusOne), 2);
        assert_eq!(raw.counter_count(enemy, CounterKind::PlusOne), 0);
        assert!(raw.has_subtype(b, "Goblin"));
    }
}
#[test]
fn bolg_actual_modified_damage_and_prior_damage_determine_excess() {
    for p in [Player::P0, Player::P1] {
        for mode in 0..4 {
            let mut g = scenario(p);
            let offering = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let victim = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
            if mode == 0 {
                put(&mut g, p, "Twinflame Tyrant", Zone::Battlefield);
            }
            if mode == 1 {
                g.state.objects[victim as usize].damage = 2;
            }
            if mode == 2 {
                g.state.objects[offering as usize].growth.power = -4;
            }
            let source = start(&mut g, p);
            if mode == 3 {
                g.add_counter(source, CounterKind::Deathtouch, 1);
            }
            sacrifice(&mut g, offering);
            targets(&mut g);
            bind(&mut g, victim);
            finish(&mut g);
            if mode == 2 {
                assert!(armies(&g, p).is_empty());
                assert_eq!(g.state.objects[victim as usize].damage, 0);
            } else {
                assert_eq!(g.counter_count(armies(&g, p)[0], CounterKind::PlusOne), 1);
                assert_eq!(g.object_view(victim).zone, Zone::Graveyard);
            }
        }
    }
}
#[test]
fn bolg_exact_lethal_and_no_offering_do_not_create_army() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let offering = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.state.objects[offering as usize].growth.power = 1;
        let victim = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        start(&mut g, p);
        sacrifice(&mut g, offering);
        targets(&mut g);
        bind(&mut g, victim);
        finish(&mut g);
        assert_eq!(g.object_view(victim).zone, Zone::Graveyard);
        assert!(armies(&g, p).is_empty());
        let mut g = scenario(p);
        let id = put(&mut g, p, "Bolg of the North", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 1, 1, 0, 3];
        cast(&mut g, id, None);
        finish(&mut g);
        assert_eq!(g.object_view(id).zone, Zone::Battlefield);
        assert!(armies(&g, p).is_empty());
    }
}

#[test]
fn bolg_token_and_counter_replacement_resumes_before_army_choice() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p, "Doubling Season", Zone::Battlefield);
        let offering = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.state.objects[offering as usize].growth.power = 3;
        let victim = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        start(&mut g, p);
        sacrifice(&mut g, offering);
        targets(&mut g);
        bind(&mut g, victim);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectChoice { .. }) && armies(g, p).len() == 2
        });
        let candidates = armies(&g, p);
        let object = g.object_ref(candidates[1]);
        contracts::branch_contract(g.clone());
        act(&mut g, Action::ChooseArmy { object });
        finish(&mut g);
        assert_eq!(g.counter_count(candidates[0], CounterKind::PlusOne), 0);
        assert_eq!(g.counter_count(candidates[1], CounterKind::PlusOne), 4);
    }
}
