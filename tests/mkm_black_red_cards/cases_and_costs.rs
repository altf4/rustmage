use super::{activation, additional, enter_step, pass_pair, target, tokens};
use crate::support::*;
use rustmage_core::{cards::*, engine::*};

fn solved(game: &Game, player: Player, case: CardId) -> bool {
    game.observe(player)
        .solved_cases
        .contains(&game.object_ref(case))
}

#[test]
fn gorgons_kiss_destroys_a_damaged_creature_and_counts_graveyards_of_both_players() {
    let player = Player::P0;
    let mut game = scenario(player);
    let case = put(&mut game, player, "Case of the Gorgon's Kiss", Zone::Hand);
    let damaged = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    let untouched = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let shock = put(&mut game, player, "Shock", Zone::Hand);
    let murder = put(&mut game, player, "Murder", Zone::Hand);
    let discard = put(&mut game, player, "Grizzly Bears", Zone::Hand);
    let answers = put(&mut game, player, "Demand Answers", Zone::Hand);

    cast(&mut game, shock, Some(Target::Card(damaged)));
    settle(&mut game);
    cast(&mut game, case, None);
    pass_pair(&mut game);
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(untouched),
        },
    );
    let mut no_target = game.clone();
    act(&mut no_target, Action::FinishTargets);
    settle(&mut no_target);
    assert!(no_target.on_battlefield(damaged));
    target(&mut game, damaged);
    settle(&mut game);
    assert_eq!(game.object_view(damaged).zone, Zone::Graveyard);
    assert_eq!(game.object_view(case).kinds & 33, 32);

    cast(&mut game, murder, Some(Target::Card(untouched)));
    settle(&mut game);
    let mut short = game.clone();
    enter_step(&mut short, Step::End);
    settle(&mut short);
    assert!(!solved(&short, player, case));

    act(&mut game, Action::AnnounceCast { card: answers });
    additional(&mut game, discard);
    settle(&mut game);
    assert_eq!(game.object_view(discard).zone, Zone::Graveyard);
    enter_step(&mut game, Step::End);
    settle(&mut game);
    assert!(solved(&game, player, case));
    assert_eq!(game.object_view(case).kinds & 33, 33);
    assert!(game.has_subtype(case, "Gorgon"));
    assert!(game.has_subtype(case, "Case"));
    assert_eq!(stats(&game, case), (4, 4));
    assert!(
        game.keywords(case)
            .contains(Keywords::DEATHTOUCH.union(Keywords::LIFELINK))
    );
}

#[test]
fn stashed_skeleton_requires_no_suspected_skeleton_and_tutors_at_sorcery_speed() {
    const SKELETON: Card = Card::from_catalogue_id(5681);
    let player = Player::P1;
    let mut game = scenario(player);
    let case = put(
        &mut game,
        player,
        "Case of the Stashed Skeleton",
        Zone::Hand,
    );
    let person = put(&mut game, player, "Person of Interest", Zone::Hand);
    let shock = put(&mut game, player, "Shock", Zone::Hand);
    let found = put(&mut game, player, "Hill Giant", Zone::Library);

    cast(&mut game, case, None);
    settle(&mut game);
    let [skeleton] = tokens(&game, player, SKELETON).try_into().unwrap();
    assert_eq!(stats(&game, skeleton), (2, 1));
    assert_eq!(game.object_view(skeleton).colors, 4);
    assert!(game.has_subtype(skeleton, "Skeleton"));
    assert!(
        game.observe(player.other())
            .suspected
            .contains(&game.object_ref(skeleton))
    );
    reject(&mut game, activation(case));

    let mut suspected_skeleton = game.clone();
    enter_step(&mut suspected_skeleton, Step::End);
    settle(&mut suspected_skeleton);
    assert!(!solved(&suspected_skeleton, player, case));

    cast(&mut game, person, None);
    settle(&mut game);
    cast(&mut game, shock, Some(Target::Card(skeleton)));
    settle(&mut game);
    game.create_tokens(player, SKELETON, 1, false);
    assert_eq!(game.observe(player).suspected, [game.object_ref(person)]);
    enter_step(&mut game, Step::End);
    settle(&mut game);
    assert!(solved(&game, player, case));
    reject(&mut game, activation(case));

    enter_step(&mut game, Step::Main2);
    act(&mut game, activation(case));
    pay(&mut game);
    assert_eq!(game.object_view(case).zone, Zone::Graveyard);
    pass_pair(&mut game);
    assert!(matches!(game.state.pending, Pending::EffectSearch { .. }));
    reject(&mut game, Action::FinishSearch);
    act(&mut game, Action::ChooseSearch { card: found });
    act(&mut game, Action::FinishSearch);
    settle(&mut game);
    assert_eq!(game.object_view(found).zone, Zone::Hand);
}

