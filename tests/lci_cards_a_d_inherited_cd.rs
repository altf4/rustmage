//! Rules evidence for admitted LCI identities from Captivating Cave through Diamond Pick-Axe.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

const MAP: Card = Card::from_catalogue_id(5284);
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

fn attack(game: &mut Game, card: CardId) {
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    act(game, Action::Attack { card });
    act(game, Action::FinishAttackers);
}

fn finish_pay(game: &mut Game) {
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(game);
    }
}

fn end_turn(game: &mut Game) {
    let turn = game.state.turn;
    game.begin_step(Step::End);
    game.prepare_triggers();
    advance(game, |game| game.state.turn != turn);
}

fn token(game: &Game, player: Player, card: Card) -> CardId {
    game.observe(player)
        .objects
        .into_iter()
        .find(|object| {
            object.zone == Zone::Battlefield
                && object.controller == player
                && object.card == card
                && object.is_token()
        })
        .expect("expected battlefield token")
        .id
}

#[test]
fn captivating_cave_has_two_mana_routes_and_pays_itself_for_two_counters() {
    let p = Player::P0;
    let mut game = scenario(p);
    let cave = put(&mut game, p, "Captivating Cave", Zone::Battlefield);
    let enemy = put(&mut game, p.other(), "Bear Cub", Zone::Battlefield);
    assert!(game.object_view(cave).has_subtype("Cave"));
    let mut colorless = game.clone();
    choose(
        &mut colorless,
        |action| matches!(action, Action::ActivateMana { card, ability: 0, production: [0, 0, 0, 0, 0, 1], .. } if *card == cave),
    );
    assert_eq!(colorless.state.players[p.index()].mana[5], 9);
    assert!(colorless.object_view(cave).tapped);
    reject(&mut colorless, activate(cave, 2));

    let mut filtered = game.clone();
    let initial = filtered.state.players[p.index()].mana;
    let blue = Action::ActivateMana {
        card: cave,
        ability: 1,
        payment: [0, 0, 0, 0, 0, 1],
        counter_payment: Counters::default(),
        production: [0, 1, 0, 0, 0, 0],
    };
    act(&mut filtered, blue);
    assert_eq!(filtered.state.players[p.index()].mana[5], initial[5] - 1);
    assert_eq!(filtered.state.players[p.index()].mana[1], initial[1] + 1);
    game.state.step = Step::End;
    reject(&mut game, activate(cave, 2));
    game.state.step = Step::Main1;
    act(&mut game, activate(cave, 2));
    act(&mut game, target(enemy));
    assert_eq!(game.observe(p).payment_request.unwrap().generic, 4);
    pay(&mut game);
    assert_eq!(game.object_view(cave).zone, Zone::Graveyard);
    settle(&mut game);
    assert_eq!(game.counter_count(enemy, CounterKind::PlusOne), 2);
}

#[test]
fn careening_mine_cart_can_crew_with_a_sick_creature_and_rewards_its_own_attack() {
    let p = Player::P1;
    let mut game = scenario(p);
    let cart = put(&mut game, p, "Careening Mine Cart", Zone::Battlefield);
    let crew = put(&mut game, p, "Bear Cub", Zone::Battlefield);
    game.state.objects[crew as usize].summoning_sick = true;
    act(&mut game, activate(cart, 0));
    finish_pay(&mut game);
    let object = game.object_ref(crew);
    act(&mut game, Action::ChooseCrewer { object });
    if game.legal_actions().contains(&Action::FinishCrew) {
        act(&mut game, Action::FinishCrew);
    }
    settle(&mut game);
    assert!(game.object_view(crew).tapped);
    assert_eq!(game.object_view(cart).kinds, 17);
    assert_eq!(stats(&game, cart), (3, 3));
    attack(&mut game, cart);
    settle(&mut game);
    assert_eq!(game.object_view(token(&game, p, TREASURE)).kinds, 16);
    end_turn(&mut game);
    assert_eq!(game.object_view(cart).kinds, 16);
}

