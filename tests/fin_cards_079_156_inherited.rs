//! Runtime outcomes for previously admitted Final Fantasy cards, rows 79 through 156.
mod support;
use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn passes(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn target(game: &mut Game, card: CardId) {
    act(
        game,
        Action::ChooseTarget {
            target: Target::Card(card),
        },
    );
}

fn activate(game: &mut Game, card: CardId, ability: u16, target_id: Option<CardId>) {
    act(
        game,
        Action::AnnounceActivate {
            card,
            ability,
            counter_payment: Counters::default(),
        },
    );
    if let Some(card) = target_id {
        target(game, card);
    }
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(game);
    }
}

fn choose_search(game: &mut Game, card: CardId) {
    act(game, Action::ChooseSearch { card });
    act(game, Action::FinishSearch);
}

fn mana_action(card: CardId, color: usize) -> Action {
    let mut production = [0; 6];
    production[color] = 1;
    Action::ActivateMana {
        card,
        ability: 0,
        payment: [0; 6],
        counter_payment: Counters::default(),
        production,
    }
}

#[test]
fn basic_lands_and_towns_have_their_printed_entry_and_mana_choices() {
    let p = Player::P0;
    for (name, color) in [("Forest", 4), ("Island", 1), ("Mountain", 3)] {
        let mut game = scenario(p);
        let card = put(&mut game, p, name, Zone::Hand);
        act(&mut game, Action::PlayLand { card });
        assert!(!game.object_view(card).tapped);
        game.state.players[p.index()].mana = [0; 6];
        act(&mut game, Action::TapMana { card });
        let mut expected = [0; 6];
        expected[color] = 1;
        assert_eq!(game.state.players[p.index()].mana, expected);
    }
    for (name, colors) in [
        ("Gohn, Town of Ruin", [2, 4]),
        ("Gongaga, Reactor Town", [3, 4]),
        ("Guadosalam, Farplane Gateway", [1, 4]),
        ("Insomnia, Crown City", [0, 2]),
    ] {
        let mut game = scenario(p);
        let card = put(&mut game, p, name, Zone::Hand);
        act(&mut game, Action::PlayLand { card });
        assert!(game.object_view(card).tapped && game.object_view(card).has_subtype("Town"));
        reject(&mut game, mana_action(card, colors[0]));
        game.state.objects[card as usize].tapped = false;
        game.state.players[p.index()].mana = [0; 6];
        for color in 0..6 {
            let mut branch = game.clone();
            if colors.contains(&color) {
                act(&mut branch, mana_action(card, color));
                let mut expected = [0; 6];
                expected[color] = 1;
                assert_eq!(branch.state.players[p.index()].mana, expected);
            } else {
                reject(&mut branch, mana_action(card, color));
            }
        }
    }
}

#[test]
fn fate_discounts_only_a_tapped_creature_and_destroys_it() {
    let p = Player::P1;
    let mut game = scenario(p);
    let spell = put(&mut game, p, "Fate of the Sun-Cryst", Zone::Hand);
    let creature = put(&mut game, p.other(), "Hill Giant", Zone::Battlefield);
    let artifact = put(&mut game, p.other(), "Short Bow", Zone::Battlefield);
    game.state.objects[artifact as usize].tapped = true;
    game.state.players[p.index()].mana = [1, 0, 0, 0, 0, 2];
    reject(&mut game, Action::AnnounceCast { card: spell });
    game.state.objects[creature as usize].tapped = true;
    act(&mut game, Action::AnnounceCast { card: spell });
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(artifact),
        },
    );
    target(&mut game, creature);
    pay(&mut game);
    assert_eq!(game.state.players[p.index()].mana, [0; 6]);
    settle(&mut game);
    assert_eq!(game.object_view(creature).zone, Zone::Graveyard);
}

#[test]
fn fight_on_allows_zero_targets_or_returns_only_the_surviving_own_creature() {
    let p = Player::P0;
    let mut game = scenario(p);
    let spell = put(&mut game, p, "Fight On!", Zone::Hand);
    let own = ["Grizzly Bears", "Hill Giant"].map(|name| put(&mut game, p, name, Zone::Graveyard));
    let enemy = put(&mut game, p.other(), "Grizzly Bears", Zone::Graveyard);
    let mut zero = game.clone();
    cast(&mut zero, spell, None);
    settle(&mut zero);
    assert_eq!(zero.hand_count(p), 0);
    act(&mut game, Action::AnnounceCast { card: spell });
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(enemy),
        },
    );
    for card in own {
        target(&mut game, card);
    }
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(&mut game, Action::FinishTargets);
    }
    pay(&mut game);
    game.move_object(own[0], Zone::Exile);
    settle(&mut game);
    assert_eq!(game.object_view(own[0]).zone, Zone::Exile);
    assert_eq!(game.object_view(own[1]).zone, Zone::Hand);
}

