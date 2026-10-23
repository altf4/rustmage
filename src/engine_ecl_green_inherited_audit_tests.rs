//! Transferred inherited green/land audit: paid actions and owned branch checks.
use super::*;
use crate::cards::AnimationDuration;

fn until(g: &mut Game, stop: impl Fn(&Game) -> bool) {
    g.prepare_triggers();
    for _ in 0..100 {
        if stop(g) {
            return;
        }
        let actions = g.legal_actions();
        let action = if g.state.pending == Pending::Priority {
            Action::Pass
        } else {
            actions
                .into_iter()
                .find(|a| !matches!(a, Action::Concede))
                .expect("audit continuation")
        };
        checked(g, action);
    }
    panic!("green audit continuation {:?}", g.state.pending);
}
fn surveil(g: &mut Game, owner: Player, graveyard: bool) -> usize {
    g.prepare_triggers();
    let mut count = 0;
    for _ in 0..100 {
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty()
            && g.state.resolving.is_none()
        {
            return count;
        }
        let actor = g.actor().unwrap();
        let action = if let Some(choice) = g.observe(actor).library_choice {
            assert_eq!(actor, owner);
            assert!(g.observe(owner.other()).library_choice.is_none());
            count += 1;
            Action::ArrangeLibrary {
                card: choice.remaining[0].reference.card,
                destination: if graveyard {
                    LibraryDestination::Graveyard
                } else {
                    LibraryDestination::Top
                },
            }
        } else if g.state.pending == Pending::Priority {
            Action::Pass
        } else {
            g.legal_actions()
                .into_iter()
                .find(|a| !matches!(a, Action::Concede))
                .unwrap()
        };
        checked(g, action);
    }
    panic!("surveil audit did not settle");
}
fn mana_action(g: &Game, id: CardId, production: Mana) -> Action {
    g.legal_actions().into_iter().find(|a|matches!(a,Action::ActivateMana{card,production:actual,..}if *card==id&&*actual==production)).unwrap()
}
#[test]
fn inherited_green_blossoming_defense_own_target_counteracts_bolt_and_expires() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let own = spawn(&mut g, p, Card::GrizzlyBears, Zone::Battlefield);
        let enemy = spawn(&mut g, p.other(), Card::HillGiant, Zone::Battlefield);
        take_test_priority(&mut g, p.other());
        let bolt = spell(&mut g, p.other(), "Lightning Bolt", &[Target::Card(own)]);
        take_test_priority(&mut g, p);
        let id = spawn(
            &mut g,
            p,
            Card::from_name("Blossoming Defense").unwrap(),
            Zone::Hand,
        );
        g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 0];
        checked(&mut g, Action::AnnounceCast { card: id });
        reject_atomic(
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
        finish_cast_choices(&mut g);
        settle(&mut g);
        assert_eq!(g.stats(own), Stats::from(4));
        assert_eq!(g.object_view(own).damage, 0);
        assert!(g.keywords(own).contains(Keywords::HEXPROOF));
        assert_eq!(g.object_view(bolt).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        super::super::ability_timeline_tests::next_turn(&mut g);
        assert_eq!(g.stats(own), Stats::from(2));
        assert!(!g.keywords(own).contains(Keywords::HEXPROOF));
    }
}
#[test]
fn inherited_green_gilt_leaf_flash_enemy_attachment_lki_indestructible_and_expiry() {
    for p in [Player::P0, Player::P1] {
        for leave in [false, true] {
            let mut g = setup(p);
            let target = spawn(&mut g, p.other(), Card::GrizzlyBears, Zone::Battlefield);
            g.state.active = p.other();
            g.state.step = Step::BeginCombat;
            g.state.priority = p;
            let aura = spell(&mut g, p, "Gilt-Leaf's Embrace", &[Target::Card(target)]);
            checked(&mut g, Action::Pass);
            checked(&mut g, Action::Pass);
            assert_eq!(
                g.stats(target),
                Stats {
                    power: 4,
                    toughness: 2
                }
            );
            if leave {
                g.move_object(aura, Zone::Graveyard);
            }
            settle(&mut g);
            assert!(
                g.keywords(target)
                    .contains(Keywords::TRAMPLE.union(Keywords::INDESTRUCTIBLE))
            );
            take_test_priority(&mut g, p);
            spell(&mut g, p, "Lightning Bolt", &[Target::Card(target)]);
            settle(&mut g);
            assert_eq!(g.object_view(target).zone, Zone::Battlefield);
            assert_eq!(g.object_view(target).damage, 3);
            super::super::ability_timeline_tests::next_turn(&mut g);
            assert!(!g.keywords(target).contains(Keywords::INDESTRUCTIBLE));
            assert!(!g.keywords(target).contains(Keywords::TRAMPLE));
            assert_eq!(
                g.stats(target),
                if leave {
                    Stats::from(2)
                } else {
                    Stats {
                        power: 4,
                        toughness: 2,
                    }
                }
            );
            assert_eq!(g.object_view(target).damage, 0);
        }
    }
}
#[test]
fn inherited_green_crossroads_and_virulent_count_each_other_owned_entry_and_capture_controller() {
    for p in [Player::P0, Player::P1] {
        for leave in [false, true] {
            let mut g = setup(p);
            let watcher = cast_named(&mut g, p, "Crossroads Watcher");
            assert_eq!(g.stats(watcher), Stats::from(3));
            assert!(g.keywords(watcher).contains(Keywords::TRAMPLE));
            let emissary = cast_named(&mut g, p, "Virulent Emissary");
            assert_eq!(
                g.stats(watcher),
                Stats {
                    power: 4,
                    toughness: 3
                }
            );
            assert_eq!(g.stats(emissary), Stats::from(1));
            assert!(g.keywords(emissary).contains(Keywords::DEATHTOUCH));
            assert_eq!(g.state.players[p.index()].life, 20);
            spell(&mut g, p, "Clachan Festival", &[]);
            until(&mut g, |g| {
                g.state
                    .objects
                    .iter()
                    .filter(|o| {
                        o.zone == Zone::Battlefield
                            && o.controller == p
                            && o.card == Card::from_name("@token/c48e9e904360e4cee1deeed8").unwrap()
                    })
                    .count()
                    == 2
            });
            if leave {
                g.move_object(watcher, Zone::Hand);
                g.change_controller(g.object_ref(emissary), p.other());
            }
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].life, 22);
            assert_eq!(g.state.players[p.other().index()].life, 20);
            if !leave {
                assert_eq!(
                    g.stats(watcher),
                    Stats {
                        power: 6,
                        toughness: 3
                    }
                );
                take_test_priority(&mut g, p.other());
                spell(&mut g, p.other(), "High Fae Trickster", &[]);
                settle(&mut g);
                assert_eq!(g.stats(watcher).power, 6);
                assert_eq!(g.state.players[p.index()].life, 22);
                super::super::ability_timeline_tests::next_turn(&mut g);
                assert_eq!(g.stats(watcher), Stats::from(3));
            }
        }
    }
}
#[test]
fn inherited_green_dawns_light_archer_flash_reach_blocks_actual_flying_attacker() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p.other());
        let angel = spawn(
            &mut g,
            p.other(),
            Card::from_name("Serra Angel").unwrap(),
            Zone::Battlefield,
        );
        thoughtweft_attack(&mut g, &[angel]);
        take_test_priority(&mut g, p);
        let archer = spell(&mut g, p, "Dawn's Light Archer", &[]);
        settle(&mut g);
        assert_eq!(
            g.stats(archer),
            Stats {
                power: 4,
                toughness: 2
            }
        );
        assert!(g.keywords(archer).contains(Keywords::REACH));
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        checked(
            &mut g,
            Action::Block {
                blocker: archer,
                attacker: angel,
            },
        );
        checked(&mut g, Action::FinishBlockers);
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        assert_eq!(g.object_view(archer).zone, Zone::Graveyard);
        assert_eq!(g.object_view(angel).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].life, 20);
    }
}
#[test]
fn inherited_green_dundoolin_three_creatures_intervening_if_and_own_permanent_target() {
    for p in [Player::P0, Player::P1] {
        for eligible in [false, true] {
            for lose in [false, true] {
                let mut g = setup(p);
                let a = spawn(&mut g, p, Card::GrizzlyBears, Zone::Battlefield);
                if eligible {
                    spawn(&mut g, p, Card::GrizzlyBears, Zone::Battlefield);
                }
                let land = spawn(&mut g, p, Card::Forest, Zone::Graveyard);
                let instant = spawn(&mut g, p, Card::LightningBolt, Zone::Graveyard);
                let enemy = spawn(&mut g, p.other(), Card::GrizzlyBears, Zone::Graveyard);
                let source = spell(&mut g, p, "Dundoolin Weaver", &[]);
                if eligible {
                    until(&mut g, |g| {
                        matches!(g.state.pending, Pending::Targets { .. })
                    });
                    reject_atomic(
                        &mut g,
                        Action::ChooseTarget {
                            target: Target::Card(instant),
                        },
                    );
                    reject_atomic(
                        &mut g,
                        Action::ChooseTarget {
                            target: Target::Card(enemy),
                        },
                    );
                    checked(
                        &mut g,
                        Action::ChooseTarget {
                            target: Target::Card(land),
                        },
                    );
                    if lose {
                        g.move_object(a, Zone::Hand);
                    }
                }
                settle(&mut g);
                assert_eq!(
                    g.object_view(land).zone,
                    if eligible && !lose {
                        Zone::Hand
                    } else {
                        Zone::Graveyard
                    }
                );
                assert_eq!(
                    g.stats(source),
                    Stats {
                        power: 2,
                        toughness: 1
                    }
                );
            }
        }
    }
}
#[test]
fn inherited_green_great_forest_druid_real_sickness_and_five_colors_without_stack() {
    for p in [Player::P0, Player::P1] {
        for color in 0..5 {
            let mut g = setup(p);
            let druid = cast_named(&mut g, p, "Great Forest Druid");
            assert_eq!(
                g.stats(druid),
                Stats {
                    power: 0,
                    toughness: 4
                }
            );
            assert!(
                !g.legal_actions()
                    .iter()
                    .any(|a| matches!(a,Action::ActivateMana{card,..}if *card==druid))
            );
            super::super::ability_timeline_tests::next_turn(&mut g);
            super::super::ability_timeline_tests::next_turn(&mut g);
            g.state.step = Step::Main1;
            g.state.priority = p;
            g.state.players[p.index()].mana = [0; 6];
            let choices = g
                .legal_actions()
                .into_iter()
                .filter(|a| matches!(a,Action::ActivateMana{card,..}if *card==druid))
                .collect::<Vec<_>>();
            assert_eq!(choices.len(), 5);
            reject_atomic(
                &mut g,
                Action::ActivateMana {
                    card: druid,
                    ability: 0,
                    payment: [0; 6],
                    counter_payment: Counters::default(),
                    production: [0, 0, 0, 0, 0, 1],
                },
            );
            let mut production = [0; 6];
            production[color] = 1;
            let action = mana_action(&g, druid, production);
            checked(&mut g, action);
            assert_eq!(g.state.players[p.index()].mana, production);
            assert!(g.object_view(druid).tapped);
            assert!(g.state.stack.is_empty());
            assert_eq!(g.actor(), Some(p));
        }
    }
}
#[test]
fn inherited_green_luminollusk_vivid_uses_resolution_controller_permanents_not_opponents_colors() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        spawn(
            &mut g,
            p,
            Card::from_name("@token/c48e9e904360e4cee1deeed8").unwrap(),
            Zone::Battlefield,
        );
        let faerie = spawn(
            &mut g,
            p,
            Card::from_name("@token/91416e259800bff2445a15f5").unwrap(),
            Zone::Battlefield,
        );
        spawn(&mut g, p.other(), Card::HillGiant, Zone::Battlefield);
        let source = spell(&mut g, p, "Luminollusk", &[]);
        until(&mut g, |g| {
            g.state
                .stack
                .iter()
                .any(|s| s.card == source && matches!(s.kind, StackKind::TriggeredAbility { .. }))
        });
        g.move_object(faerie, Zone::Hand);
        g.change_controller(g.object_ref(source), p.other());
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, 22);
        assert_eq!(g.state.players[p.other().index()].life, 20);
        assert_eq!(
            g.stats(source),
            Stats {
                power: 2,
                toughness: 4
            }
        );
        assert!(g.keywords(source).contains(Keywords::DEATHTOUCH));
    }
}
#[test]
fn inherited_green_lys_alana_entry_and_death_surveil_private_for_each_event_controller() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = spell(&mut g, p, "Lys Alana Informant", &[]);
        assert_eq!(surveil(&mut g, p, true), 1);
        assert_eq!(g.state.players[p.index()].library.len(), 19);
        assert_eq!(
            g.stats(source),
            Stats {
                power: 3,
                toughness: 1
            }
        );
        g.change_controller(g.object_ref(source), p.other());
        take_test_priority(&mut g, p);
        spell(&mut g, p, "Lightning Bolt", &[Target::Card(source)]);
        assert_eq!(surveil(&mut g, p.other(), false), 1);
        assert_eq!(g.object_view(source).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.other().index()].library.len(), 20);
    }
}
#[test]
fn inherited_green_midnight_tilling_only_exact_four_milled_permanents_and_optional_decline() {
    for p in [Player::P0, Player::P1] {
        for recover in [false, true] {
            let mut g = setup(p);
            let old = spawn(&mut g, p, Card::GrizzlyBears, Zone::Graveyard);
            let land = spawn(&mut g, p, Card::Forest, Zone::Library);
            let instant = spawn(&mut g, p, Card::LightningBolt, Zone::Library);
            spawn(&mut g, p, Card::GrizzlyBears, Zone::Library);
            spawn(
                &mut g,
                p,
                Card::from_name("Boggart Mischief").unwrap(),
                Zone::Library,
            );
            spell(&mut g, p, "Midnight Tilling", &[]);
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::RecoverMilled { .. })
            });
            let view = g.observe(p).recovery_choice.unwrap();
            assert_eq!(view.milled.len(), 4);
            assert_eq!(view.candidates.len(), 3);
            assert_eq!(
                view.candidates,
                g.observe(p.other()).recovery_choice.unwrap().candidates
            );
            for card in [old, instant] {
                let object = g.object_ref(card);
                reject_atomic(
                    &mut g,
                    Action::RecoverMilled {
                        object: Some(object),
                    },
                );
            }
            let object = recover.then(|| g.object_ref(land));
            checked(&mut g, Action::RecoverMilled { object });
            settle(&mut g);
            assert_eq!(
                g.object_view(land).zone,
                if recover { Zone::Hand } else { Zone::Graveyard }
            );
            assert_eq!(g.state.players[p.index()].library.len(), 20);
            if recover {
                assert!(
                    g.observe(p.other())
                        .objects
                        .iter()
                        .any(|o| o.reference.card == land && o.zone == Zone::Hand)
                );
            }
        }
    }
}
#[test]
fn inherited_green_mistmeadow_kithkin_kindred_and_wildvine_five_colors_reduce_only_generic() {
    for p in [Player::P0, Player::P1] {
        for reduction in [false, true] {
            let mut g = setup(p);
            if reduction {
                spawn(
                    &mut g,
                    p,
                    Card::from_name("Clachan Festival").unwrap(),
                    Zone::Battlefield,
                );
            } else {
                spawn(
                    &mut g,
                    p.other(),
                    Card::from_name("@token/c48e9e904360e4cee1deeed8").unwrap(),
                    Zone::Battlefield,
                );
            }
            let id = spawn(
                &mut g,
                p,
                Card::from_name("Mistmeadow Council").unwrap(),
                Zone::Hand,
            );
            g.state.players[p.index()].mana = [0, 0, 0, 0, 1, if reduction { 2 } else { 3 }];
            reject_atomic(&mut g, Action::AnnounceCast { card: id });
            g.state.players[p.index()].mana[5] += 1;
            super::super::ability_timeline_tests::cast(&mut g, id, None, &[]);
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert_eq!(
                g.stats(id),
                Stats {
                    power: 4,
                    toughness: 3
                }
            );
            assert_eq!(g.hand_count(p), 1);
            let mut g = setup(p);
            let owner = if reduction { p } else { p.other() };
            for name in [
                "@token/c48e9e904360e4cee1deeed8",
                "@token/91416e259800bff2445a15f5",
                "Hill Giant",
            ] {
                spawn(
                    &mut g,
                    owner,
                    Card::from_name(name).unwrap(),
                    Zone::Battlefield,
                );
            }
            let id = spawn(
                &mut g,
                p,
                Card::from_name("Wildvine Pummeler").unwrap(),
                Zone::Hand,
            );
            g.state.players[p.index()].mana = [0, 0, 0, 0, 1, if reduction { 0 } else { 5 }];
            reject_atomic(&mut g, Action::AnnounceCast { card: id });
            g.state.players[p.index()].mana[5] += 1;
            super::super::ability_timeline_tests::cast(&mut g, id, None, &[]);
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert_eq!(
                g.stats(id),
                Stats {
                    power: 6,
                    toughness: 5
                }
            );
            assert!(
                g.keywords(id)
                    .contains(Keywords::REACH.union(Keywords::TRAMPLE))
            );
        }
    }
}
#[test]
fn inherited_green_moon_vigil_dynamic_battlefield_and_graveyard_counts_use_current_controller() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let own = spawn(&mut g, p, Card::GrizzlyBears, Zone::Battlefield);
        let dead = spawn(&mut g, p, Card::GrizzlyBears, Zone::Graveyard);
        spawn(
            &mut g,
            p,
            Card::from_name("Morcant's Eyes").unwrap(),
            Zone::Graveyard,
        );
        let enemy = spawn(&mut g, p.other(), Card::HillGiant, Zone::Battlefield);
        spawn(&mut g, p.other(), Card::GrizzlyBears, Zone::Graveyard);
        let source = cast_named(&mut g, p, "Moon-Vigil Adherents");
        assert_eq!(g.stats(source), Stats::from(3));
        assert!(g.keywords(source).contains(Keywords::TRAMPLE));
        g.move_object(own, Zone::Graveyard);
        assert_eq!(g.stats(source), Stats::from(3));
        g.move_object(dead, Zone::Exile);
        assert_eq!(g.stats(source), Stats::from(2));
        g.change_controller(g.object_ref(enemy), p);
        assert_eq!(g.stats(source), Stats::from(3));
        g.change_controller(g.object_ref(source), p.other());
        assert_eq!(g.stats(source), Stats::from(2));
    }
}
#[test]
fn inherited_green_morcants_eyes_own_upkeep_private_surveil_sorcery_sacrifice_and_all_elf_cards() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let eyes = cast_named(&mut g, p, "Morcant's Eyes");
        g.state.active = p.other();
        g.begin_step(Step::Upkeep);
        assert_eq!(surveil(&mut g, p, false), 0);
        g.state.active = p;
        g.begin_step(Step::Upkeep);
        assert_eq!(surveil(&mut g, p, true), 1);
        for name in ["Lys Alana Informant", "Morcant's Eyes", "Graveshifter"] {
            spawn(&mut g, p, Card::from_name(name).unwrap(), Zone::Graveyard);
        }
        spawn(
            &mut g,
            p.other(),
            Card::from_name("Lys Alana Informant").unwrap(),
            Zone::Graveyard,
        );
        g.state.active = p.other();
        g.state.priority = p;
        reject_atomic(
            &mut g,
            Action::AnnounceActivate {
                card: eyes,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        g.state.active = p;
        g.state.step = Step::Main1;
        g.state.priority = p;
        g.state.players[p.index()].mana = [0, 0, 0, 0, 2, 4];
        activate(&mut g, eyes, 0, None);
        finish_cast_choices(&mut g);
        assert_eq!(g.object_view(eyes).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        settle(&mut g);
        let token = Card::from_name("@token/ea4dc4a4f4dec81ce9fea75f").unwrap();
        let tokens = g
            .state
            .objects
            .iter()
            .enumerate()
            .filter(|(_, o)| o.zone == Zone::Battlefield && o.controller == p && o.card == token)
            .map(|(id, _)| id as CardId)
            .collect::<Vec<_>>();
        assert_eq!(tokens.len(), 4);
        for id in tokens {
            assert_eq!(g.stats(id), Stats::from(2));
            assert_eq!(g.object_view(id).colors, 0b10100);
            assert!(g.has_subtype(id, "Elf"));
            assert!(!g.object_view(id).tapped);
        }
    }
}
#[test]
fn inherited_green_surly_farrier_sorcery_tap_owned_target_and_vigilance_expiry() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = cast_named(&mut g, p, "Surly Farrier");
        let own = spawn(&mut g, p, Card::GrizzlyBears, Zone::Battlefield);
        let enemy = spawn(&mut g, p.other(), Card::GrizzlyBears, Zone::Battlefield);
        let begin = Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        };
        reject_atomic(&mut g, begin.clone());
        g.state.objects[source as usize].summoning_sick = false;
        g.state.active = p.other();
        reject_atomic(&mut g, begin.clone());
        g.state.active = p;
        checked(&mut g, begin);
        reject_atomic(
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
        finish_cast_choices(&mut g);
        assert!(g.object_view(source).tapped);
        settle(&mut g);
        assert_eq!(g.stats(own), Stats::from(3));
        assert!(g.keywords(own).contains(Keywords::VIGILANCE));
        thoughtweft_attack(&mut g, &[own]);
        assert!(!g.object_view(own).tapped);
        super::super::ability_timeline_tests::next_turn(&mut g);
        assert_eq!(g.stats(own), Stats::from(2));
        assert!(!g.keywords(own).contains(Keywords::VIGILANCE));
    }
}
#[test]
fn inherited_green_unforgiving_aim_all_modes_and_flying_recheck() {
    for p in [Player::P0, Player::P1] {
        for mode in 0..3 {
            for lose_flying in [false, true] {
                let mut g = setup(p);
                let bear = spawn(&mut g, p.other(), Card::GrizzlyBears, Zone::Battlefield);
                let flyer = spawn(
                    &mut g,
                    p.other(),
                    Card::from_name("Serra Angel").unwrap(),
                    Zone::Battlefield,
                );
                let enchantment = spawn(
                    &mut g,
                    p.other(),
                    Card::from_name("Morcant's Eyes").unwrap(),
                    Zone::Battlefield,
                );
                let source = spawn(
                    &mut g,
                    p,
                    Card::from_name("Unforgiving Aim").unwrap(),
                    Zone::Hand,
                );
                checked(&mut g, Action::AnnounceCast { card: source });
                checked(&mut g, Action::ChooseMode { mode });
                if mode == 0 {
                    reject_atomic(
                        &mut g,
                        Action::ChooseTarget {
                            target: Target::Card(bear),
                        },
                    );
                    checked(
                        &mut g,
                        Action::ChooseTarget {
                            target: Target::Card(flyer),
                        },
                    );
                } else if mode == 1 {
                    checked(
                        &mut g,
                        Action::ChooseTarget {
                            target: Target::Card(enchantment),
                        },
                    );
                }
                finish_cast_choices(&mut g);
                if mode == 0 && lose_flying {
                    let object = g.object_ref(flyer);
                    g.remove_abilities_exact(&[object], AnimationDuration::Permanent);
                }
                settle(&mut g);
                if mode == 0 {
                    assert_eq!(
                        g.object_view(flyer).zone,
                        if lose_flying {
                            Zone::Battlefield
                        } else {
                            Zone::Graveyard
                        }
                    );
                } else if mode == 1 {
                    assert_eq!(g.object_view(enchantment).zone, Zone::Graveyard);
                } else {
                    assert!(g.state.objects.iter().any(|o| o.zone == Zone::Battlefield
                        && o.controller == p
                        && o.card == Card::from_name("@token/ea4dc4a4f4dec81ce9fea75f").unwrap()));
                }
            }
        }
    }
}
#[test]
fn inherited_lands_five_shocks_actual_preentry_payment_and_exact_two_intrinsic_colors() {
    for p in [Player::P0, Player::P1] {
        for (name, mask, types) in [
            ("Blood Crypt", 0b01100, ["Swamp", "Mountain"]),
            ("Hallowed Fountain", 0b00011, ["Plains", "Island"]),
            ("Overgrown Tomb", 0b10100, ["Swamp", "Forest"]),
            ("Steam Vents", 0b01010, ["Island", "Mountain"]),
            ("Temple Garden", 0b10001, ["Forest", "Plains"]),
        ] {
            for pay in [false, true] {
                let mut g = setup(p);
                let id = spawn(&mut g, p, Card::from_name(name).unwrap(), Zone::Hand);
                g.state.players[p.index()].mana = [0; 6];
                let before = g.object_ref(id);
                checked(&mut g, Action::PlayLand { card: id });
                assert_eq!(g.object_view(id).zone, Zone::Hand);
                assert_eq!(g.observe(p).entry_life.unwrap().cost, 2);
                checked(&mut g, Action::ChooseEntryLife { pay });
                assert_eq!(g.object_view(id).zone, Zone::Battlefield);
                assert_ne!(g.object_ref(id), before);
                assert_eq!(g.state.players[p.index()].life, if pay { 18 } else { 20 });
                assert_eq!(g.object_view(id).tapped, !pay);
                for typ in types {
                    assert!(g.has_subtype(id, typ));
                }
                assert!(!g.state.objects[id as usize].card.is_basic_land());
                if pay {
                    let choices = g
                        .legal_actions()
                        .into_iter()
                        .filter(|a| matches!(a,Action::ActivateMana{card,..}if *card==id))
                        .collect::<Vec<_>>();
                    assert_eq!(choices.len(), 2);
                    for color in 0..6 {
                        assert_eq!(choices.iter().any(|a|matches!(a,Action::ActivateMana{production,..}if production[color]==1)),mask&(1<<color)!=0);
                    }
                    let action = choices[0].clone();
                    checked(&mut g, action);
                    assert_eq!(g.state.players[p.index()].mana.into_iter().sum::<u32>(), 1);
                } else {
                    assert!(
                        !g.legal_actions()
                            .iter()
                            .any(|a| matches!(a,Action::ActivateMana{card,..}if *card==id))
                    );
                }
            }
        }
    }
}
#[test]
fn inherited_lands_evolving_wilds_instant_sacrifice_private_basic_search_tapped_and_fail_to_find() {
    for p in [Player::P0, Player::P1] {
        for find in [false, true] {
            let mut g = setup(p);
            let source = spawn(
                &mut g,
                p,
                Card::from_name("Evolving Wilds").unwrap(),
                Zone::Hand,
            );
            let nonbasic = spawn(
                &mut g,
                p,
                Card::from_name("Temple Garden").unwrap(),
                Zone::Library,
            );
            let basic = g.state.players[p.index()].library[0];
            checked(&mut g, Action::PlayLand { card: source });
            g.state.active = p.other();
            g.state.step = Step::BeginCombat;
            g.state.priority = p;
            activate(&mut g, source, 0, None);
            finish_cast_choices(&mut g);
            assert_eq!(g.object_view(source).zone, Zone::Graveyard);
            until(&mut g, |g| g.observe(p).search_choice.is_some());
            assert!(g.observe(p.other()).search_choice.is_none());
            reject_atomic(&mut g, Action::ChooseSearch { card: nonbasic });
            if find {
                checked(&mut g, Action::ChooseSearch { card: basic });
            }
            checked(&mut g, Action::FinishSearch);
            settle(&mut g);
            assert_eq!(
                g.object_view(basic).zone,
                if find {
                    Zone::Battlefield
                } else {
                    Zone::Library
                }
            );
            if find {
                assert!(g.object_view(basic).tapped);
                assert_eq!(g.object_view(basic).controller, p);
            }
            assert_eq!(g.object_view(nonbasic).zone, Zone::Library);
        }
    }
}
