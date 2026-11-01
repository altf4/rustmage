//! Focused printed outcomes; shared engine invariants live in core tests.
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
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let offering = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.state.objects[offering as usize].growth.power = 3;
        let victim = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let bolg = start(&mut g, p);
        let self_ref = g.object_ref(bolg);
        reject(&mut g, Action::Sacrifice { object: self_ref });

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
    {
        let p = Player::P0;
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
fn bolg_actual_modified_damage_and_prior_damage_determine_excess() {
    {
        let p = Player::P0;
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
    {
        let p = Player::P0;
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
