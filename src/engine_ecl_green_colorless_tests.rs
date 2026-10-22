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
        if g.actor() != Some(p) {
            checked(&mut g, Action::Pass);
        }
        let a = g.legal_actions().into_iter().find(|a| matches!(a, Action::ActivateMana { card, production, .. } if *card == core && production[2] == 1)).unwrap();
        checked(&mut g, a);
        assert!(g.object_view(core).tapped);
    }
}
fn choose_trigger_target(g: &mut Game, target: CardId) {
    g.prepare_triggers();
    for _ in 0..40 {
        if g.legal_actions().contains(&Action::ChooseTarget {
            target: Target::Card(target),
        }) {
            return;
        }
        let a = if g.state.pending == Pending::Priority {
            Action::Pass
        } else {
            g.legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::PutTrigger { .. } | Action::FinishTargets))
                .unwrap()
        };
        checked(g, a);
    }
    panic!("target choice never offered");
}
#[test]
fn ecl_green_brawler_paid_attack_elf_target_and_live_block_requirement() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let brawler = named(&mut g, p, "Vinebred Brawler", Zone::Hand);
        cast(&mut g, brawler, None, &[]);
        settle(&mut g);
        let elf = named(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let foe = named(&mut g, p.other(), "Llanowar Elves", Zone::Battlefield);
        g.state.objects[brawler as usize].summoning_sick = false;
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        checked(&mut g, Action::Attack { card: brawler });
        checked(&mut g, Action::Attack { card: bear });
        checked(&mut g, Action::FinishAttackers);
        choose_trigger_target(&mut g, elf);
        for invalid in [brawler, bear, foe] {
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(invalid),
                },
            );
        }
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(elf),
            },
        );
        settle(&mut g);
        assert_eq!(
            g.stats(elf),
            Stats {
                power: 3,
                toughness: 2
            }
        );
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        assert_eq!(g.block_requirement_weight(brawler), 1);
        reject(&mut g, Action::FinishBlockers);
        reject(
            &mut g,
            Action::Block {
                blocker: foe,
                attacker: bear,
            },
        );
        checked(
            &mut g,
            Action::Block {
                blocker: foe,
                attacker: brawler,
            },
        );
        checked(&mut g, Action::FinishBlockers);
    }
}
#[test]
fn ecl_green_brawler_losing_ability_removes_requirement() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let brawler = named(&mut g, p, "Vinebred Brawler", Zone::Battlefield);
        let aura = named(&mut g, p, "Witness Protection", Zone::Hand);
        cast(&mut g, aura, None, &[Target::Card(brawler)]);
        settle(&mut g);
        assert_eq!(g.block_requirement_weight(brawler), 0);
        let foe = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        checked(&mut g, Action::Attack { card: brawler });
        checked(&mut g, Action::FinishAttackers);
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        assert!(g.legal_actions().contains(&Action::FinishBlockers));
        let _ = foe;
    }
}
#[test]
fn ecl_green_pummeler_gain_uses_live_giant_power_and_entry_controller() {
    for p in [Player::P0, Player::P1] {
        for mode in 0..4 {
            let mut g = setup(p);
            named(&mut g, p.other(), "Bygone Colossus", Zone::Battlefield);
            if mode >= 2 {
                let giant = named(&mut g, p, "Skyraker Giant", Zone::Battlefield);
                g.state.objects[giant as usize].growth.power = 3;
            }
            let spell = named(&mut g, p, "Pummeler for Hire", Zone::Hand);
            cast(&mut g, spell, None, &[]);
            checked(&mut g, Action::Pass);
            checked(&mut g, Action::Pass);
            if mode == 1 || mode == 3 {
                g.move_object(spell, Zone::Exile);
            }
            let life = g.state.players[p.index()].life;
            settle(&mut g);
            assert_eq!(
                g.state.players[p.index()].life - life,
                if mode >= 2 {
                    7
                } else if mode == 1 {
                    0
                } else {
                    4
                }
            );
            if mode != 1 && mode != 3 {
                assert!(
                    g.keywords(spell)
                        .contains(Keywords::REACH.union(Keywords::VIGILANCE))
                );
            }
        }
    }
}
#[test]
fn ecl_green_charge_draw_uses_creature_history_at_resolution_and_fizzles() {
    for p in [Player::P0, Player::P1] {
        for mode in 0..6 {
            let mut g = setup(p);
            let target = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let entrant = named(
                &mut g,
                if mode == 2 { p.other() } else { p },
                "Bear Cub",
                Zone::Hand,
            );
            if mode != 0 && mode != 4 {
                g.move_object(entrant, Zone::Battlefield);
            }
            if mode == 3 {
                g.move_object(entrant, Zone::Graveyard);
            }
            let spell = named(&mut g, p, "Thoughtweft Charge", Zone::Hand);
            cast(&mut g, spell, None, &[Target::Card(target)]);
            let hand = g
                .state
                .objects
                .iter()
                .filter(|o| o.owner == p && o.zone == Zone::Hand)
                .count();
            if mode == 4 {
                g.move_object(entrant, Zone::Battlefield);
            }
            if mode == 5 {
                g.move_object(target, Zone::Exile);
            }
            settle(&mut g);
            let after = g
                .state
                .objects
                .iter()
                .filter(|o| o.owner == p && o.zone == Zone::Hand)
                .count();
            assert_eq!(
                after,
                hand + usize::from([1, 3, 4].contains(&mode)) - usize::from(mode == 4)
            );
            if mode != 5 {
                assert_eq!(
                    g.stats(target),
                    Stats {
                        power: 5,
                        toughness: 5
                    }
                );
            }
        }
    }
}
#[test]
fn ecl_green_outrider_excludes_current_incarnation_and_tracks_departed_other_entries() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let outrider = named(&mut g, p, "Bristlebane Outrider", Zone::Hand);
        cast(&mut g, outrider, None, &[]);
        settle(&mut g);
        assert_eq!(
            g.stats(outrider),
            Stats {
                power: 3,
                toughness: 5
            }
        );
        assert!(
            g.keywords(outrider)
                .contains(Keywords::NO_POWER_LE_TWO_BLOCKERS)
        );
        let foe = named(&mut g, p.other(), "Bear Cub", Zone::Hand);
        g.move_object(foe, Zone::Battlefield);
        assert_eq!(g.stats(outrider).power, 3);
        let own = named(&mut g, p, "Bear Cub", Zone::Hand);
        cast(&mut g, own, None, &[]);
        settle(&mut g);
        assert_eq!(g.stats(outrider).power, 5);
        g.move_object(own, Zone::Graveyard);
        assert_eq!(g.stats(outrider).power, 5);
        super::ability_timeline_tests::next_turn(&mut g);
        assert_eq!(g.stats(outrider).power, 3);
        g.move_object(outrider, Zone::Exile);
        g.move_object(outrider, Zone::Battlefield);
        assert_eq!(g.stats(outrider).power, 3);
        g.move_object(outrider, Zone::Exile);
        g.move_object(outrider, Zone::Battlefield);
        assert_eq!(g.stats(outrider).power, 5);
    }
}
#[test]
fn ecl_green_bloom_tender_adds_each_present_color_once_and_ignores_other_zones_and_opponent() {
    for p in [Player::P0, Player::P1] {
        for all_colors in [false, true] {
            let mut g = setup(p);
            let tender = named(&mut g, p, "Bloom Tender", Zone::Hand);
            cast(&mut g, tender, None, &[]);
            settle(&mut g);
            assert!(
                !g.legal_actions()
                    .iter()
                    .any(|a| matches!(a, Action::ActivateMana { card, .. } if *card == tender))
            );
            g.state.objects[tender as usize].summoning_sick = false;
            named(&mut g, p.other(), "Savannah Lions", Zone::Battlefield);
            named(&mut g, p, "Highborn Vampire", Zone::Hand);
            named(&mut g, p, "Forest", Zone::Battlefield);
            if all_colors {
                for name in [
                    "Savannah Lions",
                    "Aegis Turtle",
                    "Highborn Vampire",
                    "Swab Goblin",
                    "Bear Cub",
                    "Swab Goblin",
                ] {
                    let id = named(&mut g, p, name, Zone::Battlefield);
                    g.state.objects[id as usize].tapped = true;
                }
            }
            let before = g.state.players[p.index()].mana;
            let production = if all_colors {
                [1, 1, 1, 1, 1, 0]
            } else {
                [0, 0, 0, 0, 1, 0]
            };
            checked(
                &mut g,
                Action::ActivateMana {
                    card: tender,
                    ability: 0,
                    payment: [0; 6],
                    counter_payment: Counters::default(),
                    production,
                },
            );
            assert!(g.state.stack.is_empty());
            assert_eq!(
                g.state.players[p.index()].mana,
                std::array::from_fn(|i| before[i] + production[i])
            );
        }
    }
}
#[test]
fn ecl_green_bloom_tender_funds_actual_guided_cast_with_nonmana_color_witnesses() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let tender = named(&mut g, p, "Bloom Tender", Zone::Battlefield);
        named(&mut g, p, "Aegis Turtle", Zone::Battlefield);
        let spell = named(&mut g, p, "Negate", Zone::Hand);
        let target = named(&mut g, p, "Giant Growth", Zone::Hand);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        cast(&mut g, target, None, &[Target::Card(bear)]);
        g.state.players[p.index()].mana = [0; 6];
        checked(&mut g, Action::AnnounceCast { card: spell });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Spell(target),
            },
        );
        let action = g.legal_actions().into_iter().find(|a| matches!(a, Action::ActivateMana { card, production, .. } if *card == tender && *production == [0, 1, 0, 0, 1, 0])).unwrap();
        checked(&mut g, action);
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(target).zone, Zone::Graveyard);
        assert_eq!(g.object_view(bear).damage, 0);
        assert_eq!(g.stats(bear).power, 2);
    }
}
fn definition_objects(g: &Game, p: Player, name: &str) -> Vec<CardId> {
    let c = Card::from_name(name).unwrap();
    g.state
        .objects
        .iter()
        .enumerate()
        .filter(|(_, o)| o.controller == p && o.zone == Zone::Battlefield && o.card == c)
        .map(|(id, _)| id as CardId)
        .collect()
}
#[test]
fn ecl_green_mutable_explorer_creates_tapped_land_with_paid_temporary_all_types_animation() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let explorer = named(&mut g, p, "Mutable Explorer", Zone::Hand);
        assert!(g.state.objects[explorer as usize].card.has_subtype("Giant"));
        cast(&mut g, explorer, None, &[]);
        settle(&mut g);
        assert_eq!(
            g.stats(explorer),
            Stats {
                power: 1,
                toughness: 1
            }
        );
        let tokens = definition_objects(&g, p, "@token/mutavault");
        assert_eq!(tokens.len(), 1);
        let land = tokens[0];
        assert_eq!(g.object_view(land).name(), "Mutavault");
        assert!(g.object_view(land).tapped);
        assert!(!g.is_creature(land));
        assert_eq!(g.current_kinds(land) & 2, 2);
        assert!(!g.has_subtype(land, "Elf"));
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: land,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut g);
        settle(&mut g);
        assert_eq!(
            g.stats(land),
            Stats {
                power: 2,
                toughness: 2
            }
        );
        assert!(
            g.is_creature(land) && g.has_subtype(land, "Elf") && g.has_subtype(land, "Treefolk")
        );
        assert_eq!(g.current_kinds(land) & 2, 2);
        super::ability_timeline_tests::next_turn(&mut g);
        assert!(!g.is_creature(land));
        assert!(!g.has_subtype(land, "Treefolk"));
        g.state.objects[land as usize].tapped = false;
        if g.actor() != Some(p) {
            checked(&mut g, Action::Pass);
        }
        let before = g.state.players[p.index()].mana[5];
        checked(&mut g, Action::TapMana { card: land });
        assert_eq!(g.state.players[p.index()].mana[5], before + 1);
    }
}
#[test]
fn ecl_green_tend_search_is_private_and_union_threshold_counts_each_permanent_once() {
    for p in [Player::P0, Player::P1] {
        for forests in [4, 5] {
            let mut g = setup(p);
            for _ in 0..forests {
                named(&mut g, p, "Forest", Zone::Battlefield);
            }
            let land = named(&mut g, p, "@token/mutavault", Zone::Battlefield);
            checked(
                &mut g,
                Action::AnnounceActivate {
                    card: land,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            pay(&mut g);
            settle(&mut g);
            let basic = named(&mut g, p, "Swamp", Zone::Library);
            let wrong = named(&mut g, p, "Bear Cub", Zone::Library);
            let spell = named(&mut g, p, "Tend the Sprigs", Zone::Hand);
            cast(&mut g, spell, None, &[]);
            checked(&mut g, Action::Pass);
            checked(&mut g, Action::Pass);
            assert!(g.observe(p).search_choice.is_some());
            assert!(g.observe(p.other()).search_choice.is_none());
            reject(&mut g, Action::ChooseSearch { card: wrong });
            checked(&mut g, Action::ChooseSearch { card: basic });
            checked(&mut g, Action::FinishSearch);
            settle(&mut g);
            assert_eq!(g.object_view(basic).zone, Zone::Battlefield);
            assert!(g.object_view(basic).tapped);
            let tokens = definition_objects(&g, p, "@token/treefolk_3_4_green_reach");
            assert_eq!(tokens.len(), usize::from(forests == 5));
            if let Some(&token) = tokens.first() {
                assert_eq!(
                    g.stats(token),
                    Stats {
                        power: 3,
                        toughness: 4
                    }
                );
                assert!(g.keywords(token).contains(Keywords::REACH));
                assert!(!g.object_view(token).tapped);
                assert_eq!(g.object_view(token).colors, 16);
            }
        }
    }
}
fn equip(g: &mut Game, source: CardId, target: CardId) {
    checked(
        g,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    checked(
        g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    pay(g);
    settle(g);
}
#[test]
fn ecl_green_stalactite_creates_changeling_and_paid_equipment_changes_live_types() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let dagger = named(&mut g, p, "Stalactite Dagger", Zone::Hand);
        cast(&mut g, dagger, None, &[]);
        settle(&mut g);
        let tokens = definition_objects(&g, p, "@token/shapeshifter_1_1_colorless_changeling");
        assert_eq!(tokens.len(), 1);
        let token = tokens[0];
        assert_eq!(
            g.stats(token),
            Stats {
                power: 1,
                toughness: 1
            }
        );
        assert!(g.has_subtype(token, "Elf") && g.has_subtype(token, "Treefolk"));
        assert!(g.keywords(token).contains(Keywords::CHANGELING));
        assert_eq!(g.object_view(token).colors, 0);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let other = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let mana = g.state.players[p.index()].mana.iter().sum::<u32>();
        equip(&mut g, dagger, bear);
        assert_eq!(
            g.state.players[p.index()].mana.iter().sum::<u32>(),
            mana - 2
        );
        assert_eq!(
            g.stats(bear),
            Stats {
                power: 3,
                toughness: 3
            }
        );
        assert!(g.has_subtype(bear, "Elf") && g.has_subtype(bear, "Giant"));
        assert!(!g.keywords(bear).contains(Keywords::CHANGELING));
        equip(&mut g, dagger, other);
        assert_eq!(
            g.stats(bear),
            Stats {
                power: 2,
                toughness: 2
            }
        );
        assert!(!g.has_subtype(bear, "Elf"));
        assert_eq!(
            g.stats(other),
            Stats {
                power: 3,
                toughness: 3
            }
        );
        assert!(g.has_subtype(other, "Treefolk"));
        g.move_object(dagger, Zone::Graveyard);
        assert_eq!(
            g.stats(other),
            Stats {
                power: 2,
                toughness: 2
            }
        );
        assert!(!g.has_subtype(other, "Elf"));
    }
}
#[test]
fn ecl_green_stalactite_type_effect_starts_before_source_ability_removal() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let dagger = named(&mut g, p, "Stalactite Dagger", Zone::Battlefield);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        equip(&mut g, dagger, bear);
        let aura = named(&mut g, p, "Stop Cold", Zone::Hand);
        cast(&mut g, aura, None, &[Target::Card(dagger)]);
        settle(&mut g);
        assert!(g.has_subtype(bear, "Elf"));
        assert_eq!(
            g.stats(bear),
            Stats {
                power: 3,
                toughness: 3
            }
        );
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: dagger,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
    }
}
#[test]
fn ecl_green_sapling_affinity_landfall_and_exile_cost_freeze_filtered_indestructible() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let mut forests = Vec::new();
        for _ in 0..6 {
            forests.push(named(&mut g, p, "Forest", Zone::Battlefield));
        }
        let nursery = named(&mut g, p, "Sapling Nursery", Zone::Hand);
        let mana = g.state.players[p.index()].mana.iter().sum::<u32>();
        cast(&mut g, nursery, None, &[]);
        settle(&mut g);
        assert_eq!(
            g.state.players[p.index()].mana.iter().sum::<u32>(),
            mana - 2
        );
        let enemy_forest = named(&mut g, p.other(), "Forest", Zone::Hand);
        g.move_object(enemy_forest, Zone::Battlefield);
        settle(&mut g);
        assert!(definition_objects(&g, p, "@token/treefolk_3_4_green_reach").is_empty());
        let own = named(&mut g, p, "Forest", Zone::Hand);
        checked(&mut g, Action::PlayLand { card: own });
        settle(&mut g);
        let trees = definition_objects(&g, p, "@token/treefolk_3_4_green_reach");
        assert_eq!(trees.len(), 1);
        let core = named(&mut g, p, "Firdoch Core", Zone::Battlefield);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let mana = g.state.players[p.index()].mana.iter().sum::<u32>();
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: nursery,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut g);
        assert_eq!(g.object_view(nursery).zone, Zone::Exile);
        assert_eq!(
            g.state.players[p.index()].mana.iter().sum::<u32>(),
            mana - 2
        );
        settle(&mut g);
        for id in forests.into_iter().chain([own, trees[0], core]) {
            assert!(g.keywords(id).contains(Keywords::INDESTRUCTIBLE));
        }
        assert!(!g.keywords(enemy_forest).contains(Keywords::INDESTRUCTIBLE));
        assert!(!g.keywords(bear).contains(Keywords::INDESTRUCTIBLE));
        let late = named(&mut g, p, "Magnigoth Sentry", Zone::Battlefield);
        assert!(!g.keywords(late).contains(Keywords::INDESTRUCTIBLE));
        super::ability_timeline_tests::next_turn(&mut g);
        assert!(!g.keywords(trees[0]).contains(Keywords::INDESTRUCTIBLE));
        assert!(!g.keywords(core).contains(Keywords::INDESTRUCTIBLE));
    }
}
#[test]
fn ecl_green_speaker_optional_discard_tutor_reveal_and_source_departure() {
    for p in [Player::P0, Player::P1] {
        for (accept, leaves) in [(false, false), (true, false), (true, true)] {
            let mut g = setup(p);
            let discard = named(&mut g, p, "Forest", Zone::Hand);
            let wanted = named(&mut g, p, "Bear Cub", Zone::Library);
            let wrong = named(&mut g, p, "Negate", Zone::Library);
            let speaker = named(&mut g, p, "Formidable Speaker", Zone::Hand);
            cast(&mut g, speaker, None, &[]);
            for _ in 0..40 {
                g.prepare_triggers();
                if g.legal_actions().contains(&Action::DeclineEffectPayment) {
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
            assert!(g.observe(p.other()).effect_choice.is_none());
            if leaves {
                g.move_object(speaker, Zone::Exile);
            }
            if accept {
                checked(&mut g, Action::Discard { card: discard });
                assert!(g.observe(p).search_choice.is_some());
                assert!(g.observe(p.other()).search_choice.is_none());
                reject(&mut g, Action::ChooseSearch { card: wrong });
                checked(&mut g, Action::ChooseSearch { card: wanted });
                checked(&mut g, Action::FinishSearch);
                settle(&mut g);
                assert_eq!(g.object_view(discard).zone, Zone::Graveyard);
                assert_eq!(g.object_view(wanted).zone, Zone::Hand);
                assert!(
                    g.observe(p.other())
                        .known_hand
                        .iter()
                        .any(|object| object.card == wanted)
                );
            } else {
                checked(&mut g, Action::DeclineEffectPayment);
                settle(&mut g);
                assert_eq!(g.object_view(discard).zone, Zone::Hand);
                assert_eq!(g.object_view(wanted).zone, Zone::Library);
            }
        }
    }
}
#[test]
fn ecl_green_speaker_paid_tap_untaps_other_permanent_including_enemy_and_rejects_self() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let speaker = named(&mut g, p, "Formidable Speaker", Zone::Battlefield);
        let land = named(&mut g, p.other(), "Forest", Zone::Battlefield);
        g.state.objects[land as usize].tapped = true;
        let mana = g.state.players[p.index()].mana.iter().sum::<u32>();
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: speaker,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(speaker),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(land),
            },
        );
        pay(&mut g);
        assert!(g.object_view(speaker).tapped);
        assert_eq!(
            g.state.players[p.index()].mana.iter().sum::<u32>(),
            mana - 1
        );
        g.move_object(speaker, Zone::Exile);
        settle(&mut g);
        assert!(!g.object_view(land).tapped);
    }
}
fn choose_type(g: &mut Game, name: &str) {
    let kind = crate::cards::CreatureType::from_name(name).unwrap();
    checked(
        g,
        Action::ChooseEntryValue {
            value: EntryChoiceValue::CreatureType(kind),
        },
    );
}
fn enter_chronicle(g: &mut Game, p: Player, kind: &str) -> CardId {
    let source = named(g, p, "Chronicle of Victory", Zone::Hand);
    cast(g, source, None, &[]);
    checked(g, Action::Pass);
    checked(g, Action::Pass);
    choose_type(g, kind);
    settle(g);
    source
}
fn enter_realms(g: &mut Game, p: Player) -> CardId {
    let land = named(g, p, "Eclipsed Realms", Zone::Hand);
    checked(g, Action::PlayLand { card: land });
    let choices: Vec<_> = g
        .legal_actions()
        .into_iter()
        .filter(|a| matches!(a, Action::ChooseEntryValue { .. }))
        .collect();
    assert_eq!(choices.len(), 8);
    reject(
        g,
        Action::ChooseEntryValue {
            value: EntryChoiceValue::CreatureType(
                crate::cards::CreatureType::from_name("Human").unwrap(),
            ),
        },
    );
    choose_type(g, "Elf");
    land
}
fn restricted_realms_mana(g: &mut Game, land: CardId, color: usize) {
    let mut production = [0; 6];
    production[color] = 1;
    checked(
        g,
        Action::ActivateMana {
            card: land,
            ability: 1,
            payment: [0; 6],
            counter_payment: Counters::default(),
            production,
        },
    );
}
#[test]
fn ecl_green_realms_constrained_choice_and_mana_restriction_survive_source_departure() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let land = enter_realms(&mut g, p);
        g.state.players[p.index()].mana = [0; 6];
        restricted_realms_mana(&mut g, land, 4);
        let wrong = named(&mut g, p, "Giant Growth", Zone::Hand);
        named(&mut g, p, "Bear Cub", Zone::Battlefield);
        reject(&mut g, Action::AnnounceCast { card: wrong });
        g.move_object(land, Zone::Exile);
        let elf = named(&mut g, p, "Llanowar Elves", Zone::Hand);
        cast(&mut g, elf, None, &[]);
        settle(&mut g);
        assert_eq!(g.object_view(elf).zone, Zone::Battlefield);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}
