//! Paid whole-card craft, transformed face, and vehicle actions.
use super::big_helpers::{cast, checked, settle};
use super::hartford_craft_core::{
    begin_craft, craft_clause, craft_mana, craft_material, craft_source,
};
use super::*;

#[test]
fn sawblades_flash_entry_and_required_tapped_opponent_target() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        g.state.active = player.other();
        g.state.step = Step::End;
        let card = put(&mut g, player, "Spring-Loaded Sawblades", Zone::Hand);
        let victim = put(&mut g, player.other(), "Hill Giant", Zone::Battlefield);
        g.state.objects[victim as usize].tapped = true;
        cast(&mut g, card, None, &[]);
        settle(&mut g);
        assert_eq!(g.object_view(card).zone, Zone::Battlefield);
        assert_eq!(g.object_view(victim).zone, Zone::Graveyard);
    }
}

#[test]
fn sawblades_crafts_from_either_zone_and_returns_under_owner() {
    for owner in [Player::P0, Player::P1] {
        for graveyard in [false, true] {
            let controller = owner.other();
            let mut g = scenario(controller);
            let source = put(&mut g, owner, "Spring-Loaded Sawblades", Zone::Battlefield);
            g.state.objects[source as usize].controller = controller;
            let material = put(
                &mut g,
                controller,
                "Short Bow",
                if graveyard {
                    Zone::Graveyard
                } else {
                    Zone::Battlefield
                },
            );
            begin_craft(&mut g, source, 0);
            craft_clause(&mut g, CastCostClause::Additional);
            let source_ref = g.object_ref(source);
            reject(&mut g, Action::ChooseAdditionalCost { object: source_ref });
            let material_ref = g.object_ref(material);
            checked(
                &mut g,
                Action::ChooseAdditionalCost {
                    object: material_ref,
                },
            );
            craft_source(&mut g);
            craft_mana(&mut g);
            assert_eq!(g.object_view(source).zone, Zone::Exile);
            assert_eq!(g.object_view(material).zone, Zone::Exile);
            settle(&mut g);
            assert_eq!(g.object_view(source).zone, Zone::Battlefield);
            assert_eq!(g.object_view(source).controller, owner);
            assert_eq!(g.object_view(source).card.name(), "Bladewheel Chariot");
            assert!(!g.is_creature(source));
        }
    }
}

#[test]
fn sawblades_craft_fails_to_return_after_a_later_zone_change() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let source = put(&mut g, player, "Spring-Loaded Sawblades", Zone::Battlefield);
        let material = put(&mut g, player, "Short Bow", Zone::Battlefield);
        begin_craft(&mut g, source, 0);
        craft_material(&mut g, material);
        craft_source(&mut g);
        craft_mana(&mut g);
        g.move_object(source, Zone::Graveyard);
        g.move_object(source, Zone::Exile);
        settle(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Exile);
    }
}

#[test]
fn bladewheel_chariot_activates_by_tapping_two_other_artifacts_and_crews() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let source = put(&mut g, player, "Spring-Loaded Sawblades", Zone::Battlefield);
        let material = put(&mut g, player, "Short Bow", Zone::Graveyard);
        begin_craft(&mut g, source, 0);
        craft_material(&mut g, material);
        craft_source(&mut g);
        craft_mana(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(source).card.name(), "Bladewheel Chariot");
        let one = put(&mut g, player, "Short Bow", Zone::Battlefield);
        let two = put(&mut g, player, "Short Bow", Zone::Battlefield);
        let pilot = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        let mut crew_branch = g.clone();
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: source,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        let own_ref = g.object_ref(source);
        reject(&mut g, Action::ChooseAdditionalCost { object: own_ref });
        let one_ref = g.object_ref(one);
        let two_ref = g.object_ref(two);
        checked(&mut g, Action::ChooseAdditionalCost { object: one_ref });
        reject(&mut g, Action::ChooseAdditionalCost { object: one_ref });
        checked(&mut g, Action::ChooseAdditionalCost { object: two_ref });
        pay(&mut g);
        settle(&mut g);
        assert!(g.is_creature(source));
        assert_eq!((g.stats(source).power, g.stats(source).toughness), (5, 5));
        assert!(g.object_view(one).tapped && g.object_view(two).tapped);
        g.finish_cleanup();
        assert!(!g.is_creature(source));
        checked(
            &mut crew_branch,
            Action::AnnounceActivate {
                card: source,
                ability: 1,
                counter_payment: Counters::default(),
            },
        );
        let pilot_ref = crew_branch.object_ref(pilot);
        checked(&mut crew_branch, Action::ChooseCrewer { object: pilot_ref });
        checked(&mut crew_branch, Action::FinishCrew);
        settle(&mut crew_branch);
        assert!(crew_branch.is_creature(source));
        assert_eq!(
            (
                crew_branch.stats(source).power,
                crew_branch.stats(source).toughness
            ),
            (5, 5)
        );
    }
}

