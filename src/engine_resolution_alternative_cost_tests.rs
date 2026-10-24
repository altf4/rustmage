//! One private resolution cost, two disjoint payment routes, owned continuations.
use super::assurance::big_helpers::{cast, checked, named, settle, setup};
use super::*;
fn reject(g: &mut Game, a: Action) {
    let before = g.snapshot();
    let bytes = g.state_bytes();
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.state_bytes(), bytes);
}
fn vision(g: &mut Game, p: Player) -> CardId {
    let spell = named(g, p, "Vision of Love", Zone::Hand);
    let mana = g.state.players[p.index()].mana;
    cast(g, spell, None, &[]);
    let remaining = g.state.players[p.index()].mana;
    assert_eq!(mana.iter().sum::<u32>() - remaining.iter().sum::<u32>(), 2);
    assert!(remaining[3] < mana[3]);
    for _ in 0..10 {
        if matches!(g.state.pending, Pending::EffectChoice { .. }) {
            assert_eq!(g.actor(), Some(p));
            return spell;
        }
        checked(g, Action::Pass);
    }
    panic!("missing optional payment");
}
fn draws(g: &Game, p: Player) -> u64 {
    g.state.players[p.index()].cards_drawn_this_turn
}
fn typed_union_fixture(g: &mut Game, nonland: bool, optional: bool) {
    let r = g.state.resolving.as_mut().unwrap();
    let old = r.choice.take().unwrap();
    let item = r.item.clone();
    assert!(g.start_effect_choice(
        &item,
        EffectChoiceKind::SacrificeOrDiscard { nonland },
        PlayerSet::Controller,
        old.filter,
        1,
        optional.then_some(1),
    ));
}
#[test]
fn alternative_cost_vision_paid_both_routes_including_land_and_opponent_turn() {
    for p in [Player::P0, Player::P1] {
        for discard in [false, true] {
            let mut g = setup(p.other());
            checked(&mut g, Action::Pass);
            let artifact = named(&mut g, p, "Gilded Lotus", Zone::Battlefield);
            let hand = named(&mut g, p, "Forest", Zone::Hand);
            let spell = vision(&mut g, p);
            let view = g.observe(p).effect_choice.unwrap();
            assert_eq!(view.required[p.index()], 1);
            assert!(view.optional);
            assert!(view.candidates.contains(&g.object_ref(artifact)));
            assert!(view.candidates.contains(&g.object_ref(hand)));
            assert!(g.observe(p.other()).effect_choice.is_none());
            assert_eq!(g.observe(p).decision, g.observe(p.other()).decision);
            let action = if discard {
                Action::Discard { card: hand }
            } else {
                Action::Sacrifice {
                    object: g.object_ref(artifact),
                }
            };
            checked(&mut g, action);
            settle(&mut g);
            assert_eq!(draws(&g, p), 2);
            assert_eq!(draws(&g, p.other()), 0);
            assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
            assert_eq!(
                g.object_view(if discard { hand } else { artifact }).zone,
                Zone::Graveyard
            );
            assert_eq!(
                g.object_view(if discard { artifact } else { hand }).zone,
                if discard {
                    Zone::Battlefield
                } else {
                    Zone::Hand
                }
            );
            assert_eq!(
                g.state.players[p.index()].cards_discarded_this_turn,
                u64::from(discard)
            );
        }
    }
}
#[test]
fn alternative_cost_decline_with_or_without_resources_always_retains_private_phase() {
    for p in [Player::P0, Player::P1] {
        for resources in [false, true] {
            let mut g = setup(p);
            if resources {
                named(&mut g, p, "Gilded Lotus", Zone::Battlefield);
            }
            vision(&mut g, p);
            assert_eq!(g.observe(p).effect_choice.unwrap().required[p.index()], 1);
            assert!(g.observe(p.other()).effect_choice.is_none());
            checked(&mut g, Action::DeclineEffectPayment);
            settle(&mut g);
            assert_eq!(draws(&g, p), 0);
        }
    }
}
#[test]
fn alternative_cost_exact_control_zone_and_route_legality_are_atomic() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let stolen = named(&mut g, p.other(), "Gilded Lotus", Zone::Battlefield);
        g.change_controller(g.object_ref(stolen), p);
        g.state.objects[stolen as usize].tapped = true;
        let enemy = named(&mut g, p.other(), "Gilded Lotus", Zone::Battlefield);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let foreign_hand = named(&mut g, p.other(), "Shock", Zone::Hand);
        let own_hand = named(&mut g, p, "Shock", Zone::Hand);
        let grave = named(&mut g, p, "Gilded Lotus", Zone::Graveyard);
        let spell = vision(&mut g, p);
        for id in [enemy, bear, grave, spell, own_hand] {
            let object = g.object_ref(id);
            reject(&mut g, Action::Sacrifice { object });
        }
        for card in [foreign_hand, stolen, spell, grave] {
            reject(&mut g, Action::Discard { card });
        }
        let old = g.object_ref(stolen);
        g.move_object(stolen, Zone::Hand);
        g.move_object(stolen, Zone::Battlefield);
        g.change_controller(g.object_ref(stolen), p);
        reject(&mut g, Action::Sacrifice { object: old });
        let object = g.object_ref(stolen);
        checked(&mut g, Action::Sacrifice { object });
        settle(&mut g);
        assert_eq!(draws(&g, p), 2);
        assert_eq!(g.object_view(stolen).owner, p.other());
        assert_eq!(g.object_view(stolen).zone, Zone::Graveyard);
    }
}
#[test]
fn alternative_cost_artifact_removed_in_response_leaves_discard_or_decline() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let artifact = named(&mut g, p, "Gilded Lotus", Zone::Battlefield);
        let spell = named(&mut g, p, "Vision of Love", Zone::Hand);
        cast(&mut g, spell, None, &[]);
        let departed = g.object_ref(artifact);
        g.move_object(artifact, Zone::Exile);
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        assert!(matches!(g.state.pending, Pending::EffectChoice { .. }));
        reject(&mut g, Action::Sacrifice { object: departed });
        checked(&mut g, Action::DeclineEffectPayment);
        settle(&mut g);
        assert_eq!(draws(&g, p), 0);
    }
}
#[test]
fn alternative_cost_private_nonland_fixture_filters_only_discard_and_preserves_phase() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let hand = named(&mut g, p, "Forest", Zone::Hand);
        vision(&mut g, p);
        // A typed resolution-only fixture exercises the shared flag without
        // admitting Bullseye's unimplemented activation cost.
        let mut eligible = g.clone();
        eligible.state.objects[hand as usize].card = Card::from_name("Shock").unwrap();
        typed_union_fixture(&mut g, true, true);
        typed_union_fixture(&mut eligible, true, true);
        assert_eq!(
            serde_json::to_value(g.observe(p.other())).unwrap(),
            serde_json::to_value(eligible.observe(p.other())).unwrap()
        );
        assert!(g.observe(p).effect_choice.unwrap().candidates.is_empty());
        assert_eq!(
            g.legal_actions(),
            vec![Action::Concede, Action::DeclineEffectPayment]
        );
        reject(&mut g, Action::Discard { card: hand });
        checked(&mut eligible, Action::Discard { card: hand });
        settle(&mut eligible);
        assert_eq!(draws(&eligible, p), 2);
        checked(&mut g, Action::DeclineEffectPayment);
        assert_eq!(draws(&g, p), 0);
    }
}
#[test]
fn alternative_cost_replaced_sacrifice_and_discard_still_draw_exactly_once() {
    for p in [Player::P0, Player::P1] {
        for discard in [false, true] {
            let mut g = setup(p);
            named(&mut g, p, "Rest in Peace", Zone::Battlefield);
            let id = named(
                &mut g,
                p,
                if discard { "Forest" } else { "Gilded Lotus" },
                if discard {
                    Zone::Hand
                } else {
                    Zone::Battlefield
                },
            );
            vision(&mut g, p);
            let action = if discard {
                Action::Discard { card: id }
            } else {
                Action::Sacrifice {
                    object: g.object_ref(id),
                }
            };
            checked(&mut g, action);
            settle(&mut g);
            assert_eq!(g.object_view(id).zone, Zone::Exile);
            assert_eq!(draws(&g, p), 2);
            assert_eq!(
                g.state.players[p.index()].cards_discarded_this_turn,
                u64::from(discard)
            );
        }
    }
}
#[test]
fn alternative_cost_self_replacement_indestructible_and_discard_cause() {
    for p in [Player::P0, Player::P1] {
        for (name, discard) in [
            ("Darksteel Colossus", false),
            ("Progenitus", true),
            ("Wilt-Leaf Liege", true),
        ] {
            let mut g = setup(p.other());
            checked(&mut g, Action::Pass);
            let id = named(
                &mut g,
                p,
                name,
                if discard {
                    Zone::Hand
                } else {
                    Zone::Battlefield
                },
            );
            vision(&mut g, p);
            let action = if discard {
                Action::Discard { card: id }
            } else {
                Action::Sacrifice {
                    object: g.object_ref(id),
                }
            };
            checked(&mut g, action);
            settle(&mut g);
            assert_eq!(draws(&g, p), 2);
            assert_eq!(
                g.object_view(id).zone,
                if name == "Wilt-Leaf Liege" {
                    Zone::Graveyard
                } else {
                    Zone::Library
                }
            );
            assert_eq!(
                g.state.players[p.index()].cards_discarded_this_turn,
                u64::from(discard)
            );
        }
    }
}
#[test]
fn alternative_cost_same_tapped_mana_artifact_is_valid_resolution_resource() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        g.state.players[p.index()].mana = [0; 6];
        let lotus = named(&mut g, p, "Gilded Lotus", Zone::Battlefield);
        let mana_action = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(a,
            Action::ActivateMana { card, production, .. }
                if *card == lotus && *production == [0,0,0,3,0,0])
            })
            .unwrap();
        checked(&mut g, mana_action);
        assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 3, 0, 0]);
        vision(&mut g, p);
        assert!(g.object_view(lotus).tapped);
        let object = g.object_ref(lotus);
        checked(&mut g, Action::Sacrifice { object });
        settle(&mut g);
        assert_eq!(draws(&g, p), 2);
        assert_eq!(g.object_view(lotus).zone, Zone::Graveyard);
    }
}

