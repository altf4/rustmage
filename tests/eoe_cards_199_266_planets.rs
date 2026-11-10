//! Artifacts and lands through their printed activation and trigger routes.
mod support;
use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn activate(card: CardId, ability: u16) -> Action {
    Action::AnnounceActivate {
        card,
        ability,
        counter_payment: Counters::default(),
    }
}

fn select(game: &mut Game, card: CardId) {
    act(
        game,
        Action::ChooseTarget {
            target: Target::Card(card),
        },
    );
}

fn passes(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn order_trigger(game: &mut Game) {
    if matches!(game.state.pending, Pending::OrderTriggers { .. }) {
        choose(game, |action| matches!(action, Action::PutTrigger { .. }));
    }
}

fn robot(game: &Game, player: Player) -> ObjectView {
    game.observe(player)
        .objects
        .into_iter()
        .find(|o| {
            o.zone == Zone::Battlefield
                && o.controller == player
                && o.is_token()
                && o.has_subtype("Robot")
        })
        .expect("Robot token")
}

fn mana(card: CardId, ability: u16, payment: Mana, production: Mana) -> Action {
    Action::ActivateMana {
        card,
        ability,
        payment,
        production,
        counter_payment: Counters::default(),
    }
}

#[test]
fn pinnacle_kill_ship_can_decline_ten_damage_and_animates_only_at_seven() {
    let player = Player::P0;
    let mut game = scenario(player);
    let ship = put(&mut game, player, "Pinnacle Kill-Ship", Zone::Hand);
    let victim = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    cast(&mut game, ship, None);
    passes(&mut game);
    order_trigger(&mut game);
    let mut declined = game.clone();
    act(&mut declined, Action::FinishTargets);
    settle(&mut declined);
    assert_eq!(declined.object_view(victim).zone, Zone::Battlefield);
    select(&mut game, victim);
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(&mut game, Action::FinishTargets);
    }
    settle(&mut game);
    assert_eq!(game.object_view(victim).zone, Zone::Graveyard);
    game.add_counter(ship, CounterKind::Charge, 6);
    assert!(!game.object_view(ship).is_creature());
    game.add_counter(ship, CounterKind::Charge, 1);
    assert_eq!(stats(&game, ship), (7, 7));
    assert!(game.object_view(ship).keywords.contains(Keywords::FLYING));
}

#[test]
fn wurmwall_sweeper_surveils_two_privately_then_stations_using_the_paid_creature() {
    let player = Player::P0;
    let mut game = scenario(player);
    let ship = put(&mut game, player, "Wurmwall Sweeper", Zone::Hand);
    let donor = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    game.add_counter(donor, CounterKind::PlusOne, 2);
    let keep = put(&mut game, player, "Plains", Zone::Library);
    let mill = put(&mut game, player, "Island", Zone::Library);
    cast(&mut game, ship, None);
    passes(&mut game);
    order_trigger(&mut game);
    passes(&mut game);
    assert!(game.observe(player.other()).library_choice.is_none());
    for (card, destination) in [
        (mill, LibraryDestination::Graveyard),
        (keep, LibraryDestination::Top),
    ] {
        act(&mut game, Action::ArrangeLibrary { card, destination });
    }
    assert_eq!(game.object_view(mill).zone, Zone::Graveyard);
    assert_eq!(
        game.state.players[player.index()].library.last(),
        Some(&keep)
    );
    act(&mut game, activate(ship, 0));
    let object = game.object_ref(donor);
    act(&mut game, Action::ChooseAdditionalCost { object });
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.counter_count(ship, CounterKind::Charge), 4);
    assert_eq!(stats(&game, ship), (2, 2));
    assert!(game.object_view(ship).keywords.contains(Keywords::FLYING));
}

#[test]
fn survey_mechan_locks_distinct_land_names_and_accepts_the_same_player_in_both_groups() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let source = put(&mut game, player, "Survey Mechan", Zone::Battlefield);
    for name in ["Forest", "Forest", "Mountain"] {
        put(&mut game, player, name, Zone::Battlefield);
    }
    put(&mut game, opponent, "Plains", Zone::Battlefield);
    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 8];
    assert!(
        game.object_view(source)
            .keywords
            .contains(Keywords::HEXPROOF.union(Keywords::FLYING))
    );
    let library = game.state.players[opponent.index()].library.len();
    act(&mut game, activate(source, 0));
    for _ in 0..2 {
        act(
            &mut game,
            Action::ChooseTarget {
                target: Target::Player(opponent),
            },
        );
    }
    assert_eq!(game.observe(player).payment_request.unwrap().generic, 8);
    act(
        &mut game,
        Action::PayCosts {
            payment: [0, 0, 0, 0, 0, 8],
        },
    );
    assert_eq!(game.object_view(source).zone, Zone::Graveyard);
    settle(&mut game);
    assert_eq!(
        game.state.players[opponent.index()].library.len(),
        library - 3
    );
    assert_eq!(game.state.players[opponent.index()].life, 20);
}

