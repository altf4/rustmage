mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

fn priority(game: &mut Game) {
    choose(game, |a| {
        matches!(a, Action::PutTrigger { .. } | Action::Pass)
    });
}

fn until(game: &mut Game, action: Action) {
    for _ in 0..20 {
        if game.legal_actions().contains(&action) {
            return;
        }
        priority(game);
    }
    panic!("missing {action:?} in {:?}", game.state.pending);
}

fn end_step(game: &mut Game) {
    game.begin_step(Step::End);
    game.prepare_triggers();
    settle(game);
}

#[test]
fn driftgloom_coyote_uses_the_power_at_exile_and_returns_the_linked_creature() {
    let player = Player::P0;
    for (name, counters) in [("Grizzly Bears", 1), ("Hill Giant", 0)] {
        let mut game = scenario(player);
        let victim = put(&mut game, player.other(), name, Zone::Battlefield);
        let own = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let coyote = put(&mut game, player, "Driftgloom Coyote", Zone::Hand);
        let original = game.object_ref(victim);
        cast(&mut game, coyote, None);
        until(&mut game, target(victim));
        reject(&mut game, target(own));
        act(&mut game, target(victim));
        settle(&mut game);
        assert_eq!(game.object_view(victim).zone, Zone::Exile);
        assert_eq!(game.counter_count(coyote, CounterKind::PlusOne), counters);
        game.move_object(coyote, Zone::Graveyard);
        assert_eq!(game.object_view(victim).zone, Zone::Battlefield);
        assert_ne!(game.object_ref(victim), original);
        assert_eq!(game.object_view(victim).controller, player.other());
    }
}

#[test]
fn salvation_swan_flashes_and_watches_other_birds_for_the_flying_counter() {
    let player = Player::P1;
    let mut game = scenario(player.other());
    let grounded = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let second = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let flier = put(&mut game, player, "Healer's Hawk", Zone::Battlefield);
    let swan = put(&mut game, player, "Salvation Swan", Zone::Hand);
    act(&mut game, Action::Pass);
    cast(&mut game, swan, None);
    until(&mut game, target(grounded));
    reject(&mut game, target(flier));
    act(&mut game, target(grounded));
    settle(&mut game);
    assert!(game.object_view(swan).keywords.contains(Keywords::FLYING));
    assert_eq!(game.object_view(grounded).zone, Zone::Exile);
    end_step(&mut game);
    assert_eq!(game.counter_count(grounded, CounterKind::Flying), 1);

    game.finish_cleanup();
    game.state.step = Step::Main1;
    game.state.priority = player;
    game.state.players[player.index()].mana = [8; 6];
    let bird = put(&mut game, player, "Healer's Hawk", Zone::Hand);
    cast(&mut game, bird, None);
    until(&mut game, target(second));
    act(&mut game, target(second));
    settle(&mut game);
    assert_eq!(game.object_view(second).zone, Zone::Exile);
    end_step(&mut game);
    assert_eq!(game.counter_count(second, CounterKind::Flying), 1);
}

#[test]
fn skyskipper_blinks_only_another_controlled_creature_under_its_owner() {
    let player = Player::P0;
    let mut game = scenario(player);
    let recipient = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    game.record_control(game.object_ref(recipient), player, None);
    let opposing = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    let duo = put(&mut game, player, "Skyskipper Duo", Zone::Hand);
    cast(&mut game, duo, None);
    until(&mut game, target(recipient));
    assert!(game.object_view(duo).keywords.contains(Keywords::FLYING));
    reject(&mut game, target(duo));
    reject(&mut game, target(opposing));
    let mut decline = game.clone();
    act(&mut decline, Action::FinishTargets);
    settle(&mut decline);
    assert_eq!(decline.object_view(recipient).zone, Zone::Battlefield);
    act(&mut game, target(recipient));
    settle(&mut game);
    assert_eq!(game.object_view(recipient).zone, Zone::Exile);
    end_step(&mut game);
    assert_eq!(game.object_view(recipient).controller, player.other());
    assert_eq!(game.object_view(recipient).zone, Zone::Battlefield);
}

#[test]
fn splash_portal_checks_the_returned_creatures_subtype() {
    let player = Player::P1;
    for (name, draws) in [("Healer's Hawk", 1), ("Grizzly Bears", 0)] {
        let mut game = scenario(player);
        let creature = put(&mut game, player.other(), name, Zone::Battlefield);
        game.record_control(game.object_ref(creature), player, None);
        let original = game.object_ref(creature);
        if draws == 0 {
            game.apply_temporary_copy(
                original,
                CopyRecipe::new(Card::from_name("Healer's Hawk").unwrap()),
                CopyDeadline::EndOfTurn,
            );
        }
        let spell = put(&mut game, player, "Splash Portal", Zone::Hand);
        cast(&mut game, spell, Some(Target::Card(creature)));
        settle(&mut game);
        assert_ne!(game.object_ref(creature), original);
        assert_eq!(game.object_view(creature).zone, Zone::Battlefield);
        assert_eq!(game.object_view(creature).controller, player.other());
        assert_eq!(
            game.state.players[player.index()].cards_drawn_this_turn,
            draws
        );
    }
}

