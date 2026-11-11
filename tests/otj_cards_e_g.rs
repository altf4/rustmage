//! Outlaws of Thunder Junction card scenarios, E through G.
#[path = "support/otj_e_l.rs"]
mod otj;
mod support;

use otj::*;
use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

#[test]
fn emergent_haunting_uses_both_intervening_conditions_and_surveys() {
    let player = Player::P0;
    let mut game = scenario(player);
    let haunting = put(&mut game, player, "Emergent Haunting", Zone::Battlefield);
    let shock = put(&mut game, player, "Shock", Zone::Hand);
    let top = put(&mut game, player, "Island", Zone::Library);
    announce_activation(&mut game, haunting, 0);
    pay(&mut game);
    let surveil = Action::ArrangeLibrary {
        card: top,
        destination: LibraryDestination::Graveyard,
    };
    checkpoint(&mut game, surveil.clone());
    assert!(game.observe(player.other()).library_choice.is_none());
    apply_raw_and_index(&mut game, surveil);
    assert_eq!(game.object_view(top).zone, Zone::Graveyard);

    start_step(&mut game, Step::End);
    assert_eq!(game.state.stack.len(), 1);
    let mut response = game.clone();
    cast(&mut response, shock, Some(Target::Player(player.other())));
    resolve(&mut response);
    assert_eq!(response.object_view(haunting).kinds & 1, 0);

    resolve(&mut game);
    let animated = game.object_view(haunting);
    assert_eq!((animated.power, animated.toughness), (3, 3));
    assert_eq!(animated.kinds & (1 | 32), 1 | 32);
    assert!(animated.has_subtype("Spirit"));
    assert!(animated.keywords.contains(Keywords::FLYING));
    start_step(&mut game, Step::End);
    assert!(game.state.stack.is_empty());
}

#[test]
fn eriette_takes_the_exact_aura_recipient_until_the_attachment_ends() {
    let player = Player::P1;
    let mut game = scenario(player);
    let eriette = put(
        &mut game,
        player,
        "Eriette, the Beguiler",
        Zone::Battlefield,
    );
    let eligible = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    let too_costly = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    let aura = put(&mut game, player, "Pacifism", Zone::Hand);
    assert!(game.keywords(eriette).contains(Keywords::LIFELINK));

    let mut costly = game.clone();
    cast(&mut costly, aura, Some(Target::Card(too_costly)));
    resolve(&mut costly);
    assert_eq!(costly.object_view(too_costly).controller, player.other());

    cast(&mut game, aura, Some(Target::Card(eligible)));
    resolve(&mut game);
    assert_eq!(game.object_view(eligible).controller, player);
    game.move_object(eriette, Zone::Graveyard);
    assert_eq!(game.object_view(eligible).controller, player);
    game.move_object(aura, Zone::Graveyard);
    assert_eq!(game.object_view(eligible).controller, player.other());
}

#[test]
fn ertha_jo_creates_a_mercenary_and_copies_its_sorcery_speed_activation() {
    let player = Player::P0;
    let mut game = scenario(player);
    let ertha = put(&mut game, player, "Ertha Jo, Frontier Mentor", Zone::Hand);
    cast(&mut game, ertha, None);
    resolve(&mut game);
    let [mercenary] = tokens(&game, player, MERCENARY)[..] else {
        panic!("Ertha Jo must create exactly one Mercenary");
    };
    let body = game.object_view(mercenary);
    assert_eq!((body.power, body.toughness, body.colors), (1, 1, 8));
    assert!(body.has_subtype("Mercenary"));
    game.state.objects[mercenary as usize].summoning_sick = false;
    let second = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    choose(&mut game, |action| {
        matches!(action, Action::Activate { card, ability: 0, target: Some(Target::Card(target)), .. }
            if *card == mercenary && *target == ertha)
    });
    let change_target = Action::SetStackTarget {
        index: 0,
        target: Target::Card(second),
    };
    checkpoint(&mut game, change_target.clone());
    apply_raw_and_index(&mut game, change_target);
    act(&mut game, Action::FinishStackTargets);
    resolve(&mut game);
    assert!(game.object_view(mercenary).tapped);
    assert_eq!(stats(&game, ertha), (3, 4));
    assert_eq!(stats(&game, second), (3, 2));
}

