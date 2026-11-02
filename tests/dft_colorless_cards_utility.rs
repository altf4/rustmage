//! Three artifact cards with paid abilities and an intervening entry condition.
mod support;
use rustmage as _;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

fn action(card: CardId, ability: u16) -> Action {
    Action::AnnounceActivate {
        card,
        ability,
        counter_payment: Counters::default(),
    }
}
fn drain(g: &mut Game) {
    for _ in 0..48 {
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty()
            && g.state.resolving.is_none()
        {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("bounded utility artifact resolution");
}
fn reach_entry(g: &mut Game, source: CardId) {
    for _ in 0..32 {
        if g.object_view(source).zone == Zone::Battlefield {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("paid permanent must enter");
}
#[test]
fn utility_artifacts_have_their_actual_paid_bodies_and_matrix_draws_one() {
    for p in [Player::P0, Player::P1] {
        for (name, cost) in [
            ("Aetherjacket", 3),
            ("Guidelight Matrix", 2),
            ("Ticket Tortoise", 2),
        ] {
            let mut g = scenario(p);
            let source = put(&mut g, p, name, Zone::Hand);
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, cost];
            cast(&mut g, source, None);
            drain(&mut g);
            assert_eq!(g.object_view(source).zone, Zone::Battlefield);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert_eq!(g.object_view(source).colors, 0);
            match name {
                "Aetherjacket" => {
                    assert_eq!(stats(&g, source), (2, 1));
                    assert!(
                        g.keywords(source)
                            .contains(Keywords::FLYING.union(Keywords::VIGILANCE))
                    );
                    assert!(g.has_subtype(source, "Thopter"));
                }
                "Ticket Tortoise" => {
                    assert_eq!(stats(&g, source), (3, 1));
                    assert!(g.keywords(source).contains(Keywords::DEFENDER));
                    assert!(g.has_subtype(source, "Turtle"));
                }
                _ => {
                    assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
                    assert_eq!(g.state.players[p.index()].library.len(), 7);
                }
            }
        }
    }
}
#[test]
fn aetherjacket_pays_two_taps_and_sacrifices_at_sorcery_speed_to_destroy_another_artifact() {
    for p in [Player::P0, Player::P1] {
        for indestructible in [false, true] {
            let mut g = scenario(p);
            let source = put(&mut g, p, "Aetherjacket", Zone::Battlefield);
            let victim = put(&mut g, p.other(), "Swiftfoot Boots", Zone::Battlefield);
            let creature = put(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
            if indestructible {
                g.state.objects[victim as usize].gained_keywords = Keywords::INDESTRUCTIBLE;
            }
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
            g.state.step = Step::Upkeep;
            reject(&mut g, action(source, 0));
            g.state.step = Step::Main1;
            g.state.objects[source as usize].tapped = true;
            reject(&mut g, action(source, 0));
            g.state.objects[source as usize].tapped = false;
            act(&mut g, action(source, 0));
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(source),
                },
            );
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(creature),
                },
            );
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(victim),
                },
            );
            if g.legal_actions().contains(&Action::FinishTargets) {
                act(&mut g, Action::FinishTargets);
            }
            pay(&mut g);
            assert_eq!(g.object_view(source).zone, Zone::Graveyard);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert_eq!(g.object_view(victim).zone, Zone::Battlefield);
            drain(&mut g);
            assert_eq!(
                g.object_view(victim).zone,
                if indestructible {
                    Zone::Battlefield
                } else {
                    Zone::Graveyard
                }
            );
        }
    }
}
#[test]
fn guidelight_matrix_uses_separate_mount_sorcery_and_vehicle_instant_abilities() {
    for p in [Player::P0, Player::P1] {
        for vehicle in [false, true] {
            let mut g = scenario(p);
            let matrix = put(&mut g, p, "Guidelight Matrix", Zone::Battlefield);
            let name = if vehicle {
                "Cultivator's Caravan"
            } else {
                "Trained Arynx"
            };
            let target = put(&mut g, p, name, Zone::Battlefield);
            let foreign = put(&mut g, p.other(), name, Zone::Battlefield);
            let plain = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
            let helper = put(&mut g, p, "Llanowar Elves", Zone::Battlefield);
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
            let ability = u16::from(vehicle);
            g.state.step = Step::Upkeep;
            if !vehicle {
                reject(&mut g, action(matrix, ability));
                g.state.step = Step::Main1;
            }
            act(&mut g, action(matrix, ability));
            for id in [foreign, plain] {
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
                    target: Target::Card(target),
                },
            );
            if g.legal_actions().contains(&Action::FinishTargets) {
                act(&mut g, Action::FinishTargets);
            }
            pay(&mut g);
            assert!(g.object_view(matrix).tapped);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            let mut stale = g.clone();
            stale.move_object(target, Zone::Exile);
            stale.move_object(target, Zone::Battlefield);
            drain(&mut stale);
            assert!(!stale.object_view(target).saddled);
            if vehicle {
                assert!(!stale.is_creature(target));
            }
            drain(&mut g);
            assert!(
                !g.object_view(helper).tapped,
                "direct designation/animation has no crew or saddle participants"
            );
            if vehicle {
                assert!(g.is_creature(target));
                assert_eq!(stats(&g, target), (5, 5));
            } else {
                assert!(g.object_view(target).saddled);
            }
            g.finish_cleanup();
            if vehicle {
                assert!(!g.is_creature(target));
            } else {
                assert!(!g.object_view(target).saddled);
            }
        }
    }
}
#[test]
fn ticket_tortoise_checks_more_lands_on_entry_and_again_at_resolution() {
    for p in [Player::P0, Player::P1] {
        for (own, opposing, keep, expected) in [
            (0, 0, true, 0),
            (1, 1, true, 0),
            (0, 1, false, 0),
            (1, 2, true, 1),
        ] {
            let mut g = scenario(p);
            for _ in 0..own {
                put(&mut g, p, "Forest", Zone::Battlefield);
            }
            for _ in 0..opposing {
                put(&mut g, p.other(), "Forest", Zone::Battlefield);
            }
            let source = put(&mut g, p, "Ticket Tortoise", Zone::Hand);
            cast(&mut g, source, None);
            reach_entry(&mut g, source);
            if !keep {
                put(&mut g, p, "Forest", Zone::Battlefield);
            }
            if own == opposing {
                put(&mut g, p.other(), "Forest", Zone::Battlefield);
            }
            // The captured trigger must work after its source departs.
            g.move_object(source, Zone::Graveyard);
            drain(&mut g);
            let treasures = g
                .state
                .objects
                .iter()
                .enumerate()
                .filter(|(i, o)| {
                    o.zone == Zone::Battlefield
                        && o.controller == p
                        && o.is_token()
                        && g.has_subtype(*i as CardId, "Treasure")
                })
                .map(|(i, _)| i as CardId)
                .collect::<Vec<_>>();
            assert_eq!(treasures.len(), expected);
            if let Some(&treasure) = treasures.first() {
                assert_eq!(g.object_view(treasure).colors, 0);
                assert!(!g.is_creature(treasure));
                assert!(!g.object_view(treasure).tapped);
                g.state.players[p.index()].mana = [0; 6];
                choose(
                    &mut g,
                    |a| matches!(a,Action::ActivateMana {card,production,..}if *card==treasure&&*production==[0,0,0,0,1,0]),
                );
                assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 0, 1, 0]);
                assert!(!g.on_battlefield(treasure));
            }
        }
    }
}
