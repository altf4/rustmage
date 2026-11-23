//! Affordability at proposal time must preserve affordable routes and manual mana.
use crate::support::*;
use rustmage as _;
use rustmage_core::cards::Counters;
use rustmage_core::engine::*;

fn reject_at_priority(game: &mut Game, player: Player, action: Action) {
    let snapshot = game.snapshot();
    let revision = game.revision();
    assert!(!game.legal_actions().contains(&action));
    assert!(game.apply(player, action).is_err());
    assert_eq!(game.revision(), revision);
    assert_eq!(game.snapshot(), snapshot);
}

#[test]
fn unpaid_spell_is_hidden_and_funded_spell_casts_from_either_seat() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player).with_proposal_affordability(true);
        game.state.players[player.index()].mana = [0; 6];
        let spell = put(&mut game, player, "Hill Giant", Zone::Hand);
        let cast = Action::AnnounceCast { card: spell };
        Game::reset_payment_planning_work();
        reject_at_priority(&mut game, player, cast.clone());

        let mountain = put(&mut game, player, "Mountain", Zone::Battlefield);
        let tap = Action::TapMana { card: mountain };
        assert!(game.legal_actions().contains(&tap));
        assert!(game.legal_actions().contains(&Action::Pass));
        reject_at_priority(&mut game, player, cast.clone());

        game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 3];
        let index = game
            .legal_actions()
            .iter()
            .position(|a| a == &cast)
            .unwrap();
        let mut indexed = game.clone();
        indexed
            .apply_index(player, indexed.revision(), index)
            .unwrap();
        act(&mut game, cast);
        assert_eq!(game.snapshot(), indexed.snapshot());
        act(&mut game, tap);
        act(
            &mut game,
            Action::PayCosts {
                payment: [0, 0, 0, 1, 0, 3],
            },
        );
        assert_eq!(game.state.players[player.index()].spells_cast_this_turn, 1);
        let work = Game::payment_planning_work();
        assert_eq!(
            (
                work.projected_moves,
                work.projected_steps,
                work.continuation_plans
            ),
            (0, 0, 0)
        );
    }
}

#[test]
fn tap_cost_reserves_the_activating_permanent_for_fomori_vault() {
    let player = Player::P0;
    let mut game = scenario(player).with_proposal_affordability(true);
    game.state.players[player.index()].mana = [0; 6];
    let source = put(&mut game, player, "Fomori Vault", Zone::Battlefield);
    let discard = put(&mut game, player, "Hill Giant", Zone::Hand);
    let discard = game.object_ref(discard);
    let activate = Action::AnnounceActivate {
        card: source,
        ability: 1,
        counter_payment: Counters::default(),
    };
    for _ in 0..2 {
        put(&mut game, player, "Mountain", Zone::Battlefield);
    }
    reject_at_priority(&mut game, player, activate.clone());
    put(&mut game, player, "Mountain", Zone::Battlefield);
    act(&mut game, activate);
    act(&mut game, Action::ChooseAdditionalCost { object: discard });
    let attempt = game.attempt_view(player).unwrap();
    let result = game
        .auto_pay_mana(
            player,
            game.revision(),
            attempt.attempt_id,
            &Default::default(),
        )
        .unwrap();
    assert_eq!(result.status, AutoTapStatus::Completed);
    assert!(game.object_view(source).tapped);
}

#[test]
fn conditional_mana_requires_its_land_type_but_a_tapped_land_supplies_that_type() {
    let player = Player::P0;
    let mut game = scenario(player).with_proposal_affordability(true);
    game.state.players[player.index()].mana = [0; 6];
    let verge = put(&mut game, player, "Riverpyre Verge", Zone::Battlefield);
    let spell = put(&mut game, player, "Opt", Zone::Hand);
    let cast = Action::AnnounceCast { card: spell };
    reject_at_priority(&mut game, player, cast.clone());
    let mountain = put(&mut game, player, "Mountain", Zone::Battlefield);
    game.state.objects[mountain as usize].tapped = true;
    act(&mut game, cast);
    choose(&mut game, |action| {
        matches!(action,
        Action::ActivateMana { card, production, .. }
            if *card == verge && *production == [0, 1, 0, 0, 0, 0])
    });
    act(
        &mut game,
        Action::PayCosts {
            payment: [0, 1, 0, 0, 0, 0],
        },
    );
    assert_eq!(game.state.players[player.index()].spells_cast_this_turn, 1);
}

#[test]
fn a_single_dual_land_cannot_pay_both_colored_symbols() {
    let player = Player::P0;
    let mut game = scenario(player).with_proposal_affordability(true);
    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 1];
    put(&mut game, player, "Concealed Courtyard", Zone::Battlefield);
    let card = put(&mut game, player, "Amalia Benavides Aguirre", Zone::Hand);
    let cast = Action::AnnounceCast { card };
    reject_at_priority(&mut game, player, cast.clone());
    put(&mut game, player, "Plains", Zone::Battlefield);
    act(&mut game, cast);
    let attempt = game.attempt_view(player).unwrap();
    let payment = game
        .auto_pay_mana(
            player,
            game.revision(),
            attempt.attempt_id,
            &Default::default(),
        )
        .unwrap();
    assert_eq!(payment.status, AutoTapStatus::Completed);
}

#[test]
fn a_graveyard_discount_keeps_an_affordable_spell_available() {
    let player = Player::P0;
    let mut game = scenario(player).with_proposal_affordability(true);
    game.state.players[player.index()].mana = [0, 2, 0, 0, 0, 0];
    let card = put(&mut game, player, "Eddymurk Crab", Zone::Hand);
    for _ in 0..5 {
        put(&mut game, player, "Opt", Zone::Graveyard);
    }
    act(&mut game, Action::AnnounceCast { card });
    act(
        &mut game,
        Action::PayCosts {
            payment: [0, 2, 0, 0, 0, 0],
        },
    );
    assert_eq!(game.state.players[player.index()].spells_cast_this_turn, 1);
}

#[test]
fn a_turn_discount_keeps_a_face_down_cast_available() {
    let player = Player::P0;
    let mut game = scenario(player).with_proposal_affordability(true);
    let disguised = put(&mut game, player, "Fugitive Codebreaker", Zone::Hand);
    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 1];
    // The native continuous rule added by two Goblin Maskmaker attack triggers.
    game.grant_face_down_spell_reduction(player, 2);
    let cast = Action::AnnounceDisguise { card: disguised };
    assert!(game.legal_actions().contains(&cast));
    act(&mut game, cast);
    act(
        &mut game,
        Action::PayCosts {
            payment: [0, 0, 0, 0, 0, 1],
        },
    );
    assert_eq!(game.state.players[player.index()].spells_cast_this_turn, 1);
}