#[test]
fn thaumaton_torpedo_uses_a_spacecraft_declaration_after_the_attacker_leaves() {
    let player = Player::P0;
    let mut game = scenario(player);
    let torpedo = put(&mut game, player, "Thaumaton Torpedo", Zone::Battlefield);
    let spacecraft = put(&mut game, player, "Warmaker Gunship", Zone::Battlefield);
    let victim = put(
        &mut game,
        player.other(),
        "Swiftfoot Boots",
        Zone::Battlefield,
    );
    game.add_counter(spacecraft, CounterKind::Charge, 6);
    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 3];
    reject(&mut game, activate(torpedo, 0));
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    act(&mut game, Action::Attack { card: spacecraft });
    act(&mut game, Action::FinishAttackers);
    game.move_object(spacecraft, Zone::Exile);
    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 3];
    act(&mut game, activate(torpedo, 0));
    select(&mut game, victim);
    assert_eq!(game.observe(player).payment_request.unwrap().generic, 3);
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(torpedo).zone, Zone::Graveyard);
    assert_eq!(game.object_view(victim).zone, Zone::Graveyard);
}

#[test]
fn adagia_enters_tapped_and_copies_a_controlled_artifact_with_a_legendary_exception() {
    let player = Player::P0;
    let mut game = scenario(player);
    let planet = put(&mut game, player, "Adagia, Windswept Bastion", Zone::Hand);
    let target = put(&mut game, player, "Swiftfoot Boots", Zone::Battlefield);
    let foreign = put(
        &mut game,
        player.other(),
        "Swiftfoot Boots",
        Zone::Battlefield,
    );
    game.add_counter(target, CounterKind::Charge, 3);
    act(&mut game, Action::PlayLand { card: planet });
    assert!(game.object_view(planet).tapped);
    game.state.objects[planet as usize].tapped = false;
    let mut regular = game.clone();
    let before = regular.state.players[player.index()].mana[0];
    act(&mut regular, mana(planet, 0, [0; 6], [1, 0, 0, 0, 0, 0]));
    assert_eq!(regular.state.players[player.index()].mana[0], before + 1);
    game.add_counter(planet, CounterKind::Charge, 11);
    reject(&mut game, activate(planet, 2));
    game.add_counter(planet, CounterKind::Charge, 1);
    act(&mut game, activate(planet, 2));
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(foreign),
        },
    );
    select(&mut game, target);
    pay(&mut game);
    settle(&mut game);
    let copy = game
        .observe(player)
        .objects
        .into_iter()
        .find(|o| o.zone == Zone::Battlefield && o.is_token() && o.name == "Swiftfoot Boots")
        .expect("legendary Equipment token");
    assert!(copy.is_legendary() && copy.has_subtype("Equipment"));
    assert_eq!(copy.counters.get(CounterKind::Charge), 0);
    assert!(!game.object_view(target).is_legendary());
}

#[test]
fn command_bridge_accepts_a_summoning_sick_permanent_and_sacrifices_if_declined() {
    let player = Player::P0;
    let mut game = scenario(player);
    let bridge = put(&mut game, player, "Command Bridge", Zone::Hand);
    let creature = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    game.state.objects[creature as usize].summoning_sick = true;
    act(&mut game, Action::PlayLand { card: bridge });
    assert!(game.object_view(bridge).tapped);
    order_trigger(&mut game);
    passes(&mut game);
    let mut declined = game.clone();
    act(&mut declined, Action::DeclineResolutionMana);
    assert_eq!(declined.object_view(bridge).zone, Zone::Graveyard);
    let object = game.object_ref(creature);
    let tapped = game.object_ref(bridge);
    reject(&mut game, Action::ChooseAdditionalCost { object: tapped });
    act(&mut game, Action::ChooseAdditionalCost { object });
    act(&mut game, Action::PayResolutionMana { payment: [0; 6] });
    assert_eq!(game.object_view(bridge).zone, Zone::Battlefield);
    assert!(game.object_view(creature).tapped);
    game.state.objects[bridge as usize].tapped = false;
    let before = game.state.players[player.index()].mana[3];
    act(&mut game, mana(bridge, 0, [0; 6], [0, 0, 0, 1, 0, 0]));
    assert_eq!(game.state.players[player.index()].mana[3], before + 1);
}

