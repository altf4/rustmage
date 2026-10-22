//! Evoke is an alternative cast cost; colored mana qualification uses actual payment.
use super::ability_timeline_tests::{checked, named, setup};
use super::*;
fn cast_route(g: &mut Game, id: CardId, cost: ManaCost) {
    checked(g, Action::AnnounceCast { card: id });
    let choice = g
        .observe(g.actor().unwrap())
        .cost_choices
        .into_iter()
        .find(|c| c.cost == cost)
        .unwrap();
    checked(g, Action::ChooseSpellCost { cost: choice.index });
    let action = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    checked(g, action);
}
#[test]
fn ecl_evoke_vibrance_three_routes_and_both_seats() {
    for p in [Player::P0, Player::P1] {
        for (r, green) in [(2, 0), (1, 1), (0, 2)] {
            let mut g = setup(p);
            let id = named(&mut g, p, "Vibrance", Zone::Hand);
            g.state.players[p.index()].mana = [0, 0, 0, r, green, 0];
            cast_route(
                &mut g,
                id,
                ManaCost {
                    colored: [0, 0, 0, r, green, 0],
                    generic: 0,
                },
            );
            assert!(g.state.stack.last().unwrap().cast.unwrap().evoked);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            resolve(&mut g, Some(Target::Player(p.other())), None);
            assert_eq!(g.state.objects[id as usize].zone, Zone::Graveyard);
            assert_eq!(
                g.state.players[p.index()].life,
                if green == 2 { 22 } else { 20 }
            );
            assert_eq!(
                g.state.players[p.other().index()].life,
                if r == 2 { 17 } else { 20 }
            );
        }
    }
}
#[test]
fn ecl_evoke_normal_generic_payment_can_qualify_both_entry_abilities() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = named(&mut g, p, "Vibrance", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 2, 3, 0];
        cast_route(
            &mut g,
            id,
            ManaCost {
                colored: [0, 0, 0, 0, 2, 0],
                generic: 3,
            },
        );
        assert!(!g.state.stack.last().unwrap().cast.unwrap().evoked);
        assert_eq!(
            g.state.stack.last().unwrap().cast.unwrap().payment,
            [0, 0, 0, 2, 3, 0]
        );
        resolve(&mut g, Some(Target::Player(p.other())), None);
        assert_eq!(g.state.objects[id as usize].zone, Zone::Battlefield);
        assert_eq!(g.state.players[p.index()].life, 22);
        assert_eq!(g.state.players[p.other().index()].life, 17);
    }
}
#[test]
fn ecl_evoke_uncast_entries_do_not_inherit_cast_qualification() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = named(&mut g, p, "Vibrance", Zone::Hand);
        g.move_object(id, Zone::Battlefield);
        resolve(&mut g, Some(Target::Player(p.other())), None);
        assert_eq!(g.state.objects[id as usize].zone, Zone::Battlefield);
        assert_eq!(g.state.players[p.index()].life, 20);
        assert_eq!(g.state.players[p.other().index()].life, 20);
    }
}