#[test]
fn ecl_green_realms_mana_pays_kindred_noncreature_spells_and_their_activations() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let land = enter_realms(&mut g, p);
        g.state.players[p.index()].mana = [0; 6];
        restricted_realms_mana(&mut g, land, 1);
        g.state.players[p.index()].mana[5] = 2;
        let core = named(&mut g, p, "Firdoch Core", Zone::Hand);
        cast(&mut g, core, None, &[]);
        settle(&mut g);
        assert!(!g.is_creature(core));
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        g.state.objects[land as usize].tapped = false;
        restricted_realms_mana(&mut g, land, 3);
        g.state.players[p.index()].mana[5] = 3;
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
        assert!(g.is_creature(core));
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        g.state.objects[land as usize].tapped = false;
        checked(
            &mut g,
            Action::ActivateMana {
                card: land,
                ability: 0,
                payment: [0; 6],
                counter_payment: Counters::default(),
                production: [0, 0, 0, 0, 0, 1],
            },
        );
        assert_eq!(g.state.players[p.index()].mana[5], 1);
    }
}
#[test]
fn ecl_green_chronicle_chosen_type_lord_keywords_control_and_source_removal() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let own = named(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Llanowar Elves", Zone::Battlefield);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let source = enter_chronicle(&mut g, p, "Elf");
        assert_eq!(
            g.stats(own),
            Stats {
                power: 3,
                toughness: 3
            }
        );
        assert!(
            g.keywords(own)
                .contains(Keywords::FIRST_STRIKE.union(Keywords::TRAMPLE))
        );
        assert_eq!(g.stats(enemy).power, 1);
        assert_eq!(g.stats(bear).power, 2);
        assert!(!g.keywords(enemy).contains(Keywords::TRAMPLE));
        let core = named(&mut g, p, "Firdoch Core", Zone::Battlefield);
        assert!(!g.is_creature(core));
        assert!(!g.keywords(core).contains(Keywords::FIRST_STRIKE));
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
            g.stats(core),
            Stats {
                power: 6,
                toughness: 6
            }
        );
        assert!(g.keywords(core).contains(Keywords::FIRST_STRIKE));
        g.state.objects[source as usize].controller = p.other();
        assert_eq!(g.stats(own).power, 1);
        assert_eq!(g.stats(enemy).power, 3);
        assert!(!g.keywords(own).contains(Keywords::TRAMPLE));
        assert!(g.keywords(enemy).contains(Keywords::TRAMPLE));
        g.move_object(source, Zone::Exile);
        assert_eq!(g.stats(enemy).power, 1);
        assert!(!g.keywords(enemy).contains(Keywords::TRAMPLE));
    }
}
#[test]
fn ecl_green_chronicle_cast_trigger_counts_kindred_spells_and_survives_source_departure() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = enter_chronicle(&mut g, p, "Elf");
        let before = g.state.players[p.index()].cards_drawn_this_turn;
        let wrong = named(&mut g, p, "Bear Cub", Zone::Hand);
        cast(&mut g, wrong, None, &[]);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, before);
        let elf = named(&mut g, p, "Llanowar Elves", Zone::Hand);
        cast(&mut g, elf, None, &[]);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, before + 1);
        let core = named(&mut g, p, "Firdoch Core", Zone::Hand);
        cast(&mut g, core, None, &[]);
        g.move_object(source, Zone::Exile);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, before + 2);
    }
}
#[test]
fn ecl_green_chronicle_source_ability_removal_stops_lord_and_cast_trigger() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = enter_chronicle(&mut g, p, "Elf");
        let elf = named(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        let aura = named(&mut g, p, "Stop Cold", Zone::Hand);
        cast(&mut g, aura, None, &[Target::Card(source)]);
        settle(&mut g);
        assert_eq!(g.stats(elf).power, 1);
        assert!(!g.keywords(elf).contains(Keywords::TRAMPLE));
        let before = g.state.players[p.index()].cards_drawn_this_turn;
        let next = named(&mut g, p, "Llanowar Elves", Zone::Hand);
        cast(&mut g, next, None, &[]);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, before);
    }
}
fn enter_pennant(g: &mut Game, p: Player) -> CardId {
    let source = named(g, p, "Dawn-Blessed Pennant", Zone::Hand);
    cast(g, source, None, &[]);
    checked(g, Action::Pass);
    checked(g, Action::Pass);
    assert_eq!(
        g.legal_actions()
            .iter()
            .filter(|a| matches!(a, Action::ChooseEntryValue { .. }))
            .count(),
        8
    );
    choose_type(g, "Elf");
    settle(g);
    source
}
#[test]
fn ecl_green_pennant_entry_trigger_counts_chosen_permanents_and_survives_departure() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = enter_pennant(&mut g, p);
        let life = g.state.players[p.index()].life;
        let wrong = named(&mut g, p, "Bear Cub", Zone::Hand);
        cast(&mut g, wrong, None, &[]);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, life);
        let enemy = named(&mut g, p.other(), "Llanowar Elves", Zone::Hand);
        g.move_object(enemy, Zone::Battlefield);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, life);
        let elf = named(&mut g, p, "Llanowar Elves", Zone::Hand);
        cast(&mut g, elf, None, &[]);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, life + 1);
        let core = named(&mut g, p, "Firdoch Core", Zone::Hand);
        cast(&mut g, core, None, &[]);
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        g.move_object(source, Zone::Exile);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, life + 2);
    }
}
#[test]
fn ecl_green_pennant_sacrifice_return_uses_frozen_chosen_type_after_reentry() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = enter_pennant(&mut g, p);
        let core = named(&mut g, p, "Firdoch Core", Zone::Graveyard);
        let wrong = named(&mut g, p, "Bear Cub", Zone::Graveyard);
        let enemy = named(&mut g, p.other(), "Llanowar Elves", Zone::Graveyard);
        let hand = named(&mut g, p, "Llanowar Elves", Zone::Hand);
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: source,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        for invalid in [wrong, enemy, hand, source] {
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(invalid),
                },
            );
        }
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(core),
            },
        );
        let mana = g.state.players[p.index()].mana.iter().sum::<u32>();
        pay(&mut g);
        assert_eq!(
            g.state.players[p.index()].mana.iter().sum::<u32>(),
            mana - 2
        );
        assert_eq!(g.object_view(source).zone, Zone::Graveyard);
        assert!(g.entry_value(g.object_ref(source)).is_none());
        assert_eq!(
            g.state
                .stack
                .last()
                .unwrap()
                .source_chosen_type
                .unwrap()
                .name(),
            "Elf"
        );
        let movement = g.entry_move(source, Zone::Battlefield, false);
        g.stage_entries(vec![movement], None);
        assert!(g.advance_entries());
        choose_type(&mut g, "Giant");
        settle(&mut g);
        assert_eq!(g.object_view(core).zone, Zone::Hand);
        assert_eq!(
            g.object_view(source).entry_choice,
            Some(EntryChoiceValue::CreatureType(
                crate::cards::CreatureType::from_name("Giant").unwrap()
            ))
        );
    }
}
#[test]
fn ecl_green_pennant_does_not_rebind_departed_graveyard_target() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = enter_pennant(&mut g, p);
        let elf = named(&mut g, p, "Llanowar Elves", Zone::Graveyard);
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: source,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(elf),
            },
        );
        pay(&mut g);
        g.move_object(elf, Zone::Hand);
        g.move_object(elf, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.object_view(elf).zone, Zone::Graveyard);
    }
}

