//! Whole-card paid hybrid Aura and exact graveyard return outcomes.
use super::*;
const FIXTURES: &[&str] = &[
    "Eagle's Rescue",
    "Forest",
    "Llanowar Elves",
    "Bear Cub",
    "Giant Growth",
    "Unsummon",
    "Rest in Peace",
    "Elvish Archivist",
    "Shock",
    "@token/18e92c0b45eb6124d717a226",
];
fn fixture(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    assert!(FIXTURES.contains(&name), "unlisted fixture {name}");
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
    for _ in 0..48 {
        if ready(g) {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("eagle decision budget {:?}", g.state.pending);
}
fn finish(g: &mut Game) {
    until(g, quiet);
}
fn bind(g: &mut Game, target: Target) {
    act(g, Action::ChooseTarget { target });
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
}
fn activation(id: CardId) -> Action {
    Action::AnnounceActivate {
        card: id,
        ability: 0,
        counter_payment: Counters::default(),
    }
}
fn mana(route: u16) -> Mana {
    match route {
        0 => [0, 2, 0, 0, 0, 2],
        1 => [1, 1, 0, 0, 0, 2],
        2 => [2, 0, 0, 0, 0, 2],
        _ => panic!("hybrid route"),
    }
}
fn cast_aura(g: &mut Game, p: Player, source: CardId, target: CardId, route: u16) {
    g.state.players[p.index()].mana = mana(route);
    act(g, Action::AnnounceCast { card: source });
    act(g, Action::ChooseSpellCost { cost: route });
    bind(g, Target::Card(target));
    pay(g);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    finish(g);
}
fn return_aura(g: &mut Game, p: Player, source: CardId, target: CardId, route: u16) {
    g.state.players[p.index()].mana = mana(route);
    act(g, activation(source));
    act(g, Action::ChooseActivationCost { cost: route });
    bind(g, Target::Card(target));
    pay(g);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
}
#[test]
fn eagles_rescue_cast_all_hybrids_enchants_any_creature_with_live_two_two_flying() {
    for p in [Player::P0, Player::P1] {
        for route in 0..3 {
            let mut g = scenario(p);
            let target = fixture(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let source = fixture(&mut g, p, "Eagle's Rescue", Zone::Hand);
            let forest = fixture(&mut g, p, "Forest", Zone::Battlefield);
            g.state.players[p.index()].mana = mana(route);
            act(&mut g, Action::AnnounceCast { card: source });
            act(&mut g, Action::ChooseSpellCost { cost: route });
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(forest),
                },
            );
            bind(&mut g, Target::Card(target));
            pay(&mut g);
            finish(&mut g);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert_eq!(g.object_view(source).zone, Zone::Battlefield);
            assert_eq!(
                g.object_view(source).attached_to,
                Some(g.object_ref(target))
            );
            assert_eq!(stats(&g, target), (4, 4));
            assert!(g.keywords(target).contains(Keywords::FLYING));
            assert!(!g.keywords(source).contains(Keywords::FLYING));
            reject(&mut g, activation(source));
            g.record_control(g.object_ref(target), p, None);
            assert_eq!(g.object_view(source).zone, Zone::Battlefield);
            assert_eq!(stats(&g, target), (4, 4));
            g.move_object(source, Zone::Hand);
            assert_eq!(stats(&g, target), (2, 2));
            assert!(!g.keywords(target).contains(Keywords::FLYING));
        }
    }
}
#[test]
fn eagles_rescue_graveyard_all_hybrids_return_attached_and_power_can_then_exceed_one() {
    for p in [Player::P0, Player::P1] {
        for route in 0..3 {
            let mut g = scenario(p);
            let target = fixture(&mut g, p, "Llanowar Elves", Zone::Battlefield);
            let source = fixture(&mut g, p, "Eagle's Rescue", Zone::Graveyard);
            let before = g.object_ref(source);
            return_aura(&mut g, p, source, target, route);
            assert_eq!(g.object_view(source).zone, Zone::Graveyard);
            finish(&mut g);
            assert_ne!(g.object_ref(source), before);
            assert_eq!(
                g.object_view(source).attached_to,
                Some(g.object_ref(target))
            );
            assert_eq!(stats(&g, target), (3, 3));
            assert!(g.keywords(target).contains(Keywords::FLYING));
            assert_eq!(g.object_view(source).zone, Zone::Battlefield);
            g.record_control(g.object_ref(target), p.other(), None);
            assert_eq!(g.object_view(source).zone, Zone::Battlefield);
            assert_eq!(stats(&g, target), (3, 3));
        }
    }
}
#[test]
fn eagles_rescue_activation_only_own_graveyard_sorcery_and_affordable_colored_cost() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let target = fixture(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        let source = fixture(&mut g, p, "Eagle's Rescue", Zone::Graveyard);
        let foreign = fixture(&mut g, p.other(), "Eagle's Rescue", Zone::Graveyard);
        let hand = fixture(&mut g, p, "Eagle's Rescue", Zone::Hand);
        reject(&mut g, activation(foreign));
        reject(&mut g, activation(hand));
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 8];
        reject(&mut g, activation(source));
        g.state.players[p.index()].mana = [1, 0, 0, 0, 0, 2];
        reject(&mut g, activation(source));
        g.state.players[p.index()].mana = mana(1);
        g.state.step = Step::BeginCombat;
        reject(&mut g, activation(source));
        g.state.step = Step::Main1;
        let growth = fixture(&mut g, p, "Giant Growth", Zone::Hand);
        g.state.players[p.index()].mana = [1, 1, 0, 0, 1, 2];
        cast(&mut g, growth, Some(Target::Card(target)));
        reject(&mut g, activation(source));
        finish(&mut g);
        // Opponent's turn and priority cannot make its owner's graveyard ability legal.
        g.state.active = p.other();
        reject(&mut g, activation(source));
        g.state.active = p;
        g.state.objects[target as usize].growth.power = -3;
        return_aura(&mut g, p, source, target, 1);
        finish(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Battlefield);
    }
}
#[test]
fn eagles_rescue_power_boundary_negative_zero_one_two_and_token_recipient() {
    for p in [Player::P0, Player::P1] {
        for power in [-2, 0, 1, 2] {
            let mut g = scenario(p);
            let target = fixture(&mut g, p, "Llanowar Elves", Zone::Battlefield);
            g.state.objects[target as usize].growth.power = power - 1;
            let source = fixture(&mut g, p, "Eagle's Rescue", Zone::Graveyard);
            g.state.players[p.index()].mana = mana(0);
            if power > 1 {
                reject(&mut g, activation(source));
                continue;
            }
            return_aura(&mut g, p, source, target, 0);
            finish(&mut g);
            assert_eq!(stats(&g, target), (power + 2, 3));
            assert_eq!(
                g.object_view(source).attached_to,
                Some(g.object_ref(target))
            );
        }
        let mut g = scenario(p);
        let token_card = Card::from_name("@token/18e92c0b45eb6124d717a226").unwrap();
        g.create_tokens(p, token_card, 1, false);
        let target = g
            .state
            .objects
            .iter()
            .position(|o| o.card == token_card && o.zone == Zone::Battlefield)
            .unwrap() as CardId;
        assert!(g.object_view(target).is_token());
        let source = fixture(&mut g, p, "Eagle's Rescue", Zone::Graveyard);
        return_aura(&mut g, p, source, target, 2);
        finish(&mut g);
        assert_eq!(stats(&g, target), (3, 3));
        assert_eq!(
            g.object_view(source).attached_to,
            Some(g.object_ref(target))
        );
    }
}
#[test]
fn eagles_rescue_return_targets_owned_current_small_creature_not_other_types_or_opponents() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let target = fixture(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        g.state.objects[target as usize].gained_keywords = Keywords::HEXPROOF;
        let large = fixture(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = fixture(&mut g, p.other(), "Llanowar Elves", Zone::Battlefield);
        let forest = fixture(&mut g, p, "Forest", Zone::Battlefield);
        let grave = fixture(&mut g, p, "Llanowar Elves", Zone::Graveyard);
        let source = fixture(&mut g, p, "Eagle's Rescue", Zone::Graveyard);
        g.state.players[p.index()].mana = mana(0);
        act(&mut g, activation(source));
        act(&mut g, Action::ChooseActivationCost { cost: 0 });
        for id in [large, enemy, forest, grave] {
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(id),
                },
            );
        }
        bind(&mut g, Target::Card(target));
        pay(&mut g);
        finish(&mut g);
        assert_eq!(
            g.object_view(source).attached_to,
            Some(g.object_ref(target))
        );
    }
}
#[test]
fn eagles_rescue_resolution_rechecks_power_controller_protection_and_exact_target() {
    for p in [Player::P0, Player::P1] {
        for change in 0..4 {
            let mut g = scenario(p);
            let target = fixture(&mut g, p, "Llanowar Elves", Zone::Battlefield);
            let source = fixture(&mut g, p, "Eagle's Rescue", Zone::Graveyard);
            let original = g.object_ref(source);
            return_aura(&mut g, p, source, target, 0);
            match change {
                0 => {
                    let growth = fixture(&mut g, p, "Giant Growth", Zone::Hand);
                    g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 0];
                    cast(&mut g, growth, Some(Target::Card(target)));
                }
                1 => g.record_control(g.object_ref(target), p.other(), None),
                2 => {
                    g.move_object(target, Zone::Hand);
                    g.move_object(target, Zone::Battlefield);
                }
                3 => {
                    g.state.objects[target as usize].gained_keywords =
                        Keywords::PROTECTION_EVERYTHING
                }
                _ => unreachable!(),
            }
            finish(&mut g);
            assert_eq!(g.object_view(source).zone, Zone::Graveyard);
            assert_eq!(g.object_ref(source), original);
            assert_eq!(g.object_view(source).attached_to, None);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        }
    }
}
#[test]
fn eagles_rescue_source_exiled_or_new_graveyard_incarnation_cannot_be_returned_by_old_packet() {
    for p in [Player::P0, Player::P1] {
        for reenter in [false, true] {
            let mut g = scenario(p);
            let target = fixture(&mut g, p, "Llanowar Elves", Zone::Battlefield);
            let source = fixture(&mut g, p, "Eagle's Rescue", Zone::Graveyard);
            let original = g.object_ref(source);
            return_aura(&mut g, p, source, target, 1);
            g.move_object(source, Zone::Exile);
            if reenter {
                g.move_object(source, Zone::Graveyard);
            }
            assert_ne!(g.object_ref(source), original);
            finish(&mut g);
            assert_eq!(
                g.object_view(source).zone,
                if reenter {
                    Zone::Graveyard
                } else {
                    Zone::Exile
                }
            );
            assert_eq!(stats(&g, target), (1, 1));
            assert_eq!(g.object_view(source).attached_to, None);
        }
    }
}
#[test]
fn eagles_rescue_actual_aura_sba_graveyard_replacement_and_repeatable_return() {
    for p in [Player::P0, Player::P1] {
        for replacement in [false, true] {
            let mut g = scenario(p);
            let target = fixture(&mut g, p, "Llanowar Elves", Zone::Battlefield);
            let source = fixture(&mut g, p, "Eagle's Rescue", Zone::Hand);
            cast_aura(&mut g, p, source, target, 2);
            if replacement {
                let rip = fixture(&mut g, p, "Rest in Peace", Zone::Hand);
                g.state.players[p.index()].mana = [1, 0, 0, 0, 0, 1];
                cast(&mut g, rip, None);
                finish(&mut g);
            }
            let bounce = fixture(&mut g, p, "Unsummon", Zone::Hand);
            g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 0];
            cast(&mut g, bounce, Some(Target::Card(target)));
            finish(&mut g);
            assert_eq!(g.object_view(target).zone, Zone::Hand);
            assert_eq!(
                g.object_view(source).zone,
                if replacement {
                    Zone::Exile
                } else {
                    Zone::Graveyard
                }
            );
            if replacement {
                reject(&mut g, activation(source));
            } else {
                g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 0];
                cast(&mut g, target, None);
                finish(&mut g);
                return_aura(&mut g, p, source, target, 1);
                finish(&mut g);
                assert_eq!(
                    g.object_view(source).attached_to,
                    Some(g.object_ref(target))
                );
                assert_eq!(stats(&g, target), (3, 3));
            }
        }
    }
}
#[test]
fn eagles_rescue_attachment_exists_before_actual_enchantment_entry_observer_freezes_source_power() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let observer = fixture(&mut g, p, "Elvish Archivist", Zone::Battlefield);
        let source = fixture(&mut g, p, "Eagle's Rescue", Zone::Graveyard);
        let draw_before = g.state.players[p.index()].cards_drawn_this_turn;
        return_aura(&mut g, p, source, observer, 0);
        until(&mut g, |g| g.object_view(source).zone == Zone::Battlefield);
        assert_eq!(stats(&g, observer), (2, 3));
        assert!(g.keywords(observer).contains(Keywords::FLYING));
        assert!(!matches!(g.state.pending, Pending::AuraEntry { .. }));
        let trigger = g.state.stack.last().expect("single trigger auto-stacks");
        assert_eq!(trigger.card, observer);
        assert_eq!(trigger.source_facts.power, 2);
        finish(&mut g);
        assert_eq!(
            g.state.players[p.index()].cards_drawn_this_turn,
            draw_before + 1
        );
    }
}
#[test]
fn eagles_rescue_owned_hybrid_payment_raw_index_restore_parent_drop_and_public_graveyard() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = fixture(&mut g, p, "Eagle's Rescue", Zone::Graveyard);
        let target = fixture(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        let hidden = fixture(&mut g, p, "Forest", Zone::Hand);
        g.state.players[p.index()].mana = [2, 2, 0, 0, 0, 2];
        act(&mut g, activation(source));
        for viewer in [p, p.other()] {
            let obs = g.observe(viewer);
            assert!(
                obs.objects
                    .iter()
                    .any(|o| o.id == source && o.zone == Zone::Graveyard)
            );
            if viewer != p {
                assert!(!obs.objects.iter().any(|o| o.id == hidden));
            }
        }
        super::contracts::branch_contract(g.clone());
        let saved = g.snapshot();
        let mut raw = g.clone();
        let mut indexed = g.clone();
        drop(g);
        let action = Action::ChooseActivationCost { cost: 1 };
        let index = indexed
            .legal_actions()
            .iter()
            .position(|a| *a == action)
            .unwrap();
        act(&mut raw, action);
        indexed.apply_index(p, indexed.revision(), index).unwrap();
        for child in [&mut raw, &mut indexed] {
            bind(child, Target::Card(target));
            pay(child);
            finish(child);
        }
        assert_eq!(raw.snapshot(), indexed.snapshot());
        assert_eq!(raw.state.players[p.index()].mana, [1, 1, 0, 0, 0, 0]);
        indexed.restore(&saved).unwrap();
        act(&mut indexed, Action::ChooseActivationCost { cost: 2 });
        bind(&mut indexed, Target::Card(target));
        pay(&mut indexed);
        finish(&mut indexed);
        assert_eq!(indexed.state.players[p.index()].mana, [0, 2, 0, 0, 0, 0]);
        assert_eq!(stats(&indexed, target), (3, 3));
        assert_eq!(stats(&raw, target), (3, 3));
    }
}
