mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

fn advance(game: &mut Game) {
    choose(game, |action| {
        matches!(action, Action::PutTrigger { .. } | Action::Pass)
    });
}

fn drain(game: &mut Game) {
    for _ in 0..32 {
        if game.state.stack.is_empty()
            && game.state.pending_triggers.is_empty()
            && game.state.ordering_triggers.is_empty()
            && game.state.resolving.is_none()
            && game.state.pending == Pending::Priority
        {
            return;
        }
        advance(game);
    }
    panic!("gift test did not settle");
}

fn gift_cast(game: &mut Game, card: CardId, recipient: Option<CardId>, promise: bool) {
    let beneficiary = promise.then(|| game.actor().unwrap().other());
    act(game, Action::AnnounceCast { card });
    act(game, Action::ChooseGift { beneficiary });
    if let Some(recipient) = recipient {
        act(game, target(recipient));
    }
    pay(game);
}

#[test]
fn kitnap_and_gerbils_observe_only_the_gift_that_the_aura_gives() {
    for promise in [false, true] {
        let player = Player::P0;
        let other = player.other();
        let mut game = scenario(player);
        put(&mut game, player, "Jolly Gerbils", Zone::Battlefield);
        let creature = put(&mut game, other, "Grizzly Bears", Zone::Battlefield);
        let aura = put(&mut game, player, "Kitnap", Zone::Hand);
        gift_cast(&mut game, aura, Some(creature), promise);
        assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 0);
        assert_eq!(game.state.players[other.index()].cards_drawn_this_turn, 0);
        let mut stale = game.clone();
        stale.move_object(creature, Zone::Exile);
        drain(&mut stale);
        assert_eq!(stale.object_view(aura).zone, Zone::Graveyard);
        assert_eq!(stale.state.players[player.index()].cards_drawn_this_turn, 0);
        assert_eq!(stale.state.players[other.index()].cards_drawn_this_turn, 0);

        drain(&mut game);
        assert_eq!(game.object_view(creature).controller, player);
        assert!(game.object_view(creature).tapped);
        assert_eq!(
            game.counter_count(creature, CounterKind::Stun),
            if promise { 0 } else { 3 }
        );
        for seat in [player, other] {
            assert_eq!(
                game.state.players[seat.index()].cards_drawn_this_turn,
                u64::from(promise)
            );
        }
        game.move_object(aura, Zone::Graveyard);
        assert_eq!(game.object_view(creature).controller, other);
    }
}

#[test]
fn parting_gust_returns_the_exact_card_only_without_a_promise() {
    for promise in [false, true] {
        let player = Player::P1;
        let other = player.other();
        let mut game = scenario(player);
        let creature = put(&mut game, other, "Bear Cub", Zone::Battlefield);
        game.record_control(game.object_ref(creature), player, None);
        game.create_tokens(player, Card(5317), 1, false);
        let original_token = game.state.objects.len() as CardId - 1;
        let spell = put(&mut game, player, "Parting Gust", Zone::Hand);
        act(&mut game, Action::AnnounceCast { card: spell });
        act(
            &mut game,
            Action::ChooseGift {
                beneficiary: promise.then_some(other),
            },
        );
        reject(&mut game, target(original_token));
        act(&mut game, target(creature));
        pay(&mut game);
        drain(&mut game);
        assert_eq!(game.object_view(creature).zone, Zone::Exile);
        assert_eq!(
            game.observe(player).delayed_returns.len(),
            usize::from(!promise)
        );
        assert_eq!(
            game.observe(other)
                .objects
                .iter()
                .filter(|object| {
                    object.is_token()
                        && object.zone == Zone::Battlefield
                        && object.controller == other
                        && object.tapped
                        && object.card == Card(5317)
                })
                .count(),
            usize::from(promise)
        );
        game.begin_step(Step::End);
        game.prepare_triggers();
        drain(&mut game);
        if promise {
            assert_eq!(game.object_view(creature).zone, Zone::Exile);
        } else {
            assert_eq!(game.object_view(creature).zone, Zone::Battlefield);
            assert_eq!(game.object_view(creature).controller, other);
            assert_eq!(game.counter_count(creature, CounterKind::PlusOne), 1);
        }
    }
}

#[test]
fn starfall_returns_one_of_the_owners_actual_destroyed_cards_only_with_gift() {
    for promise in [false, true] {
        let player = Player::P0;
        let other = player.other();
        let mut game = scenario(player);
        let prior = put(&mut game, player, "Hill Giant", Zone::Graveyard);
        let own = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
        let enemy = put(&mut game, other, "Grizzly Bears", Zone::Battlefield);
        game.record_control(game.object_ref(own), other, None);
        game.record_control(game.object_ref(enemy), player, None);
        let doomed = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        game.add_counter(doomed, CounterKind::Finality, 1);
        let spell = put(&mut game, player, "Starfall Invocation", Zone::Hand);
        gift_cast(&mut game, spell, None, promise);
        advance(&mut game);
        advance(&mut game);
        if promise {
            for ineligible in [prior, enemy, doomed] {
                let object = game.object_ref(ineligible);
                reject(&mut game, Action::ChooseEffectObject { object });
            }
            let object = game.object_ref(own);
            act(&mut game, Action::ChooseEffectObject { object });
        }
        drain(&mut game);
        assert_eq!(
            game.object_view(own).zone,
            if promise {
                Zone::Battlefield
            } else {
                Zone::Graveyard
            }
        );
        assert_eq!(game.object_view(enemy).zone, Zone::Graveyard);
        assert_eq!(game.object_view(doomed).zone, Zone::Exile);
        assert_eq!(game.object_view(prior).zone, Zone::Graveyard);
        assert_eq!(
            game.state.players[other.index()].cards_drawn_this_turn != 0,
            promise
        );
    }
}

#[test]
fn whiskervale_only_offers_battlefield_on_its_controllers_turn() {
    for own_turn in [false, true] {
        let player = Player::P1;
        let active = if own_turn { player } else { player.other() };
        let mut game = scenario(active);
        let mouse = put(
            &mut game,
            player,
            "Whiskervale Forerunner",
            Zone::Battlefield,
        );
        let giant = put(&mut game, player, "Hill Giant", Zone::Library);
        let bear = put(&mut game, player, "Grizzly Bears", Zone::Library);
        let spell = put(&mut game, player, "Shore Up", Zone::Hand);
        if !own_turn {
            act(&mut game, Action::Pass);
        }
        cast(&mut game, spell, Some(Target::Card(mouse)));
        for _ in 0..12 {
            if game.observe(player).library_selection.is_some() {
                break;
            }
            advance(&mut game);
        }
        let selection = game.observe(player).library_selection.unwrap();
        assert_eq!(selection.remaining.len(), 5);
        assert!(game.observe(player.other()).library_selection.is_none());
        reject(&mut game, Action::SelectLibrary { card: giant });
        act(&mut game, Action::SelectLibrary { card: bear });
        act(&mut game, Action::FinishLibrarySelection);
        if own_turn {
            act(&mut game, Action::ChooseOptional { accept: true });
        }
        drain(&mut game);
        assert_eq!(
            game.object_view(bear).zone,
            if own_turn {
                Zone::Battlefield
            } else {
                Zone::Hand
            }
        );
        assert_eq!(game.object_view(giant).zone, Zone::Library);
        let again = put(&mut game, player, "Shore Up", Zone::Hand);
        if game.actor() != Some(player) {
            act(&mut game, Action::Pass);
        }
        cast(&mut game, again, Some(Target::Card(mouse)));
        drain(&mut game);
    }
}
