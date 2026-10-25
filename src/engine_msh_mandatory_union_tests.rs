//! Whole paid MSH consumers and one announced cost with two payment routes.
use super::assurance::big_helpers::{checked, named, settle, setup};
use super::*;
fn reject(g: &mut Game, a: Action) {
    let old = g.snapshot();
    let bytes = g.state_bytes();
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), old);
    assert_eq!(g.state_bytes(), bytes);
}
fn until_payment(g: &mut Game) {
    for _ in 0..15 {
        if matches!(g.state.pending, Pending::EffectChoice { .. }) {
            return;
        }
        checked(g, Action::Pass);
    }
    panic!("missing union payment");
}
fn finish(g: &mut Game) {
    for _ in 0..12 {
        if g.state.announcing.is_none() {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::FinishTargets | Action::PayCosts { .. }))
            .unwrap();
        checked(g, a);
    }
    panic!("unpaid announcement");
}
fn announce(g: &mut Game, id: CardId, target: Target) {
    checked(
        g,
        Action::AnnounceActivate {
            card: id,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    checked(g, Action::ChooseTarget { target });
    if g.legal_actions().contains(&Action::FinishTargets) {
        checked(g, Action::FinishTargets);
    }
    assert!(matches!(g.state.pending, Pending::AdditionalCost { .. }));
}
fn cast_union(g: &mut Game, id: CardId) {
    checked(g, Action::AnnounceCast { card: id });
    for _ in 0..12 {
        if g.state.announcing.is_none() {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::ChooseSpellCost { .. }
                        | Action::FinishTargets
                        | Action::PayCosts { .. }
                )
            })
            .unwrap();
        checked(g, a);
    }
    panic!("unpaid union creature");
}
fn bull(g: &mut Game, p: Player) -> CardId {
    let id = named(g, p, "Bullseye, Death Dealer", Zone::Hand);
    cast_union(g, id);
    until_payment(g);
    checked(g, Action::DeclineEffectPayment);
    settle(g);
    assert_eq!(
        g.stats(id),
        Stats {
            power: 2,
            toughness: 3
        }
    );
    g.state.objects[id as usize].summoning_sick = false;
    id
}
#[test]
fn msh_union_kun_lun_paid_entry_routes_and_decline_draw_once_private() {
    for p in [Player::P0, Player::P1] {
        for route in 0..3 {
            let mut g = setup(p);
            let artifact = named(&mut g, p, "Gilded Lotus", Zone::Battlefield);
            let hand = named(&mut g, p, "Forest", Zone::Hand);
            let top = *g.state.players[p.index()].library.last().unwrap();
            let id = named(&mut g, p, "K'un-Lun Warrior", Zone::Hand);
            let mana = g.state.players[p.index()].mana.iter().sum::<u32>();
            cast_union(&mut g, id);
            assert_eq!(
                mana - g.state.players[p.index()].mana.iter().sum::<u32>(),
                2
            );
            until_payment(&mut g);
            assert!(g.observe(p.other()).effect_choice.is_none());
            let a = match route {
                0 => Action::Sacrifice {
                    object: g.object_ref(artifact),
                },
                1 => Action::Discard { card: hand },
                _ => Action::DeclineEffectPayment,
            };
            checked(&mut g, a);
            settle(&mut g);
            assert_eq!(
                g.stats(id),
                Stats {
                    power: 2,
                    toughness: 2
                }
            );
            assert_eq!(
                g.state.players[p.index()].cards_drawn_this_turn,
                u64::from(route < 2)
            );
            if route < 2 {
                assert_eq!(g.object_view(top).zone, Zone::Hand);
                assert!(g.observe(p.other()).objects.iter().all(|o| o.id != top));
            }
        }
    }
}
#[test]
fn msh_union_bull_entry_nonland_payment_then_distinct_reflexive_target_and_response() {
    for p in [Player::P0, Player::P1] {
        for discard in [false, true] {
            let mut g = setup(p);
            let artifact = named(&mut g, p, "Gilded Lotus", Zone::Battlefield);
            let hand = named(&mut g, p, "Shock", Zone::Hand);
            let land = named(&mut g, p, "Forest", Zone::Hand);
            let victim = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let id = named(&mut g, p, "Bullseye, Death Dealer", Zone::Hand);
            cast_union(&mut g, id);
            until_payment(&mut g);
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(victim),
                },
            );
            reject(&mut g, Action::Discard { card: land });
            let a = if discard {
                Action::Discard { card: hand }
            } else {
                Action::Sacrifice {
                    object: g.object_ref(artifact),
                }
            };
            checked(&mut g, a);
            assert_eq!(
                g.object_view(if discard { hand } else { artifact }).zone,
                Zone::Graveyard
            );
            assert_eq!(g.object_view(victim).zone, Zone::Battlefield);
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(victim),
                },
            );
            if g.legal_actions().contains(&Action::FinishTargets) {
                checked(&mut g, Action::FinishTargets);
            }
            // The separate trigger can be responded to. A new incarnation is not hit.
            g.move_object(victim, Zone::Hand);
            g.move_object(victim, Zone::Battlefield);
            settle(&mut g);
            assert_eq!(g.object_view(victim).zone, Zone::Battlefield);
            assert_eq!(g.object_view(victim).damage, 0);
        }
    }
}
#[test]
fn msh_union_bull_announcement_both_routes_exact_payment_and_departed_source() {
    for p in [Player::P0, Player::P1] {
        for discard in [false, true] {
            let mut g = setup(p);
            let id = bull(&mut g, p);
            let artifact = named(&mut g, p.other(), "Gilded Lotus", Zone::Battlefield);
            g.change_controller(g.object_ref(artifact), p);
            let hand = named(&mut g, p, "Shock", Zone::Hand);
            let land = named(&mut g, p, "Forest", Zone::Hand);
            let enemy = named(&mut g, p.other(), "Gilded Lotus", Zone::Battlefield);
            let foreign = named(&mut g, p.other(), "Shock", Zone::Hand);
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
            announce(&mut g, id, Target::Player(p.other()));
            assert!(g.observe(p.other()).additional_cost.is_none());
            reject(&mut g, Action::DeclineEffectPayment);
            for bad in [land, enemy, foreign, id] {
                let object = g.object_ref(bad);
                reject(&mut g, Action::ChooseAdditionalCost { object });
            }
            let resource = if discard { hand } else { artifact };
            let object = g.object_ref(resource);
            checked(&mut g, Action::ChooseAdditionalCost { object });
            finish(&mut g);
            assert!(g.object_view(id).tapped);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert_eq!(g.object_view(resource).zone, Zone::Graveyard);
            g.move_object(id, Zone::Exile);
            g.move_object(id, Zone::Battlefield);
            g.change_controller(g.object_ref(id), p.other());
            settle(&mut g);
            assert_eq!(g.state.players[p.other().index()].life, 18);
            assert_eq!(
                g.state.players[p.index()].cards_discarded_this_turn,
                u64::from(discard)
            );
        }
    }
}
#[test]
fn msh_union_bull_private_hand_affordability_and_atomic_illegal_announcement() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = bull(&mut g, p);
        let land = named(&mut g, p, "Forest", Zone::Hand);
        let activation = Action::AnnounceActivate {
            card: id,
            ability: 0,
            counter_payment: Counters::default(),
        };
        reject(&mut g, activation.clone());
        let mut eligible = g.clone();
        eligible.state.objects[land as usize].card = Card::from_name("Shock").unwrap();
        assert_eq!(
            serde_json::to_value(g.observe(p.other())).unwrap(),
            serde_json::to_value(eligible.observe(p.other())).unwrap()
        );
        assert!(eligible.legal_actions().contains(&activation));
        g.state.objects[id as usize].summoning_sick = true;
        named(&mut g, p, "Gilded Lotus", Zone::Battlefield);
        reject(&mut g, activation);
    }
}
#[test]
fn msh_union_bull_virtual_funding_retains_one_resource_and_can_tap_it_for_mana_first() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = bull(&mut g, p);
        g.state.players[p.index()].mana = [0; 6];
        let lotus = named(&mut g, p, "Gilded Lotus", Zone::Battlefield);
        let activation = Action::AnnounceActivate {
            card: id,
            ability: 0,
            counter_payment: Counters::default(),
        };
        assert!(g.legal_actions().contains(&activation));
        let before = g.snapshot();
        let actions = g.legal_actions();
        assert_eq!(g.snapshot(), before);
        assert_eq!(actions, g.legal_actions());
        announce(&mut g, id, Target::Player(p.other()));
        let object = g.object_ref(lotus);
        checked(&mut g, Action::ChooseAdditionalCost { object });
        let mana=g.legal_actions().into_iter().find(|a|matches!(a,Action::ActivateMana { card,production,.. } if *card==lotus && *production==[0,0,0,3,0,0])).unwrap();
        checked(&mut g, mana);
        finish(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(lotus).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.other().index()].life, 18);
        // A sacrificial producer cannot both leave for mana and remain the cost.
        let mut h = setup(p);
        let id = bull(&mut h, p);
        h.state.players[p.index()].mana = [0; 6];
        for _ in 0..3 {
            named(
                &mut h,
                p,
                "@token/b60f6cd3b44b22bd289fe4e2",
                Zone::Battlefield,
            );
        }
        reject(
            &mut h,
            Action::AnnounceActivate {
                card: id,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        named(&mut h, p, "Shock", Zone::Hand);
        assert!(h.legal_actions().contains(&Action::AnnounceActivate {
            card: id,
            ability: 0,
            counter_payment: Counters::default()
        }));
    }
}
#[test]
fn msh_union_bull_payment_replacements_succeed_and_suspension_precedes_damage() {
    for p in [Player::P0, Player::P1] {
        for discard in [false, true] {
            let mut g = setup(p);
            let id = bull(&mut g, p);
            named(&mut g, p, "Rest in Peace", Zone::Battlefield);
            let resource = named(
                &mut g,
                p,
                if discard {
                    "Progenitus"
                } else {
                    "Darksteel Colossus"
                },
                if discard {
                    Zone::Hand
                } else {
                    Zone::Battlefield
                },
            );
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
            announce(&mut g, id, Target::Player(p.other()));
            let object = g.object_ref(resource);
            checked(&mut g, Action::ChooseAdditionalCost { object });
            finish(&mut g);
            assert!(matches!(g.state.pending, Pending::ZoneReplacement { .. }));
            assert_eq!(g.state.players[p.other().index()].life, 20);
            assert!(g.observe(p.other()).zone_replacement_choice.is_none());
            checked(
                &mut g,
                Action::ChooseZoneReplacement {
                    effect: ZoneReplacementKind::Exile,
                },
            );
            settle(&mut g);
            assert_eq!(g.object_view(resource).zone, Zone::Exile);
            assert_eq!(g.state.players[p.other().index()].life, 18);
        }
    }
}

