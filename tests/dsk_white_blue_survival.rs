//! Actual paid survival, impending, and enduring creature rules.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

const PLAYER: Player = Player::P0;
const OPPONENT: Player = Player::P1;
const MISTMOORS: &str = "Overlord of the Mistmoors";
const FLOODPITS: &str = "Overlord of the Floodpits";

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn resolve(game: &mut Game) {
    for _ in 0..32 {
        match game.state.pending {
            Pending::Priority if game.state.stack.is_empty() => return,
            Pending::Priority => act(game, Action::Pass),
            Pending::OrderTriggers { .. } => {
                choose(game, |action| matches!(action, Action::PutTrigger { .. }));
            }
            Pending::EntryTimestamp { .. } => {
                choose(game, |action| {
                    matches!(action, Action::ChooseEntryTimestamp { .. })
                });
            }
            _ => return,
        }
    }
    panic!("survival scenario did not reach its next choice");
}

fn end_turn(game: &mut Game) {
    game.state.step = Step::End;
    pass_pair(game);
}

fn next_own_turn(game: &mut Game) {
    end_turn(game);
    assert_eq!(game.state.active, OPPONENT);
    end_turn(game);
    assert_eq!(game.state.active, PLAYER);
}

fn second_main(game: &mut Game) {
    game.state.players[game.state.active.index()]
        .turn_facts
        .main_phases_started = 1;
    game.begin_step(Step::Main2);
    game.prepare_triggers();
}

fn attack(game: &mut Game, cards: &[CardId]) {
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    for &card in cards {
        act(game, Action::Attack { card });
    }
    act(game, Action::FinishAttackers);
}

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

fn select_target(game: &mut Game, card: CardId) {
    act(game, target(card));
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(game, Action::FinishTargets);
    }
}

fn cast_overlord(game: &mut Game, card: CardId, impending: bool, cost: ManaCost) {
    act(game, Action::AnnounceCast { card });
    let route = game
        .observe(PLAYER)
        .cost_choices
        .into_iter()
        .find(|choice| choice.impending == impending)
        .unwrap();
    assert_eq!(route.cost, cost);
    act(game, Action::ChooseSpellCost { cost: route.index });
    pay(game);
    assert_eq!(
        game.state.stack.last().unwrap().cast.unwrap().impending,
        impending
    );
    resolve(game);
}

fn tokens(game: &Game, subtype: &str) -> Vec<ObjectView> {
    game.observe(PLAYER)
        .objects
        .into_iter()
        .filter(|object| {
            object.is_token()
                && object.controller == PLAYER
                && object.zone == Zone::Battlefield
                && object.has_subtype(subtype)
        })
        .collect()
}

#[test]
fn mistmoors_makes_its_two_flying_insects_on_entry_and_after_it_can_attack() {
    let mut game = scenario(PLAYER);
    let overlord = put(&mut game, PLAYER, MISTMOORS, Zone::Hand);
    let mut normal = game.clone();
    cast_overlord(
        &mut normal,
        overlord,
        false,
        ManaCost {
            colored: [2, 0, 0, 0, 0, 0],
            generic: 5,
        },
    );
    assert_eq!(stats(&normal, overlord), (6, 6));
    assert!(normal.object_view(overlord).is_creature());
    assert_eq!(normal.counter_count(overlord, CounterKind::Time), 0);
    assert_eq!(tokens(&normal, "Insect").len(), 2);

    game.state.players[PLAYER.index()].mana = [2, 0, 0, 0, 0, 2];
    cast_overlord(
        &mut game,
        overlord,
        true,
        ManaCost {
            colored: [2, 0, 0, 0, 0, 0],
            generic: 2,
        },
    );
    let view = game.object_view(overlord);
    assert_eq!(view.kinds, 32);
    assert!(view.cast_for_impending);
    assert_eq!(game.counter_count(overlord, CounterKind::Time), 4);
    assert_eq!(tokens(&game, "Insect").len(), 2);
    for remaining in [3, 2, 1, 0] {
        game.begin_step(Step::End);
        game.prepare_triggers();
        settle(&mut game);
        assert_eq!(game.counter_count(overlord, CounterKind::Time), remaining);
        assert_eq!(game.object_view(overlord).is_creature(), remaining == 0);
        next_own_turn(&mut game);
    }
    assert_eq!(stats(&game, overlord), (6, 6));
    attack(&mut game, &[overlord]);
    resolve(&mut game);
    let insects = tokens(&game, "Insect");
    assert_eq!(insects.len(), 4);
    for insect in insects {
        assert_eq!(
            (insect.name, insect.kinds, insect.colors),
            ("Insect Token", 1, 1)
        );
        assert_eq!((insect.power, insect.toughness), (2, 1));
        assert!(insect.keywords.contains(Keywords::FLYING));
    }
}

