use crate::support::*;
use rustmage_core::{cards::*, engine::*};

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn reach(game: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..32 {
        if ready(game) {
            return;
        }
        choose(game, |action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. })
        });
    }
    panic!("missing Firebending choice");
}

fn attack(game: &mut Game, cards: &[CardId]) {
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    for &card in cards {
        act(game, Action::Attack { card });
    }
    act(game, Action::FinishAttackers);
}

fn tap_ability(game: &mut Game, source: CardId, index: u16) {
    choose(
        game,
        |action| matches!(action, Action::ActivateMana { card, ability, .. } if *card == source && *ability == index),
    );
}

fn activate(game: &mut Game, source: CardId, index: u16) {
    choose(
        game,
        |action| matches!(action, Action::Activate { card, ability, .. } if *card == source && *ability == index),
    );
}

fn tokens(game: &Game, subtype: &str) -> Vec<ObjectView> {
    (0..game.state.objects.len())
        .map(|id| game.object_view(id as CardId))
        .filter(|object| {
            object.zone == Zone::Battlefield && object.is_token() && object.has_subtype(subtype)
        })
        .collect()
}

#[test]
fn firebender_ascension_copies_the_students_current_power_firebending() {
    let player = Player::P1;
    let mut game = scenario(player);
    let student = put(&mut game, player, "Firebending Student", Zone::Battlefield);
    let ascension = put(&mut game, player, "Firebender Ascension", Zone::Hand);
    cast(&mut game, ascension, None);
    settle(&mut game);
    assert_eq!(stats(&game, student), (2, 3));
    let soldiers = tokens(&game, "Soldier");
    assert_eq!(soldiers.len(), 1);
    assert_eq!(
        (soldiers[0].power, soldiers[0].toughness, soldiers[0].colors),
        (2, 2, 8)
    );
    game.add_counter(ascension, CounterKind::Quest, 3);
    attack(&mut game, &[student]);
    let pump = put(&mut game, player, "Giant Growth", Zone::Hand);
    cast(&mut game, pump, Some(Target::Card(student)));
    let copy = Action::ChooseOptional { accept: true };
    reach(&mut game, |g| g.legal_actions().contains(&copy));
    assert_eq!(stats(&game, student), (6, 7));
    assert_eq!(game.counter_count(ascension, CounterKind::Quest), 4);
    let mut decline = game.clone();
    act(&mut decline, Action::ChooseOptional { accept: false });
    settle(&mut decline);
    assert_eq!(decline.state.players[player.index()].combat_red_mana, 6);
    act(&mut game, copy);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].combat_red_mana, 12);
    assert_eq!(game.counter_count(ascension, CounterKind::Quest), 4);
}

#[test]
fn palace_and_cadets_supply_separate_attack_abilities() {
    let player = Player::P0;
    let mut game = scenario(player);
    let palace = put(&mut game, player, "Fire Nation Palace", Zone::Hand);
    let mut no_basic = game.clone();
    act(&mut no_basic, Action::PlayLand { card: palace });
    assert!(no_basic.object_view(palace).tapped);
    put(&mut game, player, "Mountain", Zone::Battlefield);
    act(&mut game, Action::PlayLand { card: palace });
    assert!(!game.object_view(palace).tapped);
    let mut red = game.clone();
    red.state.players[player.index()].mana = [0; 6];
    tap_ability(&mut red, palace, 0);
    assert_eq!(red.state.players[player.index()].mana, [0, 0, 0, 1, 0, 0]);

    let cadet = put(&mut game, player, "Fire Nation Cadets", Zone::Battlefield);
    let opposing = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    put(
        &mut game,
        player.other(),
        "Firebending Lesson",
        Zone::Graveyard,
    );
    activate(&mut game, cadet, 0);
    settle(&mut game);
    assert_eq!(stats(&game, cadet), (2, 2));
    act(
        &mut game,
        Action::AnnounceActivate {
            card: palace,
            ability: 1,
            counter_payment: Counters::default(),
        },
    );
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(opposing),
        },
    );
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(cadet),
        },
    );
    pay(&mut game);
    settle(&mut game);

    let mut missing = game.clone();
    attack(&mut missing, &[cadet]);
    settle(&mut missing);
    assert_eq!(missing.state.players[player.index()].combat_red_mana, 4);

    put(&mut game, player, "Firebending Lesson", Zone::Graveyard);
    attack(&mut game, &[cadet]);
    game.move_object(cadet, Zone::Graveyard);
    reach(&mut game, |g| matches!(g.state.pending, Pending::Priority));
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].combat_red_mana, 6);
}

