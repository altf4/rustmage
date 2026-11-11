//! Card wiring for Discover, descend, and exact creature or spell references.
mod support;
use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

const TREASURE: Card = Card(5302);

fn own(game: &mut Game, player: Player, name: &str, zone: Zone) -> CardId {
    spawn(game, player, Card::from_name(name).unwrap(), zone)
}

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn advance_to(game: &mut Game, pending: Pending) {
    for _ in 0..24 {
        if game.state.pending == pending {
            return;
        }
        choose(game, |action| {
            matches!(
                action,
                Action::Pass | Action::PutTrigger { .. } | Action::PayCosts { .. }
            )
        });
    }
    panic!("expected {pending:?}");
}

fn discover(game: &mut Game, player: Player, expected: CardId) {
    advance_to(game, Pending::ResolutionCasts { player });
    assert_eq!(
        game.observe(player.other()).cascade.unwrap().discovered,
        Some(game.object_ref(expected))
    );
    act(game, Action::FinishResolutionCasts);
    assert_eq!(game.object_view(expected).zone, Zone::Hand);
}

fn treasures(game: &Game) -> impl Iterator<Item = &Object> {
    game.state
        .objects
        .iter()
        .filter(|object| object.zone == Zone::Battlefield && object.card == TREASURE)
}

#[test]
fn digsite_conservator_limits_its_targets_and_offers_discover_before_exiling() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let conservator = own(&mut game, player, "Digsite Conservator", Zone::Battlefield);
    let first = put(&mut game, opponent, "Forest", Zone::Graveyard);
    let second = put(&mut game, opponent, "Island", Zone::Graveyard);
    let wrong = put(&mut game, player, "Shock", Zone::Graveyard);
    let found = put(&mut game, player, "Galvanic Giant", Zone::Library);
    let activation = Action::AnnounceActivate {
        card: conservator,
        ability: 0,
        counter_payment: Counters::default(),
    };
    game.state.step = Step::Upkeep;
    reject(&mut game, activation.clone());
    game.state.step = Step::Main1;
    act(&mut game, activation);
    let selection = |id| Action::ChooseTarget {
        target: Target::Card(id),
    };
    act(&mut game, selection(first));
    reject(&mut game, selection(wrong));
    reject(&mut game, selection(first));
    act(&mut game, selection(second));
    act(&mut game, Action::FinishTargets);
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(&mut game);
    }
    assert_eq!(game.object_view(conservator).zone, Zone::Graveyard);
    advance_to(&mut game, Pending::ResolutionMana { player });
    assert_eq!(game.object_view(first).zone, Zone::Graveyard);
    let mut declined = game.clone();
    act(&mut declined, Action::DeclineResolutionMana);
    settle(&mut declined);
    assert_eq!(declined.object_view(first).zone, Zone::Exile);
    assert_eq!(declined.object_view(second).zone, Zone::Exile);
    assert_eq!(declined.top_library_card(player), Some(found));

    game.move_object(second, Zone::Hand);
    act(
        &mut game,
        Action::PayResolutionMana {
            payment: [0, 0, 0, 0, 0, 4],
        },
    );
    discover(&mut game, player, found);
    settle(&mut game);
    assert_eq!(game.object_view(first).zone, Zone::Exile);
    assert_eq!(game.object_view(second).zone, Zone::Hand);
    assert_eq!(game.object_view(wrong).zone, Zone::Graveyard);
}

