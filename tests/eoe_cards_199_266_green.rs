//! Actual Edge of Eternities definitions through normal priority, combat and cost actions.
mod support;
use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn passes(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn advance(game: &mut Game, until: impl Fn(&Game) -> bool) {
    for _ in 0..48 {
        if until(game) {
            return;
        }
        choose(game, |action| {
            matches!(
                action,
                Action::Pass
                    | Action::FinishAttackers
                    | Action::FinishBlockers
                    | Action::PutTrigger { .. }
            )
        });
    }
    panic!("scenario did not reach the requested state");
}

fn order_trigger(game: &mut Game) {
    if matches!(game.state.pending, Pending::OrderTriggers { .. }) {
        choose(game, |action| matches!(action, Action::PutTrigger { .. }));
    }
}

fn attack(game: &mut Game, attacker: CardId) {
    advance(game, |game| game.state.pending == Pending::Attackers);
    act(game, Action::Attack { card: attacker });
    act(game, Action::FinishAttackers);
    order_trigger(game);
}

fn tokens(game: &Game, player: Player, subtype: &str) -> Vec<ObjectView> {
    game.observe(player)
        .objects
        .into_iter()
        .filter(|object| {
            object.zone == Zone::Battlefield
                && object.controller == player
                && object.is_token()
                && object.has_subtype(subtype)
        })
        .collect()
}

fn station(game: &mut Game, ship: CardId, donor: CardId) {
    act(
        game,
        Action::AnnounceActivate {
            card: ship,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    let object = game.object_ref(donor);
    act(game, Action::ChooseAdditionalCost { object });
    pay(game);
    settle(game);
}

#[test]
fn ouroboroid_samples_power_at_departure_and_grows_only_its_controllers_creatures() {
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Ouroboroid", Zone::Hand);
    let bearer = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let foreign = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    cast(&mut game, source, None);
    settle(&mut game);
    advance(&mut game, |g| !g.state.stack.is_empty());
    game.add_counter(source, CounterKind::PlusOne, 2);
    game.move_object(source, Zone::Graveyard);
    settle(&mut game);
    assert_eq!(game.counter_count(bearer, CounterKind::PlusOne), 3);
    assert_eq!(game.counter_count(foreign, CounterKind::PlusOne), 0);
}

#[test]
fn pull_through_the_weft_selects_independent_graveyard_groups() {
    let player = Player::P0;
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Pull Through the Weft", Zone::Hand);
    let creature = put(&mut game, player, "Grizzly Bears", Zone::Graveyard);
    let artifact = put(&mut game, player, "Nutrient Block", Zone::Graveyard);
    let instant = put(&mut game, player, "Opt", Zone::Graveyard);
    let forest = put(&mut game, player, "Forest", Zone::Graveyard);
    let island = put(&mut game, player, "Island", Zone::Graveyard);
    act(&mut game, Action::AnnounceCast { card: spell });
    for wrong in [instant, forest] {
        reject(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(wrong),
            },
        );
    }
    for legal in [creature, artifact, forest, island] {
        act(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(legal),
            },
        );
    }
    pay(&mut game);
    game.move_object(artifact, Zone::Exile);
    settle(&mut game);
    assert_eq!(game.object_view(creature).zone, Zone::Hand);
    assert_eq!(game.object_view(artifact).zone, Zone::Exile);
    for card in [forest, island] {
        assert_eq!(game.object_view(card).zone, Zone::Battlefield);
        assert!(game.object_view(card).tapped);
    }
}

#[test]
fn seedship_impact_uses_legal_targets_pre_destruction_mana_value() {
    let player = Player::P0;
    let mut game = scenario(player);
    let resistant = put(
        &mut game,
        player.other(),
        "Nutrient Block",
        Zone::Battlefield,
    );
    let costly = put(
        &mut game,
        player.other(),
        "Warmaker Gunship",
        Zone::Battlefield,
    );
    let first = put(&mut game, player, "Seedship Impact", Zone::Hand);
    let second = put(&mut game, player, "Seedship Impact", Zone::Hand);
    cast(&mut game, first, Some(Target::Card(resistant)));
    settle(&mut game);
    assert_eq!(game.object_view(resistant).zone, Zone::Battlefield);
    assert_eq!(tokens(&game, player, "Lander").len(), 1);
    cast(&mut game, second, Some(Target::Card(costly)));
    settle(&mut game);
    assert_eq!(game.object_view(costly).zone, Zone::Graveyard);
    assert_eq!(tokens(&game, player, "Lander").len(), 1);
}

#[test]
fn skystinger_only_grows_on_a_block_of_a_flyer() {
    let player = Player::P0;
    for (attacker, expected) in [("Healer's Hawk", 8), ("Hill Giant", 3)] {
        let mut game = scenario(player.other());
        let blocker = put(&mut game, player, "Skystinger", Zone::Battlefield);
        let attacker = put(&mut game, player.other(), attacker, Zone::Battlefield);
        assert!(game.object_view(blocker).keywords.contains(Keywords::REACH));
        attack(&mut game, attacker);
        advance(&mut game, |g| g.state.pending == Pending::Blockers);
        act(&mut game, Action::Block { blocker, attacker });
        act(&mut game, Action::FinishBlockers);
        order_trigger(&mut game);
        settle(&mut game);
        assert_eq!(stats(&game, blocker), (expected, 3));
    }
}

#[test]
fn sledge_class_seedship_uses_warden_toughness_then_offers_one_private_entrant() {
    let player = Player::P0;
    let mut game = scenario(player);
    let ship = put(
        &mut game,
        player,
        "Sledge-Class Seedship",
        Zone::Battlefield,
    );
    let warden = put(&mut game, player, "Tapestry Warden", Zone::Battlefield);
    let giant = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let passenger = put(&mut game, player, "Grizzly Bears", Zone::Hand);
    assert!(!game.object_view(ship).is_creature());
    station(&mut game, ship, warden);
    assert_eq!(game.counter_count(ship, CounterKind::Charge), 4);
    assert!(!game.object_view(ship).is_creature());
    station(&mut game, ship, giant);
    assert_eq!(game.counter_count(ship, CounterKind::Charge), 7);
    let view = game.object_view(ship);
    assert_eq!(stats(&game, ship), (4, 5));
    assert!(view.is_creature() && view.has_subtype("Spacecraft"));
    assert!(view.keywords.contains(Keywords::FLYING));
    attack(&mut game, ship);
    passes(&mut game);
    assert!(game.observe(player.other()).effect_choice.is_none());
    let object = game.object_ref(passenger);
    act(&mut game, Action::PutHandCreature { object });
    assert_eq!(game.object_view(passenger).zone, Zone::Battlefield);
    assert!(
        !game
            .state
            .attackers
            .iter()
            .any(|attack| attack.card == passenger)
    );
}

#[test]
fn tapestry_warden_has_vigilance_and_assigns_its_higher_toughness() {
    let player = Player::P0;
    let mut game = scenario(player);
    let warden = put(&mut game, player, "Tapestry Warden", Zone::Battlefield);
    attack(&mut game, warden);
    assert!(!game.object_view(warden).tapped);
    advance(&mut game, |g| {
        g.state.players[player.other().index()].life < 20
    });
    assert_eq!(game.state.players[player.other().index()].life, 16);
}

#[test]
fn terrasymbiosis_draws_actual_doubled_counters_and_declining_preserves_use() {
    let player = Player::P0;
    let mut game = scenario(player);
    put(&mut game, player, "Terrasymbiosis", Zone::Battlefield);
    put(&mut game, player, "Loading Zone", Zone::Battlefield);
    let creature = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    for (index, accept) in [false, true, false].into_iter().enumerate() {
        let spell = put(&mut game, player, "Honor", Zone::Hand);
        let before = game.state.players[player.index()].library.len();
        cast(&mut game, spell, Some(Target::Card(creature)));
        passes(&mut game);
        if index < 2 {
            order_trigger(&mut game);
            passes(&mut game);
            act(&mut game, Action::ChooseOptional { accept });
        }
        assert!(game.state.stack.is_empty());
        assert_eq!(
            game.state.players[player.index()].library.len(),
            before - 1 - if accept { 2 } else { 0 }
        );
    }
    assert_eq!(game.counter_count(creature, CounterKind::PlusOne), 6);
}

#[test]
fn alpharael_draws_first_and_allows_only_an_artifact_to_pay_the_alternative() {
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Alpharael, Dreaming Acolyte", Zone::Hand);
    let ordinary = put(&mut game, player, "Island", Zone::Library);
    let artifact = put(&mut game, player, "Nutrient Block", Zone::Library);
    cast(&mut game, source, None);
    passes(&mut game);
    order_trigger(&mut game);
    passes(&mut game);
    assert_eq!(game.object_view(ordinary).zone, Zone::Hand);
    assert_eq!(game.object_view(artifact).zone, Zone::Hand);
    let normal = game.object_ref(ordinary);
    let paid = game.object_ref(artifact);
    reject(&mut game, Action::ChooseAdditionalCost { object: normal });
    let mut declined = game.clone();
    act(&mut declined, Action::DeclineResolutionMana);
    act(&mut declined, Action::Discard { card: ordinary });
    act(&mut declined, Action::Discard { card: artifact });
    assert_eq!(declined.object_view(ordinary).zone, Zone::Graveyard);
    act(&mut game, Action::ChooseAdditionalCost { object: paid });
    act(&mut game, Action::PayResolutionMana { payment: [0; 6] });
    assert_eq!(game.object_view(artifact).zone, Zone::Graveyard);
    assert_eq!(game.object_view(ordinary).zone, Zone::Hand);
    assert!(
        game.object_view(source)
            .keywords
            .contains(Keywords::DEATHTOUCH)
    );
    advance(&mut game, |g| g.state.active != player);
    assert!(
        !game
            .object_view(source)
            .keywords
            .contains(Keywords::DEATHTOUCH)
    );
}

#[test]
fn dyadrine_counts_paid_x_and_removes_one_counter_from_each_of_two_creatures() {
    let player = Player::P0;
    let mut game = scenario(player);
    let dyadrine = put(&mut game, player, "Dyadrine, Synthesis Amalgam", Zone::Hand);
    let other = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    game.add_counter(other, CounterKind::PlusOne, 1);
    act(&mut game, Action::AnnounceCast { card: dyadrine });
    act(&mut game, Action::ChooseX { value: 2 });
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.counter_count(dyadrine, CounterKind::PlusOne), 4);
    let library = game.state.players[player.index()].library.len();
    attack(&mut game, other);
    passes(&mut game);
    let first = game.object_ref(dyadrine);
    let second = game.object_ref(other);
    act(&mut game, Action::ChooseAdditionalCost { object: first });
    reject(&mut game, Action::ChooseAdditionalCost { object: first });
    act(&mut game, Action::ChooseAdditionalCost { object: second });
    act(&mut game, Action::PayResolutionMana { payment: [0; 6] });
    assert_eq!(game.counter_count(dyadrine, CounterKind::PlusOne), 3);
    assert_eq!(game.counter_count(other, CounterKind::PlusOne), 0);
    assert_eq!(
        game.state.players[player.index()].library.len(),
        library - 1
    );
    assert_eq!(tokens(&game, player, "Robot").len(), 1);
}

