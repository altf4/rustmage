//! Current paid Oracle outcomes for fourteen inherited HOB spell identities.
use super::*;
fn wait(g: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..40 {
        if ready(g) {
            return;
        }
        assert_eq!(g.state.pending, Pending::Priority);
        act(g, Action::Pass);
    }
    panic!("bounded inherited-spell choice missing");
}
fn begin(g: &mut Game, p: Player, name: &str, mode: Option<u16>, mana: Mana) -> CardId {
    let id = put(g, p, name, Zone::Hand);
    g.state.players[p.index()].mana = mana;
    act(g, Action::AnnounceCast { card: id });
    if let Some(mode) = mode {
        act(g, Action::ChooseMode { mode });
    }
    id
}
fn target(g: &mut Game, id: CardId) {
    act(
        g,
        Action::ChooseTarget {
            target: Target::Card(id),
        },
    );
}
fn finish(g: &mut Game) {
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
    let p = g.actor().unwrap();
    pay(g);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
}
fn paid(
    g: &mut Game,
    p: Player,
    name: &str,
    mode: Option<u16>,
    targets: &[CardId],
    mana: Mana,
) -> CardId {
    let id = begin(g, p, name, mode, mana);
    for &id in targets {
        target(g, id);
    }
    finish(g);
    id
}
fn hit(g: &mut Game, source: CardId, target: Target, amount: i128) {
    g.apply_damage_batch(&[DamageEvent {
        preventable: true,
        source: g.damage_source(source),
        target,
        amount,
        combat: false,
    }]);
}

#[test]
fn hob_inherited_spells_deadly_slice_paid_any_creature_indestructible_and_actual_exile_replacement()
{
    for p in [Player::P0, Player::P1] {
        for replacement in [false, true] {
            let mut g = scenario(p);
            let bear = put(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
            let artifact = put(&mut g, p, "Goldvein Pick", Zone::Battlefield);
            if replacement {
                put(&mut g, p, "Rest in Peace", Zone::Battlefield);
            }
            begin(&mut g, p, "Bilbo's Deadly Slice", None, [0, 0, 2, 0, 0, 1]);
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(artifact),
                },
            );
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Player(p.other()),
                },
            );
            target(&mut g, bear);
            finish(&mut g);
            settle(&mut g);
            assert_eq!(
                g.object_view(bear).zone,
                if replacement {
                    Zone::Exile
                } else {
                    Zone::Graveyard
                }
            );
            let own = put(&mut g, p, "Zetalpa, Primal Dawn", Zone::Battlefield);
            paid(
                &mut g,
                p,
                "Bilbo's Deadly Slice",
                None,
                &[own],
                [0, 0, 2, 0, 0, 1],
            );
            settle(&mut g);
            assert_eq!(g.object_view(own).zone, Zone::Battlefield);
        }
    }
}

