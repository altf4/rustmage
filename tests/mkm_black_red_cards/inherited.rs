use super::{activation, activation_at, begin_attack, enter_step, life, pass_pair, target, tokens};
use crate::support::*;
use rustmage_core::{cards::*, engine::*};

const CLUE: Card = Card::from_catalogue_id(5296);

fn target_player(game: &mut Game, player: Player) {
    act(
        game,
        Action::ChooseTarget {
            target: Target::Player(player),
        },
    );
}

fn collect_cost(game: &mut Game, spell: CardId, fuel: Option<CardId>) {
    act(game, Action::AnnounceCast { card: spell });
    act(
        game,
        Action::ChooseCollectEvidence {
            collect: fuel.is_some(),
        },
    );
    if let Some(card) = fuel {
        if game.legal_actions().contains(&Action::BeginOrderedCosts) {
            act(game, Action::BeginOrderedCosts);
            let clauses = &game
                .state
                .announcing
                .as_ref()
                .unwrap()
                .cast_costs
                .as_ref()
                .unwrap()
                .clauses;
            let clause = clauses
                .iter()
                .position(|row| row.clause == CastCostClause::Additional)
                .unwrap();
            act(
                game,
                Action::ChooseCastCost {
                    clause: u16::try_from(clause).unwrap(),
                },
            );
        }
        let object = game.object_ref(card);
        act(game, Action::ChooseEvidence { object });
        act(game, Action::FinishEvidence);
        if matches!(game.state.pending, Pending::CastCosts { .. }) {
            choose(game, |action| {
                matches!(action, Action::ChooseCastCost { .. })
            });
        }
    }
    pay(game);
}

fn exile_choice(game: &Game, card: CardId) -> Action {
    Action::SelectResolutionExile {
        object: game.object_ref(card),
    }
}

#[test]
fn unauthorized_exit_returns_a_nonland_and_its_surveil_fizzles_with_a_stale_target() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let exit = put(&mut game, player, "Unauthorized Exit", Zone::Hand);
    let creature = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    let land = put(&mut game, opponent, "Forest", Zone::Battlefield);
    let top = put(&mut game, player, "Opt", Zone::Library);
    let unsummon = put(&mut game, opponent, "Unsummon", Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: exit });
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(land),
        },
    );
    target(&mut game, creature);
    pay(&mut game);

    let mut stale = game.clone();
    act(&mut stale, Action::Pass);
    cast(&mut stale, unsummon, Some(Target::Card(creature)));
    settle(&mut stale);
    assert_eq!(stale.object_view(top).zone, Zone::Library);
    assert_eq!(stale.object_view(creature).zone, Zone::Hand);

    pass_pair(&mut game);
    assert_eq!(game.object_view(creature).zone, Zone::Hand);
    assert!(game.observe(player).library_choice.unwrap().surveil);
    assert!(game.observe(opponent).library_choice.is_none());
    act(
        &mut game,
        Action::ArrangeLibrary {
            card: top,
            destination: LibraryDestination::Graveyard,
        },
    );
    assert_eq!(game.object_view(top).zone, Zone::Graveyard);
}

