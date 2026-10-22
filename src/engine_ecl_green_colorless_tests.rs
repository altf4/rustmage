//! Paid ECL rules scenarios, with raw/indexed branches and restore on every action.
use super::ability_timeline_tests::{cast, checked, named, settle, setup};
use super::*;

fn reject(g: &mut Game, a: Action) {
    let before = g.snapshot();
    let rev = g.revision();
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), rev);
}
fn pay(g: &mut Game) {
    let a = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    checked(g, a);
}
#[test]
fn ecl_green_assert_optional_bite_boosts_first_group_and_revalidates_targets() {
    for p in [Player::P0, Player::P1] {
        for mode in 0..4 {
            let mut g = setup(p);
            let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let foe = named(
                &mut g,
                p.other(),
                "Quakestrider Ceratops",
                Zone::Battlefield,
            );
            let spell = named(&mut g, p, "Assert Perfection", Zone::Hand);
            let mana = g.state.players[p.index()].mana.iter().sum::<u32>();
            let targets = if mode == 0 {
                vec![Target::Card(bear)]
            } else {
                vec![Target::Card(bear), Target::Card(foe)]
            };
            cast(&mut g, spell, None, &targets);
            assert_eq!(
                g.state.players[p.index()].mana.iter().sum::<u32>(),
                mana - 2
            );
            if mode == 2 {
                g.move_object(foe, Zone::Exile);
            }
            if mode == 3 {
                g.move_object(bear, Zone::Exile);
                g.move_object(bear, Zone::Battlefield);
            }
            settle(&mut g);
            assert_eq!(g.stats(bear).power, if mode == 3 { 2 } else { 3 });
            assert_eq!(g.object_view(foe).damage, if mode == 1 { 3 } else { 0 });
            assert_eq!(g.object_view(bear).damage, 0);
        }
    }
}
#[test]
fn ecl_green_cavalry_paid_elf_permanent_boost_and_one_blocker() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let cavalry = named(&mut g, p, "Safewright Cavalry", Zone::Hand);
        cast(&mut g, cavalry, None, &[]);
        settle(&mut g);
        assert_eq!(
            g.stats(cavalry),
            Stats {
                power: 4,
                toughness: 4
            }
        );
        assert!(g.keywords(cavalry).contains(Keywords::MAX_ONE_BLOCKER));
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Llanowar Elves", Zone::Battlefield);
        let mana = g.state.players[p.index()].mana.iter().sum::<u32>();
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: cavalry,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bear),
            },
        );
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(cavalry),
            },
        );
        pay(&mut g);
        settle(&mut g);
        assert_eq!(
            g.stats(cavalry),
            Stats {
                power: 6,
                toughness: 6
            }
        );
        assert_eq!(
            g.state.players[p.index()].mana.iter().sum::<u32>(),
            mana - 5
        );
        g.state.active = p;
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        g.state.objects[cavalry as usize].summoning_sick = false;
        checked(&mut g, Action::Attack { card: cavalry });
        checked(&mut g, Action::FinishAttackers);
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        let blocker = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        checked(
            &mut g,
            Action::Block {
                blocker: enemy,
                attacker: cavalry,
            },
        );
        reject(
            &mut g,
            Action::Block {
                blocker,
                attacker: cavalry,
            },
        );
    }
}
#[test]
fn ecl_green_battler_entry_counters_trigger_only_while_counter_present() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let battler = named(&mut g, p, "Bristlebane Battler", Zone::Hand);
        cast(&mut g, battler, None, &[]);
        settle(&mut g);
        assert_eq!(g.counter_count(battler, CounterKind::MinusOne), 5);
        assert_eq!(
            g.stats(battler),
            Stats {
                power: 1,
                toughness: 1
            }
        );
        assert!(g.keywords(battler).contains(Keywords::TRAMPLE));
        for expected in (0..5).rev() {
            let creature = named(&mut g, p, "Bear Cub", Zone::Hand);
            cast(&mut g, creature, None, &[]);
            settle(&mut g);
            assert_eq!(g.counter_count(battler, CounterKind::MinusOne), expected);
        }
        assert_eq!(
            g.stats(battler),
            Stats {
                power: 6,
                toughness: 6
            }
        );
        let creature = named(&mut g, p, "Bear Cub", Zone::Hand);
        cast(&mut g, creature, None, &[]);
        settle(&mut g);
        assert_eq!(g.counter_count(battler, CounterKind::MinusOne), 0);
        assert!(g.step_guard(
            StepGuard::SourceCounters {
                kind: CounterKind::MinusOne,
                minimum: None,
                maximum: Some(0)
            },
            p,
            g.object_ref(battler),
            false
        ));
    }
}
#[test]
fn ecl_green_drum_taps_summoning_sick_creature_as_cost_and_adds_each_color_without_stack() {
    for p in [Player::P0, Player::P1] {
        for color in 0..5 {
            let mut g = setup(p);
            let drum = named(&mut g, p, "Springleaf Drum", Zone::Hand);
            cast(&mut g, drum, None, &[]);
            settle(&mut g);
            assert!(
                !g.legal_actions()
                    .iter()
                    .any(|a| matches!(a, Action::AnnounceMana { card, .. } if *card == drum))
            );
            let creature = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            g.state.objects[creature as usize].summoning_sick = true;
            let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let mut production = [0; 6];
            production[color] = 1;
            let before = g.state.players[p.index()].mana;
            checked(
                &mut g,
                Action::AnnounceMana {
                    card: drum,
                    ability: 0,
                    counter_payment: Counters::default(),
                    production,
                },
            );
            let object = g.object_ref(enemy);
            reject(&mut g, Action::ChooseAdditionalCost { object });
            let object = g.object_ref(drum);
            reject(&mut g, Action::ChooseAdditionalCost { object });
            let object = g.object_ref(creature);
            checked(&mut g, Action::ChooseAdditionalCost { object });
            pay(&mut g);
            assert!(g.state.stack.is_empty());
            assert!(g.object_view(creature).tapped && g.object_view(drum).tapped);
            for (index, old) in before.into_iter().enumerate() {
                assert_eq!(
                    g.state.players[p.index()].mana[index],
                    old + u32::from(index == color)
                );
            }
        }
    }
}
#[test]
fn ecl_green_battler_ward_paid_declined_and_source_leaves() {
    for p in [Player::P0, Player::P1] {
        for (accept, source_leaves) in [(false, false), (true, false), (false, true)] {
            let mut g = setup(p);
            let battler = named(&mut g, p.other(), "Bristlebane Battler", Zone::Battlefield);
            let spell = named(&mut g, p, "Shock", Zone::Hand);
            cast(&mut g, spell, None, &[Target::Card(battler)]);
            g.prepare_triggers();
            for _ in 0..20 {
                if g.legal_actions().contains(&Action::DeclineResolutionMana) {
                    break;
                }
                let a = if g.state.pending == Pending::Priority {
                    Action::Pass
                } else {
                    g.legal_actions()
                        .into_iter()
                        .find(|a| matches!(a, Action::PutTrigger { .. }))
                        .unwrap()
                };
                checked(&mut g, a);
            }
            let before = g.state.players[p.index()].mana.iter().sum::<u32>();
            if source_leaves {
                g.move_object(battler, Zone::Exile);
            }
            reject(&mut g, Action::PayResolutionMana { payment: [0; 6] });
            let a = if accept {
                g.legal_actions()
                    .into_iter()
                    .find(|a| matches!(a, Action::PayResolutionMana { .. }))
                    .unwrap()
            } else {
                Action::DeclineResolutionMana
            };
            checked(&mut g, a);
            settle(&mut g);
            assert_eq!(
                g.state.players[p.index()].mana.iter().sum::<u32>(),
                before - if accept { 2 } else { 0 }
            );
            assert_eq!(g.object_view(battler).damage, if accept { 2 } else { 0 });
            assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
        }
    }
}
#[test]
fn ecl_green_drum_funding_requires_distinct_untapped_creatures_and_resumes_outer_spell() {
    for p in [Player::P0, Player::P1] {
        for creatures in 0..=2 {
            let mut g = setup(p);
            let drums = [
                named(&mut g, p, "Springleaf Drum", Zone::Battlefield),
                named(&mut g, p, "Springleaf Drum", Zone::Battlefield),
            ];
            let mut witnesses = Vec::new();
            for _ in 0..creatures {
                let c = named(&mut g, p, "Bear Cub", Zone::Battlefield);
                g.state.objects[c as usize].summoning_sick = true;
                witnesses.push(c);
            }
            g.state.players[p.index()].mana = [0; 6];
            let spell = named(&mut g, p, "Bear Cub", Zone::Hand);
            let announce = Action::AnnounceCast { card: spell };
            assert_eq!(g.legal_actions().contains(&announce), creatures == 2);
            if creatures != 2 {
                reject(&mut g, announce);
                continue;
            }
            checked(&mut g, announce);
            for _ in 0..2 {
                let a = g
                    .legal_actions()
                    .into_iter()
                    .find(|a| matches!(a, Action::ActivateManaWithTapCost { .. }))
                    .unwrap();
                if let Action::ActivateManaWithTapCost { objects, .. } = &a {
                    assert_eq!(objects.len(), 1);
                    assert!(witnesses.contains(&objects[0].card));
                }
                checked(&mut g, a);
                assert_eq!(g.state.announcing.as_ref().unwrap().item.card, spell);
            }
            pay(&mut g);
            settle(&mut g);
            assert_eq!(g.object_view(spell).zone, Zone::Battlefield);
            assert!(
                drums
                    .iter()
                    .chain(&witnesses)
                    .all(|&id| g.object_view(id).tapped)
            );
        }
    }
}
#[test]
fn ecl_green_firdoch_changeling_any_mana_paid_animation_and_cleanup() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let core = named(&mut g, p, "Firdoch Core", Zone::Hand);
        assert!(g.state.objects[core as usize].card.has_subtype("Elf"));
        cast(&mut g, core, None, &[]);
        settle(&mut g);
        assert!(!g.is_creature(core));
        assert!(g.has_subtype(core, "Giant"));
        let mana = g.state.players[p.index()].mana.iter().sum::<u32>();
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: core,
                ability: 1,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut g);
        settle(&mut g);
        assert_eq!(
            g.state.players[p.index()].mana.iter().sum::<u32>(),
            mana - 4
        );
        assert!(g.is_creature(core));
        assert_ne!(g.current_kinds(core) & 16, 0);
        assert_eq!(
            g.stats(core),
            Stats {
                power: 4,
                toughness: 4
            }
        );
        assert!(g.has_subtype(core, "Elf") && g.has_subtype(core, "Shapeshifter"));
        assert!(
            !g.legal_actions()
                .iter()
                .any(|a| matches!(a, Action::ActivateMana { card, .. } if *card == core))
        );
        super::ability_timeline_tests::next_turn(&mut g);
        assert!(!g.is_creature(core));
        assert!(g.has_subtype(core, "Elf"));
        g.state.objects[core as usize].summoning_sick = false;
        if g.actor() != Some(p) { checked(&mut g, Action::Pass); }
        let a = g.legal_actions().into_iter().find(|a| matches!(a, Action::ActivateMana { card, production, .. } if *card == core && production[2] == 1)).unwrap();
        checked(&mut g, a);
        assert!(g.object_view(core).tapped);
    }
}
