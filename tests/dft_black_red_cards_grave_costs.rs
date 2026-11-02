//! Complete graveyard, sacrifice and public hand-choice cards.
mod support;
use rustmage as _;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;
fn activation(card: CardId, ability: u16) -> Action {
    Action::AnnounceActivate {
        card,
        ability,
        counter_payment: Counters::default(),
    }
}
fn drain(g: &mut Game) {
    for _ in 0..64 {
        g.prepare_triggers();
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty()
            && g.state.resolving.is_none()
        {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("graveyard/cost resolution bound");
}
fn reach(g: &mut Game, wanted: fn(&Pending) -> bool) {
    for _ in 0..20 {
        g.prepare_triggers();
        if wanted(&g.state.pending) {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("expected choice missing");
}
fn additional(g: &mut Game, id: CardId) {
    let object = g.object_ref(id);
    act(g, Action::ChooseAdditionalCost { object });
}
fn pilots(g: &Game, p: Player) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter(|(id, o)| {
            o.zone == Zone::Battlefield
                && o.controller == p
                && o.is_token()
                && g.has_subtype(*id as CardId, "Pilot")
        })
        .map(|(id, _)| id as CardId)
        .collect()
}

#[test]
fn back_on_track_returns_own_creature_or_vehicle_and_the_pilot_actually_crews_for_three() {
    for p in [Player::P0, Player::P1] {
        for name in ["Bear Cub", "Clamorous Ironclad"] {
            let mut g = scenario(p);
            let target = put(&mut g, p, name, Zone::Graveyard);
            let foreign = put(&mut g, p.other(), "Bear Cub", Zone::Graveyard);
            let artifact = put(&mut g, p, "Gilded Lotus", Zone::Graveyard);
            let spell = put(&mut g, p, "Back on Track", Zone::Hand);
            g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 4];
            act(&mut g, Action::AnnounceCast { card: spell });
            for id in [foreign, artifact] {
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
            pay(&mut g);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            drain(&mut g);
            assert_eq!(g.object_view(target).zone, Zone::Battlefield);
            assert_eq!(g.object_view(target).controller, p);
            assert!(!g.object_view(target).tapped);
            let tokens = pilots(&g, p);
            assert_eq!(tokens.len(), 1);
            let pilot = tokens[0];
            assert_eq!(stats(&g, pilot), (1, 1));
            assert_eq!(g.object_view(pilot).colors, 0);
            assert!(!g.object_view(pilot).tapped);
            assert!(g.keywords(pilot).contains(Keywords::CREW_SADDLE_PLUS_TWO));
            let vehicle = if name == "Clamorous Ironclad" {
                target
            } else {
                put(&mut g, p, "Clamorous Ironclad", Zone::Battlefield)
            };
            act(&mut g, activation(vehicle, 0));
            if matches!(g.state.pending, Pending::PayCosts { .. }) {
                pay(&mut g);
            }
            let object = g.object_ref(pilot);
            act(&mut g, Action::ChooseCrewer { object });
            act(&mut g, Action::FinishCrew);
            drain(&mut g);
            assert_eq!(stats(&g, vehicle), (6, 3));
            assert!(g.object_view(pilot).tapped);
        }
    }
}

#[test]
fn back_on_tracks_pilot_is_independent_of_failed_return_but_not_an_all_illegal_spell() {
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|v| v.set(None));
        }
    }
    let _reset = Reset;
    for stale in [false, true] {
        let p = Player::P0;
        let mut g = scenario(p);
        let target = put(&mut g, p, "Bear Cub", Zone::Graveyard);
        let spell = put(&mut g, p, "Back on Track", Zone::Hand);
        cast(&mut g, spell, Some(Target::Card(target)));
        if stale {
            g.move_object(target, Zone::Hand);
            g.move_object(target, Zone::Graveyard);
        } else {
            zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|v| v.set(Some(g.object_ref(target))));
        }
        drain(&mut g);
        assert_eq!(g.object_view(target).zone, Zone::Graveyard);
        assert_eq!(pilots(&g, p).len(), usize::from(!stale));
        zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|v| v.set(None));
    }
}