#[test]
fn fblthp_plots_the_exact_private_top_and_wards_itself() {
    let player = Player::P0;
    let mut game = scenario(player);
    let fblthp = put(
        &mut game,
        player,
        "Fblthp, Lost on the Range",
        Zone::Battlefield,
    );
    let land = put(&mut game, player, "Island", Zone::Library);
    let top = put(&mut game, player, "Freestrider Commando", Zone::Library);
    assert_eq!(
        game.observe(player).known_library_top[player.index()]
            .unwrap()
            .object
            .card,
        top
    );
    assert!(game.observe(player.other()).known_library_top[player.index()].is_none());
    choose(&mut game, |action| {
        matches!(action, Action::AnnounceLibraryTopPlot { object, cost, .. }
            if object.card == top && cost.generic == 2)
    });
    assert_eq!(
        game.observe(player).known_library_top[player.index()]
            .unwrap()
            .object
            .card,
        top
    );
    choose(&mut game, |action| matches!(action, Action::PayPlot { .. }));
    assert_eq!(game.object_view(top).zone, Zone::Exile);
    assert_eq!(
        game.observe(player).known_library_top[player.index()]
            .unwrap()
            .object
            .card,
        land
    );
    assert!(
        !game
            .legal_actions()
            .iter()
            .any(|a| matches!(a, Action::AnnounceLibraryTopPlot { .. }))
    );

    act(&mut game, Action::Pass);
    let shock = put(&mut game, player.other(), "Shock", Zone::Hand);
    cast(&mut game, shock, Some(Target::Card(fblthp)));
    choose_when_ready(&mut game, Action::DeclineResolutionMana);
    resolve(&mut game);
    assert_eq!(game.object_view(fblthp).zone, Zone::Battlefield);
    assert_eq!(game.object_view(shock).zone, Zone::Graveyard);
}

#[test]
fn final_showdown_resolves_all_selected_modes_in_order_and_chooses_without_targeting() {
    let player = Player::P0;
    for mode in 0..7 {
        let mut game = scenario(player);
        let chooser = put(
            &mut game,
            player,
            "Sphinx of the Final Word",
            Zone::Battlefield,
        );
        let rival = put(
            &mut game,
            player.other(),
            "Kotis, the Fangkeeper",
            Zone::Battlefield,
        );
        let casualty = put(
            &mut game,
            player.other(),
            "Grizzly Bears",
            Zone::Battlefield,
        );
        let spell = put(&mut game, player, "Final Showdown", Zone::Hand);
        spree(&mut game, spell, mode, &[]);
        let remove = matches!(mode, 0 | 3 | 4 | 6);
        let protect = matches!(mode, 1 | 3 | 5 | 6);
        let destroy = matches!(mode, 2 | 4 | 5 | 6);
        assert!(game.state.stack.last().unwrap().targets.is_empty());
        if protect {
            let selection = Action::ChooseEffectObject {
                object: game.object_ref(chooser),
            };
            checkpoint(&mut game, selection.clone());
            assert_eq!(game.keywords(chooser).contains(Keywords::HEXPROOF), !remove);
            act(&mut game, selection);
        }
        resolve(&mut game);
        assert_eq!(game.on_battlefield(chooser), !destroy || protect);
        assert_eq!(game.on_battlefield(rival), !destroy || !remove);
        assert_eq!(game.on_battlefield(casualty), !destroy);
    }
}

#[test]
fn fleeting_reflection_untaps_one_creature_and_copies_only_a_distinct_legal_blueprint() {
    let player = Player::P1;
    let mut game = scenario(player);
    let recipient = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let blueprint = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    game.state.objects[recipient as usize].tapped = true;
    game.state.objects[blueprint as usize].tapped = true;
    game.set_counters(recipient, Counters::of(CounterKind::PlusOne, 1));
    game.set_counters(blueprint, Counters::of(CounterKind::PlusOne, 2));
    let reflection = put(&mut game, player, "Fleeting Reflection", Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: reflection });
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(recipient),
        },
    );
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(recipient),
        },
    );
    choose_targets(&mut game, &[Target::Card(blueprint)]);
    pay(&mut game);
    let mut removed = game.clone();
    removed.move_object(blueprint, Zone::Graveyard);
    resolve(&mut removed);
    assert_eq!(removed.object_view(recipient).name, "Grizzly Bears");
    assert!(!removed.object_view(recipient).tapped);
    assert!(removed.keywords(recipient).contains(Keywords::HEXPROOF));

    resolve(&mut game);
    assert_eq!(game.object_view(recipient).name, "Hill Giant");
    assert_eq!(stats(&game, recipient), (4, 4));
    assert!(game.object_view(blueprint).tapped);
    assert!(!game.keywords(blueprint).contains(Keywords::HEXPROOF));
}

