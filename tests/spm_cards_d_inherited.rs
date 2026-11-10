//! Printed interaction checks for the already admitted cards in the final SPM cohort.
mod support;

use rustmage as _;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn activate(game: &mut Game, card: CardId, ability: u16, target: Option<Target>) {
    act(
        game,
        Action::AnnounceActivate {
            card,
            ability,
            counter_payment: Counters::default(),
        },
    );
    if let Some(target) = target {
        act(game, Action::ChooseTarget { target });
    }
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(game);
    }
}

fn resolve_target(game: &mut Game, target: Target) {
    act(game, Action::ChooseTarget { target });
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(game, Action::FinishTargets);
    }
    settle(game);
}

fn declare(game: &mut Game, source: CardId) {
    game.begin_step(Step::BeginCombat);
    pass_pair(game);
    act(game, Action::Attack { card: source });
    act(game, Action::FinishAttackers);
}

#[test]
fn spider_woman_taps_opposing_creatures_and_artifacts_and_has_flying() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let hero = put(
        &mut game,
        player,
        "Spider-Woman, Stunning Savior",
        Zone::Hand,
    );
    cast(&mut game, hero, None);
    settle(&mut game);
    assert!(game.keywords(hero).contains(Keywords::FLYING));
    let own = put(&mut game, player, "Grizzly Bears", Zone::Hand);
    cast(&mut game, own, None);
    settle(&mut game);
    assert!(!game.object_view(own).tapped);
    game.state.active = opponent;
    game.give_priority();
    for name in ["Grizzly Bears", "Swiftfoot Boots"] {
        let opposing = put(&mut game, opponent, name, Zone::Hand);
        cast(&mut game, opposing, None);
        settle(&mut game);
        assert!(game.object_view(opposing).tapped);
    }
}

#[test]
fn starling_and_taxi_driver_grant_flight_and_haste_through_their_distinct_costs() {
    let player = Player::P0;
    let mut game = scenario(player);
    let ally = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let hero = put(&mut game, player, "Starling, Aerial Ally", Zone::Hand);
    let taxi = put(&mut game, player, "Taxi Driver", Zone::Battlefield);
    cast(&mut game, hero, None);
    pass_pair(&mut game);
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(hero),
        },
    );
    resolve_target(&mut game, Target::Card(ally));
    assert!(game.keywords(hero).contains(Keywords::FLYING));
    activate(&mut game, taxi, 0, Some(Target::Card(ally)));
    settle(&mut game);
    assert!(game.object_view(taxi).tapped);
    let grants = Keywords::FLYING.union(Keywords::HASTE);
    assert!(game.keywords(ally).contains(grants));
    game.begin_step(Step::End);
    game.give_priority();
    pass_pair(&mut game);
    assert!(!game.keywords(ally).intersects(grants));
}

#[test]
fn steel_wrecking_ball_destroys_an_artifact_from_hand_and_deals_five_on_entry() {
    let player = Player::P0;
    let mut game = scenario(player);
    let enemy = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    let equipment = put(
        &mut game,
        player.other(),
        "Swiftfoot Boots",
        Zone::Battlefield,
    );
    let discarded = put(&mut game, player, "Steel Wrecking Ball", Zone::Hand);
    activate(&mut game, discarded, 0, Some(Target::Card(equipment)));
    settle(&mut game);
    assert_eq!(game.object_view(discarded).zone, Zone::Graveyard);
    assert_eq!(game.object_view(equipment).zone, Zone::Graveyard);
    let entered = put(&mut game, player, "Steel Wrecking Ball", Zone::Hand);
    cast(&mut game, entered, None);
    pass_pair(&mut game);
    resolve_target(&mut game, Target::Card(enemy));
    assert_eq!(game.object_view(enemy).zone, Zone::Graveyard);
}