#[test]
fn sideswipe_sacrifices_artifact_or_creature_and_draws_only_for_a_paid_vehicle() {
    for p in [Player::P0, Player::P1] {
        for name in ["Bear Cub", "Gilded Lotus", "Clamorous Ironclad"] {
            let mut g = scenario(p);
            let cost = put(&mut g, p, name, Zone::Battlefield);
            let target = put(&mut g, p.other(), "Aegis Turtle", Zone::Battlefield);
            let invalid = put(&mut g, p, "Mountain", Zone::Battlefield);
            let spell = put(&mut g, p, "Hellish Sideswipe", Zone::Hand);
            g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 0];
            act(&mut g, Action::AnnounceCast { card: spell });
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(target),
                },
            );
            let object = g.object_ref(invalid);
            reject(&mut g, Action::ChooseAdditionalCost { object });
            additional(&mut g, cost);
            pay(&mut g);
            assert_eq!(g.object_view(cost).zone, Zone::Graveyard);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
            drain(&mut g);
            assert_eq!(g.object_view(target).zone, Zone::Graveyard);
            assert_eq!(
                g.state.players[p.index()].cards_drawn_this_turn,
                u64::from(name == "Clamorous Ironclad")
            );
        }
    }
}

#[test]
fn sideswipe_vehicle_draw_does_not_require_destruction_but_does_require_a_legal_target() {
    for stale in [false, true] {
        let p = Player::P0;
        let mut g = scenario(p);
        let cost = put(&mut g, p, "Clamorous Ironclad", Zone::Battlefield);
        let target = put(&mut g, p.other(), "Clamorous Ironclad", Zone::Battlefield);
        g.state.objects[target as usize].gained_keywords = Keywords::INDESTRUCTIBLE;
        let spell = put(&mut g, p, "Hellish Sideswipe", Zone::Hand);
        act(&mut g, Action::AnnounceCast { card: spell });
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        additional(&mut g, cost);
        pay(&mut g);
        if stale {
            g.move_object(target, Zone::Hand);
            g.move_object(target, Zone::Battlefield);
        }
        drain(&mut g);
        assert_eq!(g.object_view(target).zone, Zone::Battlefield);
        assert_eq!(
            g.state.players[p.index()].cards_drawn_this_turn,
            u64::from(!stale)
        );
    }
}

#[test]
fn intimidation_tactics_reveals_all_cards_and_exiles_the_chosen_artifact_or_creature_permanently() {
    for p in [Player::P0, Player::P1] {
        for artifact in [false, true] {
            let mut g = scenario(p);
            let creature = put(&mut g, p.other(), "Bear Cub", Zone::Hand);
            let item = put(&mut g, p.other(), "Gilded Lotus", Zone::Hand);
            let land = put(&mut g, p.other(), "Mountain", Zone::Hand);
            let instant = put(&mut g, p.other(), "Shock", Zone::Hand);
            let spell = put(&mut g, p, "Intimidation Tactics", Zone::Hand);
            act(&mut g, Action::AnnounceCast { card: spell });
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Player(p),
                },
            );
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Player(p.other()),
                },
            );
            pay(&mut g);
            reach(&mut g, |q| matches!(q, Pending::HandCardChoice { .. }));
            for viewer in [p, p.other()] {
                let observation = g.observe(viewer);
                for id in [creature, item, land, instant] {
                    assert!(observation.known_hand.contains(&g.object_ref(id)));
                }
            }
            reject(&mut g, Action::ChooseHandCard { object: None });
            for id in [land, instant] {
                let object = g.object_ref(id);
                reject(
                    &mut g,
                    Action::ChooseHandCard {
                        object: Some(object),
                    },
                );
            }
            let chosen = if artifact { item } else { creature };
            let object = g.object_ref(chosen);
            act(
                &mut g,
                Action::ChooseHandCard {
                    object: Some(object),
                },
            );
            drain(&mut g);
            assert_eq!(g.object_view(chosen).zone, Zone::Exile);
            assert_eq!(g.object_view(chosen).owner, p.other());
            g.move_object(spell, Zone::Hand);
            assert_eq!(g.object_view(chosen).zone, Zone::Exile);
            assert_eq!(
                g.state.players[p.other().index()].cards_discarded_this_turn,
                0
            );
        }
    }
}

#[test]
fn intimidation_tactics_still_reveals_an_unmatched_hand_and_cycles_for_three() {
    let p = Player::P1;
    let mut g = scenario(p);
    let land = put(&mut g, p.other(), "Mountain", Zone::Hand);
    let instant = put(&mut g, p.other(), "Shock", Zone::Hand);
    let spell = put(&mut g, p, "Intimidation Tactics", Zone::Hand);
    cast(&mut g, spell, Some(Target::Player(p.other())));
    drain(&mut g);
    for id in [land, instant] {
        assert_eq!(g.object_view(id).zone, Zone::Hand);
        assert!(g.observe(p).known_hand.contains(&g.object_ref(id)));
    }
    let card = put(&mut g, p, "Intimidation Tactics", Zone::Hand);
    g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
    reject(&mut g, activation(card, 0));
    g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
    act(&mut g, activation(card, 0));
    pay(&mut g);
    assert_eq!(g.object_view(card).zone, Zone::Graveyard);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    drain(&mut g);
    assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
}