#[test]
fn activation_lock_ends_only_at_first_actual_untap_and_survives_source_departure() {
    use crate::engine::ability_world::CharacteristicWorld;
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let source = put(&mut g, player, "Short Bow", Zone::Battlefield);
        let recipient = put(&mut g, player.other(), "Short Bow", Zone::Battlefield);
        let recipient_ref = g.object_ref(recipient);
        let mut item = g.activation_item(player, source, 0);
        item.targets.push(BoundTarget {
            group: 0,
            target: Target::Card(recipient),
            object: Some(recipient_ref),
        });
        g.state.objects[recipient as usize].tapped = true;
        g.prevent_target_activations_until_untapped(&item);
        assert!(CharacteristicWorld::activation_prohibited(
            &g,
            player.other(),
            recipient
        ));
        g.move_object(source, Zone::Graveyard);
        assert!(CharacteristicWorld::activation_prohibited(
            &g,
            player.other(),
            recipient
        ));
        g.add_counter(recipient, CounterKind::Stun, 1);
        let checkpoint = g.snapshot();
        let mut branch = g.clone();
        g.untap_objects(&[recipient], untap_rules::UntapCause::Effect);
        assert!(g.object_view(recipient).tapped);
        assert!(CharacteristicWorld::activation_prohibited(
            &g,
            player.other(),
            recipient
        ));
        g.untap_objects(&[recipient], untap_rules::UntapCause::Effect);
        assert!(!g.object_view(recipient).tapped);
        g.tap_objects(&[recipient]);
        assert!(!CharacteristicWorld::activation_prohibited(
            &g,
            player.other(),
            recipient
        ));
        branch.restore(&checkpoint).unwrap();
        branch.untap_objects(&[recipient], untap_rules::UntapCause::Effect);
        branch.untap_objects(&[recipient], untap_rules::UntapCause::Effect);
        branch.tap_objects(&[recipient]);
        assert_eq!(g.snapshot(), branch.snapshot());
    }
}

#[test]
fn source_put_third_counts_real_cards_and_uses_owners_library() {
    for owner in [Player::P0, Player::P1] {
        for count in 0_usize..=3 {
            let mut g = scenario(owner.other());
            g.state.players[owner.index()].library.clear();
            let source = put(&mut g, owner, "Short Bow", Zone::Battlefield);
            g.state.objects[source as usize].controller = owner.other();
            let previous: Vec<_> = (0..count)
                .map(|_| put(&mut g, owner, "Forest", Zone::Library))
                .collect();
            let packet = g.activation_item(owner.other(), source, 0);
            let before = g.snapshot();
            let mut branch = g.clone();
            g.put_source_third_from_top(&packet);
            assert_eq!(g.object_view(source).zone, Zone::Library);
            let library = &g.state.players[owner.index()].library;
            let expected_position = count.saturating_sub(2);
            assert_eq!(library[expected_position], source);
            assert_eq!(library.len(), count + 1);
            assert!(previous.iter().all(|card| library.contains(card)));
            branch.restore(&before).unwrap();
            branch.put_source_third_from_top(&packet);
            assert_eq!(g.snapshot(), branch.snapshot());
        }
    }
}

