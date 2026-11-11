//! Inherited A–D Outlaws definitions exercised through real game actions.
#[path = "support/otj_e_l.rs"]
mod scenarios;
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use scenarios::{announce_activation, choose_targets, resolve, saddle, tokens};
use support::*;

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

#[test]
fn three_desert_duals_enter_tapped_deal_damage_to_the_opponent_and_make_either_color() {
    let player = Player::P0;
    for (name, colors) in [
        ("Abraded Bluffs", [0, 3]),
        ("Bristling Backwoods", [3, 4]),
        ("Creosote Heath", [0, 4]),
    ] {
        let mut game = scenario(player);
        let land = put(&mut game, player, name, Zone::Hand);
        act(&mut game, Action::PlayLand { card: land });
        assert!(game.object_view(land).tapped && game.has_subtype(land, "Desert"));
        reject(
            &mut game,
            Action::ChooseTarget {
                target: Target::Player(player),
            },
        );
        choose_targets(&mut game, &[Target::Player(player.other())]);
        resolve(&mut game);
        assert_eq!(game.state.players[player.other().index()].life, 19);
        game.state.objects[land as usize].tapped = false;
        for color in colors {
            let mut branch = game.clone();
            let mut production = [0; 6];
            production[color] = 1;
            choose(&mut branch, |a| {
                matches!(a, Action::ActivateMana { card, production: mana, .. }
                if *card == land && *mana == production)
            });
            assert!(branch.object_view(land).tapped);
        }
    }
}

#[test]
fn three_fastlands_use_the_other_land_threshold_and_both_printed_colors() {
    let player = Player::P1;
    for (name, colors) in [
        ("Blooming Marsh", [2, 4]),
        ("Botanical Sanctum", [1, 4]),
        ("Concealed Courtyard", [0, 2]),
    ] {
        for existing in [2, 3] {
            let mut game = scenario(player);
            for _ in 0..existing {
                put(&mut game, player, "Forest", Zone::Battlefield);
            }
            let land = put(&mut game, player, name, Zone::Hand);
            act(&mut game, Action::PlayLand { card: land });
            assert_eq!(game.object_view(land).tapped, existing == 3);
            for color in colors {
                let mut production = [0; 6];
                production[color] = 1;
                assert_eq!(
                    game.legal_actions().iter().any(|a| matches!(a,
                    Action::ActivateMana { card, production: mana, .. }
                    if *card == land && *mana == production)),
                    existing == 2
                );
            }
        }
    }
}

#[test]
fn gigapede_and_thunder_thief_cast_off_turn_and_resolve_their_distinct_entries() {
    let player = Player::P0;
    let mut game = scenario(player);
    let gigapede = put(&mut game, player, "Ambush Gigapede", Zone::Hand);
    let thief = put(&mut game, player, "Daring Thunder-Thief", Zone::Hand);
    let enemy = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    let own = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    game.state.active = player.other();
    game.state.step = Step::Upkeep;
    cast(&mut game, gigapede, None);
    pass_pair(&mut game);
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(own),
        },
    );
    choose_targets(&mut game, &[Target::Card(enemy)]);
    resolve(&mut game);
    assert_eq!(stats(&game, enemy), (1, 1));
    game.state.priority = player;
    cast(&mut game, thief, None);
    resolve(&mut game);
    assert!(game.object_view(thief).tapped);
}

#[test]
fn ankle_biter_deathtouch_makes_one_damage_lethal_to_a_larger_blocker() {
    let player = Player::P1;
    let mut game = scenario(player);
    let biter = put(&mut game, player, "Ankle Biter", Zone::Battlefield);
    let giant = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    assert!(game.keywords(biter).contains(Keywords::DEATHTOUCH));
    game.apply_damage_batch(&[DamageEvent {
        preventable: true,
        source: game.damage_source(biter),
        target: Target::Card(giant),
        amount: 1,
        combat: true,
    }]);
    game.state_based_actions();
    assert_eq!(game.object_view(giant).zone, Zone::Graveyard);
}

