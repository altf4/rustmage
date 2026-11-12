use crate::{reach, support::*};
use rustmage_core::engine::*;

#[test]
fn allies_at_last_uses_live_affinity_and_two_sources_without_fighting() {
    let player = Player::P1;
    let mut game = scenario(player);
    let first = put(&mut game, player, "Haru, Hidden Talent", Zone::Battlefield);
    let second = put(&mut game, player, "Wartime Protestors", Zone::Battlefield);
    let opponent = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    let spell = put(&mut game, player, "Allies at Last", Zone::Hand);
    let cast_action = Action::AnnounceCast { card: spell };
    game.state.players[player.index()].mana = [0, 0, 0, 0, 1, 0];
    let mut short = game.clone();
    short.move_object(first, Zone::Hand);
    reject(&mut short, cast_action.clone());

    act(&mut game, cast_action);
    let mut zero = game.clone();
    act(&mut zero, Action::FinishTargets);
    act(
        &mut zero,
        Action::ChooseTarget {
            target: Target::Card(opponent),
        },
    );
    pay(&mut zero);
    settle(&mut zero);
    assert_eq!(zero.state.objects[opponent as usize].damage, 0);

    for card in [first, second, opponent] {
        act(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(card),
            },
        );
    }
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(opponent).zone, Zone::Graveyard);
    assert_eq!(game.state.objects[first as usize].damage, 0);
    assert_eq!(game.state.objects[second as usize].damage, 0);
}

#[test]
fn avatar_destiny_uses_the_bearers_last_power_and_recovers_the_exact_cards() {
    let player = Player::P0;
    let mut game = scenario(player);
    let bearer = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let opposing = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    put(&mut game, player, "Grizzly Bears", Zone::Graveyard);
    let aura = put(&mut game, player, "Avatar Destiny", Zone::Hand);
    let land = put(&mut game, player, "Island", Zone::Library);
    let left = put(&mut game, player, "Hill Giant", Zone::Library);
    let right = put(&mut game, player, "Llanowar Elves", Zone::Library);
    act(&mut game, Action::AnnounceCast { card: aura });
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(opposing),
        },
    );
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(bearer),
        },
    );
    pay(&mut game);
    settle(&mut game);
    assert_eq!(stats(&game, bearer), (3, 3));
    assert!(game.object_view(bearer).has_subtype("Avatar"));
    assert!(game.object_view(bearer).has_subtype("Bear"));
    let kill = put(&mut game, player, "Murder", Zone::Hand);
    cast(&mut game, kill, Some(Target::Card(bearer)));
    reach(&mut game, |g| {
        g.legal_actions()
            .iter()
            .any(|action| matches!(action, Action::SelectMilled { .. }))
    });
    assert_eq!(game.object_view(aura).zone, Zone::Hand);
    for card in [left, right, land] {
        assert_eq!(game.object_view(card).zone, Zone::Graveyard);
    }
    let land = game.object_ref(land);
    reject(&mut game, Action::SelectMilled { object: land });
    let right_ref = game.object_ref(right);
    act(&mut game, Action::SelectMilled { object: right_ref });
    settle(&mut game);
    assert_eq!(game.object_view(right).zone, Zone::Battlefield);
    assert_eq!(game.object_view(left).zone, Zone::Graveyard);
}

#[test]
fn cycle_of_renewal_sacrifices_an_earthbent_land_and_then_searches() {
    let player = Player::P1;
    let mut game = scenario(player);
    let land = put(&mut game, player, "Forest", Zone::Battlefield);
    let enemy = put(&mut game, player.other(), "Mountain", Zone::Battlefield);
    let first = put(&mut game, player, "Island", Zone::Library);
    let second = put(&mut game, player, "Plains", Zone::Library);
    let illegal = put(&mut game, player, "Restless Cottage", Zone::Library);
    let lesson = put(&mut game, player, "Earthbending Lesson", Zone::Hand);
    cast(&mut game, lesson, Some(Target::Card(land)));
    settle(&mut game);
    let exact = game.object_ref(land);
    let spell = put(&mut game, player, "Cycle of Renewal", Zone::Hand);
    cast(&mut game, spell, None);
    reach(&mut game, |g| {
        g.legal_actions()
            .iter()
            .any(|action| matches!(action, Action::Sacrifice { .. }))
    });
    let enemy_ref = game.object_ref(enemy);
    let land_ref = game.object_ref(land);
    reject(&mut game, Action::Sacrifice { object: enemy_ref });
    act(&mut game, Action::Sacrifice { object: land_ref });
    assert_eq!(game.object_view(land).zone, Zone::Graveyard);
    reject(&mut game, Action::ChooseSearch { card: illegal });
    for card in [first, second] {
        act(&mut game, Action::ChooseSearch { card });
    }
    act(&mut game, Action::FinishSearch);
    settle(&mut game);
    assert_ne!(game.object_ref(land), exact);
    for card in [land, first, second] {
        assert_eq!(game.object_view(card).zone, Zone::Battlefield);
        assert!(game.object_view(card).tapped);
    }
}

