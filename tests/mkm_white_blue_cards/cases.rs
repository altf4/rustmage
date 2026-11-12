use crate::support::*;
use rustmage_core::{cards::*, engine::*};

const CLUE: Card = Card::from_catalogue_id(5296);

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn begin_step(game: &mut Game, step: Step) {
    game.state.priority = game.state.active;
    game.begin_step(step);
    game.prepare_triggers();
}

fn solved(game: &Game, case: CardId) -> bool {
    game.observe(Player::P0)
        .solved_cases
        .contains(&game.object_ref(case))
}

fn clues(game: &Game, player: Player) -> usize {
    game.controlled_token_ids(player)
        .filter(|&id| game.object_view(id).card == CLUE)
        .count()
}

fn activate_case(game: &mut Game, case: CardId, target: Option<Target>) {
    choose(game, |action| match action {
        Action::Activate {
            card,
            ability: 0,
            target: candidate,
            ..
        } => *card == case && *candidate == target,
        Action::AnnounceActivate {
            card, ability: 0, ..
        }
        | Action::BeginActivate {
            card, ability: 0, ..
        } => *card == case,
        _ => false,
    });
    if matches!(game.state.pending, Pending::Targets { .. }) {
        act(
            game,
            Action::ChooseTarget {
                target: target.unwrap(),
            },
        );
    }
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(game);
    }
}

#[test]
fn shattered_pact_reveals_a_basic_then_needs_five_colors_at_both_solve_checks() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let pact = put(&mut game, player, "Case of the Shattered Pact", Zone::Hand);
    let basic = put(&mut game, player, "Plains", Zone::Library);
    let spell = put(&mut game, player, "Shock", Zone::Library);
    cast(&mut game, pact, None);
    pass_pair(&mut game);
    pass_pair(&mut game);
    assert!(game.observe(opponent).search_choice.is_none());
    reject(&mut game, Action::ChooseSearch { card: spell });
    act(&mut game, Action::ChooseSearch { card: basic });
    act(&mut game, Action::FinishSearch);
    assert_eq!(game.object_view(basic).zone, Zone::Hand);
    assert_eq!(game.observe(opponent).zone_reveals[0].card, Card::Plains);

    let white = put(&mut game, player, "Savannah Lions", Zone::Battlefield);
    put(
        &mut game,
        player,
        "Benthic Criminologists",
        Zone::Battlefield,
    );
    put(&mut game, player, "Burglar Rat", Zone::Battlefield);
    put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let red = put(&mut game, opponent, "Goblin Piker", Zone::Battlefield);

    begin_step(&mut game, Step::BeginCombat);
    assert!(game.state.stack.is_empty());
    begin_step(&mut game, Step::End);
    assert!(game.state.stack.is_empty());
    game.record_control(game.object_ref(red), player, None);
    begin_step(&mut game, Step::End);
    assert_eq!(game.state.stack.len(), 1);

    let mut failed = game.clone();
    failed.record_control(failed.object_ref(red), opponent, None);
    settle(&mut failed);
    assert!(!solved(&failed, pact));

    settle(&mut game);
    assert!(solved(&game, pact));
    begin_step(&mut game, Step::BeginCombat);
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(white),
        },
    );
    settle(&mut game);
    let keywords = game.object_view(white).keywords;
    assert!(keywords.contains(Keywords::FLYING));
    assert!(keywords.contains(Keywords::DOUBLE_STRIKE));
    assert!(keywords.contains(Keywords::VIGILANCE));
    assert!(!game.object_view(red).keywords.contains(Keywords::FLYING));
}