#[test]
fn activation_lock_expires_in_normal_untap_step() {
    use crate::engine::ability_world::CharacteristicWorld;
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let source = put(&mut g, player.other(), "Short Bow", Zone::Battlefield);
        let recipient = put(&mut g, player, "Short Bow", Zone::Battlefield);
        let recipient_ref = g.object_ref(recipient);
        let mut item = g.activation_item(player.other(), source, 0);
        item.targets.push(BoundTarget {
            group: 0,
            target: Target::Card(recipient),
            object: Some(recipient_ref),
        });
        g.state.objects[recipient as usize].tapped = true;
        g.prevent_target_activations_until_untapped(&item);
        g.state.active = player;
        g.untap_active_player();
        assert!(!g.object_view(recipient).tapped);
        g.tap_objects(&[recipient]);
        assert!(!CharacteristicWorld::activation_prohibited(
            &g, player, recipient
        ));
    }
}

#[test]
fn source_put_third_respects_a_prevented_zone_change() {
    use crate::engine::zone_replacements::TEST_PREVENT_ZONE_MOVE;
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            TEST_PREVENT_ZONE_MOVE.with(|p| p.set(None));
        }
    }
    let _reset = Reset;
    for owner in [Player::P0, Player::P1] {
        let mut g = scenario(owner);
        let source = put(&mut g, owner, "Short Bow", Zone::Battlefield);
        let item = g.activation_item(owner, source, 0);
        TEST_PREVENT_ZONE_MOVE.with(|p| p.set(Some(g.object_ref(source))));
        let library = g.state.players[owner.index()].library.clone();
        g.put_source_third_from_top(&item);
        assert_eq!(g.object_view(source).zone, Zone::Battlefield);
        assert_eq!(g.state.players[owner.index()].library, library);
        TEST_PREVENT_ZONE_MOVE.with(|p| p.set(None));
    }
}

#[test]
fn craft_reserves_its_material_during_mana_funding() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let source = put(&mut g, player, "Spring-Loaded Sawblades", Zone::Battlefield);
        g.create_tokens(player, Card::test_index(5302), 1, false);
        settle(&mut g);
        g.state.players[player.index()].mana = [0, 0, 0, 0, 0, 3];
        let announce = Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        };
        assert!(!g.legal_actions().contains(&announce));
        put(&mut g, player, "Short Bow", Zone::Battlefield);
        assert!(g.legal_actions().contains(&announce));
    }
}

#[test]
fn craft_can_pay_mana_by_removing_a_graveyard_exile_prohibition() {
    use crate::engine::ability_world::TEST_EXILE_PROHIBITOR;
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            TEST_EXILE_PROHIBITOR.with(|p| p.set(None));
        }
    }
    let _reset = Reset;
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let source = put(&mut g, player, "Spring-Loaded Sawblades", Zone::Battlefield);
        let material = put(&mut g, player, "Short Bow", Zone::Graveyard);
        g.create_tokens(player, Card::test_index(5302), 1, false);
        settle(&mut g);
        let treasure = g
            .state
            .objects
            .iter()
            .position(|o| o.card == Card::test_index(5302) && o.zone == Zone::Battlefield)
            .unwrap() as CardId;
        TEST_EXILE_PROHIBITOR.with(|p| p.set(Some(g.object_ref(treasure))));
        g.state.players[player.index()].mana = [0, 0, 0, 0, 0, 3];
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: source,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        let mana = g.legal_actions().into_iter().find(|a| matches!(a, Action::ActivateMana { card, production, .. } if *card == treasure && *production == [1, 0, 0, 0, 0, 0])).expect("Treasure can remove the prohibition while producing white mana");
        checked(&mut g, mana);
        checked(&mut g, Action::BeginOrderedCosts);
        craft_material(&mut g, material);
        craft_source(&mut g);
        craft_mana(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(source).card.name(), "Bladewheel Chariot");
        assert_eq!(g.object_view(material).zone, Zone::Exile);
        TEST_EXILE_PROHIBITOR.with(|p| p.set(None));
    }
}

