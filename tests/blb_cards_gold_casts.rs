//! Printed cast, play, and exact replay routes on six Bloomburrow permanents.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn announce_ability(game: &mut Game, source: CardId) {
    act(
        game,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(game);
    }
}

fn resolve_until(game: &mut Game, condition: impl Fn(&Game) -> bool) {
    for _ in 0..24 {
        if condition(game) {
            return;
        }
        choose(game, |action| {
            matches!(
                action,
                Action::Pass | Action::PutTrigger { .. } | Action::FinishBlockers
            )
        });
    }
    panic!(
        "resolution did not reach expected choice: {:?}",
        game.state.pending
    );
}

fn beginning(game: &mut Game, step: Step) {
    game.begin_step(step);
    game.give_priority();
    game.prepare_triggers();
}

#[test]
fn alania_offers_draw_then_copies_the_first_qualifying_spell() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    put(
        &mut game,
        player,
        "Alania, Divergent Storm",
        Zone::Battlefield,
    );
    let shock = put(&mut game, player, "Shock", Zone::Hand);
    cast(&mut game, shock, Some(Target::Player(opponent)));
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(opponent),
        },
    );
    resolve_until(&mut game, |g| {
        matches!(g.state.pending, Pending::EffectOptional { .. })
    });

    let mut decline = game.clone();
    act(&mut decline, Action::ChooseOptional { accept: false });
    settle(&mut decline);
    assert_eq!(decline.state.players[opponent.index()].life, 18);
    assert_eq!(
        decline.state.players[opponent.index()].cards_drawn_this_turn,
        0
    );

    act(&mut game, Action::ChooseOptional { accept: true });
    if game.legal_actions().contains(&Action::FinishStackTargets) {
        act(&mut game, Action::FinishStackTargets);
    }
    settle(&mut game);
    assert_eq!(game.state.players[opponent.index()].life, 16);
    assert_eq!(
        game.state.players[opponent.index()].cards_drawn_this_turn,
        1
    );
    assert_eq!(game.state.players[player.index()].spells_cast_this_turn, 1);
    let later = put(&mut game, player, "Shock", Zone::Hand);
    cast(&mut game, later, Some(Target::Player(opponent)));
    settle(&mut game);
    assert_eq!(game.state.players[opponent.index()].life, 14);
    assert_eq!(
        game.state.players[opponent.index()].cards_drawn_this_turn,
        1
    );
}

#[test]
fn fireglass_only_offers_one_of_two_on_the_second_main_phase() {
    let player = Player::P1;
    let mut game = scenario(player);
    put(&mut game, player, "Fireglass Mentor", Zone::Battlefield);
    let first = put(&mut game, player, "Forest", Zone::Library);
    let second = put(&mut game, player, "Plains", Zone::Library);
    beginning(&mut game, Step::Main1);
    game.lose_life(player.other(), 1);
    game.gain_life(player.other(), 1);
    beginning(&mut game, Step::Main2);
    resolve_until(&mut game, |g| g.observe(player).milled_choice.is_some());
    for card in [first, second] {
        assert_eq!(game.object_view(card).zone, Zone::Exile);
    }
    let exact = game.object_ref(first);
    act(&mut game, Action::SelectMilled { object: exact });
    if game
        .legal_actions()
        .contains(&Action::FinishMilledSelection)
    {
        act(&mut game, Action::FinishMilledSelection);
    }
    assert_eq!(game.observe(player).exile_permissions.len(), 1);
    assert_eq!(game.observe(player).exile_permissions[0].object, exact);
    reject(&mut game, Action::PlayLand { card: second });
    act(&mut game, Action::PlayLand { card: first });
    assert_eq!(game.object_view(first).zone, Zone::Battlefield);
    beginning(&mut game, Step::Main2);
    assert!(game.state.stack.is_empty());
    assert_eq!(game.object_view(second).zone, Zone::Exile);
}

#[test]
fn glarb_keeps_its_top_private_and_surveys_into_a_payable_x_spell() {
    let player = Player::P0;
    let mut game = scenario(player);
    let glarb = put(
        &mut game,
        player,
        "Glarb, Calamity's Augur",
        Zone::Battlefield,
    );
    let expensive = put(&mut game, player, "Exsanguinate", Zone::Library);
    let cheap = put(&mut game, player, "Opt", Zone::Library);
    let land = put(&mut game, player, "Island", Zone::Library);
    assert!(game.keywords(glarb).contains(Keywords::DEATHTOUCH));
    assert_eq!(
        game.observe(player).known_library_top[player.index()]
            .as_ref()
            .unwrap()
            .object,
        game.object_ref(land)
    );
    assert!(game.observe(player.other()).known_library_top[player.index()].is_none());
    act(&mut game, Action::PlayLand { card: land });
    reject(&mut game, Action::AnnounceCast { card: cheap });
    announce_ability(&mut game, glarb);
    pass_pair(&mut game);
    let look = game.observe(player).library_choice.unwrap();
    assert!(look.surveil && look.remaining.len() == 2);
    act(
        &mut game,
        Action::ArrangeLibrary {
            card: cheap,
            destination: LibraryDestination::Graveyard,
        },
    );
    act(
        &mut game,
        Action::ArrangeLibrary {
            card: expensive,
            destination: LibraryDestination::Top,
        },
    );
    act(&mut game, Action::AnnounceCast { card: expensive });
    reject(&mut game, Action::ChooseX { value: 1 });
    assert!(game.observe(player).known_library_top[player.index()].is_none());
    act(&mut game, Action::ChooseX { value: 2 });
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(cheap).zone, Zone::Graveyard);
    assert_eq!(game.state.players[player.other().index()].life, 18);
}