#[test]
fn ecl_green_pitiless_fists_optional_fight_tracks_exact_attachment_and_current_power() {
    for p in [Player::P0, Player::P1] {
        for mode in 0..7 {
            let mut g = setup(p);
            let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let other = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let foe = named(&mut g, p.other(), "Aegis Turtle", Zone::Battlefield);
            let aura = named(&mut g, p, "Pitiless Fists", Zone::Hand);
            checked(&mut g, Action::AnnounceCast { card: aura });
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(foe),
                },
            );
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(bear),
                },
            );
            pay(&mut g);
            checked(&mut g, Action::Pass);
            checked(&mut g, Action::Pass);
            assert_eq!(
                g.stats(bear),
                Stats {
                    power: 4,
                    toughness: 4
                }
            );
            if mode == 0 {
                checked(&mut g, Action::FinishTargets);
            } else {
                reject(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(other),
                    },
                );
                checked(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(foe),
                    },
                );
            }
            match mode {
                2 => {
                    g.move_object(aura, Zone::Graveyard);
                }
                3 => {
                    g.move_object(bear, Zone::Exile);
                    g.move_object(bear, Zone::Battlefield);
                }
                4 => {
                    g.move_object(foe, Zone::Exile);
                    g.move_object(foe, Zone::Battlefield);
                }
                5 => {
                    g.move_object(aura, Zone::Graveyard);
                    let mut movement = g.entry_move(aura, Zone::Battlefield, false);
                    movement.attachment = Some(g.object_ref(other));
                    g.stage_entries(vec![movement], None);
                }
                6 => {
                    let growth = named(&mut g, p, "Giant Growth", Zone::Hand);
                    cast(&mut g, growth, None, &[Target::Card(bear)]);
                }
                _ => {}
            }
            settle(&mut g);
            assert_eq!(g.object_view(bear).damage, 0);
            if mode == 6 {
                assert_eq!(g.object_view(foe).zone, Zone::Graveyard);
                assert_eq!(g.stats(bear).power, 7);
            } else {
                assert_eq!(g.object_view(foe).zone, Zone::Battlefield);
                assert_eq!(
                    g.object_view(foe).damage,
                    match mode {
                        1 => 4,
                        2 | 5 => 2,
                        _ => 0,
                    }
                );
            }
            if mode == 5 {
                assert_eq!(g.stats(other).power, 4);
                assert_eq!(g.object_view(other).damage, 0);
            }
        }
    }
}
#[test]
fn ecl_green_pitiless_fists_deals_both_directions_and_own_control_is_continuous() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let foe = named(&mut g, p.other(), "Swab Goblin", Zone::Battlefield);
        let aura = named(&mut g, p, "Pitiless Fists", Zone::Hand);
        cast(&mut g, aura, None, &[Target::Card(bear)]);
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(foe),
            },
        );
        settle(&mut g);
        assert_eq!(g.object_view(foe).zone, Zone::Graveyard);
        assert_eq!(g.object_view(bear).damage, 2);
        assert_eq!(g.stats(bear).toughness, 4);
        // A fixed control effect makes the enchant restriction fail at the next SBA.
        // Use a paid control spell rather than mutating the attachment's controller.
        let steal = named(&mut g, p, "Harmless Offering", Zone::Hand);
        cast(
            &mut g,
            steal,
            None,
            &[Target::Player(p.other()), Target::Card(bear)],
        );
        settle(&mut g);
        assert_eq!(g.object_view(aura).zone, Zone::Graveyard);
        assert_eq!(g.object_view(bear).zone, Zone::Graveyard);
    }
}