#[test]
fn form_a_posse_uses_the_chosen_x_and_creates_printed_tokens() {
    let player = Player::P0;
    for value in [0, 3] {
        let mut game = scenario(player);
        let spell = put(&mut game, player, "Form a Posse", Zone::Hand);
        cast_x(&mut game, spell, value, &[]);
        resolve(&mut game);
        let posse = tokens(&game, player, MERCENARY);
        assert_eq!(posse.len(), value as usize);
        for mercenary in posse {
            assert_eq!(stats(&game, mercenary), (1, 1));
            assert_eq!(game.object_view(mercenary).colors, 8);
            assert!(game.object_view(mercenary).has_subtype("Mercenary"));
        }
    }
}

#[test]
fn forsaken_miner_returns_only_the_exact_graveyard_card_after_a_paid_crime_trigger() {
    let player = Player::P0;
    let mut game = scenario(player);
    let miner = put(&mut game, player, "Forsaken Miner", Zone::Graveyard);
    let shock = put(&mut game, player, "Shock", Zone::Hand);
    cast(&mut game, shock, Some(Target::Player(player.other())));
    checkpoint(&mut game, Action::DeclineResolutionMana);
    let mut decline = game.clone();
    act(&mut decline, Action::DeclineResolutionMana);
    resolve(&mut decline);
    assert_eq!(decline.object_view(miner).zone, Zone::Graveyard);
    act(
        &mut game,
        Action::PayResolutionMana {
            payment: [0, 0, 1, 0, 0, 0],
        },
    );
    resolve(&mut game);
    assert_eq!(game.object_view(miner).zone, Zone::Battlefield);
    assert!(!game.object_view(miner).tapped);
    assert!(game.keywords(miner).contains(Keywords::CANT_BLOCK));
}

#[test]
fn fortune_blinks_itself_and_one_paid_rider_at_end_of_combat() {
    let player = Player::P0;
    let mut game = scenario(player);
    let fortune = put(&mut game, player, "Fortune, Loyal Steed", Zone::Battlefield);
    let rider = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    let exact_rider = game.object_ref(rider);
    let exact_mount = game.object_ref(fortune);
    game.record_control(exact_rider, player, None);
    saddle(&mut game, fortune, &[rider]);
    attack(&mut game, fortune);
    resolve(&mut game);
    assert_eq!(game.object_ref(fortune), exact_mount);
    let selection = Action::ChooseEffectObject {
        object: exact_rider,
    };
    checkpoint(&mut game, selection.clone());
    assert_eq!(game.state.step, Step::EndCombat);
    apply_raw_and_index(&mut game, selection);
    assert_ne!(game.object_ref(fortune), exact_mount);
    assert_ne!(game.object_ref(rider), exact_rider);
    assert_eq!(game.object_view(rider).controller, player.other());
    advance(&mut game, |game| {
        game.observe(player).library_choice.is_some()
    });
    assert_eq!(
        game.observe(player).library_choice.unwrap().remaining.len(),
        2
    );
}

