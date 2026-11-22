//! A named follow-up is a visible play. Its mana and source are preferences for this payment.
use crate::support::*;
use rustmage::{AutoTapFollowUp, AutoTapOptions, AutoTapStatus, ManaCost};
use rustmage_core::engine::*;

fn named(action: AutoTapFollowUp) -> AutoTapOptions {
    AutoTapOptions {
        follow_up_action: Some(action),
        ..Default::default()
    }
}

fn pay_with(game: &mut Game, options: &AutoTapOptions) {
    let actor = game.actor().unwrap();
    let revision = game.revision();
    let attempt = game.attempt_view(actor).unwrap().attempt_id;
    let preview = game
        .plan_mana_payment(actor, revision, attempt, options)
        .unwrap();
    assert_eq!(preview.status, AutoTapStatus::Completed);
    assert_eq!(
        game.auto_pay_mana(actor, revision, attempt, options)
            .unwrap(),
        preview
    );
}

#[test]
fn a_named_counterspell_can_pay_for_its_response_to_the_first_cast() {
    let payer = Player::P0;
    let mut game = scenario(payer);
    game.state.players[payer.index()].mana = [0; 6];
    let lands = ["Island", "Island", "Forest", "Plains"]
        .map(|name| put(&mut game, payer, name, Zone::Battlefield));
    let response = put(&mut game, payer, "Counterspell", Zone::Hand);
    let spell = put(&mut game, payer, "Boom Box", Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: spell });
    let options = named(AutoTapFollowUp::Cast {
        object: game.object_ref(response),
    });
    pay_with(&mut game, &options);
    assert_eq!(
        lands.map(|id| game.object_view(id).tapped),
        [false, false, true, true]
    );
    let target = Target::Spell(spell);
    act(&mut game, Action::AnnounceCast { card: response });
    act(&mut game, Action::ChooseTarget { target });
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(&mut game, Action::FinishTargets);
    }
    pay_with(&mut game, &AutoTapOptions::default());
    settle(&mut game);
    assert_eq!(game.object_view(spell).zone, Zone::Graveyard);
}

#[test]
fn preserves_named_tap_ability_but_can_spend_its_source_to_complete_the_current_cast() {
    for other_lands in [4, 1] {
        let payer = Player::P1;
        let mut game = scenario(payer);
        game.state.players[payer.index()].mana = [0; 6];
        let processor = put(&mut game, payer, "Isolinear Processor", Zone::Battlefield);
        for _ in 0..other_lands {
            put(&mut game, payer, "Forest", Zone::Battlefield);
        }
        let spell = put(&mut game, payer, "Boom Box", Zone::Hand);
        act(&mut game, Action::AnnounceCast { card: spell });
        let object = game.object_ref(processor);
        let options = named(AutoTapFollowUp::Activate { object, ability: 1 });
        if other_lands == 1 {
            let attempt = game.attempt_view(payer).unwrap().attempt_id;
            let hard = AutoTapOptions {
                protected_sources: vec![object],
                ..options.clone()
            };
            assert_eq!(
                game.plan_mana_payment(payer, game.revision(), attempt, &hard)
                    .unwrap()
                    .status,
                AutoTapStatus::InsufficientSupportedMana
            );
        }
        pay_with(&mut game, &options);
        assert_eq!(game.object_view(processor).tapped, other_lands == 1);
        if other_lands == 4 {
            act(
                &mut game,
                Action::AnnounceActivate {
                    card: processor,
                    ability: 1,
                    counter_payment: Default::default(),
                },
            );
            pay_with(&mut game, &AutoTapOptions::default());
            assert!(game.object_view(processor).tapped);
        }
    }
}

