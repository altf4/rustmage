//! Mandatory discarded-land recovery and exact graveyard landfall payments.
use super::*;
const FIXTURES: &[&str] = &[
    "Silvan Reveler",
    "Forest",
    "Island",
    "Bear Cub",
    "Shock",
    "Rest in Peace",
    "Secluded Courtyard",
    "Lorehold, the Historian",
];
fn fixture(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    assert!(FIXTURES.contains(&name), "unlisted {name}");
    put(g, p, name, zone)
}
fn quiet(g: &Game) -> bool {
    g.state.pending == Pending::Priority
        && g.state.stack.is_empty()
        && g.state.pending_triggers.is_empty()
        && g.state.ordering_triggers.is_empty()
        && g.state.resolving.is_none()
}
fn until(g: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..96 {
        if ready(g) {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("Silvan budget {:?}", g.state.pending)
}
fn finish(g: &mut Game) {
    until(g, quiet)
}
fn paid(g: &mut Game, p: Player) -> CardId {
    let id = fixture(g, p, "Silvan Reveler", Zone::Hand);
    g.state.players[p.index()].mana = [0, 1, 0, 0, 1, 2];
    cast(g, id, None);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    id
}
fn discard(g: &mut Game) {
    until(g, |g| {
        matches!(g.state.pending, Pending::EffectDiscard { .. })
    });
}
fn first_hand(g: &Game, p: Player) -> CardId {
    g.state
        .objects
        .iter()
        .enumerate()
        .find_map(|(i, o)| (o.zone == Zone::Hand && o.owner == p).then_some(i as CardId))
        .unwrap()
}
fn landfall(g: &mut Game, p: Player, name: &str) -> CardId {
    let id = fixture(g, p, name, Zone::Hand);
    act(g, Action::PlayLand { card: id });
    id
}
fn payment(g: &mut Game) {
    until(g, |g| {
        matches!(g.state.pending, Pending::ResolutionMana { .. })
    });
}
fn pay_return(g: &mut Game, p: Player) {
    g.state.players[p.index()].mana = [0, 1, 0, 0, 1, 1];
    act(
        g,
        Action::PayResolutionMana {
            payment: [0, 1, 0, 0, 1, 1],
        },
    );
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
}
#[test]
fn silvan_paid_body_private_draw_mandatory_discard_exact_land_automatic_tapped() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let unrelated = fixture(&mut g, p, "Forest", Zone::Graveyard);
        let source = paid(&mut g, p);
        discard(&mut g);
        assert_eq!(stats(&g, source), (3, 2));
        assert!(g.has_subtype(source, "Elf"));
        assert!(g.has_subtype(source, "Citizen"));
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
        let land = first_hand(&g, p);
        assert!(g.observe(p).objects.iter().any(|o| o.id == land));
        assert!(!g.observe(p.other()).objects.iter().any(|o| o.id == land));
        assert!(g.observe(p.other()).effect_discards.is_empty());
        reject(&mut g, Action::Pass);
        let old = g.object_ref(land);
        act(&mut g, Action::Discard { card: land });
        finish(&mut g);
        let view = g.object_view(land);
        assert_eq!(
            (view.zone, view.controller, view.owner, view.tapped),
            (Zone::Battlefield, p, p, true)
        );
        assert_eq!(g.object_ref(land).generation, old.generation + 2);
        assert_eq!(g.object_view(unrelated).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].cards_discarded_this_turn, 1);
        assert!(g.observe(p).effect_choice.is_none());
        assert!(!g.state.stack.iter().any(|s| s.source.is_some_and(|identity| identity.id == source)));
    }
}
#[test]
fn silvan_nonland_discard_does_not_recover_unrelated_land_and_source_can_leave() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let bear = fixture(&mut g, p, "Bear Cub", Zone::Hand);
        let oldland = fixture(&mut g, p, "Forest", Zone::Graveyard);
        let source = paid(&mut g, p);
        discard(&mut g);
        g.move_object(source, Zone::Exile);
        act(&mut g, Action::Discard { card: bear });
        finish(&mut g);
        assert_eq!(g.object_view(bear).zone, Zone::Graveyard);
        assert_eq!(g.object_view(oldland).zone, Zone::Graveyard);
        assert_eq!(g.hand_count(p), 1);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
    }
}
#[test]
fn silvan_discard_replaced_by_exile_is_not_put_on_battlefield() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        fixture(&mut g, p.other(), "Rest in Peace", Zone::Battlefield);
        paid(&mut g, p);
        discard(&mut g);
        let land = first_hand(&g, p);
        let old = g.object_ref(land);
        act(&mut g, Action::Discard { card: land });
        finish(&mut g);
        assert_eq!(g.object_view(land).zone, Zone::Exile);
        assert_eq!(g.object_ref(land).generation, old.generation + 1);
        assert_eq!(g.state.players[p.index()].cards_discarded_this_turn, 1);
        assert!(g.observe(p).effect_choice.is_none());
    }
}
#[test]
fn silvan_discard_entry_choice_retains_exact_move_and_owned_private_branches() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let land = fixture(&mut g, p, "Secluded Courtyard", Zone::Hand);
        let unrelated = fixture(&mut g, p, "Forest", Zone::Graveyard);
        paid(&mut g, p);
        discard(&mut g);
        contracts::branch_contract(g.clone());
        act(&mut g, Action::Discard { card: land });
        assert!(g.observe(p).entry_value.is_some());
        assert_eq!(g.object_view(land).zone, Zone::Graveyard);
        assert!(g.observe(p).effect_choice.is_none());
        contracts::branch_contract(g.clone());
        let saved = g.snapshot();
        let mut raw = g.clone();
        let mut indexed = g.clone();
        drop(g);
        let action = raw
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::ChooseEntryValue { .. }))
            .unwrap();
        let actor = raw.actor().unwrap();
        let index = indexed
            .legal_actions()
            .iter()
            .position(|a| *a == action)
            .unwrap();
        act(&mut raw, action);
        indexed
            .apply_index(actor, indexed.revision(), index)
            .unwrap();
        finish(&mut raw);
        finish(&mut indexed);
        assert_eq!(raw.snapshot(), indexed.snapshot());
        assert_eq!(raw.object_view(land).zone, Zone::Battlefield);
        assert!(raw.object_view(land).tapped);
        assert_eq!(raw.object_view(unrelated).zone, Zone::Graveyard);
        raw.restore(&saved).unwrap();
        choose(&mut raw, |a| matches!(a, Action::ChooseEntryValue { .. }));
        finish(&mut raw);
        assert!(raw.object_view(land).tapped);
    }
}
#[test]
fn silvan_miracle_suspends_draw_before_discard_then_source_in_graveyard_observes_land() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        fixture(&mut g, p, "Lorehold, the Historian", Zone::Battlefield);
        let land = fixture(&mut g, p, "Forest", Zone::Hand);
        let shock = fixture(&mut g, p, "Shock", Zone::Library);
        let source = paid(&mut g, p);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::MiracleReveal { .. })
        });
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
        assert!(g.observe(p).miracle.unwrap().can_reveal);
        assert!(g.observe(p.other()).miracle.is_none());
        assert!(!g.observe(p.other()).objects.iter().any(|o| o.id == shock));
        g.move_object(source, Zone::Graveyard);
        let exact = g.object_ref(source);
        contracts::branch_contract(g.clone());
        let saved = g.snapshot();
        let mut child = g.clone();
        drop(g);
        act(&mut child, Action::DeclineMiracle);
        discard(&mut child);
        act(&mut child, Action::Discard { card: land });
        payment(&mut child);
        assert_eq!(child.object_view(land).zone, Zone::Battlefield);
        assert!(child.object_view(land).tapped);
        assert_eq!(child.state.players[p.index()].cards_drawn_this_turn, 1);
        pay_return(&mut child, p);
        finish(&mut child);
        assert_eq!(child.object_view(source).zone, Zone::Hand);
        assert_ne!(child.object_ref(source), exact);
        let expected = child.snapshot();
        child.restore(&saved).unwrap();
        act(&mut child, Action::DeclineMiracle);
        discard(&mut child);
        act(&mut child, Action::Discard { card: land });
        payment(&mut child);
        pay_return(&mut child, p);
        finish(&mut child);
        assert_eq!(child.snapshot(), expected);
    }
}
#[test]
fn silvan_graveyard_landfall_paid_decline_unpayable_and_battlefield_exile_hand_excluded() {
    for p in [Player::P0, Player::P1] {
        for mode in 0..3 {
            let mut g = scenario(p);
            let source = fixture(&mut g, p, "Silvan Reveler", Zone::Graveyard);
            for zone in [Zone::Battlefield, Zone::Hand, Zone::Exile] {
                fixture(&mut g, p, "Silvan Reveler", zone);
            }
            g.state.players[p.index()].mana = if mode == 2 {
                [0; 6]
            } else {
                [0, 1, 0, 0, 1, 1]
            };
            landfall(&mut g, p, "Forest");
            payment(&mut g);
            assert_eq!(
                g.state.resolving.as_ref().unwrap().item.source.unwrap().object,
                g.object_ref(source)
            );
            assert_eq!(g.state.resolving.as_ref().unwrap().item.controller, p);
            reject(&mut g, Action::PayResolutionMana { payment: [0; 6] });
            if mode == 0 {
                act(
                    &mut g,
                    Action::PayResolutionMana {
                        payment: [0, 1, 0, 0, 1, 1],
                    },
                );
            } else {
                if mode == 2 {
                    assert!(
                        !g.legal_actions()
                            .iter()
                            .any(|a| matches!(a, Action::PayResolutionMana { .. }))
                    );
                }
                act(&mut g, Action::DeclineResolutionMana);
            }
            finish(&mut g);
            assert_eq!(
                g.object_view(source).zone,
                if mode == 0 {
                    Zone::Hand
                } else {
                    Zone::Graveyard
                }
            );
            assert_eq!(
                g.state.players[p.index()].mana,
                if mode == 1 {
                    [0, 1, 0, 0, 1, 1]
                } else {
                    [0; 6]
                }
            );
            if mode == 0 {
                assert!(g.observe(p.other()).objects.iter().any(|o| o.id == source));
            }
        }
    }
}
#[test]
fn silvan_graveyard_landfall_can_generate_mana_during_optional_payment() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = fixture(&mut g, p, "Silvan Reveler", Zone::Graveyard);
        g.state.players[p.index()].mana = [0, 1, 0, 0, 1, 0];
        let forest = landfall(&mut g, p, "Forest");
        payment(&mut g);
        act(&mut g, Action::TapMana { card: forest });
        act(
            &mut g,
            Action::PayResolutionMana {
                payment: [0, 1, 0, 0, 2, 0],
            },
        );
        finish(&mut g);
        assert!(g.object_view(forest).tapped);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(g.object_view(source).zone, Zone::Hand);
    }
}
#[test]
fn silvan_landfall_uses_controller_not_land_owner_and_source_graveyard_uses_owner() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = fixture(&mut g, p, "Silvan Reveler", Zone::Battlefield);
        g.record_control(g.object_ref(source), p.other(), None);
        g.refresh_controllers();
        g.move_object(source, Zone::Graveyard);
        assert_eq!(g.object_view(source).controller, p);
        let opponent_land = fixture(&mut g, p, "Forest", Zone::Hand);
        let mut movement = g.entry_move(opponent_land, Zone::Battlefield, false);
        movement.controller = p.other();
        g.stage_entries(vec![movement], None);
        g.prepare_triggers();
        finish(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Graveyard);
        let foreign = fixture(&mut g, p.other(), "Forest", Zone::Hand);
        let mut movement = g.entry_move(foreign, Zone::Battlefield, false);
        movement.controller = p;
        g.stage_entries(vec![movement], None);
        g.prepare_triggers();
        payment(&mut g);
        pay_return(&mut g, p);
        finish(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Hand);
        assert_eq!(g.object_view(foreign).owner, p.other());
    }
}
#[test]
fn silvan_landfall_token_land_and_multiple_lands_make_distinct_exact_triggers() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = fixture(&mut g, p, "Silvan Reveler", Zone::Graveyard);
        let old = g.object_ref(source);
        let token = g.create_copy_groups_inner(
            p,
            &[TokenGroupView {
                token: Card::Forest,
                amount: 2,
                tapped: false,
            }],
            &[Some(CopyRecipe::new(Card::Forest))],
            None,
        );
        assert_eq!(token.len(), 2);
        g.prepare_triggers();
        until(&mut g, |g| g.state.stack.len() == 2);
        assert!(g.state.stack.iter().all(|s| s.source.is_some_and(|source| source.object == old)));
        payment(&mut g);
        pay_return(&mut g, p);
        assert_eq!(g.object_view(source).zone, Zone::Hand);
        payment(&mut g);
        act(&mut g, Action::DeclineResolutionMana);
        finish(&mut g);
        assert!(token.iter().all(|o| g.object_view(o.card).is_token()));
        assert_eq!(g.object_view(source).zone, Zone::Hand);
    }
}
#[test]
fn silvan_landfall_stale_graveyard_incarnation_can_pay_without_moving_reentry() {
    for p in [Player::P0, Player::P1] {
        for return_to_graveyard in [false, true] {
            let mut g = scenario(p);
            let source = fixture(&mut g, p, "Silvan Reveler", Zone::Graveyard);
            let old = g.object_ref(source);
            landfall(&mut g, p, "Forest");
            g.move_object(source, Zone::Exile);
            if return_to_graveyard {
                g.move_object(source, Zone::Graveyard);
            }
            payment(&mut g);
            assert_eq!(g.state.resolving.as_ref().unwrap().item.source.unwrap().object, old);
            contracts::branch_contract(g.clone());
            pay_return(&mut g, p);
            finish(&mut g);
            assert_eq!(
                g.object_view(source).zone,
                if return_to_graveyard {
                    Zone::Graveyard
                } else {
                    Zone::Exile
                }
            );
            assert_ne!(g.object_ref(source), old);
        }
    }
}
#[test]
fn silvan_simultaneous_return_with_land_uses_post_entry_zone_not_old_graveyard() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = fixture(&mut g, p, "Silvan Reveler", Zone::Graveyard);
        let land = fixture(&mut g, p, "Forest", Zone::Hand);
        let a = g.entry_move(source, Zone::Battlefield, false);
        let b = g.entry_move(land, Zone::Battlefield, false);
        g.stage_entries(vec![a, b], None);
        g.prepare_triggers();
        discard(&mut g);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
        let card = first_hand(&g, p);
        act(&mut g, Action::Discard { card });
        finish(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Battlefield);
        assert!(g.state.stack.is_empty());
    }
}
