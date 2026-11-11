//! Remaining printed obligations for inherited Seismic Monstrosaur through Swamp.
mod support;

use rustmage as _;
use rustmage_core::{CardName, cards::*, engine::*};
use support::*;

const MAP: Card = Card::from_catalogue_id(5284);

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

fn additional(game: &Game, card: CardId) -> Action {
    Action::ChooseAdditionalCost {
        object: game.object_ref(card),
    }
}

fn attack(game: &mut Game, attacker: CardId) {
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    act(game, Action::Attack { card: attacker });
    act(game, Action::FinishAttackers);
}

fn next_turn(game: &mut Game) {
    let turn = game.state.turn;
    game.begin_step(Step::End);
    game.prepare_triggers();
    advance(game, |game| game.state.turn != turn);
}

#[test]
fn seismic_monstrosaur_and_soaring_sandwing_landcycle_for_a_typed_nonbasic() {
    let p = Player::P0;
    for (name, ability, keyword, size) in [
        ("Seismic Monstrosaur", 1, Keywords::TRAMPLE, (6, 5)),
        ("Soaring Sandwing", 0, Keywords::FLYING, (3, 5)),
    ] {
        let mut root = scenario(p);
        let source = put(&mut root, p, name, Zone::Hand);
        let typed = put(&mut root, p, "Sacred Foundry", Zone::Library);
        let wrong = put(&mut root, p, "Forest", Zone::Library);
        let land = put(&mut root, p, "Island", Zone::Battlefield);
        let creature = put(&mut root, p, "Bear Cub", Zone::Battlefield);
        let foreign = put(&mut root, p.other(), "Island", Zone::Battlefield);
        let mut cycled = root.clone();
        act(&mut cycled, activate(source, ability));
        assert_eq!(cycled.observe(p).payment_request.unwrap().generic, 2);
        pay(&mut cycled);
        assert_eq!(cycled.object_view(source).zone, Zone::Graveyard);
        advance(&mut cycled, |game| {
            matches!(game.state.pending, Pending::EffectSearch { .. })
        });
        reject(&mut cycled, Action::ChooseSearch { card: wrong });
        act(&mut cycled, Action::ChooseSearch { card: typed });
        act(&mut cycled, Action::FinishSearch);
        settle(&mut cycled);
        assert_eq!(cycled.object_view(typed).zone, Zone::Hand);
        assert!(
            cycled
                .observe(p.other())
                .known_hand
                .contains(&cycled.object_ref(typed))
        );
        assert_eq!(cycled.state.players[p.index()].cards_drawn_this_turn, 0);
        assert_eq!(cycled.state.players[p.index()].life, STARTING_LIFE);

        cast(&mut root, source, None);
        settle(&mut root);
        assert_eq!(stats(&root, source), size);
        assert!(root.object_view(source).keywords.contains(keyword));
        if ability == 0 {
            assert_eq!(root.state.players[p.index()].life, STARTING_LIFE + 3);
        } else {
            act(&mut root, activate(source, 0));
            for card in [creature, foreign] {
                let invalid = additional(&root, card);
                reject(&mut root, invalid);
            }
            let payment = additional(&root, land);
            act(&mut root, payment);
            let request = root.observe(p).payment_request.unwrap();
            assert_eq!(request.generic, 2);
            assert_eq!(request.required, [0, 0, 0, 1, 0, 0]);
            pay(&mut root);
            assert_eq!(root.object_view(land).zone, Zone::Graveyard);
            settle(&mut root);
            assert_eq!(root.state.players[p.index()].cards_drawn_this_turn, 1);
        }
    }
}