#[test]
fn freestrider_lookout_searches_a_private_five_once_and_puts_a_land_in_tapped() {
    let player = Player::P1;
    let mut game = scenario(player);
    let lookout = put(&mut game, player, "Freestrider Lookout", Zone::Battlefield);
    let land = put(&mut game, player, "Mountain", Zone::Library);
    let mut rest = Vec::new();
    for name in ["Shock", "Opt", "Negate", "Grizzly Bears"] {
        rest.push(put(&mut game, player, name, Zone::Library));
    }
    assert!(game.keywords(lookout).contains(Keywords::REACH));
    let crime = put(&mut game, player, "Shock", Zone::Hand);
    cast(&mut game, crime, Some(Target::Player(player.other())));
    let pick = Action::SelectLibrary { card: land };
    checkpoint(&mut game, pick.clone());
    assert!(game.observe(player.other()).library_selection.is_none());
    reject(&mut game, Action::SelectLibrary { card: rest[0] });
    apply_raw_and_index(&mut game, pick);
    act(&mut game, Action::FinishLibrarySelection);
    resolve(&mut game);
    assert_eq!(game.object_view(land).zone, Zone::Battlefield);
    assert!(game.object_view(land).tapped);
    assert!(
        game.state.players[player.index()].library[..4]
            .iter()
            .all(|id| rest.contains(id))
    );
    let next = put(&mut game, player, "Shock", Zone::Hand);
    cast(&mut game, next, Some(Target::Player(player.other())));
    resolve(&mut game);
}

#[test]
fn full_steam_ahead_gives_current_creatures_a_removable_single_blocker_ability() {
    let player = Player::P0;
    let mut game = scenario(player);
    let attacker = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let first = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    let second = put(&mut game, player.other(), "Goblin Piker", Zone::Battlefield);
    let steam = put(&mut game, player, "Full Steam Ahead", Zone::Hand);
    cast(&mut game, steam, None);
    resolve(&mut game);
    let later = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    assert_eq!(stats(&game, attacker), (4, 4));
    assert_eq!(stats(&game, later), (2, 2));
    assert!(game.keywords(attacker).contains(Keywords::TRAMPLE));
    assert!(!game.keywords(later).contains(Keywords::MAX_ONE_BLOCKER));
    attack(&mut game, attacker);
    advance(&mut game, |game| game.state.pending == Pending::Blockers);
    act(
        &mut game,
        Action::Block {
            blocker: first,
            attacker,
        },
    );
    reject(
        &mut game,
        Action::Block {
            blocker: second,
            attacker,
        },
    );
}

#[test]
fn geralf_counts_preexisting_casts_and_other_zombie_entries() {
    let player = Player::P1;
    let mut game = scenario(player);
    let prior = put(&mut game, player, "Diregraf Ghoul", Zone::Hand);
    cast(&mut game, prior, None);
    resolve(&mut game);
    game.move_object(prior, Zone::Graveyard);
    put(
        &mut game,
        player,
        "Geralf, the Fleshwright",
        Zone::Battlefield,
    );
    for count in 1..=2 {
        let spell = put(&mut game, player, "Shock", Zone::Hand);
        cast(&mut game, spell, Some(Target::Player(player.other())));
        resolve(&mut game);
        let rogues = tokens(&game, player, ZOMBIE_ROGUE);
        assert_eq!(rogues.len(), count);
        let latest = game.object_view(*rogues.last().unwrap());
        assert!(latest.has_subtype("Zombie") && latest.has_subtype("Rogue"));
        assert_eq!(latest.colors, 6);
        assert_eq!(latest.counters.plus_one, count as u64);
    }
}

#[test]
fn getaway_glamer_blinks_first_and_evaluates_current_power_for_destruction() {
    let player = Player::P0;
    let mut game = scenario(player);
    let high = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    let tied = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    let victim = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    let spell = put(&mut game, player, "Getaway Glamer", Zone::Hand);
    let exact = game.object_ref(high);
    spree(
        &mut game,
        spell,
        2,
        &[Target::Card(high), Target::Card(victim)],
    );
    resolve(&mut game);
    assert_eq!(game.object_view(high).zone, Zone::Exile);
    assert_eq!(game.object_view(victim).zone, Zone::Graveyard);
    assert_eq!(game.object_view(tied).zone, Zone::Battlefield);
    start_step(&mut game, Step::End);
    resolve(&mut game);
    assert_eq!(game.object_view(high).zone, Zone::Battlefield);
    assert_eq!(game.object_view(high).controller, player.other());
    assert_ne!(game.object_ref(high), exact);
}