#[test]
fn fire_magic_prices_and_deals_each_tier_to_both_sides() {
    let p = Player::P0;
    for (mode, generic, damage) in [(0, 0, 1), (1, 2, 2), (2, 5, 3)] {
        let mut game = scenario(p);
        let spell = put(&mut game, p, "Fire Magic", Zone::Hand);
        let creatures =
            [p, p.other()].map(|owner| put(&mut game, owner, "Gigantosaurus", Zone::Battlefield));
        let artifact = put(&mut game, p, "Short Bow", Zone::Battlefield);
        game.state.players[p.index()].mana = [0, 0, 0, 1, 0, generic];
        act(&mut game, Action::AnnounceCast { card: spell });
        act(&mut game, Action::ChooseMode { mode });
        pay(&mut game);
        assert_eq!(game.state.players[p.index()].mana, [0; 6]);
        settle(&mut game);
        for card in creatures {
            assert_eq!(game.object_view(card).damage, damage);
        }
        assert_eq!(game.object_view(artifact).damage, 0);
    }
}

#[test]
fn gaelicat_and_gigantoad_use_live_controller_thresholds() {
    let p = Player::P1;
    let mut game = scenario(p);
    let cat = put(&mut game, p, "Gaelicat", Zone::Battlefield);
    let toad = put(&mut game, p, "Gigantoad", Zone::Battlefield);
    put(&mut game, p.other(), "Short Bow", Zone::Battlefield);
    put(&mut game, p, "Short Bow", Zone::Battlefield);
    assert_eq!(stats(&game, cat), (1, 3));
    let second = put(&mut game, p, "Lavaspur Boots", Zone::Battlefield);
    assert_eq!(stats(&game, cat), (3, 3));
    assert!(
        game.keywords(cat)
            .contains(Keywords::FLYING.union(Keywords::VIGILANCE))
    );
    game.move_object(second, Zone::Graveyard);
    assert_eq!(stats(&game, cat), (1, 3));

    for _ in 0..6 {
        put(&mut game, p, "Forest", Zone::Battlefield);
    }
    assert_eq!(stats(&game, toad), (4, 4));
    let seventh = put(&mut game, p, "Forest", Zone::Battlefield);
    assert_eq!(stats(&game, toad), (6, 6));
    game.state.objects[toad as usize].damage = 4;
    game.move_object(seventh, Zone::Graveyard);
    game.state_based_actions();
    assert_eq!(game.object_view(toad).zone, Zone::Graveyard);
}

#[test]
fn gaius_uses_three_distinct_sacrifice_filters_and_collects_apnap_choices() {
    let p = Player::P0;
    for mode in 0..3 {
        let mut game = scenario(p);
        let mut cohorts = [[0; 3]; 2];
        for owner in [p, p.other()] {
            cohorts[owner.index()] = [
                spawn(
                    &mut game,
                    owner,
                    Card::from_catalogue_id(5328),
                    Zone::Battlefield,
                ),
                put(&mut game, owner, "Grizzly Bears", Zone::Battlefield),
                put(&mut game, owner, "Spelunking", Zone::Battlefield),
            ];
        }
        let gaius = put(&mut game, p, "Gaius van Baelsar", Zone::Hand);
        cast(&mut game, gaius, None);
        passes(&mut game);
        passes(&mut game);
        act(&mut game, Action::ChooseMode { mode });
        for actor in [p, p.other()] {
            assert_eq!(game.actor(), Some(actor));
            let invalid = game.object_ref(cohorts[actor.index()][(usize::from(mode) + 1) % 3]);
            reject(&mut game, Action::Sacrifice { object: invalid });
            let object = game.object_ref(cohorts[actor.index()][usize::from(mode)]);
            act(&mut game, Action::Sacrifice { object });
            if actor == p {
                assert_eq!(game.object_view(object.card).zone, Zone::Battlefield);
            }
        }
        for owner in [p, p.other()] {
            for (index, card) in cohorts[owner.index()].into_iter().enumerate() {
                assert_eq!(
                    game.object_view(card).zone == Zone::Battlefield,
                    index != usize::from(mode)
                );
            }
        }
    }
}

#[test]
fn galufs_final_act_uses_the_dying_creatures_final_power() {
    let p = Player::P0;
    let mut game = scenario(p);
    let dying = put(&mut game, p, "Hill Giant", Zone::Battlefield);
    let recipient = put(&mut game, p, "Grizzly Bears", Zone::Battlefield);
    let spell = put(&mut game, p, "Galuf's Final Act", Zone::Hand);
    game.add_counter(dying, CounterKind::PlusOne, 2);
    cast(&mut game, spell, Some(Target::Card(dying)));
    settle(&mut game);
    assert_eq!(stats(&game, dying), (6, 5));
    game.move_object(dying, Zone::Graveyard);
    game.prepare_triggers();
    target(&mut game, recipient);
    settle(&mut game);
    assert_eq!(game.counter_count(recipient, CounterKind::PlusOne), 6);
}