#[test]
fn msh_union_entry_controller_is_captured_after_source_changes_or_departs() {
    for p in [Player::P0, Player::P1] {
        for name in ["K'un-Lun Warrior", "Bullseye, Death Dealer"] {
            for depart in [false, true] {
                let mut g = setup(p);
                let hand = named(&mut g, p, "Wilt-Leaf Liege", Zone::Hand);
                let id = named(&mut g, p, name, Zone::Hand);
                cast_union(&mut g, id);
                // Freeze the entry's controller before any cost choice resolves.
                while g.state.objects[id as usize].zone != Zone::Battlefield {
                    checked(&mut g, Action::Pass);
                }
                if depart {
                    g.move_object(id, Zone::Exile);
                } else {
                    g.change_controller(g.object_ref(id), p.other());
                }
                until_payment(&mut g);
                assert_eq!(g.actor(), Some(p));
                checked(&mut g, Action::Discard { card: hand });
                if name.starts_with("Bullseye") {
                    checked(
                        &mut g,
                        Action::ChooseTarget {
                            target: Target::Player(p.other()),
                        },
                    );
                    if g.legal_actions().contains(&Action::FinishTargets) {
                        checked(&mut g, Action::FinishTargets);
                    }
                }
                settle(&mut g);
                assert_eq!(g.object_view(hand).zone, Zone::Graveyard);
                assert_eq!(g.state.players[p.index()].cards_discarded_this_turn, 1);
                assert_eq!(g.state.players[p.other().index()].cards_drawn_this_turn, 0);
                assert_eq!(
                    g.state.players[p.index()].cards_drawn_this_turn,
                    u64::from(!name.starts_with("Bullseye"))
                );
                assert_eq!(
                    g.state.players[p.other().index()].life,
                    if name.starts_with("Bullseye") { 18 } else { 20 }
                );
            }
        }
    }
}
#[test]
fn msh_union_bull_artifact_copy_can_tap_then_sacrifice_exact_self_and_use_lki() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let mut recipe = CopyRecipe::new(Card::from_name("Bullseye, Death Dealer").unwrap());
        recipe.exceptions = CopyExceptions::ARTIFACT;
        g.stage_copy_creation(p, recipe, 1);
        until_payment(&mut g);
        checked(&mut g, Action::DeclineEffectPayment);
        settle(&mut g);
        let id = g
            .state
            .objects
            .iter()
            .enumerate()
            .find(|(_, o)| o.zone == Zone::Battlefield && o.is_copy_token())
            .unwrap()
            .0 as CardId;
        g.state.objects[id as usize].summoning_sick = false;
        g.state.objects[id as usize].gained_keywords = Keywords::LIFELINK;
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
        announce(&mut g, id, Target::Player(p.other()));
        let object = g.object_ref(id);
        checked(&mut g, Action::ChooseAdditionalCost { object });
        finish(&mut g);
        assert_eq!(g.object_view(id).zone, Zone::Outside);
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 18);
        assert_eq!(g.state.players[p.index()].life, 22);
    }
}
#[test]
fn msh_union_bull_artifact_lands_are_sacrificable_but_never_discardable_as_nonlands() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = bull(&mut g, p);
        let mut recipe = CopyRecipe::new(Card::Forest);
        recipe.exceptions = CopyExceptions::ARTIFACT;
        g.stage_copy_creation(p, recipe, 1);
        settle(&mut g);
        let land = g
            .state
            .objects
            .iter()
            .enumerate()
            .find(|(_, o)| o.zone == Zone::Battlefield && o.is_copy_token())
            .unwrap()
            .0 as CardId;
        assert_eq!(g.current_kinds(land) & 18, 18);
        g.state.objects[land as usize].gained_keywords =
            Keywords::HEXPROOF.union(Keywords::INDESTRUCTIBLE);
        announce(&mut g, id, Target::Player(p.other()));
        let object = g.object_ref(land);
        checked(&mut g, Action::ChooseAdditionalCost { object });
        finish(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(land).zone, Zone::Outside);
        assert_eq!(g.state.players[p.other().index()].life, 18);
    }
}
#[test]
fn msh_union_bull_targets_respect_hexproof_prevention_and_actual_fizzle() {
    for p in [Player::P0, Player::P1] {
        for target_kind in 0..3 {
            let mut g = setup(p);
            let id = bull(&mut g, p);
            let artifact = named(&mut g, p, "Gilded Lotus", Zone::Battlefield);
            let victim = if target_kind == 1 {
                let king = named(&mut g, p, "King T'Challa", Zone::Hand);
                checked(&mut g, Action::AnnounceBack { card: king });
                finish(&mut g);
                settle(&mut g);
                g.change_controller(g.object_ref(king), p.other());
                king
            } else {
                named(&mut g, p.other(), "Bear Cub", Zone::Battlefield)
            };
            if target_kind == 2 {
                g.state.objects[victim as usize].gained_keywords = Keywords::HEXPROOF;
            }
            checked(
                &mut g,
                Action::AnnounceActivate {
                    card: id,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            if target_kind == 2 {
                reject(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(victim),
                    },
                );
                checked(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Player(p.other()),
                    },
                );
            } else {
                checked(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(victim),
                    },
                );
            }
            if g.legal_actions().contains(&Action::FinishTargets) {
                checked(&mut g, Action::FinishTargets);
            }
            let object = g.object_ref(artifact);
            checked(&mut g, Action::ChooseAdditionalCost { object });
            finish(&mut g);
            settle(&mut g);
            if target_kind == 0 {
                assert_eq!(g.object_view(victim).zone, Zone::Graveyard);
            }
            if target_kind == 1 {
                assert_eq!(g.object_view(victim).damage, 0);
                assert_eq!(g.object_view(victim).zone, Zone::Battlefield);
            }
            if target_kind == 2 {
                assert_eq!(g.state.players[p.other().index()].life, 18);
            }
        }
    }
}