#[test]
fn ghired_grants_an_actual_tap_ability_to_copy_a_current_controlled_token() {
    let player = Player::P0;
    let mut game = scenario(player);
    let ghired = put(&mut game, player, "Ghired, Mirror of the Wilds", Zone::Hand);
    cast(&mut game, ghired, None);
    resolve(&mut game);
    let fresh = game.create_token_batch_capture(player, TREASURE, 1, true, None, true)[0];
    let foreign =
        game.create_token_batch_capture(player.other(), TREASURE, 1, false, None, true)[0];
    assert!(game.keywords(ghired).contains(Keywords::HASTE));
    let source = game.object_ref(ghired);
    act(
        &mut game,
        Action::AnnounceCreatureGrant {
            source,
            grantor: source,
            ability: 0,
        },
    );
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(foreign.card),
        },
    );
    choose_targets(&mut game, &[Target::Card(fresh.card)]);
    pay(&mut game);
    resolve(&mut game);
    assert!(game.object_view(ghired).tapped);
    let ours = tokens(&game, player, TREASURE);
    assert_eq!(ours.len(), 2);
    assert!(!game.object_view(*ours.last().unwrap()).tapped);
}

#[test]
fn giant_beaver_rewards_only_a_current_exact_saddle_contributor() {
    let player = Player::P1;
    let mut game = scenario(player);
    let beaver = put(&mut game, player, "Giant Beaver", Zone::Battlefield);
    let first = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let second = put(&mut game, player, "Goblin Piker", Zone::Battlefield);
    let idle = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    saddle(&mut game, beaver, &[first, second]);
    attack(&mut game, beaver);
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(idle),
        },
    );
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(second),
        },
    );
    resolve(&mut game);
    assert_eq!(game.counter_count(second, CounterKind::PlusOne), 1);
    assert_eq!(game.counter_count(first, CounterKind::PlusOne), 0);
    assert!(!game.object_view(beaver).tapped);
}

#[test]
fn gisa_grants_menace_and_one_buff_and_makes_a_tapped_batch_once_per_turn() {
    let player = Player::P0;
    let mut game = scenario(player);
    let gisa = put(&mut game, player, "Gisa, the Hellraiser", Zone::Battlefield);
    let skeleton = put(&mut game, player, "Forsaken Miner", Zone::Battlefield);
    let rival = put(
        &mut game,
        player.other(),
        "Diregraf Ghoul",
        Zone::Battlefield,
    );
    assert_eq!(stats(&game, skeleton), (3, 3));
    assert!(game.keywords(skeleton).contains(Keywords::MENACE));
    assert_eq!(stats(&game, rival), (2, 2));

    for _ in 0..2 {
        let shock = put(&mut game, player, "Shock", Zone::Hand);
        cast(&mut game, shock, Some(Target::Player(player.other())));
        resolve(&mut game);
        let rogues = tokens(&game, player, ZOMBIE_ROGUE);
        assert_eq!(rogues.len(), 2);
        for rogue in rogues {
            assert_eq!(stats(&game, rogue), (3, 3));
            assert!(game.object_view(rogue).tapped);
        }
    }

    act(&mut game, Action::Pass);
    let shock = put(&mut game, player.other(), "Shock", Zone::Hand);
    cast(&mut game, shock, Some(Target::Card(gisa)));
    checkpoint(&mut game, Action::DeclineResolutionMana);
    choose(&mut game, |action| {
        matches!(action, Action::PayResolutionMana { .. })
    });
    resolve(&mut game);
    assert_eq!(game.object_view(gisa).damage, 2);
    assert_eq!(game.state.players[player.other().index()].life, 14);
}

#[test]
fn gold_rush_counts_its_treasure_and_may_be_cast_without_a_target() {
    let player = Player::P0;
    let mut game = scenario(player);
    let bear = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    let spell = put(&mut game, player, "Gold Rush", Zone::Hand);
    let mut no_target = game.clone();
    cast(&mut no_target, spell, None);
    resolve(&mut no_target);
    assert_eq!(tokens(&no_target, player, TREASURE).len(), 1);
    assert_eq!(stats(&no_target, bear), (2, 2));

    cast(&mut game, spell, Some(Target::Card(bear)));
    let mut fizzled = game.clone();
    fizzled.move_object(bear, Zone::Graveyard);
    resolve(&mut fizzled);
    assert!(tokens(&fizzled, player, TREASURE).is_empty());
    resolve(&mut game);
    assert_eq!(stats(&game, bear), (4, 4));
}