#[test]
fn deadly_cover_up_destroys_creatures_then_exiles_same_name_cards_across_zones_when_paid() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let source = put(&mut game, player, "Deadly Cover-Up", Zone::Hand);
    let fuel = put(&mut game, player, "Earthshaker Dreadmaw", Zone::Graveyard);
    let creature = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    let survivor = put(&mut game, opponent, "Barbed Servitor", Zone::Battlefield);
    let grave = put(&mut game, opponent, "Bear Cub", Zone::Graveyard);
    let hand = put(&mut game, opponent, "Bear Cub", Zone::Hand);
    let library = put(&mut game, opponent, "Bear Cub", Zone::Library);
    let wrong = put(&mut game, opponent, "Grizzly Bears", Zone::Library);
    let mut unpaid = game.clone();
    collect_cost(&mut unpaid, source, None);
    settle(&mut unpaid);
    assert_eq!(unpaid.object_view(creature).zone, Zone::Graveyard);
    assert!(unpaid.on_battlefield(survivor));
    assert_eq!(unpaid.object_view(hand).zone, Zone::Hand);
    assert_eq!(unpaid.object_view(grave).zone, Zone::Graveyard);

    collect_cost(&mut game, source, Some(fuel));
    pass_pair(&mut game);
    assert!(game.on_battlefield(survivor));
    assert_eq!(game.object_view(creature).zone, Zone::Graveyard);
    let first = exile_choice(&game, creature);
    act(&mut game, first);
    assert_eq!(game.object_view(creature).zone, Zone::Exile);
    assert!(
        game.observe(player)
            .effect_choice
            .unwrap()
            .inspected_cards
            .is_some()
    );
    assert!(game.observe(opponent).effect_choice.is_none());
    let wrong_choice = exile_choice(&game, wrong);
    reject(&mut game, wrong_choice);
    for card in [grave, hand, library] {
        let action = exile_choice(&game, card);
        act(&mut game, action);
    }
    act(&mut game, Action::FinishSearch);
    settle(&mut game);
    for card in [grave, hand, library, fuel] {
        assert_eq!(game.object_view(card).zone, Zone::Exile);
    }
    assert_eq!(
        game.state.players[opponent.index()].cards_drawn_this_turn,
        1
    );
}

#[test]
fn extract_a_confession_uses_actual_paid_evidence_to_limit_the_sacrifice_to_greatest_power() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Extract a Confession", Zone::Hand);
    let fuel = put(&mut game, player, "Earthshaker Dreadmaw", Zone::Graveyard);
    let small = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    let first = put(&mut game, opponent, "Hill Giant", Zone::Battlefield);
    let second = put(&mut game, opponent, "Hill Giant", Zone::Battlefield);
    let mut unpaid = game.clone();
    collect_cost(&mut unpaid, spell, None);
    pass_pair(&mut unpaid);
    let object = unpaid.object_ref(small);
    act(&mut unpaid, Action::Sacrifice { object });
    assert_eq!(unpaid.object_view(small).zone, Zone::Graveyard);

    collect_cost(&mut game, spell, Some(fuel));
    pass_pair(&mut game);
    let object = game.object_ref(small);
    reject(&mut game, Action::Sacrifice { object });
    let mut tie = game.clone();
    let object = tie.object_ref(first);
    act(&mut tie, Action::Sacrifice { object });
    assert_eq!(tie.object_view(first).zone, Zone::Graveyard);
    let object = game.object_ref(second);
    act(&mut game, Action::Sacrifice { object });
    assert_eq!(game.object_view(second).zone, Zone::Graveyard);
    assert!(game.on_battlefield(first) && game.on_battlefield(small));
}

#[test]
fn festerleech_has_one_paid_boost_per_turn_and_mills_its_own_controller_on_combat_damage() {
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Festerleech", Zone::Battlefield);
    let top = [
        put(&mut game, player, "Island", Zone::Library),
        put(&mut game, player, "Opt", Zone::Library),
    ];
    act(&mut game, activation(source));
    pay(&mut game);
    settle(&mut game);
    assert_eq!(stats(&game, source), (3, 3));
    reject(&mut game, activation(source));

    begin_attack(&mut game, source);
    pass_pair(&mut game);
    act(&mut game, Action::FinishBlockers);
    pass_pair(&mut game);
    settle(&mut game);
    assert_eq!(life(&game, player.other()), 17);
    for card in top {
        assert_eq!(game.object_view(card).zone, Zone::Graveyard);
    }
    enter_step(&mut game, Step::End);
    pass_pair(&mut game);
    assert_eq!(stats(&game, source), (1, 1));
    game.state.players[player.index()].mana = [0, 0, 1, 0, 0, 1];
    if game.actor() != Some(player) {
        act(&mut game, Action::Pass);
    }
    act(&mut game, activation(source));
    pay(&mut game);
    settle(&mut game);
    assert_eq!(stats(&game, source), (3, 3));
}

