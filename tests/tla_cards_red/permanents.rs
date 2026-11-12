use crate::{reach, support::*};
use rustmage_core::{cards::*, engine::*};

fn activate(source: CardId, ability: u16) -> Action {
    Action::AnnounceActivate {
        card: source,
        ability,
        counter_payment: Counters::default(),
    }
}

fn attack(game: &mut Game, source: CardId) {
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    act(game, Action::Attack { card: source });
    act(game, Action::FinishAttackers);
}

fn target(game: &mut Game, card: CardId) {
    act(
        game,
        Action::ChooseTarget {
            target: Target::Card(card),
        },
    );
}

#[test]
fn combustion_man_leaves_the_choice_to_the_targets_controller_and_uses_last_power() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let man = put(&mut game, player, "Combustion Man", Zone::Battlefield);
    let land = put(&mut game, opponent, "Mountain", Zone::Battlefield);
    attack(&mut game, man);
    target(&mut game, land);
    let mut destroy = game.clone();
    reach(&mut destroy, |g| {
        g.legal_actions()
            .contains(&Action::ChooseOptional { accept: false })
    });
    assert_eq!(destroy.actor(), Some(opponent));
    act(&mut destroy, Action::ChooseOptional { accept: false });
    settle(&mut destroy);
    assert_eq!(destroy.object_view(land).zone, Zone::Graveyard);
    assert_eq!(destroy.state.players[opponent.index()].life, 20);

    game.add_counter(man, CounterKind::PlusOne, 2);
    game.move_object(man, Zone::Graveyard);
    let accept = Action::ChooseOptional { accept: true };
    reach(&mut game, |g| g.legal_actions().contains(&accept));
    assert_eq!(game.actor(), Some(opponent));
    act(&mut game, accept);
    settle(&mut game);
    assert_eq!(game.object_view(land).zone, Zone::Battlefield);
    assert_eq!(game.state.players[opponent.index()].life, 14);
}

#[test]
fn deserters_disciple_checks_other_controller_and_current_power() {
    let player = Player::P1;
    let mut game = scenario(player);
    let disciple = put(&mut game, player, "Deserter's Disciple", Zone::Battlefield);
    let ally = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let big = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let opponent = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    act(&mut game, activate(disciple, 0));
    for card in [disciple, big, opponent] {
        reject(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(card),
            },
        );
    }
    target(&mut game, ally);
    pay(&mut game);
    let mut invalid = game.clone();
    invalid.add_counter(ally, CounterKind::PlusOne, 1);
    settle(&mut invalid);
    assert!(!invalid.keywords(ally).contains(Keywords::CANT_BE_BLOCKED));
    settle(&mut game);
    assert!(game.keywords(ally).contains(Keywords::CANT_BE_BLOCKED));
    assert!(game.object_view(disciple).tapped);
}

#[test]
fn mai_exhausts_for_a_double_strike_counter_and_has_separate_prowess() {
    let player = Player::P0;
    let mut game = scenario(player);
    let mai = put(&mut game, player, "Mai, Jaded Edge", Zone::Battlefield);
    let spell = put(&mut game, player, "Shock", Zone::Hand);
    cast(&mut game, spell, Some(Target::Player(player.other())));
    settle(&mut game);
    assert_eq!(stats(&game, mai), (2, 4));
    let action = activate(mai, 0);
    act(&mut game, action.clone());
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.counter_count(mai, CounterKind::DoubleStrike), 1);
    assert!(game.keywords(mai).contains(Keywords::DOUBLE_STRIKE));
    reject(&mut game, action);
    game.state.step = Step::End;
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    assert_eq!(stats(&game, mai), (1, 3));
    assert!(game.keywords(mai).contains(Keywords::DOUBLE_STRIKE));
}

#[test]
fn ty_lee_pays_before_the_reflexive_target_and_keeps_prowess_separate() {
    let player = Player::P1;
    let mut game = scenario(player);
    let ty = put(
        &mut game,
        player,
        "Ty Lee, Artful Acrobat",
        Zone::Battlefield,
    );
    let enemy = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    let spell = put(&mut game, player, "Shock", Zone::Hand);
    cast(&mut game, spell, Some(Target::Player(player.other())));
    settle(&mut game);
    assert_eq!(stats(&game, ty), (4, 3));
    attack(&mut game, ty);
    let decline = Action::DeclineResolutionMana;
    reach(&mut game, |g| g.legal_actions().contains(&decline));
    let mut no_pay = game.clone();
    act(&mut no_pay, decline);
    settle(&mut no_pay);
    assert!(!no_pay.keywords(enemy).contains(Keywords::CANT_BLOCK));
    choose(&mut game, |action| {
        matches!(action, Action::PayResolutionMana { .. })
    });
    target(&mut game, enemy);
    settle(&mut game);
    assert!(game.keywords(enemy).contains(Keywords::CANT_BLOCK));
}