#[test]
fn auditor_keeps_the_look_private_and_observes_only_its_controllers_case_solves() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let auditor = put(&mut game, player, "Case File Auditor", Zone::Hand);
    let outside = put(&mut game, player, "Pacifism", Zone::Library);
    let looked = [
        "Island",
        "Savannah Lions",
        "Swamp",
        "Pacifism",
        "Forest",
        "Shock",
    ]
    .map(|name| put(&mut game, player, name, Zone::Library));
    cast(&mut game, auditor, None);
    pass_pair(&mut game);
    pass_pair(&mut game);
    let private = game.observe(player).library_selection.unwrap();
    assert_eq!(private.remaining.len(), 6);
    assert_eq!(private.candidates[0].card, looked[3]);
    assert_eq!(private.candidates.len(), 1);
    assert!(game.observe(opponent).library_selection.is_none());
    reject(&mut game, Action::SelectLibrary { card: outside });
    reject(&mut game, Action::SelectLibrary { card: looked[5] });

    let mut declined = game.clone();
    act(&mut declined, Action::FinishLibrarySelection);
    assert!(declined.observe(opponent).zone_reveals.is_empty());
    assert_eq!(
        *declined.state.players[player.index()]
            .library
            .last()
            .unwrap(),
        outside
    );

    act(&mut game, Action::SelectLibrary { card: looked[3] });
    assert!(game.observe(opponent).zone_reveals.is_empty());
    let mut indexed = game.clone();
    let index = indexed
        .legal_actions()
        .iter()
        .position(|action| *action == Action::FinishLibrarySelection)
        .unwrap();
    indexed
        .apply_index(player, indexed.revision(), index)
        .unwrap();
    act(&mut game, Action::FinishLibrarySelection);
    assert_eq!(game.snapshot(), indexed.snapshot());
    assert_eq!(game.object_view(looked[3]).zone, Zone::Hand);
    let public = game.observe(opponent);
    assert_eq!(public.known_hand, [game.object_ref(looked[3])]);
    assert!(public.objects.iter().any(|object| {
        object.reference == game.object_ref(looked[3])
            && object.card == Card::from_name("Pacifism").unwrap()
    }));
    let mut bottom = game.state.players[player.index()].library[..5].to_vec();
    bottom.sort_unstable();
    assert_eq!(
        bottom,
        [looked[0], looked[1], looked[2], looked[4], looked[5]]
    );

    let cases = [
        put(
            &mut game,
            player,
            "Case of the Filched Falcon",
            Zone::Battlefield,
        ),
        put(
            &mut game,
            player,
            "Case of the Filched Falcon",
            Zone::Battlefield,
        ),
    ];
    let opposing_case = put(
        &mut game,
        opponent,
        "Case of the Filched Falcon",
        Zone::Battlefield,
    );
    game.create_tokens(player, CLUE, 3, false);
    game.create_tokens(opponent, CLUE, 3, false);
    for _ in 0..12 {
        put(&mut game, player, "Pacifism", Zone::Library);
    }
    begin_step(&mut game, Step::End);
    while matches!(game.state.pending, Pending::OrderTriggers { .. }) {
        choose(&mut game, |action| !matches!(action, Action::Concede));
    }
    let mut audits = 0;
    for _ in 0..12 {
        if game.state.stack.is_empty() && game.state.pending == Pending::Priority {
            break;
        }
        if game.observe(player).library_selection.is_some() {
            act(&mut game, Action::FinishLibrarySelection);
            audits += 1;
        } else {
            act(&mut game, Action::Pass);
        }
    }
    assert_eq!(audits, 2);
    assert!(cases.into_iter().all(|case| solved(&game, case)));
    game.state.active = opponent;
    begin_step(&mut game, Step::End);
    settle(&mut game);
    assert!(solved(&game, opposing_case));

    game.state.active = player;
    begin_step(&mut game, Step::Main1);
    let case = put(&mut game, player, "Case of the Pilfered Proof", Zone::Hand);
    let lion = put(&mut game, player, "Savannah Lions", Zone::Hand);
    let colorless = [0, 0, 0, 0, 0, 2];
    game.state.players[player.index()].mana = colorless;
    reject(&mut game, Action::AnnounceCast { card: lion });
    act(
        &mut game,
        Action::Cast {
            card: case,
            target: None,
            payment: colorless,
        },
    );
    settle(&mut game);
    assert_eq!(game.object_view(case).zone, Zone::Battlefield);
}

