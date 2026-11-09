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

fn declare(game: &mut Game, card: CardId) {
    game.begin_step(Step::BeginCombat);
    passes(game);
    act(game, Action::Attack { card });
    act(game, Action::FinishAttackers);
}

fn arrange(game: &mut Game, card: CardId, destination: LibraryDestination) {
    act(game, Action::ArrangeLibrary { card, destination });
}

fn ordered_triggers(game: &mut Game) {
    while matches!(game.state.pending, Pending::OrderTriggers { .. }) {
        choose(game, |action| matches!(action, Action::PutTrigger { .. }));
    }
}

#[test]
fn ice_magic_prices_all_tiers_and_uses_the_targets_owner() {
    let p = Player::P0;
    for (mode, blue, generic, expected) in [
        (0, 1, 0, Zone::Hand),
        (1, 1, 2, Zone::Library),
        (2, 2, 5, Zone::Library),
    ] {
        let mut game = scenario(p);
        let spell = put(&mut game, p, "Ice Magic", Zone::Hand);
        let creature = put(&mut game, p.other(), "Hill Giant", Zone::Battlefield);
        game.record_control(game.object_ref(creature), p, None);
        game.state.players[p.index()].mana = [0, blue, 0, 0, 0, generic];
        act(&mut game, Action::AnnounceCast { card: spell });
        act(&mut game, Action::ChooseMode { mode });
        target(&mut game, creature);
        pay(&mut game);
        assert_eq!(game.state.players[p.index()].mana, [0; 6]);
        passes(&mut game);
        if mode == 1 {
            assert_eq!(game.actor(), Some(p.other()));
            let object = game.object_ref(creature);
            act(
                &mut game,
                Action::ChooseLibraryDestination {
                    object,
                    destination: LibraryDestination::Bottom,
                },
            );
            assert_eq!(game.state.players[p.other().index()].library[0], creature);
        }
        assert_eq!(game.object_view(creature).zone, expected);
        assert_eq!(game.object_view(creature).owner, p.other());
    }
}

#[test]
fn il_mheg_pixie_surveils_privately_on_a_declared_attack() {
    let p = Player::P1;
    let mut game = scenario(p);
    let pixie = put(&mut game, p, "Il Mheg Pixie", Zone::Battlefield);
    let top = game.top_library_card(p).unwrap();
    assert!(game.keywords(pixie).contains(Keywords::FLYING));
    declare(&mut game, pixie);
    passes(&mut game);
    assert!(game.observe(p).library_choice.is_some());
    assert!(game.observe(p.other()).library_choice.is_none());
    arrange(&mut game, top, LibraryDestination::Graveyard);
    assert_eq!(game.object_view(top).zone, Zone::Graveyard);
}

#[test]
fn instant_ramen_has_flash_draws_on_entry_and_is_sacrificed_for_three_life() {
    let active = Player::P0;
    let p = active.other();
    let mut game = scenario(active);
    let ramen = put(&mut game, p, "Instant Ramen", Zone::Hand);
    act(&mut game, Action::Pass);
    cast(&mut game, ramen, None);
    settle(&mut game);
    assert_eq!(game.hand_count(p), 1);
    assert!(game.object_view(ramen).has_subtype("Food"));
    if game.actor() != Some(p) {
        act(&mut game, Action::Pass);
    }
    activate(&mut game, ramen, 0, None);
    assert_eq!(game.object_view(ramen).zone, Zone::Graveyard);
    settle(&mut game);
    assert_eq!(game.state.players[p.index()].life, 23);
}

#[test]
fn iron_giant_has_reach_vigilance_and_trample_and_can_block_a_flyer() {
    let p = Player::P0;
    let mut game = scenario(p);
    let flyer = put(&mut game, p, "Picklock Prankster", Zone::Battlefield);
    let giant = put(&mut game, p.other(), "Iron Giant", Zone::Battlefield);
    let view = game.object_view(giant);
    assert!(view.has_subtype("Demon"));
    assert_eq!(view.kinds & 17, 17);
    assert_eq!(stats(&game, giant), (6, 6));
    assert!(
        game.keywords(giant).contains(
            Keywords::REACH
                .union(Keywords::VIGILANCE)
                .union(Keywords::TRAMPLE)
        )
    );
    declare(&mut game, flyer);
    passes(&mut game);
    act(
        &mut game,
        Action::Block {
            blocker: giant,
            attacker: flyer,
        },
    );
    act(&mut game, Action::FinishBlockers);
    passes(&mut game);
    assert_eq!(game.object_view(flyer).zone, Zone::Graveyard);
    assert_eq!(game.object_view(giant).damage, 1);
}

