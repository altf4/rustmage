//! Outlaws of Thunder Junction card scenarios, H through J.
#[path = "support/otj_e_l.rs"]
mod otj;
mod support;

use otj::*;
use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn tap_for(game: &mut Game, card: CardId, ability: u16, production: Mana) {
    act(
        game,
        Action::ActivateMana {
            card,
            ability,
            payment: [0; 6],
            counter_payment: Counters::default(),
            production,
        },
    );
}

#[test]
fn hardbristle_bandit_produces_a_color_and_untaps_only_on_the_first_crime() {
    let player = Player::P1;
    let mut game = scenario(player);
    let bandit = put(&mut game, player, "Hardbristle Bandit", Zone::Battlefield);
    let white = [1, 0, 0, 0, 0, 0];
    let colorless = Action::ActivateMana {
        card: bandit,
        ability: 0,
        payment: [0; 6],
        counter_payment: Counters::default(),
        production: [0, 0, 0, 0, 0, 1],
    };
    reject(&mut game, colorless);
    for crime in 0..2 {
        tap_for(&mut game, bandit, 0, white);
        assert!(game.object_view(bandit).tapped);
        let shock = put(&mut game, player, "Shock", Zone::Hand);
        cast(&mut game, shock, Some(Target::Player(player.other())));
        resolve(&mut game);
        assert_eq!(game.object_view(bandit).tapped, crime != 0);
    }
}

#[test]
fn hell_to_pay_creates_tapped_treasures_for_excess_over_remaining_lethal() {
    let player = Player::P0;
    let mut game = scenario(player);
    let creature = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    let shock = put(&mut game, player, "Shock", Zone::Hand);
    cast(&mut game, shock, Some(Target::Card(creature)));
    resolve(&mut game);
    let hell = put(&mut game, player, "Hell to Pay", Zone::Hand);
    let mut zero = game.clone();
    cast_x(&mut zero, hell, 0, &[Target::Card(creature)]);
    resolve(&mut zero);
    assert!(tokens(&zero, player, TREASURE).is_empty());

    cast_x(&mut game, hell, 4, &[Target::Card(creature)]);
    resolve(&mut game);
    assert_eq!(game.object_view(creature).zone, Zone::Graveyard);
    let found = tokens(&game, player, TREASURE);
    assert_eq!(found.len(), 3);
    assert!(found.into_iter().all(|id| game.object_view(id).tapped));
}

#[test]
fn hellspur_posse_boss_gives_haste_to_other_own_outlaws_and_both_new_mercenaries() {
    let player = Player::P0;
    let mut game = scenario(player);
    let boss = put(&mut game, player, "Hellspur Posse Boss", Zone::Hand);
    let outlaw = put(&mut game, player, "Forsaken Miner", Zone::Battlefield);
    let opposing = put(
        &mut game,
        player.other(),
        "Forsaken Miner",
        Zone::Battlefield,
    );
    let ordinary = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    cast(&mut game, boss, None);
    resolve(&mut game);
    let posse = tokens(&game, player, MERCENARY);
    assert_eq!(posse.len(), 2);
    for subject in posse.into_iter().chain([outlaw]) {
        assert!(game.keywords(subject).contains(Keywords::HASTE));
    }
    for subject in [boss, opposing, ordinary] {
        assert!(!game.keywords(subject).contains(Keywords::HASTE));
    }
    game.move_object(boss, Zone::Graveyard);
    assert!(!game.keywords(outlaw).contains(Keywords::HASTE));
}

#[test]
fn high_noon_limits_committed_casts_and_sacrifices_before_dealing_damage() {
    let player = Player::P0;
    let mut game = scenario(player);
    let noon = put(&mut game, player, "High Noon", Zone::Battlefield);
    let first = put(&mut game, player, "Shock", Zone::Hand);
    let second = put(&mut game, player, "Shock", Zone::Hand);
    let target = Target::Player(player.other());
    cast(&mut game, first, Some(target));
    resolve(&mut game);
    reject(&mut game, Action::AnnounceCast { card: second });
    announce_activation(&mut game, noon, 0);
    choose_targets(&mut game, &[target]);
    pay(&mut game);
    assert_eq!(game.object_view(noon).zone, Zone::Graveyard);
    assert!(
        game.legal_actions()
            .contains(&Action::AnnounceCast { card: second })
    );
    resolve(&mut game);
    assert_eq!(game.state.players[player.other().index()].life, 13);
}

