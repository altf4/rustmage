//! Paid entry, attack, death, and linked shuffle scenarios for inherited creatures.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

const PLAYER: Player = Player::P0;
const OTHER: Player = Player::P1;

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

fn attack(game: &mut Game, attacker: CardId) {
    game.begin_step(Step::BeginCombat);
    pass_pair(game);
    act(game, Action::Attack { card: attacker });
    act(game, Action::FinishAttackers);
}

#[test]
fn fear_of_failed_tests_draws_for_its_actual_damage_to_a_player() {
    let mut game = scenario(PLAYER);
    let fear = put(&mut game, PLAYER, "Fear of Failed Tests", Zone::Battlefield);
    let growth = put(&mut game, PLAYER, "Giant Growth", Zone::Hand);
    cast(&mut game, growth, Some(Target::Card(fear)));
    settle(&mut game);
    attack(&mut game, fear);
    pass_pair(&mut game);
    act(&mut game, Action::FinishBlockers);
    pass_pair(&mut game);
    assert_eq!(game.state.players[OTHER.index()].life, 15);
    settle(&mut game);
    assert_eq!(game.state.players[PLAYER.index()].cards_drawn_this_turn, 5);
}

#[test]
fn fear_of_immobility_can_decline_or_tap_either_creature_and_stuns_only_the_opponents() {
    let mut base = scenario(PLAYER);
    let own = put(&mut base, PLAYER, "Grizzly Bears", Zone::Battlefield);
    let enemy = put(&mut base, OTHER, "Hill Giant", Zone::Battlefield);
    let fear = put(&mut base, PLAYER, "Fear of Immobility", Zone::Hand);
    cast(&mut base, fear, None);
    pass_pair(&mut base);

    let mut no_target = base.clone();
    act(&mut no_target, Action::FinishTargets);
    settle(&mut no_target);
    assert!(!no_target.object_view(own).tapped && !no_target.object_view(enemy).tapped);
    for (card, stun) in [(own, 0), (enemy, 1)] {
        let mut game = base.clone();
        act(&mut game, target(card));
        settle(&mut game);
        assert!(game.object_view(card).tapped);
        assert_eq!(game.counter_count(card, CounterKind::Stun), stun);
    }
}

#[test]
fn fear_of_surveillance_attacks_without_tapping_and_surveils_privately() {
    let mut game = scenario(PLAYER);
    let fear = put(&mut game, PLAYER, "Fear of Surveillance", Zone::Battlefield);
    let top = put(&mut game, PLAYER, "Shock", Zone::Library);
    attack(&mut game, fear);
    assert!(!game.object_view(fear).tapped);
    pass_pair(&mut game);
    assert!(game.observe(PLAYER).library_choice.unwrap().surveil);
    assert!(game.observe(OTHER).library_choice.is_none());
    act(
        &mut game,
        Action::ArrangeLibrary {
            card: top,
            destination: LibraryDestination::Graveyard,
        },
    );
    assert_eq!(game.object_view(top).zone, Zone::Graveyard);
}

#[test]
fn floodpits_drowner_flashes_stuns_and_shuffles_the_exact_cards_into_their_owners_libraries() {
    let mut game = scenario(OTHER);
    let drowner = put(&mut game, PLAYER, "Floodpits Drowner", Zone::Hand);
    let enemy = put(&mut game, OTHER, "Hill Giant", Zone::Battlefield);
    let own = put(&mut game, PLAYER, "Grizzly Bears", Zone::Battlefield);
    act(&mut game, Action::Pass);
    cast(&mut game, drowner, None);
    pass_pair(&mut game);
    reject(&mut game, target(own));
    act(&mut game, target(enemy));
    settle(&mut game);
    assert!(game.object_view(enemy).tapped);
    assert_eq!(game.counter_count(enemy, CounterKind::Stun), 1);
    assert!(game.keywords(drowner).contains(Keywords::VIGILANCE));
    let ability = Action::AnnounceActivate {
        card: drowner,
        ability: 0,
        counter_payment: Counters::default(),
    };
    if game.actor() == Some(OTHER) {
        act(&mut game, Action::Pass);
    }
    reject(&mut game, ability.clone());
    game.state.objects[drowner as usize].summoning_sick = false;
    game.change_controller(game.object_ref(enemy), PLAYER);
    act(&mut game, ability);
    reject(&mut game, target(own));
    act(&mut game, target(enemy));
    pay(&mut game);
    assert!(game.object_view(drowner).tapped);
    settle(&mut game);
    for (owner, card) in [(PLAYER, drowner), (OTHER, enemy)] {
        assert_eq!(game.object_view(card).zone, Zone::Library);
        assert!(game.state.players[owner.index()].library.contains(&card));
    }
}