#[test]
fn disturbed_slumber_animates_a_controlled_land_and_requires_a_block() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let land = put(&mut game, player, "Forest", Zone::Battlefield);
    let opposing_land = put(&mut game, opponent, "Forest", Zone::Battlefield);
    let blocker = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    let spell = own(&mut game, player, "Disturbed Slumber", Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: spell });
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(opposing_land),
        },
    );
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(blocker),
        },
    );
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(land),
        },
    );
    pay(&mut game);
    settle(&mut game);
    assert_eq!(stats(&game, land), (4, 4));
    assert!(game.has_subtype(land, "Forest"));
    assert!(game.has_subtype(land, "Dinosaur"));
    assert!(
        game.keywords(land)
            .contains(Keywords::REACH.union(Keywords::HASTE))
    );
    assert!(
        game.legal_actions()
            .contains(&Action::TapMana { card: land })
    );

    game.begin_step(Step::BeginCombat);
    pass_pair(&mut game);
    act(&mut game, Action::Attack { card: land });
    act(&mut game, Action::FinishAttackers);
    pass_pair(&mut game);
    reject(&mut game, Action::FinishBlockers);
    act(
        &mut game,
        Action::Block {
            blocker,
            attacker: land,
        },
    );
    act(&mut game, Action::FinishBlockers);
}

#[test]
fn earthshaker_dreadmaw_draws_for_other_live_controlled_dinosaurs() {
    let player = Player::P0;
    let mut game = scenario(player);
    let removed = put(&mut game, player, "Hulking Raptor", Zone::Battlefield);
    put(&mut game, player, "Hulking Raptor", Zone::Battlefield);
    put(
        &mut game,
        player.other(),
        "Hulking Raptor",
        Zone::Battlefield,
    );
    put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let big = own(&mut game, player, "Earthshaker Dreadmaw", Zone::Hand);
    cast(&mut game, big, None);
    pass_pair(&mut game);
    assert!(game.keywords(big).contains(Keywords::TRAMPLE));
    game.move_object(removed, Zone::Graveyard);
    let top = game.top_library_card(player).unwrap();
    settle(&mut game);
    assert_eq!(game.object_view(top).zone, Zone::Hand);
    assert_eq!(game.hand_count(player), 1);
}

#[test]
fn enterprising_scallywag_checks_graveyard_arrival_history_and_own_end_step() {
    let player = Player::P0;
    for permanent in [false, true] {
        let mut game = scenario(player);
        put(&mut game, player, "Forest", Zone::Graveyard);
        let arrived = put(
            &mut game,
            player,
            if permanent { "Island" } else { "Shock" },
            Zone::Library,
        );
        game.move_object(arrived, Zone::Graveyard);
        game.move_object(arrived, Zone::Exile);
        let scallywag = own(&mut game, player, "Enterprising Scallywag", Zone::Hand);
        cast(&mut game, scallywag, None);
        settle(&mut game);
        game.state.active = player.other();
        game.begin_step(Step::End);
        game.prepare_triggers();
        settle(&mut game);
        assert_eq!(treasures(&game).count(), 0);
        game.state.active = player;
        game.begin_step(Step::End);
        game.prepare_triggers();
        settle(&mut game);
        assert_eq!(treasures(&game).count(), usize::from(permanent));
    }
}

#[test]
fn etalis_favor_requires_own_creature_and_discovers_only_if_the_aura_enters() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let creature = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let opponent_creature = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    let hit = put(&mut game, player, "Shock", Zone::Library);
    let aura = own(&mut game, player, "Etali's Favor", Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: aura });
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(opponent_creature),
        },
    );
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(creature),
        },
    );
    pay(&mut game);

    let mut invalidated = game.clone();
    invalidated.move_object(creature, Zone::Hand);
    settle(&mut invalidated);
    assert_eq!(invalidated.object_view(aura).zone, Zone::Graveyard);
    assert_eq!(invalidated.top_library_card(player), Some(hit));

    discover(&mut game, player, hit);
    assert_eq!(
        game.object_view(aura).attached_to,
        Some(game.object_ref(creature))
    );
    assert_eq!(stats(&game, creature), (3, 3));
    assert!(game.keywords(creature).contains(Keywords::TRAMPLE));
}