#[test]
fn ecl_green_spry_exact_two_public_nontarget_choices_freeze_absolute_power_difference() {
    for p in [Player::P0, Player::P1] {
        for mode in 0..4 {
            let mut g = setup(p);
            let first = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let second = named(
                &mut g,
                p,
                if mode == 1 {
                    "Bear Cub"
                } else {
                    "Aegis Turtle"
                },
                Zone::Battlefield,
            );
            let foe = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let land = named(&mut g, p, "Forest", Zone::Battlefield);
            if mode == 2 {
                g.state.objects[first as usize].growth.power = -4;
            }
            if mode == 3 {
                g.grant_keyword_effects(
                    &[first, second],
                    g.object_ref(first),
                    Keywords::PROTECTION_EVERYTHING.union(Keywords::HEXPROOF),
                );
            }
            let initial = [g.stats(first), g.stats(second)];
            let x = initial[0].power.abs_diff(initial[1].power);
            let spell = named(&mut g, p, "Spry and Mighty", Zone::Hand);
            let mana = g.state.players[p.index()].mana.iter().sum::<u32>();
            cast(&mut g, spell, None, &[]);
            assert_eq!(
                g.state.players[p.index()].mana.iter().sum::<u32>(),
                mana - 5
            );
            let hand = g
                .state
                .objects
                .iter()
                .filter(|o| o.owner == p && o.zone == Zone::Hand)
                .count();
            checked(&mut g, Action::Pass);
            checked(&mut g, Action::Pass);
            for id in [foe, land] {
                let object = g.object_ref(id);
                reject(&mut g, Action::ChooseCreature { object });
            }
            reject(&mut g, Action::DeclineEffectPayment);
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(first),
                },
            );
            for viewer in [p, p.other()] {
                let view = g.effect_choice_view(viewer).unwrap();
                assert!(view.candidates.contains(&g.object_ref(first)));
                assert!(view.candidates.contains(&g.object_ref(second)));
                assert!(!view.candidates.contains(&g.object_ref(foe)));
            }
            let reference = g.object_ref(first);
            checked(&mut g, Action::ChooseCreature { object: reference });
            reject(&mut g, Action::ChooseCreature { object: reference });
            assert_eq!(g.stats(first), initial[0]);
            assert_eq!(
                g.state
                    .objects
                    .iter()
                    .filter(|o| o.owner == p && o.zone == Zone::Hand)
                    .count(),
                hand
            );
            let reference = g.object_ref(second);
            checked(&mut g, Action::ChooseCreature { object: reference });
            settle(&mut g);
            assert_eq!(
                g.state
                    .objects
                    .iter()
                    .filter(|o| o.owner == p && o.zone == Zone::Hand)
                    .count(),
                hand + usize::try_from(x).unwrap()
            );
            for (id, before) in [first, second].into_iter().zip(initial) {
                let x = i128::try_from(x).unwrap();
                assert_eq!(
                    g.stats(id),
                    Stats {
                        power: before.power + x,
                        toughness: before.toughness + x
                    }
                );
                assert!(g.keywords(id).contains(Keywords::TRAMPLE));
            }
            assert!(!g.keywords(foe).contains(Keywords::TRAMPLE));
            let later = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            assert_eq!(g.stats(later).power, 2);
            assert!(!g.keywords(later).contains(Keywords::TRAMPLE));
            super::ability_timeline_tests::next_turn(&mut g);
            for id in [first, second] {
                assert!(!g.keywords(id).contains(Keywords::TRAMPLE));
            }
        }
    }
}
#[test]
fn ecl_green_spry_insufficient_creatures_does_nothing_and_failed_draw_still_boosts() {
    for p in [Player::P0, Player::P1] {
        for count in 0..=2 {
            let mut g = setup(p);
            let mut creatures = Vec::new();
            for i in 0..count {
                creatures.push(named(
                    &mut g,
                    p,
                    if i == 0 { "Bear Cub" } else { "Aegis Turtle" },
                    Zone::Battlefield,
                ));
            }
            if count == 2 {
                let library = g.state.players[p.index()].library.clone();
                for id in library {
                    g.move_object(id, Zone::Graveyard);
                }
            }
            let spell = named(&mut g, p, "Spry and Mighty", Zone::Hand);
            cast(&mut g, spell, None, &[]);
            checked(&mut g, Action::Pass);
            checked(&mut g, Action::Pass);
            if count < 2 {
                assert_eq!(g.state.pending, Pending::Priority);
                assert!(g.state.resolving.is_none());
                assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
                for id in creatures {
                    assert!(!g.keywords(id).contains(Keywords::TRAMPLE));
                }
            } else {
                for id in creatures.iter().copied() {
                    let object = g.object_ref(id);
                    checked(&mut g, Action::ChooseCreature { object });
                }
                assert_eq!(g.stats(creatures[0]).power, 4);
                assert_eq!(g.stats(creatures[1]).power, 2);
                assert!(g.keywords(creatures[0]).contains(Keywords::TRAMPLE));
                assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
            }
        }
    }
}

fn enter_stone(g: &mut Game, p: Player) -> CardId {
    let stone = named(g, p, "Gathering Stone", Zone::Hand);
    cast(g, stone, None, &[]);
    checked(g, Action::Pass);
    checked(g, Action::Pass);
    choose_type(g, "Elf");
    stone
}
#[test]
fn ecl_green_gathering_private_look_reveal_optional_graveyard_or_unchanged_top() {
    for p in [Player::P0, Player::P1] {
        for mode in 0..5 {
            let mut g = setup(p);
            let below = named(&mut g, p, "Forest", Zone::Library);
            let top = named(
                &mut g,
                p,
                if mode < 3 { "Firdoch Core" } else { "Bear Cub" },
                Zone::Library,
            );
            let original = g.state.players[p.index()].library.clone();
            enter_stone(&mut g, p);
            checked(&mut g, Action::Pass);
            checked(&mut g, Action::Pass);
            let view = g.observe(p).library_selection.unwrap();
            assert_eq!(view.remaining.len(), 1);
            assert_eq!(view.remaining[0].id, top);
            assert!(g.observe(p.other()).library_selection.is_none());
            reject(&mut g, Action::MillLookedCard { card: below });
            if mode == 0 {
                reject(&mut g, Action::MillLookedCard { card: top });
                checked(&mut g, Action::SelectLibrary { card: top });
                checked(&mut g, Action::FinishLibrarySelection);
                assert_eq!(g.object_view(top).zone, Zone::Hand);
                assert!(g.state.known_hand.contains(&g.object_ref(top)));
            } else {
                if mode < 3 {
                    checked(&mut g, Action::FinishLibrarySelection);
                } else {
                    reject(&mut g, Action::SelectLibrary { card: top });
                    checked(&mut g, Action::FinishLibrarySelection);
                }
                assert!(g.observe(p).library_selection.unwrap().ordering);
                assert!(g.observe(p.other()).library_selection.is_none());
                if mode == 1 || mode == 3 {
                    checked(&mut g, Action::MillLookedCard { card: top });
                    assert_eq!(g.object_view(top).zone, Zone::Graveyard);
                } else {
                    checked(&mut g, Action::FinishLibrarySelection);
                    assert_eq!(g.object_view(top).zone, Zone::Library);
                    assert_eq!(g.state.players[p.index()].library, original);
                }
            }
            settle(&mut g);
            assert_eq!(g.object_view(below).zone, Zone::Library);
        }
    }
}
#[test]
fn ecl_green_gathering_costs_reduce_kindred_noncreatures_and_follow_live_source_abilities_control()
{
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let stone = enter_stone(&mut g, p);
        settle(&mut g);
        let core = named(&mut g, p, "Firdoch Core", Zone::Hand);
        let bear = named(&mut g, p, "Bear Cub", Zone::Hand);
        let cavalry = named(&mut g, p, "Safewright Cavalry", Zone::Hand);
        let before = g.state.players[p.index()].mana.iter().sum::<u32>();
        cast(&mut g, core, None, &[]);
        assert_eq!(
            g.state.players[p.index()].mana.iter().sum::<u32>(),
            before - 2
        );
        settle(&mut g);
        assert_eq!(g.spell_reduction(p, bear), 0);
        assert_eq!(g.spell_reduction(p, cavalry), 1);
        let blank = named(&mut g, p, "Stop Cold", Zone::Hand);
        cast(&mut g, blank, None, &[Target::Card(stone)]);
        settle(&mut g);
        assert_eq!(g.spell_reduction(p, cavalry), 0);
        g.move_object(blank, Zone::Graveyard);
        assert_eq!(g.spell_reduction(p, cavalry), 1);
        let offering = named(&mut g, p, "Harmless Offering", Zone::Hand);
        cast(
            &mut g,
            offering,
            None,
            &[Target::Player(p.other()), Target::Card(stone)],
        );
        settle(&mut g);
        assert_eq!(g.object_view(stone).controller, p.other());
        let their = named(&mut g, p.other(), "Safewright Cavalry", Zone::Hand);
        assert_eq!(g.spell_reduction(p, cavalry), 0);
        assert_eq!(g.spell_reduction(p.other(), their), 1);
        g.move_object(stone, Zone::Graveyard);
        assert_eq!(g.spell_reduction(p.other(), their), 0);
    }
}
#[test]
fn ecl_green_gathering_trigger_choice_is_frozen_through_source_reentry_and_own_upkeep_only() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let top = named(&mut g, p, "Llanowar Elves", Zone::Library);
        let stone = enter_stone(&mut g, p);
        g.move_object(stone, Zone::Graveyard);
        let movement = g.entry_move(stone, Zone::Battlefield, false);
        g.stage_entries(vec![movement], None);
        assert!(g.advance_entries());
        choose_type(&mut g, "Giant");
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        // New Giant-choice trigger cannot select the Elf card.
        reject(&mut g, Action::SelectLibrary { card: top });
        checked(&mut g, Action::FinishLibrarySelection);
        assert!(g.observe(p).library_selection.unwrap().ordering);
        checked(&mut g, Action::FinishLibrarySelection);
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::SelectLibrary { card: top });
        checked(&mut g, Action::FinishLibrarySelection);
        settle(&mut g);
        assert_eq!(g.object_view(top).zone, Zone::Hand);
        super::ability_timeline_tests::next_turn(&mut g);
        assert_eq!(g.state.active, p.other());
        assert!(g.state.stack.is_empty());
        super::ability_timeline_tests::next_turn(&mut g);
        assert_eq!(g.state.active, p);
        assert_eq!(g.state.step, Step::Upkeep);
        assert_eq!(g.state.stack.len(), 1);
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        assert!(g.observe(p).library_selection.is_some());
        assert!(g.observe(p.other()).library_selection.is_none());
        settle(&mut g);
    }
}