#[test]
fn goldvein_hydra_enters_with_x_and_counts_its_last_power_at_death() {
    let player = Player::P0;
    let mut game = scenario(player);
    let hydra = put(&mut game, player, "Goldvein Hydra", Zone::Hand);
    cast_x(&mut game, hydra, 2, &[]);
    resolve(&mut game);
    let live = game.object_view(hydra);
    assert_eq!((live.power, live.toughness), (2, 2));
    assert_eq!(live.counters.plus_one, 2);
    assert!(
        live.keywords.contains(
            Keywords::VIGILANCE
                .union(Keywords::TRAMPLE)
                .union(Keywords::HASTE)
        )
    );
    let growth = put(&mut game, player, "Giant Growth", Zone::Hand);
    cast(&mut game, growth, Some(Target::Card(hydra)));
    resolve(&mut game);
    game.move_object(hydra, Zone::Graveyard);
    resolve(&mut game);
    let treasure = tokens(&game, player, TREASURE);
    assert_eq!(treasure.len(), 5);
    assert!(treasure.into_iter().all(|id| game.object_view(id).tapped));
}

#[test]
fn great_train_heist_all_modes_schedule_combat_and_tapped_hit_treasures() {
    let player = Player::P0;
    let mut game = scenario(player);
    let attacker = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    game.state.objects[attacker as usize].tapped = true;
    let spell = put(&mut game, player, "Great Train Heist", Zone::Hand);
    start_step(&mut game, Step::BeginCombat);
    game.state.players[player.index()].mana = [8; 6];
    spree(&mut game, spell, 6, &[Target::Player(player.other())]);
    resolve(&mut game);
    assert!(!game.object_view(attacker).tapped);
    assert_eq!(stats(&game, attacker), (3, 2));
    assert!(game.keywords(attacker).contains(Keywords::FIRST_STRIKE));
    assert_eq!(game.observe(player).additional_combats, 1);
    attack(&mut game, attacker);
    advance(&mut game, |game| {
        game.state.step == Step::EndCombat && game.state.stack.is_empty()
    });
    assert_eq!(game.state.players[player.other().index()].life, 17);
    let treasure = tokens(&game, player, TREASURE);
    assert_eq!(treasure.len(), 1);
    assert!(game.object_view(treasure[0]).tapped);
}

