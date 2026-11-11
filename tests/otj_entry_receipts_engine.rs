//! Exact batch entry observers use actual cast receipts and entry characteristics.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const OBSERVER: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::CreaturesEnterYouWithoutManaSpent,
        targets: TargetPlan::NONE,
        program: &[Instruction::Draw(PlayerSet::Controller, 1)],
    }],
    ..CardRules::EMPTY
};

#[test]
fn entry_observer_distinguishes_paid_free_and_paid_additional_casts() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &OBSERVER)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let paid_observer = put(&mut game, player, "Grizzly Bears", Zone::Hand);
    cast(&mut game, paid_observer, None);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 0);

    let paid_other = put(&mut game, player, "Savannah Lions", Zone::Hand);
    cast(&mut game, paid_other, None);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 0);

    put(&mut game, player, "Omniscience", Zone::Battlefield);
    for (kicked, expected_draws) in [(true, 0), (false, 1)] {
        let entrant = put(&mut game, player, "Gnarlid Colony", Zone::Hand);
        act(
            &mut game,
            Action::AnnounceFree {
                card: entrant,
                face: CastFace::Normal,
                flash_surcharge: false,
            },
        );
        act(&mut game, Action::ChooseKicker { pay: kicked });
        if matches!(game.state.pending, Pending::PayCosts { .. }) {
            pay(&mut game);
        }
        assert_eq!(
            game.state.stack.last().unwrap().cast.unwrap().method,
            CastMethod::Free
        );
        assert_eq!(
            game.state.stack.last().unwrap().cast.unwrap().payment != [0; 6],
            kicked
        );
        settle(&mut game);
        assert_eq!(game.object_view(entrant).zone, Zone::Battlefield);
        assert_eq!(
            game.state.players[player.index()].cards_drawn_this_turn,
            expected_draws
        );
    }
}

fn order_entries(game: &mut Game) {
    for _ in 0..4 {
        let Some(action) = game
            .legal_actions()
            .into_iter()
            .find(|action| matches!(action, Action::ChooseEntryTimestamp { .. }))
        else {
            assert!(game.state.entry_batch.is_none());
            game.prepare_triggers();
            return;
        };
        act(game, action);
    }
    panic!("entry timestamp choice budget exhausted");
}

#[test]
fn a_batch_including_the_uncast_observer_triggers_only_once_and_ignores_other_tokens() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &OBSERVER)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let observer = put(&mut game, player, "Grizzly Bears", Zone::Graveyard);
    let other = put(&mut game, player, "Savannah Lions", Zone::Graveyard);
    let moves = [observer, other]
        .into_iter()
        .map(|card| game.entry_move(card, Zone::Battlefield, false))
        .collect();
    game.stage_entries(moves, None);
    order_entries(&mut game);
    assert_eq!(game.state.stack.len(), 1);
    assert_eq!(game.state.stack[0].source.unwrap().id, observer);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);

    let tokens = put(&mut game, player, "Raise the Alarm", Zone::Hand);
    cast(&mut game, tokens, None);
    for _ in 0..12 {
        if game.state.stack.is_empty()
            && game.state.resolving.is_none()
            && game.state.pending == Pending::Priority
        {
            break;
        }
        choose(&mut game, |action| {
            matches!(action, Action::Pass | Action::ChooseEntryTimestamp { .. })
        });
    }
    assert!(game.state.stack.is_empty() && game.state.resolving.is_none());
    assert_eq!(game.object_view(tokens).zone, Zone::Graveyard);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
}

#[test]
fn a_paid_cast_receipt_disqualifies_the_otherwise_eligible_whole_batch() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &OBSERVER)]);
    let player = Player::P0;
    let mut parent = scenario(player);
    put(&mut parent, player, "Grizzly Bears", Zone::Battlefield);
    let uncast = put(&mut parent, player, "Llanowar Elves", Zone::Graveyard);
    let casted = put(&mut parent, player, "Savannah Lions", Zone::Hand);
    cast(&mut parent, casted, None);
    let spell = parent.state.stack.pop().unwrap();
    let cast_event = spell.actual_cast_event().unwrap();
    assert_ne!(cast_event.payment, [0; 6]);
    for (has_payment, draws) in [(false, 1), (true, 0)] {
        let mut game = parent.clone();
        let first = game.entry_move(uncast, Zone::Battlefield, false);
        let mut second = game.entry_move(casted, Zone::Battlefield, false);
        // Exercise the common batch dispatcher with both kinds of actual
        // receipts in a simultaneous event assembled by the fixture.
        second.cast_origin = Some(Zone::Hand);
        second.cast_event = Some(Box::new(CastPaymentEvent {
            payment: if has_payment {
                cast_event.payment
            } else {
                [0; 6]
            },
            ..cast_event
        }));
        second.cast_payment = second.cast_event.as_ref().map(|event| event.payment);
        game.stage_entries(vec![first, second], None);
        order_entries(&mut game);
        settle(&mut game);
        assert_eq!(
            game.state.players[player.index()].cards_drawn_this_turn,
            draws
        );
    }
}
