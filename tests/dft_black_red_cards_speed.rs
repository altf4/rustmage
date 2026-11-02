//! Printed black/red speed creatures on the accepted frozen engine.
mod support;
use rustmage as _;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

fn announce(card: CardId, ability: u16) -> Action {
    Action::AnnounceActivate {
        card,
        ability,
        counter_payment: Counters::default(),
    }
}
fn grant(g: &Game, card: CardId, ability: u16) -> Action {
    Action::AnnounceSourceGrant {
        source: g.object_ref(card),
        ability,
    }
}
fn drain(g: &mut Game) {
    for _ in 0..48 {
        g.prepare_triggers();
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty()
            && g.state.resolving.is_none()
        {
            return;
        }
        let opponent = g.state.active.other();
        choose(g, |a| {
            matches!(a, Action::Pass | Action::PutTrigger { .. })
                || matches!(a,Action::ChooseTarget{target:Target::Player(p)} if *p==opponent)
        });
    }
    panic!("speed-family resolution bound");
}
fn wait_hand(g: &mut Game) {
    for _ in 0..16 {
        g.prepare_triggers();
        if matches!(g.state.pending, Pending::HandCardChoice { .. }) {
            return;
        }
        let opponent = g.state.active.other();
        choose(g, |a| {
            matches!(a, Action::Pass | Action::PutTrigger { .. })
                || matches!(a,Action::ChooseTarget{target:Target::Player(p)} if *p==opponent)
        });
    }
    panic!("hand selection missing");
}
fn main_phase(g: &mut Game, p: Player) {
    g.state.active = p;
    g.state.priority = p;
    g.state.step = Step::Main1;
    g.state.pending = Pending::Priority;
}

#[test]
fn every_speed_creature_starts_its_controller_and_keeps_its_printed_body() {
    for p in [Player::P0, Player::P1] {
        for (name, body, keywords, tapped) in [
            ("Burnout Bashtronaut", (1, 1), Keywords::MENACE, false),
            ("Endrider Catalyzer", (3, 1), Keywords::NONE, false),
            ("Endrider Spikespitter", (3, 4), Keywords::REACH, false),
            ("Gas Guzzler", (2, 1), Keywords::NONE, true),
            ("Gastal Raider", (2, 1), Keywords::NONE, false),
            ("Goblin Surveyor", (3, 2), Keywords::TRAMPLE, false),
            ("Mutant Surveyor", (2, 3), Keywords::NONE, false),
            ("Risen Necroregent", (5, 4), Keywords::NONE, false),
            (
                "Streaking Oilgorger",
                (3, 3),
                Keywords::FLYING.union(Keywords::HASTE),
                false,
            ),
            (
                "The Speed Demon",
                (5, 5),
                Keywords::FLYING.union(Keywords::TRAMPLE),
                false,
            ),
            (
                "Hazoret, Godseeker",
                (5, 3),
                Keywords::INDESTRUCTIBLE.union(Keywords::HASTE),
                false,
            ),
        ] {
            let mut g = scenario(p);
            let source = put(&mut g, p, name, Zone::Hand);
            cast(&mut g, source, None);
            drain(&mut g);
            assert_eq!(g.object_view(source).zone, Zone::Battlefield, "{name}");
            assert_eq!(g.state.players[p.index()].speed, 1, "{name}");
            assert_eq!(g.state.players[p.other().index()].speed, 0, "{name}");
            assert_eq!(stats(&g, source), body, "{name}");
            assert!(g.keywords(source).contains(keywords), "{name}");
            assert_eq!(g.object_view(source).tapped, tapped, "{name}");
            g.move_object(source, Zone::Hand);
            assert_eq!(
                g.state.players[p.index()].speed,
                1,
                "designation persists after {name} leaves"
            );
        }
    }
}