fn scaled_planet(name: &str, color: usize, witnesses: [&str; 2]) {
    let player = Player::P0;
    let mut game = scenario(player);
    let planet = put(&mut game, player, name, Zone::Hand);
    for witness in witnesses {
        put(&mut game, player, witness, Zone::Battlefield);
        put(&mut game, player.other(), witness, Zone::Battlefield);
    }
    act(&mut game, Action::PlayLand { card: planet });
    assert!(game.object_view(planet).tapped);
    game.state.objects[planet as usize].tapped = false;
    let mut one = [0; 6];
    one[color] = 1;
    let mut two = [0; 6];
    two[color] = 2;
    game.state.players[player.index()].mana = one;
    let mut base = game.clone();
    act(&mut base, mana(planet, 0, [0; 6], one));
    assert_eq!(base.state.players[player.index()].mana, two);
    game.add_counter(planet, CounterKind::Charge, 11);
    reject(&mut game, mana(planet, 2, one, two));
    game.add_counter(planet, CounterKind::Charge, 1);
    act(&mut game, mana(planet, 2, one, two));
    assert_eq!(game.state.players[player.index()].mana, two);
    assert!(game.object_view(planet).tapped);
}

#[test]
fn evendo_can_pay_green_for_green_per_controlled_creature_at_twelve() {
    scaled_planet("Evendo, Waking Haven", 4, ["Grizzly Bears", "Hill Giant"]);
}

#[test]
fn uthros_can_pay_blue_for_blue_per_controlled_artifact_at_twelve() {
    scaled_planet(
        "Uthros, Titanic Godcore",
        1,
        ["Nutrient Block", "Swiftfoot Boots"],
    );
}

#[test]
fn kavaron_can_sacrifice_itself_and_grants_haste_and_power_to_its_new_robot() {
    let player = Player::P0;
    let mut game = scenario(player);
    let planet = put(
        &mut game,
        player,
        "Kavaron, Memorial World",
        Zone::Battlefield,
    );
    let ally = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let enemy = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    game.add_counter(planet, CounterKind::Charge, 12);
    act(&mut game, activate(planet, 2));
    let object = game.object_ref(planet);
    act(&mut game, Action::ChooseAdditionalCost { object });
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(planet).zone, Zone::Graveyard);
    let robot = robot(&game, player);
    assert_eq!((robot.power, robot.toughness), (3, 2));
    assert!(robot.keywords.contains(Keywords::HASTE));
    assert_eq!(stats(&game, ally), (3, 2));
    assert_eq!(stats(&game, enemy), (2, 2));
}

#[test]
fn starforge_pays_one_tap_plus_two_distinct_artifacts_and_can_make_a_robot() {
    let player = Player::P0;
    let mut game = scenario(player);
    let land = put(&mut game, player, "Secluded Starforge", Zone::Battlefield);
    let target = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    let first = put(&mut game, player, "Nutrient Block", Zone::Battlefield);
    let second = put(&mut game, player, "Swiftfoot Boots", Zone::Battlefield);
    let mut other = game.clone();
    act(&mut other, activate(land, 2));
    pay(&mut other);
    settle(&mut other);
    let token = robot(&other, player);
    assert_eq!((token.power, token.toughness), (2, 2));
    assert!(other.object_view(land).tapped);

    act(&mut game, activate(land, 1));
    act(&mut game, Action::ChooseX { value: 2 });
    select(&mut game, target);
    pay(&mut game);
    choose(&mut game, |action| {
        matches!(action, Action::ChooseCastCost { .. })
    });
    let first_ref = game.object_ref(first);
    let second_ref = game.object_ref(second);
    act(
        &mut game,
        Action::ChooseAdditionalCost { object: first_ref },
    );
    reject(
        &mut game,
        Action::ChooseAdditionalCost { object: first_ref },
    );
    act(
        &mut game,
        Action::ChooseAdditionalCost { object: second_ref },
    );
    settle(&mut game);
    for card in [land, first, second] {
        assert!(game.object_view(card).tapped);
    }
    assert_eq!(stats(&game, target), (4, 2));
}

#[test]
fn susur_secundi_pays_life_and_draws_the_sacrificed_creatures_paid_power() {
    let player = Player::P0;
    let mut game = scenario(player);
    let altar = put(
        &mut game,
        player,
        "Susur Secundi, Void Altar",
        Zone::Battlefield,
    );
    let offering = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let wrong = put(&mut game, player, "Nutrient Block", Zone::Battlefield);
    game.add_counter(altar, CounterKind::Charge, 12);
    game.add_counter(offering, CounterKind::PlusOne, 1);
    let library = game.state.players[player.index()].library.len();
    act(&mut game, activate(altar, 2));
    let invalid = game.object_ref(wrong);
    let object = game.object_ref(offering);
    reject(&mut game, Action::ChooseAdditionalCost { object: invalid });
    act(&mut game, Action::ChooseAdditionalCost { object });
    pay(&mut game);
    settle(&mut game);
    assert!(game.object_view(altar).tapped);
    assert_eq!(game.object_view(offering).zone, Zone::Graveyard);
    assert_eq!(game.state.players[player.index()].life, 18);
    assert_eq!(
        game.state.players[player.index()].library.len(),
        library - 4
    );
}
