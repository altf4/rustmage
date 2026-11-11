//! Rules evidence for previously admitted LCI identities from Abrade through Braided Net.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

const TREASURE: Card = Card::from_catalogue_id(5302);

fn advance(game: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..20 {
        if ready(game) {
            return;
        }
        choose(game, |action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. })
        });
    }
    panic!("scenario did not reach its next decision");
}

fn activate(card: CardId, ability: u16) -> Action {
    Action::AnnounceActivate {
        card,
        ability,
        counter_payment: Counters::default(),
    }
}

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

fn cost(game: &mut Game, card: CardId) {
    let object = game.object_ref(card);
    act(game, Action::ChooseAdditionalCost { object });
}

fn finish_pay(game: &mut Game) {
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(game);
    }
}

fn attack(game: &mut Game, card: CardId) {
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    act(game, Action::Attack { card });
    act(game, Action::FinishAttackers);
}

fn end_turn(game: &mut Game) {
    let current = game.state.turn;
    game.begin_step(Step::End);
    game.prepare_triggers();
    advance(game, |game| game.state.turn != current);
}

#[test]
fn abrade_distinguishes_three_damage_from_artifact_destruction() {
    let p = Player::P0;
    let mut root = scenario(p);
    let creature = put(&mut root, p.other(), "Hill Giant", Zone::Battlefield);
    let artifact = put(&mut root, p.other(), "Swiftfoot Boots", Zone::Battlefield);
    let indestructible = put(&mut root, p.other(), "Diamond Pick-Axe", Zone::Battlefield);
    let spell = put(&mut root, p, "Abrade", Zone::Hand);
    for (mode, valid, invalid, expected) in [
        (0, creature, artifact, Zone::Graveyard),
        (1, artifact, creature, Zone::Graveyard),
        (1, indestructible, creature, Zone::Battlefield),
    ] {
        let mut game = root.clone();
        act(&mut game, Action::AnnounceCast { card: spell });
        act(&mut game, Action::ChooseMode { mode });
        reject(&mut game, target(invalid));
        act(&mut game, target(valid));
        pay(&mut game);
        settle(&mut game);
        assert_eq!(game.object_view(valid).zone, expected);
        assert_eq!(game.object_view(invalid).zone, Zone::Battlefield);
    }
}

#[test]
fn abyssal_gorestalker_requires_apnap_choices_before_a_single_sacrifice_batch() {
    let p = Player::P1;
    let mut game = scenario(p);
    let source = put(&mut game, p, "Abyssal Gorestalker", Zone::Hand);
    let ally = put(&mut game, p, "Bear Cub", Zone::Battlefield);
    let first = put(&mut game, p.other(), "Grizzly Bears", Zone::Battlefield);
    let second = put(&mut game, p.other(), "Bear Cub", Zone::Battlefield);
    let safe = put(&mut game, p.other(), "Hill Giant", Zone::Battlefield);
    cast(&mut game, source, None);
    advance(&mut game, |game| {
        matches!(game.state.pending, Pending::EffectChoice { .. })
    });
    assert_eq!(game.actor(), Some(p));
    assert_eq!(stats(&game, source), (6, 6));
    let wrong = Action::Sacrifice {
        object: game.object_ref(first),
    };
    reject(&mut game, wrong);
    for card in [ally, source, first, second] {
        let object = game.object_ref(card);
        act(&mut game, Action::Sacrifice { object });
        if card == source {
            assert_eq!(game.actor(), Some(p.other()));
            assert_eq!(game.object_view(source).zone, Zone::Battlefield);
            assert_eq!(game.object_view(ally).zone, Zone::Battlefield);
        }
    }
    settle(&mut game);
    for card in [ally, source, first, second] {
        assert_eq!(game.object_view(card).zone, Zone::Graveyard);
    }
    assert_eq!(game.object_view(safe).zone, Zone::Battlefield);
}

