//! Oracle outcomes for seven inherited complete HOB Adventure pairs.
use super::*;
use crate::engine::ability_world::CharacteristicWorld;
fn wait(g: &mut Game, predicate: impl Fn(&Game) -> bool) {
    for _ in 0..32 {
        if predicate(g) {
            return;
        }
        assert_eq!(g.state.pending, Pending::Priority);
        act(g, Action::Pass);
    }
    panic!("bounded inherited-pair choice missing");
}
fn announce_face(g: &mut Game, card: CardId, adventure: bool, mana: Mana) {
    let p = g.actor().unwrap();
    g.state.players[p.index()].mana = mana;
    act(
        g,
        if adventure {
            Action::AnnounceAdventure { card }
        } else if g.object_view(card).zone == Zone::Exile {
            Action::AnnounceExile { card }
        } else {
            Action::AnnounceCast { card }
        },
    );
    if matches!(g.state.pending, Pending::SpellCost { .. }) {
        choose(g, |a| matches!(a, Action::ChooseSpellCost { .. }));
    }
}
fn finish(g: &mut Game) {
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
    pay(g);
    assert_eq!(
        g.state.players[g.state.stack.last().unwrap().controller.index()].mana,
        [0; 6]
    );
}
fn paid(g: &mut Game, card: CardId, adventure: bool, target: Option<Target>, mana: Mana) {
    announce_face(g, card, adventure, mana);
    if let Some(target) = target {
        act(g, Action::ChooseTarget { target });
    }
    finish(g);
}
fn normal(
    g: &mut Game,
    p: Player,
    card: CardId,
    mana: Mana,
    body: Option<(i128, i128)>,
    keywords: Keywords,
) {
    assert_eq!(g.object_view(card).zone, Zone::Exile);
    assert!(g.can_play_adventure(p, card));
    assert!(!g.can_play_adventure(p.other(), card));
    g.state.active = p;
    g.give_priority();
    g.state.players[p.index()].mana = [8; 6];
    reject(g, Action::AnnounceAdventure { card });
    paid(g, card, false, None, mana);
    settle(g);
    assert_eq!(g.object_view(card).zone, Zone::Battlefield);
    if let Some(body) = body {
        assert_eq!(stats(g, card), body);
    }
    assert!(g.keywords(card).contains(keywords));
    assert!(!g.can_play_adventure(p, card));
}

#[test]
fn hob_inherited_two_beorn_paid_faces_extra_land_is_controller_only_and_expires() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let lands: Vec<_> = (0..3)
            .map(|_| put(&mut g, p, "Forest", Zone::Hand))
            .collect();
        act(&mut g, Action::PlayLand { card: lands[0] });
        reject(&mut g, Action::PlayLand { card: lands[1] });
        let beorn = put(&mut g, p, "Beorn, Reluctant Host", Zone::Hand);
        paid(&mut g, beorn, true, None, [0, 0, 0, 0, 1, 1]);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].extra_land_plays_remaining, 1);
        assert_eq!(
            g.state.players[p.other().index()].extra_land_plays_remaining,
            0
        );
        act(&mut g, Action::PlayLand { card: lands[1] });
        reject(&mut g, Action::PlayLand { card: lands[2] });
        normal(
            &mut g,
            p,
            beorn,
            [0, 0, 0, 0, 1, 4],
            Some((5, 5)),
            Keywords::TRAMPLE,
        );
        g.finish_cleanup();
        assert_eq!(g.state.players[p.index()].extra_land_plays_granted, 0);
        assert_eq!(g.state.players[p.index()].extra_land_plays_remaining, 0);
        // Fresh own main-phase fixture after expiry: exactly the ordinary land.
        g.state.active = p;
        g.state.step = Step::Main1;
        g.state.land_played = false;
        g.state.continuous_land_plays_used = 0;
        g.give_priority();
        let fourth = put(&mut g, p, "Island", Zone::Hand);
        act(&mut g, Action::PlayLand { card: lands[2] });
        reject(&mut g, Action::PlayLand { card: fourth });
    }
}