#[test]
fn wickerfolk_intrinsic_graveyard_route_pays_life_and_either_material_then_dies_normally() {
    for p in [Player::P0, Player::P1] {
        for name in ["Bear Cub", "Gilded Lotus"] {
            let mut g = scenario(p);
            let source = put(&mut g, p, "Wickerfolk Indomitable", Zone::Graveyard);
            let route = Action::AnnounceGraveyardAdditionalCosts { card: source };
            reject(&mut g, route.clone());
            let cost = put(&mut g, p, name, Zone::Battlefield);
            g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 3];
            act(&mut g, route);
            additional(&mut g, cost);
            pay(&mut g);
            assert_eq!(g.state.players[p.index()].life, 18);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert_eq!(g.object_view(cost).zone, Zone::Graveyard);
            assert_eq!(g.object_view(source).zone, Zone::Stack);
            drain(&mut g);
            assert_eq!(stats(&g, source), (4, 3));
            let damage = put(&mut g, p, "Lightning Strike", Zone::Hand);
            g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 1];
            cast(&mut g, damage, Some(Target::Card(source)));
            drain(&mut g);
            assert_eq!(
                g.object_view(source).zone,
                Zone::Graveyard,
                "intrinsic permission has no flashback exile"
            );
        }
    }
}

#[test]
fn wickerfolk_hand_and_external_graveyard_casts_do_not_inherit_route_costs() {
    for p in [Player::P0, Player::P1] {
        for grave in [false, true] {
            let mut g = scenario(p);
            g.state.players[p.index()].life = 1;
            let source = put(
                &mut g,
                p,
                "Wickerfolk Indomitable",
                if grave { Zone::Graveyard } else { Zone::Hand },
            );
            if grave {
                let object = g.object_ref(source);
                g.grant_graveyard_permission(p, object, GraveyardCastKind::Cast);
            }
            g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 3];
            cast(&mut g, source, None);
            drain(&mut g);
            assert_eq!(g.object_view(source).zone, Zone::Battlefield);
            assert_eq!(stats(&g, source), (4, 3));
            assert_eq!(g.state.players[p.index()].life, 1);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        }
    }
}