#[test]
fn acolyte_taps_and_sacrifices_another_permanent_to_drain_after_it_leaves() {
    let p = Player::P0;
    let mut game = scenario(p);
    let acolyte = put(&mut game, p, "Acolyte of Aclazotz", Zone::Battlefield);
    let gear = put(&mut game, p, "Swiftfoot Boots", Zone::Battlefield);
    let foe = put(&mut game, p.other(), "Bear Cub", Zone::Battlefield);
    game.state.objects[acolyte as usize].summoning_sick = true;
    reject(&mut game, activate(acolyte, 0));
    game.state.objects[acolyte as usize].summoning_sick = false;
    act(&mut game, activate(acolyte, 0));
    for invalid in [acolyte, foe] {
        let action = Action::ChooseAdditionalCost {
            object: game.object_ref(invalid),
        };
        reject(&mut game, action);
    }
    cost(&mut game, gear);
    finish_pay(&mut game);
    assert!(game.object_view(acolyte).tapped);
    assert_eq!(game.object_view(gear).zone, Zone::Graveyard);
    game.move_object(acolyte, Zone::Graveyard);
    settle(&mut game);
    assert_eq!(game.state.players[p.index()].life, STARTING_LIFE + 1);
    assert_eq!(
        game.state.players[p.other().index()].life,
        STARTING_LIFE - 1
    );
}

#[test]
fn acrobatic_leap_untaps_and_grants_temporary_stats_and_flight() {
    let p = Player::P0;
    let mut game = scenario(p);
    let creature = put(&mut game, p, "Bear Cub", Zone::Battlefield);
    game.state.objects[creature as usize].tapped = true;
    let leap = put(&mut game, p, "Acrobatic Leap", Zone::Hand);
    cast(&mut game, leap, Some(Target::Card(creature)));
    settle(&mut game);
    assert!(!game.object_view(creature).tapped);
    assert_eq!(stats(&game, creature), (3, 5));
    assert!(
        game.object_view(creature)
            .keywords
            .contains(Keywords::FLYING)
    );
    end_turn(&mut game);
    assert_eq!(stats(&game, creature), (2, 2));
    assert!(
        !game
            .object_view(creature)
            .keywords
            .contains(Keywords::FLYING)
    );
}

#[test]
fn akawalli_and_basking_capybara_recalculate_their_distinct_descend_bonuses() {
    let p = Player::P1;
    let mut game = scenario(p);
    let akawalli = put(
        &mut game,
        p,
        "Akawalli, the Seething Tower",
        Zone::Battlefield,
    );
    let capybara = put(&mut game, p, "Basking Capybara", Zone::Battlefield);
    put(&mut game, p, "Shock", Zone::Graveyard);
    put(&mut game, p.other(), "Forest", Zone::Graveyard);
    for index in 1..=8 {
        let grave = put(&mut game, p, "Forest", Zone::Graveyard);
        let amount = if index >= 8 {
            7
        } else if index >= 4 {
            5
        } else {
            3
        };
        assert_eq!(stats(&game, akawalli), (amount, amount));
        let keywords = game.object_view(akawalli).keywords;
        assert_eq!(keywords.contains(Keywords::TRAMPLE), index >= 4);
        assert_eq!(keywords.contains(Keywords::MAX_ONE_BLOCKER), index >= 8);
        assert_eq!(stats(&game, capybara), (if index >= 4 { 4 } else { 1 }, 3));
        if index == 8 {
            game.move_object(grave, Zone::Exile);
            assert_eq!(stats(&game, akawalli), (5, 5));
            assert!(
                !game
                    .object_view(akawalli)
                    .keywords
                    .contains(Keywords::MAX_ONE_BLOCKER)
            );
        }
    }
}