#[test]
fn self_reflection_flashback_exiles_the_spell_and_copies_characteristics_without_counters() {
    let p = Player::P1;
    let mut game = scenario(p);
    let spell = put(&mut game, p, "Self-Reflection", Zone::Graveyard);
    let source = put(&mut game, p, "Healer's Hawk", Zone::Battlefield);
    let foreign = put(&mut game, p.other(), "Bear Cub", Zone::Battlefield);
    game.add_counter(source, CounterKind::PlusOne, 2);
    act(&mut game, Action::AnnounceFlashback { card: spell });
    reject(&mut game, target(foreign));
    act(&mut game, target(source));
    let request = game.observe(p).payment_request.unwrap();
    assert_eq!(request.generic, 3);
    assert_eq!(request.required, [0, 1, 0, 0, 0, 0]);
    pay(&mut game);
    let mut departed = game.clone();
    departed.move_object(source, Zone::Hand);
    settle(&mut departed);
    assert_eq!(departed.object_view(spell).zone, Zone::Exile);
    assert!(!departed.observe(p).objects.iter().any(ObjectView::is_token));
    settle(&mut game);
    assert_eq!(game.object_view(spell).zone, Zone::Exile);
    let token = game
        .observe(p)
        .objects
        .into_iter()
        .find(|object| object.zone == Zone::Battlefield && object.is_token())
        .unwrap();
    assert_eq!(token.card, game.object_view(source).card);
    assert!(
        token
            .keywords
            .contains(Keywords::FLYING.union(Keywords::LIFELINK))
    );
    assert_eq!((token.power, token.toughness), (1, 1));
}

#[test]
fn sentinel_and_siren_make_a_map_on_entry_and_only_sentinel_repeats_on_attack() {
    let p = Player::P0;
    for (name, keyword, second_map) in [
        ("Sentinel of the Nameless City", Keywords::VIGILANCE, true),
        ("Spyglass Siren", Keywords::FLYING, false),
    ] {
        let mut game = scenario(p);
        let source = put(&mut game, p, name, Zone::Hand);
        cast(&mut game, source, None);
        settle(&mut game);
        assert!(game.object_view(source).keywords.contains(keyword));
        let maps = |game: &Game| {
            game.observe(p)
                .objects
                .into_iter()
                .filter(|o| o.card == MAP && o.zone == Zone::Battlefield && o.controller == p)
                .count()
        };
        assert_eq!(maps(&game), 1);
        game.state.objects[source as usize].summoning_sick = false;
        attack(&mut game, source);
        settle(&mut game);
        assert_eq!(maps(&game), 1 + usize::from(second_map));
        assert_eq!(game.object_view(source).tapped, !second_map);
    }
}

#[test]
fn sorcerous_spyglass_inspects_the_hand_privately_and_leaves_named_mana_abilities_legal() {
    let p = Player::P1;
    let mut game = scenario(p);
    let spell = put(&mut game, p, "Sorcerous Spyglass", Zone::Hand);
    let hidden = put(&mut game, p.other(), "Shock", Zone::Hand);
    let cave = put(&mut game, p, "Captivating Cave", Zone::Battlefield);
    let other_cave = put(&mut game, p.other(), "Captivating Cave", Zone::Battlefield);
    put(&mut game, p, "Bear Cub", Zone::Battlefield);
    assert!(game.legal_actions().contains(&activate(cave, 2)));
    cast(&mut game, spell, None);
    advance(&mut game, |game| game.observe(p).entry_value.is_some());
    let hand = game.observe(p).entry_value.unwrap().opponent_hand.unwrap();
    assert_eq!(
        hand.iter().map(|object| object.id).collect::<Vec<_>>(),
        [hidden]
    );
    let public_choice = game.observe(p.other()).entry_value.unwrap();
    assert_eq!(public_choice.card, game.object_view(spell).card);
    assert!(public_choice.opponent_hand.is_none());
    let named = EntryChoiceValue::CardName(CardName::from_name("Captivating Cave").unwrap());
    act(&mut game, Action::ChooseEntryValue { value: named });
    assert_eq!(game.object_view(spell).entry_choice, Some(named));
    assert!(game.observe(p).known_hand.is_empty());
    reject(&mut game, activate(cave, 2));
    assert!(game.named_activation_prohibited(other_cave));
    choose(
        &mut game,
        |action| matches!(action, Action::ActivateMana { card, production: [0, 0, 0, 0, 0, 1], .. } if *card == cave),
    );
    assert!(game.object_view(cave).tapped);
}

