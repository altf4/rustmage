//! Focused printed outcomes; shared engine invariants live in core tests.
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
    {
        let p = Player::P0;
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
fn hob_inherited_spells_burn_saga_paid_four_chapters_restrictions_mana_and_final_sacrifice() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let own = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Galewind Moose", Zone::Battlefield);
        // A surviving seven-toughness target distinguishes exactly six damage
        // from both insufficient and excessive damage.
        g.add_counter(enemy, CounterKind::PlusOne, 1);
        assert_eq!(stats(&g, enemy), (7, 7));
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
        assert_eq!(g.object_view(enemy).zone, Zone::Battlefield);
        assert_eq!(g.object_view(enemy).damage, 6);
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
    {
        let p = Player::P0;
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
    {
        let p = Player::P0;
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
fn hob_inherited_spells_reverent_howl_both_modes_any_player_or_creature_and_lifelink_expiry() {
    {
        let p = Player::P0;
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
    {
        let p = Player::P0;
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
fn hob_inherited_spells_uneasy_partings_attacking_nontoken_discount_owner_choice_and_token_exclusion()
 {
    {
        let p = Player::P0;
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
