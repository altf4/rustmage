//! Real DFT speed permanents, graveyard activations and restricted blue mana.
mod support;
use rustmage::Card;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

fn drain(g: &mut Game) {
    for _ in 0..48 {
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty()
            && g.state.resolving.is_none()
        {
            return;
        }
        match g.state.pending {
            Pending::OrderTriggers { .. } => {
                choose(g, |action| matches!(action, Action::PutTrigger { .. }))
            }
            Pending::Priority => act(g, Action::Pass),
            _ => panic!("unexpected speed-family choice {:?}", g.state.pending),
        }
    }
    panic!("speed family resolution exceeded its action bound");
}
fn activate(g: &mut Game, source: CardId) {
    act(
        g,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    pay(g);
    drain(g);
}

#[test]
fn paid_speed_permanents_start_at_one_and_keep_their_printed_bodies() {
    for p in [Player::P0, Player::P1] {
        for (name, body, keywords) in [
            ("Aether Syphon", None, Keywords::NONE),
            ("Glitch Ghost Surveyor", Some((2, 2)), Keywords::FLYING),
            ("Leonin Surveyor", Some((2, 2)), Keywords::FIRST_STRIKE),
            ("Nesting Bot", Some((1, 1)), Keywords::NONE),
            ("Pride of the Road", Some((2, 5)), Keywords::VIGILANCE),
            ("Swiftwing Assailant", Some((3, 3)), Keywords::FLYING),
            ("Vnwxt, Verbose Host", Some((0, 4)), Keywords::NONE),
        ] {
            let mut g = scenario(p);
            let source = put(&mut g, p, name, Zone::Hand);
            cast(&mut g, source, None);
            drain(&mut g);
            assert_eq!(g.state.players[p.index()].speed, 1, "{name}");
            assert_eq!(g.state.players[p.other().index()].speed, 0, "{name}");
            assert_eq!(g.object_view(source).zone, Zone::Battlefield, "{name}");
            if let Some((power, toughness)) = body {
                assert_eq!(
                    (g.stats(source).power, g.stats(source).toughness),
                    (power, toughness),
                    "{name}"
                );
            }
            assert!(g.keywords(source).contains(keywords), "{name}");
        }
    }
}

#[test]
fn swiftwing_and_leonin_follow_live_speed_turn_and_controller() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let bird = put(&mut g, p, "Swiftwing Assailant", Zone::Battlefield);
        let cat = put(&mut g, p, "Leonin Surveyor", Zone::Battlefield);
        g.state.players[p.index()].speed = 3;
        assert_eq!((g.stats(bird).power, g.stats(bird).toughness), (3, 3));
        assert!(g.keywords(bird).contains(Keywords::FLYING));
        assert!(!g.keywords(bird).contains(Keywords::VIGILANCE));
        assert!(g.keywords(cat).contains(Keywords::FIRST_STRIKE));
        g.state.players[p.index()].speed = 4;
        assert_eq!((g.stats(bird).power, g.stats(bird).toughness), (3, 4));
        assert!(g.keywords(bird).contains(Keywords::VIGILANCE));
        g.state.active = p.other();
        assert!(!g.keywords(cat).contains(Keywords::FIRST_STRIKE));
        g.state.objects[bird as usize].controller = p.other();
        assert_eq!(g.stats(bird).toughness, 3);
        assert!(!g.keywords(bird).contains(Keywords::VIGILANCE));
    }
}

#[test]
fn aether_syphon_mills_for_each_actual_controller_draw_only_at_max_speed() {
    for p in [Player::P0, Player::P1] {
        for speed in [3, 4] {
            let mut g = scenario(p);
            let source = put(&mut g, p, "Aether Syphon", Zone::Battlefield);
            g.state.players[p.index()].speed = speed;
            activate(&mut g, source);
            assert!(g.object_view(source).tapped);
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
            assert_eq!(
                g.state.players[p.other().index()].library.len(),
                if speed == 4 { 6 } else { 8 }
            );
            let spell = put(&mut g, p, "Divination", Zone::Hand);
            cast(&mut g, spell, None);
            drain(&mut g);
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 3);
            assert_eq!(
                g.state.players[p.other().index()].library.len(),
                if speed == 4 { 2 } else { 8 }
            );
            g.draw_many(p.other(), 1);
            drain(&mut g);
            assert_eq!(g.state.players[p.index()].library.len(), 5);
        }
    }
}