#[test]
fn hob_inherited_spells_gambit_gift_owner_routing_uncounterable_no_cast_and_fizzle() {
    for p in [Player::P0, Player::P1] {
        for outcome in 0..3 {
            let mut g = scenario(p.other());
            let original = put(&mut g, p, "Curator of Destinies", Zone::Exile);
            let object = g.object_ref(original);
            g.grant_exile_play_permission(p.other(), object, false);
            g.state.players[p.other().index()].mana = [0, 2, 0, 0, 0, 4];
            act(&mut g, Action::AnnounceExile { card: original });
            finish(&mut g);
            assert_eq!(g.state.stack.last().unwrap().controller, p.other());
            act(&mut g, Action::Pass);
            assert_eq!(g.actor(), Some(p));
            begin(&mut g, p, "Bilbo's Gambit", None, [1, 0, 0, 0, 0, 1]);
            reject(
                &mut g,
                Action::ChooseGift {
                    beneficiary: Some(p),
                },
            );
            act(
                &mut g,
                Action::ChooseGift {
                    beneficiary: (outcome != 0).then_some(p.other()),
                },
            );
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Spell(original),
                },
            );
            finish(&mut g);
            if outcome == 2 {
                g.move_object(original, Zone::Hand);
            }
            settle(&mut g);
            assert_eq!(g.object_view(original).zone, Zone::Hand);
            assert_eq!(g.object_view(original).owner, p);
            assert_eq!(
                g.state.players[p.other().index()].cards_drawn_this_turn,
                0,
                "returned spell never entered"
            );
            let tokens: Vec<_> = g
                .state
                .objects
                .iter()
                .enumerate()
                .filter_map(|(id, o)| {
                    (o.zone == Zone::Battlefield && o.card.has_subtype("Treasure"))
                        .then_some(id as CardId)
                })
                .collect();
            assert_eq!(tokens.len(), usize::from(outcome == 1));
            assert_eq!(g.state.casting_prohibited, outcome == 1);
            let spells = [
                put(&mut g, p, "Shock", Zone::Hand),
                put(&mut g, p.other(), "Shock", Zone::Hand),
            ];
            for (actor, card) in [(p, spells[0]), (p.other(), spells[1])] {
                g.state.priority = actor;
                g.state.players[actor.index()].mana = [0, 0, 0, 1, 0, 0];
                if outcome == 1 {
                    reject(&mut g, Action::AnnounceCast { card });
                } else {
                    assert!(g.legal_actions().contains(&Action::AnnounceCast { card }));
                }
            }
            if outcome == 1 {
                let token = tokens[0];
                let v = g.object_view(token);
                assert_eq!((v.owner, v.controller, v.kinds), (p.other(), p.other(), 16));
                g.state.players[p.other().index()].mana = [0; 6];
                act(
                    &mut g,
                    Action::ActivateMana {
                        card: token,
                        ability: 0,
                        payment: [0; 6],
                        counter_payment: Counters::default(),
                        production: [0, 1, 0, 0, 0, 0],
                    },
                );
                assert_eq!(
                    g.object_view(token).zone,
                    Zone::Outside,
                    "departed token ceases"
                );
                assert_eq!(g.state.players[p.other().index()].mana, [0, 1, 0, 0, 0, 0]);
                let land = put(&mut g, p.other(), "Forest", Zone::Hand);
                act(&mut g, Action::PlayLand { card: land });
                g.finish_cleanup();
                assert!(!g.state.casting_prohibited);
            }
            g.state.active = p;
            g.give_priority();
            g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 0];
            act(&mut g, Action::AnnounceCast { card: spells[0] });
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Player(p.other()),
                },
            );
            finish(&mut g);
            settle(&mut g);
            assert_eq!(g.state.players[p.other().index()].life, 18);
        }
    }
}

#[test]
fn hob_inherited_spells_confusticate_counter_payment_and_private_draw_then_discard() {
    for p in [Player::P0, Player::P1] {
        for tax in 0..3 {
            let pay_tax = tax == 1;
            let available = if tax == 2 { 3 } else { 4 };
            let mut g = scenario(p.other());
            let victim = put(&mut g, p.other(), "Grizzly Bears", Zone::Hand);
            g.state.players[p.other().index()].mana = [0, 0, 0, 0, 1, 1];
            act(&mut g, Action::AnnounceCast { card: victim });
            finish(&mut g);
            act(&mut g, Action::Pass);
            begin(
                &mut g,
                p,
                "Confusticate and Bebother",
                Some(0),
                [0, 1, 0, 0, 0, 2],
            );
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Spell(victim),
                },
            );
            finish(&mut g);
            g.state.players[p.other().index()].mana = [0, 0, 0, 0, 0, available];
            wait(&mut g, |g| {
                g.observe(p.other()).resolution_payment.is_some()
            });
            assert_eq!(g.actor(), Some(p.other()));
            let view = g.observe(p.other()).resolution_payment.unwrap();
            assert_eq!(view.cost.generic, 4);
            if tax == 2 {
                assert!(
                    !g.legal_actions()
                        .iter()
                        .any(|a| matches!(a, Action::PayResolutionMana { .. }))
                );
            }
            if pay_tax {
                choose(&mut g, |a| matches!(a, Action::PayResolutionMana { .. }));
            } else {
                act(&mut g, Action::DeclineResolutionMana);
            }
            settle(&mut g);
            assert_eq!(
                g.object_view(victim).zone,
                if pay_tax {
                    Zone::Battlefield
                } else {
                    Zone::Graveyard
                }
            );
            assert_eq!(
                g.state.players[p.other().index()].mana[5],
                if pay_tax { 0 } else { available }
            );
        }
        let mut g = scenario(p);
        let second = put(&mut g, p, "Island", Zone::Library);
        let first = put(&mut g, p, "Shock", Zone::Library);
        paid(
            &mut g,
            p,
            "Confusticate and Bebother",
            Some(1),
            &[],
            [0, 1, 0, 0, 0, 2],
        );
        wait(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectDiscard { .. })
        });
        assert_eq!(g.hand_count(p), 2);
        for id in [first, second] {
            assert!(g.observe(p.other()).objects.iter().all(|v| v.id != id));
        }
        if p == Player::P1 {
            contracts::branch_contract(g.clone());
        }
        act(&mut g, Action::Discard { card: first });
        settle(&mut g);
        assert_eq!(g.object_view(first).zone, Zone::Graveyard);
        assert_eq!(g.object_view(second).zone, Zone::Hand);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 2);
    }
}