#[test]
fn elemental_teachings_lets_the_opponent_split_only_the_public_selection() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let [first, second, third, fourth] = ["Plains", "Island", "Swamp", "Mountain"]
        .map(|name| put(&mut game, player, name, Zone::Library));
    let duplicate = put(&mut game, player, "Plains", Zone::Library);
    let hidden = put(&mut game, player, "Bear Cub", Zone::Library);
    let spell = put(&mut game, player, "Elemental Teachings", Zone::Hand);
    cast(&mut game, spell, None);
    reach(&mut game, |g| {
        g.legal_actions()
            .contains(&Action::ChooseSearch { card: first })
    });
    act(&mut game, Action::ChooseSearch { card: first });
    reject(&mut game, Action::ChooseSearch { card: duplicate });
    reject(&mut game, Action::ChooseSearch { card: hidden });
    assert!(game.observe(opponent).search_choice.is_none());
    let mut short = game.clone();
    act(&mut short, Action::FinishSearch);
    assert_eq!(short.object_view(first).zone, Zone::Graveyard);
    for card in [second, third, fourth] {
        act(&mut game, Action::ChooseSearch { card });
    }
    act(&mut game, Action::FinishSearch);
    assert_eq!(game.actor(), Some(opponent));
    let public = game.observe(opponent).search_distribution.unwrap();
    assert_eq!(
        public
            .candidates
            .iter()
            .map(|row| row.id)
            .collect::<Vec<_>>(),
        [first, second, third, fourth]
    );
    act(&mut game, Action::ChooseSearchGraveyard { card: first });
    act(&mut game, Action::ChooseSearchGraveyard { card: fourth });
    for card in [first, fourth] {
        assert_eq!(game.object_view(card).zone, Zone::Graveyard);
    }
    for card in [second, third] {
        assert_eq!(game.object_view(card).zone, Zone::Battlefield);
        assert!(game.object_view(card).tapped);
    }
}

#[test]
fn seismic_sense_uses_the_live_land_count_and_only_reveals_its_pick() {
    let player = Player::P1;
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Seismic Sense", Zone::Hand);
    put(&mut game, player, "Forest", Zone::Battlefield);
    let below = put(&mut game, player, "Island", Zone::Library);
    let creature = put(&mut game, player, "Bear Cub", Zone::Library);
    let ineligible = put(&mut game, player, "Shock", Zone::Library);
    cast(&mut game, spell, None);
    put(&mut game, player, "Mountain", Zone::Battlefield);
    reach(&mut game, |g| {
        g.legal_actions()
            .contains(&Action::SelectLibrary { card: creature })
    });
    assert!(game.observe(player.other()).library_choice.is_none());
    reject(&mut game, Action::SelectLibrary { card: ineligible });
    reject(&mut game, Action::SelectLibrary { card: below });
    act(&mut game, Action::SelectLibrary { card: creature });
    if game
        .legal_actions()
        .contains(&Action::FinishLibrarySelection)
    {
        act(&mut game, Action::FinishLibrarySelection);
    }
    settle(&mut game);
    assert_eq!(game.object_view(creature).zone, Zone::Hand);
    assert_eq!(
        game.state.players[player.index()].library.first(),
        Some(&ineligible)
    );
    let public = game.observe(player.other());
    assert!(
        public
            .objects
            .iter()
            .any(|object| object.id == creature && object.zone == Zone::Hand)
    );
    assert!(!public.objects.iter().any(|object| object.id == ineligible));
}