#[test]
fn outcaster_rechecks_ferocious_and_buzzard_samples_a_real_death_before_entry() {
    let player = Player::P0;
    let mut game = scenario(player);
    let large = put(&mut game, player, "Daring Thunder-Thief", Zone::Battlefield);
    let outcaster = put(&mut game, player, "Beastbond Outcaster", Zone::Hand);
    let buzzard = put(&mut game, player, "Blacksnag Buzzard", Zone::Hand);
    cast(&mut game, outcaster, None);
    pass_pair(&mut game);
    let mut draw = game.clone();
    resolve(&mut draw);
    assert_eq!(draw.state.players[player.index()].cards_drawn_this_turn, 1);
    game.move_object(large, Zone::Graveyard);
    resolve(&mut game);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 0);
    cast(&mut game, buzzard, None);
    resolve(&mut game);
    assert_eq!(game.counter_count(buzzard, CounterKind::PlusOne), 1);
    assert!(game.keywords(buzzard).contains(Keywords::FLYING));
}

#[test]
fn bonny_creates_live_legendary_beau_and_attack_puts_a_land_from_the_graveyard() {
    let player = Player::P1;
    let mut game = scenario(player);
    let bonny = put(&mut game, player, "Bonny Pall, Clearcutter", Zone::Hand);
    put(&mut game, player, "Forest", Zone::Battlefield);
    let land = put(&mut game, player, "Island", Zone::Graveyard);
    let attacker = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    cast(&mut game, bonny, None);
    resolve(&mut game);
    let beau = tokens(&game, player, Card::from_catalogue_id(5251))[0];
    assert!(game.object_view(beau).is_legendary());
    assert_eq!(stats(&game, beau), (1, 1));
    assert!(game.keywords(bonny).contains(Keywords::REACH));
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    act(&mut game, Action::Attack { card: attacker });
    act(&mut game, Action::FinishAttackers);
    pass_pair(&mut game);
    let object = game.object_ref(land);
    act(&mut game, Action::SelectLand { object });
    if game.legal_actions().contains(&Action::FinishLandSelection) {
        act(&mut game, Action::FinishLandSelection);
    }
    assert!(game.on_battlefield(land));
    assert_eq!(stats(&game, beau), (2, 2));
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
}

#[test]
fn four_inherited_mounts_use_actual_paid_saddle_before_their_attack_payoff() {
    let player = Player::P0;
    for name in [
        "Bounding Felidar",
        "Bridled Bighorn",
        "Congregation Gryff",
        "Drover Grizzly",
    ] {
        let mut game = scenario(player);
        let mount = put(&mut game, player, name, Zone::Battlefield);
        let rider = put(&mut game, player, "Hill Giant", Zone::Battlefield);
        let other = put(&mut game, player, "Trained Arynx", Zone::Battlefield);
        saddle(&mut game, mount, &[rider]);
        assert!(game.object_view(mount).saddled && game.object_view(rider).tapped);
        game.state.step = Step::Attackers;
        game.state.pending = Pending::Attackers;
        act(&mut game, Action::Attack { card: mount });
        act(&mut game, Action::FinishAttackers);
        resolve(&mut game);
        match name {
            "Bounding Felidar" => {
                assert_eq!(game.counter_count(mount, CounterKind::PlusOne), 0);
                assert_eq!(game.counter_count(rider, CounterKind::PlusOne), 1);
                assert_eq!(game.counter_count(other, CounterKind::PlusOne), 1);
                assert_eq!(game.state.players[player.index()].life, 22);
            }
            "Bridled Bighorn" => {
                assert!(!game.object_view(mount).tapped);
                assert_eq!(
                    tokens(&game, player, Card::from_catalogue_id(5322)).len(),
                    1
                );
            }
            "Congregation Gryff" => {
                assert_eq!(stats(&game, mount), (3, 6));
                assert!(
                    game.keywords(mount)
                        .contains(Keywords::FLYING.union(Keywords::LIFELINK))
                );
            }
            _ => assert!(
                [mount, rider, other]
                    .iter()
                    .all(|&id| game.keywords(id).contains(Keywords::TRAMPLE))
            ),
        }
    }
}

#[test]
fn bovine_intervention_gives_the_ox_to_the_controller_of_a_legal_indestructible_target() {
    let player = Player::P1;
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Bovine Intervention", Zone::Hand);
    let victim = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    let source = game.object_ref(victim);
    game.grant_keyword_effects(&[victim], source, Keywords::INDESTRUCTIBLE);
    cast(&mut game, spell, Some(Target::Card(victim)));
    resolve(&mut game);
    assert!(game.on_battlefield(victim));
    let ox = tokens(&game, player.other(), Card::from_catalogue_id(5479));
    assert_eq!(ox.len(), 1);
    assert_eq!(stats(&game, ox[0]), (2, 2));
}