#[test]
fn faith_and_grief_returns_legal_artifacts_or_enchantments_before_playing_ishgard() {
    let p = Player::P1;
    let mut game = scenario(p);
    let town = put(&mut game, p, "Ishgard, the Holy See", Zone::Hand);
    let artifact = put(&mut game, p, "Short Bow", Zone::Graveyard);
    let enchantment = put(&mut game, p, "Doubling Season", Zone::Graveyard);
    let invalid = put(&mut game, p.other(), "Short Bow", Zone::Graveyard);
    act(&mut game, Action::AnnounceAdventure { card: town });
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(invalid),
        },
    );
    target(&mut game, artifact);
    target(&mut game, enchantment);
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(&mut game, Action::FinishTargets);
    }
    pay(&mut game);
    game.move_object(artifact, Zone::Exile);
    settle(&mut game);
    assert_eq!(game.object_view(enchantment).zone, Zone::Battlefield);
    assert_eq!(game.object_view(artifact).zone, Zone::Exile);
    assert_eq!(game.object_view(town).zone, Zone::Exile);
    act(&mut game, Action::PlayLand { card: town });
    assert!(game.object_view(town).tapped && game.object_view(town).has_subtype("Town"));
}

#[test]
fn laughing_mad_requires_a_distinct_discard_in_both_cast_routes() {
    let p = Player::P0;
    for origin in [Zone::Hand, Zone::Graveyard] {
        let mut game = scenario(p);
        let spell = put(&mut game, p, "Laughing Mad", origin);
        let announce = if origin == Zone::Hand {
            Action::AnnounceCast { card: spell }
        } else {
            Action::AnnounceFlashback { card: spell }
        };
        reject(&mut game, announce.clone());
        let discard = put(&mut game, p, "Forest", Zone::Hand);
        act(&mut game, announce);
        let object = game.object_ref(discard);
        act(&mut game, Action::ChooseAdditionalCost { object });
        pay(&mut game);
        assert_eq!(game.object_view(discard).zone, Zone::Graveyard);
        settle(&mut game);
        assert_eq!(game.hand_count(p), 2);
        assert_eq!(
            game.object_view(spell).zone,
            if origin == Zone::Hand {
                Zone::Graveyard
            } else {
                Zone::Exile
            }
        );
    }
}

#[test]
fn lion_heart_deals_entry_damage_and_its_modifier_follows_equip() {
    let p = Player::P1;
    let mut game = scenario(p);
    let lion = put(&mut game, p, "Lion Heart", Zone::Hand);
    let own = put(&mut game, p, "Grizzly Bears", Zone::Battlefield);
    let opponent = put(&mut game, p.other(), "Grizzly Bears", Zone::Battlefield);
    cast(&mut game, lion, None);
    passes(&mut game);
    target(&mut game, opponent);
    settle(&mut game);
    assert_eq!(game.object_view(opponent).zone, Zone::Graveyard);
    assert_eq!(stats(&game, own), (2, 2));
    activate(&mut game, lion, 0, Some(own));
    settle(&mut game);
    assert_eq!(stats(&game, own), (4, 3));
}

#[test]
fn locke_loots_on_real_player_combat_damage_and_has_lifelink() {
    let p = Player::P0;
    let mut game = scenario(p);
    let locke = put(&mut game, p, "Locke Cole", Zone::Battlefield);
    let drawn = game.top_library_card(p).unwrap();
    assert!(
        game.keywords(locke)
            .contains(Keywords::DEATHTOUCH.union(Keywords::LIFELINK))
    );
    declare(&mut game, locke);
    passes(&mut game);
    act(&mut game, Action::FinishBlockers);
    passes(&mut game);
    assert_eq!(game.state.players[p.index()].life, 22);
    assert_eq!(game.state.players[p.other().index()].life, 18);
    passes(&mut game);
    assert_eq!(game.object_view(drawn).zone, Zone::Hand);
    act(&mut game, Action::Discard { card: drawn });
    assert_eq!(game.hand_count(p), 0);
}