#[test]
fn highway_robbery_draws_only_after_discarding_or_sacrificing_a_land() {
    let player = Player::P1;
    let mut game = scenario(player);
    let land = put(&mut game, player, "Mountain", Zone::Battlefield);
    let creature = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let discard = put(&mut game, player, "Shock", Zone::Hand);
    let spell = put(&mut game, player, "Highway Robbery", Zone::Hand);
    let exact_land = game.object_ref(land);
    let exact_creature = game.object_ref(creature);
    let length = game.state.players[player.index()].library.len();

    let mut plotted = game.clone();
    act(&mut plotted, Action::AnnouncePlot { card: spell });
    choose(&mut plotted, |action| {
        matches!(action, Action::PayPlot { .. })
    });
    assert_eq!(plotted.object_view(spell).zone, Zone::Exile);
    assert_eq!(plotted.state.players[player.index()].library.len(), length);

    cast(&mut game, spell, None);
    checkpoint(&mut game, Action::DeclineEffectPayment);
    assert!(game.observe(player.other()).effect_choice.is_none());
    reject(
        &mut game,
        Action::Sacrifice {
            object: exact_creature,
        },
    );
    let mut declined = game.clone();
    act(&mut declined, Action::DeclineEffectPayment);
    assert_eq!(declined.state.players[player.index()].library.len(), length);
    let mut discarded = game.clone();
    act(&mut discarded, Action::Discard { card: discard });
    resolve(&mut discarded);
    assert_eq!(discarded.object_view(discard).zone, Zone::Graveyard);
    assert_eq!(
        discarded.state.players[player.index()].library.len(),
        length - 2
    );
    apply_raw_and_index(&mut game, Action::Sacrifice { object: exact_land });
    resolve(&mut game);
    assert_eq!(game.object_view(land).zone, Zone::Graveyard);
    assert_eq!(game.state.players[player.index()].library.len(), length - 2);
}

#[test]
fn hollow_marauder_reduces_cost_by_own_graveyard_creatures_and_checks_discard_value() {
    let player = Player::P0;
    for (discard_name, draws) in [("Shock", true), ("Hill Giant", false)] {
        let mut game = scenario(player);
        put(&mut game, player, "Grizzly Bears", Zone::Graveyard);
        put(&mut game, player, "Hill Giant", Zone::Graveyard);
        put(&mut game, player, "Shock", Zone::Graveyard);
        put(&mut game, player.other(), "Hill Giant", Zone::Graveyard);
        let discard = put(&mut game, player.other(), discard_name, Zone::Hand);
        let marauder = put(&mut game, player, "Hollow Marauder", Zone::Hand);
        game.state.players[player.index()].mana = [0, 0, 1, 0, 0, 3];
        reject(&mut game, Action::AnnounceCast { card: marauder });
        game.state.players[player.index()].mana[5] += 1;
        cast(&mut game, marauder, None);
        choose_when_ready(
            &mut game,
            Action::ChooseTarget {
                target: Target::Player(player.other()),
            },
        );
        if game.legal_actions().contains(&Action::FinishTargets) {
            act(&mut game, Action::FinishTargets);
        }
        let length = game.state.players[player.index()].library.len();
        choose_when_ready(&mut game, Action::Discard { card: discard });
        resolve(&mut game);
        assert_eq!(game.object_view(discard).zone, Zone::Graveyard);
        assert!(game.keywords(marauder).contains(Keywords::FLYING));
        assert_eq!(
            game.state.players[player.index()].library.len(),
            length - usize::from(draws)
        );
    }
}