#[test]
fn cactarantula_uses_its_own_desert_discount_and_an_opponents_targeting() {
    let player = Player::P0;
    let mut game = scenario(player);
    game.state.players[player.index()].mana = [0, 0, 0, 0, 2, 3];
    let spider = put(&mut game, player, "Cactarantula", Zone::Hand);
    put(
        &mut game,
        player.other(),
        "Abraded Bluffs",
        Zone::Battlefield,
    );
    reject(&mut game, Action::AnnounceCast { card: spider });
    let desert = put(&mut game, player, "Bristling Backwoods", Zone::Battlefield);
    game.state.objects[desert as usize].tapped = true;
    cast(&mut game, spider, None);
    resolve(&mut game);
    assert_eq!(stats(&game, spider), (6, 5));
    assert!(game.keywords(spider).contains(Keywords::REACH));
    game.state.priority = player.other();
    let shock = put(&mut game, player.other(), "Shock", Zone::Hand);
    cast(&mut game, shock, Some(Target::Card(spider)));
    pass_pair(&mut game);
    let mut decline = game.clone();
    act(&mut decline, Action::ChooseOptional { accept: false });
    act(&mut game, Action::ChooseOptional { accept: true });
    resolve(&mut game);
    resolve(&mut decline);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
    assert_eq!(
        decline.state.players[player.index()].cards_drawn_this_turn,
        0
    );
    assert_eq!(game.object_view(spider).damage, 2);
}

#[test]
fn conduit_pylons_surveils_privately_and_pays_for_colored_production() {
    let player = Player::P1;
    let mut game = scenario(player);
    let land = put(&mut game, player, "Conduit Pylons", Zone::Hand);
    let top = put(&mut game, player, "Mountain", Zone::Library);
    act(&mut game, Action::PlayLand { card: land });
    assert!(!game.object_view(land).tapped && game.has_subtype(land, "Desert"));
    pass_pair(&mut game);
    assert!(
        game.observe(player)
            .library_choice
            .as_ref()
            .unwrap()
            .surveil
    );
    assert!(game.observe(player.other()).library_choice.is_none());
    act(
        &mut game,
        Action::ArrangeLibrary {
            card: top,
            destination: LibraryDestination::Graveyard,
        },
    );
    assert_eq!(game.object_view(top).zone, Zone::Graveyard);
    let mut branch = game.clone();
    choose(&mut branch, |action| {
        matches!(action, Action::ActivateMana {
        card, production: [0, 0, 0, 0, 0, 1], ..
    } if *card == land)
    });
    assert_eq!(branch.state.players[player.index()].mana[5], 9);
    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 1];
    for color in 0..5 {
        let mut branch = game.clone();
        let mut desired = [0; 6];
        desired[color] = 1;
        choose(&mut branch, |action| {
            matches!(action, Action::ActivateMana { card, production, .. }
            if *card == land && *production == desired)
        });
        assert_eq!(branch.state.players[player.index()].mana, desired);
        assert!(branch.object_view(land).tapped);
    }
}

#[test]
fn corrupted_conviction_sacrifices_only_its_controllers_creature_as_a_cost() {
    let player = Player::P0;
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Corrupted Conviction", Zone::Hand);
    let opposing = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    reject(&mut game, Action::AnnounceCast { card: spell });
    let own = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    act(&mut game, Action::AnnounceCast { card: spell });
    let object = game.object_ref(opposing);
    reject(&mut game, Action::ChooseAdditionalCost { object });
    let object = game.object_ref(own);
    act(&mut game, Action::ChooseAdditionalCost { object });
    pay(&mut game);
    assert_eq!(game.object_view(own).zone, Zone::Graveyard);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 0);
    resolve(&mut game);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 2);
}

fn plot_and_announce(game: &mut Game, card: CardId) {
    act(game, Action::AnnouncePlot { card });
    choose(game, |a| matches!(a, Action::PayPlot { .. }));
    assert_eq!(game.object_view(card).zone, Zone::Exile);
    reject(game, Action::AnnouncePlotted { card });
    game.state.turn += 1;
    act(game, Action::AnnouncePlotted { card });
}

