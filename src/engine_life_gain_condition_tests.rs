use super::tests::{act, cast, scenario, spawn};
use super::*;
fn named(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    let c = Card::from_name(name).unwrap();
    assert!(c.is_supported(), "{name}");
    spawn(g, p, c, zone)
}
fn replay(g: &mut Game, a: Action) {
    let saved = g.snapshot();
    let revision = g.revision();
    let p = g.actor().unwrap();
    let mut f = g.clone();
    act(&mut f, a.clone());
    act(g, a.clone());
    let e = g.snapshot();
    assert_eq!(f.snapshot(), e);
    g.restore(&saved).unwrap();
    assert_eq!(g.apply_index(p, revision, 0), Err(Error::StaleRevision));
    act(g, a);
    assert_eq!(g.snapshot(), e);
}
fn settle(g: &mut Game) {
    for _ in 0..200 {
        g.prepare_triggers();
        if g.state.pending == Pending::Priority && g.state.stack.is_empty() {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| match a {
                Action::Pass | Action::PutTrigger { .. } | Action::ArrangeLibrary { .. } => true,
                Action::ChooseTarget {
                    target: Target::Card(id),
                } => g.object_view(*id).owner != g.actor().unwrap(),
                Action::ChooseOptional { accept: true } => true,
                _ => false,
            })
            .expect("fixture choice");
        replay(g, a);
    }
    panic!("did not settle");
}
#[test]
fn continuous_bonuses_use_positive_gain_history_not_net_life_or_signed_conversion() {
    let mut g = scenario();
    let t = named(&mut g, Player::P0, "Tenured Concocter", Zone::Battlefield);
    let u = named(&mut g, Player::P0, "Ulna Alley Shopkeep", Zone::Battlefield);
    let s = named(&mut g, Player::P0, "Thornfist Striker", Zone::Battlefield);
    let enemy = named(&mut g, Player::P1, "Thornfist Striker", Zone::Battlefield);
    assert_eq!(g.stats(t).power, 4);
    assert_eq!(g.stats(u).power, 2);
    assert_eq!(g.stats(s).power, 3);
    assert!(!g.keywords(t).contains(Keywords::TRAMPLE));
    g.gain_life(Player::P0, 0);
    g.gain_life(Player::P0, -5);
    assert_eq!(g.stats(t).power, 4);
    g.gain_life(Player::P1, 2);
    assert_eq!(g.stats(enemy).power, 4);
    assert_eq!(g.stats(t).power, 4);
    g.gain_life(Player::P0, 1);
    g.lose_life(Player::P0, 10);
    assert_eq!(g.stats(t).power, 7);
    assert_eq!(g.stats(u).power, 5);
    assert_eq!(g.stats(s).power, 4);
    assert!(g.keywords(t).contains(Keywords::TRAMPLE));
    g.state.players[0].life_gained_this_turn = u128::MAX;
    let saved = g.snapshot();
    assert_eq!(g.stats(t).power, 7);
    assert_eq!(g.snapshot(), saved);
    g.move_object(s, Zone::Graveyard);
    assert_eq!(g.stats(t).power, 6);
    assert!(!g.keywords(t).contains(Keywords::TRAMPLE));
    g.move_object(s, Zone::Battlefield);
    assert_eq!(g.stats(t).power, 7);
    g.finish_cleanup();
    assert_eq!(g.stats(t).power, 4);
    assert_eq!(g.stats(u).power, 2);
    assert_eq!(g.stats(enemy).power, 3);
    assert!(!g.keywords(t).contains(Keywords::TRAMPLE));
}
#[test]
fn conditional_entry_effects_choose_targets_without_gain_and_check_history_at_resolution() {
    for gained in [false, true] {
        let mut g = scenario();
        let old = named(&mut g, Player::P0, "Old-Growth Educator", Zone::Hand);
        g.move_object(old, Zone::Battlefield);
        assert_eq!(g.object_view(old).counters.plus_one, 0);
        assert_eq!(g.state.pending_triggers.len(), 1);
        if gained {
            g.gain_life(Player::P0, 1);
        }
        settle(&mut g);
        assert_eq!(
            g.object_view(old).counters.plus_one,
            if gained { 2 } else { 0 }
        );
        assert!(g.keywords(old).contains(Keywords::REACH));
        assert!(g.keywords(old).contains(Keywords::VIGILANCE));
        let enemy = named(&mut g, Player::P1, "Hill Giant", Zone::Battlefield);
        let p = named(&mut g, Player::P0, "Poisoner's Apprentice", Zone::Hand);
        g.move_object(p, Zone::Battlefield);
        g.prepare_triggers();
        while let Some(a) = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PutTrigger { .. }))
        {
            replay(&mut g, a);
        }
        assert_eq!(g.observe(Player::P0).decision, "targets");
        let saved = g.snapshot();
        let rev = g.revision();
        assert_eq!(
            g.apply(
                Player::P0,
                Action::ChooseTarget {
                    target: Target::Card(old)
                }
            ),
            Err(Error::IllegalAction)
        );
        assert_eq!(g.snapshot(), saved);
        assert_eq!(g.revision(), rev);
        replay(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        settle(&mut g);
        assert_eq!(
            g.object_view(enemy).zone,
            if gained {
                Zone::Graveyard
            } else {
                Zone::Battlefield
            }
        );
    }
}
#[test]
fn apprentice_can_gain_life_after_target_declaration_and_before_its_trigger_resolves() {
    let mut g = scenario();
    let enemy = named(&mut g, Player::P1, "Hill Giant", Zone::Battlefield);
    let p = named(&mut g, Player::P0, "Poisoner's Apprentice", Zone::Hand);
    g.move_object(p, Zone::Battlefield);
    g.prepare_triggers();
    replay(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(enemy),
        },
    );
    let gain = named(&mut g, Player::P0, "Moment of Triumph", Zone::Hand);
    g.state.players[0].mana = [10; 6];
    cast(&mut g, gain, Some(Target::Card(p)));
    settle(&mut g);
    assert_eq!(g.object_view(enemy).zone, Zone::Graveyard);
    assert_eq!(g.state.players[0].life_gained_this_turn, 2);
}
#[test]
fn efflorescence_always_adds_counters_but_only_grants_keywords_on_resolution_history() {
    for late_gain in [false, true] {
        let mut g = scenario();
        let bear = named(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
        let spell = named(&mut g, Player::P0, "Efflorescence", Zone::Hand);
        g.state.players[0].mana = [10; 6];
        cast(&mut g, spell, Some(Target::Card(bear)));
        if late_gain {
            let gain = named(&mut g, Player::P0, "Moment of Triumph", Zone::Hand);
            cast(&mut g, gain, Some(Target::Card(bear)));
        }
        settle(&mut g);
        assert_eq!(g.object_view(bear).counters.plus_one, 2);
        assert_eq!(g.keywords(bear).contains(Keywords::TRAMPLE), late_gain);
        assert_eq!(
            g.keywords(bear).contains(Keywords::INDESTRUCTIBLE),
            late_gain
        );
        g.finish_cleanup();
        assert_eq!(g.object_view(bear).counters.plus_one, 2);
        assert!(!g.keywords(bear).contains(Keywords::INDESTRUCTIBLE));
    }
}
#[test]
fn withering_curse_chooses_exactly_one_branch_and_indestructible_distinguishes_them() {
    for gain in [false, true] {
        let mut g = scenario();
        let bear = named(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield);
        g.state.objects[bear as usize].gained_keywords = Keywords::INDESTRUCTIBLE;
        let giant = named(&mut g, Player::P1, "Hill Giant", Zone::Battlefield);
        let spell = named(&mut g, Player::P0, "Withering Curse", Zone::Hand);
        g.state.players[0].mana = [10; 6];
        cast(&mut g, spell, None);
        if gain {
            let life = named(&mut g, Player::P0, "Moment of Triumph", Zone::Hand);
            cast(&mut g, life, Some(Target::Card(giant)));
        } else {
            g.gain_life(Player::P1, 1);
        }
        settle(&mut g);
        assert_eq!(
            g.object_view(bear).zone,
            if gain {
                Zone::Battlefield
            } else {
                Zone::Graveyard
            }
        );
        assert_eq!(
            g.object_view(giant).zone,
            if gain {
                Zone::Graveyard
            } else {
                Zone::Battlefield
            }
        );
        if gain {
            assert_eq!(
                g.stats(bear),
                Stats {
                    power: 2,
                    toughness: 2
                }
            );
        }
        if !gain {
            assert_eq!(
                g.stats(giant),
                Stats {
                    power: 1,
                    toughness: 1
                }
            );
        }
    }
}
#[test]
fn six_other_whole_bodies_keep_opponent_only_debuffs_and_source_death_identity() {
    for (name, p, t, flash) in [
        ("Ambush Gigapede", -2, -2, true),
        ("Burrog Befuddler", -1, 0, true),
        ("Cogwork Wrestler", -2, 0, true),
        ("Front Porch Sentries", -1, -1, false),
        ("Grim Bauble", -2, -2, false),
        ("Humbling Elder", -2, 0, true),
    ] {
        let mut g = scenario();
        let enemy = named(&mut g, Player::P1, "Hill Giant", Zone::Battlefield);
        let source = named(&mut g, Player::P0, name, Zone::Hand);
        g.move_object(source, Zone::Battlefield);
        if name == "Front Porch Sentries" {
            g.move_object(source, Zone::Graveyard);
        }
        g.prepare_triggers();
        while let Some(a) = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PutTrigger { .. }))
        {
            replay(&mut g, a);
        }
        assert!(g.legal_actions().contains(&Action::ChooseTarget {
            target: Target::Card(enemy)
        }));
        assert!(!g.legal_actions().contains(&Action::ChooseTarget {
            target: Target::Card(source)
        }));
        replay(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        settle(&mut g);
        assert_eq!(
            g.stats(enemy),
            Stats {
                power: 3 + p,
                toughness: 3 + t
            }
        );
        assert_eq!(g.keywords(source).contains(Keywords::FLASH), flash);
        g.finish_cleanup();
        assert_eq!(
            g.stats(enemy),
            Stats {
                power: 3,
                toughness: 3
            }
        );
    }
}
#[test]
fn grim_bauble_sacrifice_survives_in_private_surveil_continuation() {
    let mut g = scenario();
    let a = named(&mut g, Player::P0, "Grim Bauble", Zone::Battlefield);
    g.state.players[0].mana = [10; 6];
    let action = g
        .legal_actions()
        .into_iter()
        .find(|x| matches!(x,Action::Activate{card,..}if *card==a))
        .unwrap();
    replay(&mut g, action);
    assert_eq!(g.object_view(a).zone, Zone::Graveyard);
    replay(&mut g, Action::Pass);
    replay(&mut g, Action::Pass);
    assert_eq!(g.observe(Player::P0).decision, "library_choice");
    assert!(g.observe(Player::P1).library_choice.is_none());
    settle(&mut g);
}
#[test]
fn tenured_optional_opponent_target_draw_coexists_with_live_history_bonus() {
    let mut g = scenario();
    let a = named(&mut g, Player::P1, "Tenured Concocter", Zone::Battlefield);
    let spell = named(&mut g, Player::P0, "Giant Growth", Zone::Hand);
    g.state.players[0].mana = [10; 6];
    cast(&mut g, spell, Some(Target::Card(a)));
    assert_eq!(g.state.stack.len(), 2);
    g.gain_life(Player::P1, 2);
    settle(&mut g);
    assert_eq!(g.state.players[1].cards_drawn_this_turn, 1);
    assert_eq!(g.stats(a).power, 9);
}