#[test]
fn ecl_green_gathering_private_eligibility_preserves_public_phase_sequence() {
    for p in [Player::P0, Player::P1] {
        let mut games = [setup(p), setup(p)];
        for (g, name) in games.iter_mut().zip(["Firdoch Core", "Bear Cub"]) {
            named(g, p, name, Zone::Library);
            enter_stone(g, p);
            checked(g, Action::Pass);
            checked(g, Action::Pass);
        }
        for _ in 0..2 {
            assert_eq!(
                serde_json::to_value(games[0].observe(p.other())).unwrap(),
                serde_json::to_value(games[1].observe(p.other())).unwrap()
            );
            for g in &mut games {
                checked(g, Action::FinishLibrarySelection);
            }
        }
        assert_eq!(
            serde_json::to_value(games[0].observe(p.other())).unwrap(),
            serde_json::to_value(games[1].observe(p.other())).unwrap()
        );
    }
}

#[test]
fn ecl_green_prismatic_resolution_colors_private_basic_search_shuffle_and_extra_land() {
    for p in [Player::P0, Player::P1] {
        for mode in 0..4 {
            let mut g = setup(p);
            let source = named(&mut g, p, "Prismatic Undercurrents", Zone::Hand);
            if mode < 2 {
                named(&mut g, p, "Savannah Lions", Zone::Battlefield);
                named(&mut g, p, "Savannah Lions", Zone::Battlefield);
                named(&mut g, p, "Highborn Vampire", Zone::Battlefield);
            }
            if mode == 3 {
                named(&mut g, p, "Bear Cub", Zone::Battlefield);
            }
            let wrong = named(&mut g, p, "Bear Cub", Zone::Library);
            cast(&mut g, source, None, &[]);
            checked(&mut g, Action::Pass);
            checked(&mut g, Action::Pass);
            if mode == 1 || mode == 2 {
                g.move_object(source, Zone::Graveyard);
            }
            checked(&mut g, Action::Pass);
            checked(&mut g, Action::Pass);
            let expected = [3, 2, 0, 1][mode];
            assert_eq!(
                g.state
                    .resolving
                    .as_ref()
                    .unwrap()
                    .search
                    .as_ref()
                    .unwrap()
                    .spec
                    .maximum,
                expected
            );
            assert!(g.observe(p).search_choice.is_some());
            assert!(g.observe(p.other()).search_choice.is_none());
            reject(&mut g, Action::ChooseSearch { card: wrong });
            let before_rng = g.state.rng;
            let mut found = Vec::new();
            if mode < 2 {
                for _ in 0..expected {
                    let id = g.state.players[p.index()]
                        .library
                        .iter()
                        .copied()
                        .find(|id| !found.contains(id) && g.object_view(*id).card.is_basic_land())
                        .unwrap();
                    checked(&mut g, Action::ChooseSearch { card: id });
                    found.push(id);
                }
                let extra = g.state.players[p.index()]
                    .library
                    .iter()
                    .copied()
                    .find(|id| !found.contains(id) && g.object_view(*id).card.is_basic_land())
                    .unwrap();
                reject(&mut g, Action::ChooseSearch { card: extra });
            }
            checked(&mut g, Action::FinishSearch);
            assert_ne!(g.state.rng, before_rng);
            settle(&mut g);
            for id in found {
                assert_eq!(g.object_view(id).zone, Zone::Hand);
                assert!(g.state.known_hand.contains(&g.object_ref(id)));
            }
            if mode == 0 {
                let lands: Vec<_> = (0..3)
                    .map(|_| named(&mut g, p, "Forest", Zone::Hand))
                    .collect();
                checked(&mut g, Action::PlayLand { card: lands[0] });
                checked(&mut g, Action::PlayLand { card: lands[1] });
                reject(&mut g, Action::PlayLand { card: lands[2] });
            }
        }
    }
}
#[test]
fn ecl_green_aurora_public_reveal_stops_after_distinct_colors_and_nested_entry_choices_resume() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        named(&mut g, p, "Savannah Lions", Zone::Battlefield);
        named(&mut g, p, "Swab Goblin", Zone::Battlefield);
        named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let unseen = named(&mut g, p, "Opt", Zone::Library);
        let forest = named(&mut g, p, "Forest", Zone::Library);
        let stone = named(&mut g, p, "Gathering Stone", Zone::Library);
        let shock = named(&mut g, p, "Shock", Zone::Library);
        let realms = named(&mut g, p, "Eclipsed Realms", Zone::Library);
        let growth = named(&mut g, p, "Giant Growth", Zone::Library);
        let source = named(&mut g, p, "Aurora Awakener", Zone::Hand);
        cast(&mut g, source, None, &[]);
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        assert_eq!(
            g.stats(source),
            Stats {
                power: 7,
                toughness: 7
            }
        );
        assert!(g.keywords(source).contains(Keywords::TRAMPLE));
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        for viewer in [p, p.other()] {
            let view = g.observe(viewer).library_selection.unwrap();
            assert_eq!(
                view.remaining.iter().map(|o| o.id).collect::<Vec<_>>(),
                vec![growth, realms, shock, stone, forest]
            );
            assert_eq!(view.maximum, 3);
            assert!(view.public);
            assert!(!view.remaining.iter().any(|o| o.id == unseen));
        }
        reject(&mut g, Action::SelectLibrary { card: growth });
        reject(&mut g, Action::SelectLibrary { card: unseen });
        checked(&mut g, Action::SelectLibrary { card: realms });
        checked(&mut g, Action::SelectLibrary { card: stone });
        let before_rng = g.state.rng;
        checked(&mut g, Action::FinishLibrarySelection);
        choose_type(&mut g, "Elf");
        choose_type(&mut g, "Elf");
        assert_eq!(g.object_view(realms).zone, Zone::Battlefield);
        assert_eq!(g.object_view(stone).zone, Zone::Battlefield);
        assert_ne!(g.state.rng, before_rng);
        assert_eq!(g.state.players[p.index()].library.last(), Some(&unseen));
        let bottom = &g.state.players[p.index()].library[..3];
        for id in [growth, shock, forest] {
            assert!(bottom.contains(&id));
        }
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].library.last(), Some(&unseen));
    }
}
#[test]
fn ecl_green_aurora_zero_colors_does_not_reveal_and_short_library_allows_declining_everything() {
    for p in [Player::P0, Player::P1] {
        for zero in [false, true] {
            let mut g = setup(p);
            let library = g.state.players[p.index()].library.clone();
            for id in library {
                g.move_object(id, Zone::Graveyard);
            }
            let land = named(&mut g, p, "Forest", Zone::Library);
            let instant = named(&mut g, p, "Giant Growth", Zone::Library);
            if !zero {
                named(&mut g, p, "Savannah Lions", Zone::Battlefield);
            }
            let source = named(&mut g, p, "Aurora Awakener", Zone::Hand);
            cast(&mut g, source, None, &[]);
            checked(&mut g, Action::Pass);
            checked(&mut g, Action::Pass);
            if zero {
                g.move_object(source, Zone::Graveyard);
            }
            let before = g.state.players[p.index()].library.clone();
            let before_rng = g.state.rng;
            checked(&mut g, Action::Pass);
            checked(&mut g, Action::Pass);
            if zero {
                assert!(g.observe(p).library_selection.is_none());
                assert_eq!(g.state.players[p.index()].library, before);
                assert_eq!(g.state.rng, before_rng);
            } else {
                assert_eq!(g.observe(p).library_selection.unwrap().remaining.len(), 2);
                checked(&mut g, Action::FinishLibrarySelection);
                assert_eq!(g.object_view(land).zone, Zone::Library);
                assert_eq!(g.object_view(instant).zone, Zone::Library);
                assert_ne!(g.state.rng, before_rng);
            }
        }
    }
}

fn settle_safewright(g: &mut Game) {
    g.prepare_triggers();
    for _ in 0..80 {
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.resolving.is_none()
            && g.state.pending_triggers.is_empty()
        {
            return;
        }
        let action = if matches!(g.state.pending, Pending::ResolutionCreatureType { .. }) {
            Action::ChooseCreatureType {
                creature_type: crate::cards::CreatureType::from_name("Elf").unwrap(),
            }
        } else if g.state.pending == Pending::Priority {
            Action::Pass
        } else {
            panic!("unexpected Safewright phase {:?}", g.state.pending);
        };
        checked(g, action);
    }
    panic!("Safewright failed to finish");
}
#[test]
fn ecl_green_selfless_flash_grants_other_chosen_permanents_and_actual_hexproof_indestructible() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p.other());
        let elf = named(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        let kindred = named(&mut g, p, "Firdoch Core", Zone::Battlefield);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let foe = named(&mut g, p.other(), "Llanowar Elves", Zone::Battlefield);
        let source = named(&mut g, p, "Selfless Safewright", Zone::Hand);
        checked(&mut g, Action::Pass);
        cast(&mut g, source, None, &[]);
        settle_safewright(&mut g);
        assert_eq!(
            g.stats(source),
            Stats {
                power: 4,
                toughness: 2
            }
        );
        for id in [elf, kindred] {
            assert!(
                g.keywords(id)
                    .contains(Keywords::HEXPROOF.union(Keywords::INDESTRUCTIBLE))
            );
        }
        for id in [source, bear, foe] {
            assert!(
                !g.keywords(id)
                    .intersects(Keywords::HEXPROOF.union(Keywords::INDESTRUCTIBLE))
            );
        }
        let shock = named(&mut g, p.other(), "Shock", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: shock });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(elf),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p),
            },
        );
        pay(&mut g);
        settle(&mut g);
        checked(&mut g, Action::Pass);
        let murder = named(&mut g, p, "Murder", Zone::Hand);
        cast(&mut g, murder, None, &[Target::Card(elf)]);
        settle(&mut g);
        assert_eq!(g.object_view(elf).zone, Zone::Battlefield);
        let later = named(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        assert!(!g.keywords(later).contains(Keywords::INDESTRUCTIBLE));
        g.move_object(source, Zone::Graveyard);
        assert!(g.keywords(kindred).contains(Keywords::INDESTRUCTIBLE));
        super::ability_timeline_tests::next_turn(&mut g);
        for id in [elf, kindred] {
            assert!(
                !g.keywords(id)
                    .intersects(Keywords::HEXPROOF.union(Keywords::INDESTRUCTIBLE))
            );
        }
    }
}
#[test]
fn ecl_green_selfless_exact_source_exclusion_survives_departure_control_and_reentry() {
    for p in [Player::P0, Player::P1] {
        for mode in 0..3 {
            let mut g = setup(p);
            let elf = named(&mut g, p, "Llanowar Elves", Zone::Battlefield);
            let foe = named(&mut g, p.other(), "Llanowar Elves", Zone::Battlefield);
            let source = named(&mut g, p, "Selfless Safewright", Zone::Hand);
            cast(&mut g, source, None, &[]);
            checked(&mut g, Action::Pass);
            checked(&mut g, Action::Pass);
            if mode == 1 {
                let item = g.state.stack.last().unwrap().clone();
                g.gain_control(&item, Subject::Source, PlayerSet::Opponent);
            } else {
                g.move_object(source, Zone::Graveyard);
                if mode == 2 {
                    let movement = g.entry_move(source, Zone::Battlefield, false);
                    g.stage_entries(vec![movement], None);
                    g.advance_entries();
                }
            }
            settle_safewright(&mut g);
            assert!(g.keywords(elf).contains(Keywords::INDESTRUCTIBLE));
            assert!(!g.keywords(foe).contains(Keywords::INDESTRUCTIBLE));
            if mode == 1 {
                assert_eq!(g.object_view(source).controller, p.other());
                assert!(!g.keywords(source).contains(Keywords::INDESTRUCTIBLE));
            }
            if mode == 2 {
                assert!(g.keywords(source).contains(Keywords::INDESTRUCTIBLE));
            }
            g.move_object(elf, Zone::Exile);
            g.move_object(elf, Zone::Battlefield);
            assert!(!g.keywords(elf).contains(Keywords::INDESTRUCTIBLE));
        }
    }
}
#[test]
fn ecl_green_selfless_convoke_pays_green_and_generic_with_distinct_sick_creatures() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let mut helpers = Vec::new();
        for i in 0..5 {
            helpers.push(named(
                &mut g,
                p,
                if i < 2 { "Bear Cub" } else { "Savannah Lions" },
                Zone::Battlefield,
            ));
        }
        let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let source = named(&mut g, p, "Selfless Safewright", Zone::Hand);
        g.state.players[p.index()].mana = [0; 6];
        checked(&mut g, Action::AnnounceCast { card: source });
        checked(&mut g, Action::BeginConvoke);
        let object = g.object_ref(enemy);
        reject(
            &mut g,
            Action::ChooseConvoker {
                object,
                contribution: ConvokeContribution::Green,
            },
        );
        for (i, id) in helpers.iter().copied().enumerate() {
            let object = g.object_ref(id);
            let contribution = if i < 2 {
                ConvokeContribution::Green
            } else {
                ConvokeContribution::Generic
            };
            checked(
                &mut g,
                Action::ChooseConvoker {
                    object,
                    contribution,
                },
            );
            reject(
                &mut g,
                Action::ChooseConvoker {
                    object,
                    contribution,
                },
            );
        }
        pay(&mut g);
        for id in helpers {
            assert!(g.object_view(id).tapped);
        }
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        settle_safewright(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Battlefield);
    }
}

