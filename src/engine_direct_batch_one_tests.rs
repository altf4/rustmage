//! Complete direct definitions exercised through paid actions and owned replay.
use super::*;

fn activate_on(g: &mut Game, source: CardId, target: CardId) {
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
    while g.state.announcing.is_some() {
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. } | Action::FinishTargets))
            .unwrap();
        checked(g, a);
    }
}
fn fixture_subtype(g: &mut Game, p: Player, subtype: &str, zone: Zone) -> CardId {
    let c = Card::catalogue()
        .find(|c| {
            c.is_supported()
                && !c.is_creature()
                && c.info().type_line.split_whitespace().any(|s| s == subtype)
        })
        .unwrap();
    spawn(g, p, c, zone)
}
#[test]
fn direct349_paid_etbs_targets_and_optional_recovery_both_seats() {
    for p in [Player::P0, Player::P1] {
        for (name, own, expected) in [
            ("Sunshower Druid", false, 1),
            ("Pileated Provisioner", true, 1),
        ] {
            let mut g = setup(p);
            let target = named(
                &mut g,
                if own { p } else { p.other() },
                "Bear Cub",
                Zone::Battlefield,
            );
            let id = named(&mut g, p, name, Zone::Hand);
            let mana = mana_sum(g.state.players[p.index()].mana);
            cast(&mut g, id, None, &[]);
            settle_on(&mut g, Some(Target::Card(target)));
            assert!(mana_sum(g.state.players[p.index()].mana) < mana);
            assert_eq!(g.state.objects[target as usize].counters.plus_one, expected);
            assert_eq!(
                g.state.players[p.index()].life,
                if name == "Sunshower Druid" { 21 } else { 20 }
            );
        }
        for count in 0..=3 {
            let mut g = setup(p);
            let ids: Vec<_> = (0..count)
                .map(|_| named(&mut g, p.other(), "Bear Cub", Zone::Battlefield))
                .collect();
            let id = named(&mut g, p, "Armaggon, Future Shark", Zone::Hand);
            cast(&mut g, id, None, &[]);
            // Finish the spell, then announce the real ETB's optional distinct targets.
            while g.state.pending == Pending::Priority && !g.state.stack.is_empty() {
                checked(&mut g, Action::Pass);
            }
            g.prepare_triggers();
            for target in &ids {
                checked(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(*target),
                    },
                );
            }
            settle(&mut g);
            for target in ids {
                assert_eq!(g.state.objects[target as usize].zone, Zone::Graveyard);
            }
            assert!(g.on_battlefield(id));
            assert!(g.keywords(id).contains(Keywords::FLASH));
        }
        for food in [false, true] {
            for decline in [false, true] {
                let mut g = setup(p);
                let target = if food {
                    fixture_subtype(&mut g, p, "Food", Zone::Graveyard)
                } else {
                    named(&mut g, p, "Bear Cub", Zone::Graveyard)
                };
                let id = named(&mut g, p, "Ragamuffin Raptor", Zone::Hand);
                cast(&mut g, id, None, &[]);
                settle_on(
                    &mut g,
                    if decline {
                        None
                    } else {
                        Some(Target::Card(target))
                    },
                );
                assert_eq!(
                    g.state.objects[target as usize].zone,
                    if decline { Zone::Graveyard } else { Zone::Hand }
                );
            }
        }
    }
}
#[test]
fn direct349_paid_activated_subtypes_haste_and_captain_both_seats() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let vehicle = fixture_subtype(&mut g, p.other(), "Vehicle", Zone::Battlefield);
        let id = named(&mut g, p, "Daring Mechanic", Zone::Hand);
        cast(&mut g, id, None, &[]);
        settle(&mut g);
        let before = mana_sum(g.state.players[p.index()].mana);
        activate_on(&mut g, id, vehicle);
        settle(&mut g);
        assert_eq!(before - mana_sum(g.state.players[p.index()].mana), 4);
        assert_eq!(g.state.objects[vehicle as usize].counters.plus_one, 1);
        let mut g = setup(p);
        let target = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let id = named(&mut g, p, "Hotfoot Gnome", Zone::Hand);
        cast(&mut g, id, None, &[]);
        settle(&mut g);
        activate_on(&mut g, id, target);
        settle(&mut g);
        assert!(g.state.objects[id as usize].tapped);
        assert!(g.keywords(target).contains(Keywords::HASTE));
        g.finish_cleanup();
        assert!(!g.keywords(target).contains(Keywords::HASTE));
        let mut g = setup(p);
        let captain = named(&mut g, p, "Captain Storm, Cosmium Raider", Zone::Hand);
        cast(&mut g, captain, None, &[]);
        settle(&mut g);
        let artifact = named(&mut g, p, "Hotfoot Gnome", Zone::Hand);
        cast(&mut g, artifact, None, &[]);
        settle_on(&mut g, Some(Target::Card(captain)));
        assert_eq!(g.state.objects[captain as usize].counters.plus_one, 1);
    }
}
#[test]
fn direct349_paid_spells_and_second_spell_trigger_both_seats() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let id = named(&mut g, p, "Oracle's Restoration", Zone::Hand);
        let hand = g.hand_count(p);
        cast(&mut g, id, None, &[Target::Card(bear)]);
        settle(&mut g);
        assert_eq!(g.stats(bear).power, 3);
        assert_eq!(g.hand_count(p), hand);
        assert_eq!(g.state.players[p.index()].life, 21);
        g.finish_cleanup();
        assert_eq!(g.stats(bear).power, 2);
        let mut g = setup(p);
        let bear = named(&mut g, p, "Bear Cub", Zone::Graveyard);
        let id = named(&mut g, p, "Helping Hand", Zone::Hand);
        cast(&mut g, id, None, &[Target::Card(bear)]);
        settle(&mut g);
        assert!(g.on_battlefield(bear));
        assert!(g.state.objects[bear as usize].tapped);
        for name in ["Kraum, Violent Cacophony", "Sunstar Lightsmith"] {
            let mut g = setup(p);
            let id = named(&mut g, p, name, Zone::Hand);
            cast(&mut g, id, None, &[]);
            settle(&mut g);
            let hand = g.hand_count(p);
            let second = named(&mut g, p, "Bear Cub", Zone::Hand);
            cast(&mut g, second, None, &[]);
            settle(&mut g);
            assert_eq!(g.state.objects[id as usize].counters.plus_one, 1);
            assert_eq!(g.hand_count(p), hand + 1);
            let third = named(&mut g, p, "Bear Cub", Zone::Hand);
            cast(&mut g, third, None, &[]);
            settle(&mut g);
            assert_eq!(g.state.objects[id as usize].counters.plus_one, 1);
        }
    }
}
#[test]
fn direct349_paid_combat_and_haiku_groups_both_seats() {
    for p in [Player::P0, Player::P1] {
        for name in ["Wraith, Vicious Vigilante", "Jumbo Cactuar"] {
            let mut g = setup(p);
            let id = named(&mut g, p, name, Zone::Hand);
            cast(&mut g, id, None, &[]);
            settle(&mut g);
            g.state.objects[id as usize].summoning_sick = false;
            g.state.turn = 2;
            g.state.step = Step::Attackers;
            g.state.pending = Pending::Attackers;
            checked(&mut g, Action::Attack { card: id });
            checked(&mut g, Action::FinishAttackers);
            settle(&mut g);
            if name == "Jumbo Cactuar" {
                assert_eq!(g.stats(id).power, 10000);
                g.finish_cleanup();
                assert_eq!(g.stats(id).power, 1);
            } else {
                assert!(
                    g.keywords(id)
                        .contains(Keywords::DOUBLE_STRIKE.union(Keywords::CANT_BE_BLOCKED))
                );
            }
        }
        let mut g = setup(p);
        let land = named(&mut g, p, "Island", Zone::Battlefield);
        g.state.objects[land as usize].tapped = true;
        let spell = named(&mut g, p, "Bear Cub", Zone::Hand);
        cast(&mut g, spell, None, &[]);
        let stack_id = spell;
        let haiku = named(&mut g, p, "Sokka's Haiku", Zone::Hand);
        let hand = g.hand_count(p);
        cast(
            &mut g,
            haiku,
            None,
            &[Target::Spell(stack_id), Target::Card(land)],
        );
        settle(&mut g);
        assert_eq!(g.state.objects[spell as usize].zone, Zone::Graveyard);
        assert!(!g.state.objects[land as usize].tapped);
        assert_eq!(g.hand_count(p), hand);
    }
}