#[test]
fn floodpits_draws_then_discards_on_impending_entry_and_on_attack() {
    let mut game = scenario(PLAYER);
    let overlord = put(&mut game, PLAYER, FLOODPITS, Zone::Hand);
    let top = put(&mut game, PLAYER, "Shock", Zone::Library);
    let mut impending = game.clone();
    impending.state.players[PLAYER.index()].mana = [0, 2, 0, 0, 0, 1];
    cast_overlord(
        &mut impending,
        overlord,
        true,
        ManaCost {
            colored: [0, 2, 0, 0, 0, 0],
            generic: 1,
        },
    );
    assert_eq!(impending.object_view(overlord).kinds, 32);
    assert_eq!(impending.counter_count(overlord, CounterKind::Time), 4);
    assert_eq!(
        impending.state.players[PLAYER.index()].cards_drawn_this_turn,
        2
    );
    assert_eq!(impending.object_view(top).zone, Zone::Hand);
    act(&mut impending, Action::Discard { card: top });
    assert_eq!(impending.object_view(top).zone, Zone::Graveyard);

    cast_overlord(
        &mut game,
        overlord,
        false,
        ManaCost {
            colored: [0, 2, 0, 0, 0, 0],
            generic: 3,
        },
    );
    act(&mut game, Action::Discard { card: top });
    assert_eq!(stats(&game, overlord), (5, 3));
    assert!(
        game.object_view(overlord)
            .keywords
            .contains(Keywords::FLYING)
    );
    assert_eq!(game.counter_count(overlord, CounterKind::Time), 0);
    next_own_turn(&mut game);
    let next = *game.state.players[PLAYER.index()].library.last().unwrap();
    attack(&mut game, &[overlord]);
    resolve(&mut game);
    assert_eq!(game.state.players[PLAYER.index()].cards_drawn_this_turn, 2);
    assert_eq!(game.object_view(next).zone, Zone::Hand);
    act(&mut game, Action::Discard { card: next });
    assert_eq!(game.object_view(next).zone, Zone::Graveyard);
}

#[test]
fn veteran_survivor_exiles_from_either_graveyard_and_checks_three_exact_links_live() {
    let mut game = scenario(PLAYER);
    let veteran = put(&mut game, PLAYER, "Veteran Survivor", Zone::Battlefield);
    let spare = put(&mut game, PLAYER, "Bear Cub", Zone::Battlefield);
    let victims =
        [PLAYER, OPPONENT, PLAYER].map(|player| put(&mut game, player, "Forest", Zone::Graveyard));
    let exact = game.object_ref(veteran);
    for (index, victim) in victims.into_iter().enumerate() {
        game.state.objects[veteran as usize].tapped = true;
        second_main(&mut game);
        if index == 0 {
            let mut decline = game.clone();
            act(&mut decline, Action::FinishTargets);
            settle(&mut decline);
            assert_eq!(decline.object_view(victim).zone, Zone::Graveyard);
        }
        select_target(&mut game, victim);
        settle(&mut game);
        assert_eq!(game.object_view(victim).zone, Zone::Exile);
        assert_eq!(game.object_ref(veteran), exact);
        assert_eq!(
            game.object_view(veteran)
                .keywords
                .contains(Keywords::HEXPROOF),
            index == 2
        );
        assert_eq!(
            stats(&game, veteran),
            if index == 2 { (5, 4) } else { (2, 1) }
        );
        if index != 2 {
            next_own_turn(&mut game);
        }
    }
    let removal = put(&mut game, OPPONENT, "Murder", Zone::Hand);
    game.state.players[OPPONENT.index()].mana = [8; 6];
    act(&mut game, Action::Pass);
    act(&mut game, Action::AnnounceCast { card: removal });
    reject(&mut game, target(veteran));
    assert!(game.legal_actions().contains(&target(spare)));
    game.move_object(victims[0], Zone::Graveyard);
    game.move_object(victims[0], Zone::Exile);
    assert_eq!(stats(&game, veteran), (2, 1));
    assert!(
        !game
            .object_view(veteran)
            .keywords
            .contains(Keywords::HEXPROOF)
    );
    assert!(game.legal_actions().contains(&target(veteran)));
}