#[test]
fn hob_inherited_spells_burn_saga_paid_four_chapters_restrictions_mana_and_final_sacrifice() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let own = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Galewind Moose", Zone::Battlefield);
        let artifact = put(&mut g, p.other(), "Goldvein Pick", Zone::Battlefield);
        let own_art = put(&mut g, p, "Goldvein Pick", Zone::Battlefield);
        let saga = paid(
            &mut g,
            p,
            "Burn, Burn, Tree and Fern",
            None,
            &[],
            [0, 0, 0, 1, 0, 3],
        );
        wait(&mut g, |g| {
            g.legal_actions().contains(&Action::ChooseTarget {
                target: Target::Card(enemy),
            })
        });
        assert_eq!(g.counter_count(saga, CounterKind::Lore), 1);
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(own),
            },
        );
        target(&mut g, enemy);
        if g.legal_actions().contains(&Action::FinishTargets) {
            act(&mut g, Action::FinishTargets);
        }
        settle(&mut g);
        assert_eq!(g.object_view(enemy).zone, Zone::Graveyard);
        g.begin_step(Step::Main1);
        wait(&mut g, |g| {
            g.legal_actions().contains(&Action::ChooseTarget {
                target: Target::Card(artifact),
            })
        });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(own_art),
            },
        );
        target(&mut g, artifact);
        if g.legal_actions().contains(&Action::FinishTargets) {
            act(&mut g, Action::FinishTargets);
        }
        settle(&mut g);
        assert_eq!(g.object_view(artifact).zone, Zone::Graveyard);
        for chapter in [3, 4] {
            g.state.players[p.index()].mana = [0; 6];
            g.begin_step(Step::Main1);
            g.prepare_triggers();
            assert_eq!(g.counter_count(saga, CounterKind::Lore), chapter);
            assert_eq!(g.object_view(saga).zone, Zone::Battlefield);
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 1, 0, 0]);
        }
        assert_eq!(g.object_view(saga).zone, Zone::Graveyard);
        assert_eq!(g.object_view(own).damage, 0);
    }
}