#[test]
fn spm_dual_lands_enter_tapped_offer_their_two_colors_and_pay_to_surveil() {
    let player = Player::P0;
    for (name, second) in [("Suburban Sanctuary", 4), ("University Campus", 1)] {
        let mut game = scenario(player);
        let land = put(&mut game, player, name, Zone::Hand);
        act(&mut game, Action::PlayLand { card: land });
        assert!(game.object_view(land).tapped);
        game.state.objects[land as usize].tapped = false;
        let productions: Vec<_> = game
            .legal_actions()
            .into_iter()
            .filter_map(|action| match action {
                Action::ActivateMana {
                    card, production, ..
                } if card == land => Some(production),
                _ => None,
            })
            .collect();
        let mut other = [0; 6];
        other[second] = 1;
        assert_eq!(productions.len(), 2);
        assert!(productions.contains(&[1, 0, 0, 0, 0, 0]));
        assert!(productions.contains(&other));
        let top = put(&mut game, player, "Shock", Zone::Library);
        activate(&mut game, land, 1, None);
        pass_pair(&mut game);
        choose(
            &mut game,
            |action| matches!(action, Action::ArrangeLibrary { card, destination: LibraryDestination::Graveyard } if *card == top),
        );
        assert!(game.object_view(land).tapped);
        assert_eq!(game.object_view(top).zone, Zone::Graveyard);
    }
}

#[test]
fn sudden_strike_only_destroys_an_attacker_or_blocker() {
    let player = Player::P0;
    let mut game = scenario(player);
    let creature = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let strike = put(&mut game, player, "Sudden Strike", Zone::Hand);
    reject(&mut game, Action::AnnounceCast { card: strike });
    declare(&mut game, creature);
    game.state.players[player.index()].mana = [1, 0, 0, 0, 0, 1];
    cast(&mut game, strike, Some(Target::Card(creature)));
    settle(&mut game);
    assert_eq!(game.object_view(creature).zone, Zone::Graveyard);
}

#[test]
fn the_spots_portal_checks_for_a_villain_after_moving_the_target_to_library_bottom() {
    let player = Player::P0;
    let mut game = scenario(player);
    let villain = put(&mut game, player, "Common Crook", Zone::Battlefield);
    let portal = put(&mut game, player, "The Spot's Portal", Zone::Hand);
    cast(&mut game, portal, Some(Target::Card(villain)));
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].library[0], villain);
    assert_eq!(game.state.players[player.index()].life, 18);
}

#[test]
fn thwip_grows_and_grants_flying_to_any_creature_and_only_a_spider_gains_its_caster_life() {
    let player = Player::P0;
    for (name, life) in [("Treetop Snarespinner", 22), ("Bear Cub", 20)] {
        let mut game = scenario(player);
        let target = put(&mut game, player.other(), name, Zone::Battlefield);
        let spell = put(&mut game, player, "Thwip!", Zone::Hand);
        let before = stats(&game, target);
        cast(&mut game, spell, Some(Target::Card(target)));
        settle(&mut game);
        assert_eq!(stats(&game, target), (before.0 + 2, before.1 + 2));
        assert!(game.keywords(target).contains(Keywords::FLYING));
        assert_eq!(game.state.players[player.index()].life, life);
    }
}

#[test]
fn tombstone_returns_its_owners_villain_and_reduces_its_controller_villain_spell_costs() {
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Tombstone, Career Criminal", Zone::Hand);
    let ally = put(&mut game, player, "Common Crook", Zone::Graveyard);
    let enemy = put(&mut game, player.other(), "Common Crook", Zone::Graveyard);
    cast(&mut game, source, None);
    pass_pair(&mut game);
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(enemy),
        },
    );
    resolve_target(&mut game, Target::Card(ally));
    assert_eq!(game.object_view(ally).zone, Zone::Hand);
    game.state.players[player.index()].mana = [0, 0, 1, 0, 0, 0];
    cast(&mut game, ally, None);
    settle(&mut game);
    assert_eq!(game.object_view(ally).zone, Zone::Battlefield);
}

#[test]
fn unstable_experiment_draws_for_the_targeted_player_and_connives_with_the_legal_ally() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Unstable Experiment", Zone::Hand);
    let ally = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let discard = put(&mut game, player, "Shock", Zone::Hand);
    announce(
        &mut game,
        spell,
        &[Target::Player(opponent), Target::Card(ally)],
    );
    pass_pair(&mut game);
    assert_eq!(game.hand_count(opponent), 1);
    act(&mut game, Action::Discard { card: discard });
    settle(&mut game);
    assert_eq!(game.counter_count(ally, CounterKind::PlusOne), 1);
}