#[test]
fn moogles_valor_counts_the_original_creatures_and_loporrit_sees_both_tokens() {
    let p = Player::P0;
    let mut game = scenario(p);
    let scout = put(&mut game, p, "Loporrit Scout", Zone::Battlefield);
    let bear = put(&mut game, p, "Grizzly Bears", Zone::Battlefield);
    let enemy = put(&mut game, p.other(), "Grizzly Bears", Zone::Battlefield);
    let spell = put(&mut game, p, "Moogles' Valor", Zone::Hand);
    cast(&mut game, spell, None);
    passes(&mut game);
    while matches!(game.state.pending, Pending::EntryTimestamp { .. }) {
        choose(&mut game, |action| {
            matches!(action, Action::ChooseEntryTimestamp { .. })
        });
    }
    ordered_triggers(&mut game);
    settle(&mut game);
    let tokens = game
        .observe(p)
        .objects
        .into_iter()
        .filter(|object| {
            object.zone == Zone::Battlefield && object.card == Card::from_catalogue_id(5260)
        })
        .collect::<Vec<_>>();
    assert_eq!(tokens.len(), 2);
    for token in tokens {
        assert!(token.has_subtype("Moogle") && token.is_token());
        assert_eq!(stats(&game, token.id), (1, 2));
        assert!(
            game.keywords(token.id)
                .contains(Keywords::LIFELINK.union(Keywords::INDESTRUCTIBLE))
        );
    }
    assert_eq!(stats(&game, scout), (5, 4));
    assert!(game.keywords(scout).contains(Keywords::INDESTRUCTIBLE));
    assert!(game.keywords(bear).contains(Keywords::INDESTRUCTIBLE));
    assert!(!game.keywords(enemy).contains(Keywords::INDESTRUCTIBLE));
}

#[test]
fn lunatic_pandora_has_a_private_surveil_and_a_distinct_paid_destruction() {
    let p = Player::P1;
    let mut game = scenario(p);
    let pandora = put(&mut game, p, "Lunatic Pandora", Zone::Battlefield);
    let target_id = put(&mut game, p.other(), "Hill Giant", Zone::Battlefield);
    let mut surveil = game.clone();
    activate(&mut surveil, pandora, 0, None);
    assert!(surveil.object_view(pandora).tapped);
    passes(&mut surveil);
    assert!(surveil.observe(p).library_choice.is_some());
    assert!(surveil.observe(p.other()).library_choice.is_none());
    let top = surveil.top_library_card(p).unwrap();
    arrange(&mut surveil, top, LibraryDestination::Graveyard);
    assert_eq!(surveil.object_view(top).zone, Zone::Graveyard);
    activate(&mut game, pandora, 1, Some(target_id));
    assert_eq!(game.object_view(pandora).zone, Zone::Graveyard);
    settle(&mut game);
    assert_eq!(game.object_view(target_id).zone, Zone::Graveyard);
}

#[test]
fn machinists_arsenal_and_monks_fist_move_their_job_grants_with_the_equipment() {
    let p = Player::P0;
    for (name, subtype, hero_stats, recipient_stats) in [
        ("Machinist's Arsenal", "Artificer", (5, 5), (6, 6)),
        ("Monk's Fist", "Monk", (2, 1), (3, 2)),
    ] {
        let mut game = scenario(p);
        let equipment = put(&mut game, p, name, Zone::Hand);
        let bear = put(&mut game, p, "Grizzly Bears", Zone::Battlefield);
        put(&mut game, p, "Short Bow", Zone::Battlefield);
        cast(&mut game, equipment, None);
        settle(&mut game);
        let hero = game.object_view(equipment).attached_to.unwrap().card;
        assert!(game.object_view(hero).has_subtype("Hero"));
        assert!(game.object_view(hero).has_subtype(subtype));
        assert_eq!(stats(&game, hero), hero_stats);
        activate(&mut game, equipment, 0, Some(bear));
        settle(&mut game);
        assert!(!game.object_view(hero).has_subtype(subtype));
        assert_eq!(stats(&game, hero), (1, 1));
        assert!(game.object_view(bear).has_subtype(subtype));
        assert_eq!(stats(&game, bear), recipient_stats);
    }
}

#[test]
fn magic_damper_gives_hexproof_in_response_and_untap_consumes_stun() {
    let p = Player::P0;
    let mut game = scenario(p);
    let bear = put(&mut game, p, "Grizzly Bears", Zone::Battlefield);
    game.state.objects[bear as usize].tapped = true;
    game.add_counter(bear, CounterKind::Stun, 1);
    let damper = put(&mut game, p, "Magic Damper", Zone::Hand);
    let bolt = put(&mut game, p.other(), "Lightning Bolt", Zone::Hand);
    act(&mut game, Action::Pass);
    cast(&mut game, bolt, Some(Target::Card(bear)));
    act(&mut game, Action::Pass);
    cast(&mut game, damper, Some(Target::Card(bear)));
    settle(&mut game);
    assert_eq!(stats(&game, bear), (3, 3));
    assert_eq!(game.object_view(bear).damage, 0);
    assert!(game.keywords(bear).contains(Keywords::HEXPROOF));
    assert!(game.object_view(bear).tapped);
    assert_eq!(game.counter_count(bear, CounterKind::Stun), 0);
}