#[test]
fn homicide_investigator_investigates_once_for_simultaneous_and_later_nontoken_deaths() {
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(
        &mut game,
        player,
        "Homicide Investigator",
        Zone::Battlefield,
    );
    game.state.objects[source as usize].gained_keywords = Keywords::INDESTRUCTIBLE;
    let first = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let second = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let sweep = put(&mut game, player, "Day of Judgment", Zone::Hand);
    let murder = put(&mut game, player, "Murder", Zone::Hand);
    cast(&mut game, sweep, None);
    settle(&mut game);
    assert!(game.on_battlefield(source));
    for dead in [first, second] {
        assert_eq!(game.object_view(dead).zone, Zone::Graveyard);
    }
    assert_eq!(tokens(&game, player, CLUE).len(), 1);
    let later = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    cast(&mut game, murder, Some(Target::Card(later)));
    settle(&mut game);
    assert_eq!(tokens(&game, player, CLUE).len(), 1);
}

#[test]
fn it_doesnt_add_up_returns_its_exact_creature_and_suspects_it_after_entry() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let source = put(&mut game, player, "It Doesn't Add Up", Zone::Hand);
    let card = put(&mut game, player, "Novice Inspector", Zone::Graveyard);
    let foreign = put(&mut game, opponent, "Bear Cub", Zone::Graveyard);
    let ooze = put(&mut game, opponent, "Scavenging Ooze", Zone::Battlefield);
    act(&mut game, Action::AnnounceCast { card: source });
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(foreign),
        },
    );
    target(&mut game, card);
    pay(&mut game);

    let mut stale = game.clone();
    act(&mut stale, Action::Pass);
    act(&mut stale, activation(ooze));
    target(&mut stale, card);
    pay(&mut stale);
    settle(&mut stale);
    assert_eq!(stale.object_view(card).zone, Zone::Exile);
    assert!(stale.observe(player).suspected.is_empty());

    settle(&mut game);
    assert!(game.on_battlefield(card));
    assert_eq!(game.observe(player).suspected, [game.object_ref(card)]);
    assert_eq!(tokens(&game, player, CLUE).len(), 1);
}

#[test]
fn leering_onlooker_exiles_itself_from_the_graveyard_and_makes_two_tapped_flying_bats() {
    const BAT: Card = Card::from_catalogue_id(5276);
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Leering Onlooker", Zone::Graveyard);
    let battlefield = put(&mut game, player, "Leering Onlooker", Zone::Battlefield);
    assert!(game.keywords(battlefield).contains(Keywords::FLYING));
    reject(&mut game, activation(battlefield));
    act(&mut game, activation(source));
    act(
        &mut game,
        Action::PayCosts {
            payment: [0, 0, 2, 0, 0, 2],
        },
    );
    assert_eq!(game.object_view(source).zone, Zone::Exile);
    settle(&mut game);
    let bats = tokens(&game, player, BAT);
    assert_eq!(bats.len(), 2);
    for bat in bats {
        assert_eq!(stats(&game, bat), (1, 1));
        assert!(game.object_view(bat).tapped);
        assert_eq!(game.object_view(bat).colors, 4);
        assert!(game.keywords(bat).contains(Keywords::FLYING));
    }
    reject(&mut game, activation(source));
}

#[test]
fn long_goodbye_destroys_a_three_mana_walker_or_a_disguised_creature_despite_ward() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(opponent);
    let source = put(&mut game, player, "Long Goodbye", Zone::Hand);
    let hidden = put(&mut game, opponent, "Offender at Large", Zone::Hand);
    let expensive = put(&mut game, opponent, "Hill Giant", Zone::Battlefield);
    let walker = put(
        &mut game,
        opponent,
        "Kaito, Cunning Infiltrator",
        Zone::Battlefield,
    );
    game.add_counter(walker, CounterKind::Loyalty, 3);
    act(&mut game, Action::AnnounceDisguise { card: hidden });
    pay(&mut game);
    settle(&mut game);
    act(&mut game, Action::Pass);
    act(&mut game, Action::AnnounceCast { card: source });
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(expensive),
        },
    );
    let mut walker_choice = game.clone();
    target(&mut walker_choice, walker);
    pay(&mut walker_choice);
    settle(&mut walker_choice);
    assert_eq!(walker_choice.object_view(walker).zone, Zone::Graveyard);

    target(&mut game, hidden);
    pay(&mut game);
    pass_pair(&mut game);
    if game
        .legal_actions()
        .contains(&Action::DeclineResolutionMana)
    {
        act(&mut game, Action::DeclineResolutionMana);
    }
    settle(&mut game);
    assert_eq!(game.object_view(hidden).zone, Zone::Graveyard);
}

