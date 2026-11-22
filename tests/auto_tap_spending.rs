//! Opportunity costs and actual lot debits are observable through the same staged actions.
mod support;

use rustmage::{AutoTapOptions, AutoTapStatus, Card};
use rustmage_core::cards::ManaRetention;
use rustmage_core::engine::{mana_lots::Quality, *};
use support::*;

const BLUE: [u32; 6] = [0, 1, 0, 0, 0, 0];

#[test]
fn reserving_a_declared_harmonizer_leaves_the_later_tap_payable() {
    let payer = Player::P0;
    for another_land in [false, true] {
        let mut game = scenario(payer);
        game.state.players[0].mana = [0, 0, 0, 2, 0, 3];
        let elves = put(&mut game, payer, "Llanowar Elves", Zone::Battlefield);
        if another_land {
            put(&mut game, payer, "Forest", Zone::Battlefield);
        }
        let spell = put(&mut game, payer, "Channeled Dragonfire", Zone::Graveyard);
        act(&mut game, Action::AnnounceHarmonize { card: spell });
        let harmonizer = game.object_ref(elves);
        act(
            &mut game,
            Action::ChooseHarmonizer {
                object: Some(harmonizer),
            },
        );
        act(
            &mut game,
            Action::ChooseTarget {
                target: Target::Player(payer.other()),
            },
        );
        if game.legal_actions().contains(&Action::FinishTargets) {
            act(&mut game, Action::FinishTargets);
        }
        let root = game.snapshot();
        let plan = pay_selected(&mut game, &Default::default());
        if another_land {
            assert_eq!(plan.status, AutoTapStatus::Paid);
            assert!(!game.state.objects[elves as usize].tapped);
            let costs = game
                .state
                .announcing
                .as_ref()
                .unwrap()
                .cast_costs
                .as_ref()
                .unwrap();
            let clause = costs
                .clauses
                .iter()
                .position(|row| row.clause == CastCostClause::Harmonize(harmonizer))
                .unwrap() as u16;
            act(&mut game, Action::ChooseCastCost { clause });
            settle(&mut game);
            assert!(game.state.objects[elves as usize].tapped);
            assert_eq!(game.state.players[1].life, 18);
        } else {
            assert_eq!(plan.status, AutoTapStatus::InsufficientSupportedMana);
            assert_eq!(game.snapshot(), root);
        }
    }
}

fn pay_selected(game: &mut Game, options: &AutoTapOptions) -> AutoTapResult {
    let actor = game.actor().unwrap();
    let id = game.attempt_view(actor).unwrap().attempt_id;
    let revision = game.revision();
    let before = game.snapshot();
    let plan = game
        .plan_mana_payment(actor, revision, id, options)
        .unwrap();
    assert_eq!(game.snapshot(), before);
    assert_eq!(
        game.auto_pay_mana(actor, revision, id, options).unwrap(),
        plan
    );
    plan
}

#[test]
fn an_external_ability_layer_hands_off_before_permanent_pricing() {
    let player = Player::P0;
    let mut game = scenario(player);
    game.state.players[0].mana = [0; 6];
    put(&mut game, player, "Food Fight", Zone::Battlefield);
    for _ in 0..12 {
        let creature = put(&mut game, player, "Llanowar Elves", Zone::Battlefield);
        game.state.objects[creature as usize].summoning_sick = true;
    }
    let island = put(&mut game, player, "Island", Zone::Battlefield);
    let spell = put(&mut game, player, "Spectral Sailor", Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: spell });
    Game::take_auto_tap_inventory_counts();
    let before = game.snapshot();
    let result = pay_selected(&mut game, &Default::default());
    assert_eq!(result.status, AutoTapStatus::NeedsDecision);
    assert_eq!(game.snapshot(), before);
    assert_eq!(Game::take_auto_tap_inventory_counts(), (0, 0));
    act(&mut game, Action::TapMana { card: island });
    assert_eq!(
        pay_selected(&mut game, &Default::default()).status,
        AutoTapStatus::Completed
    );
}