#[test]
fn alternative_cost_replacement_choice_replays_exact_private_payment_before_drawing() {
    for p in [Player::P0, Player::P1] {
        for effect in [
            ZoneReplacementKind::Exile,
            ZoneReplacementKind::RevealShuffle,
        ] {
            let mut g = setup(p);
            let id = named(&mut g, p.other(), "Darksteel Colossus", Zone::Battlefield);
            g.change_controller(g.object_ref(id), p);
            g.set_counters(id, Counters::of(CounterKind::Finality, 1));
            vision(&mut g, p);
            let object = g.object_ref(id);
            checked(&mut g, Action::Sacrifice { object });
            assert!(matches!(g.state.pending, Pending::ZoneReplacement { .. }));
            assert_eq!(g.actor(), Some(p));
            assert_eq!(draws(&g, p), 0);
            assert_eq!(g.object_view(id).zone, Zone::Battlefield);
            assert!(g.observe(p.other()).zone_replacement_choice.is_none());
            reject(
                &mut g,
                Action::ChooseZoneReplacement {
                    effect: ZoneReplacementKind::Battlefield,
                },
            );
            checked(&mut g, Action::ChooseZoneReplacement { effect });
            settle(&mut g);
            assert_eq!(draws(&g, p), 2);
            assert_eq!(
                g.object_view(id).zone,
                if effect == ZoneReplacementKind::Exile {
                    Zone::Exile
                } else {
                    Zone::Library
                }
            );
            assert_eq!(g.state.creatures_died_this_turn, 0);
        }
    }
}
#[test]
fn alternative_cost_nonland_flag_does_not_exclude_artifact_lands_or_tokens() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let mut recipe = CopyRecipe::new(Card::Forest);
        recipe.exceptions = CopyExceptions::ARTIFACT;
        g.stage_copy_creation(p, recipe, 1);
        settle(&mut g);
        let id = g
            .state
            .objects
            .iter()
            .enumerate()
            .find(|(_, o)| o.zone == Zone::Battlefield && o.is_copy_token())
            .unwrap()
            .0 as CardId;
        assert_eq!(g.current_kinds(id) & 18, 18);
        g.state.objects[id as usize].gained_keywords =
            Keywords::HEXPROOF.union(Keywords::INDESTRUCTIBLE);
        let land = named(&mut g, p, "Forest", Zone::Hand);
        vision(&mut g, p);
        typed_union_fixture(&mut g, true, true);
        reject(&mut g, Action::Discard { card: land });
        let object = g.object_ref(id);
        checked(&mut g, Action::Sacrifice { object });
        settle(&mut g);
        assert_eq!(draws(&g, p), 2);
        assert_eq!(g.object_view(id).zone, Zone::Outside);
    }
}
#[test]
fn alternative_cost_mandatory_typed_choice_has_no_decline_or_partial_route() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let hand = named(&mut g, p, "Shock", Zone::Hand);
        vision(&mut g, p);
        typed_union_fixture(&mut g, false, false);
        reject(&mut g, Action::DeclineEffectPayment);
        let object = g.object_ref(hand);
        reject(&mut g, Action::Sacrifice { object });
        checked(&mut g, Action::Discard { card: hand });
        settle(&mut g);
        assert_eq!(draws(&g, p), 2);
        assert_eq!(g.object_view(hand).zone, Zone::Graveyard);
    }
}