#[test]
fn ecl_green_prismabasher_freezes_target_cap_and_recomputes_resolution_boost_with_partial_legality()
{
    for p in [Player::P0, Player::P1] {
        for mode in 0..7 {
            let mut g = setup(p);
            let white = named(&mut g, p, "Savannah Lions", Zone::Battlefield);
            let red = named(&mut g, p, "Swab Goblin", Zone::Battlefield);
            let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let foe = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let land = named(&mut g, p, "Forest", Zone::Battlefield);
            let source = named(&mut g, p, "Prismabasher", Zone::Hand);
            cast(&mut g, source, None, &[]);
            checked(&mut g, Action::Pass);
            checked(&mut g, Action::Pass);
            assert_eq!(g.state.announcing.as_ref().unwrap().spec.max, 3);
            for id in [foe, land] {
                reject(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(id),
                    },
                );
            }
            if mode == 0 {
                checked(&mut g, Action::FinishTargets);
                settle(&mut g);
                assert_eq!(g.stats(source).power, 6);
                assert_eq!(g.stats(bear).power, 2);
                continue;
            }
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(bear),
                },
            );
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(bear),
                },
            );
            if mode == 6 {
                // The owned limit is fixed when targets start, despite a later color departure.
                g.move_object(red, Zone::Exile);
                assert_eq!(g.state.announcing.as_ref().unwrap().spec.max, 3);
            }
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(white),
                },
            );
            if mode == 1 || mode == 6 {
                checked(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(source),
                    },
                );
                assert!(g.state.announcing.is_none());
                reject(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(red),
                    },
                );
            } else {
                checked(&mut g, Action::FinishTargets);
            }
            match mode {
                2 => {
                    g.move_object(red, Zone::Graveyard);
                }
                3 => {
                    g.move_object(white, Zone::Exile);
                    g.move_object(white, Zone::Battlefield);
                }
                4 => {
                    g.move_object(bear, Zone::Exile);
                }
                5 => {
                    g.move_object(bear, Zone::Exile);
                    g.move_object(white, Zone::Exile);
                }
                _ => {}
            }
            settle(&mut g);
            let amount = if mode == 2 || mode == 6 { 2 } else { 3 };
            if mode != 4 && mode != 5 {
                assert_eq!(g.stats(bear).power, 2 + amount);
            }
            if mode != 3 && mode != 5 {
                assert_eq!(g.stats(white).power, 2 + amount);
            }
            if mode == 3 {
                assert_eq!(g.stats(white).power, 2);
            }
            assert_eq!(
                g.stats(source).power,
                if mode == 1 || mode == 6 {
                    6 + amount
                } else {
                    6
                }
            );
            assert!(g.keywords(source).contains(Keywords::TRAMPLE));
            assert_eq!(g.stats(foe).power, 2);
            if mode == 1 {
                super::ability_timeline_tests::next_turn(&mut g);
                assert_eq!(g.stats(source).power, 6);
                assert_eq!(g.stats(bear).power, 2);
            }
        }
    }
}
#[test]
fn ecl_green_prismabasher_source_exit_can_reduce_vivid_boost_to_zero_without_invalidating_targets()
{
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let colorless = named(&mut g, p, "Juggernaut", Zone::Battlefield);
        let source = named(&mut g, p, "Prismabasher", Zone::Hand);
        cast(&mut g, source, None, &[]);
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        assert_eq!(g.state.announcing.as_ref().unwrap().spec.max, 1);
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(colorless),
            },
        );
        g.move_object(source, Zone::Graveyard);
        let before = g.stats(colorless);
        settle(&mut g);
        assert_eq!(g.stats(colorless), before);
        assert_eq!(g.object_view(colorless).zone, Zone::Battlefield);
    }
}

#[test]
fn ecl_green_puca_draw_precedes_public_color_choice_and_five_color_activation_locks_guard() {
    assert!(serde_json::from_str::<crate::cards::ChoiceColor>("\"Colorless\"").is_err());
    for p in [Player::P0, Player::P1] {
        for color in [
            crate::cards::ChoiceColor::White,
            crate::cards::ChoiceColor::Blue,
            crate::cards::ChoiceColor::Black,
            crate::cards::ChoiceColor::Red,
            crate::cards::ChoiceColor::Green,
        ] {
            let mut g = setup(p);
            let source = named(&mut g, p, "Puca's Eye", Zone::Hand);
            cast(&mut g, source, None, &[]);
            checked(&mut g, Action::Pass);
            checked(&mut g, Action::Pass);
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
            checked(&mut g, Action::Pass);
            checked(&mut g, Action::Pass);
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
            assert_eq!(
                g.legal_actions()
                    .iter()
                    .filter(|a| matches!(a, Action::ChooseColor { .. }))
                    .count(),
                5
            );
            for viewer in [p, p.other()] {
                assert_eq!(g.observe(viewer).decision, "resolution_color");
            }
            checked(&mut g, Action::ChooseColor { color });
            assert_eq!(g.object_view(source).colors, 1 << color.index());
            reject(
                &mut g,
                Action::AnnounceActivate {
                    card: source,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            let mut permanents = Vec::new();
            for name in [
                "Savannah Lions",
                "Aegis Turtle",
                "Highborn Vampire",
                "Swab Goblin",
                "Bear Cub",
            ] {
                permanents.push(named(&mut g, p, name, Zone::Battlefield));
            }
            checked(
                &mut g,
                Action::AnnounceActivate {
                    card: source,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            let before = g.state.players[p.index()].mana.iter().sum::<u32>();
            pay(&mut g);
            assert_eq!(
                g.state.players[p.index()].mana.iter().sum::<u32>(),
                before - 3
            );
            assert!(g.object_view(source).tapped);
            // Losing a different color after activation does not undo its printed restriction.
            let removed = permanents[(color.index() + 1) % 5];
            g.move_object(removed, Zone::Graveyard);
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 2);
            assert_eq!(g.object_view(source).colors, 1 << color.index());
            super::ability_timeline_tests::next_turn(&mut g);
            assert_eq!(g.object_view(source).colors, 1 << color.index());
        }
    }
}
#[test]
fn ecl_green_puca_source_departure_and_reentry_do_not_rebind_color_choice() {
    for p in [Player::P0, Player::P1] {
        for reenter in [false, true] {
            let mut g = setup(p);
            let source = named(&mut g, p, "Puca's Eye", Zone::Hand);
            cast(&mut g, source, None, &[]);
            checked(&mut g, Action::Pass);
            checked(&mut g, Action::Pass);
            g.move_object(source, Zone::Graveyard);
            if reenter {
                let movement = g.entry_move(source, Zone::Battlefield, false);
                g.stage_entries(vec![movement], None);
                g.advance_entries();
                g.prepare_triggers();
                checked(&mut g, Action::Pass);
                checked(&mut g, Action::Pass);
                checked(
                    &mut g,
                    Action::ChooseColor {
                        color: crate::cards::ChoiceColor::Red,
                    },
                );
            }
            checked(&mut g, Action::Pass);
            checked(&mut g, Action::Pass);
            checked(
                &mut g,
                Action::ChooseColor {
                    color: crate::cards::ChoiceColor::Blue,
                },
            );
            assert_eq!(
                g.state.players[p.index()].cards_drawn_this_turn,
                if reenter { 2 } else { 1 }
            );
            assert_eq!(g.object_view(source).colors, if reenter { 8 } else { 0 });
        }
    }
}
#[test]
fn ecl_green_puca_resolved_color_survives_ability_removal_and_control_change() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = named(&mut g, p, "Puca's Eye", Zone::Hand);
        cast(&mut g, source, None, &[]);
        for _ in 0..4 {
            checked(&mut g, Action::Pass);
        }
        checked(
            &mut g,
            Action::ChooseColor {
                color: crate::cards::ChoiceColor::Blue,
            },
        );
        for name in [
            "Savannah Lions",
            "Highborn Vampire",
            "Swab Goblin",
            "Bear Cub",
        ] {
            named(&mut g, p, name, Zone::Battlefield);
        }
        let cold = named(&mut g, p, "Stop Cold", Zone::Hand);
        cast(&mut g, cold, None, &[Target::Card(source)]);
        settle(&mut g);
        assert_eq!(g.object_view(source).colors, 2);
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: source,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        let offering = named(&mut g, p, "Harmless Offering", Zone::Hand);
        cast(
            &mut g,
            offering,
            None,
            &[Target::Player(p.other()), Target::Card(source)],
        );
        settle(&mut g);
        assert_eq!(g.object_view(source).controller, p.other());
        assert_eq!(g.object_view(source).colors, 2);
        g.move_object(source, Zone::Exile);
        assert!(
            !g.observe(p)
                .color_effects
                .iter()
                .any(|row| row.object.card == source)
        );
    }
}

#[test]
fn ecl_green_wickermaw_paid_surveillance_private_choice_and_once_per_turn_color_mana() {
    for p in [Player::P0, Player::P1] {
        for color in 0..5 {
            let mut g = setup(p);
            let top = named(&mut g, p, "Shock", Zone::Library);
            let source = named(&mut g, p, "Foraging Wickermaw", Zone::Hand);
            cast(&mut g, source, None, &[]);
            for _ in 0..4 {
                checked(&mut g, Action::Pass);
            }
            assert!(g.observe(p).library_choice.is_some());
            assert!(g.observe(p.other()).library_choice.is_none());
            let destination = if color % 2 == 0 {
                LibraryDestination::Top
            } else {
                LibraryDestination::Graveyard
            };
            checked(
                &mut g,
                Action::ArrangeLibrary {
                    card: top,
                    destination,
                },
            );
            assert_eq!(
                g.object_view(top).zone,
                if color % 2 == 0 {
                    Zone::Library
                } else {
                    Zone::Graveyard
                }
            );
            assert_eq!(
                g.stats(source),
                Stats {
                    power: 1,
                    toughness: 3
                }
            );
            assert!(g.object_view(source).summoning_sick);
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
            let mut production = [0; 6];
            production[color] = 1;
            checked(
                &mut g,
                Action::AnnounceMana {
                    card: source,
                    ability: 0,
                    counter_payment: Counters::default(),
                    production,
                },
            );
            reject(&mut g, Action::PayCosts { payment: [0; 6] });
            checked(
                &mut g,
                Action::PayCosts {
                    payment: [0, 0, 0, 0, 0, 1],
                },
            );
            assert!(g.state.stack.is_empty());
            assert_eq!(g.state.players[p.index()].mana, production);
            assert_eq!(g.object_view(source).colors, 1 << color);
            assert!(!g.object_view(source).tapped);
            reject(
                &mut g,
                Action::AnnounceMana {
                    card: source,
                    ability: 0,
                    counter_payment: Counters::default(),
                    production,
                },
            );
            super::ability_timeline_tests::next_turn(&mut g);
            assert_eq!(g.object_view(source).colors, 0);
            if g.actor() != Some(p) {
                checked(&mut g, Action::Pass);
            }
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
            checked(
                &mut g,
                Action::AnnounceMana {
                    card: source,
                    ability: 0,
                    counter_payment: Counters::default(),
                    production,
                },
            );
            checked(
                &mut g,
                Action::PayCosts {
                    payment: [0, 0, 0, 0, 0, 1],
                },
            );
            assert_eq!(g.object_view(source).colors, 1 << color);
        }
    }
}
#[test]
fn ecl_green_wickermaw_color_change_enables_later_bloom_tender_in_same_guided_funding() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let wicker = named(&mut g, p, "Foraging Wickermaw", Zone::Battlefield);
        let bloom = named(&mut g, p, "Bloom Tender", Zone::Battlefield);
        g.state.objects[bloom as usize].summoning_sick = false;
        let spell = named(&mut g, p, "Divination", Zone::Hand);
        g.state.players[p.index()].mana = [0; 6];
        reject(&mut g, Action::AnnounceCast { card: spell });
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        checked(&mut g, Action::AnnounceCast { card: spell });
        let convert = g.legal_actions().into_iter().find(|action| matches!(action, Action::ActivateMana { card, production, .. } if *card == wicker && *production == [0,1,0,0,0,0])).unwrap();
        checked(&mut g, convert);
        assert_eq!(g.object_view(wicker).colors, 2);
        assert_eq!(g.state.announcing.as_ref().unwrap().item.card, spell);
        let tender = g.legal_actions().into_iter().find(|action| matches!(action, Action::ActivateMana { card, production, .. } if *card == bloom && *production == [0,1,0,0,1,0])).unwrap();
        checked(&mut g, tender);
        pay(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 2);
        assert!(g.object_view(bloom).tapped);
        assert!(!g.object_view(wicker).tapped);
    }
}