#[test]
fn burning_masks_uses_three_damage_sources_and_grants_only_one_paid_play() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let case = put(&mut game, player, "Case of the Burning Masks", Zone::Hand);
    let opposing = put(&mut game, opponent, "Hill Giant", Zone::Battlefield);
    let controlled = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let first = put(&mut game, player, "Shock", Zone::Hand);
    let second = put(&mut game, player, "Shock", Zone::Hand);
    let other_exile = put(&mut game, player, "Shock", Zone::Library);
    let unused = put(&mut game, player, "Forest", Zone::Library);
    let playable = put(&mut game, player, "Shock", Zone::Library);

    cast(&mut game, case, None);
    pass_pair(&mut game);
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(controlled),
        },
    );
    target(&mut game, opposing);
    settle(&mut game);
    assert_eq!(game.object_view(opposing).zone, Zone::Graveyard);
    cast(&mut game, first, Some(Target::Player(opponent)));
    settle(&mut game);

    let mut short = game.clone();
    enter_step(&mut short, Step::End);
    settle(&mut short);
    assert!(!solved(&short, player, case));
    cast(&mut game, second, Some(Target::Player(opponent)));
    settle(&mut game);
    enter_step(&mut game, Step::End);
    settle(&mut game);
    assert!(solved(&game, player, case));

    act(&mut game, activation(case));
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(&mut game);
    }
    pass_pair(&mut game);
    assert_eq!(game.object_view(case).zone, Zone::Graveyard);
    for card in [other_exile, unused, playable] {
        assert_eq!(game.object_view(card).zone, Zone::Exile);
    }
    reject(&mut game, Action::FinishMilledSelection);
    let object = game.object_ref(playable);
    act(&mut game, Action::SelectMilled { object });
    reject(&mut game, Action::AnnounceExile { card: other_exile });
    let mana = game.state.players[player.index()].mana;
    let mut unpaid = game.clone();
    unpaid.state.players[player.index()].mana = [0; 6];
    reject(&mut unpaid, Action::AnnounceExile { card: playable });
    act(&mut game, Action::AnnounceExile { card: playable });
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(opponent),
        },
    );
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].mana[3], mana[3] - 1);
    assert_eq!(game.observe(opponent).players[opponent.index()].life, 14);
}

#[test]
fn crimson_pulse_rechecks_the_empty_hand_and_its_upkeep_reward_survives_departure() {
    let player = Player::P0;
    let mut game = scenario(player);
    let case = put(&mut game, player, "Case of the Crimson Pulse", Zone::Hand);
    let trail = put(&mut game, player, "Candy Trail", Zone::Battlefield);
    let first = put(&mut game, player, "Shock", Zone::Library);
    let second = put(&mut game, player, "Shock", Zone::Library);
    let discard = put(&mut game, player, "Hill Giant", Zone::Hand);
    let disenchant = put(&mut game, player.other(), "Disenchant", Zone::Hand);
    let mut empty_entry = game.clone();
    empty_entry.move_object(discard, Zone::Library);
    cast(&mut empty_entry, case, None);
    settle(&mut empty_entry);
    assert_eq!(
        empty_entry.state.players[player.index()].cards_drawn_this_turn,
        2
    );

    cast(&mut game, case, None);
    pass_pair(&mut game);
    pass_pair(&mut game);
    act(&mut game, Action::Discard { card: discard });
    settle(&mut game);
    assert_eq!(game.object_view(discard).zone, Zone::Graveyard);
    for shock in [first, second] {
        assert_eq!(game.object_view(shock).zone, Zone::Hand);
        cast(&mut game, shock, Some(Target::Player(player.other())));
        settle(&mut game);
    }
    enter_step(&mut game, Step::End);
    let mut changed = game.clone();
    act(&mut changed, activation(trail));
    pay(&mut changed);
    settle(&mut changed);
    assert!(!solved(&changed, player, case));

    settle(&mut game);
    assert!(solved(&game, player, case));
    let old = [
        put(&mut game, player, "Forest", Zone::Hand),
        put(&mut game, player, "Mountain", Zone::Hand),
    ];
    let new = [
        put(&mut game, player, "Bear Cub", Zone::Library),
        put(&mut game, player, "Grizzly Bears", Zone::Library),
    ];
    enter_step(&mut game, Step::Upkeep);
    act(&mut game, Action::Pass);
    cast(&mut game, disenchant, Some(Target::Card(case)));
    pass_pair(&mut game);
    pass_pair(&mut game);
    for card in old {
        act(&mut game, Action::Discard { card });
    }
    settle(&mut game);
    assert_eq!(game.object_view(case).zone, Zone::Graveyard);
    assert!(
        old.into_iter()
            .all(|card| game.object_view(card).zone == Zone::Graveyard)
    );
    assert!(
        new.into_iter()
            .all(|card| game.object_view(card).zone == Zone::Hand)
    );
}