#[test]
fn direct349_sole_target_fizzle_and_haiku_partial_resolution_both_seats() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let restoration = named(&mut g, p, "Oracle's Restoration", Zone::Hand);
        cast(&mut g, restoration, None, &[Target::Card(bear)]);
        let bounce = named(&mut g, p, "Unsummon", Zone::Hand);
        cast(&mut g, bounce, None, &[Target::Card(bear)]);
        let hand = g.hand_count(p);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, 20);
        assert_eq!(g.hand_count(p), hand + 1); // only the returned creature; no draw
        assert_eq!(g.state.objects[bear as usize].zone, Zone::Hand);

        for invalidate_spell in [false, true] {
            let mut g = setup(p);
            let land = named(&mut g, p, "Evolving Wilds", Zone::Battlefield);
            g.state.objects[land as usize].tapped = true;
            let original = named(&mut g, p, "Bear Cub", Zone::Hand);
            cast(&mut g, original, None, &[]);
            let haiku = named(&mut g, p, "Sokka's Haiku", Zone::Hand);
            cast(
                &mut g,
                haiku,
                None,
                &[Target::Spell(original), Target::Card(land)],
            );
            if invalidate_spell {
                let cancel = named(&mut g, p, "Cancel", Zone::Hand);
                cast(&mut g, cancel, None, &[Target::Spell(original)]);
            } else {
                g.state.objects[land as usize].tapped = false;
                checked(
                    &mut g,
                    Action::AnnounceActivate {
                        card: land,
                        ability: 0,
                        counter_payment: Counters::default(),
                    },
                );
                while g.state.announcing.is_some() {
                    let a = g
                        .legal_actions()
                        .into_iter()
                        .find(|a| matches!(a, Action::PayCosts { .. } | Action::FinishTargets))
                        .unwrap();
                    checked(&mut g, a);
                }
            }
            let hand = g.hand_count(p);
            let gy = g
                .state
                .objects
                .iter()
                .filter(|o| o.owner == p && o.zone == Zone::Graveyard)
                .count();
            settle(&mut g);
            assert_eq!(g.hand_count(p), hand + 1);
            assert!(
                g.state
                    .objects
                    .iter()
                    .filter(|o| o.owner == p && o.zone == Zone::Graveyard)
                    .count()
                    >= gy + 5
            ); // mill3 and at least two resolved spells
            assert_eq!(
                g.state.objects[land as usize].zone,
                if invalidate_spell {
                    Zone::Battlefield
                } else {
                    Zone::Graveyard
                }
            );
            if invalidate_spell {
                assert!(!g.state.objects[land as usize].tapped);
            }
        }
    }
}