#[test]
fn wickerfolk_route_rejects_bad_life_timing_and_owner_and_keeps_paid_owned_branches() {
    let p = Player::P0;
    let mut g = scenario(p);
    let source = put(&mut g, p, "Wickerfolk Indomitable", Zone::Graveyard);
    let cost = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let route = Action::AnnounceGraveyardAdditionalCosts { card: source };
    g.state.players[0].life = 1;
    reject(&mut g, route.clone());
    g.state.players[0].life = 20;
    g.state.step = Step::Upkeep;
    reject(&mut g, route.clone());
    g.state.step = Step::Main1;
    let foreign = put(&mut g, p.other(), "Wickerfolk Indomitable", Zone::Graveyard);
    reject(
        &mut g,
        Action::AnnounceGraveyardAdditionalCosts { card: foreign },
    );
    act(&mut g, route);
    additional(&mut g, cost);
    let saved = g.snapshot();
    let mut raw = g.clone();
    let mut indexed = g.clone();
    let payment = raw
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    let at = indexed
        .legal_actions()
        .iter()
        .position(|a| *a == payment)
        .unwrap();
    act(&mut raw, payment);
    indexed.apply_index(p, indexed.revision(), at).unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    indexed.restore(&saved).unwrap();
    drop(g);
    pay(&mut indexed);
    drain(&mut raw);
    drain(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(indexed.state.players[0].life, 18);
}

#[test]
fn carrion_cruiser_mills_then_must_choose_from_the_whole_own_graveyard_and_crews_for_one() {
    for old in [false, true] {
        let p = Player::P0;
        let mut g = scenario(p);
        let existing = put(&mut g, p, "Bear Cub", Zone::Graveyard);
        let wrong = put(&mut g, p, "Gilded Lotus", Zone::Graveyard);
        let foreign = put(&mut g, p.other(), "Bear Cub", Zone::Graveyard);
        let milled = put(&mut g, p, "Clamorous Ironclad", Zone::Library);
        let other = put(&mut g, p, "Shock", Zone::Library);
        let before = g.state.players[0].library.len();
        let source = put(&mut g, p, "Carrion Cruiser", Zone::Hand);
        cast(&mut g, source, None);
        reach(&mut g, |q| matches!(q, Pending::MilledSelection { .. }));
        assert_eq!(g.state.players[0].library.len(), before - 2);
        assert_eq!(g.object_view(milled).zone, Zone::Graveyard);
        assert_eq!(g.object_view(other).zone, Zone::Graveyard);
        reject(&mut g, Action::FinishMilledSelection);
        for id in [wrong, foreign] {
            let object = g.object_ref(id);
            reject(&mut g, Action::SelectMilled { object });
        }
        let selected = if old { existing } else { milled };
        let object = g.object_ref(selected);
        act(&mut g, Action::SelectMilled { object });
        drain(&mut g);
        assert_eq!(g.object_view(selected).zone, Zone::Hand);
        let crewer = put(&mut g, p, "Marauding Mako", Zone::Battlefield);
        act(&mut g, activation(source, 0));
        if matches!(g.state.pending, Pending::PayCosts { .. }) {
            pay(&mut g);
        }
        let object = g.object_ref(crewer);
        act(&mut g, Action::ChooseCrewer { object });
        act(&mut g, Action::FinishCrew);
        drain(&mut g);
        assert_eq!(stats(&g, source), (3, 2));
    }
}

#[test]
fn carrion_cruiser_without_any_matching_graveyard_card_still_mills_and_finishes() {
    let p = Player::P0;
    let mut g = scenario(p);
    let first = put(&mut g, p, "Shock", Zone::Library);
    let second = put(&mut g, p, "Divination", Zone::Library);
    let source = put(&mut g, p, "Carrion Cruiser", Zone::Hand);
    cast(&mut g, source, None);
    drain(&mut g);
    assert_eq!(g.object_view(first).zone, Zone::Graveyard);
    assert_eq!(g.object_view(second).zone, Zone::Graveyard);
    assert_eq!(g.object_view(source).zone, Zone::Battlefield);
}

#[test]
fn gastal_blockbuster_sacrifices_before_choosing_a_separate_enemy_artifact_target() {
    for vehicle in [false, true] {
        for accept in [false, true] {
            let p = Player::P0;
            let mut g = scenario(p);
            let enemy = put(&mut g, p.other(), "Gilded Lotus", Zone::Battlefield);
            let own = put(&mut g, p, "Gilded Lotus", Zone::Battlefield);
            let material = vehicle.then(|| put(&mut g, p, "Clamorous Ironclad", Zone::Battlefield));
            let source = put(&mut g, p, "Gastal Blockbuster", Zone::Hand);
            cast(&mut g, source, None);
            reach(&mut g, |q| matches!(q, Pending::EffectChoice { .. }));
            assert_eq!(g.object_view(enemy).zone, Zone::Battlefield);
            if !accept {
                act(&mut g, Action::DeclineEffectPayment);
                drain(&mut g);
                assert_eq!(g.object_view(enemy).zone, Zone::Battlefield);
                assert_eq!(g.object_view(source).zone, Zone::Battlefield);
                continue;
            }
            let paid = material.unwrap_or(source);
            let object = g.object_ref(paid);
            act(&mut g, Action::Sacrifice { object });
            assert_eq!(g.object_view(paid).zone, Zone::Graveyard);
            for _ in 0..16 {
                g.prepare_triggers();
                if g.legal_actions().contains(&Action::ChooseTarget {
                    target: Target::Card(enemy),
                }) {
                    break;
                }
                choose(&mut g, |a| {
                    matches!(a, Action::Pass | Action::PutTrigger { .. })
                });
            }
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(own),
                },
            );
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(enemy),
                },
            );
            drain(&mut g);
            assert_eq!(g.object_view(enemy).zone, Zone::Graveyard);
            assert_eq!(g.object_view(own).zone, Zone::Battlefield);
        }
    }
}

#[test]
fn gastal_blockbuster_may_sacrifice_itself_even_when_the_reflexive_trigger_has_no_target() {
    let p = Player::P0;
    let mut g = scenario(p);
    let source = put(&mut g, p, "Gastal Blockbuster", Zone::Hand);
    cast(&mut g, source, None);
    reach(&mut g, |q| matches!(q, Pending::EffectChoice { .. }));
    let object = g.object_ref(source);
    act(&mut g, Action::Sacrifice { object });
    drain(&mut g);
    assert_eq!(g.object_view(source).zone, Zone::Graveyard);
}