#[test]
fn macabre_reconstruction_uses_arrival_history_and_keeps_its_other_legal_target() {
    let player = Player::P0;
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Macabre Reconstruction", Zone::Hand);
    let old = put(&mut game, player, "Hill Giant", Zone::Graveyard);
    let new = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let foreign = put(&mut game, player.other(), "Grizzly Bears", Zone::Graveyard);
    let ooze = put(
        &mut game,
        player.other(),
        "Scavenging Ooze",
        Zone::Battlefield,
    );
    let shock = put(&mut game, player, "Shock", Zone::Hand);
    game.state.players[player.index()].mana = [0, 0, 1, 1, 0, 1];
    reject(&mut game, Action::AnnounceCast { card: spell });
    cast(&mut game, shock, Some(Target::Card(new)));
    settle(&mut game);

    act(&mut game, Action::AnnounceCast { card: spell });
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(foreign),
        },
    );
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(new),
        },
    );
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(old),
        },
    );
    pay(&mut game);
    let mut full = game.clone();
    settle(&mut full);
    assert_eq!(full.object_view(new).zone, Zone::Hand);
    assert_eq!(full.object_view(old).zone, Zone::Hand);
    assert_eq!(full.state.players[player.index()].mana, [0; 6]);

    act(&mut game, Action::Pass);
    act(&mut game, activation(ooze));
    target(&mut game, new);
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(new).zone, Zone::Exile);
    assert_eq!(game.object_view(old).zone, Zone::Hand);
}

#[test]
fn gearbane_orangutan_can_destroy_an_artifact_or_pay_the_required_sacrifice() {
    let player = Player::P0;
    let mut game = scenario(player);
    let creature = put(&mut game, player, "Gearbane Orangutan", Zone::Hand);
    let foreign = put(&mut game, player.other(), "Candy Trail", Zone::Battlefield);
    let mut no_payment = game.clone();
    cast(&mut no_payment, creature, None);
    pass_pair(&mut no_payment);
    act(&mut no_payment, Action::ChooseMode { mode: 1 });
    settle(&mut no_payment);
    assert_eq!(stats(&no_payment, creature), (2, 2));

    let owned = put(&mut game, player, "Candy Trail", Zone::Battlefield);
    cast(&mut game, creature, None);
    pass_pair(&mut game);
    let mut destroy = game.clone();
    act(&mut destroy, Action::ChooseMode { mode: 0 });
    target(&mut destroy, foreign);
    settle(&mut destroy);
    assert_eq!(destroy.object_view(foreign).zone, Zone::Graveyard);
    assert!(destroy.on_battlefield(owned));
    assert_eq!(stats(&destroy, creature), (2, 2));

    act(&mut game, Action::ChooseMode { mode: 1 });
    pass_pair(&mut game);
    reject(&mut game, Action::DeclineEffectPayment);
    let opponent = game.object_ref(foreign);
    reject(&mut game, Action::Sacrifice { object: opponent });
    let object = game.object_ref(owned);
    act(&mut game, Action::Sacrifice { object });
    settle(&mut game);
    assert_eq!(game.object_view(owned).zone, Zone::Graveyard);
    assert!(game.on_battlefield(foreign));
    assert_eq!(game.counter_count(creature, CounterKind::PlusOne), 2);
    assert_eq!(stats(&game, creature), (4, 4));
    assert!(game.keywords(creature).contains(Keywords::REACH));
}