#[test]
fn hob_inherited_two_bofur_both_target_kinds_control_protection_lifelink_and_expiry() {
    for p in [Player::P0, Player::P1] {
        for artifact in [false, true] {
            let mut g = scenario(p);
            let protected = put(
                &mut g,
                p,
                if artifact {
                    "Goldvein Pick"
                } else {
                    "Grizzly Bears"
                },
                Zone::Battlefield,
            );
            let foreign = put(&mut g, p.other(), "Goldvein Pick", Zone::Battlefield);
            let land = put(&mut g, p, "Forest", Zone::Battlefield);
            let bofur = put(&mut g, p, "Bofur, Reliable Guardian", Zone::Hand);
            g.state.active = p.other(); // Instant inset remains castable on opponent turn.
            announce_face(&mut g, bofur, true, [1, 0, 0, 0, 0, 1]);
            for id in [foreign, land] {
                reject(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(id),
                    },
                );
            }
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(protected),
                },
            );
            finish(&mut g);
            settle(&mut g);
            assert!(
                g.keywords(protected)
                    .contains(Keywords::HEXPROOF.union(Keywords::INDESTRUCTIBLE))
            );
            // Opponent cannot target the protected object, but has a legal fallback.
            g.state.priority = p.other();
            let hostile = put(
                &mut g,
                p.other(),
                if artifact { "Disenchant" } else { "Shock" },
                Zone::Hand,
            );
            announce_face(
                &mut g,
                hostile,
                false,
                if artifact {
                    [1, 0, 0, 0, 0, 1]
                } else {
                    [0, 0, 0, 1, 0, 0]
                },
            );
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(protected),
                },
            );
            act(
                &mut g,
                Action::ChooseTarget {
                    target: if artifact {
                        Target::Card(foreign)
                    } else {
                        Target::Player(p)
                    },
                },
            );
            finish(&mut g);
            settle(&mut g);
            // Its controller can target it; the actual destroy spell still cannot destroy it.
            g.state.priority = p;
            let destroy = put(
                &mut g,
                p,
                if artifact { "Disenchant" } else { "Murder" },
                Zone::Hand,
            );
            paid(
                &mut g,
                destroy,
                false,
                Some(Target::Card(protected)),
                if artifact {
                    [1, 0, 0, 0, 0, 1]
                } else {
                    [0, 0, 2, 0, 0, 1]
                },
            );
            settle(&mut g);
            assert_eq!(g.object_view(protected).zone, Zone::Battlefield);
            normal(
                &mut g,
                p,
                bofur,
                [1, 0, 0, 0, 0, 0],
                Some((1, 1)),
                Keywords::LIFELINK,
            );
            let life = g.state.players[p.index()].life;
            let _ = g.apply_damage_batch(&[DamageEvent {
                preventable: true,
                source: g.damage_source(bofur),
                target: Target::Player(p.other()),
                amount: 1,
                combat: true,
            }]);
            assert_eq!(g.state.players[p.index()].life, life + 1);
            g.finish_cleanup();
            assert!(!g.keywords(protected).contains(Keywords::HEXPROOF));
            assert!(!g.keywords(protected).contains(Keywords::INDESTRUCTIBLE));
        }
    }
}

#[test]
fn hob_inherited_two_glamdring_exact_six_all_recovery_uses_normal_outside_stack_types() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let old = put(&mut g, p, "Shock", Zone::Graveyard);
        let eligible: Vec<_> = ["Shock", "Quick Study", "Duress"]
            .into_iter()
            .map(|n| put(&mut g, p, n, Zone::Library))
            .collect();
        let excluded: Vec<_> = ["Bilbo Baggins, Burglar", "Forest", "Goldvein Pick"]
            .into_iter()
            .map(|n| put(&mut g, p, n, Zone::Library))
            .collect();
        let glam = put(&mut g, p, "Glamdring, Foe-hammer", Zone::Hand);
        let library = g.state.players[p.index()].library.len();
        paid(&mut g, glam, true, None, [0, 1, 0, 0, 0, 3]);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].library.len(), library - 6);
        for id in eligible {
            assert_eq!(g.object_view(id).zone, Zone::Hand);
        }
        for id in excluded {
            assert_eq!(g.object_view(id).zone, Zone::Graveyard);
        }
        assert_eq!(g.object_view(old).zone, Zone::Graveyard);
        assert!(g.observe(p).milled_choice.is_none());
        normal(&mut g, p, glam, [0, 0, 0, 0, 0, 2], None, Keywords::NONE);
        assert!(!g.is_creature(glam));
        assert!(g.is_equipment(glam));
    }
}