#[test]
fn acrobatic_cheerleader_uses_its_survival_once_per_incarnation() {
    let mut game = scenario(PLAYER);
    let cheerleader = put(
        &mut game,
        PLAYER,
        "Acrobatic Cheerleader",
        Zone::Battlefield,
    );
    let shore_up = put(&mut game, PLAYER, "Shore Up", Zone::Hand);
    let mut untapped = game.clone();
    second_main(&mut untapped);
    assert!(untapped.state.stack.is_empty());
    game.state.objects[cheerleader as usize].tapped = true;
    second_main(&mut game);
    let mut interrupted = game.clone();
    cast(&mut interrupted, shore_up, Some(Target::Card(cheerleader)));
    settle(&mut interrupted);
    assert_eq!(
        interrupted.counter_count(cheerleader, CounterKind::Flying),
        0
    );
    next_own_turn(&mut interrupted);
    interrupted.state.objects[cheerleader as usize].tapped = true;
    second_main(&mut interrupted);
    assert!(interrupted.state.stack.is_empty());

    settle(&mut game);
    let exact = game.object_ref(cheerleader);
    assert_eq!(game.counter_count(cheerleader, CounterKind::Flying), 1);
    assert!(
        game.object_view(cheerleader)
            .keywords
            .contains(Keywords::FLYING)
    );
    game.move_object(cheerleader, Zone::Exile);
    game.move_object(cheerleader, Zone::Battlefield);
    assert_ne!(game.object_ref(cheerleader), exact);
    next_own_turn(&mut game);
    game.state.objects[cheerleader as usize].tapped = true;
    second_main(&mut game);
    settle(&mut game);
    assert_eq!(game.counter_count(cheerleader, CounterKind::Flying), 1);
}

#[test]
fn enduring_innocence_counts_qualifying_entries_once_then_returns_as_an_enchantment() {
    let mut game = scenario(PLAYER);
    let innocence = put(&mut game, PLAYER, "Enduring Innocence", Zone::Hand);
    let too_large = put(&mut game, PLAYER, "Hill Giant", Zone::Hand);
    let mistmoors = put(&mut game, PLAYER, MISTMOORS, Zone::Hand);
    let later = put(&mut game, PLAYER, "Bear Cub", Zone::Hand);
    let murder = put(&mut game, PLAYER, "Murder", Zone::Hand);
    cast(&mut game, innocence, None);
    settle(&mut game);
    assert!(
        game.object_view(innocence)
            .keywords
            .contains(Keywords::LIFELINK)
    );
    cast(&mut game, too_large, None);
    settle(&mut game);
    assert_eq!(game.state.players[PLAYER.index()].cards_drawn_this_turn, 0);
    cast_overlord(
        &mut game,
        mistmoors,
        false,
        ManaCost {
            colored: [2, 0, 0, 0, 0, 0],
            generic: 5,
        },
    );
    assert_eq!(tokens(&game, "Insect").len(), 2);
    assert_eq!(game.state.players[PLAYER.index()].cards_drawn_this_turn, 1);
    cast(&mut game, later, None);
    settle(&mut game);
    assert_eq!(game.state.players[PLAYER.index()].cards_drawn_this_turn, 1);

    let exact = game.object_ref(innocence);
    game.record_control(exact, OPPONENT, None);
    cast(&mut game, murder, Some(Target::Card(innocence)));
    settle(&mut game);
    let returned = game.object_view(innocence);
    assert_ne!(returned.reference, exact);
    assert_eq!((returned.controller, returned.kinds), (PLAYER, 32));
    assert!(!returned.has_subtype("Sheep"));
    end_turn(&mut game);
    act(&mut game, Action::Pass);
    game.state.players[PLAYER.index()].mana = [8; 6];
    let sailor = put(&mut game, PLAYER, "Spectral Sailor", Zone::Hand);
    cast(&mut game, sailor, None);
    settle(&mut game);
    assert_eq!(game.state.players[PLAYER.index()].cards_drawn_this_turn, 1);
}