#[test]
fn demand_answers_requires_a_distinct_cost_and_draws_after_either_payment() {
    let player = Player::P1;
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Demand Answers", Zone::Hand);
    let creature = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let announce = Action::AnnounceCast { card: spell };
    reject(&mut game, announce.clone());

    let treasure = Card::from_catalogue_id(5302);
    game.create_tokens(player, treasure, 1, false);
    let [token] = tokens(&game, player, treasure).try_into().unwrap();
    game.state.players[player.index()].mana = [0, 0, 0, 1, 0, 0];
    reject(&mut game, announce.clone());
    game.state.players[player.index()].mana[5] = 1;

    let hand = put(&mut game, player, "Forest", Zone::Hand);
    act(&mut game, announce);
    let invalid = game.object_ref(creature);
    reject(&mut game, Action::ChooseAdditionalCost { object: invalid });
    let mut discard = game.clone();
    additional(&mut discard, hand);
    settle(&mut discard);
    assert_eq!(discard.object_view(hand).zone, Zone::Graveyard);
    assert!(discard.on_battlefield(token));
    assert_eq!(
        discard.state.players[player.index()].cards_drawn_this_turn,
        2
    );

    additional(&mut game, token);
    settle(&mut game);
    assert_ne!(game.object_view(token).zone, Zone::Battlefield);
    assert_eq!(game.object_view(hand).zone, Zone::Hand);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 2);
}

#[test]
fn suspicious_detonation_counts_an_exiled_sacrifice_and_ignores_ward() {
    let player = Player::P0;
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Suspicious Detonation", Zone::Hand);
    let warded = put(&mut game, player.other(), "Vein Ripper", Zone::Battlefield);
    let sacrificial_creature = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    put(
        &mut game,
        player.other(),
        "Rest in Peace",
        Zone::Battlefield,
    );
    let treasure = Card::from_catalogue_id(5302);
    game.create_tokens(player, treasure, 1, false);
    let [token] = tokens(&game, player, treasure).try_into().unwrap();
    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 1];

    reject(&mut game, Action::AnnounceCast { card: spell });
    act(
        &mut game,
        Action::ActivateMana {
            card: token,
            ability: 0,
            payment: [0; 6],
            counter_payment: Counters::default(),
            production: [0, 0, 0, 1, 0, 0],
        },
    );
    assert_ne!(game.object_view(token).zone, Zone::Battlefield);
    cast(&mut game, spell, Some(Target::Card(warded)));
    pass_pair(&mut game);
    if game.legal_actions().contains(&Action::DeclineEffectPayment) {
        act(&mut game, Action::DeclineEffectPayment);
    }
    settle(&mut game);
    assert_eq!(game.object_view(warded).damage, 4);
    assert!(game.on_battlefield(sacrificial_creature));
}

#[test]
fn agency_coroner_counts_the_paid_creatures_designation_not_its_keywords() {
    let player = Player::P1;
    let mut game = scenario(player);
    let coroner = put(&mut game, player, "Agency Coroner", Zone::Battlefield);
    let marked = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let menace = put(
        &mut game,
        player,
        "Prowler, Clawed Thief",
        Zone::Battlefield,
    );
    let mark = game.object_ref(marked);
    game.set_suspected(mark, true);
    let protection = put(&mut game, player, "Witness Protection", Zone::Hand);
    cast(&mut game, protection, Some(Target::Card(marked)));
    settle(&mut game);
    assert!(!game.keywords(marked).contains(Keywords::MENACE));
    assert!(game.keywords(menace).contains(Keywords::MENACE));

    act(&mut game, activation(coroner));
    let source = game.object_ref(coroner);
    reject(&mut game, Action::ChooseAdditionalCost { object: source });
    let mut one = game.clone();
    additional(&mut one, menace);
    settle(&mut one);
    assert_eq!(one.state.players[player.index()].cards_drawn_this_turn, 1);
    assert_eq!(one.object_view(menace).zone, Zone::Graveyard);

    additional(&mut game, marked);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 2);
    assert_eq!(game.object_view(marked).zone, Zone::Graveyard);
}