#[test]
fn geological_appraiser_discovers_after_a_cast_and_stays_quiet_after_reanimation() {
    let player = Player::P1;
    let mut game = scenario(player);
    let found = put(&mut game, player, "Shock", Zone::Library);
    let too_expensive = put(&mut game, player, "Galvanic Giant", Zone::Library);
    let card = own(&mut game, player, "Geological Appraiser", Zone::Hand);
    let mut reanimated = game.clone();
    reanimated.move_object(card, Zone::Graveyard);
    let zombify = put(&mut reanimated, player, "Zombify", Zone::Hand);
    cast(&mut reanimated, zombify, Some(Target::Card(card)));
    settle(&mut reanimated);
    assert_eq!(reanimated.object_view(card).zone, Zone::Battlefield);
    assert_eq!(reanimated.top_library_card(player), Some(too_expensive));
    cast(&mut game, card, None);
    discover(&mut game, player, found);
    assert_eq!(game.object_view(too_expensive).zone, Zone::Library);
}

#[test]
fn hit_the_mother_lode_awards_tapped_treasures_for_the_physical_discovered_card() {
    let player = Player::P0;
    for (candidate, expected) in [(None, 0), (Some("Galvanic Giant"), 6)] {
        let mut game = scenario(player);
        let candidate = candidate.map(|name| put(&mut game, player, name, Zone::Library));
        let spell = own(&mut game, player, "Hit the Mother Lode", Zone::Hand);
        game.state.players[player.index()].mana = [0, 0, 0, 3, 0, 4];
        cast(&mut game, spell, None);
        if let Some(found) = candidate {
            advance_to(&mut game, Pending::ResolutionCasts { player });
            assert_eq!(treasures(&game).count(), 0);
            let mut declined = game.clone();
            act(&mut declined, Action::FinishResolutionCasts);
            assert_eq!(declined.object_view(found).zone, Zone::Hand);
            assert_eq!(treasures(&declined).count(), expected);
            let object = game.object_ref(found);
            act(
                &mut game,
                Action::CastExiledDuringResolution {
                    object,
                    face: CastFace::Adventure,
                },
            );
            advance_to(&mut game, Pending::ResolutionCasts { player });
            act(&mut game, Action::FinishResolutionCasts);
        } else {
            settle(&mut game);
        }
        assert_eq!(treasures(&game).count(), expected);
        assert!(treasures(&game).all(|treasure| treasure.tapped));
    }
}

#[test]
fn hurl_into_history_counters_a_creature_and_discovers_its_chosen_mana_value() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let creature = put(&mut game, player, "Ingenious Prodigy", Zone::Hand);
    let found = put(&mut game, opponent, "Galvanic Giant", Zone::Library);
    let hurl = own(&mut game, opponent, "Hurl into History", Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: creature });
    act(&mut game, Action::ChooseX { value: 3 });
    pay(&mut game);
    act(&mut game, Action::Pass);
    cast(&mut game, hurl, Some(Target::Spell(creature)));
    discover(&mut game, opponent, found);
    assert_eq!(game.object_view(creature).zone, Zone::Graveyard);
}

#[test]
fn join_the_dead_counts_physical_permanents_at_resolution() {
    let player = Player::P0;
    let mut game = scenario(player);
    let target = put(
        &mut game,
        player.other(),
        "Gigantosaurus",
        Zone::Battlefield,
    );
    let spell = own(&mut game, player, "Join the Dead", Zone::Hand);
    for name in ["Forest", "Bear Cub", "Short Bow", "Shock"] {
        put(&mut game, player, name, Zone::Graveyard);
    }
    let fourth = put(&mut game, player, "Island", Zone::Library);
    cast(&mut game, spell, Some(Target::Card(target)));
    let mut below_threshold = game.clone();
    settle(&mut below_threshold);
    assert_eq!(below_threshold.object_view(target).zone, Zone::Battlefield);
    assert_eq!(stats(&below_threshold, target), (5, 5));
    game.move_object(fourth, Zone::Graveyard);
    settle(&mut game);
    assert_eq!(game.object_view(target).zone, Zone::Graveyard);
}