#[test]
fn max_speed_keywords_and_raider_bonus_follow_live_controller_speed() {
    for p in [Player::P0, Player::P1] {
        for (name, keyword, low, high) in [
            (
                "Burnout Bashtronaut",
                Keywords::DOUBLE_STRIKE,
                (1, 1),
                (1, 1),
            ),
            ("Gastal Raider", Keywords::MENACE, (2, 1), (3, 2)),
            ("Streaking Oilgorger", Keywords::LIFELINK, (3, 3), (3, 3)),
        ] {
            let mut g = scenario(p);
            let source = put(&mut g, p, name, Zone::Battlefield);
            for speed in [3, 4, 3] {
                g.state.players[p.index()].speed = speed;
                assert_eq!(g.keywords(source).contains(keyword), speed == 4, "{name}");
                assert_eq!(
                    stats(&g, source),
                    if speed == 4 { high } else { low },
                    "{name}"
                );
            }
            g.state.players[p.index()].speed = 4;
            g.state.players[p.other().index()].speed = 2;
            g.state.objects[source as usize].controller = p.other();
            assert!(!g.keywords(source).contains(keyword));
            assert_eq!(stats(&g, source), low);
        }
    }
}

#[test]
fn burnout_and_mutant_pumps_pay_two_each_and_expire_without_changing_base_stats() {
    for (name, base, toughness) in [
        ("Burnout Bashtronaut", (1, 1), false),
        ("Mutant Surveyor", (2, 3), true),
    ] {
        let mut g = scenario(Player::P0);
        let source = put(&mut g, Player::P0, name, Zone::Battlefield);
        for count in 1..=2 {
            g.state.players[0].mana = [0, 0, 0, 0, 0, 2];
            act(&mut g, announce(source, 0));
            pay(&mut g);
            drain(&mut g);
            assert_eq!(g.state.players[0].mana, [0; 6]);
            assert_eq!(
                stats(&g, source),
                (base.0 + count, base.1 + if toughness { count } else { 0 })
            );
        }
        g.finish_cleanup();
        assert_eq!(stats(&g, source), base);
    }
}

#[test]
fn endrider_catalyzer_max_speed_mana_funds_a_real_spell_and_requires_tap_readiness() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Endrider Catalyzer", Zone::Battlefield);
        let spell = put(&mut g, p, "Lightning Strike", Zone::Hand);
        g.state.players[p.index()].mana = [0; 6];
        g.state.players[p.index()].speed = 3;
        reject(&mut g, Action::AnnounceCast { card: spell });
        g.state.players[p.index()].speed = 4;
        g.state.objects[source as usize].summoning_sick = true;
        reject(&mut g, Action::AnnounceCast { card: spell });
        g.state.objects[source as usize].summoning_sick = false;
        act(&mut g, Action::AnnounceCast { card: spell });
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        choose(
            &mut g,
            |a| matches!(a,Action::ActivateMana{card,ability:0,production,..} if *card==source && *production==[0,0,0,2,0,0]),
        );
        assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 2, 0, 0]);
        assert!(g.object_view(source).tapped);
        pay(&mut g);
        drain(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(g.state.players[p.other().index()].life, 17);
    }
}

#[test]
fn black_and_red_surveyors_exile_the_owned_graveyard_card_at_owner_max_speed() {
    for p in [Player::P0, Player::P1] {
        for (name, ability) in [("Goblin Surveyor", 0), ("Mutant Surveyor", 1)] {
            let mut g = scenario(p);
            let source = put(&mut g, p, name, Zone::Graveyard);
            g.state.objects[source as usize].controller = p.other();
            g.state.players[p.other().index()].speed = 4;
            g.state.players[p.index()].speed = 3;
            let action = grant(&g, source, ability);
            reject(&mut g, action.clone());
            g.state.players[p.index()].speed = 4;
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
            act(&mut g, action);
            let saved = g.snapshot();
            let mut indexed = g.clone();
            let payment = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::PayCosts { .. }))
                .unwrap();
            let at = indexed
                .legal_actions()
                .iter()
                .position(|a| *a == payment)
                .unwrap();
            act(&mut g, payment);
            indexed.apply_index(p, indexed.revision(), at).unwrap();
            assert_eq!(g.snapshot(), indexed.snapshot());
            assert_eq!(g.object_view(source).zone, Zone::Exile);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
            drain(&mut g);
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
            indexed.restore(&saved).unwrap();
            pay(&mut indexed);
            drain(&mut indexed);
            assert_eq!(g.snapshot(), indexed.snapshot());
        }
    }
}