#[test]
fn lurker_grants_distinct_wards_and_its_untargeted_damage_blink_survives_departure() {
    let player = Player::P0;
    let other = player.other();
    let mut game = scenario(player);
    let frog = put(&mut game, player, "Splash Lasher", Zone::Battlefield);
    let watched = put(&mut game, other, "Grizzly Bears", Zone::Battlefield);
    let original = game.object_ref(watched);
    game.change_controller(original, player);
    game.grant_keyword_effects(&[watched], original, Keywords::HASTE);
    let lurker = put(&mut game, player, "Long River Lurker", Zone::Hand);
    cast(&mut game, lurker, None);
    until(&mut game, target(watched));
    act(&mut game, target(watched));
    settle(&mut game);
    assert!(
        game.object_view(watched)
            .keywords
            .contains(Keywords::CANT_BE_BLOCKED)
    );

    for recipient in [frog, lurker] {
        let mut ward = game.clone();
        if ward.actor() != Some(other) {
            act(&mut ward, Action::Pass);
        }
        let spell = put(&mut ward, other, "Shock", Zone::Hand);
        cast(&mut ward, spell, Some(Target::Card(recipient)));
        until(&mut ward, Action::DeclineResolutionMana);
        act(&mut ward, Action::DeclineResolutionMana);
        settle(&mut ward);
        assert_eq!(ward.object_view(recipient).damage, 0);
    }

    game.move_object(lurker, Zone::Graveyard);
    let attack = Action::Attack { card: watched };
    until(&mut game, attack.clone());
    act(&mut game, attack);
    act(&mut game, Action::FinishAttackers);
    until(&mut game, Action::FinishBlockers);
    act(&mut game, Action::FinishBlockers);
    let blink = Action::ChooseOptional { accept: true };
    until(&mut game, blink.clone());
    act(&mut game, blink);
    settle(&mut game);
    assert_ne!(game.object_ref(watched), original);
    assert_eq!(game.object_view(watched).controller, other);
    assert_eq!(game.state.players[other.index()].life, 18);
}

#[test]
fn waverider_and_wishing_well_offer_only_their_legal_free_casts() {
    let player = Player::P1;
    for well in [false, true] {
        let mut game = scenario(player);
        let spell = put(&mut game, player, "Divination", Zone::Graveyard);
        let too_expensive = put(&mut game, player, "Overrun", Zone::Graveyard);
        let wrong_type = put(&mut game, player, "Llanowar Elves", Zone::Graveyard);
        let wrong_owner = put(&mut game, player.other(), "Divination", Zone::Graveyard);
        if well {
            let source = put(&mut game, player, "Wishing Well", Zone::Battlefield);
            game.add_counter(source, CounterKind::Coin, 2);
            choose(&mut game, |action| {
                matches!(action,
                    Action::Activate { card, ability: 0, .. }
                    | Action::AnnounceActivate { card, ability: 0, .. } if *card == source
                )
            });
            if matches!(game.state.pending, Pending::PayCosts { .. }) {
                pay(&mut game);
            }
            until(&mut game, target(spell));
            assert_eq!(game.counter_count(source, CounterKind::Coin), 3);
        } else {
            let source = put(&mut game, player, "Daring Waverider", Zone::Hand);
            cast(&mut game, source, None);
            until(&mut game, target(spell));
        }
        for card in [too_expensive, wrong_type, wrong_owner] {
            reject(&mut game, target(card));
        }
        act(&mut game, target(spell));
        let offer = Action::CastExiledDuringResolution {
            object: game.object_ref(spell),
            face: CastFace::Normal,
        };
        until(&mut game, offer.clone());
        let mut decline = game.clone();
        act(&mut decline, Action::FinishResolutionCasts);
        settle(&mut decline);
        assert_eq!(decline.object_view(spell).zone, Zone::Graveyard);
        game.state.players[player.index()].mana = [0; 6];
        act(&mut game, offer);
        if matches!(game.state.pending, Pending::PayCosts { .. }) {
            pay(&mut game);
        }
        if matches!(game.state.pending, Pending::ResolutionCasts { .. }) {
            act(&mut game, Action::FinishResolutionCasts);
        }
        settle(&mut game);
        assert_eq!(game.object_view(spell).zone, Zone::Exile);
        assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 2);
    }
}