fn resolve(g: &mut Game, target: Option<Target>, search: Option<CardId>) {
    g.prepare_triggers();
    for _ in 0..100 {
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty()
            && g.state.resolving.is_none()
            && g.state.announcing.is_none()
        {
            return;
        }
        let actions = g.legal_actions();
        let chosen = target
            .map(|target| Action::ChooseTarget { target })
            .filter(|a| actions.contains(a));
        let searched = search
            .map(|card| Action::ChooseSearch { card })
            .filter(|a| actions.contains(a));
        let action = chosen.or(searched).unwrap_or_else(|| {
            if g.state.pending == Pending::Priority {
                Action::Pass
            } else {
                actions
                    .iter()
                    .find(|a| matches!(a, Action::FinishTargets | Action::FinishSearch))
                    .or_else(|| actions.iter().find(|a| !matches!(a, Action::Concede)))
                    .unwrap()
                    .clone()
            }
        });
        checked(g, action);
    }
    panic!("unsettled {:?}", g.state.pending);
}
#[test]
fn ecl_evoke_catharsis_three_payments_token_and_haste_branches() {
    for p in [Player::P0, Player::P1] {
        for w in 0..=2 {
            let mut g = setup(p);
            let body = named(&mut g, p, "Hill Giant", Zone::Battlefield);
            let id = named(&mut g, p, "Catharsis", Zone::Hand);
            let cost = ManaCost {
                colored: [w, 0, 0, 2 - w, 0, 0],
                generic: 0,
            };
            g.state.players[p.index()].mana = cost.colored;
            cast_route(&mut g, id, cost);
            resolve(&mut g, None, None);
            assert_eq!(g.state.objects[id as usize].zone, Zone::Graveyard);
            assert_eq!(g.stats(body).power, if w == 0 { 4 } else { 3 });
            assert_eq!(g.keywords(body).contains(Keywords::HASTE), w == 0);
            let tokens = g
                .state
                .objects
                .iter()
                .filter(|o| {
                    o.zone == Zone::Battlefield
                        && o.controller == p
                        && o.is_token()
                        && o.card == Card::from_name("@token/c48e9e904360e4cee1deeed8").unwrap()
                })
                .count();
            assert_eq!(tokens, if w == 2 { 2 } else { 0 });
            g.finish_cleanup();
            assert_eq!(g.stats(body).power, 3);
            assert!(!g.keywords(body).contains(Keywords::HASTE));
        }
    }
}
#[test]
fn ecl_evoke_emptiness_reanimates_small_creature_or_puts_three_counters() {
    for p in [Player::P0, Player::P1] {
        for white in [true, false] {
            let mut g = setup(p);
            let grave = named(&mut g, p, "Bear Cub", Zone::Graveyard);
            let foe = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
            let id = named(&mut g, p, "Emptiness", Zone::Hand);
            let mut payment = [0; 6];
            payment[if white { 0 } else { 2 }] = 2;
            g.state.players[p.index()].mana = payment;
            cast_route(
                &mut g,
                id,
                ManaCost {
                    colored: payment,
                    generic: 0,
                },
            );
            resolve(
                &mut g,
                Some(Target::Card(if white { grave } else { foe })),
                None,
            );
            assert_eq!(
                g.state.objects[grave as usize].zone,
                if white {
                    Zone::Battlefield
                } else {
                    Zone::Graveyard
                }
            );
            assert_eq!(
                g.state.objects[foe as usize].zone,
                if white {
                    Zone::Battlefield
                } else {
                    Zone::Graveyard
                }
            );
            assert_eq!(g.state.objects[id as usize].zone, Zone::Graveyard);
        }
    }
}
#[test]
fn ecl_evoke_deceit_bounces_other_permanent_or_reveals_and_discards_nonland() {
    for p in [Player::P0, Player::P1] {
        for blue in [true, false] {
            let mut g = setup(p);
            let foe = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
            let discard = named(&mut g, p.other(), "Bear Cub", Zone::Hand);
            let land = named(&mut g, p.other(), "Forest", Zone::Hand);
            let id = named(&mut g, p, "Deceit", Zone::Hand);
            let mut payment = [0; 6];
            payment[if blue { 1 } else { 2 }] = 2;
            g.state.players[p.index()].mana = payment;
            cast_route(
                &mut g,
                id,
                ManaCost {
                    colored: payment,
                    generic: 0,
                },
            );
            resolve(
                &mut g,
                Some(if blue {
                    Target::Card(foe)
                } else {
                    Target::Player(p.other())
                }),
                None,
            );
            assert_eq!(
                g.state.objects[foe as usize].zone,
                if blue { Zone::Hand } else { Zone::Battlefield }
            );
            assert_eq!(
                g.state.objects[discard as usize].zone,
                if blue { Zone::Hand } else { Zone::Graveyard }
            );
            assert_eq!(g.state.objects[land as usize].zone, Zone::Hand);
            assert_eq!(g.state.objects[id as usize].zone, Zone::Graveyard);
        }
    }
}
#[test]
fn ecl_evoke_wistfulness_exiles_opponents_artifact_or_draws_then_discards() {
    for p in [Player::P0, Player::P1] {
        for blue in [true, false] {
            let mut g = setup(p);
            let foe = named(&mut g, p.other(), "Hedron Archive", Zone::Battlefield);
            let id = named(&mut g, p, "Wistfulness", Zone::Hand);
            let hand = g.hand_count(p);
            let mut payment = [0; 6];
            payment[if blue { 1 } else { 4 }] = 2;
            g.state.players[p.index()].mana = payment;
            cast_route(
                &mut g,
                id,
                ManaCost {
                    colored: payment,
                    generic: 0,
                },
            );
            resolve(&mut g, Some(Target::Card(foe)), None);
            assert_eq!(
                g.state.objects[foe as usize].zone,
                if blue { Zone::Battlefield } else { Zone::Exile }
            );
            assert_eq!(g.hand_count(p), if blue { hand } else { hand - 1 });
            assert_eq!(g.state.objects[id as usize].zone, Zone::Graveyard);
        }
    }
}