#[test]
fn spikespitter_exiles_only_on_own_max_speed_upkeep_and_queued_trigger_survives_speed_loss() {
    for p in [Player::P0, Player::P1] {
        for speed in [3, 4] {
            for own_turn in [false, true] {
                let mut g = scenario(p);
                let source = put(&mut g, p, "Endrider Spikespitter", Zone::Battlefield);
                let top = put(&mut g, p, "Mountain", Zone::Library);
                g.state.players[p.index()].speed = speed;
                g.state.active = if own_turn { p } else { p.other() };
                g.begin_step(Step::Upkeep);
                g.prepare_triggers();
                if speed == 4 && own_turn {
                    g.state.players[p.index()].speed = 3;
                    g.move_object(source, Zone::Hand);
                }
                drain(&mut g);
                assert_eq!(
                    g.object_view(top).zone,
                    if speed == 4 && own_turn {
                        Zone::Exile
                    } else {
                        Zone::Library
                    }
                );
            }
        }
    }
}

#[test]
fn spikespitter_exiled_land_and_spell_have_normal_paid_plays_only_this_turn() {
    for land in [false, true] {
        let p = Player::P0;
        let mut g = scenario(p);
        let source = put(&mut g, p, "Endrider Spikespitter", Zone::Battlefield);
        let top = put(
            &mut g,
            p,
            if land { "Mountain" } else { "Shock" },
            Zone::Library,
        );
        g.state.players[0].speed = 4;
        g.begin_step(Step::Upkeep);
        drain(&mut g);
        assert_eq!(g.object_view(top).zone, Zone::Exile);
        g.move_object(source, Zone::Graveyard);
        main_phase(&mut g, p);
        let mut expired = g.clone();
        expired.finish_cleanup();
        main_phase(&mut expired, p);
        expired.state.players[p.index()].mana = [8; 6];
        reject(
            &mut expired,
            if land {
                Action::PlayLand { card: top }
            } else {
                Action::AnnounceExile { card: top }
            },
        );
        if land {
            act(&mut g, Action::PlayLand { card: top });
            assert_eq!(g.object_view(top).zone, Zone::Battlefield);
        } else {
            g.state.players[0].mana = [0; 6];
            reject(&mut g, Action::AnnounceExile { card: top });
            g.state.players[0].mana = [0, 0, 0, 1, 0, 0];
            act(&mut g, Action::AnnounceExile { card: top });
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Player(p.other()),
                },
            );
            pay(&mut g);
            drain(&mut g);
            assert_eq!(g.state.players[1].life, 18);
            assert_eq!(g.object_view(top).zone, Zone::Graveyard);
            assert_eq!(g.state.players[0].mana, [0; 6]);
        }
    }
}

#[test]
fn gastal_raider_publicly_reveals_and_must_choose_an_instant_or_sorcery() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let instant = put(&mut g, p.other(), "Shock", Zone::Hand);
        let sorcery = put(&mut g, p.other(), "Divination", Zone::Hand);
        let creature = put(&mut g, p.other(), "Bear Cub", Zone::Hand);
        let land = put(&mut g, p.other(), "Mountain", Zone::Hand);
        let source = put(&mut g, p, "Gastal Raider", Zone::Hand);
        assert!(!g.observe(p).objects.iter().any(|o| o.id == instant));
        cast(&mut g, source, None);
        wait_hand(&mut g);
        for viewer in [p, p.other()] {
            let observation = g.observe(viewer);
            for id in [instant, sorcery, creature, land] {
                assert!(observation.known_hand.contains(&g.object_ref(id)));
            }
        }
        reject(&mut g, Action::ChooseHandCard { object: None });
        for id in [creature, land] {
            let object = g.object_ref(id);
            reject(
                &mut g,
                Action::ChooseHandCard {
                    object: Some(object),
                },
            );
        }
        let object = g.object_ref(sorcery);
        act(
            &mut g,
            Action::ChooseHandCard {
                object: Some(object),
            },
        );
        drain(&mut g);
        assert_eq!(g.object_view(sorcery).zone, Zone::Graveyard);
        assert_eq!(g.object_view(instant).zone, Zone::Hand);
        assert_eq!(
            g.state.players[p.other().index()].cards_discarded_this_turn,
            1
        );
    }
}