#[test]
fn amalia_explores_and_destroys_others_only_at_exactly_twenty_power() {
    let p = Player::P0;
    for (before, survives) in [(19, false), (20, true)] {
        let mut game = scenario(p);
        let amalia = put(&mut game, p, "Amalia Benavides Aguirre", Zone::Battlefield);
        let ally = put(&mut game, p, "Bear Cub", Zone::Battlefield);
        let enemy = put(&mut game, p.other(), "Hill Giant", Zone::Battlefield);
        let card = put(&mut game, p, "Shock", Zone::Library);
        game.add_counter(amalia, CounterKind::PlusOne, before - 2);
        game.gain_life(p, 1);
        game.prepare_triggers();
        advance(&mut game, |game| game.observe(p).explore_choice.is_some());
        let object = game.object_ref(card);
        act(
            &mut game,
            Action::ChooseExplore {
                object,
                graveyard: true,
            },
        );
        settle(&mut game);
        assert_eq!(stats(&game, amalia).0, i128::from(before) + 1);
        assert_eq!(game.object_view(card).zone, Zone::Graveyard);
        for creature in [ally, enemy] {
            assert_eq!(
                game.object_view(creature).zone == Zone::Battlefield,
                survives
            );
        }
    }
    let mut game = scenario(p.other());
    let amalia = put(&mut game, p, "Amalia Benavides Aguirre", Zone::Battlefield);
    let shock = put(&mut game, p.other(), "Shock", Zone::Hand);
    cast(&mut game, shock, Some(Target::Card(amalia)));
    advance(&mut game, |game| {
        game.legal_actions()
            .contains(&Action::DeclineResolutionLife)
    });
    for paid in [false, true] {
        let mut branch = game.clone();
        act(
            &mut branch,
            if paid {
                Action::PayResolutionLife
            } else {
                Action::DeclineResolutionLife
            },
        );
        settle(&mut branch);
        assert_eq!(
            branch.state.players[p.other().index()].life,
            STARTING_LIFE - if paid { 3 } else { 0 }
        );
        assert_eq!(branch.object_view(amalia).zone == Zone::Graveyard, paid);
    }
}

#[test]
fn ancestors_aid_grants_power_and_first_strike_and_creates_one_treasure() {
    let p = Player::P1;
    let mut root = scenario(p);
    let ally = put(&mut root, p, "Bear Cub", Zone::Battlefield);
    let spell = put(&mut root, p, "Ancestors' Aid", Zone::Hand);
    cast(&mut root, spell, Some(Target::Card(ally)));
    let mut departed = root.clone();
    departed.move_object(ally, Zone::Exile);
    settle(&mut departed);
    assert!(
        !departed
            .observe(p)
            .objects
            .iter()
            .any(|object| object.card == TREASURE)
    );
    settle(&mut root);
    assert_eq!(stats(&root, ally), (4, 2));
    assert!(
        root.object_view(ally)
            .keywords
            .contains(Keywords::FIRST_STRIKE)
    );
    let treasures: Vec<_> = root
        .observe(p)
        .objects
        .into_iter()
        .filter(|object| object.card == TREASURE && object.zone == Zone::Battlefield)
        .collect();
    assert_eq!(treasures.len(), 1);
    let treasure = treasures[0].id;
    choose(
        &mut root,
        |action| matches!(action, Action::ActivateMana { card, production: [0, 0, 0, 0, 1, 0], .. } if *card == treasure),
    );
    assert_ne!(root.object_view(treasure).zone, Zone::Battlefield);
}

#[test]
fn ancestral_reminiscence_draws_three_then_allows_only_its_actors_discard() {
    let p = Player::P1;
    let mut game = scenario(p);
    let own = put(&mut game, p, "Bear Cub", Zone::Hand);
    let wrong = put(&mut game, p.other(), "Opt", Zone::Hand);
    let spell = put(&mut game, p, "Ancestral Reminiscence", Zone::Hand);
    cast(&mut game, spell, None);
    advance(&mut game, |game| {
        matches!(game.state.pending, Pending::EffectDiscard { .. })
    });
    assert_eq!(game.actor(), Some(p));
    assert_eq!(game.state.players[p.index()].cards_drawn_this_turn, 3);
    reject(&mut game, Action::Discard { card: wrong });
    act(&mut game, Action::Discard { card: own });
    settle(&mut game);
    assert_eq!(game.object_view(own).zone, Zone::Graveyard);
    assert_eq!(game.object_view(wrong).zone, Zone::Hand);
}