#[test]
fn hob_inherited_spells_river_aura_all_counters_ability_layers_and_controller_untap_only() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let elf = put(&mut g, p.other(), "Llanowar Elves", Zone::Battlefield);
        let companion = put(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        let artifact = put(&mut g, p, "Goldvein Pick", Zone::Battlefield);
        for (kind, n) in [
            (CounterKind::PlusOne, 2),
            (CounterKind::Stun, 2),
            (CounterKind::Charge, 3),
            (CounterKind::Loyalty, 1),
            (CounterKind::Flying, 1),
        ] {
            g.add_counter(elf, kind, n);
        }
        begin(
            &mut g,
            p,
            "Enchanted River's Grasp",
            None,
            [0, 1, 0, 0, 0, 2],
        );
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(artifact),
            },
        );
        target(&mut g, elf);
        finish(&mut g);
        settle(&mut g);
        let aura = g
            .state
            .objects
            .iter()
            .enumerate()
            .find_map(|(id, o)| {
                (o.zone == Zone::Battlefield
                    && o.card == Card::from_name("Enchanted River's Grasp").unwrap())
                .then_some(id as CardId)
            })
            .unwrap();
        assert!(g.object_view(elf).tapped);
        for kind in CounterKind::ALL {
            assert_eq!(g.counter_count(elf, kind), 0);
        }
        assert!(!g.printed_abilities_available(elf));
        assert!(!g.keywords(elf).contains(Keywords::FLYING));
        g.state.active = p.other();
        g.state.objects[companion as usize].tapped = true;
        g.untap_active_player();
        assert!(g.object_view(elf).tapped);
        assert!(!g.object_view(companion).tapped);
        g.untap_objects(&[elf], untap_rules::UntapCause::Effect);
        assert!(!g.object_view(elf).tapped);
        g.give_priority();
        reject(&mut g, Action::TapMana { card: elf });
        paid(
            &mut g,
            p.other(),
            "Warg Tactics",
            Some(1),
            &[elf],
            [0, 0, 0, 0, 1, 1],
        );
        settle(&mut g);
        assert!(
            g.keywords(elf)
                .contains(Keywords::TRAMPLE.union(Keywords::HEXPROOF))
        );
        assert!(!g.printed_abilities_available(elf));
        g.state.priority = p;
        paid(&mut g, p, "Disenchant", None, &[aura], [1, 0, 0, 0, 0, 1]);
        settle(&mut g);
        assert!(g.printed_abilities_available(elf));
        g.state.priority = p.other();
        g.state.players[p.other().index()].mana = [0; 6];
        act(&mut g, Action::TapMana { card: elf });
        assert_eq!(g.state.players[p.other().index()].mana[4], 1);
    }
}

#[test]
fn hob_inherited_spells_magnificent_end_discount_is_targeted_tapped_and_locked() {
    for p in [Player::P0, Player::P1] {
        for tapped in [false, true] {
            let mut g = scenario(p);
            let victim = put(&mut g, p.other(), "Galewind Moose", Zone::Battlefield);
            let alternative = put(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
            g.state.objects[victim as usize].tapped = tapped;
            g.state.objects[alternative as usize].tapped = false;
            begin(
                &mut g,
                p,
                "Magnificent End",
                None,
                [1, 0, 0, 0, 0, if tapped { 1 } else { 4 }],
            );
            if tapped {
                reject(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(alternative),
                    },
                );
            }
            target(&mut g, victim);
            finish(&mut g);
            g.state.objects[victim as usize].tapped = !tapped;
            settle(&mut g);
            assert_eq!(g.object_view(victim).damage, 5);
            assert_eq!(g.object_view(alternative).damage, 0);
        }
    }
}

