mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn cast_x(game: &mut Game, card: CardId, x: u32) {
    act(game, Action::AnnounceCast { card });
    act(game, Action::ChooseX { value: x });
    pay(game);
    act(game, Action::Pass);
    act(game, Action::Pass);
}

#[test]
fn mockingbird_uses_the_total_mana_spent_and_adds_flying_and_bird() {
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Mockingbird", Zone::Hand);
    let bear = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    let giant = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    game.add_counter(bear, CounterKind::PlusOne, 2);
    game.state.players[player.index()].mana = [0, 1, 0, 0, 0, 1];
    cast_x(&mut game, source, 1);
    let choose = |g: &Game, card| Action::ChooseEntryCopy {
        object: Some(g.object_ref(card)),
    };
    let too_expensive = choose(&game, giant);
    reject(&mut game, too_expensive);
    let mut original = game.clone();
    act(&mut original, Action::ChooseEntryCopy { object: None });
    settle(&mut original);
    assert_eq!(stats(&original, source), (1, 1));
    let copy_bear = choose(&game, bear);
    act(&mut game, copy_bear);
    settle(&mut game);
    let view = game.object_view(source);
    assert_eq!(view.name, "Grizzly Bears");
    assert!(view.has_subtype("Bird") && view.has_subtype("Bear"));
    assert!(view.keywords.contains(Keywords::FLYING));
    assert_eq!(stats(&game, source), (2, 2));
}

#[test]
fn otterball_antics_marks_only_the_token_created_by_the_nonhand_cast() {
    let player = Player::P0;
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Otterball Antics", Zone::Hand);
    cast(&mut game, spell, None);
    settle(&mut game);
    let first = game.state.objects.len() as CardId - 1;
    assert_eq!(game.object_view(spell).zone, Zone::Graveyard);
    assert_eq!(game.counter_count(first, CounterKind::PlusOne), 0);
    assert_eq!(stats(&game, first), (1, 1));
    act(&mut game, Action::AnnounceFlashback { card: spell });
    pay(&mut game);
    settle(&mut game);
    let second = game.state.objects.len() as CardId - 1;
    assert_ne!(first, second);
    assert_eq!(game.object_view(spell).zone, Zone::Exile);
    assert_eq!(stats(&game, first), (2, 2));
    assert_eq!(game.counter_count(first, CounterKind::PlusOne), 0);
    assert_eq!(game.counter_count(second, CounterKind::PlusOne), 1);
    for card in [first, second] {
        let view = game.object_view(card);
        assert_eq!(view.colors, 10);
        assert!(view.has_subtype("Otter") && view.is_token());
    }
}

#[test]
fn portent_reveals_publicly_uses_distinct_types_and_can_cast_a_fourth_selection() {
    let player = Player::P1;
    let mut game = scenario(player);
    let duplicate = put(&mut game, player, "Negate", Zone::Library);
    let land = put(&mut game, player, "Island", Zone::Library);
    let creature = put(&mut game, player, "Grizzly Bears", Zone::Library);
    let instant = put(&mut game, player, "Opt", Zone::Library);
    let draw = put(&mut game, player, "Divination", Zone::Library);
    let spell = put(&mut game, player, "Portent of Calamity", Zone::Hand);
    cast_x(&mut game, spell, 5);
    let public = game.observe(player.other()).library_selection.unwrap();
    assert!(public.public);
    assert_eq!(public.remaining.len(), 5);
    for card in [draw, instant, creature] {
        act(&mut game, Action::SelectLibrary { card });
    }
    reject(&mut game, Action::SelectLibrary { card: duplicate });
    let mut three = game.clone();
    act(&mut three, Action::FinishLibrarySelection);
    assert_eq!(three.state.pending, Pending::Priority);
    assert_eq!(three.object_view(draw).zone, Zone::Hand);
    assert_eq!(three.object_view(land).zone, Zone::Graveyard);

    act(&mut game, Action::SelectLibrary { card: land });
    act(&mut game, Action::FinishLibrarySelection);
    assert_eq!(game.state.pending, Pending::ResolutionCasts { player });
    let invalid_land = Action::CastExiledDuringResolution {
        object: game.object_ref(land),
        face: CastFace::Normal,
    };
    reject(&mut game, invalid_land);
    let action = Action::CastExiledDuringResolution {
        object: game.object_ref(draw),
        face: CastFace::Normal,
    };
    game.state.players[player.index()].mana = [0; 6];
    act(&mut game, action);
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(&mut game);
    }
    if matches!(game.state.pending, Pending::ResolutionCasts { .. }) {
        act(&mut game, Action::FinishResolutionCasts);
    }
    settle(&mut game);
    for card in [instant, creature, land] {
        assert_eq!(game.object_view(card).zone, Zone::Hand);
    }
    assert_eq!(game.object_view(duplicate).zone, Zone::Graveyard);
    assert_eq!(game.object_view(draw).zone, Zone::Graveyard);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 2);
}