#[test]
fn cartographers_companion_creates_a_map_that_explores_only_with_an_own_creature() {
    let p = Player::P0;
    let mut game = scenario(p);
    let card = put(&mut game, p, "Cartographer's Companion", Zone::Hand);
    let foe = put(&mut game, p.other(), "Bear Cub", Zone::Battlefield);
    let land = put(&mut game, p, "Island", Zone::Library);
    cast(&mut game, card, None);
    settle(&mut game);
    assert_eq!(stats(&game, card), (2, 1));
    let map = token(&game, p, MAP);
    let view = game.object_view(map);
    assert_eq!(view.kinds, 16);
    assert!(view.has_subtype("Map"));
    game.state.step = Step::End;
    reject(&mut game, activate(map, 0));
    game.state.step = Step::Main1;
    act(&mut game, activate(map, 0));
    reject(&mut game, target(foe));
    act(&mut game, target(card));
    pay(&mut game);
    assert_ne!(game.object_view(map).zone, Zone::Battlefield);
    settle(&mut game);
    assert_eq!(game.object_view(land).zone, Zone::Hand);
    assert_eq!(game.counter_count(card, CounterKind::PlusOne), 0);
}

#[test]
fn cavern_stomper_privately_orders_two_cards_and_can_exclude_small_blockers() {
    let p = Player::P0;
    let mut game = scenario(p);
    let stomper = put(&mut game, p, "Cavern Stomper", Zone::Hand);
    let low = put(&mut game, p.other(), "Bear Cub", Zone::Battlefield);
    let high = put(&mut game, p.other(), "Hill Giant", Zone::Battlefield);
    let second = put(&mut game, p, "Opt", Zone::Library);
    let first = put(&mut game, p, "Shock", Zone::Library);
    cast(&mut game, stomper, None);
    advance(&mut game, |game| game.observe(p).library_choice.is_some());
    assert_eq!(game.observe(p).library_choice.unwrap().remaining.len(), 2);
    assert!(game.observe(p.other()).library_choice.is_none());
    act(
        &mut game,
        Action::ArrangeLibrary {
            card: first,
            destination: LibraryDestination::Bottom,
        },
    );
    act(
        &mut game,
        Action::ArrangeLibrary {
            card: second,
            destination: LibraryDestination::Top,
        },
    );
    settle(&mut game);
    let library = &game.state.players[p.index()].library;
    assert_eq!(library[0], first);
    assert_eq!(library.last().copied(), Some(second));
    assert_eq!(stats(&game, stomper), (7, 7));
    act(&mut game, activate(stomper, 0));
    pay(&mut game);
    settle(&mut game);
    game.state.objects[stomper as usize].summoning_sick = false;
    attack(&mut game, stomper);
    advance(&mut game, |game| {
        matches!(game.state.pending, Pending::Blockers)
    });
    reject(
        &mut game,
        Action::Block {
            blocker: low,
            attacker: stomper,
        },
    );
    act(
        &mut game,
        Action::Block {
            blocker: high,
            attacker: stomper,
        },
    );
}

#[test]
fn cavern_of_souls_restricts_colored_mana_and_protects_only_the_matching_spell_paid_with_it() {
    let p = Player::P1;
    let mut root = scenario(p);
    root.state.players[p.index()].mana = [0, 0, 0, 0, 1, 0];
    let cave = put(&mut root, p, "Cavern of Souls", Zone::Hand);
    let bear = put(&mut root, p, "Bear Cub", Zone::Hand);
    let wrong = put(&mut root, p, "Llanowar Elves", Zone::Hand);
    let counter = put(&mut root, p.other(), "Essence Scatter", Zone::Hand);
    act(&mut root, Action::PlayLand { card: cave });
    let bear_type = CreatureType::from_name("Bear").unwrap();
    act(
        &mut root,
        Action::ChooseEntryValue {
            value: EntryChoiceValue::CreatureType(bear_type),
        },
    );
    assert_eq!(
        root.object_view(cave).entry_choice,
        Some(EntryChoiceValue::CreatureType(bear_type))
    );
    for (ability, production) in [(0, [0, 0, 0, 0, 0, 1]), (1, [0, 0, 0, 0, 1, 0])] {
        let mut game = root.clone();
        choose(
            &mut game,
            |action| matches!(action, Action::ActivateMana { card, ability: actual, production: mana, .. } if *card == cave && *actual == ability && *mana == production),
        );
        if ability == 1 {
            let mut restricted = game.clone();
            restricted.state.players[p.index()].mana[4] = 1;
            reject(&mut restricted, Action::AnnounceCast { card: wrong });
        }
        cast(&mut game, bear, None);
        act(&mut game, Action::Pass);
        cast(&mut game, counter, Some(Target::Spell(bear)));
        settle(&mut game);
        assert_eq!(
            game.object_view(bear).zone,
            if ability == 1 {
                Zone::Battlefield
            } else {
                Zone::Graveyard
            }
        );
    }
}

