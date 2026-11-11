//! Outlaws of Thunder Junction card scenarios, K through L.
#[path = "support/otj_e_l.rs"]
mod otj;
mod support;

use otj::*;
use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

#[test]
fn kaervek_copies_only_an_own_black_graveyard_card_and_charges_life_on_cast() {
    let player = Player::P0;
    let mut game = scenario(player);
    put(
        &mut game,
        player,
        "Kaervek, the Punisher",
        Zone::Battlefield,
    );
    let original = put(&mut game, player, "Diregraf Ghoul", Zone::Graveyard);
    let foreign = put(&mut game, player.other(), "Diregraf Ghoul", Zone::Graveyard);
    let nonblack = put(&mut game, player, "Grizzly Bears", Zone::Graveyard);
    let shock = put(&mut game, player, "Shock", Zone::Hand);
    cast(&mut game, shock, Some(Target::Player(player.other())));
    let target = Action::ChooseTarget {
        target: Target::Card(original),
    };
    checkpoint(&mut game, target.clone());
    for id in [foreign, nonblack] {
        reject(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(id),
            },
        );
    }
    act(&mut game, target);
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(&mut game, Action::FinishTargets);
    }
    checkpoint(&mut game, Action::FinishResolutionCasts);
    assert_eq!(game.object_view(original).zone, Zone::Exile);
    let mut decline = game.clone();
    act(&mut decline, Action::FinishResolutionCasts);
    resolve(&mut decline);
    assert_eq!(decline.state.players[player.index()].life, 20);
    let exact = game.state.resolution_free_casts.as_ref().unwrap().remaining[0];
    assert_ne!(exact.card, original);
    act(
        &mut game,
        Action::CastExiledDuringResolution {
            object: exact,
            face: CastFace::Normal,
        },
    );
    pay(&mut game);
    act(&mut game, Action::FinishResolutionCasts);
    resolve(&mut game);
    assert_eq!(game.state.players[player.index()].life, 18);
    assert_eq!(game.object_view(original).zone, Zone::Exile);
    assert_eq!(game.object_view(exact.card).zone, Zone::Battlefield);
    assert!(game.object_view(exact.card).is_token());
}

#[test]
fn kambal_copies_an_opposing_batch_once_and_drains_for_each_owned_batch() {
    let player = Player::P1;
    let mut game = scenario(player);
    put(
        &mut game,
        player,
        "Kambal, Profiteering Mayor",
        Zone::Battlefield,
    );
    game.create_token_groups(
        player.other(),
        &[
            TokenGroupView {
                token: TREASURE,
                amount: 1,
                tapped: false,
            },
            TokenGroupView {
                token: MERCENARY,
                amount: 1,
                tapped: false,
            },
        ],
        None,
        true,
    );
    resolve(&mut game);
    for card in [TREASURE, MERCENARY] {
        let found = tokens(&game, player, card);
        assert_eq!(found.len(), 1);
        assert!(game.object_view(found[0]).tapped);
    }
    assert_eq!(game.state.players[player.index()].life, 21);
    assert_eq!(game.state.players[player.other().index()].life, 19);
    game.create_token_batch_capture(player.other(), TREASURE, 2, false, None, true);
    resolve(&mut game);
    assert_eq!(tokens(&game, player, TREASURE).len(), 1);
    game.create_token_batch_capture(player, MERCENARY, 2, false, None, true);
    resolve(&mut game);
    assert_eq!(game.state.players[player.index()].life, 22);
    assert_eq!(game.state.players[player.other().index()].life, 18);
}