#[test]
fn outrageous_robbery_grants_private_indefinite_exact_exile_play_with_normal_land_timing() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let robbery = put(&mut game, player, "Outrageous Robbery", Zone::Hand);
    let land = put(&mut game, opponent, "Island", Zone::Library);
    let spell = put(&mut game, opponent, "Think Twice", Zone::Library);
    act(&mut game, Action::AnnounceCast { card: robbery });
    act(&mut game, Action::ChooseX { value: 2 });
    target_player(&mut game, opponent);
    pay(&mut game);
    settle(&mut game);

    for card in [land, spell] {
        let object = game.object_ref(card);
        assert!(game.exiled_face_down(object));
        assert!(
            game.observe(opponent)
                .objects
                .iter()
                .all(|view| view.id != card)
        );
        assert!(
            game.observe(player)
                .objects
                .iter()
                .any(|view| view.id == card)
        );
        assert!(
            game.observe(player)
                .exile_permissions
                .iter()
                .any(|permission| permission.object == object
                    && permission.expiry == ExilePlayExpiry::WhileExiled)
        );
    }
    let mut off_phase = game.clone();
    enter_step(&mut off_phase, Step::Upkeep);
    reject(&mut off_phase, Action::PlayLand { card: land });
    act(&mut game, Action::PlayLand { card: land });
    assert!(game.on_battlefield(land));
    assert_eq!(game.object_view(land).owner, opponent);
    assert_eq!(game.object_view(land).controller, player);

    game.state.players[player.index()].mana = [0, 0, 0, 0, 2, 0];
    act(&mut game, Action::AnnounceExile { card: spell });
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
    assert_eq!(game.object_view(spell).zone, Zone::Graveyard);
}

#[test]
fn persuasive_interrogators_investigates_and_counts_real_clue_sacrifices_to_lethal_poison() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let source = put(&mut game, player, "Persuasive Interrogators", Zone::Hand);
    let nonclue = put(&mut game, player, "Soul-Guide Lantern", Zone::Battlefield);
    let equipment = put(&mut game, player, "Lead Pipe", Zone::Battlefield);
    cast(&mut game, source, None);
    settle(&mut game);
    let [clue] = tokens(&game, player, CLUE).try_into().unwrap();
    assert!(!game.has_subtype(nonclue, "Clue"));
    act(&mut game, activation_at(nonclue, 1));
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(nonclue).zone, Zone::Graveyard);
    assert_eq!(
        game.observe(opponent).players[opponent.index()].poison_counters,
        0
    );

    act(&mut game, activation_at(equipment, 0));
    pay(&mut game);
    target_player(&mut game, opponent);
    settle(&mut game);
    assert_eq!(
        game.observe(opponent).players[opponent.index()].poison_counters,
        2
    );
    assert_eq!(game.object_view(equipment).zone, Zone::Graveyard);

    game.state.players[opponent.index()].poison_counters = 8;
    act(&mut game, activation(clue));
    pay(&mut game);
    target_player(&mut game, opponent);
    settle(&mut game);
    assert_eq!(
        game.observe(opponent).players[opponent.index()].poison_counters,
        10
    );
    assert!(game.outcome().is_some());
}

#[test]
fn presumed_dead_returns_a_stolen_creature_to_its_owner_and_suspects_the_new_object() {
    let player = Player::P0;
    let owner = player.other();
    let mut game = scenario(player);
    let source = put(&mut game, player, "Presumed Dead", Zone::Hand);
    let card = put(&mut game, owner, "Bear Cub", Zone::Battlefield);
    let murder = put(&mut game, player, "Murder", Zone::Hand);
    let exact = game.object_ref(card);
    game.record_control(exact, player, None);
    cast(&mut game, source, Some(Target::Card(card)));
    settle(&mut game);
    assert_eq!(stats(&game, card), (4, 2));

    let mut redirected = game.clone();
    put(&mut redirected, owner, "Rest in Peace", Zone::Battlefield);
    cast(&mut redirected, murder, Some(Target::Card(card)));
    settle(&mut redirected);
    assert_eq!(redirected.object_view(card).zone, Zone::Exile);

    cast(&mut game, murder, Some(Target::Card(card)));
    settle(&mut game);
    assert!(game.on_battlefield(card));
    assert_eq!(game.object_view(card).controller, owner);
    assert_eq!(stats(&game, card), (2, 2));
    assert_ne!(game.object_ref(card), exact);
    assert_eq!(game.observe(owner).suspected, [game.object_ref(card)]);
}