#[test]
fn coyote_ruckus_and_djinn_use_their_exact_plot_prices_and_printed_effects() {
    let player = Player::P1;
    for (name, mana) in [
        ("Cunning Coyote", [0, 0, 0, 1, 0, 1]),
        ("Demonic Ruckus", [0, 0, 0, 1, 0, 0]),
        ("Djinn of Fool's Fall", [0, 1, 0, 0, 0, 3]),
    ] {
        let mut game = scenario(player);
        let card = put(&mut game, player, name, Zone::Hand);
        let own = put(&mut game, player, "Hill Giant", Zone::Battlefield);
        let opposing = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
        game.state.players[player.index()].mana = mana;
        plot_and_announce(&mut game, card);
        if name == "Demonic Ruckus" {
            choose_targets(&mut game, &[Target::Card(opposing)]);
        }
        pay(&mut game);
        assert_eq!(game.state.players[player.index()].mana, [0; 6]);
        pass_pair(&mut game);
        match name {
            "Cunning Coyote" => {
                assert!(game.keywords(card).contains(Keywords::HASTE));
                for target in [card, opposing] {
                    reject(
                        &mut game,
                        Action::ChooseTarget {
                            target: Target::Card(target),
                        },
                    );
                }
                choose_targets(&mut game, &[Target::Card(own)]);
                resolve(&mut game);
                assert_eq!(stats(&game, own), (4, 4));
                assert!(game.keywords(own).contains(Keywords::HASTE));
            }
            "Demonic Ruckus" => {
                assert_eq!(stats(&game, opposing), (4, 4));
                assert!(
                    game.keywords(opposing)
                        .contains(Keywords::MENACE.union(Keywords::TRAMPLE))
                );
                game.move_object(opposing, Zone::Graveyard);
                game.state_based_actions();
                resolve(&mut game);
                assert_eq!(game.object_view(card).zone, Zone::Graveyard);
                assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
            }
            _ => {
                assert_eq!(stats(&game, card), (4, 3));
                assert!(game.keywords(card).contains(Keywords::FLYING));
            }
        }
    }
}

#[test]
fn deadeye_duelist_pays_and_taps_to_damage_only_the_opponent() {
    let player = Player::P0;
    let mut game = scenario(player);
    let duelist = put(&mut game, player, "Deadeye Duelist", Zone::Battlefield);
    assert!(game.keywords(duelist).contains(Keywords::REACH));
    game.state.players[player.index()].mana = [0; 6];
    reject(
        &mut game,
        Action::AnnounceActivate {
            card: duelist,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    game.state.players[player.index()].mana[5] = 1;
    announce_activation(&mut game, duelist, 0);
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(player),
        },
    );
    choose_targets(&mut game, &[Target::Player(player.other())]);
    pay(&mut game);
    resolve(&mut game);
    assert!(game.object_view(duelist).tapped);
    assert_eq!(game.state.players[player.other().index()].life, 19);
}

#[test]
fn bloodseeker_can_mill_either_player_and_has_lifelink() {
    let player = Player::P1;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Desperate Bloodseeker", Zone::Hand);
    cast(&mut game, source, None);
    pass_pair(&mut game);
    assert!(game.keywords(source).contains(Keywords::LIFELINK));
    for who in [player, player.other()] {
        let mut branch = game.clone();
        choose_targets(&mut branch, &[Target::Player(who)]);
        resolve(&mut branch);
        assert_eq!(branch.state.players[who.index()].library.len(), 6);
        assert_eq!(branch.state.players[who.other().index()].library.len(), 8);
    }
}

#[test]
fn discerning_peddler_draws_only_after_its_private_optional_discard() {
    let player = Player::P1;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Discerning Peddler", Zone::Hand);
    let discarded = put(&mut game, player, "Plains", Zone::Hand);
    let drawn = put(&mut game, player, "Mountain", Zone::Library);
    cast(&mut game, source, None);
    pass_pair(&mut game);
    pass_pair(&mut game);
    assert!(game.observe(player).effect_choice.is_some());
    assert!(game.observe(player.other()).effect_choice.is_none());
    let mut decline = game.clone();
    act(&mut decline, Action::DeclineEffectPayment);
    assert_eq!(decline.object_view(discarded).zone, Zone::Hand);
    assert_eq!(decline.object_view(drawn).zone, Zone::Library);
    let object = game.object_ref(discarded);
    act(&mut game, Action::ChooseEffectObject { object });
    assert_eq!(game.object_view(discarded).zone, Zone::Graveyard);
    assert_eq!(game.object_view(drawn).zone, Zone::Hand);
}