#[test]
fn gain_only_suspension_cannot_commit_unrepresentable_continuous_stats() {
    for global in [false, true] {
        for net_zero in [false, true] {
            let mut g = scenario();
            let target = if global {
                named(&mut g, Player::P0, "Thornfist Striker", Zone::Battlefield);
                named(&mut g, Player::P0, "Grizzly Bears", Zone::Battlefield)
            } else {
                named(&mut g, Player::P0, "Tenured Concocter", Zone::Battlefield)
            };
            g.state.objects[target as usize].growth.power = i128::MAX - if global { 2 } else { 4 };
            assert_eq!(g.stats(target).power, i128::MAX);
            let beavers = named(&mut g, Player::P0, "Dream Beavers", Zone::Hand);
            g.move_object(beavers, Zone::Battlefield);
            g.prepare_triggers();
            assert_eq!(
                g.state.stack.last().unwrap().source_card.name(),
                "Dream Beavers"
            );
            let action = if net_zero {
                // Synthetic owned IR isolates history dirtiness when every
                // life total is unchanged; no printed-body admission is implied.
                let item = g.state.stack.pop().unwrap();
                g.state.resolving = Some(Resolution {
                    explore: None,
                    evidence: None,
                    item,
                    program: &[
                        Instruction::May { skip: 3 },
                        Instruction::LoseLife(PlayerSet::Controller, 1),
                        Instruction::GainLife(PlayerSet::Controller, 1),
                        Instruction::Scry(1),
                    ],
                    pc: 0,
                    target_group: 0,
                    moved_count: None,
                    connive: None,
                    mana_payment: None,
                    forage: None,
                    discards: [0; 2],
                    selected_discards: [Vec::new(), Vec::new()],
                    milled: Vec::new(),
                    milled_selection: None,
                    library: None,
                    selection: None,
                    target_library: None,
                    search: None,
                    choice: None,
                });
                g.run_program();
                Action::ChooseOptional { accept: true }
            } else {
                replay(&mut g, Action::Pass);
                Action::Pass
            };
            let saved = g.snapshot();
            let revision = g.revision();
            let p = g.actor().unwrap();
            let mut fork = g.clone();
            assert!(matches!(
                fork.apply(p, action.clone()),
                Err(Error::CapacityExceeded(_))
            ));
            assert_eq!(fork.snapshot(), saved);
            for _ in 0..2 {
                assert!(matches!(
                    g.apply(p, action.clone()),
                    Err(Error::CapacityExceeded(_))
                ));
                assert_eq!(g.snapshot(), saved);
                assert_eq!(g.revision(), revision);
                assert_eq!(g.observe(Player::P0).players[0].life_gained_this_turn, 0);
                assert_eq!(g.stats(target).power, i128::MAX);
            }
            g.restore(&saved).unwrap();
            assert!(matches!(
                g.apply(p, action),
                Err(Error::CapacityExceeded(_))
            ));
            assert_eq!(g.snapshot(), saved);
        }
    }
}