#[test]
fn venom_hunger_reduces_the_cost_with_a_villain_and_then_gains_two_life() {
    let player = Player::P0;
    let mut game = scenario(player);
    let enemy = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    let spell = put(&mut game, player, "Venom's Hunger", Zone::Hand);
    game.state.players[player.index()].mana = [0, 0, 1, 0, 0, 2];
    reject(&mut game, Action::AnnounceCast { card: spell });
    put(&mut game, player, "Common Crook", Zone::Battlefield);
    cast(&mut game, spell, Some(Target::Card(enemy)));
    settle(&mut game);
    assert_eq!(game.object_view(enemy).zone, Zone::Graveyard);
    assert_eq!(game.state.players[player.index()].life, 22);
}

#[test]
fn venom_graveyard_activation_and_venomized_cat_keep_their_distinct_zone_effects() {
    let player = Player::P0;
    let mut game = scenario(player);
    let venom = put(&mut game, player, "Venom, Evil Unleashed", Zone::Graveyard);
    let cat = put(&mut game, player, "Venomized Cat", Zone::Hand);
    let top = [
        put(&mut game, player, "Island", Zone::Library),
        put(&mut game, player, "Mountain", Zone::Library),
    ];
    let ally = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    cast(&mut game, cat, None);
    settle(&mut game);
    assert!(game.keywords(cat).contains(Keywords::DEATHTOUCH));
    for id in top {
        assert_eq!(game.object_view(id).zone, Zone::Graveyard);
    }
    activate(&mut game, venom, 0, Some(Target::Card(ally)));
    settle(&mut game);
    assert_eq!(game.object_view(venom).zone, Zone::Exile);
    assert_eq!(game.counter_count(ally, CounterKind::PlusOne), 2);
    assert!(game.keywords(ally).contains(Keywords::DEATHTOUCH));
    game.begin_step(Step::End);
    game.give_priority();
    pass_pair(&mut game);
    assert!(!game.keywords(ally).contains(Keywords::DEATHTOUCH));
}

#[test]
fn vibrant_cityscape_sacrifices_itself_and_fetches_a_tapped_basic_land() {
    let player = Player::P0;
    let mut game = scenario(player);
    let city = put(&mut game, player, "Vibrant Cityscape", Zone::Hand);
    let land = put(&mut game, player, "Swamp", Zone::Library);
    act(&mut game, Action::PlayLand { card: city });
    activate(&mut game, city, 0, None);
    pass_pair(&mut game);
    act(&mut game, Action::ChooseSearch { card: land });
    if game.legal_actions().contains(&Action::FinishSearch) {
        act(&mut game, Action::FinishSearch);
    }
    settle(&mut game);
    assert_eq!(game.object_view(city).zone, Zone::Graveyard);
    assert_eq!(game.object_view(land).zone, Zone::Battlefield);
    assert!(game.object_view(land).tapped);
    game.state.objects[land as usize].tapped = false;
    let before = game.state.players[player.index()].mana[2];
    act(&mut game, Action::TapMana { card: land });
    assert_eq!(game.state.players[player.index()].mana[2], before + 1);
}

