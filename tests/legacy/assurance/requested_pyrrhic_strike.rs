//! Optional blight selects modes before paying a non-targeted counter cost.
use super::big_helpers::{checked, settle};
use super::*;

fn choose_cost(g: &mut Game, blight: bool) {
    let index = g
        .state
        .announcing
        .as_ref()
        .unwrap()
        .cast_costs
        .as_ref()
        .unwrap()
        .clauses
        .iter()
        .position(|r| {
            if blight {
                matches!(r.clause, CastCostClause::Blight { .. })
            } else {
                r.clause == CastCostClause::Mana
            }
        })
        .unwrap() as u16;
    checked(g, Action::ChooseCastCost { clause: index });
}

#[test]
fn pyrrhic_modes_follow_blight_and_counter_cost_is_not_targeted() {
    for p in [Player::P0, Player::P1] {
        for paid in [false, true] {
            let mut g = scenario(p);
            let spell = put(&mut g, p, "Pyrrhic Strike", Zone::Hand);
            let artifact = put(&mut g, p.other(), "Short Bow", Zone::Battlefield);
            let victim = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
            let creature = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            checked(&mut g, Action::AnnounceCast { card: spell });
            checked(&mut g, Action::ChooseBlight { pay: paid });
            reject(
                &mut g,
                Action::ChooseMode {
                    mode: if paid { 0 } else { 2 },
                },
            );
            checked(
                &mut g,
                Action::ChooseMode {
                    mode: if paid { 2 } else { 0 },
                },
            );
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(artifact),
                },
            );
            if paid {
                if !g.legal_actions().contains(&Action::ChooseTarget {
                    target: Target::Card(victim),
                }) {
                    checked(&mut g, Action::FinishTargets);
                }
                checked(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(victim),
                    },
                );
                checked(&mut g, Action::BeginOrderedCosts);
                choose_cost(&mut g, true);
                let enemy = g.object_ref(victim);
                reject(&mut g, Action::PayBlight { object: enemy });
                let object = g.object_ref(creature);
                checked(&mut g, Action::PayBlight { object });
                assert_eq!(g.counter_count(creature, CounterKind::MinusOne), 2);
                choose_cost(&mut g, false);
            }
            pay(&mut g);
            settle(&mut g);
            assert_eq!(g.object_view(artifact).zone, Zone::Graveyard);
            assert_eq!(
                g.object_view(victim).zone,
                if paid {
                    Zone::Graveyard
                } else {
                    Zone::Battlefield
                }
            );
            assert_eq!(
                g.object_view(creature).zone,
                if paid {
                    Zone::Graveyard
                } else {
                    Zone::Battlefield
                }
            );
        }
    }
}

#[test]
fn pyrrhic_paid_mode_unavailable_without_controlled_creature() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let spell = put(&mut g, p, "Pyrrhic Strike", Zone::Hand);
        put(&mut g, p.other(), "Short Bow", Zone::Battlefield);
        put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        checked(&mut g, Action::AnnounceCast { card: spell });
        reject(&mut g, Action::ChooseBlight { pay: true });
        checked(&mut g, Action::ChooseBlight { pay: false });
    }
}

#[test]
fn counter_prohibition_blocks_blight_and_effects_until_aura_leaves() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let creature = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let aura = put(&mut g, p, "Pacifism", Zone::Battlefield);
        g.state.objects[aura as usize].card = Card::from_name("Blossombind").unwrap();
        g.state.objects[aura as usize].attached_to = Some(g.object_ref(creature));
        let object = g.object_ref(creature);
        assert!(replacements::counters_prohibited(&g, object));
        g.effect_counters(
            vec![(object, Counters::of(CounterKind::PlusOne, 1))],
            replacements::ResumeMode::Pending,
        );
        g.effect_counter(
            creature,
            CounterKind::PlusOne,
            1,
            replacements::ResumeMode::Pending,
        );
        assert_eq!(g.counter_count(creature, CounterKind::PlusOne), 0);
        let spell = put(&mut g, p, "Pyrrhic Strike", Zone::Hand);
        put(&mut g, p.other(), "Short Bow", Zone::Battlefield);
        put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        checked(&mut g, Action::AnnounceCast { card: spell });
        reject(&mut g, Action::ChooseBlight { pay: true });
        checked(&mut g, Action::ChooseBlight { pay: false });
        g.move_object(aura, Zone::Graveyard);
        assert!(!replacements::counters_prohibited(&g, object));
    }
}

#[test]
fn pyrrhic_blight_and_mana_can_be_paid_in_either_order() {
    for p in [Player::P0, Player::P1] {
        for mana_first in [false, true] {
            let mut g = scenario(p);
            let spell = put(&mut g, p, "Pyrrhic Strike", Zone::Hand);
            let artifact = put(&mut g, p.other(), "Short Bow", Zone::Battlefield);
            let victim = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
            let creature = put(&mut g, p, "Hill Giant", Zone::Battlefield);
            checked(&mut g, Action::AnnounceCast { card: spell });
            checked(&mut g, Action::ChooseBlight { pay: true });
            checked(&mut g, Action::ChooseMode { mode: 2 });
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(artifact),
                },
            );
            if g.legal_actions().contains(&Action::FinishTargets) {
                checked(&mut g, Action::FinishTargets);
            }
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(victim),
                },
            );
            checked(&mut g, Action::BeginOrderedCosts);
            super::contracts::branch_contract(g.clone());
            if mana_first {
                choose_cost(&mut g, false);
                pay(&mut g);
            }
            choose_cost(&mut g, true);
            let object = g.object_ref(creature);
            checked(&mut g, Action::PayBlight { object });
            if !mana_first {
                choose_cost(&mut g, false);
                pay(&mut g);
            }
            settle(&mut g);
            assert_eq!(g.counter_count(creature, CounterKind::MinusOne), 2);
            assert_eq!(g.object_view(artifact).zone, Zone::Graveyard);
            assert_eq!(g.object_view(victim).zone, Zone::Graveyard);
        }
    }
}

#[test]
fn pyrrhic_blight_does_not_reallocate_an_illegal_target() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let spell = put(&mut g, p, "Pyrrhic Strike", Zone::Hand);
        let artifact = put(&mut g, p.other(), "Short Bow", Zone::Battlefield);
        let victim = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let creature = put(&mut g, p, "Hill Giant", Zone::Battlefield);
        checked(&mut g, Action::AnnounceCast { card: spell });
        checked(&mut g, Action::ChooseBlight { pay: true });
        checked(&mut g, Action::ChooseMode { mode: 2 });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(victim),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(artifact),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            checked(&mut g, Action::FinishTargets);
        }
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(victim),
            },
        );
        checked(&mut g, Action::BeginOrderedCosts);
        choose_cost(&mut g, true);
        let object = g.object_ref(creature);
        checked(&mut g, Action::PayBlight { object });
        choose_cost(&mut g, false);
        pay(&mut g);
        g.move_object(victim, Zone::Hand);
        settle(&mut g);
        assert_eq!(g.object_view(artifact).zone, Zone::Graveyard);
        assert_eq!(g.object_view(victim).zone, Zone::Hand);
        assert_eq!(g.counter_count(creature, CounterKind::MinusOne), 2);
    }
}