#[test]
fn foretelling_sozins_comet_hides_it_and_the_grant_uses_the_resolution_cohort() {
    let player = Player::P0;
    let mut game = scenario(player);
    let comet = put(&mut game, player, "Sozin's Comet", Zone::Hand);
    let first = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let second = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let object = game.object_ref(comet);
    act(&mut game, Action::AnnounceForetell { object });
    choose(&mut game, |action| {
        matches!(action, Action::PayForetell { .. })
    });
    assert_eq!(game.observe(player.other()).foretold_cards[0].object, None);
    let object = game.object_ref(comet);
    assert_eq!(game.observe(player).foretold_cards[0].object, Some(object));
    let cast_comet = Action::AnnounceForetold {
        object,
        face: CastFace::Normal,
        flash_surcharge: false,
    };
    reject(&mut game, cast_comet.clone());
    game.state.turn += 1;
    game.state.players[player.index()].mana = [0, 2, 0, 1, 0, 0];
    act(&mut game, cast_comet);
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(comet).zone, Zone::Graveyard);
    let late = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    attack(&mut game, &[first, second, late]);
    reach(&mut game, |g| matches!(g.state.pending, Pending::Priority));
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].combat_red_mana, 10);
}

#[test]
fn the_soldiers_from_a_flashback_cast_have_independent_firebending() {
    let player = Player::P1;
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Fire Nation Attacks", Zone::Graveyard);
    act(&mut game, Action::AnnounceFlashback { card: spell });
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(spell).zone, Zone::Exile);
    let soldiers = tokens(&game, "Soldier");
    assert_eq!(soldiers.len(), 2);
    let ids = soldiers
        .iter()
        .map(|soldier| soldier.id)
        .collect::<Vec<_>>();
    for id in &ids {
        assert_eq!(stats(&game, *id), (2, 2));
        game.state.objects[*id as usize].summoning_sick = false;
    }
    attack(&mut game, &ids);
    reach(&mut game, |g| matches!(g.state.pending, Pending::Priority));
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].combat_red_mana, 2);
}

#[test]
fn jeong_jeongs_exhaust_tracks_one_lesson_after_the_source_leaves() {
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(
        &mut game,
        player,
        "Jeong Jeong, the Deserter",
        Zone::Battlefield,
    );
    activate(&mut game, source, 0);
    settle(&mut game);
    assert_eq!(game.counter_count(source, CounterKind::PlusOne), 1);
    assert!(!game.legal_actions().iter().any(|action| {
        matches!(action, Action::Activate { card, ability: 0, .. } if *card == source)
    }));
    let mut combat = game.clone();
    attack(&mut combat, &[source]);
    settle(&mut combat);
    assert_eq!(combat.state.players[player.index()].combat_red_mana, 1);
    game.move_object(source, Zone::Graveyard);

    let nonlesson = put(&mut game, player, "Lightning Strike", Zone::Hand);
    cast(&mut game, nonlesson, Some(Target::Player(player.other())));
    settle(&mut game);
    assert_eq!(game.delayed_spell_copy_view().len(), 1);
    let left = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    let right = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    let lesson = put(&mut game, player, "Firebending Lesson", Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: lesson });
    act(&mut game, Action::ChooseKicker { pay: false });
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(left),
        },
    );
    pay(&mut game);
    assert_eq!(game.state.stack.len(), 2);
    assert!(game.delayed_spell_copy_view().is_empty());
    pass_pair(&mut game);
    act(
        &mut game,
        Action::SetStackTarget {
            index: 0,
            target: Target::Card(right),
        },
    );
    act(&mut game, Action::FinishStackTargets);
    settle(&mut game);
    assert_eq!(game.state.objects[left as usize].damage, 2);
    assert_eq!(game.state.objects[right as usize].damage, 2);
}