#[test]
fn slice_from_the_shadows_uses_one_black_plus_x_and_kills_indestructible_at_zero_toughness() {
    let player = Player::P1;
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Slice from the Shadows", Zone::Hand);
    let target_card = put(
        &mut game,
        player.other(),
        "Barbed Servitor",
        Zone::Battlefield,
    );
    game.state.players[player.index()].mana = [0, 0, 1, 0, 0, 1];
    for (x, expected) in [(0, Zone::Battlefield), (1, Zone::Graveyard)] {
        let mut branch = game.clone();
        act(&mut branch, Action::AnnounceCast { card: spell });
        reject(&mut branch, Action::ChooseX { value: 2 });
        act(&mut branch, Action::ChooseX { value: x });
        target(&mut branch, target_card);
        act(
            &mut branch,
            Action::PayCosts {
                payment: [0, 0, 1, 0, 0, x],
            },
        );
        settle(&mut branch);
        assert_eq!(branch.object_view(target_card).zone, expected);
    }
}

#[test]
fn snarling_gorehound_counts_each_small_entrant_by_effective_power() {
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Snarling Gorehound", Zone::Hand);
    let large = put(&mut game, player, "Hill Giant", Zone::Hand);
    let hidden = put(&mut game, player, "Basilica Stalker", Zone::Hand);
    let batch = put(&mut game, player, "Hop to It", Zone::Hand);
    cast(&mut game, source, None);
    settle(&mut game);
    assert!(game.keywords(source).contains(Keywords::MENACE));
    cast(&mut game, large, None);
    settle(&mut game);

    act(&mut game, Action::AnnounceDisguise { card: hidden });
    pay(&mut game);
    pass_pair(&mut game);
    pass_pair(&mut game);
    assert!(game.observe(player).library_choice.unwrap().surveil);
    let card = game.top_library_card(player).unwrap();
    act(
        &mut game,
        Action::ArrangeLibrary {
            card,
            destination: LibraryDestination::Top,
        },
    );

    cast(&mut game, batch, None);
    pass_pair(&mut game);
    for _ in 0..2 {
        choose(&mut game, |action| {
            matches!(action, Action::PutTrigger { .. })
        });
    }
    assert_eq!(game.state.stack.len(), 3);
    for _ in 0..3 {
        pass_pair(&mut game);
        let card = game.top_library_card(player).unwrap();
        act(
            &mut game,
            Action::ArrangeLibrary {
                card,
                destination: LibraryDestination::Graveyard,
            },
        );
    }
    assert!(game.state.stack.is_empty());
}

#[test]
fn toxin_analysis_and_the_chase_is_on_investigate_only_after_a_legal_creature_target() {
    let player = Player::P0;
    let opponent = player.other();
    let cases = [
        (
            "Toxin Analysis",
            (2, 2),
            Keywords::DEATHTOUCH.union(Keywords::LIFELINK),
        ),
        ("The Chase Is On", (5, 2), Keywords::FIRST_STRIKE),
    ];
    for (name, expected, keywords) in cases {
        let mut game = scenario(player);
        let spell = put(&mut game, player, name, Zone::Hand);
        let creature = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let murder = put(&mut game, opponent, "Murder", Zone::Hand);
        cast(&mut game, spell, Some(Target::Card(creature)));
        let mut removed = game.clone();
        act(&mut removed, Action::Pass);
        cast(&mut removed, murder, Some(Target::Card(creature)));
        settle(&mut removed);
        assert!(tokens(&removed, player, CLUE).is_empty(), "{name}");

        settle(&mut game);
        assert_eq!(stats(&game, creature), expected, "{name}");
        assert!(game.keywords(creature).contains(keywords), "{name}");
        assert_eq!(tokens(&game, player, CLUE).len(), 1, "{name}");
        enter_step(&mut game, Step::End);
        pass_pair(&mut game);
        assert_eq!(stats(&game, creature), (2, 2), "{name}");
        assert!(!game.keywords(creature).contains(keywords), "{name}");
    }
}