pub(super) fn enter_growth(g: &mut Game, p: Player, land: CardId, color: usize) -> CardId {
    let aura = named(g, p, "Shimmerwilds Growth", Zone::Hand);
    cast(g, aura, None, &[Target::Card(land)]);
    checked(g, Action::Pass);
    checked(g, Action::Pass);
    assert_eq!(g.actor(), Some(p));
    checked(
        g,
        Action::ChooseEntryValue {
            value: EntryChoiceValue::Color(crate::cards::ChoiceColor::ALL[color]),
        },
    );
    settle(g);
    aura
}

#[test]
fn ecl_green_growth_paid_entry_all_colors_and_opponents_land_bonus_has_no_stack() {
    for p in [Player::P0, Player::P1] {
        for color in 0..5 {
            for opponent in [false, true] {
                let mut g = setup(p);
                let controller = if opponent { p.other() } else { p };
                let land = named(&mut g, controller, "Forest", Zone::Battlefield);
                let aura = enter_growth(&mut g, p, land, color);
                assert_eq!(g.object_view(land).colors, 1 << color);
                assert_eq!(g.object_view(land).attached_to, None);
                assert_eq!(g.object_view(aura).attached_to, Some(g.object_ref(land)));
                g.state.players[p.index()].mana = [0; 6];
                g.state.players[p.other().index()].mana = [0; 6];
                if g.actor() != Some(controller) {
                    checked(&mut g, Action::Pass);
                }
                checked(&mut g, Action::TapMana { card: land });
                let mut expected = [0; 6];
                expected[4] = 1;
                expected[color] += 1;
                assert_eq!(g.state.players[controller.index()].mana, expected);
                assert_eq!(g.state.players[controller.other().index()].mana, [0; 6]);
                assert!(g.state.stack.is_empty());
                assert!(g.state.pending_triggers.is_empty());
                g.state.objects[land as usize].tapped = false;
                g.move_object(aura, Zone::Exile);
                assert_eq!(g.object_view(land).colors, 0);
                checked(&mut g, Action::TapMana { card: land });
                expected[4] += 1;
                assert_eq!(g.state.players[controller.index()].mana, expected);
            }
        }
    }
}

#[test]
fn ecl_green_growth_restricted_realms_main_mana_and_unrestricted_bonus_fund_unrelated_spell() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let land = enter_realms(&mut g, p);
        settle(&mut g);
        let _aura = enter_growth(&mut g, p, land, 3);
        let shock = named(&mut g, p, "Shock", Zone::Hand);
        g.state.players[p.index()].mana = [0; 6];
        // Only the bonus can pay this spell; the Realms' main blue mana cannot.
        checked(&mut g, Action::AnnounceCast { card: shock });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        checked(
            &mut g,
            Action::ActivateMana {
                card: land,
                ability: 1,
                payment: [0; 6],
                counter_payment: Counters::default(),
                production: [0, 1, 0, 0, 0, 0],
            },
        );
        assert_eq!(g.state.players[p.index()].mana, [0, 1, 0, 1, 0, 0]);
        checked(
            &mut g,
            Action::PayCosts {
                payment: [0, 0, 0, 1, 0, 0],
            },
        );
        assert_eq!(g.state.players[p.index()].mana, [0, 1, 0, 0, 0, 0]);
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 18);
    }
}

#[test]
fn ecl_green_growth_external_creature_tap_cost_never_counts_as_land_tapped_for_mana() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let land = named(&mut g, p, "@token/mutavault", Zone::Battlefield);
        let _aura = enter_growth(&mut g, p, land, 2);
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: land,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.object_kinds(land, &g.state.objects[land as usize]) & 3, 3);
        let drum = named(&mut g, p, "Springleaf Drum", Zone::Battlefield);
        g.state.players[p.index()].mana = [0; 6];
        checked(
            &mut g,
            Action::AnnounceMana {
                card: drum,
                ability: 0,
                counter_payment: Counters::default(),
                production: [0, 1, 0, 0, 0, 0],
            },
        );
        let reference = g.object_ref(land);
        checked(&mut g, Action::ChooseAdditionalCost { object: reference });
        pay(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0, 1, 0, 0, 0, 0]);
        assert!(g.object_view(land).tapped);
    }
}

#[test]
fn ecl_green_growth_multiple_providers_timestamps_and_exact_departed_land_bonus_capture() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let land = named(&mut g, p, "Forest", Zone::Battlefield);
        let first = enter_growth(&mut g, p, land, 1);
        let second = enter_growth(&mut g, p, land, 3);
        assert_eq!(g.object_view(land).colors, 8);
        g.set_color_effects(&[land], 31, crate::cards::AnimationDuration::EndOfTurn);
        assert_eq!(g.object_view(land).colors, 31);
        g.expire_color_effects();
        assert_eq!(g.object_view(land).colors, 8);
        g.state.players[p.index()].mana = [0; 6];
        checked(&mut g, Action::TapMana { card: land });
        assert_eq!(g.state.players[p.index()].mana, [0, 1, 0, 1, 1, 0]);
        g.move_object(second, Zone::Graveyard);
        assert_eq!(g.object_view(land).colors, 2);
        // Typed LKI capture fixture: there is no admitted T+sacrifice land-mana
        // producer. Preserve the bonus from the exact tap event before departure.
        let captured = g.mana_tap_bonus(land);
        let snapshot = g.snapshot();
        let old = g.object_ref(land);
        g.move_object(land, Zone::Graveyard);
        assert_eq!(g.mana_tap_bonus(land), [0; 6]);
        g.add_mana(p, captured);
        assert_eq!(g.state.players[p.index()].mana, [0, 2, 0, 1, 1, 0]);
        g.move_object(land, Zone::Battlefield);
        assert_ne!(g.object_ref(land), old);
        assert_eq!(g.mana_tap_bonus(land), [0; 6]);
        assert_eq!(g.object_view(first).attached_to, Some(old));
        g.restore(&snapshot).unwrap();
        assert_eq!(g.mana_tap_bonus(land), captured);
    }
}

#[test]
fn ecl_green_growth_granted_mana_bonus_is_separate_and_overflow_restores_tap_and_pool() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let land = named(&mut g, p, "Forest", Zone::Battlefield);
        named(&mut g, p, "Bear Cub", Zone::Battlefield);
        enter_growth(&mut g, p, land, 0);
        let horizons = named(&mut g, p, "New Horizons", Zone::Hand);
        cast(&mut g, horizons, None, &[Target::Card(land)]);
        settle(&mut g);
        g.state.players[p.index()].mana = [0; 6];
        let grantor = g.object_ref(horizons);
        checked(
            &mut g,
            Action::TapGrantedMana {
                card: land,
                grantor,
                production: [0, 2, 0, 0, 0, 0],
            },
        );
        assert_eq!(g.state.players[p.index()].mana, [1, 2, 0, 0, 0, 0]);
        g.state.objects[land as usize].tapped = false;
        g.state.players[p.index()].mana[0] = u32::MAX;
        let snapshot = g.snapshot();
        let revision = g.revision();
        assert!(matches!(
            g.apply(p, Action::TapMana { card: land }),
            Err(Error::CapacityExceeded(_))
        ));
        assert_eq!(g.snapshot(), snapshot);
        assert_eq!(g.revision(), revision);
    }
}

#[test]
fn ecl_green_lys_behold_accepts_permanent_or_hand_elf_including_kindred_and_reveals_only_on_payment()
 {
    for p in [Player::P0, Player::P1] {
        for hand in [false, true] {
            for kindred in [false, true] {
                let mut g = setup(p);
                let witness = named(
                    &mut g,
                    p,
                    if kindred {
                        "Morcant's Eyes"
                    } else {
                        "Llanowar Elves"
                    },
                    if hand { Zone::Hand } else { Zone::Battlefield },
                );
                if !hand {
                    let reference = g.object_ref(witness);
                    g.grant_keyword_effects(
                        &[witness],
                        reference,
                        Keywords::HEXPROOF.union(Keywords::PROTECTION_EVERYTHING),
                    );
                }
                let wrong = named(&mut g, p, "Bear Cub", Zone::Hand);
                let opponent = named(&mut g, p.other(), "Llanowar Elves", Zone::Battlefield);
                let source = named(&mut g, p, "Lys Alana Dignitary", Zone::Hand);
                g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 1];
                checked(&mut g, Action::AnnounceCast { card: source });
                checked(&mut g, Action::ChooseSpellCost { cost: 0 });
                for card in [wrong, opponent, source] {
                    let object = g.object_ref(card);
                    reject(&mut g, Action::ChooseAdditionalCost { object });
                }
                assert!(g.observe(p.other()).additional_cost.is_none());
                let object = g.object_ref(witness);
                checked(&mut g, Action::ChooseAdditionalCost { object });
                assert!(!g.observe(p.other()).known_hand.contains(&object));
                reject(&mut g, Action::PayCosts { payment: [0; 6] });
                assert!(!g.observe(p.other()).known_hand.contains(&object));
                checked(
                    &mut g,
                    Action::PayCosts {
                        payment: [0, 0, 0, 0, 1, 1],
                    },
                );
                assert_eq!(
                    g.object_view(witness).zone,
                    if hand { Zone::Hand } else { Zone::Battlefield }
                );
                assert_eq!(g.observe(p.other()).known_hand.contains(&object), hand);
                settle(&mut g);
                assert_eq!(
                    g.stats(source),
                    Stats {
                        power: 2,
                        toughness: 3
                    }
                );
                assert_eq!(g.object_view(source).zone, Zone::Battlefield);
            }
        }
    }
}