#[test]
fn war_balloon_can_be_crewed_or_remain_a_creature_while_it_has_three_fire_counters() {
    let player = Player::P0;
    let mut game = scenario(player);
    let balloon = put(&mut game, player, "War Balloon", Zone::Battlefield);
    let crew = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    assert!(!game.object_view(balloon).is_creature());
    let mut crewed = game.clone();
    act(&mut crewed, activate(balloon, 1));
    if matches!(crewed.state.pending, Pending::PayCosts { .. }) {
        pay(&mut crewed);
    }
    let object = crewed.object_ref(crew);
    act(&mut crewed, Action::ChooseCrewer { object });
    act(&mut crewed, Action::FinishCrew);
    settle(&mut crewed);
    assert!(crewed.object_view(balloon).is_creature());
    assert_eq!(stats(&crewed, balloon), (4, 3));
    assert!(crewed.keywords(balloon).contains(Keywords::FLYING));
    crewed.state.step = Step::End;
    act(&mut crewed, Action::Pass);
    act(&mut crewed, Action::Pass);
    assert!(!crewed.object_view(balloon).is_creature());

    for count in 1..=3 {
        act(&mut game, activate(balloon, 0));
        pay(&mut game);
        settle(&mut game);
        assert_eq!(game.counter_count(balloon, CounterKind::Fire), count);
        assert_eq!(game.object_view(balloon).is_creature(), count == 3);
    }
    assert_eq!(game.object_view(balloon).kinds & 17, 17);
    assert!(game.object_view(balloon).has_subtype("Vehicle"));
    assert_eq!(stats(&game, balloon), (4, 3));
    assert!(game.keywords(balloon).contains(Keywords::FLYING));
}

#[test]
fn zhao_sets_nonbasic_land_types_only_after_the_conqueror_counter() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let zhao = put(
        &mut game,
        player,
        "Zhao, the Moon Slayer",
        Zone::Battlefield,
    );
    let own = put(&mut game, player, "Steam Vents", Zone::Hand);
    let enemy = put(&mut game, opponent, "Overgrown Tomb", Zone::Hand);
    let basic = put(&mut game, player, "Island", Zone::Battlefield);
    assert!(game.keywords(zhao).contains(Keywords::MENACE));
    act(&mut game, Action::PlayLand { card: own });
    if game
        .legal_actions()
        .contains(&Action::ChooseEntryLife { pay: true })
    {
        act(&mut game, Action::ChooseEntryLife { pay: true });
    }
    assert!(game.object_view(own).tapped);
    assert!(game.object_view(own).has_subtype("Island"));
    act(&mut game, activate(zhao, 0));
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.counter_count(zhao, CounterKind::Conqueror), 1);
    assert!(!game.object_view(own).has_subtype("Island"));
    assert!(game.object_view(own).has_subtype("Mountain"));
    assert!(game.object_view(basic).has_subtype("Island"));

    game.state.active = opponent;
    game.state.priority = opponent;
    game.state.land_played = false;
    act(&mut game, Action::PlayLand { card: enemy });
    assert!(game.object_view(enemy).tapped);
    assert!(!game.object_view(enemy).has_subtype("Swamp"));
    assert!(game.object_view(enemy).has_subtype("Mountain"));
    game.state.objects[enemy as usize].tapped = false;
    game.state.players[opponent.index()].mana = [0; 6];
    let grantor = game.object_ref(enemy);
    act(
        &mut game,
        Action::TapGrantedMana {
            card: enemy,
            grantor,
            production: [0, 0, 0, 1, 0, 0],
        },
    );
    assert_eq!(
        game.state.players[opponent.index()].mana,
        [0, 0, 0, 1, 0, 0]
    );
    game.move_object(zhao, Zone::Graveyard);
    assert!(game.object_view(own).has_subtype("Island"));
    assert!(game.object_view(enemy).has_subtype("Swamp"));
}