#[test]
fn gas_guzzler_sacrifices_another_creature_or_uncrewed_vehicle_only_at_max_speed() {
    for name in ["Bear Cub", "Clamorous Ironclad"] {
        let p = Player::P0;
        let mut g = scenario(p);
        let source = put(&mut g, p, "Gas Guzzler", Zone::Hand);
        cast(&mut g, source, None);
        drain(&mut g);
        assert!(g.object_view(source).tapped);
        let cost = put(&mut g, p, name, Zone::Battlefield);
        let artifact = put(&mut g, p, "Gilded Lotus", Zone::Battlefield);
        let foreign = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.state.players[0].speed = 3;
        let action = grant(&g, source, 0);
        reject(&mut g, action.clone());
        g.state.players[0].speed = 4;
        g.state.players[0].mana = [0, 0, 1, 0, 0, 0];
        act(&mut g, action);
        for id in [source, artifact, foreign] {
            let object = g.object_ref(id);
            reject(&mut g, Action::ChooseAdditionalCost { object });
        }
        let object = g.object_ref(cost);
        act(&mut g, Action::ChooseAdditionalCost { object });
        pay(&mut g);
        assert_eq!(g.object_view(cost).zone, Zone::Graveyard);
        assert_eq!(g.state.players[0].mana, [0; 6]);
        assert!(g.object_view(source).tapped);
        assert_eq!(g.state.players[0].cards_drawn_this_turn, 0);
        drain(&mut g);
        assert_eq!(g.state.players[0].cards_drawn_this_turn, 1);
        assert_eq!(g.object_view(source).zone, Zone::Battlefield);
    }
}

#[test]
fn risen_necroregent_creates_one_black_zombie_at_own_max_speed_end_only() {
    for p in [Player::P0, Player::P1] {
        for speed in [3, 4] {
            for own in [false, true] {
                let mut g = scenario(p);
                let source = put(&mut g, p, "Risen Necroregent", Zone::Battlefield);
                let before = g.state.objects.len();
                g.state.players[p.index()].speed = speed;
                g.state.active = if own { p } else { p.other() };
                g.begin_step(Step::End);
                g.prepare_triggers();
                if speed == 4 && own {
                    g.state.players[p.index()].speed = 3;
                    g.move_object(source, Zone::Hand);
                }
                drain(&mut g);
                let tokens: Vec<_> = g
                    .state
                    .objects
                    .iter()
                    .enumerate()
                    .skip(before)
                    .filter(|(_, o)| o.zone == Zone::Battlefield && o.is_token())
                    .map(|(id, _)| id as CardId)
                    .collect();
                assert_eq!(tokens.len(), usize::from(speed == 4 && own));
                for id in tokens {
                    assert_eq!(stats(&g, id), (2, 2));
                    assert_eq!(g.object_view(id).colors, 1 << 2);
                    assert_eq!(g.object_view(id).controller, p);
                    assert!(g.has_subtype(id, "Zombie"));
                    assert!(!g.object_view(id).tapped);
                }
            }
        }
    }
}