#[test]
fn graveyard_surveyors_require_owner_max_speed_and_exile_as_a_paid_cost() {
    for p in [Player::P0, Player::P1] {
        for name in ["Glitch Ghost Surveyor", "Leonin Surveyor"] {
            let mut g = scenario(p);
            let source = put(&mut g, p, name, Zone::Graveyard);
            g.state.objects[source as usize].controller = p.other();
            let action = Action::AnnounceSourceGrant {
                source: g.object_ref(source),
                ability: 0,
            };
            g.state.players[p.other().index()].speed = 4;
            g.state.players[p.index()].speed = 3;
            reject(&mut g, action.clone());
            g.state.players[p.index()].speed = 4;
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
            act(&mut g, action);
            let saved = g.snapshot();
            let mut indexed = g.clone();
            pay(&mut g);
            let payment = indexed
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::PayCosts { .. }))
                .unwrap();
            let index = indexed
                .legal_actions()
                .iter()
                .position(|a| *a == payment)
                .unwrap();
            indexed
                .apply_index(indexed.actor().unwrap(), indexed.revision(), index)
                .unwrap();
            assert_eq!(g.snapshot(), indexed.snapshot());
            assert_eq!(g.object_view(source).zone, Zone::Exile, "{name}");
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            drain(&mut g);
            assert_eq!(
                g.state.players[p.index()].cards_drawn_this_turn,
                1,
                "{name}"
            );
            indexed.restore(&saved).unwrap();
            pay(&mut indexed);
            drain(&mut indexed);
            assert_eq!(g.snapshot(), indexed.snapshot());
        }
    }
}

#[test]
fn nesting_bot_death_always_creates_exactly_one_colorless_artifact_servo() {
    for p in [Player::P0, Player::P1] {
        for speed in [3, 4] {
            let mut g = scenario(p);
            let bot = put(&mut g, p, "Nesting Bot", Zone::Battlefield);
            g.state.players[p.index()].speed = speed;
            assert_eq!(
                (g.stats(bot).power, g.stats(bot).toughness),
                (if speed == 4 { 2 } else { 1 }, 1)
            );
            g.move_object(bot, Zone::Graveyard);
            drain(&mut g);
            let servos = g
                .state
                .objects
                .iter()
                .enumerate()
                .filter_map(|(id, o)| {
                    let id = id as CardId;
                    (o.zone == Zone::Battlefield && o.controller == p && g.has_subtype(id, "Servo"))
                        .then_some(id)
                })
                .collect::<Vec<_>>();
            assert_eq!(servos.len(), 1);
            let token = servos[0];
            assert!(g.state.objects[token as usize].is_token());
            assert_eq!(g.current_kinds(token), 1 | 16);
            assert_eq!(g.object_view(token).colors, 0);
            assert_eq!((g.stats(token).power, g.stats(token).toughness), (1, 1));
        }
    }
}