#[test]
fn gateway_express_deals_damage_through_each_creature_and_rewards_actual_attackers() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let gateway = put(&mut game, player, "Case of the Gateway Express", Zone::Hand);
    let hawk = put(&mut game, player, "Healer's Hawk", Zone::Battlefield);
    let deathtouch = put(&mut game, player, "Fynn, the Fangbearer", Zone::Battlefield);
    let bear = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let target = put(&mut game, opponent, "Gigantosaurus", Zone::Battlefield);
    cast(&mut game, gateway, None);
    pass_pair(&mut game);
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(bear),
        },
    );
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    settle(&mut game);
    assert_eq!(game.object_view(target).zone, Zone::Graveyard);
    assert_eq!(game.state.players[player.index()].life, 21);
    for source in [hawk, deathtouch, bear] {
        assert!(game.object_view(source).dealt_damage_this_turn);
    }
    assert_eq!(stats(&game, bear), (2, 2));

    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    for card in [hawk, deathtouch, bear] {
        act(&mut game, Action::Attack { card });
    }
    act(&mut game, Action::FinishAttackers);
    settle(&mut game);
    game.state.priority = opponent;
    let shock = put(&mut game, opponent, "Shock", Zone::Hand);
    cast(&mut game, shock, Some(Target::Card(hawk)));
    settle(&mut game);
    assert_eq!(game.object_view(hawk).zone, Zone::Graveyard);
    begin_step(&mut game, Step::End);
    settle(&mut game);
    assert!(solved(&game, gateway));
    assert_eq!(stats(&game, bear), (3, 2));
    let newcomer = put(&mut game, player, "Savannah Lions", Zone::Battlefield);
    assert_eq!(stats(&game, newcomer), (3, 1));
}

#[test]
fn pilfered_proof_counters_detectives_and_adds_a_clue_only_after_solving() {
    let player = Player::P0;
    let mut game = scenario(player);
    let proof = put(&mut game, player, "Case of the Pilfered Proof", Zone::Hand);
    cast(&mut game, proof, None);
    settle(&mut game);
    for _ in 0..2 {
        put(&mut game, player, "Seasoned Consultant", Zone::Battlefield);
    }
    let enemy = put(
        &mut game,
        player.other(),
        "Seasoned Consultant",
        Zone::Battlefield,
    );
    let first = put(&mut game, player, "Case of the Filched Falcon", Zone::Hand);
    cast(&mut game, first, None);
    settle(&mut game);
    assert_eq!(clues(&game, player), 1);
    let entrant = put(&mut game, player, "Seasoned Consultant", Zone::Hand);
    cast(&mut game, entrant, None);
    settle(&mut game);
    assert_eq!(game.counter_count(entrant, CounterKind::PlusOne), 1);
    assert_eq!(game.counter_count(enemy, CounterKind::PlusOne), 0);
    begin_step(&mut game, Step::End);
    settle(&mut game);
    assert!(solved(&game, proof));

    begin_step(&mut game, Step::Main1);
    let second = put(&mut game, player, "Case of the Filched Falcon", Zone::Hand);
    cast(&mut game, second, None);
    settle(&mut game);
    assert_eq!(clues(&game, player), 3);
}

