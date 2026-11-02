//! Chandra, Cursecloth and Daretti: actual whole-card outcomes and owned choices.
mod support;
use rustmage::Card;
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
    for _ in 0..80 {
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
    panic!("bounded support resolution");
}
fn reach(g: &mut Game, wanted: impl Fn(&Action) -> bool) {
    for _ in 0..24 {
        g.prepare_triggers();
        if g.legal_actions().iter().any(&wanted) {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("missing support choice {:?}", g.state.pending);
}
fn start(g: &mut Game, id: CardId, ability: u16) {
    act(g, activation(id, ability));
    if matches!(g.state.pending, Pending::PayCosts { .. }) {
        pay(g);
    }
}
fn embalm(g: &Game, id: CardId) -> Action {
    let row = g
        .embalm_views()
        .unwrap()
        .into_iter()
        .find(|r| r.object == g.object_ref(id))
        .unwrap();
    Action::AnnounceEmbalm {
        source: row.object,
        acquisition: row.acquisition,
    }
}
fn grant_embalm(g: &mut Game, source: CardId, card: CardId) {
    act(g, activation(source, 0));
    act(
        g,
        Action::ChooseTarget {
            target: Target::Card(card),
        },
    );
    pay(g);
    drain(g);
}
fn embalmed(g: &Game) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(i, o)| {
            (o.zone == Zone::Battlefield
                && o.is_token()
                && g.copy_recipe(g.object_ref(i as CardId))
                    .is_some_and(|r| r.exceptions.embalm))
            .then_some(i as CardId)
        })
        .collect()
}
#[test]
fn chandra_enters_at_four_and_own_combat_optionally_animates_only_a_controlled_vehicle_until_cleanup()
 {
    for p in [Player::P0, Player::P1] {
        for select in [false, true] {
            let mut g = scenario(p);
            let source = put(&mut g, p, "Chandra, Spark Hunter", Zone::Hand);
            g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 3];
            cast(&mut g, source, None);
            drain(&mut g);
            assert_eq!(g.object_view(source).counters.loyalty, 4);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            let vehicle = put(&mut g, p, "Clamorous Ironclad", Zone::Battlefield);
            let enemy = put(&mut g, p.other(), "Clamorous Ironclad", Zone::Battlefield);
            let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            g.begin_step(Step::BeginCombat);
            reach(&mut g, |a| {
                matches!(a, Action::ChooseTarget { .. } | Action::FinishTargets)
            });
            for id in [enemy, bear] {
                reject(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(id),
                    },
                );
            }
            if select {
                act(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(vehicle),
                    },
                );
            } else {
                act(&mut g, Action::FinishTargets);
            }
            g.move_object(source, Zone::Graveyard);
            drain(&mut g);
            assert_eq!(g.is_creature(vehicle), select);
            assert_eq!(g.keywords(vehicle).contains(Keywords::HASTE), select);
            if select {
                assert_eq!(stats(&g, vehicle), (6, 3));
            }
            assert!(!g.is_creature(enemy));
            g.finish_cleanup();
            assert!(!g.is_creature(vehicle));
            assert!(!g.keywords(vehicle).contains(Keywords::HASTE));
        }
    }
    let p = Player::P0;
    let mut g = scenario(p);
    put(&mut g, p, "Chandra, Spark Hunter", Zone::Battlefield);
    put(&mut g, p, "Clamorous Ironclad", Zone::Battlefield);
    g.state.active = p.other();
    g.state.priority = p.other();
    g.begin_step(Step::BeginCombat);
    drain(&mut g);
    assert!(g.state.pending_triggers.is_empty());
}
#[test]
fn chandra_plus_two_may_discard_any_card_or_sacrifice_an_artifact_and_draws_only_if_paid() {
    for mode in [0, 1, 2] {
        let p = Player::P0;
        let mut g = scenario(p);
        let source = put(&mut g, p, "Chandra, Spark Hunter", Zone::Battlefield);
        g.state.objects[source as usize].counters.loyalty = 4;
        let artifact = put(&mut g, p, "Short Bow", Zone::Battlefield);
        let creature = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let hand = put(&mut g, p, "Mountain", Zone::Hand);
        let foreign = put(&mut g, p.other(), "Mountain", Zone::Hand);
        g.state.step = Step::Upkeep;
        reject(&mut g, activation(source, 0));
        g.state.step = Step::Main1;
        start(&mut g, source, 0);
        assert_eq!(g.object_view(source).counters.loyalty, 6);
        reach(&mut g, |a| matches!(a, Action::DeclineEffectPayment));
        let object = g.object_ref(creature);
        reject(&mut g, Action::Sacrifice { object });
        reject(&mut g, Action::Discard { card: foreign });
        match mode {
            0 => act(&mut g, Action::DeclineEffectPayment),
            1 => act(&mut g, Action::Discard { card: hand }),
            _ => {
                let object = g.object_ref(artifact);
                act(&mut g, Action::Sacrifice { object });
            }
        }
        drain(&mut g);
        assert_eq!(
            g.state.players[0].cards_drawn_this_turn,
            if mode == 0 { 0 } else { 1 }
        );
        assert_eq!(
            g.object_view(hand).zone,
            if mode == 1 {
                Zone::Graveyard
            } else {
                Zone::Hand
            }
        );
        assert_eq!(
            g.object_view(artifact).zone,
            if mode == 2 {
                Zone::Graveyard
            } else {
                Zone::Battlefield
            }
        );
        reject(&mut g, activation(source, 1));
        reject(&mut g, activation(source, 0));
    }
}
#[test]
fn chandra_zero_creates_untapped_colorless_three_two_vehicle_with_real_crew_one() {
    let p = Player::P1;
    let mut g = scenario(p);
    let source = put(&mut g, p, "Chandra, Spark Hunter", Zone::Battlefield);
    g.state.objects[source as usize].counters.loyalty = 4;
    start(&mut g, source, 1);
    drain(&mut g);
    assert_eq!(g.object_view(source).counters.loyalty, 4);
    let token = g
        .state
        .objects
        .iter()
        .enumerate()
        .find(|(_, o)| {
            o.zone == Zone::Battlefield
                && o.is_token()
                && o.card == Card::from_name("@token/dft-chandra-vehicle").unwrap()
        })
        .map(|(i, _)| i as CardId)
        .unwrap();
    assert_eq!(g.object_view(token).colors, 0);
    assert!(!g.object_view(token).tapped);
    assert!(!g.is_creature(token));
    let pilot = put(&mut g, p, "Marauding Mako", Zone::Battlefield);
    start(&mut g, token, 0);
    let object = g.object_ref(pilot);
    act(&mut g, Action::ChooseCrewer { object });
    act(&mut g, Action::FinishCrew);
    drain(&mut g);
    assert_eq!(stats(&g, token), (3, 2));
    assert!(g.object_view(pilot).tapped);
    g.finish_cleanup();
    assert!(!g.is_creature(token));
}
#[test]
fn chandra_minus_seven_emblem_survives_the_planeswalker_and_artifact_entries_deal_three_to_any_target()
 {
    for kind in [0, 1, 2] {
        let p = Player::P0;
        let mut g = scenario(p);
        let source = put(&mut g, p, "Chandra, Spark Hunter", Zone::Battlefield);
        g.state.objects[source as usize].counters.loyalty = 4;
        reject(&mut g, activation(source, 2));
        g.state.objects[source as usize].counters.loyalty = 7;
        start(&mut g, source, 2);
        assert_eq!(g.object_view(source).zone, Zone::Graveyard);
        drain(&mut g);
        assert_eq!(
            g.state
                .objects
                .iter()
                .filter(|o| o.zone == Zone::Command && o.controller == p && o.card.is_emblem())
                .count(),
            1
        );
        let target = match kind {
            0 => Target::Player(p.other()),
            1 => Target::Card(put(&mut g, p.other(), "Aegis Turtle", Zone::Battlefield)),
            _ => {
                let id = put(
                    &mut g,
                    p.other(),
                    "Chandra, Spark Hunter",
                    Zone::Battlefield,
                );
                g.state.objects[id as usize].counters.loyalty = 8;
                Target::Card(id)
            }
        };
        let artifact = put(&mut g, p, "Short Bow", Zone::Hand);
        cast(&mut g, artifact, None);
        reach(&mut g, |a| matches!(a, Action::ChooseTarget { .. }));
        act(&mut g, Action::ChooseTarget { target });
        drain(&mut g);
        match target {
            Target::Player(q) => assert_eq!(g.state.players[q.index()].life, 17),
            Target::Card(id) if kind == 1 => assert_eq!(g.object_view(id).damage, 3),
            Target::Card(id) => assert_eq!(g.object_view(id).counters.loyalty, 5),
            _ => unreachable!(),
        };
        let enemy = put(&mut g, p.other(), "Short Bow", Zone::Hand);
        g.state.active = p.other();
        g.state.priority = p.other();
        cast(&mut g, enemy, None);
        drain(&mut g);
        assert!(g.state.pending_triggers.is_empty());
    }
}
#[test]
fn cursecloth_live_zombie_bonus_tracks_controller_and_covers_new_and_copied_zombies() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let own = put(&mut g, p, "Streaking Oilgorger", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Streaking Oilgorger", Zone::Battlefield);
        let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let source = put(&mut g, p, "Cursecloth Wrappings", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 2, 0, 0, 2];
        cast(&mut g, source, None);
        drain(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(stats(&g, own), (4, 4));
        assert_eq!(stats(&g, enemy), (3, 3));
        assert_eq!(stats(&g, bear), (2, 2));
        let later = put(&mut g, p, "Streaking Oilgorger", Zone::Battlefield);
        assert_eq!(stats(&g, later), (4, 4));
        g.change_controller(g.object_ref(source), p.other());
        assert_eq!(stats(&g, own), (3, 3));
        assert_eq!(stats(&g, enemy), (4, 4));
        g.move_object(source, Zone::Hand);
        assert_eq!(stats(&g, enemy), (3, 3));
    }
}
#[test]
fn cursecloth_tap_targets_only_own_graveyard_creature_and_uses_exact_native_embalm_cost_after_source_leaves()
 {
    for p in [Player::P0, Player::P1] {
        for leave in [false, true] {
            let mut g = scenario(p);
            let source = put(&mut g, p, "Cursecloth Wrappings", Zone::Battlefield);
            let card = put(&mut g, p, "Bear Cub", Zone::Graveyard);
            let foreign = put(&mut g, p.other(), "Bear Cub", Zone::Graveyard);
            let artifact = put(&mut g, p, "Clamorous Ironclad", Zone::Graveyard);
            let battlefield = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            act(&mut g, activation(source, 0));
            for id in [foreign, artifact, battlefield] {
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
                    target: Target::Card(card),
                },
            );
            pay(&mut g);
            assert!(g.object_view(source).tapped);
            drain(&mut g);
            reject(&mut g, activation(source, 0));
            if leave {
                g.move_object(source, Zone::Hand);
            }
            let action = embalm(&g, card);
            g.state.step = Step::Upkeep;
            reject(&mut g, action.clone());
            g.state.step = Step::Main1;
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
            reject(&mut g, action.clone());
            g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 1];
            let casts = g.state.players[p.index()].spells_cast_this_turn;
            act(&mut g, action);
            pay(&mut g);
            assert_eq!(g.object_view(card).zone, Zone::Exile);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            drain(&mut g);
            let tokens = embalmed(&g);
            assert_eq!(tokens.len(), 1);
            let token = tokens[0];
            assert_eq!(g.object_view(token).controller, p);
            assert_eq!(g.object_view(token).colors, 1);
            assert!(g.has_subtype(token, "Bear") && g.has_subtype(token, "Zombie"));
            assert_eq!(g.object_mana_value(token), 0);
            assert!(!g.object_view(token).tapped);
            assert_eq!(stats(&g, token), if leave { (2, 2) } else { (3, 3) });
            assert_eq!(g.state.players[p.index()].spells_cast_this_turn, casts);
        }
    }
}
#[test]
fn cursecloth_embalm_acquisition_expires_at_cleanup_and_never_rebinds_a_new_graveyard_incarnation()
{
    for stale in [false, true] {
        let p = Player::P0;
        let mut g = scenario(p);
        let source = put(&mut g, p, "Cursecloth Wrappings", Zone::Battlefield);
        let card = put(&mut g, p, "Bear Cub", Zone::Graveyard);
        grant_embalm(&mut g, source, card);
        let action = embalm(&g, card);
        if stale {
            g.move_object(card, Zone::Hand);
            g.move_object(card, Zone::Graveyard);
        } else {
            g.finish_cleanup();
            g.state.active = p;
            g.state.priority = p;
            g.state.step = Step::Main1;
            g.state.players[0].mana = [8; 6];
        }
        assert!(g.embalm_views().is_none());
        reject(&mut g, action);
        assert_eq!(g.object_view(card).zone, Zone::Graveyard);
    }
}
#[test]
fn cursecloth_embalm_payment_and_copiable_token_properties_survive_raw_index_restore_and_parent_drop()
 {
    let p = Player::P0;
    let mut g = scenario(p);
    let source = put(&mut g, p, "Cursecloth Wrappings", Zone::Battlefield);
    let card = put(&mut g, p, "Bear Cub", Zone::Graveyard);
    grant_embalm(&mut g, source, card);
    let action = embalm(&g, card);
    act(&mut g, action);
    let saved = g.snapshot();
    let mut raw = g.clone();
    let mut indexed = g.clone();
    let a = raw
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    let i = indexed
        .legal_actions()
        .iter()
        .position(|x| *x == a)
        .unwrap();
    act(&mut raw, a);
    indexed.apply_index(p, indexed.revision(), i).unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    indexed.restore(&saved).unwrap();
    drop(g);
    pay(&mut indexed);
    drain(&mut raw);
    drain(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
    let token = embalmed(&indexed)[0];
    let recipe = indexed.copy_recipe(indexed.object_ref(token)).unwrap();
    let copy = put(&mut indexed, p, "Bear Cub", Zone::Battlefield);
    indexed.apply_temporary_copy(indexed.object_ref(copy), recipe, CopyDeadline::EndOfTurn);
    assert_eq!(indexed.object_view(copy).colors, 1);
    assert!(indexed.has_subtype(copy, "Zombie"));
    assert_eq!(indexed.object_mana_value(copy), 0);
    assert_eq!(stats(&indexed, copy), (3, 3));
}
#[test]
fn daretti_power_is_greatest_controlled_artifact_mana_value_in_every_zone_and_updates_live() {
    for zone in [
        Zone::Hand,
        Zone::Library,
        Zone::Graveyard,
        Zone::Exile,
        Zone::Battlefield,
    ] {
        let p = Player::P0;
        let mut g = scenario(p);
        let source = put(&mut g, p, "Daretti, Rocketeer Engineer", zone);
        assert_eq!(stats(&g, source), (0, 5));
        put(&mut g, p.other(), "Gilded Lotus", Zone::Battlefield);
        assert_eq!(stats(&g, source), (0, 5));
        put(&mut g, p, "Short Bow", Zone::Battlefield);
        assert_eq!(stats(&g, source), (1, 5));
        let big = put(&mut g, p, "Gilded Lotus", Zone::Battlefield);
        assert_eq!(stats(&g, source), (5, 5));
        g.move_object(big, Zone::Hand);
        assert_eq!(stats(&g, source), (1, 5));
    }
}
#[test]
fn daretti_entry_chooses_own_artifact_card_before_optional_sacrifice_and_returns_only_that_card() {
    for accept in [false, true] {
        let p = Player::P0;
        let mut g = scenario(p);
        let target = put(&mut g, p, "Clamorous Ironclad", Zone::Graveyard);
        let invalid = put(&mut g, p, "Bear Cub", Zone::Graveyard);
        let foreign = put(&mut g, p.other(), "Short Bow", Zone::Graveyard);
        let material = put(&mut g, p, "Short Bow", Zone::Battlefield);
        let source = put(&mut g, p, "Daretti, Rocketeer Engineer", Zone::Hand);
        g.state.players[0].mana = [0, 0, 0, 1, 0, 4];
        cast(&mut g, source, None);
        reach(&mut g, |a| matches!(a, Action::ChooseTarget { .. }));
        assert_eq!(g.state.players[0].mana, [0; 6]);
        for id in [invalid, foreign, material] {
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
        reach(&mut g, |a| matches!(a, Action::DeclineEffectPayment));
        assert_eq!(g.object_view(target).zone, Zone::Graveyard);
        if accept {
            let object = g.object_ref(material);
            act(&mut g, Action::Sacrifice { object });
        } else {
            act(&mut g, Action::DeclineEffectPayment);
        }
        drain(&mut g);
        assert_eq!(
            g.object_view(material).zone,
            if accept {
                Zone::Graveyard
            } else {
                Zone::Battlefield
            }
        );
        assert_eq!(
            g.object_view(target).zone,
            if accept {
                Zone::Battlefield
            } else {
                Zone::Graveyard
            }
        );
        assert_eq!(stats(&g, source), if accept { (4, 5) } else { (1, 5) });
        if accept {
            assert!(!g.object_view(target).tapped);
            assert!(!g.is_creature(target));
        }
    }
}
#[test]
fn daretti_attack_trigger_survives_source_departure_but_all_illegal_target_skips_optional_sacrifice()
 {
    for stale in [false, true] {
        let p = Player::P0;
        let mut g = scenario(p);
        let source = put(&mut g, p, "Daretti, Rocketeer Engineer", Zone::Battlefield);
        let material = put(&mut g, p, "Short Bow", Zone::Battlefield);
        let target = put(&mut g, p, "Gilded Lotus", Zone::Graveyard);
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        act(&mut g, Action::Attack { card: source });
        act(&mut g, Action::FinishAttackers);
        reach(&mut g, |a| matches!(a, Action::ChooseTarget { .. }));
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        g.move_object(source, Zone::Graveyard);
        if stale {
            g.move_object(target, Zone::Hand);
            g.move_object(target, Zone::Graveyard);
            drain(&mut g);
        } else {
            reach(&mut g, |a| matches!(a, Action::Sacrifice { .. }));
            let object = g.object_ref(material);
            act(&mut g, Action::Sacrifice { object });
            drain(&mut g);
        }
        assert_eq!(
            g.object_view(target).zone,
            if stale {
                Zone::Graveyard
            } else {
                Zone::Battlefield
            }
        );
        assert_eq!(
            g.object_view(material).zone,
            if stale {
                Zone::Battlefield
            } else {
                Zone::Graveyard
            }
        );
    }
}
