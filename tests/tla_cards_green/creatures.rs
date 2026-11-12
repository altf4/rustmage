use crate::{reach, support::*};
use rustmage_core::engine::ability_world::CharacteristicWorld;
use rustmage_core::{cards::*, engine::*};

fn activate(game: &mut Game, source: CardId) {
    choose(
        game,
        |action| matches!(action, Action::Activate { card, ability: 0, .. } if *card == source),
    );
}

fn mana(card: CardId, production: Mana) -> Action {
    Action::ActivateMana {
        card,
        ability: 0,
        payment: [0; 6],
        counter_payment: Counters::default(),
        production,
    }
}

fn attack(game: &mut Game, attackers: &[CardId]) {
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    for &card in attackers {
        act(game, Action::Attack { card });
    }
    act(game, Action::FinishAttackers);
}

#[test]
fn the_zookeeper_counts_creature_subtypes_and_leaves_humans_unchanged() {
    let player = Player::P1;
    let mut game = scenario(player);
    let keeper = put(&mut game, player, "Diligent Zookeeper", Zone::Battlefield);
    let bear = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let horse = put(&mut game, player, "Ostrich-Horse", Zone::Battlefield);
    let human = put(&mut game, player, "Wartime Protestors", Zone::Battlefield);
    let other = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    assert_eq!(stats(&game, keeper), (4, 4));
    assert_eq!(stats(&game, bear), (3, 3));
    assert_eq!(stats(&game, horse), (5, 3));
    assert_eq!(stats(&game, human), (4, 4));
    assert_eq!(stats(&game, other), (2, 2));
    game.move_object(keeper, Zone::Hand);
    assert_eq!(stats(&game, horse), (3, 1));
}

#[test]
fn flopsie_adds_counters_and_limits_only_high_power_attackers() {
    let player = Player::P0;
    let mut game = scenario(player);
    let big = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let little = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let first = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    let second = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    let flopsie = put(&mut game, player, "Flopsie, Bumi's Buddy", Zone::Hand);
    cast(&mut game, flopsie, None);
    settle(&mut game);
    for card in [flopsie, big, little] {
        assert_eq!(game.counter_count(card, CounterKind::PlusOne), 1);
    }
    assert_eq!(game.counter_count(first, CounterKind::PlusOne), 0);
    attack(&mut game, &[big, little]);
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    let mut two = game.clone();
    act(
        &mut two,
        Action::Block {
            blocker: first,
            attacker: little,
        },
    );
    act(
        &mut two,
        Action::Block {
            blocker: second,
            attacker: little,
        },
    );
    act(
        &mut game,
        Action::Block {
            blocker: first,
            attacker: big,
        },
    );
    reject(
        &mut game,
        Action::Block {
            blocker: second,
            attacker: big,
        },
    );
}

#[test]
fn vinebender_pays_with_a_sick_creature_and_cannot_be_blocked_by_small_creatures() {
    let player = Player::P1;
    let mut game = scenario(player);
    let source = put(
        &mut game,
        player,
        "Foggy Swamp Vinebender",
        Zone::Battlefield,
    );
    let helper = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    game.state.objects[helper as usize].summoning_sick = true;
    let enemy = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    let bigger = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    let action = Action::AnnounceActivate {
        card: source,
        ability: 0,
        counter_payment: Counters::default(),
    };
    let mut off_turn = game.clone();
    off_turn.state.active = player.other();
    reject(&mut off_turn, action.clone());
    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 4];
    act(&mut game, action);
    act(&mut game, Action::BeginWaterbend);
    let exact = game.object_ref(helper);
    act(&mut game, Action::ChooseWaterbender { object: exact });
    pay(&mut game);
    settle(&mut game);
    assert!(game.object_view(helper).tapped);
    assert_eq!(game.counter_count(source, CounterKind::PlusOne), 1);
    attack(&mut game, &[source]);
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    reject(
        &mut game,
        Action::Block {
            blocker: enemy,
            attacker: source,
        },
    );
    act(
        &mut game,
        Action::Block {
            blocker: bigger,
            attacker: source,
        },
    );
}