#[test]
fn cenote_scout_explores_a_land_or_nonland_on_entry() {
    let p = Player::P0;
    for (name, land, mill) in [
        ("Island", true, false),
        ("Opt", false, false),
        ("Shock", false, true),
    ] {
        let mut game = scenario(p);
        let top = put(&mut game, p, name, Zone::Library);
        let scout = put(&mut game, p, "Cenote Scout", Zone::Hand);
        cast(&mut game, scout, None);
        if !land {
            advance(&mut game, |game| game.observe(p).explore_choice.is_some());
            let object = game.object_ref(top);
            act(
                &mut game,
                Action::ChooseExplore {
                    object,
                    graveyard: mill,
                },
            );
        }
        settle(&mut game);
        assert_eq!(
            game.object_view(top).zone,
            if land {
                Zone::Hand
            } else if mill {
                Zone::Graveyard
            } else {
                Zone::Library
            }
        );
        assert_eq!(
            game.counter_count(scout, CounterKind::PlusOne),
            u64::from(!land)
        );
        assert_eq!(game.observe(p).last_explore.unwrap().sequence, 1);
    }
}

#[test]
fn chart_a_course_draws_twice_and_discards_only_without_an_actual_attack() {
    let p = Player::P0;
    for attacked in [false, true] {
        let mut game = scenario(p);
        let attacker = put(&mut game, p, "Bear Cub", Zone::Battlefield);
        let discard = put(&mut game, p, "Shock", Zone::Hand);
        let chart = put(&mut game, p, "Chart a Course", Zone::Hand);
        if attacked {
            attack(&mut game, attacker);
            game.begin_step(Step::Main2);
        }
        cast(&mut game, chart, None);
        if !attacked {
            advance(&mut game, |game| {
                matches!(game.state.pending, Pending::EffectDiscard { .. })
            });
            act(&mut game, Action::Discard { card: discard });
        }
        settle(&mut game);
        assert_eq!(game.state.players[p.index()].cards_drawn_this_turn, 2);
        assert_eq!(
            game.object_view(discard).zone,
            if attacked {
                Zone::Hand
            } else {
                Zone::Graveyard
            }
        );
    }
}

#[test]
fn coati_scavenger_rechecks_four_own_permanents_and_returns_only_a_legal_card() {
    let p = Player::P1;
    let mut game = scenario(p);
    let creature = put(&mut game, p, "Coati Scavenger", Zone::Hand);
    let card = put(&mut game, p, "Island", Zone::Graveyard);
    let fallback = put(&mut game, p, "Bear Cub", Zone::Graveyard);
    let third = put(&mut game, p, "Forest", Zone::Graveyard);
    put(&mut game, p, "Swiftfoot Boots", Zone::Graveyard);
    let instant = put(&mut game, p, "Shock", Zone::Graveyard);
    let foe = put(&mut game, p.other(), "Island", Zone::Graveyard);
    cast(&mut game, creature, None);
    advance(&mut game, |game| {
        matches!(game.state.pending, Pending::Targets { .. })
    });
    reject(&mut game, target(instant));
    reject(&mut game, target(foe));
    act(&mut game, target(card));
    let mut fell_short = game.clone();
    fell_short.move_object(third, Zone::Exile);
    settle(&mut fell_short);
    assert_eq!(fell_short.object_view(card).zone, Zone::Graveyard);
    settle(&mut game);
    assert_eq!(game.object_view(card).zone, Zone::Hand);
    assert_eq!(game.object_view(fallback).zone, Zone::Graveyard);
}

#[test]
fn cogwork_wrestler_casts_off_turn_and_reduces_only_an_opposing_creatures_power() {
    let p = Player::P1;
    let mut game = scenario(p.other());
    let source = put(&mut game, p, "Cogwork Wrestler", Zone::Hand);
    let own = put(&mut game, p, "Bear Cub", Zone::Battlefield);
    let foe = put(&mut game, p.other(), "Hill Giant", Zone::Battlefield);
    act(&mut game, Action::Pass);
    cast(&mut game, source, None);
    advance(&mut game, |game| {
        matches!(game.state.pending, Pending::Targets { .. })
    });
    reject(&mut game, target(own));
    act(&mut game, target(foe));
    settle(&mut game);
    assert_eq!(stats(&game, foe), (1, 3));
    assert_eq!(stats(&game, source), (1, 2));
    end_turn(&mut game);
    assert_eq!(stats(&game, foe), (3, 3));
}

