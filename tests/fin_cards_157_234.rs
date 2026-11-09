mod support;
use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn passes(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn choose_targets(game: &mut Game, targets: &[Target]) {
    for &target in targets {
        act(game, Action::ChooseTarget { target });
    }
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(game, Action::FinishTargets);
    }
}

fn cast_mode(game: &mut Game, card: CardId, mode: u16, targets: &[Target]) {
    act(game, Action::AnnounceCast { card });
    act(game, Action::ChooseMode { mode });
    choose_targets(game, targets);
    pay(game);
}

fn activate_mode(game: &mut Game, card: CardId, mode: u16, target: Target) {
    act(
        game,
        Action::AnnounceActivate {
            card,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    act(game, Action::ChooseMode { mode });
    choose_targets(game, &[target]);
    pay(game);
}

#[test]
fn nibelheim_uses_the_creatures_lifelink_and_graveyard_cast_is_the_only_wheel() {
    let player = Player::P0;
    for flashback in [false, true] {
        let mut game = scenario(player);
        let caster = put(&mut game, player, "Healer's Hawk", Zone::Battlefield);
        let own = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let foe = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
        let extra = put(&mut game, player, "Island", Zone::Hand);
        let spell = put(
            &mut game,
            player,
            "Nibelheim Aflame",
            if flashback {
                Zone::Graveyard
            } else {
                Zone::Hand
            },
        );
        let target = Target::Card(caster);
        if flashback {
            act(&mut game, Action::AnnounceFlashback { card: spell });
            choose_targets(&mut game, &[target]);
            pay(&mut game);
        } else {
            cast(&mut game, spell, Some(target));
        }
        settle(&mut game);
        assert_eq!(game.object_view(caster).damage, 0);
        for id in [own, foe] {
            assert_eq!(game.object_view(id).damage, 1);
        }
        assert_eq!(game.state.players[player.index()].life, 22);
        assert_eq!(game.hand_count(player), if flashback { 4 } else { 1 });
        assert_eq!(
            game.object_view(extra).zone,
            if flashback {
                Zone::Graveyard
            } else {
                Zone::Hand
            }
        );
        assert_eq!(
            game.object_view(spell).zone,
            if flashback {
                Zone::Exile
            } else {
                Zone::Graveyard
            }
        );
    }
}

#[test]
fn noctis_makes_an_exact_additional_life_route_and_finality_applies_to_noncreatures() {
    let player = Player::P0;
    let mut game = scenario(player);
    let noctis = put(
        &mut game,
        player,
        "Noctis, Prince of Lucis",
        Zone::Battlefield,
    );
    let artifact = put(&mut game, player, "Short Bow", Zone::Graveyard);
    let rival = put(&mut game, player.other(), "Short Bow", Zone::Graveyard);
    let route = |game: &Game, card| Action::AnnounceGraveyardWithLife {
        object: game.object_ref(card),
        face: CastFace::Normal,
        method: CastMethod::Normal,
        flash_surcharge: false,
        life: 3,
        entry_counter: CounterKind::Finality,
    };
    assert!(game.keywords(noctis).contains(Keywords::LIFELINK));
    let forbidden = route(&game, rival);
    reject(&mut game, forbidden);
    let action = route(&game, artifact);
    act(&mut game, action);
    pay(&mut game);
    let clause = game
        .observe(player)
        .cast_costs
        .unwrap()
        .clauses
        .iter()
        .position(|row| row.clause == CastCostClause::Life(3) && !row.paid)
        .unwrap() as u16;
    act(&mut game, Action::ChooseCastCost { clause });
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].life, 17);
    assert_eq!(game.counter_count(artifact, CounterKind::Finality), 1);
    game.move_object(artifact, Zone::Graveyard);
    assert_eq!(game.object_view(artifact).zone, Zone::Exile);
}

#[test]
fn opera_grants_normal_land_play_and_the_other_card_expires_at_its_controllers_end_step() {
    let player = Player::P0;
    let mut game = scenario(player);
    let land = put(&mut game, player, "Island", Zone::Library);
    let spell = put(&mut game, player, "Opt", Zone::Library);
    let song = put(&mut game, player, "Opera Love Song", Zone::Hand);
    cast_mode(&mut game, song, 0, &[]);
    settle(&mut game);
    assert_eq!(game.object_view(land).zone, Zone::Exile);
    assert_eq!(game.object_view(spell).zone, Zone::Exile);
    act(&mut game, Action::PlayLand { card: land });
    assert_eq!(game.object_view(land).zone, Zone::Battlefield);
    assert!(game.legal_actions().iter().any(|action| {
        matches!(action, Action::Cast { card, .. } | Action::AnnounceCast { card } if *card == spell)
    }));
    game.begin_step(Step::End);
    game.give_priority();
    assert!(!game.legal_actions().iter().any(|action| {
        matches!(action, Action::Cast { card, .. } | Action::AnnounceCast { card } if *card == spell)
    }));
}

