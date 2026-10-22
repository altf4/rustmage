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