#[test]
fn gladiolus_puts_a_nonbasic_land_in_tapped_then_buffs_another_own_creature() {
    let p = Player::P1;
    let mut game = scenario(p);
    let source = put(&mut game, p, "Gladiolus Amicitia", Zone::Hand);
    let bear = put(&mut game, p, "Grizzly Bears", Zone::Battlefield);
    let land = put(&mut game, p, "Guadosalam, Farplane Gateway", Zone::Library);
    cast(&mut game, source, None);
    passes(&mut game);
    passes(&mut game);
    choose_search(&mut game, land);
    assert!(game.object_view(land).tapped);
    assert_eq!(game.object_view(land).zone, Zone::Battlefield);
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(source),
        },
    );
    target(&mut game, bear);
    settle(&mut game);
    assert_eq!(stats(&game, bear), (4, 4));
    assert!(game.keywords(bear).contains(Keywords::TRAMPLE));
}

#[test]
fn goobbue_gardener_mana_respects_summoning_sickness_and_uses_no_stack() {
    let p = Player::P0;
    let mut game = scenario(p);
    let gardener = put(&mut game, p, "Goobbue Gardener", Zone::Hand);
    cast(&mut game, gardener, None);
    settle(&mut game);
    assert_eq!(stats(&game, gardener), (1, 3));
    game.state.players[p.index()].mana = [0; 6];
    reject(&mut game, mana_action(gardener, 4));
    game.state.objects[gardener as usize].summoning_sick = false;
    act(&mut game, mana_action(gardener, 4));
    assert!(game.state.stack.is_empty());
    assert_eq!(game.state.players[p.index()].mana, [0, 0, 0, 0, 1, 0]);
}

#[test]
fn gysahl_greens_uses_the_same_landfall_bird_in_both_cast_routes() {
    let p = Player::P1;
    for origin in [Zone::Hand, Zone::Graveyard] {
        let mut game = scenario(p);
        let spell = put(&mut game, p, "Gysahl Greens", origin);
        if origin == Zone::Hand {
            cast(&mut game, spell, None);
        } else {
            act(&mut game, Action::AnnounceFlashback { card: spell });
            pay(&mut game);
        }
        settle(&mut game);
        let bird = game
            .observe(p)
            .objects
            .into_iter()
            .find(|object| {
                object.zone == Zone::Battlefield && object.card == Card::from_catalogue_id(5269)
            })
            .unwrap();
        assert!(bird.has_subtype("Bird") && bird.is_token());
        assert_eq!(bird.colors, 16);
        assert_eq!(stats(&game, bird.id), (2, 2));
        let own = put(&mut game, p, "Forest", Zone::Hand);
        act(&mut game, Action::PlayLand { card: own });
        settle(&mut game);
        assert_eq!(stats(&game, bird.id), (3, 2));
        let enemy = put(&mut game, p.other(), "Island", Zone::Hand);
        game.move_object(enemy, Zone::Battlefield);
        game.prepare_triggers();
        assert!(game.state.stack.is_empty());
        assert_eq!(stats(&game, bird.id), (3, 2));
        let expected = if origin == Zone::Hand {
            Zone::Graveyard
        } else {
            Zone::Exile
        };
        assert_eq!(game.object_view(spell).zone, expected);
    }
}

#[test]
fn hecteyes_makes_only_the_opponent_choose_a_private_discard() {
    let p = Player::P0;
    let mut game = scenario(p);
    let creature = put(&mut game, p, "Hecteyes", Zone::Hand);
    let own = put(&mut game, p, "Forest", Zone::Hand);
    let enemy = put(&mut game, p.other(), "Opt", Zone::Hand);
    cast(&mut game, creature, None);
    passes(&mut game);
    passes(&mut game);
    assert_eq!(game.actor(), Some(p.other()));
    let discard = Action::Discard { card: enemy };
    assert!(!game.observe(p).actions.contains(&discard));
    act(&mut game, discard);
    assert_eq!(game.object_view(enemy).zone, Zone::Graveyard);
    assert_eq!(game.object_view(own).zone, Zone::Hand);
}

#[test]
fn hill_gigas_has_haste_and_trample_and_cycles_for_a_nonbasic_mountain() {
    let p = Player::P0;
    let mut game = scenario(p);
    let source = put(&mut game, p, "Hill Gigas", Zone::Hand);
    let mut cast_branch = game.clone();
    cast(&mut cast_branch, source, None);
    settle(&mut cast_branch);
    assert!(
        cast_branch
            .keywords(source)
            .contains(Keywords::HASTE.union(Keywords::TRAMPLE))
    );
    assert_eq!(stats(&cast_branch, source), (5, 4));
    let land = put(&mut game, p, "Stomping Ground", Zone::Library);
    activate(&mut game, source, 0, None);
    assert_eq!(game.object_view(source).zone, Zone::Graveyard);
    passes(&mut game);
    choose_search(&mut game, land);
    assert_eq!(game.object_view(land).zone, Zone::Hand);
}