#[test]
fn magic_pot_exiles_either_graveyard_and_creates_treasure_only_when_it_dies() {
    let p = Player::P1;
    let mut game = scenario(p);
    let pot = put(&mut game, p, "Magic Pot", Zone::Battlefield);
    for owner in [p, p.other()] {
        let mut branch = game.clone();
        let card = put(&mut branch, owner, "Opt", Zone::Graveyard);
        activate(&mut branch, pot, 0, Some(card));
        settle(&mut branch);
        assert_eq!(branch.object_view(card).zone, Zone::Exile);
    }
    game.move_object(pot, Zone::Graveyard);
    game.prepare_triggers();
    settle(&mut game);
    let treasure = game
        .observe(p)
        .objects
        .into_iter()
        .find(|object| {
            object.zone == Zone::Battlefield && object.card == Card::from_catalogue_id(5302)
        })
        .unwrap();
    assert!(!treasure.tapped && treasure.has_subtype("Treasure"));
}

#[test]
fn magitek_armor_crews_with_its_new_hero_and_magitek_infantry_searches_its_name() {
    let p = Player::P0;
    let mut game = scenario(p);
    let armor = put(&mut game, p, "Magitek Armor", Zone::Hand);
    cast(&mut game, armor, None);
    settle(&mut game);
    assert!(!game.object_view(armor).is_creature());
    let hero = game
        .observe(p)
        .objects
        .into_iter()
        .find(|object| object.zone == Zone::Battlefield && object.has_subtype("Hero"))
        .unwrap();
    assert!(hero.summoning_sick);
    activate(&mut game, armor, 0, None);
    act(
        &mut game,
        Action::ChooseCrewer {
            object: hero.reference,
        },
    );
    act(&mut game, Action::FinishCrew);
    settle(&mut game);
    assert!(game.object_view(hero.id).tapped);
    assert!(game.object_view(armor).is_creature());
    assert_eq!(stats(&game, armor), (4, 4));

    let mut infantry = scenario(p);
    let first = put(&mut infantry, p, "Magitek Infantry", Zone::Battlefield);
    let second = put(&mut infantry, p, "Magitek Infantry", Zone::Library);
    let decoy = put(&mut infantry, p, "Magitek Armor", Zone::Library);
    put(&mut infantry, p.other(), "Short Bow", Zone::Battlefield);
    assert_eq!(stats(&infantry, first), (1, 1));
    activate(&mut infantry, first, 0, None);
    passes(&mut infantry);
    reject(&mut infantry, Action::ChooseSearch { card: decoy });
    choose_search(&mut infantry, second);
    assert_eq!(stats(&infantry, first), (2, 1));
    assert_eq!(stats(&infantry, second), (2, 1));
    assert!(infantry.object_view(second).tapped);
}

#[test]
fn reactor_raid_offers_an_optional_own_sacrifice_and_leaves_midgar_playable() {
    let p = Player::P1;
    let mut game = scenario(p);
    let town = put(&mut game, p, "Midgar, City of Mako", Zone::Hand);
    let own = put(&mut game, p, "Short Bow", Zone::Battlefield);
    let opposing = put(&mut game, p.other(), "Grizzly Bears", Zone::Battlefield);
    act(&mut game, Action::AnnounceAdventure { card: town });
    pay(&mut game);
    assert_eq!(game.object_view(own).zone, Zone::Battlefield);
    passes(&mut game);
    let object = game.object_ref(opposing);
    reject(&mut game, Action::Sacrifice { object });
    let mut decline = game.clone();
    act(&mut decline, Action::DeclineEffectPayment);
    assert_eq!(decline.hand_count(p), 0);
    let object = game.object_ref(own);
    act(&mut game, Action::Sacrifice { object });
    assert_eq!(game.object_view(own).zone, Zone::Graveyard);
    assert_eq!(game.hand_count(p), 2);
    for branch in [&mut game, &mut decline] {
        assert_eq!(branch.object_view(town).zone, Zone::Exile);
        act(branch, Action::PlayLand { card: town });
        assert!(branch.object_view(town).tapped);
    }
}