#[test]
fn attentive_sunscribe_scries_privately_after_tapping_as_an_attacker() {
    let p = Player::P0;
    let mut game = scenario(p);
    let source = put(&mut game, p, "Attentive Sunscribe", Zone::Battlefield);
    let top = put(&mut game, p, "Shock", Zone::Library);
    attack(&mut game, source);
    assert!(game.object_view(source).tapped);
    advance(&mut game, |game| game.observe(p).library_choice.is_some());
    assert_eq!(game.observe(p).library_choice.unwrap().remaining[0].id, top);
    assert!(game.observe(p.other()).library_choice.is_none());
    act(
        &mut game,
        Action::ArrangeLibrary {
            card: top,
            destination: LibraryDestination::Bottom,
        },
    );
    settle(&mut game);
    assert_eq!(game.state.players[p.index()].library[0], top);
}

#[test]
fn bartolome_sacrifices_another_creature_or_artifact_even_while_sick() {
    let p = Player::P1;
    let mut game = scenario(p);
    let source = put(&mut game, p, "Bartolomé del Presidio", Zone::Battlefield);
    let creature = put(&mut game, p, "Bear Cub", Zone::Battlefield);
    let artifact = put(&mut game, p, "Swiftfoot Boots", Zone::Battlefield);
    let foe = put(&mut game, p.other(), "Bear Cub", Zone::Battlefield);
    game.state.objects[source as usize].summoning_sick = true;
    for payment in [creature, artifact] {
        act(&mut game, activate(source, 0));
        for wrong in [source, foe] {
            let action = Action::ChooseAdditionalCost {
                object: game.object_ref(wrong),
            };
            reject(&mut game, action);
        }
        cost(&mut game, payment);
        finish_pay(&mut game);
        assert_eq!(game.object_view(payment).zone, Zone::Graveyard);
        if payment == artifact {
            game.move_object(source, Zone::Exile);
            game.move_object(source, Zone::Battlefield);
        }
        settle(&mut game);
        assert_eq!(
            game.counter_count(source, CounterKind::PlusOne),
            u64::from(payment == creature)
        );
    }
}

#[test]
fn belligerent_yearling_can_take_the_entering_dinosaurs_power_without_its_toughness() {
    let p = Player::P0;
    let mut game = scenario(p);
    let source = put(&mut game, p, "Belligerent Yearling", Zone::Battlefield);
    let foreign = put(&mut game, p.other(), "Colossadactyl", Zone::Hand);
    let dinosaur = put(&mut game, p, "Colossadactyl", Zone::Hand);
    game.move_object(foreign, Zone::Battlefield);
    game.prepare_triggers();
    assert!(game.state.stack.is_empty());
    cast(&mut game, dinosaur, None);
    advance(&mut game, |game| {
        game.legal_actions()
            .contains(&Action::ChooseOptional { accept: false })
    });
    let before = game.snapshot();
    act(&mut game, Action::ChooseOptional { accept: false });
    settle(&mut game);
    assert_eq!(stats(&game, source), (3, 2));
    game.restore(&before).unwrap();
    game.add_counter(dinosaur, CounterKind::PlusOne, 2);
    game.move_object(dinosaur, Zone::Graveyard);
    act(&mut game, Action::ChooseOptional { accept: true });
    settle(&mut game);
    assert_eq!(stats(&game, source), (6, 2));
    assert!(
        game.object_view(source)
            .keywords
            .contains(Keywords::TRAMPLE)
    );
    end_turn(&mut game);
    assert_eq!(stats(&game, source), (3, 2));
}

