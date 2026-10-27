//! Paid Oracle-derived outcomes and representative contracts for four HOB spells.
use super::*;
fn until(g: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..32 {
        if ready(g) {
            return;
        }
        if g.legal_actions().contains(&Action::Pass) {
            act(g, Action::Pass);
        } else {
            choose(g, |a| matches!(a, Action::PutTrigger { .. }));
        }
    }
    panic!("decision budget: {:?}", g.state.pending);
}
fn bind(g: &mut Game, target: Target) {
    act(g, Action::ChooseTarget { target });
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
    pay(g);
}
fn modal(g: &mut Game, id: CardId, mode: u16, target: Target) {
    act(g, Action::AnnounceCast { card: id });
    act(g, Action::ChooseMode { mode });
    bind(g, target);
}
fn library(g: &mut Game, p: Player, names: &[&str]) -> Vec<CardId> {
    let old = g.state.players[p.index()].library.to_vec();
    for id in old {
        g.state.objects[id as usize].zone = Zone::Outside;
    }
    g.state.players[p.index()].library.clear();
    let mut ids: Vec<_> = names
        .iter()
        .rev()
        .map(|name| put(g, p, name, Zone::Library))
        .collect();
    ids.reverse();
    ids
}
fn soldiers(g: &Game, p: Player) -> usize {
    g.state
        .objects
        .iter()
        .filter(|o| o.zone == Zone::Battlefield && o.controller == p && o.card.index() == 5241)
        .count()
}
#[test]
fn sound_captures_target_mana_value_and_recruits_independent_of_counter_success() {
    for p in [Player::P0, Player::P1] {
        for (name, target, mv, uncounterable) in [
            ("Grizzly Bears", false, 2, false),
            ("Hill Giant", false, 4, false),
            ("Long Goodbye", true, 2, true),
        ] {
            let mut g = scenario(p.other());
            let bear = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
            let victim = put(&mut g, p.other(), name, Zone::Hand);
            cast(
                &mut g,
                victim,
                if target {
                    Some(Target::Card(bear))
                } else {
                    None
                },
            );
            assert_eq!(g.state.stack.last().unwrap().mana_value(), mv);
            act(&mut g, Action::Pass);
            let discard = put(&mut g, p, "Shock", Zone::Hand);
            let drawn = put(&mut g, p, "Island", Zone::Library);
            let sound = put(&mut g, p, "Sound the Trumpets", Zone::Hand);
            g.state.players[p.index()].mana = [0, 2, 0, 0, 0, 1];
            cast(&mut g, sound, Some(Target::Spell(victim)));
            if mv <= 2 {
                until(&mut g, |g| {
                    matches!(g.state.pending, Pending::EffectDiscard { .. })
                });
                assert_eq!(g.object_view(drawn).zone, Zone::Hand);
                if p == Player::P0 && name == "Grizzly Bears" {
                    super::contracts::branch_contract(g.clone());
                }
                assert_eq!(
                    g.object_view(victim).zone,
                    if uncounterable {
                        Zone::Stack
                    } else {
                        Zone::Graveyard
                    }
                );
                assert!(
                    g.observe(p.other())
                        .objects
                        .iter()
                        .all(|o| o.id != drawn && o.id != discard)
                );
                reject(&mut g, Action::Discard { card: bear });
                act(&mut g, Action::Discard { card: discard });
            }
            settle(&mut g);
            assert_eq!(soldiers(&g, p), usize::from(mv <= 2));
            assert_eq!(
                g.state.players[p.index()].cards_drawn_this_turn,
                u64::from(mv <= 2)
            );
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert_eq!(g.object_view(victim).zone, Zone::Graveyard);
            if uncounterable {
                assert_eq!(g.object_view(bear).zone, Zone::Graveyard);
            }
        }
    }
}
#[test]
fn sound_uses_x_on_the_stack_and_does_nothing_for_a_vanished_target() {
    for p in [Player::P0, Player::P1] {
        for x in [1, 2] {
            let mut g = scenario(p.other());
            let victim = put(&mut g, p.other(), "Wildwood Scourge", Zone::Hand);
            act(&mut g, Action::AnnounceCast { card: victim });
            act(&mut g, Action::ChooseX { value: x });
            pay(&mut g);
            act(&mut g, Action::Pass);
            let discard = put(&mut g, p, "Forest", Zone::Hand);
            let sound = put(&mut g, p, "Sound the Trumpets", Zone::Hand);
            cast(&mut g, sound, Some(Target::Spell(victim)));
            if x == 1 {
                until(&mut g, |g| {
                    matches!(g.state.pending, Pending::EffectDiscard { .. })
                });
                act(&mut g, Action::Discard { card: discard });
            }
            settle(&mut g);
            assert_eq!(
                g.state.players[p.index()].cards_drawn_this_turn,
                u64::from(x == 1)
            );
            assert_eq!(soldiers(&g, p), 0);
        }
        let mut g = scenario(p.other());
        let victim = put(&mut g, p.other(), "Shock", Zone::Hand);
        cast(&mut g, victim, Some(Target::Player(p)));
        act(&mut g, Action::Pass);
        let sound = put(&mut g, p, "Sound the Trumpets", Zone::Hand);
        cast(&mut g, sound, Some(Target::Spell(victim)));
        let negate = put(&mut g, p, "Negate", Zone::Hand);
        cast(&mut g, negate, Some(Target::Spell(victim)));
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
        assert_eq!(soldiers(&g, p), 0);
        assert_eq!(g.state.players[p.index()].life, 20);
    }
}
#[test]
fn gnashing_minus_five_registers_death_replacement_before_sbas_and_expires() {
    for p in [Player::P0, Player::P1] {
        for survives in [false, true] {
            let mut g = scenario(p);
            let victim = put(
                &mut g,
                p.other(),
                if survives {
                    "Gigantosaurus"
                } else {
                    "Grizzly Bears"
                },
                Zone::Battlefield,
            );
            let spell = put(&mut g, p, "Gnashing of Teeth", Zone::Hand);
            g.state.players[p.index()].mana = [0, 0, 2, 0, 0, 1];
            modal(&mut g, spell, 0, Target::Card(victim));
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            if survives {
                assert_eq!(stats(&g, victim), (5, 5));
                let murder = put(&mut g, p, "Murder", Zone::Hand);
                g.state.players[p.index()].mana = [0, 0, 2, 0, 0, 1];
                cast(&mut g, murder, Some(Target::Card(victim)));
                settle(&mut g);
            }
            assert_eq!(g.object_view(victim).zone, Zone::Exile);
        }
        let mut g = scenario(p);
        let victim = put(&mut g, p.other(), "Gigantosaurus", Zone::Battlefield);
        let spell = put(&mut g, p, "Gnashing of Teeth", Zone::Hand);
        modal(&mut g, spell, 0, Target::Card(victim));
        settle(&mut g);
        g.finish_cleanup();
        assert_eq!(stats(&g, victim), (10, 10));
        g.move_object(victim, Zone::Graveyard);
        assert_eq!(g.object_view(victim).zone, Zone::Graveyard);
    }
}
#[test]
fn gnashing_targeted_player_freezes_creatures_even_through_hexproof_and_control_changes() {
    for p in [Player::P0, Player::P1] {
        for target in [p, p.other()] {
            let mut g = scenario(p);
            let victim = put(&mut g, target, "Grizzly Bears", Zone::Battlefield);
            let protected = put(&mut g, target, "Grizzly Bears", Zone::Battlefield);
            g.state.objects[protected as usize].gained_keywords = Keywords::HEXPROOF;
            let other = put(&mut g, target.other(), "Grizzly Bears", Zone::Battlefield);
            let spell = put(&mut g, p, "Gnashing of Teeth", Zone::Hand);
            modal(&mut g, spell, 1, Target::Player(target));
            settle(&mut g);
            assert_eq!(stats(&g, victim), (1, 1));
            assert_eq!(stats(&g, protected), (1, 1));
            assert_eq!(stats(&g, other), (2, 2));
            let later = put(&mut g, target, "Grizzly Bears", Zone::Battlefield);
            assert_eq!(stats(&g, later), (2, 2));
            let reference = g.object_ref(victim);
            g.change_controller(reference, target.other());
            assert_eq!(stats(&g, victim), (1, 1));
            g.finish_cleanup();
            assert_eq!(stats(&g, victim), (2, 2));
        }
    }
}
#[test]
fn gnashing_and_stone_reject_stale_creature_incarnations() {
    for (name, mode) in [("Gnashing of Teeth", 0), ("Stone by Sunlight", 1)] {
        let p = Player::P0;
        let mut g = scenario(p);
        let victim = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let spell = put(&mut g, p, name, Zone::Hand);
        modal(&mut g, spell, mode, Target::Card(victim));
        g.move_object(victim, Zone::Hand);
        g.move_object(victim, Zone::Battlefield);
        settle(&mut g);
        assert_eq!(stats(&g, victim), (2, 2));
        assert_eq!(g.object_view(victim).kinds, 1);
        assert!(!g.keywords(victim).contains(Keywords::INDESTRUCTIBLE));
        g.move_object(victim, Zone::Graveyard);
        assert_eq!(g.object_view(victim).zone, Zone::Graveyard);
    }
}
#[test]
fn stone_destroy_rechecks_power_and_other_mode_preserves_characteristics_and_cleanup() {
    for p in [Player::P0, Player::P1] {
        for lowered in [false, true] {
            let mut g = scenario(p);
            let small = put(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
            let big = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
            g.state.objects[big as usize].counters.plus_one = 1;
            let spell = put(&mut g, p, "Stone by Sunlight", Zone::Hand);
            g.state.players[p.index()].mana = [1, 0, 0, 0, 0, 1];
            act(&mut g, Action::AnnounceCast { card: spell });
            act(&mut g, Action::ChooseMode { mode: 0 });
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(small),
                },
            );
            bind(&mut g, Target::Card(big));
            if lowered {
                g.state.objects[big as usize].counters.plus_one = 0;
            }
            settle(&mut g);
            assert_eq!(
                g.object_view(big).zone,
                if lowered {
                    Zone::Battlefield
                } else {
                    Zone::Graveyard
                }
            );
        }
        let mut g = scenario(p);
        let victim = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        g.state.objects[victim as usize].counters.plus_one = 1;
        let before = g.object_view(victim);
        let spell = put(&mut g, p, "Stone by Sunlight", Zone::Hand);
        modal(&mut g, spell, 1, Target::Card(victim));
        settle(&mut g);
        let after = g.object_view(victim);
        assert_eq!(after.kinds, before.kinds | 16);
        assert_eq!(after.colors, before.colors);
        assert_eq!(stats(&g, victim), (3, 3));
        assert!(g.has_subtype(victim, "Bear"));
        assert!(g.keywords(victim).contains(Keywords::INDESTRUCTIBLE));
        let murder = put(&mut g, p, "Murder", Zone::Hand);
        cast(&mut g, murder, Some(Target::Card(victim)));
        settle(&mut g);
        assert_eq!(g.object_view(victim).zone, Zone::Battlefield);
        super::contracts::branch_contract(g.clone());
        g.finish_cleanup();
        assert_eq!(g.object_view(victim).kinds, 1);
        assert!(!g.keywords(victim).contains(Keywords::INDESTRUCTIBLE));
    }
}
#[test]
fn stone_adds_only_artifact_when_a_later_copy_removes_creature() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let shuri = put(&mut g, p, "Shuri, Wakandan Inventor", Zone::Battlefield);
        let victim = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let pick = put(&mut g, p, "Goldvein Pick", Zone::Battlefield);
        let spell = put(&mut g, p, "Stone by Sunlight", Zone::Hand);
        modal(&mut g, spell, 1, Target::Card(victim));
        settle(&mut g);
        act(
            &mut g,
            Action::AnnounceActivate {
                card: shuri,
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
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(pick),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            act(&mut g, Action::FinishTargets);
        }
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(victim).kinds, 16);
        assert!(!g.is_creature(victim));
        assert!(g.keywords(victim).contains(Keywords::INDESTRUCTIBLE));
        super::contracts::branch_contract(g.clone());
    }
}
#[test]
fn forest_gate_private_twenty_land_selection_tapped_entry_full_shuffle_and_life() {
    for p in [Player::P0, Player::P1] {
        for take in [0, 1, 2] {
            let mut g = scenario(p);
            let mut names = vec!["Grizzly Bears"; 22];
            names[0] = "Forest";
            names[19] = "Mountain";
            names[20] = "Island";
            let top = library(&mut g, p, &names);
            let untouched = g.state.players[p.index()].library.clone();
            let spell = put(&mut g, p, "Through the Forest Gate", Zone::Hand);
            g.state.players[p.index()].mana = [0, 0, 0, 0, 2, 6];
            cast(&mut g, spell, None);
            until(&mut g, |g| {
                g.legal_actions().contains(&Action::FinishLibrarySelection)
            });
            assert_eq!(g.state.players[p.index()].life, 20);
            assert_eq!(
                g.observe(p)
                    .library_selection
                    .as_ref()
                    .unwrap()
                    .remaining
                    .len(),
                20
            );
            assert!(g.observe(p.other()).library_selection.is_none());
            reject(&mut g, Action::SelectLibrary { card: top[1] });
            reject(&mut g, Action::SelectLibrary { card: top[20] });
            if take == 0 && p == Player::P0 {
                super::contracts::branch_contract(g.clone());
            }
            if take > 0 {
                act(&mut g, Action::SelectLibrary { card: top[0] });
            }
            if take > 1 {
                act(&mut g, Action::SelectLibrary { card: top[19] });
            }
            let mut expected = untouched
                .iter()
                .copied()
                .filter(|id| !((take > 0 && *id == top[0]) || (take > 1 && *id == top[19])))
                .collect::<Vec<_>>();
            let mut expected_rng = g.state.rng;
            expected_rng.shuffle(&mut expected);
            act(&mut g, Action::FinishLibrarySelection);
            settle(&mut g);
            assert_eq!(
                g.state.players[p.index()].library.to_vec(),
                expected,
                "shuffle includes cards below the look boundary"
            );
            assert_eq!(g.state.rng, expected_rng);
            assert_eq!(g.state.players[p.index()].life, 28);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert_eq!(g.state.players[p.index()].library.len(), 22 - take);
            assert_ne!(g.state.players[p.index()].library, untouched);
            for (id, selected) in [(top[0], take > 0), (top[19], take > 1)] {
                assert_eq!(
                    g.object_view(id).zone,
                    if selected {
                        Zone::Battlefield
                    } else {
                        Zone::Library
                    }
                );
                if selected {
                    assert!(g.object_view(id).tapped);
                }
            }
            assert_eq!(g.object_view(top[20]).zone, Zone::Library);
        }
    }
}
#[test]
fn forest_gate_empty_library_no_lands_and_all_lands_are_valid_choices() {
    for p in [Player::P0, Player::P1] {
        for names in [
            &[][..],
            &["Grizzly Bears", "Hill Giant"][..],
            &["Forest", "Mountain"][..],
        ] {
            let mut g = scenario(p);
            let top = library(&mut g, p, names);
            let spell = put(&mut g, p, "Through the Forest Gate", Zone::Hand);
            cast(&mut g, spell, None);
            if !top.is_empty() {
                until(&mut g, |g| {
                    g.legal_actions().contains(&Action::FinishLibrarySelection)
                });
                assert!(g.observe(p.other()).library_selection.is_none());
                for id in &top {
                    if g.state.objects[*id as usize].card.is_land() {
                        act(&mut g, Action::SelectLibrary { card: *id });
                    }
                }
                act(&mut g, Action::FinishLibrarySelection);
            }
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].life, 28);
            for id in top {
                if g.state.objects[id as usize].card.is_land() {
                    assert_eq!(g.object_view(id).zone, Zone::Battlefield);
                    assert!(g.object_view(id).tapped);
                } else {
                    assert_eq!(g.object_view(id).zone, Zone::Library);
                }
            }
        }
    }
}
#[test]
fn forest_gate_resumes_an_owned_entry_choice_before_shuffle_and_life_gain() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let top = library(
            &mut g,
            p,
            &[
                "Secluded Courtyard",
                "Forest",
                "Grizzly Bears",
                "Hill Giant",
            ],
        );
        let spell = put(&mut g, p, "Through the Forest Gate", Zone::Hand);
        cast(&mut g, spell, None);
        until(&mut g, |g| {
            g.legal_actions().contains(&Action::FinishLibrarySelection)
        });
        act(&mut g, Action::SelectLibrary { card: top[0] });
        act(&mut g, Action::SelectLibrary { card: top[1] });
        act(&mut g, Action::FinishLibrarySelection);
        assert_eq!(g.state.players[p.index()].life, 20);
        assert!(g.observe(p).entry_value.is_some());
        assert!(g.observe(p.other()).library_selection.is_none());
        super::contracts::branch_contract(g.clone());
        act(
            &mut g,
            Action::ChooseEntryValue {
                value: EntryChoiceValue::CreatureType(
                    crate::cards::CreatureType::from_name("Wizard").unwrap(),
                ),
            },
        );
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, 28);
        for id in &top[..2] {
            assert_eq!(g.object_view(*id).zone, Zone::Battlefield);
            assert!(g.object_view(*id).tapped);
        }
        assert_eq!(g.state.players[p.index()].library.len(), 2);
    }
}
#[test]
fn artifact_only_row_does_not_suppress_an_independent_crew_animation() {
    let p = Player::P0;
    let mut g = scenario(p);
    let vehicle = put(&mut g, p, "Cultivator's Caravan", Zone::Battlefield);
    let member = put(&mut g, p, "Hill Giant", Zone::Battlefield);
    act(
        &mut g,
        Action::AnnounceActivate {
            card: vehicle,
            ability: 1,
            counter_payment: Counters::default(),
        },
    );
    let object = g.object_ref(member);
    act(&mut g, Action::ChooseCrewer { object });
    act(&mut g, Action::FinishCrew);
    if g.state.announcing.is_some() {
        pay(&mut g);
    }
    settle(&mut g);
    let spell = put(&mut g, p, "Stone by Sunlight", Zone::Hand);
    modal(&mut g, spell, 1, Target::Card(vehicle));
    settle(&mut g);
    assert!(g.is_creature(vehicle));
    assert_eq!(stats(&g, vehicle), (5, 5));
    assert_eq!(g.object_view(vehicle).kinds, 17);
    g.finish_cleanup();
    assert!(!g.is_creature(vehicle));
    assert_eq!(g.object_view(vehicle).kinds, 16);
}

#[test]
fn origin_of_metalbending_modes_and_target_restrictions() {
    for p in [Player::P0, Player::P1] {
        for (name, mode) in [
            ("Murmuring Volume", 0),
            ("Phyrexian Arena", 0),
            ("Grizzly Bears", 1),
        ] {
            let mut g = scenario(p);
            let victim = put(&mut g, p, name, Zone::Battlefield);
            let opposing = put(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
            let spell = put(&mut g, p, "Origin of Metalbending", Zone::Hand);
            g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 1];
            act(&mut g, Action::AnnounceCast { card: spell });
            act(&mut g, Action::ChooseMode { mode });
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(opposing),
                },
            );
            bind(&mut g, Target::Card(victim));
            settle(&mut g);
            if mode == 0 {
                assert_eq!(g.object_view(victim).zone, Zone::Graveyard);
            } else {
                assert_eq!(stats(&g, victim), (3, 3));
                assert!(g.keywords(victim).contains(Keywords::INDESTRUCTIBLE));
                g.finish_cleanup();
                assert!(!g.keywords(victim).contains(Keywords::INDESTRUCTIBLE));
                assert_eq!(stats(&g, victim), (3, 3));
            }
        }
    }
}