#[test]
fn msh_union_bull_cast_pays_either_hybrid_color_and_entry_can_decline_empty() {
    for p in [Player::P0, Player::P1] {
        for color in [2, 3] {
            let mut g = setup(p);
            g.state.players[p.index()].mana = [0; 6];
            g.state.players[p.index()].mana[color] = 1;
            g.state.players[p.index()].mana[5] = 2;
            let id = bull(&mut g, p);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert_eq!(g.object_view(id).zone, Zone::Battlefield);
            assert_eq!(g.state.players[p.other().index()].life, 20);
            assert_eq!(g.hand_count(p), 0);
        }
    }
}
#[test]
fn msh_union_bull_funding_cannot_sacrifice_its_unpaid_tap_source_for_mana() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = bull(&mut g, p);
        let ring = named(&mut g, p, "Lotus Ring", Zone::Hand);
        cast_union(&mut g, ring);
        settle(&mut g);
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: ring,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(id),
            },
        );
        finish(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(ring).attached_to, Some(g.object_ref(id)));
        g.state.players[p.index()].mana = [0; 6];
        // Ring is a valid artifact payment, and the source can produce three
        // mana. That path departs Bullseye before its separate tap cost is paid.
        assert!(
            g.legal_actions()
                .iter()
                .any(|a| matches!(a,Action::TapGrantedMana {card,..} if *card==id))
        );
        let a = Action::AnnounceActivate {
            card: id,
            ability: 0,
            counter_payment: Counters::default(),
        };
        reject(&mut g, a.clone());
        let lotus = named(&mut g, p, "Gilded Lotus", Zone::Battlefield);
        assert!(g.legal_actions().contains(&a));
        announce(&mut g, id, Target::Player(p.other()));
        let object = g.object_ref(ring);
        checked(&mut g, Action::ChooseAdditionalCost { object });
        reject(&mut g, Action::PayCosts { payment: [0; 6] });
        assert!(!g.object_view(id).tapped);
        assert_eq!(g.object_view(ring).zone, Zone::Battlefield);
        let mana=g.legal_actions().into_iter().find(|a|matches!(a,Action::ActivateMana {card,production,..} if *card==lotus && *production==[0,0,0,3,0,0])).unwrap();
        checked(&mut g, mana);
        finish(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(ring).zone, Zone::Graveyard);
        assert_eq!(g.object_view(id).zone, Zone::Battlefield);
        assert!(g.object_view(id).tapped);
        assert_eq!(g.state.players[p.other().index()].life, 18);
    }
}