#[test]
fn ecl_green_lys_extra_mana_route_cannot_behold_source_and_free_cast_still_requires_choice() {
    for p in [Player::P0, Player::P1] {
        for free in [false, true] {
            let mut g = setup(p);
            if free {
                named(&mut g, p, "Omniscience", Zone::Battlefield);
            }
            let source = named(&mut g, p, "Lys Alana Dignitary", Zone::Hand);
            let action = if free {
                Action::AnnounceFree {
                    card: source,
                    face: CastFace::Normal,
                    flash_surcharge: false,
                }
            } else {
                Action::AnnounceCast { card: source }
            };
            g.state.players[p.index()].mana = if free { [0; 6] } else { [0, 0, 0, 0, 1, 1] };
            reject(&mut g, action.clone());
            g.state.players[p.index()].mana = if free {
                [0, 0, 0, 0, 0, 2]
            } else {
                [0, 0, 0, 0, 1, 3]
            };
            checked(&mut g, action);
            reject(&mut g, Action::ChooseSpellCost { cost: 0 });
            checked(&mut g, Action::ChooseSpellCost { cost: 1 });
            assert!(g.observe(p).additional_cost.is_none());
            pay(&mut g);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            settle(&mut g);
            assert_eq!(g.object_view(source).zone, Zone::Battlefield);
        }
    }
}

#[test]
fn ecl_green_lys_behold_witness_can_tap_for_same_cast_and_graveyard_card_guard_ignores_owner_control()
 {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let witness = named(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        g.state.objects[witness as usize].summoning_sick = false;
        let land = named(&mut g, p, "Forest", Zone::Battlefield);
        let source = named(&mut g, p, "Lys Alana Dignitary", Zone::Hand);
        g.state.players[p.index()].mana = [0; 6];
        checked(&mut g, Action::AnnounceCast { card: source });
        checked(&mut g, Action::ChooseSpellCost { cost: 0 });
        let object = g.object_ref(witness);
        checked(&mut g, Action::ChooseAdditionalCost { object });
        checked(&mut g, Action::TapMana { card: witness });
        checked(&mut g, Action::TapMana { card: land });
        pay(&mut g);
        assert!(g.object_view(witness).tapped);
        settle(&mut g);
        g.state.objects[source as usize].summoning_sick = false;
        let action = Action::AnnounceMana {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
            production: [0, 0, 0, 0, 2, 0],
        };
        reject(&mut g, action.clone());
        let foreign = named(&mut g, p.other(), "Morcant's Eyes", Zone::Graveyard);
        reject(&mut g, action.clone());
        let elf = named(&mut g, p, "Morcant's Eyes", Zone::Graveyard);
        // Cards in a graveyard are checked by owner, not retained former control.
        g.state.objects[elf as usize].controller = p.other();
        g.state.players[p.index()].mana = [0; 6];
        checked(&mut g, action);
        g.move_object(elf, Zone::Exile);
        // This is a restriction on starting the activation, not a later guard.
        pay(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 0, 2, 0]);
        assert!(g.object_view(source).tapped);
        assert_eq!(g.object_view(foreign).owner, p.other());
    }
}

#[test]
fn ecl_green_lys_mana_route_may_decline_eligible_private_hand_witness() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let witness = named(&mut g, p, "Llanowar Elves", Zone::Hand);
        let source = named(&mut g, p, "Lys Alana Dignitary", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 3];
        checked(&mut g, Action::AnnounceCast { card: source });
        assert!(
            g.legal_actions()
                .contains(&Action::ChooseSpellCost { cost: 0 })
        );
        checked(&mut g, Action::ChooseSpellCost { cost: 1 });
        pay(&mut g);
        assert!(
            !g.observe(p.other())
                .known_hand
                .contains(&g.object_ref(witness))
        );
        assert_eq!(g.object_view(witness).zone, Zone::Hand);
        settle(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Battlefield);
    }
}

fn announce_champions(g: &mut Game, p: Player, witness: CardId) -> CardId {
    let source = named(g, p, "Champions of the Perfect", Zone::Hand);
    checked(g, Action::AnnounceCast { card: source });
    let object = g.object_ref(witness);
    checked(g, Action::ChooseAdditionalCost { object });
    pay(g);
    if g.object_view(witness).token.is_some() {
        assert_eq!(g.object_view(witness).zone, Zone::Outside);
    } else {
        assert_eq!(g.object_view(witness).zone, Zone::Exile);
        assert!(g.observe(p).linked_exile.iter().any(
            |link| link.source == g.object_ref(source) && link.exiled == g.object_ref(witness)
        ));
    }
    source
}

#[test]
fn ecl_green_champions_behold_exile_links_actual_entry_and_all_departures_return_owner_hand() {
    for p in [Player::P0, Player::P1] {
        for hand in [false, true] {
            for kindred in [false, true] {
                for destination in [Zone::Graveyard, Zone::Exile, Zone::Hand, Zone::Library] {
                    let mut g = setup(p);
                    let witness = named(
                        &mut g,
                        p,
                        if kindred {
                            "Morcant's Eyes"
                        } else {
                            "Llanowar Elves"
                        },
                        if hand { Zone::Hand } else { Zone::Battlefield },
                    );
                    let library = g.state.players[p.index()].library.len();
                    let source = announce_champions(&mut g, p, witness);
                    let spell = g.object_ref(source);
                    settle(&mut g);
                    assert_eq!(
                        g.stats(source),
                        Stats {
                            power: 6,
                            toughness: 6
                        }
                    );
                    assert_eq!(
                        g.state.players[p.index()].library.len(),
                        library,
                        "no self-cast draw"
                    );
                    assert!(g.observe(p).linked_exile.iter().any(|link| link.source == g.object_ref(source) && link.source != spell));
                    if kindred {
                        let offer = named(&mut g, p, "Harmless Offering", Zone::Hand);
                        cast(
                            &mut g,
                            offer,
                            None,
                            &[Target::Player(p.other()), Target::Card(source)],
                        );
                        settle(&mut g);
                        assert_eq!(g.object_view(source).controller, p.other());
                    }
                    g.move_object(source, destination);
                    settle(&mut g);
                    assert_eq!(g.object_view(witness).zone, Zone::Hand);
                    assert_eq!(g.object_view(witness).owner, p);
                    assert!(
                        g.observe(p.other())
                            .known_hand
                            .contains(&g.object_ref(witness))
                    );
                }
            }
        }
    }
}

#[test]
fn ecl_green_champions_countered_spell_cost_stays_exiled_and_cannot_bind_later_entry() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let witness = named(&mut g, p, "Llanowar Elves", Zone::Hand);
        let source = announce_champions(&mut g, p, witness);
        let spell = g.object_ref(source);
        checked(&mut g, Action::Pass);
        let cancel = named(&mut g, p.other(), "Cancel", Zone::Hand);
        cast(&mut g, cancel, None, &[Target::Spell(source)]);
        settle(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Graveyard);
        assert_eq!(g.object_view(witness).zone, Zone::Exile);
        g.move_object(source, Zone::Battlefield);
        assert_ne!(g.object_ref(source), spell);
        g.move_object(source, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.object_view(witness).zone, Zone::Exile);
    }
}

#[test]
fn ecl_green_champions_blink_source_stale_exile_and_token_cost_keep_exact_links() {
    for p in [Player::P0, Player::P1] {
        for mode in 0..3 {
            let mut g = setup(p);
            let witness = named(
                &mut g,
                p,
                if mode == 2 {
                    "@token/shapeshifter_1_1_colorless_changeling"
                } else {
                    "Llanowar Elves"
                },
                Zone::Battlefield,
            );
            let source = announce_champions(&mut g, p, witness);
            settle(&mut g);
            if mode == 0 {
                let old = g.object_ref(source);
                g.move_object(source, Zone::Exile);
                g.move_object(source, Zone::Battlefield);
                assert_ne!(g.object_ref(source), old);
                settle(&mut g);
                assert_eq!(g.object_view(source).zone, Zone::Battlefield);
                assert_eq!(g.object_view(witness).zone, Zone::Hand);
            } else {
                if mode == 1 {
                    g.move_object(witness, Zone::Hand);
                    g.move_object(witness, Zone::Exile);
                } else {
                    assert_eq!(g.object_view(witness).zone, Zone::Outside);
                }
                g.move_object(source, Zone::Exile);
                settle(&mut g);
                assert_eq!(
                    g.object_view(witness).zone,
                    if mode == 1 {
                        Zone::Exile
                    } else {
                        Zone::Outside
                    }
                );
            }
        }
    }
}

#[test]
fn ecl_green_champions_draw_is_controller_creature_cast_trigger_and_survives_source_departure() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let witness = named(&mut g, p, "Llanowar Elves", Zone::Hand);
        let source = announce_champions(&mut g, p, witness);
        settle(&mut g);
        let count = g.state.players[p.index()].library.len();
        let shock = named(&mut g, p, "Shock", Zone::Hand);
        cast(&mut g, shock, None, &[Target::Player(p.other())]);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].library.len(), count);
        let bear = named(&mut g, p, "Bear Cub", Zone::Hand);
        cast(&mut g, bear, None, &[]);
        g.move_object(source, Zone::Exile);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].library.len(), count - 1);
        assert_eq!(g.object_view(witness).zone, Zone::Hand);
        assert_eq!(g.object_view(bear).zone, Zone::Battlefield);
    }
}

#[test]
fn ecl_green_champions_departure_uses_frozen_printed_ability_availability_for_simultaneous_moves() {
    for p in [Player::P0, Player::P1] {
        for simultaneous in [false, true] {
            let mut g = setup(p);
            let witness = named(&mut g, p, "Llanowar Elves", Zone::Hand);
            let source = announce_champions(&mut g, p, witness);
            settle(&mut g);
            let blank = named(&mut g, p, "Witness Protection", Zone::Hand);
            cast(&mut g, blank, None, &[Target::Card(source)]);
            settle(&mut g);
            assert!(!g.printed_abilities_available(source));
            if simultaneous {
                let events = g.move_objects_raw(&[blank, source], Zone::Exile);
                g.finish_zone_events(&events, &[]);
            } else {
                g.move_object(blank, Zone::Exile);
                assert!(g.printed_abilities_available(source));
                g.move_object(source, Zone::Exile);
            }
            settle(&mut g);
            assert_eq!(
                g.object_view(witness).zone,
                if simultaneous {
                    Zone::Exile
                } else {
                    Zone::Hand
                }
            );
        }
    }
}