#[test]
fn hob_inherited_two_glamdring_paid_equip_live_power_colored_cost_and_locked_real_mana_sacrifice() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let glam = put(&mut g, p, "Glamdring, Foe-hammer", Zone::Hand);
        paid(&mut g, glam, false, None, [0, 0, 0, 0, 0, 2]);
        settle(&mut g);
        let bunny = put(&mut g, p, "Regal Bunnicorn", Zone::Battlefield);
        let treasure = put(
            &mut g,
            p,
            "@token/b60f6cd3b44b22bd289fe4e2",
            Zone::Battlefield,
        );
        put(&mut g, p, "Goldvein Pick", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        act(
            &mut g,
            Action::AnnounceActivate {
                card: glam,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bunny),
            },
        );
        finish(&mut g);
        settle(&mut g);
        assert_eq!(stats(&g, bunny), (4, 4));
        let stab = put(&mut g, p, "Stab", Zone::Hand);
        paid(
            &mut g,
            stab,
            false,
            Some(Target::Card(bunny)),
            [0, 0, 1, 0, 0, 0],
        );
        settle(&mut g);
        assert_eq!(stats(&g, bunny), (2, 2));
        let quick = put(&mut g, p, "Quick Study", Zone::Hand);
        g.state.players[p.index()].mana = [0; 6];
        act(&mut g, Action::AnnounceCast { card: quick });
        let locked = g.state.announcing.as_ref().unwrap().total_cost.unwrap();
        assert_eq!((locked.colored, locked.generic), ([0, 1, 0, 0, 0, 0], 0));
        let before = g.snapshot();
        reject(&mut g, Action::PayCosts { payment: [0; 6] });
        assert_eq!(g.snapshot(), before);
        act(
            &mut g,
            Action::ActivateMana {
                card: treasure,
                ability: 0,
                payment: [0; 6],
                counter_payment: Counters::default(),
                production: [0, 1, 0, 0, 0, 0],
            },
        );
        assert_eq!(g.object_view(treasure).zone, Zone::Graveyard);
        assert_eq!(stats(&g, bunny), (1, 1));
        assert_eq!(
            g.state.announcing.as_ref().unwrap().total_cost,
            Some(locked)
        );
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(g.hand_count(p), 2);
        // Creature spells receive no discount; opponent instants receive no discount.
        let own = put(&mut g, p, "Grizzly Bears", Zone::Hand);
        paid(&mut g, own, false, None, [0, 0, 0, 0, 1, 1]);
        settle(&mut g);
        g.state.active = p.other();
        g.give_priority();
        let other = put(&mut g, p.other(), "Quick Study", Zone::Hand);
        g.state.players[p.other().index()].mana = [0, 1, 0, 0, 0, 0];
        reject(&mut g, Action::AnnounceCast { card: other });
        paid(&mut g, other, false, None, [0, 1, 0, 0, 0, 2]);
        settle(&mut g);
        // Negative equipped power cannot increase a spell's cost.
        g.state.active = p;
        g.give_priority();
        g.state.objects[bunny as usize].growth.power = -10;
        assert!(stats(&g, bunny).0 < 0);
        let final_quick = put(&mut g, p, "Quick Study", Zone::Hand);
        paid(&mut g, final_quick, false, None, [0, 1, 0, 0, 0, 2]);
        settle(&mut g);
    }
}