#[test]
fn enduring_curiosity_flashes_and_draws_per_combat_source_before_its_exact_return() {
    let mut game = scenario(OPPONENT);
    let curiosity = put(&mut game, PLAYER, "Enduring Curiosity", Zone::Hand);
    let attackers =
        ["Bear Cub", "Grizzly Bears"].map(|name| put(&mut game, PLAYER, name, Zone::Battlefield));
    act(&mut game, Action::Pass);
    cast(&mut game, curiosity, None);
    settle(&mut game);
    assert_eq!(stats(&game, curiosity), (4, 3));
    end_turn(&mut game);
    assert_eq!(game.state.active, PLAYER);
    attack(&mut game, &attackers);
    for _ in 0..28 {
        if game.state.step == Step::Main2 && game.state.stack.is_empty() {
            break;
        }
        choose(&mut game, |action| {
            matches!(
                action,
                Action::Pass | Action::PutTrigger { .. } | Action::FinishBlockers
            )
        });
    }
    assert_eq!(game.state.step, Step::Main2);
    assert_eq!(game.state.players[PLAYER.index()].cards_drawn_this_turn, 2);
    game.state.players[PLAYER.index()].mana = [8; 6];
    let murder = put(&mut game, PLAYER, "Murder", Zone::Hand);
    let disenchant = put(&mut game, PLAYER, "Disenchant", Zone::Hand);
    let exact = game.object_ref(curiosity);
    game.record_control(exact, OPPONENT, None);
    cast(&mut game, murder, Some(Target::Card(curiosity)));
    settle(&mut game);
    let enchantment = game.object_view(curiosity);
    assert_eq!(
        (enchantment.zone, enchantment.controller, enchantment.kinds),
        (Zone::Battlefield, PLAYER, 32)
    );
    assert_ne!(enchantment.reference, exact);
    assert!(!enchantment.has_subtype("Cat") && !enchantment.has_subtype("Glimmer"));
    cast(&mut game, disenchant, Some(Target::Card(curiosity)));
    settle(&mut game);
    assert_eq!(game.object_view(curiosity).zone, Zone::Graveyard);
}

#[test]
fn glimmer_seeker_uses_current_glimmer_control_to_draw_or_create() {
    let mut game = scenario(PLAYER);
    let seeker = put(&mut game, PLAYER, "Glimmer Seeker", Zone::Battlefield);
    game.state.objects[seeker as usize].tapped = true;
    second_main(&mut game);
    settle(&mut game);
    assert_eq!(game.state.players[PLAYER.index()].cards_drawn_this_turn, 0);
    let glimmer = tokens(&game, "Glimmer").pop().unwrap();
    assert_eq!(
        (glimmer.name, glimmer.kinds, glimmer.colors),
        ("Glimmer Token", 33, 1)
    );
    assert_eq!((glimmer.power, glimmer.toughness), (1, 1));
    next_own_turn(&mut game);
    game.state.objects[seeker as usize].tapped = true;
    second_main(&mut game);
    let mut stolen = game.clone();
    stolen.record_control(glimmer.reference, OPPONENT, None);
    settle(&mut stolen);
    assert_eq!(
        stolen.state.players[PLAYER.index()].cards_drawn_this_turn,
        0
    );
    let replacement = tokens(&stolen, "Glimmer").pop().unwrap();
    assert_ne!(replacement.reference, glimmer.reference);
    settle(&mut game);
    assert_eq!(game.state.players[PLAYER.index()].cards_drawn_this_turn, 1);
    assert_eq!(tokens(&game, "Glimmer").len(), 1);
}