#[test]
fn colossadactyl_can_block_a_flyer_and_trample_through_a_smaller_blocker() {
    let p = Player::P0;
    let mut game = scenario(p.other());
    let dinosaur = put(&mut game, p, "Colossadactyl", Zone::Battlefield);
    let flyer = put(&mut game, p.other(), "Healer's Hawk", Zone::Battlefield);
    assert_eq!(stats(&game, dinosaur), (4, 5));
    assert!(
        game.object_view(dinosaur)
            .keywords
            .contains(Keywords::REACH.union(Keywords::TRAMPLE))
    );
    attack(&mut game, flyer);
    advance(&mut game, |game| {
        matches!(game.state.pending, Pending::Blockers)
    });
    act(
        &mut game,
        Action::Block {
            blocker: dinosaur,
            attacker: flyer,
        },
    );
    act(&mut game, Action::FinishBlockers);
    advance(&mut game, |game| {
        game.object_view(flyer).zone == Zone::Graveyard
    });

    let mut game = scenario(p);
    let dinosaur = put(&mut game, p, "Colossadactyl", Zone::Battlefield);
    let blocker = put(&mut game, p.other(), "Bear Cub", Zone::Battlefield);
    attack(&mut game, dinosaur);
    advance(&mut game, |game| {
        matches!(game.state.pending, Pending::Blockers)
    });
    act(
        &mut game,
        Action::Block {
            blocker,
            attacker: dinosaur,
        },
    );
    act(&mut game, Action::FinishBlockers);
    advance(&mut game, |game| {
        matches!(game.state.pending, Pending::Damage)
    });
    act(
        &mut game,
        Action::AssignDamage {
            target: Target::Card(blocker),
            amount: 2,
        },
    );
    assert_eq!(game.object_view(blocker).zone, Zone::Graveyard);
    assert_eq!(
        game.state.players[p.other().index()].life,
        STARTING_LIFE - 2
    );
}

#[test]
fn confounding_riddle_privately_picks_one_or_asks_for_four_mana() {
    let p = Player::P0;
    let mut game = scenario(p);
    let spell = put(&mut game, p, "Confounding Riddle", Zone::Hand);
    let top: Vec<_> = ["Opt", "Shock", "Island", "Bear Cub"]
        .into_iter()
        .map(|name| put(&mut game, p, name, Zone::Library))
        .collect();
    act(&mut game, Action::AnnounceCast { card: spell });
    act(&mut game, Action::ChooseMode { mode: 0 });
    pay(&mut game);
    advance(&mut game, |game| {
        game.observe(p).library_selection.is_some()
    });
    assert!(game.observe(p.other()).library_selection.is_none());
    act(&mut game, Action::SelectLibrary { card: top[1] });
    act(&mut game, Action::FinishLibrarySelection);
    settle(&mut game);
    for (i, &card) in top.iter().enumerate() {
        assert_eq!(
            game.object_view(card).zone,
            if i == 1 { Zone::Hand } else { Zone::Graveyard }
        );
    }
    let mut game = scenario(p.other());
    let shock = put(&mut game, p.other(), "Shock", Zone::Hand);
    let riddle = put(&mut game, p, "Confounding Riddle", Zone::Hand);
    cast(&mut game, shock, Some(Target::Player(p)));
    act(&mut game, Action::Pass);
    act(&mut game, Action::AnnounceCast { card: riddle });
    act(&mut game, Action::ChooseMode { mode: 1 });
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Spell(shock),
        },
    );
    pay(&mut game);
    advance(&mut game, |game| {
        game.legal_actions()
            .contains(&Action::DeclineResolutionMana)
    });
    for paid in [false, true] {
        let mut branch = game.clone();
        if paid {
            choose(
                &mut branch,
                |action| matches!(action, Action::PayResolutionMana { payment } if payment.iter().sum::<u32>() == 4),
            );
        } else {
            act(&mut branch, Action::DeclineResolutionMana);
        }
        settle(&mut branch);
        assert_eq!(
            branch.state.players[p.index()].life,
            STARTING_LIFE - if paid { 2 } else { 0 }
        );
    }
}