#[test]
fn sovereign_uses_each_creatures_own_power_excess_and_wards_for_two() {
    let p = Player::P0;
    let mut game = scenario(p);
    let sovereign = put(&mut game, p, "Sovereign Okinec Ahau", Zone::Battlefield);
    let grown = put(&mut game, p, "Bear Cub", Zone::Battlefield);
    let unchanged = put(&mut game, p, "Hill Giant", Zone::Battlefield);
    let enemy = put(&mut game, p.other(), "Bear Cub", Zone::Battlefield);
    let pump = put(&mut game, p, "Giant Growth", Zone::Hand);
    let removal = put(&mut game, p.other(), "Murder", Zone::Hand);
    game.add_counter(sovereign, CounterKind::PlusOne, 1);
    game.add_counter(enemy, CounterKind::PlusOne, 1);
    let mut ward = game.clone();
    act(&mut ward, Action::Pass);
    cast(&mut ward, removal, Some(Target::Card(sovereign)));
    advance(&mut ward, |game| {
        game.legal_actions()
            .contains(&Action::DeclineResolutionMana)
    });
    for paid in [false, true] {
        let mut branch = ward.clone();
        if paid {
            choose(
                &mut branch,
                |action| matches!(action, Action::PayResolutionMana { payment } if payment.iter().sum::<u32>() == 2),
            );
        } else {
            act(&mut branch, Action::DeclineResolutionMana);
        }
        settle(&mut branch);
        assert_eq!(branch.object_view(sovereign).zone == Zone::Graveyard, paid);
    }
    cast(&mut game, pump, Some(Target::Card(grown)));
    settle(&mut game);
    attack(&mut game, sovereign);
    settle(&mut game);
    assert_eq!(game.counter_count(sovereign, CounterKind::PlusOne), 2);
    assert_eq!(game.counter_count(grown, CounterKind::PlusOne), 3);
    assert_eq!(game.counter_count(unchanged, CounterKind::PlusOne), 0);
    assert_eq!(game.counter_count(enemy, CounterKind::PlusOne), 1);
}

#[test]
fn spelunking_can_put_the_drawn_cave_onto_the_battlefield_untapped_for_four_life() {
    let p = Player::P0;
    let mut game = scenario(p);
    let spelunking = put(&mut game, p, "Spelunking", Zone::Hand);
    let basic = put(&mut game, p, "Island", Zone::Hand);
    let nonland = put(&mut game, p, "Shock", Zone::Hand);
    let cave = spawn(
        &mut game,
        p,
        Card::from_name("Hidden Courtyard").unwrap(),
        Zone::Library,
    );
    cast(&mut game, spelunking, None);
    advance(&mut game, |game| {
        matches!(game.state.pending, Pending::EffectChoice { .. })
    });
    assert_eq!(game.state.players[p.index()].cards_drawn_this_turn, 1);
    assert_eq!(game.object_view(cave).zone, Zone::Hand);
    let wrong = Action::PutHandLand {
        object: game.object_ref(nonland),
    };
    reject(&mut game, wrong);
    let mut declined = game.clone();
    act(&mut declined, Action::DeclineHandEntry);
    settle(&mut declined);
    assert_eq!(declined.object_view(cave).zone, Zone::Hand);
    for (card, life) in [(basic, 0), (cave, 4)] {
        let mut branch = game.clone();
        let object = branch.object_ref(card);
        act(&mut branch, Action::PutHandLand { object });
        if card == cave {
            let mut tapped = branch.clone();
            let source = tapped.object_ref(spelunking);
            act(
                &mut tapped,
                Action::ChooseEntryReplacement {
                    source,
                    intrinsic: false,
                },
            );
            settle(&mut tapped);
            assert_eq!(tapped.object_view(card).zone, Zone::Battlefield);
            assert!(tapped.object_view(card).tapped);
            assert_eq!(tapped.state.players[p.index()].life, STARTING_LIFE + life);
            act(
                &mut branch,
                Action::ChooseEntryReplacement {
                    source: object,
                    intrinsic: true,
                },
            );
        }
        settle(&mut branch);
        assert_eq!(branch.object_view(card).zone, Zone::Battlefield);
        assert!(!branch.object_view(card).tapped);
        assert_eq!(branch.state.players[p.index()].life, STARTING_LIFE + life);
    }
}