#[test]
fn orphans_tap_only_selected_ready_creatures_and_grow_by_that_group() {
    let mut game = scenario(PLAYER);
    let orphans = put(&mut game, PLAYER, "Orphans of the Wheat", Zone::Battlefield);
    let ready =
        ["Bear Cub", "Llanowar Elves"].map(|name| put(&mut game, PLAYER, name, Zone::Battlefield));
    game.state.objects[ready[0] as usize].summoning_sick = true;
    let tapped = put(&mut game, PLAYER, "Grizzly Bears", Zone::Battlefield);
    game.state.objects[tapped as usize].tapped = true;
    let opposing = put(&mut game, OPPONENT, "Healer's Hawk", Zone::Battlefield);
    attack(&mut game, &[orphans]);
    pass_pair(&mut game);
    let mut none = game.clone();
    act(&mut none, Action::FinishEffectChoice);
    assert_eq!(stats(&none, orphans), (2, 1));
    for card in [orphans, tapped, opposing] {
        let object = game.object_ref(card);
        reject(&mut game, Action::ChooseEffectObject { object });
    }
    for card in ready {
        let object = game.object_ref(card);
        act(&mut game, Action::ChooseEffectObject { object });
        assert!(!game.object_view(card).tapped);
    }
    act(&mut game, Action::FinishEffectChoice);
    assert!(ready.into_iter().all(|card| game.object_view(card).tapped));
    assert_eq!(stats(&game, orphans), (4, 3));
}

#[test]
fn patched_plaything_counts_only_its_hand_cast_entry_and_keeps_double_strike() {
    let mut game = scenario(PLAYER);
    let from_hand = put(&mut game, PLAYER, "Patched Plaything", Zone::Hand);
    let from_grave = put(&mut game, PLAYER, "Patched Plaything", Zone::Graveyard);
    let zombify = put(&mut game, PLAYER, "Zombify", Zone::Hand);
    cast(&mut game, from_hand, None);
    settle(&mut game);
    assert_eq!(game.counter_count(from_hand, CounterKind::MinusOne), 2);
    assert_eq!(stats(&game, from_hand), (2, 1));
    cast(&mut game, zombify, Some(Target::Card(from_grave)));
    settle(&mut game);
    assert_eq!(game.counter_count(from_grave, CounterKind::MinusOne), 0);
    assert_eq!(stats(&game, from_grave), (4, 3));
    for card in [from_hand, from_grave] {
        assert_eq!(game.object_view(card).kinds, 17);
        assert!(
            game.object_view(card)
                .keywords
                .contains(Keywords::DOUBLE_STRIKE)
        );
    }
}

#[test]
fn savior_of_the_small_returns_its_own_creature_card_at_the_mana_value_boundary() {
    let mut game = scenario(PLAYER);
    let savior = put(&mut game, PLAYER, "Savior of the Small", Zone::Battlefield);
    let legal = put(&mut game, PLAYER, "Inspiring Overseer", Zone::Graveyard);
    let expensive = put(&mut game, PLAYER, "Hill Giant", Zone::Graveyard);
    let foreign = put(&mut game, OPPONENT, "Grizzly Bears", Zone::Graveyard);
    let noncreature = put(&mut game, PLAYER, "Shock", Zone::Graveyard);
    game.state.objects[savior as usize].tapped = true;
    second_main(&mut game);
    for card in [expensive, foreign, noncreature] {
        reject(&mut game, target(card));
    }
    select_target(&mut game, legal);
    settle(&mut game);
    assert_eq!(game.object_view(legal).zone, Zone::Hand);
    assert_eq!(game.object_view(expensive).zone, Zone::Graveyard);
}