#[test]
fn ecl_evoke_free_casts_cannot_use_a_second_alternative_or_inherit_payment() {
    for p in [Player::P0, Player::P1] {
        for name in [
            "Vibrance",
            "Catharsis",
            "Deceit",
            "Emptiness",
            "Wistfulness",
        ] {
            let mut g = setup(p);
            named(&mut g, p, "Omniscience", Zone::Battlefield);
            let id = named(&mut g, p, name, Zone::Hand);
            g.state.players[p.index()].mana = [0; 6];
            checked(
                &mut g,
                Action::AnnounceFree {
                    card: id,
                    face: CastFace::Normal,
                    flash_surcharge: false,
                },
            );
            assert!(
                !g.legal_actions()
                    .iter()
                    .any(|a| matches!(a, Action::ChooseSpellCost { .. }))
            );
            let action = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::PayCosts { .. }))
                .unwrap();
            checked(&mut g, action);
            let cast = g.state.stack.last().unwrap().cast.unwrap();
            assert!(!cast.evoked);
            assert_eq!(cast.payment, [0; 6]);
            resolve(&mut g, None, None);
            assert_eq!(g.state.objects[id as usize].zone, Zone::Battlefield);
            assert_eq!(g.state.players[p.index()].life, 20);
            assert_eq!(g.state.players[p.other().index()].life, 20);
        }
    }
}
fn enter(g: &mut Game, id: CardId) {
    for _ in 0..8 {
        if g.on_battlefield(id) {
            return;
        }
        checked(g, Action::Pass);
    }
    panic!("missing entry");
}
#[test]
fn ecl_evoke_exact_entry_sacrifice_follows_current_controller_and_never_rebinds() {
    for p in [Player::P0, Player::P1] {
        for reenter in [true, false] {
            let mut g = setup(p);
            let id = named(&mut g, p, "Vibrance", Zone::Hand);
            g.state.players[p.index()].mana = [0, 0, 0, 1, 1, 0];
            cast_route(
                &mut g,
                id,
                ManaCost {
                    colored: [0, 0, 0, 1, 1, 0],
                    generic: 0,
                },
            );
            enter(&mut g, id);
            if reenter {
                g.move_object(id, Zone::Exile);
                g.move_object(id, Zone::Battlefield);
            } else {
                let object = g.object_ref(id);
                g.change_controller(object, p.other());
            }
            resolve(&mut g, None, None);
            assert_eq!(
                g.state.objects[id as usize].zone,
                if reenter {
                    Zone::Battlefield
                } else {
                    Zone::Graveyard
                }
            );
        }
    }
}
#[test]
fn ecl_evoke_red_damage_can_resolve_after_its_source_has_been_sacrificed() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = named(&mut g, p, "Vibrance", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 2, 0, 0];
        cast_route(
            &mut g,
            id,
            ManaCost {
                colored: [0, 0, 0, 2, 0, 0],
                generic: 0,
            },
        );
        enter(&mut g, id);
        let damage = g
            .state
            .ordering_triggers
            .iter()
            .find(|t| t.kind == StackKind::TriggeredAbility { ability: 0 })
            .unwrap()
            .id;
        checked(&mut g, Action::PutTrigger { id: damage });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        assert_eq!(
            g.state.stack.last().unwrap().kind,
            StackKind::TriggeredAbility { ability: 2 }
        );
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        assert_eq!(g.state.objects[id as usize].zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.other().index()].life, 20);
        resolve(&mut g, None, None);
        assert_eq!(g.state.players[p.other().index()].life, 17);
    }
}
#[test]
fn ecl_evoke_vibrance_search_is_private_reveals_selection_and_gains_life_after_decline() {
    for p in [Player::P0, Player::P1] {
        for find in [true, false] {
            let mut g = setup(p);
            let land = named(&mut g, p, "Forest", Zone::Library);
            let bad = named(&mut g, p, "Bear Cub", Zone::Library);
            let id = named(&mut g, p, "Vibrance", Zone::Hand);
            g.state.players[p.index()].mana = [0, 0, 0, 0, 2, 0];
            cast_route(
                &mut g,
                id,
                ManaCost {
                    colored: [0, 0, 0, 0, 2, 0],
                    generic: 0,
                },
            );
            for _ in 0..30 {
                if matches!(g.state.pending, Pending::EffectSearch { .. }) {
                    break;
                }
                let actions = g.legal_actions();
                let a = if g.state.pending == Pending::Priority {
                    Action::Pass
                } else {
                    actions
                        .into_iter()
                        .find(|a| matches!(a, Action::PutTrigger { .. }))
                        .unwrap()
                };
                checked(&mut g, a);
            }
            assert!(g.observe(p.other()).search_choice.is_none());
            assert!(
                !g.legal_actions()
                    .contains(&Action::ChooseSearch { card: bad })
            );
            if find {
                checked(&mut g, Action::ChooseSearch { card: land });
            }
            checked(&mut g, Action::FinishSearch);
            resolve(&mut g, None, None);
            assert_eq!(
                g.state.objects[land as usize].zone,
                if find { Zone::Hand } else { Zone::Library }
            );
            assert_eq!(g.state.players[p.index()].life, 22);
            if find {
                assert!(
                    g.observe(p.other())
                        .known_hand
                        .iter()
                        .any(|o| o.card == land)
                );
            }
        }
    }
}