#[test]
fn cosmium_blast_deals_four_to_an_attacker_or_blocker() {
    let p = Player::P0;
    for blocked in [false, true] {
        let mut game = scenario(p);
        let attacker = put(&mut game, p, "Hill Giant", Zone::Battlefield);
        let blocker = put(&mut game, p.other(), "Hill Giant", Zone::Battlefield);
        let spell = put(&mut game, p, "Cosmium Blast", Zone::Hand);
        let plains = [
            put(&mut game, p, "Plains", Zone::Battlefield),
            put(&mut game, p, "Plains", Zone::Battlefield),
        ];
        reject(&mut game, Action::AnnounceCast { card: spell });
        attack(&mut game, attacker);
        if blocked {
            advance(&mut game, |game| {
                matches!(game.state.pending, Pending::Blockers)
            });
            act(&mut game, Action::Block { blocker, attacker });
            act(&mut game, Action::FinishBlockers);
        }
        let victim = if blocked { blocker } else { attacker };
        for card in plains {
            act(&mut game, Action::TapMana { card });
        }
        cast(&mut game, spell, Some(Target::Card(victim)));
        settle(&mut game);
        assert_eq!(game.object_view(victim).zone, Zone::Graveyard);
    }
}

#[test]
fn dead_weight_removes_two_toughness_even_from_an_indestructible_creature() {
    let p = Player::P0;
    let mut game = scenario(p);
    let bear = put(&mut game, p.other(), "Bear Cub", Zone::Battlefield);
    let aura = put(&mut game, p, "Dead Weight", Zone::Hand);
    game.state.objects[bear as usize].gained_keywords = Keywords::INDESTRUCTIBLE;
    cast(&mut game, aura, Some(Target::Card(bear)));
    settle(&mut game);
    assert_eq!(game.object_view(bear).zone, Zone::Graveyard);
    assert_eq!(game.object_view(aura).zone, Zone::Graveyard);
}

#[test]
fn deathcap_marionette_has_deathtouch_and_mills_exactly_two_only_if_accepted() {
    let p = Player::P1;
    let mut root = scenario(p);
    let source = put(&mut root, p, "Deathcap Marionette", Zone::Hand);
    let top = put(&mut root, p, "Shock", Zone::Library);
    let second = put(&mut root, p, "Bear Cub", Zone::Library);
    cast(&mut root, source, None);
    advance(&mut root, |game| {
        game.legal_actions()
            .contains(&Action::ChooseOptional { accept: true })
    });
    for accept in [false, true] {
        let mut game = root.clone();
        act(&mut game, Action::ChooseOptional { accept });
        settle(&mut game);
        assert!(
            game.object_view(source)
                .keywords
                .contains(Keywords::DEATHTOUCH)
        );
        for card in [top, second] {
            assert_eq!(
                game.object_view(card).zone,
                if accept {
                    Zone::Graveyard
                } else {
                    Zone::Library
                }
            );
        }
    }
}

#[test]
fn diamond_pick_axe_boosts_its_bearer_and_creates_treasure_only_when_it_attacks() {
    let p = Player::P0;
    let mut root = scenario(p);
    let gear = put(&mut root, p, "Diamond Pick-Axe", Zone::Battlefield);
    let bearer = put(&mut root, p, "Bear Cub", Zone::Battlefield);
    let other = put(&mut root, p, "Grizzly Bears", Zone::Battlefield);
    act(&mut root, activate(gear, 0));
    act(&mut root, target(bearer));
    assert_eq!(root.observe(p).payment_request.unwrap().generic, 2);
    pay(&mut root);
    settle(&mut root);
    assert!(
        root.object_view(gear)
            .keywords
            .contains(Keywords::INDESTRUCTIBLE)
    );
    assert_eq!(stats(&root, bearer), (3, 3));
    let mut different = root.clone();
    attack(&mut different, other);
    settle(&mut different);
    assert!(
        !different
            .observe(p)
            .objects
            .iter()
            .any(|object| object.card == TREASURE)
    );
    attack(&mut root, bearer);
    settle(&mut root);
    assert_eq!(root.object_view(token(&root, p, TREASURE)).kinds, 16);
}