#[test]
fn infinite_guideline_station_counts_multicolored_permanents_at_each_resolution() {
    let player = Player::P0;
    let mut game = scenario(player);
    let ship = put(&mut game, player, "Infinite Guideline Station", Zone::Hand);
    let enchantment = put(
        &mut game,
        player,
        "Call the Spirit Dragons",
        Zone::Battlefield,
    );
    put(
        &mut game,
        player.other(),
        "Biomechan Engineer",
        Zone::Battlefield,
    );
    cast(&mut game, ship, None);
    settle(&mut game);
    let created = tokens(&game, player, "Robot");
    assert_eq!(created.len(), 2);
    assert!(created.iter().all(|object| object.tapped));
    game.add_counter(ship, CounterKind::Charge, 12);
    game.state.objects[ship as usize].summoning_sick = false;
    assert_eq!(stats(&game, ship), (7, 15));
    assert!(game.object_view(ship).keywords.contains(Keywords::FLYING));
    let library = game.state.players[player.index()].library.len();
    attack(&mut game, ship);
    game.move_object(enchantment, Zone::Graveyard);
    settle(&mut game);
    assert_eq!(
        game.state.players[player.index()].library.len(),
        library - 1
    );
}

#[test]
fn interceptor_mechan_returns_an_artifact_and_uses_opponents_departure_for_void() {
    let player = Player::P0;
    let mut game = scenario(player);
    let artifact = put(&mut game, player, "Nutrient Block", Zone::Graveyard);
    let opponent = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    let mechan = put(&mut game, player, "Interceptor Mechan", Zone::Hand);
    cast(&mut game, mechan, None);
    passes(&mut game);
    order_trigger(&mut game);
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(artifact),
        },
    );
    settle(&mut game);
    assert_eq!(game.object_view(artifact).zone, Zone::Hand);
    assert!(game.object_view(mechan).keywords.contains(Keywords::FLYING));
    let mut quiet = game.clone();
    game.move_object(opponent, Zone::Exile);
    for (branch, expected) in [(&mut quiet, 0), (&mut game, 1)] {
        let turn = branch.state.turn;
        advance(branch, |g| g.state.turn > turn);
        assert_eq!(branch.counter_count(mechan, CounterKind::PlusOne), expected);
    }
}