#[test]
fn ecl_evoke_catharsis_normal_cost_both_colors_allows_both_trigger_orders() {
    for p in [Player::P0, Player::P1] {
        for tokens_first in [true, false] {
            let mut g = setup(p);
            let id = named(&mut g, p, "Catharsis", Zone::Hand);
            g.state.players[p.index()].mana = [2, 0, 0, 2, 0, 2];
            cast_route(
                &mut g,
                id,
                ManaCost {
                    colored: [0, 0, 0, 2, 0, 0],
                    generic: 4,
                },
            );
            enter(&mut g, id);
            assert!(
                !g.state
                    .stack
                    .iter()
                    .any(|t| t.kind == StackKind::TriggeredAbility { ability: 2 })
            );
            let ability = if tokens_first { 1 } else { 0 };
            let first = g
                .state
                .ordering_triggers
                .iter()
                .find(|t| t.kind == StackKind::TriggeredAbility { ability })
                .unwrap()
                .id;
            checked(&mut g, Action::PutTrigger { id: first });
            resolve(&mut g, None, None);
            let tokens: Vec<_> = g
                .state
                .objects
                .iter()
                .enumerate()
                .filter(|(_, o)| {
                    o.zone == Zone::Battlefield
                        && o.controller == p
                        && o.is_token()
                        && o.card == Card::from_name("@token/c48e9e904360e4cee1deeed8").unwrap()
                })
                .map(|(i, _)| i as CardId)
                .collect();
            assert_eq!(tokens.len(), 2);
            for token in tokens {
                assert_eq!(g.stats(token).power, if tokens_first { 2 } else { 1 });
                assert_eq!(g.keywords(token).contains(Keywords::HASTE), tokens_first);
            }
            assert_eq!(
                g.stats(id),
                Stats {
                    power: 4,
                    toughness: 5
                }
            );
        }
    }
}