#[test]
fn vulture_lifts_only_other_controlled_villains_on_attack() {
    let player = Player::P0;
    let mut game = scenario(player);
    let vulture = put(
        &mut game,
        player,
        "Vulture, Scheming Scavenger",
        Zone::Battlefield,
    );
    let ally = put(&mut game, player, "Common Crook", Zone::Battlefield);
    let enemy = put(&mut game, player.other(), "Common Crook", Zone::Battlefield);
    let nonvillain = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let core = put(&mut game, player, "Firdoch Core", Zone::Battlefield);
    let flyer = put(
        &mut game,
        player.other(),
        "Healer's Hawk",
        Zone::Battlefield,
    );
    let forests: Vec<_> = (0..4)
        .map(|_| put(&mut game, player, "Forest", Zone::Battlefield))
        .collect();
    assert!(game.keywords(vulture).contains(Keywords::FLYING));
    assert!(!game.keywords(ally).contains(Keywords::FLYING));
    assert!(game.has_subtype(core, "Villain"));
    assert!(!game.is_creature(core));
    assert!(!game.keywords(core).contains(Keywords::FLYING));

    declare(&mut game, vulture);
    settle(&mut game);
    assert!(game.keywords(ally).contains(Keywords::FLYING));
    assert!(!game.keywords(enemy).contains(Keywords::FLYING));
    assert!(!game.keywords(nonvillain).contains(Keywords::FLYING));
    assert!(!game.is_creature(core));
    assert!(game.keywords(core).contains(Keywords::FLYING));

    for card in forests {
        act(&mut game, Action::TapMana { card });
    }
    activate(&mut game, core, 1, None);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    assert!(game.is_creature(core));
    assert!(game.keywords(core).contains(Keywords::FLYING));

    // The first combat's resolved grant still applies during another combat.
    let turn = game.state.turn;
    game.add_combat();
    for _ in 0..16 {
        let action = match &game.state.pending {
            Pending::Attackers => break,
            Pending::Blockers => Action::FinishBlockers,
            Pending::Priority => Action::Pass,
            pending => panic!("unexpected decision before the next combat: {pending:?}"),
        };
        act(&mut game, action);
    }
    assert_eq!(game.state.turn, turn);
    act(&mut game, Action::Attack { card: core });
    act(&mut game, Action::FinishAttackers);
    pass_pair(&mut game);
    reject(
        &mut game,
        Action::Block {
            blocker: enemy,
            attacker: core,
        },
    );
    act(
        &mut game,
        Action::Block {
            blocker: flyer,
            attacker: core,
        },
    );
}

#[test]
fn web_up_only_exiles_an_opposing_nonland_and_returns_it_when_the_enchantment_leaves() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let enchantment = put(&mut game, player, "Web Up", Zone::Hand);
    let target = put(&mut game, opponent, "Swiftfoot Boots", Zone::Battlefield);
    let land = put(&mut game, opponent, "Island", Zone::Battlefield);
    cast(&mut game, enchantment, None);
    pass_pair(&mut game);
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(land),
        },
    );
    resolve_target(&mut game, Target::Card(target));
    assert_eq!(game.object_view(target).zone, Zone::Exile);
    game.move_object(enchantment, Zone::Graveyard);
    settle(&mut game);
    assert_eq!(game.object_view(target).zone, Zone::Battlefield);
    assert_eq!(game.object_view(target).controller, opponent);
}

#[test]
fn web_shooters_taps_the_chosen_opponent_creature_from_the_bearers_attack() {
    let player = Player::P0;
    let mut game = scenario(player);
    let equipment = put(&mut game, player, "Web-Shooters", Zone::Battlefield);
    let bearer = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let defender = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    activate(&mut game, equipment, 0, Some(Target::Card(bearer)));
    settle(&mut game);
    assert_eq!(stats(&game, bearer), (3, 3));
    assert!(game.keywords(bearer).contains(Keywords::REACH));
    declare(&mut game, bearer);
    resolve_target(&mut game, Target::Card(defender));
    assert!(game.object_view(defender).tapped);
}

#[test]
fn web_warriors_counters_other_creatures_you_control_only() {
    let player = Player::P0;
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Web-Warriors", Zone::Hand);
    let ally = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let enemy = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    cast(&mut game, spell, None);
    settle(&mut game);
    assert_eq!(game.counter_count(ally, CounterKind::PlusOne), 1);
    assert_eq!(game.counter_count(spell, CounterKind::PlusOne), 0);
    assert_eq!(game.counter_count(enemy, CounterKind::PlusOne), 0);
}

#[test]
fn whoosh_returns_a_nonland_permanent_and_only_a_paid_kicker_draws() {
    let player = Player::P0;
    for kicker in [false, true] {
        let mut game = scenario(player);
        let target = put(
            &mut game,
            player.other(),
            "Swiftfoot Boots",
            Zone::Battlefield,
        );
        let spell = put(&mut game, player, "Whoosh!", Zone::Hand);
        act(&mut game, Action::AnnounceCast { card: spell });
        if matches!(game.state.pending, Pending::Kicker { .. }) {
            act(&mut game, Action::ChooseKicker { pay: kicker });
        }
        act(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        if matches!(game.state.pending, Pending::Kicker { .. }) {
            act(&mut game, Action::ChooseKicker { pay: kicker });
        }
        pay(&mut game);
        settle(&mut game);
        assert_eq!(game.object_view(target).zone, Zone::Hand);
        assert_eq!(game.hand_count(player), usize::from(kicker));
    }
}