#[test]
fn hob_inherited_two_gloin_paid_opponent_only_damage_and_first_own_main_mana_packet() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let own = put(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        let small = put(&mut g, p.other(), "Llanowar Elves", Zone::Battlefield);
        let large = put(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        let artifact = put(&mut g, p.other(), "Goldvein Pick", Zone::Battlefield);
        let gloin = put(&mut g, p, "Glóin the Mighty", Zone::Hand);
        paid(&mut g, gloin, true, None, [0, 0, 0, 1, 0, 2]);
        settle(&mut g);
        assert_eq!(g.object_view(small).zone, Zone::Graveyard);
        assert_eq!(g.object_view(large).damage, 1);
        assert_eq!(g.object_view(own).damage, 0);
        assert_eq!(g.object_view(artifact).damage, 0);
        assert_eq!(g.state.players[p.other().index()].life, 20);
        normal(
            &mut g,
            p,
            gloin,
            [0, 0, 0, 1, 0, 3],
            Some((4, 3)),
            Keywords::NONE,
        );
        for (active, step) in [(p.other(), Step::Main1), (p, Step::Main2)] {
            g.state.active = active;
            g.begin_step(step);
            g.give_priority();
            g.prepare_triggers();
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        }
        g.state.active = p;
        g.begin_step(Step::Main1);
        g.give_priority();
        g.prepare_triggers();
        assert_eq!(g.state.stack.len(), 1);
        g.move_object(gloin, Zone::Exile);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 2, 0, 0]);
        let shock = put(&mut g, p, "Shock", Zone::Hand);
        act(&mut g, Action::AnnounceCast { card: shock });
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            act(&mut g, Action::FinishTargets);
        }
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 1, 0, 0]);
        assert_eq!(g.state.players[p.other().index()].life, 18);
    }
}

#[test]
fn hob_inherited_two_meager_meal_optional_creature_mandatory_player_partial_stale_and_menace() {
    for p in [Player::P0, Player::P1] {
        for choice in 0..3 {
            let mut g = scenario(p);
            let creature = put(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
            let second = put(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
            let gollum = put(&mut g, p, "Gollum, Silent Slinker", Zone::Hand);
            announce_face(&mut g, gollum, true, [0, 0, 1, 0, 0, 0]);
            if choice == 0 {
                act(&mut g, Action::FinishTargets);
            } else {
                act(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(creature),
                    },
                );
            }
            if g.state.announcing.as_ref().unwrap().group == 0 {
                act(&mut g, Action::FinishTargets);
            }
            reject(&mut g, Action::FinishTargets); // Player group is mandatory even with no creature.
            let receiver = if choice == 1 { p.other() } else { p };
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Player(receiver),
                },
            );
            finish(&mut g);
            if choice == 2 {
                g.move_object(creature, Zone::Hand);
                g.move_object(creature, Zone::Battlefield);
            }
            settle(&mut g);
            assert_eq!(
                g.counter_count(creature, CounterKind::PlusOne),
                u64::from(choice == 1)
            );
            assert_eq!(g.state.players[receiver.index()].life, 22);
            assert_eq!(g.state.players[receiver.other().index()].life, 20);
            normal(
                &mut g,
                p,
                gollum,
                [0, 0, 1, 0, 0, 3],
                Some((4, 3)),
                Keywords::MENACE,
            );
            if choice == 0 {
                g.state.objects[gollum as usize].summoning_sick = false;
                g.state.step = Step::Attackers;
                g.state.pending = Pending::Attackers;
                act(&mut g, Action::Attack { card: gollum });
                act(&mut g, Action::FinishAttackers);
                wait(&mut g, |g| g.state.pending == Pending::Blockers);
                act(
                    &mut g,
                    Action::Block {
                        blocker: creature,
                        attacker: gollum,
                    },
                );
                reject(&mut g, Action::FinishBlockers);
                act(
                    &mut g,
                    Action::Block {
                        blocker: second,
                        attacker: gollum,
                    },
                );
                act(&mut g, Action::FinishBlockers);
            }
        }
    }
}