#[test]
fn hob_inherited_spells_pinecone_every_mode_partial_stale_and_exact_death_replacement_lifetime() {
    for p in [Player::P0, Player::P1] {
        for row in 0..6 {
            let mut g = scenario(p);
            let small = put(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
            let large = put(&mut g, p.other(), "Galewind Moose", Zone::Battlefield);
            let token = put(
                &mut g,
                if row == 1 { p } else { p.other() },
                "@token/b60f6cd3b44b22bd289fe4e2",
                Zone::Battlefield,
            );
            let ordinary = put(&mut g, p, "Goldvein Pick", Zone::Battlefield);
            let mode = if row == 0 || row == 5 {
                0
            } else if row == 1 {
                1
            } else {
                2
            };
            begin(&mut g, p, "Pinecone Strike", Some(mode), [0, 0, 0, 1, 0, 1]);
            if mode == 0 {
                target(&mut g, if row == 0 { small } else { large });
            } else if mode == 1 {
                reject(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(ordinary),
                    },
                );
                reject(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(small),
                    },
                );
                target(&mut g, token);
            } else {
                target(&mut g, large);
                target(&mut g, token);
            }
            finish(&mut g);
            if row == 3 {
                g.move_object(large, Zone::Hand);
                g.move_object(large, Zone::Battlefield);
            }
            if row == 4 {
                g.move_object(token, Zone::Exile);
            }
            settle(&mut g);
            if row == 0 {
                assert_eq!(g.object_view(small).zone, Zone::Exile);
            }
            if mode != 0 && row != 4 {
                assert_eq!(
                    g.object_view(token).zone,
                    Zone::Outside,
                    "departed token ceases"
                );
            }
            if mode == 2 || row == 5 {
                assert_eq!(g.object_view(large).damage, if row == 3 { 0 } else { 3 });
                if row == 5 {
                    g.finish_cleanup();
                    g.state.active = p;
                    g.state.step = Step::Main1;
                    g.give_priority();
                }
                paid(
                    &mut g,
                    p,
                    "Bilbo's Deadly Slice",
                    None,
                    &[large],
                    [0, 0, 2, 0, 0, 1],
                );
                settle(&mut g);
                assert_eq!(
                    g.object_view(large).zone,
                    if row == 3 || row == 5 {
                        Zone::Graveyard
                    } else {
                        Zone::Exile
                    }
                );
            }
        }
    }
}

#[test]
fn hob_inherited_spells_quarrel_and_troll_sides_live_power_and_each_partial_stale() {
    for p in [Player::P0, Player::P1] {
        for fight in [false, true] {
            for stale in 0..3 {
                let mut g = scenario(p);
                let own = put(
                    &mut g,
                    p,
                    if fight {
                        "Bofur, Reliable Guardian"
                    } else {
                        "Grizzly Bears"
                    },
                    Zone::Battlefield,
                );
                let enemy = put(
                    &mut g,
                    p.other(),
                    if fight {
                        "Grizzly Bears"
                    } else {
                        "Galewind Moose"
                    },
                    Zone::Battlefield,
                );
                begin(
                    &mut g,
                    p,
                    if fight {
                        "Troll Negotiations"
                    } else {
                        "Quarrel"
                    },
                    None,
                    if fight {
                        [0, 0, 0, 0, 2, 2]
                    } else {
                        [0, 0, 0, 0, 1, 1]
                    },
                );
                reject(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(enemy),
                    },
                );
                target(&mut g, own);
                reject(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(own),
                    },
                );
                target(&mut g, enemy);
                finish(&mut g);
                if stale == 1 {
                    g.move_object(own, Zone::Hand);
                    g.move_object(own, Zone::Battlefield);
                }
                if stale == 2 {
                    g.move_object(enemy, Zone::Hand);
                    g.move_object(enemy, Zone::Battlefield);
                }
                if !fight && stale == 0 {
                    paid(&mut g, p, "Giant Growth", None, &[own], [0, 0, 0, 0, 1, 0]);
                }
                settle(&mut g);
                if fight {
                    assert_eq!(
                        g.counter_count(own, CounterKind::PlusOne),
                        if stale == 1 { 0 } else { 2 }
                    );
                    assert_eq!(g.counter_count(enemy, CounterKind::PlusOne), 0);
                    assert_eq!(
                        g.object_view(enemy).zone,
                        if stale == 0 {
                            Zone::Graveyard
                        } else {
                            Zone::Battlefield
                        }
                    );
                    assert_eq!(g.object_view(own).damage, if stale == 0 { 2 } else { 0 });
                    assert_eq!(
                        g.state.players[p.index()].life,
                        if stale == 0 { 23 } else { 20 }
                    );
                } else {
                    assert_eq!(g.object_view(enemy).damage, if stale == 0 { 5 } else { 0 });
                    assert_eq!(g.object_view(own).damage, 0);
                }
            }
        }
    }
}