#[test]
fn filched_falcon_investigates_and_animates_an_opposing_noncreature_artifact() {
    let player = Player::P0;
    let mut game = scenario(player);
    let case = put(&mut game, player, "Case of the Filched Falcon", Zone::Hand);
    let target = put(&mut game, player.other(), "Candlestick", Zone::Battlefield);
    cast(&mut game, case, None);
    settle(&mut game);
    assert_eq!(clues(&game, player), 1);
    reject(
        &mut game,
        Action::AnnounceActivate {
            card: case,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    let creature = put(&mut game, player, "Gingerbrute", Zone::Battlefield);
    put(&mut game, player, "Candlestick", Zone::Battlefield);
    begin_step(&mut game, Step::End);
    settle(&mut game);
    assert!(solved(&game, case));
    assert!(
        !game
            .legal_actions()
            .iter()
            .any(|action| matches!(action, Action::Activate {
        card, ability: 0, target: Some(Target::Card(recipient)), ..
    } if *card == case && *recipient == creature))
    );

    activate_case(&mut game, case, Some(Target::Card(target)));
    assert_eq!(game.object_view(case).zone, Zone::Graveyard);
    settle(&mut game);
    let bird = game.object_view(target);
    assert_eq!(bird.controller, player.other());
    assert_eq!(bird.kinds & (1 | 16), 1 | 16);
    assert!(bird.has_subtype("Bird"));
    assert!(bird.has_subtype("Equipment"));
    assert!(bird.keywords.contains(Keywords::FLYING));
    assert_eq!(stats(&game, target), (4, 4));
}

#[test]
fn ransacked_lab_counts_casts_from_before_entry_and_draws_only_after_solving() {
    let player = Player::P0;
    let mut game = scenario(player);
    for _ in 0..2 {
        let shock = put(&mut game, player, "Shock", Zone::Hand);
        cast(&mut game, shock, Some(Target::Player(player.other())));
        settle(&mut game);
    }
    let lab = put(&mut game, player, "Case of the Ransacked Lab", Zone::Hand);
    cast(&mut game, lab, None);
    settle(&mut game);
    let divination = put(&mut game, player, "Divination", Zone::Hand);
    let reduced = [0, 1, 0, 0, 0, 1];
    game.state.players[player.index()].mana = reduced;
    let library_size = game.state.players[player.index()].library.len();
    act(
        &mut game,
        Action::Cast {
            card: divination,
            target: None,
            payment: reduced,
        },
    );
    settle(&mut game);
    assert_eq!(
        game.state.players[player.index()].library.len(),
        library_size - 2
    );
    let shock = put(&mut game, player, "Shock", Zone::Hand);
    game.state.players[player.index()].mana = [8; 6];
    cast(&mut game, shock, Some(Target::Player(player.other())));
    settle(&mut game);
    begin_step(&mut game, Step::End);
    settle(&mut game);
    assert!(solved(&game, lab));

    let shock = put(&mut game, player, "Shock", Zone::Hand);
    cast(&mut game, shock, Some(Target::Player(player.other())));
    assert_eq!(game.state.stack.len(), 2);
    pass_pair(&mut game);
    assert_eq!(
        game.state.players[player.index()].library.len(),
        library_size - 3
    );
    assert_eq!(game.object_view(shock).zone, Zone::Stack);
}

#[test]
fn uneaten_feast_grants_only_the_creature_cards_in_the_graveyard_at_resolution() {
    let player = Player::P0;
    let mut game = scenario(player);
    let feast = put(&mut game, player, "Case of the Uneaten Feast", Zone::Hand);
    cast(&mut game, feast, None);
    settle(&mut game);
    for _ in 0..5 {
        let bear = put(&mut game, player, "Grizzly Bears", Zone::Hand);
        cast(&mut game, bear, None);
        settle(&mut game);
    }
    assert_eq!(game.state.players[player.index()].life, 25);
    begin_step(&mut game, Step::End);
    settle(&mut game);
    assert!(solved(&game, feast));
    let bear = put(&mut game, player, "Grizzly Bears", Zone::Graveyard);
    let shock = put(&mut game, player, "Shock", Zone::Graveyard);
    let enemy = put(&mut game, player.other(), "Grizzly Bears", Zone::Graveyard);
    activate_case(&mut game, feast, None);
    assert_eq!(game.object_view(feast).zone, Zone::Graveyard);
    settle(&mut game);
    let later = put(&mut game, player, "Savannah Lions", Zone::Graveyard);
    let granted = game.observe(player).graveyard_cast_permissions;
    assert_eq!(granted.len(), 1);
    assert_eq!(granted[0].object, game.object_ref(bear));
    begin_step(&mut game, Step::Main1);
    let permission = game
        .legal_actions()
        .into_iter()
        .find(|action| {
            matches!(
                action, Action::AnnounceCast { card } if *card == bear
            )
        })
        .unwrap();
    for id in [shock, later, enemy] {
        assert!(!game.legal_actions().iter().any(|action| matches!(
            action, Action::AnnounceCast { card } if *card == id
        )));
    }
    act(&mut game, permission);
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(bear).zone, Zone::Battlefield);
}