#[test]
fn speed_demons_end_step_draw_and_life_loss_use_current_speed_and_frozen_controller() {
    for p in [Player::P0, Player::P1] {
        for speed in [1, 3, 4] {
            let mut g = scenario(p);
            let source = put(&mut g, p, "The Speed Demon", Zone::Battlefield);
            g.state.players[p.index()].speed = speed;
            g.begin_step(Step::End);
            g.prepare_triggers();
            g.move_object(source, Zone::Hand);
            drain(&mut g);
            assert_eq!(
                g.state.players[p.index()].cards_drawn_this_turn,
                u64::from(speed)
            );
            assert_eq!(g.state.players[p.index()].life, 20 - i128::from(speed));
            assert_eq!(g.state.players[p.other().index()].life, 20);
        }
    }
    let mut g = scenario(Player::P0);
    put(&mut g, Player::P0, "The Speed Demon", Zone::Battlefield);
    g.state.players[0].speed = 2;
    g.begin_step(Step::End);
    g.prepare_triggers();
    g.state.players[0].speed = 4;
    drain(&mut g);
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 4);
    assert_eq!(g.state.players[0].life, 16);
}

#[test]
fn oilgorger_gains_life_from_actual_damage_only_when_its_controller_has_max_speed() {
    for speed in [3, 4] {
        let mut g = scenario(Player::P0);
        let source = put(&mut g, Player::P0, "Streaking Oilgorger", Zone::Battlefield);
        g.state.active = Player::P1;
        g.state.players[0].speed = speed;
        g.apply_damage_batch(&[DamageEvent {
            source: g.damage_source(source),
            target: Target::Player(Player::P1),
            amount: 3,
            combat: true,
            preventable: true,
        }]);
        g.advance_replacements(false);
        assert_eq!(g.state.players[1].life, 17);
        assert_eq!(g.state.players[0].life, if speed == 4 { 23 } else { 20 });
    }
}

#[test]
fn hazoret_cannot_attack_or_block_below_max_speed_and_survives_lethal_damage() {
    for speed in [3, 4] {
        let mut g = scenario(Player::P0);
        let source = put(&mut g, Player::P0, "Hazoret, Godseeker", Zone::Battlefield);
        g.state.players[0].speed = speed;
        g.state.objects[source as usize].summoning_sick = true;
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        let action = Action::Attack { card: source };
        if speed == 3 {
            reject(&mut g, action);
        } else {
            act(&mut g, action);
            assert_eq!(g.state.selected_attackers.len(), 1);
            act(&mut g, Action::FinishAttackers);
            assert_eq!(g.state.attackers.len(), 1);
        }
        let mut blocker = scenario(Player::P1);
        let god = put(
            &mut blocker,
            Player::P0,
            "Hazoret, Godseeker",
            Zone::Battlefield,
        );
        let attacker = put(&mut blocker, Player::P1, "Bear Cub", Zone::Battlefield);
        blocker.state.players[0].speed = speed;
        assert_eq!(blocker.can_block_pair(god, attacker), speed == 4);
    }
    let mut g = scenario(Player::P0);
    let source = put(&mut g, Player::P0, "Hazoret, Godseeker", Zone::Battlefield);
    let spell = put(&mut g, Player::P0, "Lightning Strike", Zone::Hand);
    cast(&mut g, spell, Some(Target::Card(source)));
    drain(&mut g);
    assert_eq!(g.object_view(source).zone, Zone::Battlefield);
    assert_eq!(g.object_view(source).damage, 3);
}

#[test]
fn hazoret_pays_one_and_tap_for_any_small_creature_and_unblockable_expires() {
    for own in [false, true] {
        let p = Player::P0;
        let mut g = scenario(p);
        let source = put(&mut g, p, "Hazoret, Godseeker", Zone::Battlefield);
        let owner = if own { p } else { p.other() };
        let target = put(&mut g, owner, "Bear Cub", Zone::Battlefield);
        let big = put(&mut g, owner, "Hill Giant", Zone::Battlefield);
        g.state.players[0].mana = [0, 0, 0, 0, 0, 1];
        act(&mut g, announce(source, 0));
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(big),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        pay(&mut g);
        drain(&mut g);
        assert!(g.object_view(source).tapped);
        assert_eq!(g.state.players[0].mana, [0; 6]);
        assert!(g.keywords(target).contains(Keywords::CANT_BE_BLOCKED));
        g.finish_cleanup();
        assert!(!g.keywords(target).contains(Keywords::CANT_BE_BLOCKED));
    }
}