#[test]
fn hob_inherited_spells_reverent_howl_both_modes_any_player_or_creature_and_lifelink_expiry() {
    for p in [Player::P0, Player::P1] {
        for receiver in [p, p.other()] {
            let mut g = scenario(p);
            begin(&mut g, p, "Reverent Howl", Some(0), [0, 0, 1, 0, 0, 2]);
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Player(receiver),
                },
            );
            finish(&mut g);
            settle(&mut g);
            assert_eq!(g.hand_count(receiver), 2);
            assert_eq!(g.state.players[receiver.index()].life, 18);
            assert_eq!(g.state.players[receiver.other().index()].life, 20);
        }
        let mut g = scenario(p);
        let enemy = put(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        paid(
            &mut g,
            p,
            "Reverent Howl",
            Some(1),
            &[enemy],
            [0, 0, 1, 0, 0, 2],
        );
        settle(&mut g);
        assert_eq!(stats(&g, enemy), (4, 4));
        assert!(g.keywords(enemy).contains(Keywords::LIFELINK));
        hit(&mut g, enemy, Target::Player(p), 4);
        assert_eq!(g.state.players[p.other().index()].life, 24);
        g.finish_cleanup();
        assert_eq!(stats(&g, enemy), (2, 2));
        assert!(!g.keywords(enemy).contains(Keywords::LIFELINK));
    }
}

#[test]
fn hob_inherited_spells_smaug_fury_reach_blocks_flyer_first_strike_prevents_retaliation_then_expires()
 {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p.other());
        let angel = put(&mut g, p.other(), "Serra Angel", Zone::Battlefield);
        let blocker = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let ground = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        act(&mut g, Action::Attack { card: angel });
        act(&mut g, Action::FinishAttackers);
        act(&mut g, Action::Pass);
        paid(
            &mut g,
            p,
            "Smaug's Fury",
            None,
            &[blocker],
            [0, 0, 0, 1, 0, 1],
        );
        settle(&mut g);
        assert_eq!(stats(&g, blocker), (5, 2));
        assert!(
            g.keywords(blocker)
                .contains(Keywords::REACH.union(Keywords::FIRST_STRIKE))
        );
        wait(&mut g, |g| g.state.pending == Pending::Blockers);
        reject(
            &mut g,
            Action::Block {
                blocker: ground,
                attacker: angel,
            },
        );
        act(
            &mut g,
            Action::Block {
                blocker,
                attacker: angel,
            },
        );
        act(&mut g, Action::FinishBlockers);
        wait(&mut g, |g| g.object_view(angel).zone == Zone::Graveyard);
        assert_eq!(g.object_view(blocker).damage, 0);
        assert_eq!(g.object_view(blocker).zone, Zone::Battlefield);
        g.finish_cleanup();
        assert_eq!(stats(&g, blocker), (2, 2));
        assert!(!g.keywords(blocker).contains(Keywords::REACH));
        assert!(!g.keywords(blocker).contains(Keywords::FIRST_STRIKE));
    }
}

#[test]
fn hob_inherited_spells_thorin_modes_snapshot_pump_destroy_restrictions_life_and_fizzle() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let bear = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let elf = put(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        paid(
            &mut g,
            p,
            "Thorin's Last Stand",
            Some(0),
            &[],
            [2, 0, 0, 0, 0, 2],
        );
        settle(&mut g);
        assert_eq!(stats(&g, bear), (4, 3));
        assert_eq!(stats(&g, elf), (3, 2));
        assert_eq!(stats(&g, enemy), (2, 2));
        let late = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        assert_eq!(stats(&g, late), (2, 2));
        g.finish_cleanup();
        assert_eq!(stats(&g, bear), (2, 2));
        for (kind, stale) in [
            ("Goldvein Pick", false),
            ("Banishing Light", false),
            ("Darksteel Colossus", false),
            ("Goldvein Pick", true),
        ] {
            let mut g = scenario(p);
            let artifact = put(&mut g, p.other(), kind, Zone::Battlefield);
            let creature = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
            begin(
                &mut g,
                p,
                "Thorin's Last Stand",
                Some(1),
                [2, 0, 0, 0, 0, 2],
            );
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(creature),
                },
            );
            target(&mut g, artifact);
            finish(&mut g);
            if stale {
                g.move_object(artifact, Zone::Hand);
                g.move_object(artifact, Zone::Battlefield);
            }
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].life, if stale { 20 } else { 22 });
            assert_eq!(
                g.object_view(artifact).zone,
                if stale || kind == "Darksteel Colossus" {
                    Zone::Battlefield
                } else {
                    Zone::Graveyard
                }
            );
        }
    }
}