#[test]
fn great_divide_guide_grants_one_ability_to_a_land_that_is_also_an_ally() {
    let player = Player::P0;
    let mut game = scenario(player);
    let guide = put(&mut game, player, "Great Divide Guide", Zone::Battlefield);
    let land = put(&mut game, player, "Forest", Zone::Battlefield);
    let nonally = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let enemy = put(&mut game, player.other(), "Island", Zone::Battlefield);
    let lesson = put(&mut game, player, "Earthbending Lesson", Zone::Hand);
    cast(&mut game, lesson, Some(Target::Card(land)));
    settle(&mut game);
    let exact = game.object_ref(land);
    game.change_object_form(
        exact,
        AnimationSpec {
            artifact: false,
            creature_types: AnimationCreatureTypes::Additional(&["Ally"]),
            base: None,
            keywords: Keywords::NONE,
            duration: AnimationDuration::Permanent,
        },
        characteristic_layers::AnimationTypeMode::AddCreature,
    );
    for card in [guide, land] {
        let grants = CharacteristicWorld::granted_mana(&game, card);
        assert_eq!(grants.len(), 1);
        assert_eq!(grants[0].source, game.object_ref(guide));
    }
    for card in [nonally, enemy] {
        assert!(CharacteristicWorld::granted_mana(&game, card).is_empty());
    }
    game.state.players[player.index()].mana = [0; 6];
    game.state.objects[guide as usize].summoning_sick = true;
    let hawk = put(&mut game, player, "Healer's Hawk", Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: hawk });
    let grantor = game.object_ref(guide);
    act(
        &mut game,
        Action::TapGrantedMana {
            card: land,
            grantor,
            production: [1, 0, 0, 0, 0, 0],
        },
    );
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(hawk).zone, Zone::Battlefield);
}

#[test]
fn raucous_audience_funds_the_two_mana_cast_only_at_the_live_power_threshold() {
    let player = Player::P1;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Raucous Audience", Zone::Battlefield);
    let creature = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    put(
        &mut game,
        player.other(),
        "Shivan Dragon",
        Zone::Battlefield,
    );
    let spell = put(&mut game, player, "Bear Cub", Zone::Hand);
    let announce = Action::AnnounceCast { card: spell };
    game.state.players[player.index()].mana = [0; 6];
    reject(&mut game, announce.clone());
    let mut ordinary = game.clone();
    act(&mut ordinary, mana(source, [0, 0, 0, 0, 1, 0]));
    assert_eq!(
        ordinary.state.players[player.index()].mana,
        [0, 0, 0, 0, 1, 0]
    );
    game.add_counter(creature, CounterKind::PlusOne, 1);
    act(&mut game, announce);
    act(&mut game, mana(source, [0, 0, 0, 0, 2, 0]));
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(spell).zone, Zone::Battlefield);
}

