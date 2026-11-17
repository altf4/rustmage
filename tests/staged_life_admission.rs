//! Mandatory known life is an immediate requirement of the selected cast route.
mod support;
use rustmage as _;
use rustmage_core::engine::*;
use support::*;

#[test]
fn an_independent_mana_additional_cost_keeps_low_life_declarations_available() {
    for actor in [Player::P0, Player::P1] {
        for route in 0..3 {
            let mut game = scenario(actor);
            let shot = put(&mut game, actor.other(), "Lightning Strike", Zone::Hand);
            game.state.priority = actor.other();
            cast(&mut game, shot, Some(Target::Player(actor)));
            if game.actor() != Some(actor) {
                act(&mut game, Action::Pass);
            }
            if route == 2 {
                put(&mut game, actor, "Omniscience", Zone::Battlefield);
            }
            let card = put(
                &mut game,
                actor,
                "Redirect Lightning",
                if route == 1 { Zone::Exile } else { Zone::Hand },
            );
            let original = game.object_ref(card);
            if route == 1 {
                game.grant_exile_play_permission(actor, original, false);
            }
            game.state.players[actor.index()].life = 4;
            game.state.players[actor.index()].mana = [0; 6];
            let action = match route {
                0 => Action::AnnounceCast { card },
                1 => Action::AnnounceExile { card },
                _ => Action::AnnounceFree {
                    card,
                    face: CastFace::Normal,
                    flash_surcharge: false,
                },
            };
            Game::reset_payment_planning_work();
            admit(&game, action.clone());
            act(&mut game, action);
            // Life is still checked on the declared nonmana equivalent. The
            // mana equivalent is allowed even when its actual pool is empty.
            reject(&mut game, Action::ChooseSpellCost { cost: 0 });
            admit(&game, Action::ChooseSpellCost { cost: 1 });
            act(&mut game, Action::ChooseSpellCost { cost: 1 });
            act(
                &mut game,
                Action::ChooseTarget {
                    target: Target::Spell(shot),
                },
            );
            reject(&mut game, Action::PayCosts { payment: [0; 6] });
            act(&mut game, Action::EndManaWindow);
            act(&mut game, Action::SubmitAttempt);
            let failed = game.last_attempt_outcome_view(actor).unwrap();
            assert_eq!(failed.outcome, AttemptOutcome::Failed);
            assert_eq!(failed.reason, Some(AttemptFailureReason::UnsatisfiedMana));
            assert_eq!(game.object_ref(card), original);
            assert_eq!(
                game.object_view(card).zone,
                if route == 1 { Zone::Exile } else { Zone::Hand }
            );
            assert_eq!(game.state.players[actor.index()].life, 4);
            assert_eq!(game.state.players[actor.index()].mana, [0; 6]);
            let work = Game::payment_planning_work();
            assert_eq!(work.projected_moves, 0);
            assert_eq!(work.projected_steps, 0);
            assert_eq!(work.continuation_plans, 0);
        }
    }
}

#[test]
fn known_route_life_is_checked_without_proving_future_mana_or_materials() {
    for actor in [Player::P0, Player::P1] {
        for (name, zone, warp) in [
            ("Wickerfolk Indomitable", Zone::Graveyard, false),
            ("Timeline Culler", Zone::Hand, true),
        ] {
            let mut game = scenario(actor);
            let card = put(&mut game, actor, name, zone);
            let object = game.object_ref(card);
            let action = if warp {
                Action::AnnounceWarp { card }
            } else {
                Action::AnnounceGraveyardAdditionalCosts { card }
            };
            game.state.players[actor.index()].mana = [0; 6];
            game.state.players[actor.index()].life = 1;
            Game::reset_payment_planning_work();
            assert!(!game.legal_actions().contains(&action));
            reject(&mut game, action.clone());
            assert_eq!(game.object_ref(card), object);
            // Exactly enough life admits the route even with no mana or
            // sacrifice material. Those concrete requirements are paid later.
            game.state.players[actor.index()].life = 2;
            admit(&game, action);
            let work = Game::payment_planning_work();
            assert_eq!(work.continuation_plans, 0);
            assert_eq!(work.projected_steps, 0);
            assert_eq!(work.projected_moves, 0);
        }
    }
}

#[test]
fn free_mana_alternative_preserves_only_its_selected_origin_life() {
    use rustmage_core::cards::authored::{CardExtras, CardRules};
    use rustmage_core::cards::*;
    const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::FreeSpellCast(CastFilter::Any)],
    }];
    const FREE: CardRules = CardRules {
        effect: Effect::Creature(2, 2),
        static_groups: GROUPS,
        extras: &CardExtras {
            static_ability_groups: GROUPS,
            ..CardExtras::EMPTY
        },
        ..CardRules::EMPTY
    };
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &FREE)]);
    for actor in [Player::P0, Player::P1] {
        let mut game = scenario(actor);
        put(&mut game, actor, "Grizzly Bears", Zone::Battlefield);
        let card = put(&mut game, actor, "Wickerfolk Indomitable", Zone::Graveyard);
        let action = Action::AnnounceFreeGraveyardAdditionalCosts {
            card,
            face: CastFace::Normal,
            flash_surcharge: false,
        };
        game.state.players[actor.index()].mana = [0; 6];
        game.state.players[actor.index()].life = 1;
        assert!(!game.legal_actions().contains(&action));
        reject(&mut game, action.clone());
        game.state.players[actor.index()].life = 2;
        admit(&game, action);
        // An independent ordinary permission does not inherit the intrinsic
        // route's life or sacrifice costs merely because the card is the same.
        game.state.players[actor.index()].life = 1;
        game.grant_graveyard_permission(actor, game.object_ref(card), GraveyardCastKind::Cast);
        admit(
            &game,
            Action::AnnounceFree {
                card,
                face: CastFace::Normal,
                flash_surcharge: false,
            },
        );
    }
}