#[test]
fn vein_ripper_wards_against_murder_and_drains_for_the_paid_creature_and_itself() {
    let controller = Player::P0;
    let player = controller.other();
    let mut game = scenario(player);
    let ripper = put(&mut game, controller, "Vein Ripper", Zone::Battlefield);
    let creature = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let artifact = put(&mut game, player, "Candy Trail", Zone::Battlefield);
    let murder = put(&mut game, player, "Murder", Zone::Hand);
    assert!(game.keywords(ripper).contains(Keywords::FLYING));
    act(&mut game, Action::AnnounceCast { card: murder });
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(artifact),
        },
    );
    target(&mut game, ripper);
    pay(&mut game);
    pass_pair(&mut game);
    let mut unpaid = game.clone();
    act(&mut unpaid, Action::DeclineEffectPayment);
    settle(&mut unpaid);
    assert!(unpaid.on_battlefield(ripper));
    assert!(unpaid.on_battlefield(creature));
    assert_eq!(unpaid.object_view(murder).zone, Zone::Graveyard);

    let object = game.object_ref(artifact);
    reject(&mut game, Action::Sacrifice { object });
    let object = game.object_ref(creature);
    act(&mut game, Action::Sacrifice { object });
    for _ in 0..12 {
        if game.state.stack.is_empty()
            && game.state.resolving.is_none()
            && matches!(game.state.pending, Pending::Priority)
        {
            break;
        }
        match game.state.pending {
            Pending::Targets { .. } => target_player(&mut game, player),
            Pending::Priority => act(&mut game, Action::Pass),
            ref other => panic!("unexpected Ward/death choice {other:?}"),
        }
    }
    assert!(game.state.stack.is_empty());
    assert_eq!(game.object_view(creature).zone, Zone::Graveyard);
    assert_eq!(game.object_view(ripper).zone, Zone::Graveyard);
    assert_eq!((life(&game, controller), life(&game, player)), (24, 16));
}

#[test]
fn convenient_target_suspects_the_enchanted_creature_and_can_return_from_the_graveyard() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let aura = put(&mut game, player, "Convenient Target", Zone::Hand);
    let creature = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    let disenchant = put(&mut game, player, "Disenchant", Zone::Hand);
    cast(&mut game, aura, Some(Target::Card(creature)));
    settle(&mut game);
    let exact = game.object_ref(creature);
    assert_eq!(stats(&game, creature), (3, 3));
    assert_eq!(game.observe(opponent).suspected, [exact]);
    cast(&mut game, disenchant, Some(Target::Card(aura)));
    settle(&mut game);
    assert_eq!(stats(&game, creature), (2, 2));
    assert_eq!(game.observe(opponent).suspected, [exact]);

    act(&mut game, activation(aura));
    act(
        &mut game,
        Action::PayCosts {
            payment: [0, 0, 0, 1, 0, 2],
        },
    );
    settle(&mut game);
    assert_eq!(game.object_view(aura).zone, Zone::Hand);
}