#[test]
fn phantom_train_sacrifices_another_permanent_and_its_counter_outlasts_animation() {
    let player = Player::P0;
    let mut game = scenario(player);
    let train = put(&mut game, player, "Phantom Train", Zone::Battlefield);
    let sacrifice = put(&mut game, player, "Short Bow", Zone::Battlefield);
    let stale = game.object_ref(train);
    let other = game.object_ref(sacrifice);
    act(
        &mut game,
        Action::AnnounceActivate {
            card: train,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    reject(&mut game, Action::ChooseAdditionalCost { object: stale });
    act(&mut game, Action::ChooseAdditionalCost { object: other });
    if game.state.announcing.is_some() {
        pay(&mut game);
    }
    settle(&mut game);
    assert_eq!(game.object_view(sacrifice).zone, Zone::Graveyard);
    assert!(game.has_subtype(train, "Spirit"));
    assert_eq!(stats(&game, train), (5, 5));
    assert!(game.keywords(train).contains(Keywords::TRAMPLE));
    game.finish_cleanup();
    assert!(!game.is_creature(train));
    assert!(!game.has_subtype(train, "Spirit"));
    assert_eq!(game.counter_count(train, CounterKind::PlusOne), 1);
}

#[test]
fn phoenix_down_has_one_paid_modal_activation_with_distinct_targets() {
    let player = Player::P0;
    for mode in [0, 1] {
        let mut game = scenario(player);
        let relic = put(&mut game, player, "Phoenix Down", Zone::Battlefield);
        let own = put(&mut game, player, "Hill Giant", Zone::Graveyard);
        let zombie = put(
            &mut game,
            player.other(),
            "Diregraf Ghoul",
            Zone::Battlefield,
        );
        let target = if mode == 0 { own } else { zombie };
        activate_mode(&mut game, relic, mode, Target::Card(target));
        assert_eq!(game.object_view(relic).zone, Zone::Exile);
        settle(&mut game);
        if mode == 0 {
            assert_eq!(game.object_view(own).zone, Zone::Battlefield);
            assert!(game.object_view(own).tapped);
            assert_eq!(game.object_view(zombie).zone, Zone::Battlefield);
        } else {
            assert_eq!(game.object_view(zombie).zone, Zone::Exile);
            assert_eq!(game.object_view(own).zone, Zone::Graveyard);
        }
    }
}

#[test]
fn poison_shrinks_each_side_or_reveals_a_hand_for_a_filtered_discard() {
    let player = Player::P0;
    let mut game = scenario(player);
    let own = put(&mut game, player, "Healer's Hawk", Zone::Battlefield);
    let enemy = put(
        &mut game,
        player.other(),
        "Healer's Hawk",
        Zone::Battlefield,
    );
    let poison = put(&mut game, player, "Poison the Waters", Zone::Hand);
    cast_mode(&mut game, poison, 0, &[]);
    settle(&mut game);
    for id in [own, enemy] {
        assert_eq!(game.object_view(id).zone, Zone::Graveyard);
    }
    let land = put(&mut game, player, "Island", Zone::Hand);
    let artifact = put(&mut game, player, "Short Bow", Zone::Hand);
    let poison = put(&mut game, player, "Poison the Waters", Zone::Hand);
    cast_mode(&mut game, poison, 1, &[Target::Player(player)]);
    passes(&mut game);
    let land = game.object_ref(land);
    reject(&mut game, Action::ChooseRevealedDiscard { object: land });
    let object = game.object_ref(artifact);
    act(&mut game, Action::ChooseRevealedDiscard { object });
    assert_eq!(game.object_view(artifact).zone, Zone::Graveyard);
    assert_eq!(game.object_view(land.card).zone, Zone::Hand);
}

#[test]
fn prishe_targets_only_after_a_search_and_can_trigger_after_failing_to_find() {
    let player = Player::P0;
    for found in [false, true] {
        let mut game = scenario(player);
        let recipient = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let town = put(&mut game, player, "Rabanastre, Royal City", Zone::Library);
        let spell = put(&mut game, player, "Prishe's Wanderings", Zone::Hand);
        cast(&mut game, spell, None);
        assert!(game.state.stack[0].targets.is_empty());
        passes(&mut game);
        if found {
            act(&mut game, Action::ChooseSearch { card: town });
        }
        if game.legal_actions().contains(&Action::FinishSearch) {
            act(&mut game, Action::FinishSearch);
        }
        if found {
            assert_eq!(game.object_view(town).zone, Zone::Battlefield);
            assert!(game.object_view(town).tapped);
        } else {
            assert_eq!(game.object_view(town).zone, Zone::Library);
        }
        choose_targets(&mut game, &[Target::Card(recipient)]);
        assert!(matches!(
            game.state.stack[0].kind,
            StackKind::TriggeredAbility { .. }
        ));
        settle(&mut game);
        assert_eq!(game.counter_count(recipient, CounterKind::PlusOne), 1);
    }
}
