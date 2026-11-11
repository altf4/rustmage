//! Complete two-face scenarios for Explore, survivor selection and paid mana provenance.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

const GNOME_SOLDIER: Card = Card::from_catalogue_id(5573);

fn candidate(game: &mut Game, owner: Player, name: &str, zone: Zone) -> CardId {
    spawn(game, owner, Card::from_name(name).unwrap(), zone)
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

fn step(game: &mut Game, next: Step) {
    game.begin_step(next);
    game.prepare_triggers();
}

fn object_choice(game: &Game, card: CardId) -> Action {
    Action::ChooseEffectObject {
        object: game.object_ref(card),
    }
}

fn color_mana(game: &mut Game, producer: CardId, expected: Mana) {
    choose(
        game,
        |action| matches!(action, Action::ActivateMana { card, production, .. } if *card == producer && *production == expected),
    );
}

#[test]
fn twists_and_turns_scries_privately_before_one_explore_then_transforms_at_seven_lands() {
    let p = Player::P0;
    let foe = p.other();
    let mut game = scenario(p);
    let name = "Twists and Turns // Mycoid Maze";
    let source = candidate(&mut game, p, name, Zone::Hand);
    let second_provider = candidate(&mut game, p, name, Zone::Battlefield);
    candidate(&mut game, foe, name, Zone::Battlefield);
    let own = put(&mut game, p, "Bear Cub", Zone::Battlefield);
    let enemy = put(&mut game, foe, "Hill Giant", Zone::Battlefield);
    let second = put(&mut game, p, "Island", Zone::Library);
    let first = put(&mut game, p, "Shock", Zone::Library);
    cast(&mut game, source, None);
    advance(&mut game, |game| {
        matches!(game.state.pending, Pending::Targets { .. })
    });
    reject(&mut game, target(enemy));
    act(&mut game, target(own));
    advance(&mut game, |game| game.observe(p).library_choice.is_some());
    for (card, destination) in [
        (first, LibraryDestination::Bottom),
        (second, LibraryDestination::Top),
    ] {
        assert_eq!(
            game.observe(p).library_choice.unwrap().remaining[0].id,
            card
        );
        assert!(game.observe(foe).library_choice.is_none());
        assert!(game.observe(foe).last_explore.is_none());
        act(&mut game, Action::ArrangeLibrary { card, destination });
    }
    settle(&mut game);
    assert_eq!(game.object_view(second).zone, Zone::Hand);
    assert_eq!(game.counter_count(own, CounterKind::PlusOne), 0);
    let explore = game.observe(p).last_explore.unwrap();
    assert_eq!(explore.sequence, 1);
    assert_eq!(game.observe(foe).last_explore, Some(explore));
    game.move_object(second_provider, Zone::Graveyard);

    for _ in 0..5 {
        put(&mut game, p, "Forest", Zone::Battlefield);
    }
    let sixth = put(&mut game, p, "Forest", Zone::Hand);
    let seventh = put(&mut game, p, "Forest", Zone::Hand);
    let foreign = put(&mut game, foe, "Forest", Zone::Hand);
    for card in [sixth, foreign] {
        game.move_object(card, Zone::Battlefield);
        game.prepare_triggers();
        assert!(game.state.stack.is_empty());
    }
    game.move_object(seventh, Zone::Battlefield);
    game.prepare_triggers();
    assert_eq!(game.state.stack.len(), 1);
    let mut fell_short = game.clone();
    fell_short.move_object(sixth, Zone::Exile);
    settle(&mut fell_short);
    assert_eq!(fell_short.object_view(source).kinds, 32);
    settle(&mut game);
    let maze = game.object_view(source);
    assert_eq!(maze.card.name(), "Mycoid Maze");
    assert_eq!(maze.kinds, 2);
    assert!(maze.has_subtype("Cave"));

    let mut mana = game.clone();
    let initial = mana.state.players[p.index()].mana[4];
    color_mana(&mut mana, source, [0, 0, 0, 0, 1, 0]);
    assert_eq!(mana.state.players[p.index()].mana[4], initial + 1);
    assert!(mana.object_view(source).tapped);

    let group = ["Opt", "Hill Giant", "Plains", "Bear Cub"]
        .map(|name| put(&mut game, p, name, Zone::Library));
    act(&mut game, activate(source, 1));
    let request = game.observe(p).payment_request.unwrap();
    assert_eq!(request.generic, 3);
    assert_eq!(request.required, [0, 0, 0, 0, 1, 0]);
    pay(&mut game);
    advance(&mut game, |game| {
        game.observe(p).library_selection.is_some()
    });
    assert!(game.observe(foe).library_selection.is_none());
    reject(&mut game, Action::SelectLibrary { card: group[0] });
    let known_before = game.observe(foe).known_hand;
    let mut declined = game.clone();
    act(&mut declined, Action::FinishLibrarySelection);
    assert_eq!(declined.observe(foe).known_hand, known_before);
    assert!(
        group
            .iter()
            .all(|card| declined.object_view(*card).zone == Zone::Library)
    );
    act(&mut game, Action::SelectLibrary { card: group[1] });
    act(&mut game, Action::FinishLibrarySelection);
    settle(&mut game);
    assert_eq!(game.object_view(group[1]).zone, Zone::Hand);
    assert!(
        game.observe(foe)
            .known_hand
            .contains(&game.object_ref(group[1]))
    );
    let bottom = &game.state.players[p.index()].library[..3];
    assert!(bottom.contains(&group[0]) && bottom.contains(&group[2]) && bottom.contains(&group[3]));
}

#[test]
fn unstable_glyphbridge_chooses_survivors_when_cast_and_crafts_into_the_flying_face() {
    let p = Player::P1;
    let foe = p.other();
    let mut game = scenario(p);
    let source = candidate(
        &mut game,
        p,
        "Unstable Glyphbridge // Sandswirl Wanderglyph",
        Zone::Hand,
    );
    let own = put(&mut game, p, "Bear Cub", Zone::Battlefield);
    let own_large = put(&mut game, p, "Hill Giant", Zone::Battlefield);
    let other = put(&mut game, foe, "Grizzly Bears", Zone::Battlefield);
    let other_small = put(&mut game, foe, "Bear Cub", Zone::Battlefield);
    game.grant_keyword_effects(&[other], game.object_ref(other), Keywords::HEXPROOF);
    let material = put(&mut game, p, "Swiftfoot Boots", Zone::Graveyard);
    let wrong_material = put(&mut game, p, "Bear Cub", Zone::Graveyard);
    let mut uncast = game.clone();
    uncast.move_object(source, Zone::Battlefield);
    uncast.prepare_triggers();
    assert!(uncast.state.stack.is_empty());
    assert_eq!(uncast.object_view(own_large).zone, Zone::Battlefield);
    cast(&mut game, source, None);
    advance(&mut game, |game| {
        matches!(game.state.pending, Pending::EffectChoice { .. })
    });
    assert_eq!(game.actor(), Some(p));
    reject(&mut game, Action::DeclineEffectPayment);
    let too_big = object_choice(&game, own_large);
    reject(&mut game, too_big);
    for card in [other, own] {
        let choice = object_choice(&game, card);
        act(&mut game, choice);
        if card == other {
            assert_eq!(game.object_view(own_large).zone, Zone::Battlefield);
            assert_eq!(game.object_view(other_small).zone, Zone::Battlefield);
        }
    }
    settle(&mut game);
    assert_eq!(game.object_view(own_large).zone, Zone::Graveyard);
    assert_eq!(game.object_view(other_small).zone, Zone::Graveyard);
    assert_eq!(game.object_view(own).zone, Zone::Battlefield);
    assert_eq!(game.object_view(other).zone, Zone::Battlefield);

    act(&mut game, activate(source, 0));
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
    let wrong = Action::ChooseAdditionalCost {
        object: game.object_ref(wrong_material),
    };
    reject(&mut game, wrong);
    let payment = Action::ChooseAdditionalCost {
        object: game.object_ref(material),
    };
    act(&mut game, payment);
    for _ in 0..8 {
        match game.state.pending {
            Pending::CastCosts { .. } => choose(&mut game, |action| {
                matches!(action, Action::ChooseCastCost { .. })
            }),
            Pending::PayCosts { .. } => pay(&mut game),
            Pending::Priority => break,
            _ => panic!("unexpected Craft cost"),
        }
    }
    assert_eq!(game.object_view(source).zone, Zone::Exile);
    assert_eq!(game.object_view(material).zone, Zone::Exile);
    settle(&mut game);
    let back = game.object_view(source);
    assert_eq!(back.card.name(), "Sandswirl Wanderglyph");
    assert_eq!(back.kinds, 17);
    assert!(back.has_subtype("Golem") && back.keywords.contains(Keywords::FLYING));
    assert_eq!(stats(&game, source), (5, 3));
}

#[test]
fn sandswirl_wanderglyph_checks_which_defender_was_attacked_and_when_the_spell_was_cast() {
    let active = Player::P0;
    let defender = active.other();
    let mut root = scenario(active);
    let source = candidate(
        &mut root,
        defender,
        "Sandswirl Wanderglyph",
        Zone::Battlefield,
    );
    let attacker = put(&mut root, active, "Bear Cub", Zone::Battlefield);
    let walker = put(
        &mut root,
        defender,
        "Ajani, Caller of the Pride",
        Zone::Battlefield,
    );
    let spell = put(&mut root, active, "Healer's Hawk", Zone::Hand);
    let instant = put(&mut root, active, "Shock", Zone::Hand);
    let battle = candidate(&mut root, active, "@fixture/siege-front", Zone::Battlefield);
    root.add_counter(walker, CounterKind::Loyalty, 4);
    root.add_counter(battle, CounterKind::Defense, 9);
    let walker_ref = root.object_ref(walker);
    let battle_ref = root.object_ref(battle);
    root.set_battle_protector(battle_ref, defender);
    let attack_walker = Action::AttackPlaneswalker {
        card: attacker,
        defender: walker_ref,
    };
    let attack_battle = Action::AttackBattle {
        card: attacker,
        defender: battle_ref,
    };

    let mut cast_first = root.clone();
    cast(&mut cast_first, spell, None);
    assert_eq!(cast_first.state.stack.len(), 2);
    cast_first.move_object(source, Zone::Graveyard);
    settle(&mut cast_first);
    cast_first.state.step = Step::Attackers;
    cast_first.state.pending = Pending::Attackers;
    reject(&mut cast_first, Action::Attack { card: attacker });
    reject(&mut cast_first, attack_walker.clone());
    act(&mut cast_first, attack_battle.clone());

    for attack in [attack_walker, attack_battle] {
        let mut game = root.clone();
        let hits_walker = matches!(&attack, Action::AttackPlaneswalker { .. });
        game.state.step = Step::Attackers;
        game.state.pending = Pending::Attackers;
        act(&mut game, attack);
        act(&mut game, Action::FinishAttackers);
        if hits_walker {
            reject(&mut game, Action::AnnounceCast { card: instant });
            game.move_object(source, Zone::Graveyard);
        }
        cast(&mut game, instant, Some(Target::Player(defender)));
        settle(&mut game);
        assert_eq!(game.state.players[defender.index()].life, STARTING_LIFE - 2);
    }
}

fn soldiers(game: &Game, player: Player) -> Vec<ObjectView> {
    game.observe(player)
        .objects
        .into_iter()
        .filter(|object| {
            object.card == GNOME_SOLDIER
                && object.zone == Zone::Battlefield
                && object.controller == player
        })
        .collect()
}

#[test]
fn smithy_taps_five_distinct_permanents_and_barracks_checks_the_mana_actually_spent() {
    let p = Player::P0;
    let foe = p.other();
    let mut game = scenario(p);
    let source = candidate(
        &mut game,
        p,
        "Thousand Moons Smithy // Barracks of the Thousand",
        Zone::Hand,
    );
    let ally = put(&mut game, p, "Bear Cub", Zone::Battlefield);
    let gear = put(&mut game, p, "Swiftfoot Boots", Zone::Battlefield);
    let enemy = put(
        &mut game,
        foe,
        "Cartographer's Companion",
        Zone::Battlefield,
    );
    put(&mut game, p, "Plains", Zone::Battlefield);
    cast(&mut game, source, None);
    settle(&mut game);
    let first = soldiers(&game, p);
    assert_eq!(first.len(), 1);
    let token = first[0].id;
    assert_eq!(first[0].kinds, 17);
    assert_eq!(first[0].colors, 1);
    assert!(first[0].has_subtype("Gnome") && first[0].has_subtype("Soldier"));
    assert_eq!(stats(&game, token), (4, 4));
    let mut short = game.clone();
    step(&mut short, Step::Main1);
    settle(&mut short);
    assert_eq!(short.object_view(source).kinds, 16);
    assert!(!short.object_view(source).tapped);

    let fifth = put(&mut game, p, "Cartographer's Companion", Zone::Battlefield);
    assert_eq!(stats(&game, token), (5, 5));
    step(&mut game, Step::Main1);
    advance(&mut game, |game| {
        matches!(game.state.pending, Pending::EffectChoice { .. })
    });
    let mut declined = game.clone();
    act(&mut declined, Action::DeclineEffectPayment);
    settle(&mut declined);
    assert!(!declined.object_view(source).tapped);
    let wrong = Action::Sacrifice {
        object: game.object_ref(enemy),
    };
    reject(&mut game, wrong);
    for card in [source, ally, gear, token, fifth] {
        let choice = Action::Sacrifice {
            object: game.object_ref(card),
        };
        act(&mut game, choice);
        if card == source {
            let duplicate = Action::Sacrifice {
                object: game.object_ref(card),
            };
            reject(&mut game, duplicate);
        }
    }
    settle(&mut game);
    let barracks = game.object_view(source);
    assert_eq!(barracks.card.name(), "Barracks of the Thousand");
    assert_eq!(barracks.kinds, 2 | 16);
    assert!(
        [source, ally, gear, token, fifth]
            .into_iter()
            .all(|card| game.object_view(card).tapped)
    );
    game.untap_objects(&[source], untap_rules::UntapCause::Effect);

    for (name, target, qualifies) in [
        ("Healer's Hawk", None, true),
        ("Basilisk Collar", None, true),
        ("Acrobatic Leap", Some(Target::Card(token)), false),
    ] {
        let mut branch = game.clone();
        branch.state.players[p.index()].mana = [0; 6];
        color_mana(&mut branch, source, [1, 0, 0, 0, 0, 0]);
        let spell = put(&mut branch, p, name, Zone::Hand);
        cast(&mut branch, spell, target);
        settle(&mut branch);
        assert_eq!(soldiers(&branch, p).len(), 1 + usize::from(qualifies));
    }
    game.state.players[p.index()].mana = [1, 0, 0, 0, 0, 0];
    color_mana(&mut game, source, [1, 0, 0, 0, 0, 0]);
    let spell = put(&mut game, p, "Healer's Hawk", Zone::Hand);
    cast(&mut game, spell, None);
    act(&mut game, Action::ChooseManaLotSpend { amount: 0 });
    settle(&mut game);
    assert_eq!(soldiers(&game, p).len(), 1);
}