#[test]
fn bitter_triumph_pays_exactly_one_additional_cost_and_destroys_either_type() {
    let p = Player::P0;
    let mut root = scenario(p);
    let creature = put(&mut root, p.other(), "Bear Cub", Zone::Battlefield);
    let walker = put(
        &mut root,
        p.other(),
        "Chandra, Flameshaper",
        Zone::Battlefield,
    );
    root.add_counter(walker, CounterKind::Loyalty, 6);
    let invalid = put(&mut root, p.other(), "Swiftfoot Boots", Zone::Battlefield);
    let discard = put(&mut root, p, "Forest", Zone::Hand);
    let spell = put(&mut root, p, "Bitter Triumph", Zone::Hand);
    for (life, victim) in [(false, creature), (true, walker)] {
        let mut game = root.clone();
        act(&mut game, Action::AnnounceCast { card: spell });
        reject(&mut game, target(invalid));
        act(&mut game, target(victim));
        act(&mut game, Action::BeginOrderedCosts);
        let additional = game
            .observe(p)
            .cast_costs
            .unwrap()
            .clauses
            .iter()
            .position(|row| row.clause == CastCostClause::Additional)
            .unwrap() as u16;
        act(&mut game, Action::ChooseCastCost { clause: additional });
        if life {
            act(&mut game, Action::ChooseAdditionalLife);
        } else {
            cost(&mut game, discard);
        }
        if matches!(game.state.pending, Pending::CastCosts { .. }) {
            choose(&mut game, |action| {
                matches!(action, Action::ChooseCastCost { .. })
            });
        }
        pay(&mut game);
        settle(&mut game);
        assert_eq!(
            game.state.players[p.index()].life,
            STARTING_LIFE - if life { 3 } else { 0 }
        );
        assert_eq!(
            game.object_view(discard).zone,
            if life { Zone::Hand } else { Zone::Graveyard }
        );
        assert_eq!(game.object_view(victim).zone, Zone::Graveyard);
    }
}

#[test]
fn braided_net_spends_a_net_counter_until_untap_then_crafts_a_drawing_quipu() {
    let p = Player::P0;
    let mut game = scenario(p);
    let net = put(&mut game, p, "Braided Net", Zone::Hand);
    let victim = put(&mut game, p, "Llanowar Elves", Zone::Battlefield);
    let gear = put(&mut game, p, "Swiftfoot Boots", Zone::Graveyard);
    let allied_artifact = put(&mut game, p, "Soul-Guide Lantern", Zone::Battlefield);
    put(&mut game, p.other(), "Swiftfoot Boots", Zone::Battlefield);
    cast(&mut game, net, None);
    settle(&mut game);
    assert_eq!(game.counter_count(net, CounterKind::Net), 3);
    let payment = Counters::of(CounterKind::Net, 1);
    act(
        &mut game,
        Action::AnnounceActivate {
            card: net,
            ability: 0,
            counter_payment: payment,
        },
    );
    act(&mut game, target(victim));
    finish_pay(&mut game);
    settle(&mut game);
    assert_eq!(game.counter_count(net, CounterKind::Net), 2);
    assert!(game.object_view(victim).tapped);
    game.untap_objects(&[victim], untap_rules::UntapCause::Effect);
    act(&mut game, Action::TapMana { card: victim });
    act(&mut game, activate(net, 1));
    act(&mut game, Action::BeginOrderedCosts);
    let additional = game
        .observe(p)
        .cast_costs
        .unwrap()
        .clauses
        .iter()
        .position(|row| row.clause == CastCostClause::Additional)
        .unwrap() as u16;
    act(&mut game, Action::ChooseCastCost { clause: additional });
    cost(&mut game, gear);
    while game.state.announcing.is_some() {
        match game.state.pending {
            Pending::CastCosts { .. } => choose(&mut game, |action| {
                matches!(action, Action::ChooseCastCost { .. })
            }),
            Pending::PayCosts { .. } => pay(&mut game),
            _ => panic!("unexpected Craft cost"),
        }
    }
    settle(&mut game);
    assert_eq!(game.object_view(gear).zone, Zone::Exile);
    assert_eq!(game.object_view(net).card.name(), "Braided Quipu");
    act(&mut game, activate(net, 0));
    pay(&mut game);
    game.move_object(allied_artifact, Zone::Graveyard);
    settle(&mut game);
    assert_eq!(game.state.players[p.index()].cards_drawn_this_turn, 1);
    let library = &game.state.players[p.index()].library;
    assert_eq!(library[library.len() - 3], net);
}