#[test]
fn holy_cow_can_be_cast_off_turn_and_gains_life_before_its_private_scry() {
    let player = Player::P1;
    let mut game = scenario(player.other());
    let cow = put(&mut game, player, "Holy Cow", Zone::Hand);
    let top = put(&mut game, player, "Mountain", Zone::Library);
    act(&mut game, Action::Pass);
    cast(&mut game, cow, None);
    let bottom = Action::ArrangeLibrary {
        card: top,
        destination: LibraryDestination::Bottom,
    };
    checkpoint(&mut game, bottom.clone());
    assert_eq!(game.state.players[player.index()].life, 22);
    assert!(game.observe(player.other()).library_choice.is_none());
    act(&mut game, bottom);
    resolve(&mut game);
    assert!(game.keywords(cow).contains(Keywords::FLYING));
    assert_eq!(
        game.state.players[player.index()].library.first(),
        Some(&top)
    );
}

#[test]
fn insatiable_avarice_tutors_before_its_draw_three_and_life_loss() {
    let player = Player::P0;
    let mut game = scenario(player);
    let wanted = put(&mut game, player, "Hill Giant", Zone::Library);
    put(&mut game, player, "Island", Zone::Library);
    let spell = put(&mut game, player, "Insatiable Avarice", Zone::Hand);
    let length = game.state.players[player.index()].library.len();
    spree(&mut game, spell, 2, &[Target::Player(player)]);
    let pick = Action::ChooseSearch { card: wanted };
    checkpoint(&mut game, pick.clone());
    assert_eq!(game.actor(), Some(player));
    apply_raw_and_index(&mut game, pick);
    if game.legal_actions().contains(&Action::FinishSearch) {
        act(&mut game, Action::FinishSearch);
    }
    resolve(&mut game);
    assert_eq!(game.object_view(wanted).zone, Zone::Hand);
    assert_eq!(game.state.players[player.index()].library.len(), length - 3);
    assert_eq!(game.state.players[player.index()].life, 17);
}

#[test]
fn intimidation_campaign_drains_and_draws_then_can_return_to_its_owner_on_crime() {
    let player = Player::P0;
    let mut game = scenario(player);
    let campaign = put(&mut game, player, "Intimidation Campaign", Zone::Hand);
    let length = game.state.players[player.index()].library.len();
    cast(&mut game, campaign, None);
    resolve(&mut game);
    assert_eq!(game.state.players[player.index()].life, 21);
    assert_eq!(game.state.players[player.other().index()].life, 19);
    assert_eq!(game.state.players[player.index()].library.len(), length - 1);

    let shock = put(&mut game, player, "Shock", Zone::Hand);
    cast(&mut game, shock, Some(Target::Player(player.other())));
    checkpoint(&mut game, Action::ChooseOptional { accept: true });
    let mut decline = game.clone();
    act(&mut decline, Action::ChooseOptional { accept: false });
    resolve(&mut decline);
    assert_eq!(decline.object_view(campaign).zone, Zone::Battlefield);
    act(&mut game, Action::ChooseOptional { accept: true });
    resolve(&mut game);
    assert_eq!(game.object_view(campaign).zone, Zone::Hand);
}

#[test]
fn intrepid_stablemaster_makes_unrestricted_green_or_two_matching_restricted_mana() {
    let player = Player::P1;
    let mut game = scenario(player);
    let stablemaster = put(
        &mut game,
        player,
        "Intrepid Stablemaster",
        Zone::Battlefield,
    );
    let schooner = put(&mut game, player, "Subterranean Schooner", Zone::Hand);
    let opt = put(&mut game, player, "Opt", Zone::Hand);
    game.state.players[player.index()].mana = [0; 6];
    assert!(game.keywords(stablemaster).contains(Keywords::REACH));
    let mut green = game.clone();
    tap_for(&mut green, stablemaster, 0, [0, 0, 0, 0, 1, 0]);
    assert_eq!(green.state.players[player.index()].mana[4], 1);

    let split = Action::ActivateMana {
        card: stablemaster,
        ability: 1,
        payment: [0; 6],
        counter_payment: Counters::default(),
        production: [1, 1, 0, 0, 0, 0],
    };
    reject(&mut game, split);
    tap_for(&mut game, stablemaster, 1, [0, 2, 0, 0, 0, 0]);
    reject(&mut game, Action::AnnounceCast { card: opt });
    cast(&mut game, schooner, None);
    resolve(&mut game);
    assert_eq!(game.object_view(schooner).zone, Zone::Battlefield);
}