#[test]
fn pinnacle_emissary_accepts_either_warp_color_and_triggers_on_artifact_cast() {
    let player = Player::P0;
    for color in [1, 3] {
        let mut game = scenario(player);
        let source = put(&mut game, player, "Pinnacle Emissary", Zone::Hand);
        game.state.players[player.index()].mana = [0; 6];
        game.state.players[player.index()].mana[color] = 1;
        act(&mut game, Action::AnnounceWarp { card: source });
        choose(&mut game, |action| {
            matches!(action, Action::ChooseSpellCost { .. })
        });
        pay(&mut game);
        assert_eq!(
            game.state.stack.last().unwrap().cast.unwrap().payment[color],
            1
        );
        settle(&mut game);
        assert!(game.object_view(source).cast_for_warp);
        game.state.players[player.index()].mana[5] = 1;
        let artifact = put(&mut game, player, "Nutrient Block", Zone::Hand);
        cast(&mut game, artifact, None);
        settle(&mut game);
        let drones = tokens(&game, player, "Drone");
        assert_eq!(drones.len(), 1);
        assert_eq!((drones[0].power, drones[0].toughness), (1, 1));
        assert!(drones[0].keywords.contains(Keywords::FLYING));
    }
}

#[test]
fn station_monitor_creates_one_drone_on_second_cast() {
    let player = Player::P0;
    let mut game = scenario(player);
    put(&mut game, player, "Station Monitor", Zone::Battlefield);
    for expected in [0, 1, 1] {
        let artifact = put(&mut game, player, "Nutrient Block", Zone::Hand);
        cast(&mut game, artifact, None);
        settle(&mut game);
        assert_eq!(tokens(&game, player, "Drone").len(), expected);
    }
}