#[test]
fn pride_triggers_at_max_speed_and_can_grant_double_strike_to_an_uncrewed_vehicle() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let pride = put(&mut g, p, "Pride of the Road", Zone::Battlefield);
        let vehicle = put(&mut g, p, "Air Response Unit", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Air Response Unit", Zone::Battlefield);
        g.state.players[p.index()].speed = 3;
        g.begin_step(Step::BeginCombat);
        g.give_priority();
        assert!(g.state.pending_triggers.is_empty());
        g.state.players[p.index()].speed = 4;
        g.begin_step(Step::BeginCombat);
        g.give_priority();
        for _ in 0..8 {
            if g.legal_actions().contains(&Action::ChooseTarget {
                target: Target::Card(vehicle),
            }) {
                break;
            }
            act(&mut g, Action::Pass);
        }
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(vehicle),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            act(&mut g, Action::FinishTargets);
        }
        g.move_object(pride, Zone::Graveyard);
        drain(&mut g);
        assert!(g.keywords(vehicle).contains(Keywords::DOUBLE_STRIKE));
        assert_eq!(g.current_kinds(vehicle) & 1, 0);
        g.finish_cleanup();
        assert!(!g.keywords(vehicle).contains(Keywords::DOUBLE_STRIKE));
    }
}

#[test]
fn vnwxt_doubles_the_normal_draw_and_loses_both_static_effects_when_blank() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let host = put(&mut g, p, "Vnwxt, Verbose Host", Zone::Battlefield);
        g.state.players[p.index()].speed = 3;
        assert_eq!(g.maximum_hand_size(p), None);
        g.draw_many(p, 1);
        assert_eq!(g.hand_count(p), 1);
        g.state.players[p.index()].speed = 4;
        g.state.step = Step::Draw;
        g.state.players[p.index()].drew_in_own_draw_step = false;
        g.draw_many(p, 1);
        assert_eq!(g.hand_count(p), 3);
        g.draw_many(p.other(), 1);
        assert_eq!(g.hand_count(p.other()), 1);
        g.state.step = Step::Main1;
        let blank = put(&mut g, p, "Witness Protection", Zone::Hand);
        cast(&mut g, blank, Some(Target::Card(host)));
        drain(&mut g);
        assert_eq!(g.maximum_hand_size(p), Some(7));
        g.draw_many(p, 1);
        assert_eq!(g.hand_count(p), 4);
    }
}

#[test]
fn guidelight_optimizer_blue_pays_artifacts_and_activations_but_not_nonartifact_spells() {
    for p in [Player::P0, Player::P1] {
        for activated in [false, true] {
            let mut g = scenario(p);
            let robot = put(&mut g, p, "Guidelight Optimizer", Zone::Battlefield);
            let syphon = put(&mut g, p, "Aether Syphon", Zone::Battlefield);
            let invalid = put(&mut g, p, "Divination", Zone::Hand);
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, if activated { 1 } else { 2 }];
            let mana = Action::ActivateMana {
                card: robot,
                ability: 0,
                payment: [0; 6],
                counter_payment: Counters::default(),
                production: [0, 1, 0, 0, 0, 0],
            };
            act(&mut g, mana.clone());
            assert!(g.object_view(robot).tapped);
            reject(&mut g, mana);
            reject(&mut g, Action::AnnounceCast { card: invalid });
            if activated {
                activate(&mut g, syphon);
                assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
            } else {
                let artifact = put(&mut g, p, "Guidelight Optimizer", Zone::Hand);
                cast(&mut g, artifact, None);
                drain(&mut g);
                assert_eq!(g.object_view(artifact).zone, Zone::Battlefield);
            }
            assert_eq!(
                g.state.players[p.index()].mana.iter().sum::<u32>(),
                if activated { 0 } else { 1 }
            );
        }
    }
}

#[test]
fn dft_vanillas_have_no_rules_abilities_and_pay_for_their_printed_bodies() {
    for (name, body) in [
        ("Caelorna, Coral Tyrant", (0, 8)),
        ("Sundial, Dawn Tyrant", (3, 3)),
    ] {
        let mut g = scenario(Player::P0);
        let source = put(&mut g, Player::P0, name, Zone::Hand);
        cast(&mut g, source, None);
        drain(&mut g);
        assert_eq!((g.stats(source).power, g.stats(source).toughness), body);
        assert!(!g.object_has_rules_abilities(source));
        assert_eq!(g.keywords(source), Keywords::NONE);
    }
}