#[test]
fn kellan_joins_up_privately_plots_a_small_nonland_and_rewards_legendary_entries() {
    let player = Player::P0;
    let mut game = scenario(player);
    let joins = put(&mut game, player, "Kellan Joins Up", Zone::Hand);
    let longhorn = put(&mut game, player, "Longhorn Sharpshooter", Zone::Hand);
    let large = put(&mut game, player, "Hill Giant", Zone::Hand);
    let land = put(&mut game, player, "Forest", Zone::Hand);
    let yours = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let theirs = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    let legendary = put(&mut game, player, "Kellan, the Kid", Zone::Hand);
    let exact = game.object_ref(longhorn);
    cast(&mut game, joins, None);
    let action = Action::SelectResolutionExile { object: exact };
    checkpoint(&mut game, action.clone());
    assert!(game.observe(player.other()).effect_choice.is_none());
    for card in [large, land] {
        let object = game.object_ref(card);
        reject(&mut game, Action::SelectResolutionExile { object });
    }
    apply_raw_and_index(&mut game, action);
    choose_when_ready(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(player.other()),
        },
    );
    resolve(&mut game);
    assert_eq!(game.object_view(longhorn).zone, Zone::Exile);
    assert_eq!(game.state.players[player.other().index()].life, 18);
    reject(&mut game, Action::AnnouncePlotted { card: longhorn });

    cast(&mut game, legendary, None);
    resolve(&mut game);
    assert_eq!(game.counter_count(yours, CounterKind::PlusOne), 1);
    assert_eq!(game.counter_count(legendary, CounterKind::PlusOne), 1);
    assert_eq!(game.counter_count(theirs, CounterKind::PlusOne), 0);
}

#[test]
fn kellan_the_kid_offers_a_private_small_permanent_or_an_untapped_land() {
    let player = Player::P1;
    let mut game = scenario(player);
    let kid = put(&mut game, player, "Kellan, the Kid", Zone::Battlefield);
    let spell = put(&mut game, player, "Divination", Zone::Exile);
    let bear = put(&mut game, player, "Grizzly Bears", Zone::Hand);
    let large = put(&mut game, player, "Hill Giant", Zone::Hand);
    let land = put(&mut game, player, "Island", Zone::Hand);
    game.state.land_played = true;
    assert!(
        game.keywords(kid)
            .contains(Keywords::FLYING.union(Keywords::LIFELINK))
    );
    let spell_ref = game.object_ref(spell);
    game.grant_exile_play_permission(player, spell_ref, false);
    act(&mut game, Action::AnnounceExile { card: spell });
    pay(&mut game);
    checkpoint(&mut game, Action::FinishResolutionCasts);
    assert!(game.observe(player.other()).resolution_free_casts.is_none());
    let too_big = Action::CastFreeDuringResolution {
        object: game.object_ref(large),
        face: CastFace::Normal,
    };
    reject(&mut game, too_big);

    let mut fallback = game.clone();
    act(&mut fallback, Action::FinishResolutionCasts);
    let land_ref = fallback.object_ref(land);
    act(&mut fallback, Action::PutHandLand { object: land_ref });
    resolve(&mut fallback);
    assert_eq!(fallback.object_view(land).zone, Zone::Battlefield);
    assert!(!fallback.object_view(land).tapped);

    let cast = Action::CastFreeDuringResolution {
        object: game.object_ref(bear),
        face: CastFace::Normal,
    };
    apply_raw_and_index(&mut game, cast);
    pay(&mut game);
    act(&mut game, Action::FinishResolutionCasts);
    resolve(&mut game);
    assert_eq!(game.object_view(bear).zone, Zone::Battlefield);
    assert_eq!(game.object_view(land).zone, Zone::Hand);
}

#[test]
fn kraum_triggers_on_exactly_the_second_spell_and_draws_if_it_leaves() {
    let player = Player::P0;
    let mut game = scenario(player);
    let kraum = put(
        &mut game,
        player,
        "Kraum, Violent Cacophony",
        Zone::Battlefield,
    );
    assert!(game.keywords(kraum).contains(Keywords::FLYING));
    let length = game.state.players[player.index()].library.len();
    let mut departed = None;
    for ordinal in 1..=3 {
        let shock = put(&mut game, player, "Shock", Zone::Hand);
        cast(&mut game, shock, Some(Target::Player(player.other())));
        if ordinal == 2 {
            let mut branch = game.clone();
            branch.move_object(kraum, Zone::Graveyard);
            resolve(&mut branch);
            departed = Some(branch);
        }
        resolve(&mut game);
        let rewarded = usize::from(ordinal >= 2);
        assert_eq!(
            game.counter_count(kraum, CounterKind::PlusOne),
            rewarded as u64
        );
        assert_eq!(
            game.state.players[player.index()].library.len(),
            length - rewarded
        );
    }
    let departed = departed.unwrap();
    assert_eq!(
        departed.state.players[player.index()].library.len(),
        length - 1
    );
    assert_eq!(departed.object_view(kraum).zone, Zone::Graveyard);
}