#[test]
fn source_put_third_skips_transient_copies_when_counting_library_cards() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let spell = put(&mut g, player, "Shock", Zone::Hand);
        let target = put(&mut g, player.other(), "Bear Cub", Zone::Battlefield);
        cast(&mut g, spell, None, &[Target::Card(target)]);
        let original = g.state.stack.last().unwrap().clone();
        let copy = g.allocate_spell_copy(&original, player);
        let fake = copy.source.unwrap().id;
        g.state.stack.push(copy);
        g.move_object(fake, Zone::Library);
        let next = put(&mut g, player, "Island", Zone::Library);
        let top = put(&mut g, player, "Forest", Zone::Library);
        let source = put(&mut g, player, "Short Bow", Zone::Battlefield);
        let packet = g.activation_item(player, source, 0);
        g.put_source_third_from_top(&packet);
        let cards: Vec<_> = g.state.players[player.index()]
            .library
            .iter()
            .rev()
            .copied()
            .filter(|&id| g.state.objects[id as usize].is_card())
            .collect();
        assert_eq!(&cards[..3], &[top, next, source]);
        assert!(g.state.players[player.index()].library.contains(&fake));
    }
}

#[test]
fn craft_offers_both_orders_and_keeps_cost_events_owned() {
    for player in [Player::P0, Player::P1] {
        for source_first in [false, true] {
            let mut g = scenario(player);
            let source = put(&mut g, player, "Spring-Loaded Sawblades", Zone::Battlefield);
            let material = put(&mut g, player, "Short Bow", Zone::Battlefield);
            begin_craft(&mut g, source, 0);
            if source_first {
                craft_source(&mut g);
                assert_eq!(g.object_view(source).zone, Zone::Exile);
                assert_eq!(g.object_view(material).zone, Zone::Battlefield);
                craft_material(&mut g, material);
            } else {
                craft_material(&mut g, material);
                assert_eq!(g.object_view(material).zone, Zone::Exile);
                assert_eq!(g.object_view(source).zone, Zone::Battlefield);
                craft_source(&mut g);
            }
            craft_mana(&mut g);
            settle(&mut g);
            assert_eq!(g.object_view(source).card.name(), "Bladewheel Chariot");
        }
    }
}

#[test]
fn craft_source_first_can_return_its_only_eligible_material() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let source = put(&mut g, player, "Spring-Loaded Sawblades", Zone::Battlefield);
        let material = put(&mut g, player, "Short Bow", Zone::Battlefield);
        let source_ref = g.object_ref(source);
        let material_ref = g.object_ref(material);
        g.exile_objects_until_source_leaves(source_ref, &[material_ref]);
        assert_eq!(g.object_view(material).zone, Zone::Exile);
        begin_craft(&mut g, source, 0);
        let costs = &g
            .state
            .announcing
            .as_ref()
            .unwrap()
            .cast_costs
            .as_ref()
            .unwrap()
            .clauses;
        let material_clause = costs
            .iter()
            .position(|row| row.clause == CastCostClause::Additional)
            .unwrap() as u16;
        assert!(!g.legal_actions().contains(&Action::ChooseCastCost {
            clause: material_clause
        }));
        craft_source(&mut g);
        assert_eq!(g.object_view(material).zone, Zone::Battlefield);
        craft_material(&mut g, material);
        craft_mana(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(source).card.name(), "Bladewheel Chariot");
        assert_eq!(g.object_view(material).zone, Zone::Exile);
    }
}