#[test]
fn explosive_derailment_inherited_spree_choices_use_distinct_or_shared_targets() {
    let player = Player::P0;
    for (mode, damage, destroy) in [(0, true, false), (1, false, true), (2, true, true)] {
        let mut game = scenario(player);
        let creature = put(
            &mut game,
            player.other(),
            "Grizzly Bears",
            Zone::Battlefield,
        );
        let artifact = put(&mut game, player.other(), "Short Bow", Zone::Battlefield);
        let spell = put(&mut game, player, "Explosive Derailment", Zone::Hand);
        let targets = [
            damage.then_some(Target::Card(creature)),
            destroy.then_some(Target::Card(artifact)),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
        spree(&mut game, spell, mode, &targets);
        resolve(&mut game);
        assert_eq!(game.on_battlefield(creature), !damage);
        assert_eq!(game.on_battlefield(artifact), !destroy);
    }
}

#[test]
fn geyser_drake_discounts_spells_during_the_opponents_turn() {
    let player = Player::P0;
    let mut game = scenario(player.other());
    let drake = put(&mut game, player, "Geyser Drake", Zone::Battlefield);
    let strike = put(&mut game, player, "Lightning Strike", Zone::Hand);
    let cow = put(&mut game, player, "Holy Cow", Zone::Hand);
    assert!(game.keywords(drake).contains(Keywords::FLYING));
    act(&mut game, Action::Pass);
    game.state.players[player.index()].mana = [1, 0, 0, 0, 0, 1];
    let mut own_turn = game.clone();
    own_turn.state.active = player;
    reject(&mut own_turn, Action::AnnounceCast { card: cow });

    let mut off_turn_creature = game.clone();
    cast(&mut off_turn_creature, cow, None);
    assert_eq!(off_turn_creature.object_view(cow).zone, Zone::Stack);
    assert_eq!(off_turn_creature.state.players[player.index()].mana, [0; 6]);

    game.state.players[player.index()].mana = [0, 0, 0, 1, 0, 0];
    let mut own_turn = game.clone();
    own_turn.state.active = player;
    reject(&mut own_turn, Action::AnnounceCast { card: strike });
    cast(&mut game, strike, Some(Target::Player(player.other())));
    resolve(&mut game);
    assert_eq!(game.state.players[player.other().index()].life, 17);
}

#[test]
fn inherited_deserts_enter_tapped_and_deal_one_to_an_opponent_then_tap_for_either_color() {
    for (name, colors) in [
        ("Eroded Canyon", [1, 3]),
        ("Festering Gulch", [2, 4]),
        ("Forlorn Flats", [0, 2]),
        ("Jagged Barrens", [2, 3]),
        ("Lonely Arroyo", [0, 1]),
        ("Lush Oasis", [1, 4]),
    ] {
        let player = Player::P1;
        let mut game = scenario(player);
        let land = put(&mut game, player, name, Zone::Hand);
        act(&mut game, Action::PlayLand { card: land });
        assert!(game.object_view(land).tapped);
        assert!(game.object_view(land).has_subtype("Desert"));
        reject(
            &mut game,
            Action::ChooseTarget {
                target: Target::Player(player),
            },
        );
        act(
            &mut game,
            Action::ChooseTarget {
                target: Target::Player(player.other()),
            },
        );
        resolve(&mut game);
        assert_eq!(game.state.players[player.other().index()].life, 19);
        game.state.objects[land as usize].tapped = false;
        let available: Vec<_> = game
            .legal_actions()
            .into_iter()
            .filter_map(|action| {
                if let Action::ActivateMana {
                    card, production, ..
                } = action
                    && card == land
                {
                    return production.iter().position(|amount| *amount == 1);
                }
                None
            })
            .collect();
        assert_eq!(available, colors, "{name}");
    }
}

#[test]
fn eriettes_lullaby_gains_life_for_a_legal_tapped_indestructible_target() {
    let player = Player::P0;
    let mut game = scenario(player);
    let creature = put(
        &mut game,
        player.other(),
        "Kotis, the Fangkeeper",
        Zone::Battlefield,
    );
    let lullaby = put(&mut game, player, "Eriette's Lullaby", Zone::Hand);
    reject(&mut game, Action::AnnounceCast { card: lullaby });
    game.state.objects[creature as usize].tapped = true;
    cast(&mut game, lullaby, Some(Target::Card(creature)));
    let mut untapped = game.clone();
    untapped.state.objects[creature as usize].tapped = false;
    resolve(&mut untapped);
    assert_eq!(untapped.state.players[player.index()].life, 20);
    resolve(&mut game);
    assert_eq!(game.object_view(creature).zone, Zone::Battlefield);
    assert_eq!(game.state.players[player.index()].life, 22);
}

#[test]
fn failed_fording_bounces_a_nonland_and_surveys_with_a_desert() {
    let player = Player::P0;
    let mut game = scenario(player);
    let desert = put(&mut game, player, "Eroded Canyon", Zone::Battlefield);
    let creature = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    let top = put(&mut game, player, "Forest", Zone::Library);
    let spell = put(&mut game, player, "Failed Fording", Zone::Hand);
    cast(&mut game, spell, Some(Target::Card(creature)));
    let mut no_desert = game.clone();
    no_desert.move_object(desert, Zone::Graveyard);
    resolve(&mut no_desert);
    assert_eq!(no_desert.object_view(creature).zone, Zone::Hand);
    assert_eq!(no_desert.object_view(top).zone, Zone::Library);
    choose_when_ready(
        &mut game,
        Action::ArrangeLibrary {
            card: top,
            destination: LibraryDestination::Graveyard,
        },
    );
    assert_eq!(game.object_view(creature).zone, Zone::Hand);
    assert_eq!(game.object_view(top).zone, Zone::Graveyard);
}

#[test]
fn fake_your_own_death_returns_a_stolen_creature_tapped_to_its_owner() {
    let player = Player::P0;
    let mut game = scenario(player);
    let creature = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    let spell = put(&mut game, player, "Fake Your Own Death", Zone::Hand);
    let original = game.object_ref(creature);
    game.record_control(original, player, None);
    cast(&mut game, spell, Some(Target::Card(creature)));
    resolve(&mut game);
    assert_eq!(stats(&game, creature), (4, 2));
    game.move_object(creature, Zone::Graveyard);
    resolve(&mut game);
    let returned = game.object_view(creature);
    assert_eq!(
        (returned.zone, returned.controller, returned.tapped),
        (Zone::Battlefield, player.other(), true)
    );
    assert_ne!(returned.reference, original);
    assert_eq!(tokens(&game, player, TREASURE).len(), 1);
    assert!(tokens(&game, player.other(), TREASURE).is_empty());
}

#[test]
fn ferocification_selects_its_mode_and_own_target_as_combat_begins() {
    let player = Player::P0;
    for mode in 0..2 {
        let mut game = scenario(player);
        put(&mut game, player, "Ferocification", Zone::Battlefield);
        let own = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
        let rival = put(
            &mut game,
            player.other(),
            "Grizzly Bears",
            Zone::Battlefield,
        );
        start_step(&mut game, Step::BeginCombat);
        act(&mut game, Action::ChooseMode { mode });
        reject(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(rival),
            },
        );
        act(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(own),
            },
        );
        resolve(&mut game);
        assert_eq!(stats(&game, own), (if mode == 0 { 4 } else { 2 }, 2));
        assert_eq!(
            game.keywords(own)
                .contains(Keywords::MENACE.union(Keywords::HASTE)),
            mode == 1
        );
    }
}