#[test]
fn helga_taps_for_power_and_rewards_only_a_large_creature_spell() {
    let player = Player::P1;
    let mut game = scenario(player);
    let helga = put(&mut game, player, "Helga, Skittish Seer", Zone::Battlefield);
    let noncreature = put(&mut game, player, "Overrun", Zone::Hand);
    let small = put(&mut game, player, "Llanowar Elves", Zone::Hand);
    let large = put(&mut game, player, "Gigantosaurus", Zone::Hand);
    game.add_counter(helga, CounterKind::PlusOne, 5);
    game.state.players[player.index()].mana = [0; 6];
    act(
        &mut game,
        Action::ActivateMana {
            card: helga,
            ability: 0,
            payment: [0; 6],
            counter_payment: Counters::default(),
            production: [0, 0, 0, 0, 6, 0],
        },
    );
    reject(&mut game, Action::AnnounceCast { card: small });
    reject(&mut game, Action::AnnounceCast { card: noncreature });
    cast(&mut game, large, None);
    settle(&mut game);
    assert_eq!(game.object_view(large).zone, Zone::Battlefield);
    assert_eq!(game.state.players[player.index()].mana[4], 1);
    assert_eq!(game.state.players[player.index()].life, 21);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
    assert_eq!(game.counter_count(helga, CounterKind::PlusOne), 6);
}

#[test]
fn hugs_uses_the_paid_x_receipt_and_grants_one_additional_land_play() {
    let player = Player::P0;
    let mut game = scenario(player);
    let hugs = put(&mut game, player, "Hugs, Grisly Guardian", Zone::Hand);
    let other_land = put(&mut game, player, "Forest", Zone::Hand);
    let exiled_land = put(&mut game, player, "Plains", Zone::Library);
    let exiled_spell = put(&mut game, player, "Shock", Zone::Library);
    act(&mut game, Action::AnnounceCast { card: hugs });
    act(&mut game, Action::ChooseX { value: 2 });
    pay(&mut game);
    settle(&mut game);
    assert!(game.keywords(hugs).contains(Keywords::TRAMPLE));
    for id in [exiled_land, exiled_spell] {
        assert_eq!(game.object_view(id).zone, Zone::Exile);
        assert!(
            game.observe(player)
                .exile_permissions
                .iter()
                .any(|p| p.object.card == id)
        );
    }
    act(&mut game, Action::PlayLand { card: other_land });
    let play_exiled = Action::PlayLand { card: exiled_land };
    let mut missing_source = game.clone();
    missing_source.move_object(hugs, Zone::Exile);
    reject(&mut missing_source, play_exiled.clone());
    act(&mut game, play_exiled);
    act(&mut game, Action::AnnounceExile { card: exiled_spell });
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(player.other()),
        },
    );
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.state.players[player.other().index()].life, 18);

    let mut uncast = scenario(player);
    let hugs = put(
        &mut uncast,
        player,
        "Hugs, Grisly Guardian",
        Zone::Graveyard,
    );
    uncast.move_object(hugs, Zone::Battlefield);
    uncast.prepare_triggers();
    settle(&mut uncast);
    assert!(uncast.observe(player).exile_permissions.is_empty());
}

#[test]
fn cruelclaw_exiles_through_lands_and_allows_only_an_immediate_discard_cast() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let source = put(
        &mut game,
        player,
        "The Infamous Cruelclaw",
        Zone::Battlefield,
    );
    let card = put(&mut game, player, "Shock", Zone::Library);
    let land = put(&mut game, player, "Forest", Zone::Library);
    let payment = put(&mut game, player, "Mountain", Zone::Hand);
    assert!(game.keywords(source).contains(Keywords::MENACE));
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    act(&mut game, Action::Attack { card: source });
    act(&mut game, Action::FinishAttackers);
    resolve_until(&mut game, |g| {
        matches!(g.state.pending, Pending::ResolutionCasts { .. })
    });
    assert_eq!(game.state.players[opponent.index()].life, 17);
    assert_eq!(game.object_view(card).zone, Zone::Exile);
    assert_eq!(game.object_view(land).zone, Zone::Exile);

    let mut declined = game.clone();
    act(&mut declined, Action::FinishResolutionCasts);
    assert_eq!(declined.object_view(payment).zone, Zone::Hand);
    reject(&mut declined, Action::AnnounceExile { card });

    let object = game.object_ref(card);
    act(
        &mut game,
        Action::CastExiledDuringResolution {
            object,
            face: CastFace::Normal,
        },
    );
    let target = Action::ChooseTarget {
        target: Target::Player(opponent),
    };
    let discard = Action::ChooseAdditionalCost {
        object: game.object_ref(payment),
    };
    let zero = Action::PayCosts { payment: [0; 6] };
    for _ in 0..12 {
        if game.state.pending == Pending::Priority {
            break;
        }
        let actions = game.legal_actions();
        if let Some(action) = [
            target.clone(),
            discard.clone(),
            zero.clone(),
            Action::FinishResolutionCasts,
        ]
        .into_iter()
        .find(|action| actions.contains(action))
        {
            act(&mut game, action);
        } else {
            choose(&mut game, |action| {
                matches!(action, Action::ChooseCastCost { .. })
            });
        }
    }
    settle(&mut game);
    assert_eq!(game.object_view(payment).zone, Zone::Graveyard);
    assert_eq!(game.state.players[opponent.index()].life, 15);
}