#[test]
fn hob_inherited_two_bird_exact_four_mandatory_one_private_choice_and_live_threshold() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let old = put(&mut g, p, "Shock", Zone::Graveyard);
        let spell = put(&mut g, p, "Shock", Zone::Library);
        let other = put(&mut g, p, "Duress", Zone::Library);
        let adventure = put(&mut g, p, "Bilbo Baggins, Burglar", Zone::Library);
        let land = put(&mut g, p, "Forest", Zone::Library);
        let bird = put(&mut g, p, "Most Decrepit Old Bird", Zone::Hand);
        paid(&mut g, bird, true, None, [0, 1, 0, 0, 0, 1]);
        wait(&mut g, |g| g.observe(p).milled_choice.is_some());
        let choice = g.observe(p).milled_choice.unwrap();
        assert_eq!(choice.milled.len(), 4);
        assert_eq!((choice.minimum, choice.maximum), (1, 1));
        assert_eq!(
            choice
                .candidates
                .iter()
                .map(|r| r.card)
                .collect::<std::collections::BTreeSet<_>>(),
            [spell, other].into_iter().collect()
        );
        assert!(g.observe(p.other()).milled_choice.is_none());
        reject(&mut g, Action::FinishMilledSelection);
        for id in [old, adventure, land] {
            let object = g.object_ref(id);
            reject(&mut g, Action::SelectMilled { object });
        }
        if p == Player::P1 {
            contracts::branch_contract(g.clone());
        }
        let object = g.object_ref(spell);
        act(&mut g, Action::SelectMilled { object });
        settle(&mut g);
        assert_eq!(g.object_view(spell).zone, Zone::Hand);
        for id in [old, other, adventure, land] {
            assert_eq!(g.object_view(id).zone, Zone::Graveyard);
        }
        normal(
            &mut g,
            p,
            bird,
            [0, 1, 0, 0, 0, 0],
            Some((1, 1)),
            Keywords::FLYING,
        );
        for _ in 0..7 {
            put(&mut g, p.other(), "Forest", Zone::Graveyard);
        }
        let six = put(&mut g, p, "Forest", Zone::Graveyard);
        put(&mut g, p, "Forest", Zone::Graveyard);
        assert_eq!(stats(&g, bird), (1, 1)); // Four mill leftovers/old cards + two = six.
        put(&mut g, p, "Forest", Zone::Graveyard);
        assert_eq!(stats(&g, bird), (2, 2));
        g.move_object(six, Zone::Exile);
        assert_eq!(stats(&g, bird), (1, 1));
    }
}

#[test]
fn hob_inherited_two_bird_no_eligible_milled_card_completes_without_finding_old_spell() {
    let p = Player::P0;
    let mut g = scenario(p);
    let old = put(&mut g, p, "Shock", Zone::Graveyard);
    for n in [
        "Forest",
        "Goldvein Pick",
        "Grizzly Bears",
        "Bilbo Baggins, Burglar",
    ] {
        put(&mut g, p, n, Zone::Library);
    }
    let bird = put(&mut g, p, "Most Decrepit Old Bird", Zone::Hand);
    paid(&mut g, bird, true, None, [0, 1, 0, 0, 0, 1]);
    settle(&mut g);
    assert!(g.observe(p).milled_choice.is_none());
    assert_eq!(g.object_view(old).zone, Zone::Graveyard);
    assert!(g.can_play_adventure(p, bird));
}

#[test]
fn hob_inherited_two_velvetwing_one_two_and_partial_invalid_targets_then_paid_flying_body() {
    for p in [Player::P0, Player::P1] {
        for (count, stale) in [(1, false), (2, false), (2, true)] {
            let mut g = scenario(p);
            let own = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
            let enemy = put(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
            let artifact = put(&mut g, p.other(), "Goldvein Pick", Zone::Battlefield);
            let wing = put(&mut g, p, "Velvetwing Butterflies", Zone::Hand);
            g.state.active = p.other();
            announce_face(&mut g, wing, true, [1, 0, 0, 0, 0, 1]);
            reject(&mut g, Action::FinishTargets);
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(artifact),
                },
            );
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(own),
                },
            );
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(own),
                },
            );
            if count == 2 {
                act(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(enemy),
                    },
                );
            }
            finish(&mut g);
            if stale {
                g.move_object(own, Zone::Hand);
                g.move_object(own, Zone::Battlefield);
            }
            settle(&mut g);
            assert_eq!(g.object_view(own).tapped, !stale);
            assert_eq!(g.object_view(enemy).tapped, count == 2);
            normal(
                &mut g,
                p,
                wing,
                [1, 0, 0, 0, 0, 2],
                Some((2, 2)),
                Keywords::FLYING,
            );
        }
    }
}