#[test]
fn ostrich_horse_and_sparring_dummy_use_only_their_own_mill_results() {
    let player = Player::P0;
    let mut game = scenario(player);
    let old_land = put(&mut game, player, "Plains", Zone::Graveyard);
    let land = put(&mut game, player, "Island", Zone::Library);
    let lesson = put(&mut game, player, "Firebending Lesson", Zone::Library);
    put(&mut game, player, "Bear Cub", Zone::Library);
    let horse = put(&mut game, player, "Ostrich-Horse", Zone::Hand);
    cast(&mut game, horse, None);
    reach(&mut game, |g| {
        g.legal_actions()
            .iter()
            .any(|action| matches!(action, Action::RecoverMilled { .. }))
    });
    let old_ref = game.object_ref(old_land);
    let land_ref = game.object_ref(land);
    reject(
        &mut game,
        Action::RecoverMilled {
            object: Some(old_ref),
        },
    );
    let mut decline = game.clone();
    act(&mut decline, Action::RecoverMilled { object: None });
    assert_eq!(stats(&decline, horse), (4, 2));
    act(
        &mut game,
        Action::RecoverMilled {
            object: Some(land_ref),
        },
    );
    assert_eq!(stats(&game, horse), (3, 1));
    assert_eq!(game.object_view(land).zone, Zone::Hand);
    assert_eq!(game.object_view(lesson).zone, Zone::Graveyard);

    let dummy = put(&mut game, player, "Sparring Dummy", Zone::Battlefield);
    assert!(game.keywords(dummy).contains(Keywords::DEFENDER));
    let top = put(&mut game, player, "Mountain", Zone::Library);
    let mut recover = game.clone();
    activate(&mut recover, dummy);
    reach(&mut recover, |g| {
        g.legal_actions()
            .iter()
            .any(|action| matches!(action, Action::RecoverMilled { .. }))
    });
    let top_ref = recover.object_ref(top);
    act(
        &mut recover,
        Action::RecoverMilled {
            object: Some(top_ref),
        },
    );
    assert_eq!(recover.object_view(top).zone, Zone::Hand);
    assert_eq!(recover.state.players[player.index()].life, 20);
    let top_lesson = put(&mut game, player, "Firebending Lesson", Zone::Library);
    activate(&mut game, dummy);
    settle(&mut game);
    assert_eq!(game.object_view(top_lesson).zone, Zone::Graveyard);
    assert_eq!(game.state.players[player.index()].life, 22);
}

#[test]
fn the_cabbage_merchant_creates_food_and_bottoms_itself_only_after_searching() {
    let player = Player::P1;
    let mut game = scenario(player);
    let merchant = put(&mut game, player, "Unlucky Cabbage Merchant", Zone::Hand);
    let land = put(&mut game, player, "Island", Zone::Library);
    cast(&mut game, merchant, None);
    settle(&mut game);
    let food = (0..game.state.objects.len())
        .map(|id| game.object_view(id as CardId))
        .find(|object| {
            object.zone == Zone::Battlefield && object.is_token() && object.has_subtype("Food")
        })
        .unwrap()
        .id;
    activate(&mut game, food);
    let accept = Action::ChooseOptional { accept: true };
    reach(&mut game, |g| g.legal_actions().contains(&accept));
    let mut no_search = game.clone();
    act(&mut no_search, Action::ChooseOptional { accept: false });
    settle(&mut no_search);
    assert_eq!(no_search.object_view(merchant).zone, Zone::Battlefield);
    assert_eq!(no_search.state.players[player.index()].life, 23);
    act(&mut game, accept);
    act(&mut game, Action::ChooseSearch { card: land });
    act(&mut game, Action::FinishSearch);
    settle(&mut game);
    assert_eq!(game.object_view(merchant).zone, Zone::Library);
    assert_eq!(game.state.players[player.index()].life, 23);
    assert_eq!(game.object_view(land).zone, Zone::Battlefield);
    assert!(game.object_view(land).tapped);
}

#[test]
fn walltop_sentries_require_a_lesson_at_death_and_on_resolution() {
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Walltop Sentries", Zone::Battlefield);
    assert!(
        game.keywords(source)
            .contains(Keywords::REACH.union(Keywords::DEATHTOUCH))
    );
    let mut no_lesson = game.clone();
    no_lesson.move_object(source, Zone::Graveyard);
    no_lesson.prepare_triggers();
    assert!(no_lesson.state.stack.is_empty());
    let lesson = put(&mut game, player, "Firebending Lesson", Zone::Graveyard);
    game.move_object(source, Zone::Graveyard);
    game.prepare_triggers();
    let mut removed = game.clone();
    removed.move_object(lesson, Zone::Exile);
    settle(&mut removed);
    assert_eq!(removed.state.players[player.index()].life, 20);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].life, 22);
}