#[test]
fn ran_and_shaw_need_the_cast_and_the_graveyard_condition_to_copy() {
    let player = Player::P1;
    let mut game = scenario(player);
    let dragon = put(&mut game, player, "Ran and Shaw", Zone::Hand);
    let lesson = put(&mut game, player, "Firebending Lesson", Zone::Graveyard);
    put(&mut game, player, "Firebending Lesson", Zone::Graveyard);
    put(&mut game, player, "Shivan Dragon", Zone::Graveyard);
    let companion = put(&mut game, player, "Bear Cub", Zone::Battlefield);

    let mut uncast = game.clone();
    uncast.move_object(dragon, Zone::Battlefield);
    uncast.prepare_triggers();
    settle(&mut uncast);
    assert!(tokens(&uncast, "Dragon").is_empty());

    cast(&mut game, dragon, None);
    pass_pair(&mut game);
    let mut shrunk = game.clone();
    shrunk.move_object(lesson, Zone::Exile);
    settle(&mut shrunk);
    assert!(tokens(&shrunk, "Dragon").is_empty());

    settle(&mut game);
    let copies = tokens(&game, "Dragon");
    assert_eq!(copies.len(), 1);
    let copy = copies[0].id;
    assert!(!copies[0].is_legendary());
    assert!(game.keywords(copy).contains(Keywords::FLYING));
    activate(&mut game, dragon, 0);
    settle(&mut game);
    assert_eq!(stats(&game, dragon), (6, 4));
    assert_eq!(stats(&game, copy), (6, 4));
    assert_eq!(stats(&game, companion), (2, 2));
    game.state.objects[copy as usize].summoning_sick = false;
    attack(&mut game, &[copy]);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].combat_red_mana, 2);
}

#[test]
fn the_legend_of_roku_plays_exiles_and_returns_as_its_exact_back_face() {
    let player = Player::P0;
    let mut game = scenario(player);
    let land = put(&mut game, player, "Mountain", Zone::Library);
    let exile_spell = put(&mut game, player, "Llanowar Elves", Zone::Library);
    let exile_top = put(&mut game, player, "Shock", Zone::Library);
    let saga = put(
        &mut game,
        player,
        "The Legend of Roku // Avatar Roku",
        Zone::Hand,
    );
    cast(&mut game, saga, None);
    settle(&mut game);
    for card in [land, exile_spell, exile_top] {
        assert_eq!(game.object_view(card).zone, Zone::Exile);
    }
    let front = game.object_ref(saga);
    game.state.turn += 1;
    act(&mut game, Action::PlayLand { card: land });
    game.begin_step(Step::Main1);
    game.prepare_triggers();
    game.state.players[player.index()].mana = [0; 6];
    reach(&mut game, |g| {
        g.legal_actions()
            .iter()
            .any(|action| matches!(action, Action::ChooseMana { .. }))
    });
    act(
        &mut game,
        Action::ChooseMana {
            production: [0, 1, 0, 0, 0, 0],
        },
    );
    assert_eq!(game.state.players[player.index()].mana, [0, 1, 0, 0, 0, 0]);
    settle(&mut game);

    game.begin_step(Step::Main1);
    game.prepare_triggers();
    settle(&mut game);
    assert_ne!(game.object_ref(saga), front);
    assert_eq!(
        game.object_view(saga).card,
        Card::from_name("Avatar Roku").unwrap()
    );
    assert_eq!(stats(&game, saga), (4, 4));
    game.state.players[player.index()].mana[3] = 8;
    activate(&mut game, saga, 0);
    settle(&mut game);
    let dragons = tokens(&game, "Dragon");
    assert_eq!(dragons.len(), 1);
    let dragon = dragons[0].id;
    assert_eq!(stats(&game, dragon), (4, 4));
    assert!(game.keywords(dragon).contains(Keywords::FLYING));
    for id in [saga, dragon] {
        game.state.objects[id as usize].summoning_sick = false;
    }
    attack(&mut game, &[saga, dragon]);
    reach(&mut game, |g| matches!(g.state.pending, Pending::Priority));
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].combat_red_mana, 8);
}

#[test]
fn fated_firepower_flashes_in_with_x_and_only_adds_damage_to_opposing_recipients() {
    let player = Player::P1;
    let mut game = scenario(player);
    game.state.active = player.other();
    put(&mut game, player, "Doubling Season", Zone::Battlefield);
    let firepower = put(&mut game, player, "Fated Firepower", Zone::Hand);
    let mut short_red = game.clone();
    short_red.state.players[player.index()].mana = [0, 8, 0, 2, 0, 0];
    reject(&mut short_red, Action::AnnounceCast { card: firepower });
    act(&mut game, Action::AnnounceCast { card: firepower });
    act(&mut game, Action::ChooseX { value: 2 });
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.counter_count(firepower, CounterKind::Fire), 4);
    if game.actor() != Some(player) {
        act(&mut game, Action::Pass);
    }
    let spell = put(&mut game, player, "Shock", Zone::Hand);
    let mut self_damage = game.clone();
    cast(&mut self_damage, spell, Some(Target::Player(player)));
    settle(&mut self_damage);
    assert_eq!(self_damage.state.players[player.index()].life, 18);
    cast(&mut game, spell, Some(Target::Player(player.other())));
    settle(&mut game);
    assert_eq!(game.state.players[player.other().index()].life, 14);
}
