//! Mandatory known life is an immediate requirement of the selected cast route.
mod support;
use rustmage as _;
use rustmage_core::engine::*;
use support::*;

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