#[test]
fn crime_novelist_and_vengeful_tracker_observe_the_actual_artifact_sacrifice_after_cast_payment() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let novelist = put(&mut game, player, "Crime Novelist", Zone::Battlefield);
    let opposite_novelist = put(&mut game, opponent, "Crime Novelist", Zone::Battlefield);
    let tracker = put(&mut game, opponent, "Vengeful Tracker", Zone::Battlefield);
    let own_tracker = put(&mut game, player, "Vengeful Tracker", Zone::Battlefield);
    let artifact = put(&mut game, player, "Candy Trail", Zone::Battlefield);
    let disenchant = put(&mut game, player, "Disenchant", Zone::Hand);
    let spell = put(&mut game, player, "Divination", Zone::Hand);
    let treasure = Card::from_catalogue_id(5302);
    game.create_tokens(player, treasure, 1, false);
    let [token] = tokens(&game, player, treasure).try_into().unwrap();
    game.state.players[player.index()].mana = [0, 1, 0, 0, 0, 1];
    act(&mut game, Action::AnnounceCast { card: spell });
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
    assert_eq!(game.counter_count(novelist, CounterKind::PlusOne), 0);
    pay(&mut game);
    assert_eq!(game.state.stack.len(), 3);
    settle(&mut game);
    assert_eq!(game.counter_count(novelist, CounterKind::PlusOne), 1);
    assert_eq!(
        game.counter_count(opposite_novelist, CounterKind::PlusOne),
        0
    );
    assert_eq!(game.state.players[player.index()].mana[3], 1);
    assert_eq!(life(&game, player), 18);
    assert_eq!(life(&game, opponent), 20);
    assert!(game.object_view(tracker).dealt_damage_this_turn);
    assert!(!game.object_view(own_tracker).dealt_damage_this_turn);

    game.state.players[player.index()].mana[0] = 1;
    cast(&mut game, disenchant, Some(Target::Card(artifact)));
    settle(&mut game);
    assert_eq!(game.object_view(artifact).zone, Zone::Graveyard);
    assert_eq!(game.counter_count(novelist, CounterKind::PlusOne), 1);
    assert_eq!(life(&game, player), 18);
}

#[test]
fn person_of_interest_suspects_only_itself_and_creates_a_detective_if_removed_in_response() {
    const DETECTIVE: Card = Card::from_catalogue_id(5329);
    let player = Player::P1;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Person of Interest", Zone::Hand);
    let murder = put(&mut game, player, "Murder", Zone::Hand);
    cast(&mut game, source, None);
    pass_pair(&mut game);
    let mut removed = game.clone();
    cast(&mut removed, murder, Some(Target::Card(source)));
    settle(&mut removed);
    assert!(removed.observe(player).suspected.is_empty());
    assert_eq!(tokens(&removed, player, DETECTIVE).len(), 1);

    settle(&mut game);
    assert_eq!(game.observe(player).suspected, [game.object_ref(source)]);
    let [token] = tokens(&game, player, DETECTIVE).try_into().unwrap();
    assert_eq!(stats(&game, token), (2, 2));
    assert_eq!(game.object_view(token).colors, 3);
    assert!(game.has_subtype(token, "Detective"));
}

#[test]
fn shock_deals_damage_to_players_creatures_planeswalkers_and_battles() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let shock = put(&mut game, player, "Shock", Zone::Hand);
    let creature = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    let walker = put(
        &mut game,
        opponent,
        "Kaito, Cunning Infiltrator",
        Zone::Battlefield,
    );
    game.add_counter(walker, CounterKind::Loyalty, 3);
    let battle = put(&mut game, opponent, "Goblin Piker", Zone::Battlefield);
    let mut recipe = CopyRecipe::new(Card::GoblinPiker);
    recipe.overrides = Some(CopyOverrides {
        kinds: Some(BATTLE_KIND),
        ..CopyOverrides::NONE
    });
    game.apply_temporary_copy(game.object_ref(battle), recipe, CopyDeadline::EndOfTurn);
    game.add_counter(battle, CounterKind::Defense, 4);
    game.set_battle_protector(game.object_ref(battle), opponent);

    let mut creature_hit = game.clone();
    cast(&mut creature_hit, shock, Some(Target::Card(creature)));
    settle(&mut creature_hit);
    assert_eq!(creature_hit.object_view(creature).zone, Zone::Graveyard);
    let mut planeswalker_hit = game.clone();
    cast(&mut planeswalker_hit, shock, Some(Target::Card(walker)));
    settle(&mut planeswalker_hit);
    assert_eq!(
        planeswalker_hit.counter_count(walker, CounterKind::Loyalty),
        1
    );
    let mut battle_hit = game.clone();
    cast(&mut battle_hit, shock, Some(Target::Card(battle)));
    settle(&mut battle_hit);
    assert_eq!(battle_hit.counter_count(battle, CounterKind::Defense), 2);
    cast(&mut game, shock, Some(Target::Player(opponent)));
    settle(&mut game);
    assert_eq!(life(&game, opponent), 18);
}