#[test]
fn hob_inherited_spells_uneasy_partings_attacking_nontoken_discount_owner_choice_and_token_exclusion()
 {
    for p in [Player::P0, Player::P1] {
        for row in 0..4 {
            let mut g = scenario(p);
            let token = row == 3;
            let attacking = row != 2;
            let victim = put(
                &mut g,
                p.other(),
                if token {
                    "@token/a79da9ee6d2f00d35c342024"
                } else {
                    "Grizzly Bears"
                },
                Zone::Battlefield,
            );
            let object = g.object_ref(victim);
            g.record_control(object, p, None);
            g.state.objects[victim as usize].summoning_sick = false;
            if attacking {
                g.state.step = Step::Attackers;
                g.state.pending = Pending::Attackers;
                act(&mut g, Action::Attack { card: victim });
                act(&mut g, Action::FinishAttackers);
            }
            paid(
                &mut g,
                p,
                "Uneasy Partings",
                None,
                &[victim],
                [0, 1, 0, 0, 0, if attacking && !token { 2 } else { 3 }],
            );
            wait(&mut g, |g| {
                matches!(g.state.pending, Pending::TargetLibrary { .. })
            });
            assert_eq!(g.actor(), Some(p.other()));
            assert_eq!(
                g.observe(p.other()).target_library.unwrap().player,
                p.other()
            );
            if row == 0 && p == Player::P1 {
                contracts::branch_contract(g.clone());
            }
            let destination = if row == 1 {
                LibraryDestination::Bottom
            } else {
                LibraryDestination::Top
            };
            let object = g.object_ref(victim);
            act(
                &mut g,
                Action::ChooseLibraryDestination {
                    object,
                    destination,
                },
            );
            settle(&mut g);
            if !token {
                let library = &g.state.players[p.other().index()].library;
                assert_eq!(
                    if row == 1 {
                        library.first()
                    } else {
                        library.last()
                    },
                    Some(&victim)
                );
                assert!(!g.state.players[p.index()].library.contains(&victim));
            } else {
                assert!(!g.state.players[p.other().index()].library.contains(&victim));
            }
        }
    }
}

#[test]
fn hob_inherited_spells_warg_tactics_flying_restriction_own_counter_hexproof_and_eot_keywords() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let flyer = put(&mut g, p.other(), "Serra Angel", Zone::Battlefield);
        let ground = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        begin(&mut g, p, "Warg Tactics", Some(0), [0, 0, 0, 0, 1, 1]);
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(ground),
            },
        );
        target(&mut g, flyer);
        finish(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(flyer).zone, Zone::Graveyard);
        let enemy = put(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        begin(&mut g, p, "Warg Tactics", Some(1), [0, 0, 0, 0, 1, 1]);
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        target(&mut g, ground);
        finish(&mut g);
        settle(&mut g);
        assert_eq!(g.counter_count(ground, CounterKind::PlusOne), 1);
        assert_eq!(stats(&g, ground), (3, 3));
        assert!(
            g.keywords(ground)
                .contains(Keywords::TRAMPLE.union(Keywords::HEXPROOF))
        );
        g.state.priority = p.other();
        begin(&mut g, p.other(), "Shock", None, [0, 0, 0, 1, 0, 0]);
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(ground),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p),
            },
        );
        finish(&mut g);
        settle(&mut g);
        g.finish_cleanup();
        assert_eq!(stats(&g, ground), (3, 3));
        assert!(!g.keywords(ground).contains(Keywords::TRAMPLE));
        assert!(!g.keywords(ground).contains(Keywords::HEXPROOF));
    }
}