#[test]
fn inventive_wingsmith_tests_the_counter_itself_as_well_as_hand_cast_history() {
    let player = Player::P0;
    let mut game = scenario(player);
    let wingsmith = put(&mut game, player, "Inventive Wingsmith", Zone::Battlefield);
    game.state.objects[wingsmith as usize].gained_keywords = Keywords::FLYING;
    let shock = put(&mut game, player, "Shock", Zone::Hand);
    start_step(&mut game, Step::End);
    assert_eq!(game.state.stack.len(), 1);

    let mut hand_cast = game.clone();
    cast(&mut hand_cast, shock, Some(Target::Player(player.other())));
    resolve(&mut hand_cast);
    assert_eq!(hand_cast.counter_count(wingsmith, CounterKind::Flying), 0);

    resolve(&mut game);
    assert_eq!(game.counter_count(wingsmith, CounterKind::Flying), 1);
    assert!(game.keywords(wingsmith).contains(Keywords::FLYING));
    start_step(&mut game, Step::End);
    assert!(game.state.stack.is_empty());
}

#[test]
fn jem_draws_at_own_end_after_exile_casts_but_not_after_hand_casts() {
    let player = Player::P1;
    let mut game = scenario(player);
    let jem = put(
        &mut game,
        player,
        "Jem Lightfoote, Sky Explorer",
        Zone::Battlefield,
    );
    let exile = put(&mut game, player, "Shock", Zone::Exile);
    let hand = put(&mut game, player, "Shock", Zone::Hand);
    let exact = game.object_ref(exile);
    game.grant_exile_play_permission(player, exact, false);
    act(&mut game, Action::AnnounceExile { card: exile });
    choose_targets(&mut game, &[Target::Player(player.other())]);
    pay(&mut game);
    resolve(&mut game);
    assert!(
        game.keywords(jem)
            .contains(Keywords::FLYING.union(Keywords::VIGILANCE))
    );

    let length = game.state.players[player.index()].library.len();
    start_step(&mut game, Step::End);
    let mut stopped = game.clone();
    cast(&mut stopped, hand, Some(Target::Player(player.other())));
    resolve(&mut stopped);
    assert_eq!(stopped.state.players[player.index()].library.len(), length);
    resolve(&mut game);
    assert_eq!(game.state.players[player.index()].library.len(), length - 1);
}

#[test]
fn jailbreak_scheme_combines_modes_and_lets_the_owner_choose_the_library_end() {
    let player = Player::P0;
    let mut game = scenario(player);
    let own = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let foreign = put(&mut game, player.other(), "Short Bow", Zone::Battlefield);
    let spell = put(&mut game, player, "Jailbreak Scheme", Zone::Hand);
    let foreign_ref = game.object_ref(foreign);
    game.record_control(foreign_ref, player, None);
    spree(
        &mut game,
        spell,
        2,
        &[Target::Card(own), Target::Card(foreign)],
    );
    let bottom = Action::ChooseLibraryDestination {
        object: foreign_ref,
        destination: LibraryDestination::Bottom,
    };
    checkpoint(&mut game, bottom.clone());
    assert_eq!(game.actor(), Some(player.other()));
    assert_eq!(game.counter_count(own, CounterKind::PlusOne), 1);
    assert!(game.keywords(own).contains(Keywords::CANT_BE_BLOCKED));
    act(&mut game, bottom);
    resolve(&mut game);
    assert_eq!(
        game.state.players[player.other().index()].library.first(),
        Some(&foreign)
    );
}