#[test]
fn friendly_ghost_and_hardened_escort_grant_distinct_targeted_combat_buffs() {
    let mut game = scenario(PLAYER);
    let ghost = put(&mut game, PLAYER, "Friendly Ghost", Zone::Hand);
    let escort = put(&mut game, PLAYER, "Hardened Escort", Zone::Battlefield);
    let own = put(&mut game, PLAYER, "Grizzly Bears", Zone::Battlefield);
    let enemy = put(&mut game, OTHER, "Hill Giant", Zone::Battlefield);
    cast(&mut game, ghost, None);
    pass_pair(&mut game);
    act(&mut game, target(enemy));
    settle(&mut game);
    assert!(game.keywords(ghost).contains(Keywords::FLYING));
    assert_eq!(stats(&game, enemy), (5, 7));

    attack(&mut game, escort);
    reject(&mut game, target(escort));
    reject(&mut game, target(enemy));
    act(&mut game, target(own));
    settle(&mut game);
    assert_eq!(stats(&game, own), (3, 2));
    assert!(game.keywords(own).contains(Keywords::INDESTRUCTIBLE));
}

#[test]
fn lionheart_glimmer_grows_its_whole_team_on_attack_and_requires_ward_payment() {
    let mut game = scenario(PLAYER);
    let glimmer = put(&mut game, PLAYER, "Lionheart Glimmer", Zone::Battlefield);
    let attacker = put(&mut game, PLAYER, "Grizzly Bears", Zone::Battlefield);
    let reserve = put(&mut game, PLAYER, "Bear Cub", Zone::Battlefield);
    let enemy = put(&mut game, OTHER, "Hill Giant", Zone::Battlefield);
    attack(&mut game, attacker);
    settle(&mut game);
    assert_eq!(stats(&game, attacker), (3, 3));
    assert_eq!(stats(&game, reserve), (3, 3));
    assert_eq!(stats(&game, glimmer), (3, 6));
    assert_eq!(stats(&game, enemy), (3, 3));

    let removal = put(&mut game, OTHER, "Murder", Zone::Hand);
    game.state.players[OTHER.index()].mana = [8; 6];
    act(&mut game, Action::Pass);
    cast(&mut game, removal, Some(Target::Card(glimmer)));
    pass_pair(&mut game);
    let mut decline = game.clone();
    act(&mut decline, Action::DeclineResolutionMana);
    settle(&mut decline);
    assert_eq!(decline.object_view(glimmer).zone, Zone::Battlefield);

    choose(&mut game, |action| {
        matches!(action, Action::PayResolutionMana { .. })
    });
    settle(&mut game);
    assert_eq!(game.object_view(glimmer).zone, Zone::Graveyard);
}

#[test]
fn living_phone_privately_looks_at_five_and_reveals_only_the_chosen_small_creature() {
    let mut game = scenario(PLAYER);
    let phone = put(&mut game, PLAYER, "Living Phone", Zone::Battlefield);
    let outside = put(&mut game, PLAYER, "Piranha Fly", Zone::Library);
    let cards = [
        "Grizzly Bears",
        "Hill Giant",
        "Island",
        "Shock",
        "Llanowar Elves",
    ]
    .map(|name| put(&mut game, PLAYER, name, Zone::Library));
    let removal = put(&mut game, PLAYER, "Murder", Zone::Hand);
    cast(&mut game, removal, Some(Target::Card(phone)));
    pass_pair(&mut game);
    pass_pair(&mut game);
    let view = game.observe(PLAYER).library_selection.unwrap();
    assert_eq!(view.remaining.len(), 5);
    assert!(game.observe(OTHER).library_selection.is_none());
    for invalid in [outside, cards[1], cards[2]] {
        reject(&mut game, Action::SelectLibrary { card: invalid });
    }

    let mut decline = game.clone();
    act(&mut decline, Action::FinishLibrarySelection);
    assert!(
        cards
            .into_iter()
            .all(|card| decline.object_view(card).zone == Zone::Library)
    );
    assert_eq!(
        decline.state.players[PLAYER.index()].library.last(),
        Some(&outside)
    );

    act(&mut game, Action::SelectLibrary { card: cards[0] });
    if game
        .legal_actions()
        .contains(&Action::FinishLibrarySelection)
    {
        act(&mut game, Action::FinishLibrarySelection);
    }
    settle(&mut game);
    assert_eq!(game.object_view(cards[0]).zone, Zone::Hand);
    assert!(
        game.observe(OTHER)
            .known_hand
            .contains(&game.object_ref(cards[0]))
    );
    assert_eq!(
        game.state.players[PLAYER.index()].library.last(),
        Some(&outside)
    );
    let bottom = &game.state.players[PLAYER.index()].library[..4];
    assert!(cards[1..].iter().all(|card| bottom.contains(card)));
}

#[test]
fn piranha_fly_is_a_flying_creature_that_enters_tapped() {
    let mut game = scenario(PLAYER);
    let fly = put(&mut game, PLAYER, "Piranha Fly", Zone::Hand);
    cast(&mut game, fly, None);
    settle(&mut game);
    assert!(game.object_view(fly).tapped);
    assert!(game.keywords(fly).contains(Keywords::FLYING));
    assert_eq!(stats(&game, fly), (2, 1));
}