#[test]
fn invalid_or_concealed_action_hints_are_identical_transactional_errors() {
    let payer = Player::P0;
    let mut game = scenario(payer);
    let opponent = put(&mut game, payer.other(), "Counterspell", Zone::Hand);
    let foreign = put(&mut game, payer.other(), "Fountainport", Zone::Battlefield);
    let processor = put(&mut game, payer, "Isolinear Processor", Zone::Battlefield);
    let secret = put(&mut game, payer, "Counterspell", Zone::Library);
    let spell = put(&mut game, payer, "Boom Box", Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: spell });
    let target = game.object_ref(processor);
    let revision = game.revision();
    let attempt = game.attempt_view(payer).unwrap().attempt_id;
    let mut different = game.clone();
    different.state.objects[secret as usize].card = rustmage::Card::Mountain;
    for branch in [&mut game, &mut different] {
        assert!(branch.draw_one_unreplaced(payer));
        let held = branch.object_ref(secret);
        assert!(branch.attempt_draw_is_face_down(held));
        let snapshot = branch.snapshot();
        let valid = named(AutoTapFollowUp::Activate {
            object: target,
            ability: 1,
        });
        let hints = [
            named(AutoTapFollowUp::Cast { object: held }),
            named(AutoTapFollowUp::Cast {
                object: branch.object_ref(opponent),
            }),
            named(AutoTapFollowUp::Activate {
                object: branch.object_ref(foreign),
                ability: 1,
            }),
            named(AutoTapFollowUp::Activate {
                object: target,
                ability: u16::MAX,
            }),
            named(AutoTapFollowUp::Activate {
                object: ObjectRef {
                    card: u32::MAX,
                    generation: 0,
                },
                ability: 1,
            }),
            named(AutoTapFollowUp::Activate {
                object: ObjectRef {
                    generation: target.generation + 1,
                    ..target
                },
                ability: 1,
            }),
            named(AutoTapFollowUp::Cast { object: target }),
            AutoTapOptions {
                follow_up: Some(ManaCost::ZERO),
                ..valid
            },
        ];
        for hint in hints {
            assert!(matches!(
                branch.plan_mana_payment(payer, revision, attempt, &hint),
                Err(Error::IllegalAction)
            ));
            assert!(matches!(
                branch.auto_pay_mana(payer, revision, attempt, &hint),
                Err(Error::IllegalAction)
            ));
            assert_eq!(branch.snapshot(), snapshot);
            assert_eq!(branch.revision(), revision);
        }
    }
}

#[test]
fn current_cost_consistency_for_named_activations_and_bound_sacrifices() {
    let payer = Player::P0;
    for (name, repeatable) in [("Isolinear Processor", false), ("Crawling Barrens", true)] {
        let mut game = scenario(payer);
        let source = put(&mut game, payer, name, Zone::Battlefield);
        let object = game.object_ref(source);
        act(
            &mut game,
            Action::AnnounceActivate {
                card: source,
                ability: 1,
                counter_payment: Default::default(),
            },
        );
        let hint = named(AutoTapFollowUp::Activate { object, ability: 1 });
        if repeatable {
            pay_with(&mut game, &hint);
        } else {
            assert_declined_hint(&mut game, &hint);
        }
    }
    let mut game = scenario(payer);
    let offering = put(&mut game, payer, "Isolinear Processor", Zone::Battlefield);
    let spell = put(&mut game, payer, "Realm-Scorcher Hellkite", Zone::Hand);
    let object = game.object_ref(offering);
    act(&mut game, Action::AnnounceCast { card: spell });
    act(
        &mut game,
        Action::ChooseBargain {
            sacrifice: Some(object),
        },
    );
    assert_declined_hint(
        &mut game,
        &named(AutoTapFollowUp::Activate { object, ability: 1 }),
    );
}

fn assert_declined_hint(game: &mut Game, hint: &AutoTapOptions) {
    let actor = game.actor().unwrap();
    let attempt = game.attempt_view(actor).unwrap().attempt_id;
    let revision = game.revision();
    let state = game.snapshot();
    assert!(matches!(
        game.plan_mana_payment(actor, revision, attempt, hint),
        Err(Error::IllegalAction)
    ));
    assert!(matches!(
        game.auto_pay_mana(actor, revision, attempt, hint),
        Err(Error::IllegalAction)
    ));
    assert_eq!(game.snapshot(), state);
    assert_eq!(game.revision(), revision);
}