#[test]
fn freestrider_commando_counts_cast_mana_and_not_its_plot_cost() {
    let player = Player::P0;
    let mut game = scenario(player);
    let commando = put(&mut game, player, "Freestrider Commando", Zone::Hand);
    let mut normal = game.clone();
    cast(&mut normal, commando, None);
    resolve(&mut normal);
    assert_eq!(normal.counter_count(commando, CounterKind::PlusOne), 0);
    act(&mut game, Action::AnnouncePlot { card: commando });
    choose(&mut game, |action| matches!(action, Action::PayPlot { .. }));
    game.state.turn += 1;
    act(&mut game, Action::AnnouncePlotted { card: commando });
    pay(&mut game);
    resolve(&mut game);
    assert_eq!(game.counter_count(commando, CounterKind::PlusOne), 2);
}

#[test]
fn frontier_seeker_privately_finds_a_mount_or_a_plains() {
    let player = Player::P0;
    let mut game = scenario(player);
    let mount = put(&mut game, player, "Gila Courser", Zone::Library);
    let plains = put(&mut game, player, "Plains", Zone::Library);
    let forest = put(&mut game, player, "Forest", Zone::Library);
    let seeker = put(&mut game, player, "Frontier Seeker", Zone::Hand);
    cast(&mut game, seeker, None);
    checkpoint(&mut game, Action::SelectLibrary { card: mount });
    assert!(game.observe(player.other()).library_selection.is_none());
    reject(&mut game, Action::SelectLibrary { card: forest });
    let mut choose_plains = game.clone();
    act(&mut choose_plains, Action::SelectLibrary { card: plains });
    act(&mut choose_plains, Action::FinishLibrarySelection);
    assert_eq!(choose_plains.object_view(plains).zone, Zone::Hand);
    act(&mut game, Action::SelectLibrary { card: mount });
    act(&mut game, Action::FinishLibrarySelection);
    assert_eq!(game.object_view(mount).zone, Zone::Hand);
}

#[test]
fn gila_courser_exiles_for_play_only_after_a_saddled_attack() {
    let player = Player::P0;
    let mut game = scenario(player);
    let courser = put(&mut game, player, "Gila Courser", Zone::Battlefield);
    let rider = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let land = put(&mut game, player, "Forest", Zone::Library);
    let mut unsaddled = game.clone();
    attack(&mut unsaddled, courser);
    resolve(&mut unsaddled);
    assert_eq!(unsaddled.object_view(land).zone, Zone::Library);
    saddle(&mut game, courser, &[rider]);
    attack(&mut game, courser);
    resolve(&mut game);
    assert_eq!(game.object_view(land).zone, Zone::Exile);
    choose_when_ready(&mut game, Action::PlayLand { card: land });
    assert_eq!(game.object_view(land).zone, Zone::Battlefield);
}