#[test]
fn lassoed_by_the_law_exiles_until_its_exact_source_leaves_and_creates_a_token_even_without_target()
{
    let player = Player::P1;
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Lassoed by the Law", Zone::Hand);
    let mut no_target = game.clone();
    cast(&mut no_target, spell, None);
    resolve(&mut no_target);
    assert_eq!(tokens(&no_target, player, MERCENARY).len(), 1);
    let victim = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    let original = game.object_ref(victim);
    cast(&mut game, spell, None);
    choose_when_ready(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(victim),
        },
    );
    resolve(&mut game);
    assert_eq!(game.object_view(victim).zone, Zone::Exile);
    assert_eq!(tokens(&game, player, MERCENARY).len(), 1);
    game.move_object(spell, Zone::Graveyard);
    assert_eq!(game.object_view(victim).zone, Zone::Battlefield);
    assert_eq!(game.object_view(victim).controller, player.other());
    assert_ne!(game.object_ref(victim), original);
}

#[test]
fn jasper_adds_mercenary_counts_a_union_and_permits_spell_only_flexible_payment() {
    let player = Player::P0;
    let mut game = scenario(player);
    let jasper = put(
        &mut game,
        player,
        "Laughing Jasper Flint",
        Zone::Battlefield,
    );
    let stolen = put(
        &mut game,
        player.other(),
        "Diregraf Ghoul",
        Zone::Battlefield,
    );
    let stolen_ref = game.object_ref(stolen);
    game.record_control(stolen_ref, player, None);
    let stolen_view = game.object_view(stolen);
    assert!(stolen_view.has_subtype("Mercenary"));
    assert!(stolen_view.has_subtype("Zombie"));
    let below = put(&mut game, player.other(), "Forest", Zone::Library);
    let land = put(&mut game, player.other(), "Island", Zone::Library);
    let shock = put(&mut game, player.other(), "Shock", Zone::Library);
    start_step(&mut game, Step::Upkeep);
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(player.other()),
        },
    );
    resolve(&mut game);
    assert_eq!(game.object_view(shock).zone, Zone::Exile);
    assert_eq!(game.object_view(land).zone, Zone::Exile);
    assert_eq!(game.object_view(below).zone, Zone::Library);
    game.move_object(jasper, Zone::Graveyard);
    assert!(!game.object_view(stolen).has_subtype("Mercenary"));
    game.state.players[player.index()].mana = [0, 0, 0, 0, 1, 0];
    act(&mut game, Action::AnnounceExile { card: shock });
    choose_targets(&mut game, &[Target::Player(player.other())]);
    pay(&mut game);
    resolve(&mut game);
    assert_eq!(game.state.players[player.other().index()].life, 18);
}

#[test]
fn lazav_has_separate_exile_and_copy_choices_and_retains_its_counter() {
    let player = Player::P1;
    let mut game = scenario(player);
    let lazav = put(
        &mut game,
        player,
        "Lazav, Familiar Stranger",
        Zone::Battlefield,
    );
    let creature = put(&mut game, player.other(), "Hill Giant", Zone::Graveyard);
    let exact = game.object_ref(creature);
    let shock = put(&mut game, player, "Shock", Zone::Hand);
    cast(&mut game, shock, Some(Target::Player(player.other())));
    let exile = Action::SelectResolutionExile { object: exact };
    checkpoint(&mut game, exile.clone());
    assert_eq!(game.counter_count(lazav, CounterKind::PlusOne), 1);
    apply_raw_and_index(&mut game, exile);
    let mut decline = game.clone();
    act(&mut decline, Action::ChooseOptional { accept: false });
    resolve(&mut decline);
    assert_eq!(decline.object_view(lazav).name, "Lazav, Familiar Stranger");
    act(&mut game, Action::ChooseOptional { accept: true });
    resolve(&mut game);
    assert_eq!(game.object_view(creature).zone, Zone::Exile);
    assert_eq!(game.object_view(lazav).name, "Hill Giant");
    assert_eq!(stats(&game, lazav), (4, 4));
    assert_eq!(game.counter_count(lazav, CounterKind::PlusOne), 1);
}