#[test]
fn sawblades_flashes_in_and_chariot_crews_or_taps_two_other_artifacts() {
    let p = Player::P1;
    let foe = p.other();
    let mut game = scenario(foe);
    let source = put(&mut game, p, "Spring-Loaded Sawblades", Zone::Hand);
    let tapped = put(&mut game, foe, "Hill Giant", Zone::Battlefield);
    let untapped = put(&mut game, foe, "Bear Cub", Zone::Battlefield);
    let crew = put(&mut game, p, "Bear Cub", Zone::Battlefield);
    let first = put(&mut game, p, "Swiftfoot Boots", Zone::Battlefield);
    let second = put(&mut game, p, "Soul-Guide Lantern", Zone::Battlefield);
    let material = put(&mut game, p, "Basilisk Collar", Zone::Graveyard);
    for card in [tapped, crew] {
        game.state.objects[card as usize].tapped = true;
    }
    act(&mut game, Action::Pass);
    cast(&mut game, source, None);
    advance(&mut game, |game| {
        matches!(game.state.pending, Pending::Targets { .. })
    });
    reject(&mut game, target(untapped));
    reject(&mut game, target(crew));
    act(&mut game, target(tapped));
    settle(&mut game);
    assert_eq!(game.object_view(tapped).zone, Zone::Graveyard);
    next_turn(&mut game);
    advance(&mut game, |game| game.state.step == Step::Main1);
    game.state.players[p.index()].mana = [8; 6];
    act(&mut game, activate(source, 0));
    act(&mut game, Action::BeginOrderedCosts);
    let clause = game
        .observe(p)
        .cast_costs
        .unwrap()
        .clauses
        .iter()
        .position(|row| row.clause == CastCostClause::Additional)
        .unwrap() as u16;
    act(&mut game, Action::ChooseCastCost { clause });
    let artifact = additional(&game, material);
    act(&mut game, artifact);
    for _ in 0..8 {
        match game.state.pending {
            Pending::CastCosts { .. } => {
                choose(&mut game, |a| matches!(a, Action::ChooseCastCost { .. }))
            }
            Pending::PayCosts { .. } => pay(&mut game),
            Pending::Priority => break,
            _ => panic!("unexpected Craft payment"),
        }
    }
    assert_eq!(game.object_view(material).zone, Zone::Exile);
    settle(&mut game);
    assert_eq!(game.object_view(source).card.name(), "Bladewheel Chariot");
    assert_eq!(game.object_view(source).kinds, 16);
    let mut crewed = game.clone();
    act(&mut crewed, activate(source, 1));
    if matches!(crewed.state.pending, Pending::PayCosts { .. }) {
        pay(&mut crewed);
    }
    let object = crewed.object_ref(crew);
    act(&mut crewed, Action::ChooseCrewer { object });
    act(&mut crewed, Action::FinishCrew);
    settle(&mut crewed);
    assert!(crewed.object_view(crew).tapped);
    assert_eq!(stats(&crewed, source), (5, 5));

    act(&mut game, activate(source, 0));
    let itself = additional(&game, source);
    reject(&mut game, itself);
    for gear in [first, second] {
        let choice = additional(&game, gear);
        act(&mut game, choice);
    }
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(&mut game);
    }
    settle(&mut game);
    assert!(game.object_view(first).tapped && game.object_view(second).tapped);
    assert!(!game.object_view(source).tapped && !game.object_view(crew).tapped);
    assert_eq!(stats(&game, source), (5, 5));
    assert_eq!(game.object_view(source).kinds, 17);
}