#[test]
fn craft_cost_replacement_applies_independently_in_both_orders() {
    use crate::engine::zone_replacements::TEST_PREVENT_ZONE_MOVE;
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            TEST_PREVENT_ZONE_MOVE.with(|p| p.set(None));
        }
    }
    let _reset = Reset;
    for player in [Player::P0, Player::P1] {
        for prevent_source in [false, true] {
            let mut g = scenario(player);
            let source = put(&mut g, player, "Spring-Loaded Sawblades", Zone::Battlefield);
            let material = put(&mut g, player, "Short Bow", Zone::Battlefield);
            let prevented = if prevent_source { source } else { material };
            TEST_PREVENT_ZONE_MOVE.with(|p| p.set(Some(g.object_ref(prevented))));
            begin_craft(&mut g, source, 0);
            if prevent_source {
                craft_source(&mut g);
                craft_material(&mut g, material);
            } else {
                craft_material(&mut g, material);
                craft_source(&mut g);
            }
            craft_mana(&mut g);
            settle(&mut g);
            assert_eq!(g.object_view(source).zone, Zone::Battlefield);
            assert_eq!(
                g.object_view(source).card.name(),
                if prevent_source {
                    "Spring-Loaded Sawblades"
                } else {
                    "Bladewheel Chariot"
                }
            );
            assert_eq!(
                g.object_view(material).zone,
                if prevent_source {
                    Zone::Exile
                } else {
                    Zone::Battlefield
                }
            );
            TEST_PREVENT_ZONE_MOVE.with(|p| p.set(None));
        }
    }
}

#[test]
fn craft_all_three_cost_components_can_be_paid_in_any_order() {
    let orders = [
        [
            CastCostClause::Mana,
            CastCostClause::CraftSource,
            CastCostClause::Additional,
        ],
        [
            CastCostClause::Mana,
            CastCostClause::Additional,
            CastCostClause::CraftSource,
        ],
        [
            CastCostClause::CraftSource,
            CastCostClause::Mana,
            CastCostClause::Additional,
        ],
        [
            CastCostClause::CraftSource,
            CastCostClause::Additional,
            CastCostClause::Mana,
        ],
        [
            CastCostClause::Additional,
            CastCostClause::Mana,
            CastCostClause::CraftSource,
        ],
        [
            CastCostClause::Additional,
            CastCostClause::CraftSource,
            CastCostClause::Mana,
        ],
    ];
    for player in [Player::P0, Player::P1] {
        for order in orders {
            let mut g = scenario(player);
            let source = put(&mut g, player, "Spring-Loaded Sawblades", Zone::Battlefield);
            let material = put(&mut g, player, "Short Bow", Zone::Battlefield);
            begin_craft(&mut g, source, 0);
            for clause in order {
                match clause {
                    CastCostClause::Mana => craft_mana(&mut g),
                    CastCostClause::CraftSource => craft_source(&mut g),
                    CastCostClause::Additional => craft_material(&mut g, material),
                    _ => unreachable!(),
                }
            }
            settle(&mut g);
            assert_eq!(g.object_view(source).card.name(), "Bladewheel Chariot");
            assert_eq!(g.object_view(material).zone, Zone::Exile);
        }
    }
}

#[test]
fn craft_source_first_can_end_a_material_exile_prohibition() {
    use crate::engine::ability_world::TEST_EXILE_PROHIBITOR;
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            TEST_EXILE_PROHIBITOR.with(|p| p.set(None));
        }
    }
    let _reset = Reset;
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let source = put(&mut g, player, "Spring-Loaded Sawblades", Zone::Battlefield);
        let material = put(&mut g, player, "Short Bow", Zone::Graveyard);
        TEST_EXILE_PROHIBITOR.with(|p| p.set(Some(g.object_ref(source))));
        begin_craft(&mut g, source, 0);
        let costs = &g
            .state
            .announcing
            .as_ref()
            .unwrap()
            .cast_costs
            .as_ref()
            .unwrap()
            .clauses;
        let material_clause = costs
            .iter()
            .position(|row| row.clause == CastCostClause::Additional)
            .unwrap() as u16;
        assert!(!g.legal_actions().contains(&Action::ChooseCastCost {
            clause: material_clause
        }));
        craft_source(&mut g);
        craft_material(&mut g, material);
        craft_mana(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(source).card.name(), "Bladewheel Chariot");
        assert_eq!(g.object_view(material).zone, Zone::Exile);
        TEST_EXILE_PROHIBITOR.with(|p| p.set(None));
    }
}