#[test]
fn lethal_treasure_trigger_is_avoided_with_floating_or_untapped_treasure() {
    for empty_library in [false, true] {
        for floating in [true, false] {
            let player = Player::P0;
            let mut game = scenario(player);
            game.state.players[0].mana = [0; 6];
            if empty_library {
                game.state.players[0].library.clear();
            } else {
                game.state.players[0].life = 1;
            }
            put(&mut game, player, "Smaug, Wicked Worm", Zone::Battlefield);
            let island = put(&mut game, player, "Island", Zone::Battlefield);
            let treasure = game.state.objects.len() as CardId;
            game.create_tokens(player, Card::from_catalogue_id(5302), 1, false);
            if floating {
                choose(
                    &mut game,
                    |a| matches!(a, Action::ActivateMana { card, production, .. } if *card == treasure && *production == BLUE),
                );
                act(&mut game, Action::TapMana { card: island });
            }
            let spell = put(&mut game, player, "Spectral Sailor", Zone::Hand);
            act(&mut game, Action::AnnounceCast { card: spell });
            let plan = pay_selected(&mut game, &Default::default());
            assert_eq!(plan.status, AutoTapStatus::Completed);
            let cast = game
                .state
                .stack
                .iter()
                .find(|item| item.source.is_some_and(|s| s.id == spell))
                .unwrap();
            assert!(!cast.cast.unwrap().treasure_mana_spent);
            assert!(game.state.objects[island as usize].tapped);
            assert_eq!(game.on_battlefield(treasure), !floating);
        }
    }
}

#[test]
fn expiring_spend_preserves_a_turn_retained_unit_of_the_same_color() {
    let player = Player::P1;
    for retained_first in [false, true] {
        let mut game = scenario(player);
        game.state.players[player.index()].mana = [0; 6];
        for retained in [retained_first, !retained_first] {
            let quality = Quality {
                retention: if retained {
                    ManaRetention::Turn
                } else {
                    ManaRetention::Step
                },
                restriction: ManaRestrictionView::CreatureSpells,
                ..Quality::ORDINARY
            };
            game.add_quality_mana(player, BLUE, Some(quality));
        }
        let spell = put(&mut game, player, "Spectral Sailor", Zone::Hand);
        act(&mut game, Action::AnnounceCast { card: spell });
        let plan = pay_selected(&mut game, &Default::default());
        assert_eq!(plan.status, AutoTapStatus::Completed);
        assert!(
            plan.actions
                .iter()
                .any(|a| matches!(a, Action::ChooseManaLotSpend { .. }))
        );
        assert_eq!(game.mana_lots(player).len(), 1);
        assert_eq!(
            game.mana_lots(player)[0].quality.retention,
            ManaRetention::Turn
        );
    }
}

#[test]
fn two_painful_sources_do_not_spend_the_last_life() {
    let player = Player::P0;
    let mut game = scenario(player);
    game.state.players[0].mana = [0, 0, 0, 0, 0, 2];
    game.state.players[0].life = 2;
    for _ in 0..2 {
        put(&mut game, player, "Starting Town", Zone::Battlefield);
    }
    let spell = put(&mut game, player, "Day of Judgment", Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: spell });
    let before = game.snapshot();
    let revision = game.revision();
    let response = pay_selected(
        &mut game,
        &AutoTapOptions {
            allow_life_payment: true,
            ..Default::default()
        },
    );
    assert_eq!(response.status, AutoTapStatus::NeedsDecision);
    assert!(response.actions.is_empty());
    assert_eq!(game.snapshot(), before);
    assert_eq!(game.revision(), revision);
}

#[test]
fn a_painful_source_preserves_life_committed_to_the_enclosing_activation() {
    let player = Player::P0;
    for available_life in [3, 4] {
        let mut game = scenario(player);
        game.state.players[0].life = available_life;
        game.state.players[0].mana = [0, 0, 0, 0, 0, 1];
        put(&mut game, player, "Starting Town", Zone::Battlefield);
        let draw = put(&mut game, player, "Lunar Convocation", Zone::Battlefield);
        act(
            &mut game,
            Action::AnnounceActivate {
                card: draw,
                ability: 0,
                counter_payment: Default::default(),
            },
        );
        let before = game.snapshot();
        let response = pay_selected(
            &mut game,
            &AutoTapOptions {
                allow_life_payment: true,
                ..Default::default()
            },
        );
        if available_life == 3 {
            assert_eq!(response.status, AutoTapStatus::InsufficientSupportedMana);
            assert_eq!(game.snapshot(), before);
        } else {
            assert_eq!(response.status, AutoTapStatus::Completed);
            assert_eq!(game.state.players[0].life, 1);
        }
    }
}