#[test]
fn staggering_size_temporarily_increases_both_stats_and_grants_trample() {
    let p = Player::P0;
    let mut game = scenario(p);
    let bear = put(&mut game, p.other(), "Bear Cub", Zone::Battlefield);
    let spell = put(&mut game, p, "Staggering Size", Zone::Hand);
    cast(&mut game, spell, Some(Target::Card(bear)));
    let mut stale = game.clone();
    stale.move_object(bear, Zone::Exile);
    stale.move_object(bear, Zone::Battlefield);
    settle(&mut stale);
    assert_eq!(stats(&stale, bear), (2, 2));
    settle(&mut game);
    assert_eq!(stats(&game, bear), (5, 5));
    assert!(game.object_view(bear).keywords.contains(Keywords::TRAMPLE));
    next_turn(&mut game);
    assert_eq!(stats(&game, bear), (2, 2));
    assert!(!game.object_view(bear).keywords.contains(Keywords::TRAMPLE));
}

#[test]
fn staunch_crewmate_privately_looks_four_and_reveals_either_qualifying_kind() {
    let p = Player::P1;
    let mut game = scenario(p);
    let crewmate = put(&mut game, p, "Staunch Crewmate", Zone::Hand);
    let artifact = put(&mut game, p, "Swiftfoot Boots", Zone::Library);
    let pirate = put(&mut game, p, "Spyglass Siren", Zone::Library);
    let bear = put(&mut game, p, "Bear Cub", Zone::Library);
    let island = put(&mut game, p, "Island", Zone::Library);
    cast(&mut game, crewmate, None);
    advance(&mut game, |game| {
        game.observe(p).library_selection.is_some()
    });
    assert!(game.observe(p.other()).library_selection.is_none());
    reject(&mut game, Action::SelectLibrary { card: bear });
    reject(&mut game, Action::SelectLibrary { card: island });
    for choice in [None, Some(artifact), Some(pirate)] {
        let mut branch = game.clone();
        if let Some(card) = choice {
            act(&mut branch, Action::SelectLibrary { card });
        }
        act(&mut branch, Action::FinishLibrarySelection);
        settle(&mut branch);
        let rest: Vec<_> = [artifact, pirate, bear, island]
            .into_iter()
            .filter(|card| Some(*card) != choice)
            .collect();
        assert!(
            rest.iter()
                .all(|card| branch.state.players[p.index()].library[..rest.len()].contains(card))
        );
        let known = branch.observe(p.other()).known_hand;
        if let Some(card) = choice {
            assert_eq!(branch.object_view(card).zone, Zone::Hand);
            assert_eq!(known, [branch.object_ref(card)]);
        } else {
            assert!(known.is_empty());
        }
    }
}

#[test]
fn stinging_cave_crawler_requires_four_own_permanents_at_attack_and_resolution() {
    let p = Player::P0;
    let mut root = scenario(p);
    let source = put(&mut root, p, "Stinging Cave Crawler", Zone::Battlefield);
    put(&mut root, p, "Opt", Zone::Graveyard);
    put(&mut root, p.other(), "Forest", Zone::Graveyard);
    for name in ["Forest", "Bear Cub", "Swiftfoot Boots"] {
        put(&mut root, p, name, Zone::Graveyard);
    }
    assert!(
        root.object_view(source)
            .keywords
            .contains(Keywords::DEATHTOUCH)
    );
    let mut insufficient = root.clone();
    attack(&mut insufficient, source);
    assert!(insufficient.state.stack.is_empty());
    let fourth = put(&mut root, p, "Forest", Zone::Graveyard);
    attack(&mut root, source);
    assert_eq!(root.state.stack.len(), 1);
    let mut fell_short = root.clone();
    fell_short.move_object(fourth, Zone::Exile);
    settle(&mut fell_short);
    assert_eq!(fell_short.state.players[p.index()].life, STARTING_LIFE);
    assert_eq!(fell_short.state.players[p.index()].cards_drawn_this_turn, 0);
    settle(&mut root);
    assert_eq!(root.state.players[p.index()].life, STARTING_LIFE - 1);
    assert_eq!(root.state.players[p.index()].cards_drawn_this_turn, 1);
}