#[test]
fn lilah_triggers_prowess_and_plots_only_a_resolving_multicolored_hand_spell() {
    let player = Player::P0;
    let mut game = scenario(player);
    let lilah = put(
        &mut game,
        player,
        "Lilah, Undefeated Slickshot",
        Zone::Battlefield,
    );
    let helix = put(&mut game, player, "Lightning Helix", Zone::Hand);
    let mono = put(&mut game, player, "Shock", Zone::Hand);
    let target = Target::Player(player.other());
    cast(&mut game, helix, Some(target));
    resolve(&mut game);
    assert_eq!(game.object_view(helix).zone, Zone::Exile);
    assert_eq!(game.state.players[player.other().index()].life, 17);
    assert_eq!(game.state.players[player.index()].life, 23);
    assert_eq!(stats(&game, lilah), (4, 4));
    reject(&mut game, Action::AnnouncePlotted { card: helix });

    cast(&mut game, mono, Some(target));
    resolve(&mut game);
    assert_eq!(game.object_view(mono).zone, Zone::Graveyard);
    assert_eq!(stats(&game, lilah), (5, 5));
}

#[test]
fn longhorn_plot_is_a_special_action_then_queues_separate_targeted_damage() {
    let player = Player::P1;
    let mut game = scenario(player);
    let longhorn = put(&mut game, player, "Longhorn Sharpshooter", Zone::Hand);
    let original = game.object_ref(longhorn);
    act(&mut game, Action::AnnouncePlot { card: longhorn });
    assert!(game.state.stack.is_empty());
    choose(&mut game, |action| matches!(action, Action::PayPlot { .. }));
    assert_eq!(game.object_view(longhorn).zone, Zone::Exile);
    assert_ne!(game.object_ref(longhorn), original);
    choose_when_ready(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(player.other()),
        },
    );
    resolve(&mut game);
    assert_eq!(game.state.players[player.other().index()].life, 18);
    reject(&mut game, Action::AnnouncePlotted { card: longhorn });
}

#[test]
fn luxurious_locomotive_can_crew_once_and_counts_each_actual_contributor() {
    let player = Player::P0;
    let mut game = scenario(player);
    let train = put(&mut game, player, "Luxurious Locomotive", Zone::Battlefield);
    let first = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let second = put(&mut game, player, "Goblin Piker", Zone::Battlefield);
    put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    assert_eq!(game.object_view(train).kinds & 1, 0);
    announce_activation(&mut game, train, 0);
    for card in [first, second] {
        let object = game.object_ref(card);
        act(&mut game, Action::ChooseCrewer { object });
    }
    act(&mut game, Action::FinishCrew);
    resolve(&mut game);
    assert_eq!(stats(&game, train), (6, 5));
    reject(
        &mut game,
        Action::AnnounceActivate {
            card: train,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    game.move_object(first, Zone::Graveyard);
    attack(&mut game, train);
    resolve(&mut game);
    let treasure = tokens(&game, player, TREASURE);
    assert_eq!(treasure.len(), 2);
    assert!(treasure.into_iter().all(|id| !game.object_view(id).tapped));
}

#[test]
fn lively_dirge_tutors_before_its_nontargeted_graveyard_return() {
    let player = Player::P0;
    let mut game = scenario(player);
    let first = put(&mut game, player, "Grizzly Bears", Zone::Library);
    let second = put(&mut game, player, "Grizzly Bears", Zone::Graveyard);
    let large = put(&mut game, player, "Hill Giant", Zone::Graveyard);
    let spell = put(&mut game, player, "Lively Dirge", Zone::Hand);
    spree(&mut game, spell, 2, &[]);
    choose_when_ready(&mut game, Action::ChooseSearch { card: first });
    if game.legal_actions().contains(&Action::FinishSearch) {
        act(&mut game, Action::FinishSearch);
    }
    let first_ref = game.object_ref(first);
    let second_ref = game.object_ref(second);
    let large_ref = game.object_ref(large);
    act(
        &mut game,
        Action::SelectGraveyardReturn { object: first_ref },
    );
    reject(
        &mut game,
        Action::SelectGraveyardReturn { object: large_ref },
    );
    act(
        &mut game,
        Action::SelectGraveyardReturn { object: second_ref },
    );
    if game
        .legal_actions()
        .contains(&Action::FinishGraveyardReturn)
    {
        act(&mut game, Action::FinishGraveyardReturn);
    }
    resolve(&mut game);
    assert_eq!(game.object_view(first).zone, Zone::Battlefield);
    assert_eq!(game.object_view(second).zone, Zone::Battlefield);
    assert_eq!(game.object_view(large).zone, Zone::Graveyard);
}