#[test]
fn direct349_activation_and_etb_target_legality_both_seats() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let own = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let source = named(&mut g, p, "Hotfoot Gnome", Zone::Hand);
        cast(&mut g, source, None, &[]);
        settle(&mut g);
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: source,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(source),
            },
        );
        assert!(g.legal_actions().contains(&Action::ChooseTarget {
            target: Target::Card(own)
        }));
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        settle(&mut g);
        assert!(g.keywords(enemy).contains(Keywords::HASTE));
        let mut g = setup(p);
        let flying = named(&mut g, p, "Kraum, Violent Cacophony", Zone::Battlefield);
        let own = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let source = named(&mut g, p, "Pileated Provisioner", Zone::Hand);
        cast(&mut g, source, None, &[]);
        while g.state.pending == Pending::Priority && !g.state.stack.is_empty() {
            checked(&mut g, Action::Pass);
        }
        g.prepare_triggers();
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(flying),
            },
        );
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(source),
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
                target: Target::Card(own),
            },
        );
        settle(&mut g);
        assert_eq!(g.state.objects[own as usize].counters.plus_one, 1);
    }
}

#[test]
fn direct349_departed_sources_keep_historical_controller_and_exact_targets() {
    for p in [Player::P0, Player::P1] {
        for name in ["Kraum, Violent Cacophony", "Sunstar Lightsmith"] {
            let mut g = setup(p);
            let source = named(&mut g, p, name, Zone::Hand);
            cast(&mut g, source, None, &[]);
            settle(&mut g);
            let second = named(&mut g, p, "Bear Cub", Zone::Hand);
            cast(&mut g, second, None, &[]);
            let bounce = named(&mut g, p, "Unsummon", Zone::Hand);
            cast(&mut g, bounce, None, &[Target::Card(source)]);
            let hand = g.hand_count(p);
            settle(&mut g);
            assert_eq!(g.hand_count(p), hand + 2);
            assert_eq!(g.state.objects[source as usize].zone, Zone::Hand);
            assert_eq!(g.state.objects[source as usize].counters.plus_one, 0);
        }
        let mut g = setup(p);
        let target = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let source = named(&mut g, p, "Sunshower Druid", Zone::Hand);
        cast(&mut g, source, None, &[]);
        while g.state.pending == Pending::Priority && !g.state.stack.is_empty() {
            checked(&mut g, Action::Pass);
        }
        g.prepare_triggers();
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        while g.state.pending != Pending::Priority {
            let a = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::FinishTargets))
                .unwrap();
            checked(&mut g, a);
        }
        let bounce = named(&mut g, p, "Unsummon", Zone::Hand);
        cast(&mut g, bounce, None, &[Target::Card(source)]);
        settle(&mut g);
        assert_eq!(g.state.objects[source as usize].zone, Zone::Hand);
        assert_eq!(g.state.objects[target as usize].counters.plus_one, 1);
        assert_eq!(g.state.players[p.index()].life, 21);
    }
}
