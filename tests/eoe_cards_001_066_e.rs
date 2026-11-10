//! Edge of Eternities card scenarios for collector numbers 055 through 066.
#[path = "eoe_cards_001_066/scenario.rs"]
mod common;
mod support;

use common::*;
use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

#[test]
fn divert_disaster_counters_unless_paid_and_awards_the_lander_to_its_caster() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(opponent);
    let divert = put(&mut game, player, "Divert Disaster", Zone::Hand);
    let spell = put(&mut game, opponent, "Short Bow", Zone::Hand);
    cast(&mut game, spell, None);
    opponent_priority(&mut game, player);
    cast(&mut game, divert, Some(Target::Spell(spell)));
    pass_pair(&mut game);
    assert_eq!(game.actor(), Some(opponent));

    let mut unpaid = game.clone();
    act(&mut unpaid, Action::DeclineResolutionMana);
    settle(&mut unpaid);
    assert_eq!(unpaid.object_view(spell).zone, Zone::Graveyard);
    assert!(battlefield_tokens(&unpaid, player, Card(5238)).is_empty());

    let old_mana: u32 = game.state.players[opponent.index()].mana.iter().sum();
    choose(&mut game, |action| {
        matches!(action, Action::PayResolutionMana { .. })
    });
    settle(&mut game);
    assert_eq!(game.object_view(spell).zone, Zone::Battlefield);
    assert_eq!(
        game.state.players[opponent.index()]
            .mana
            .iter()
            .sum::<u32>(),
        old_mana - 2
    );
    assert_eq!(battlefield_tokens(&game, player, Card(5238)).len(), 1);
    assert!(battlefield_tokens(&game, opponent, Card(5238)).is_empty());
}

#[test]
fn emissary_escort_uses_only_the_largest_other_artifact_its_controller_has() {
    let player = Player::P1;
    let mut game = scenario(player);
    let escort = put(&mut game, player, "Emissary Escort", Zone::Hand);
    let bow = put(&mut game, player, "Short Bow", Zone::Hand);
    let axe = put(&mut game, player, "Leyline Axe", Zone::Hand);
    let foreign = put(&mut game, player.other(), "Rescue Skiff", Zone::Battlefield);
    cast(&mut game, escort, None);
    settle(&mut game);
    assert_eq!(stats(&game, escort), (0, 4));

    cast(&mut game, bow, None);
    settle(&mut game);
    assert_eq!(stats(&game, escort), (2, 4));
    cast(&mut game, axe, None);
    settle(&mut game);
    assert_eq!(stats(&game, escort), (4, 4));
    game.change_controller(game.object_ref(foreign), player);
    assert_eq!(stats(&game, escort), (6, 4));
    game.move_object(foreign, Zone::Graveyard);
    assert_eq!(stats(&game, escort), (4, 4));
}

#[test]
fn gigastorm_titan_costs_three_less_after_another_spell_even_if_it_was_countered() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let titan = put(&mut game, player, "Gigastorm Titan", Zone::Hand);
    let prior = put(&mut game, player, "Shock", Zone::Hand);
    let negate = put(&mut game, opponent, "Negate", Zone::Hand);
    let announce = Action::AnnounceCast { card: titan };
    game.state.players[player.index()].mana = [0, 1, 0, 0, 0, 1];
    reject(&mut game, announce.clone());

    let mut first = game.clone();
    first.state.players[player.index()].mana[5] = 4;
    act(&mut first, announce.clone());
    assert_eq!(first.observe(player).payment_request.unwrap().generic, 4);
    pay(&mut first);
    settle(&mut first);
    assert_eq!(stats(&first, titan), (4, 4));

    game.state.players[player.index()].mana[3] = 1;
    cast(&mut game, prior, Some(Target::Player(opponent)));
    opponent_priority(&mut game, opponent);
    cast(&mut game, negate, Some(Target::Spell(prior)));
    settle(&mut game);
    assert_eq!(game.state.players[opponent.index()].life, 20);
    act(&mut game, announce);
    assert_eq!(game.observe(player).payment_request.unwrap().generic, 1);
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(titan).zone, Zone::Battlefield);
}

#[test]
fn illvoi_galeblade_flashes_in_and_sacrifices_without_waiting_for_summoning_sickness() {
    let player = Player::P0;
    let mut game = scenario(player.other());
    let galeblade = put(&mut game, player, "Illvoi Galeblade", Zone::Hand);
    let top = *game.state.players[player.index()].library.last().unwrap();
    game.state.players[player.index()].mana = [0, 1, 0, 0, 0, 2];
    opponent_priority(&mut game, player);
    cast(&mut game, galeblade, None);
    settle(&mut game);
    assert!(game.object_view(galeblade).summoning_sick);
    assert!(
        game.object_view(galeblade)
            .keywords
            .contains(Keywords::FLYING)
    );
    opponent_priority(&mut game, player);

    let mut short = game.clone();
    short.state.players[player.index()].mana[5] = 1;
    reject(
        &mut short,
        Action::AnnounceActivate {
            card: galeblade,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    activate(&mut game, galeblade, 0, None);
    assert_eq!(game.object_view(galeblade).zone, Zone::Graveyard);
    assert_eq!(game.object_view(top).zone, Zone::Library);
    settle(&mut game);
    assert_eq!(game.object_view(top).zone, Zone::Hand);
}

#[test]
fn illvoi_infiltrator_can_be_blocked_after_one_cast_and_draws_for_combat_damage_after_two() {
    let player = Player::P1;
    let mut game = scenario(player);
    let infiltrator = put(&mut game, player, "Illvoi Infiltrator", Zone::Battlefield);
    let blocker = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    let bow = put(&mut game, player, "Short Bow", Zone::Hand);
    let shock = put(&mut game, player, "Shock", Zone::Hand);
    let top = *game.state.players[player.index()].library.last().unwrap();
    cast(&mut game, bow, None);
    settle(&mut game);

    for twice in [false, true] {
        let mut branch = game.clone();
        if twice {
            cast(&mut branch, shock, Some(Target::Player(player.other())));
            settle(&mut branch);
        }
        advance(&mut branch, |game| game.state.pending == Pending::Attackers);
        act(&mut branch, Action::Attack { card: infiltrator });
        act(&mut branch, Action::FinishAttackers);
        advance(&mut branch, |game| game.state.pending == Pending::Blockers);
        let block = Action::Block {
            blocker,
            attacker: infiltrator,
        };
        if twice {
            reject(&mut branch, block);
        } else {
            act(&mut branch, block);
        }
        act(&mut branch, Action::FinishBlockers);
        pass_pair(&mut branch);
        settle(&mut branch);
        assert_eq!(
            branch.object_view(top).zone,
            if twice { Zone::Hand } else { Zone::Library }
        );
        assert_eq!(
            branch.state.players[player.other().index()].life,
            if twice { 17 } else { 20 }
        );
    }
}

#[test]
fn illvoi_light_jammer_flashes_in_and_temporary_hexproof_does_not_follow_reequip() {
    let player = Player::P0;
    for active in [player, player.other()] {
        let mut game = scenario(active);
        let jammer = put(&mut game, player, "Illvoi Light Jammer", Zone::Hand);
        let first = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let second = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let enemy = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
        opponent_priority(&mut game, player);
        cast(&mut game, jammer, None);
        pass_pair(&mut game);
        advance(&mut game, |game| {
            game.legal_actions().contains(&choose_target(first))
        });
        reject(&mut game, choose_target(enemy));
        trigger_target(&mut game, first);
        settle(&mut game);
        assert_eq!(stats(&game, first), (3, 4));
        assert!(
            game.object_view(first)
                .keywords
                .contains(Keywords::HEXPROOF)
        );

        if active == player {
            activate(&mut game, jammer, 0, Some(second));
            settle(&mut game);
            assert_eq!(stats(&game, first), (2, 2));
            assert_eq!(stats(&game, second), (3, 4));
            assert!(
                game.object_view(first)
                    .keywords
                    .contains(Keywords::HEXPROOF)
            );
            assert!(
                !game
                    .object_view(second)
                    .keywords
                    .contains(Keywords::HEXPROOF)
            );
        }
        advance(&mut game, |game| game.state.turn == 4);
        assert!(
            !game
                .object_view(first)
                .keywords
                .contains(Keywords::HEXPROOF)
        );
    }
}

#[test]
fn illvoi_operative_can_be_the_first_spell_and_only_grows_on_the_second() {
    let player = Player::P0;
    let mut game = scenario(player);
    let operative = put(&mut game, player, "Illvoi Operative", Zone::Hand);
    let bow = put(&mut game, player, "Short Bow", Zone::Hand);
    let later = put(&mut game, player, "Bear Cub", Zone::Hand);
    cast(&mut game, operative, None);
    settle(&mut game);
    assert_eq!(stats(&game, operative), (2, 1));
    cast(&mut game, bow, None);
    settle(&mut game);
    assert_eq!(stats(&game, operative), (3, 2));
    cast(&mut game, later, None);
    settle(&mut game);
    assert_eq!(stats(&game, operative), (3, 2));
}

#[test]
fn lost_in_space_lets_the_owner_choose_library_position_before_the_casters_private_surveil() {
    let player = Player::P1;
    let owner = player.other();
    let mut game = scenario(player);
    let lost = put(&mut game, player, "Lost in Space", Zone::Hand);
    let target = put(&mut game, owner, "Bear Cub", Zone::Battlefield);
    let bounce = put(&mut game, player, "Unsummon", Zone::Hand);
    let top = put(&mut game, player, "Island", Zone::Library);
    game.change_controller(game.object_ref(target), player);
    cast(&mut game, lost, Some(Target::Card(target)));

    let mut stale = game.clone();
    cast(&mut stale, bounce, Some(Target::Card(target)));
    settle(&mut stale);
    assert_eq!(stale.object_view(target).zone, Zone::Hand);
    assert_eq!(stale.object_view(top).zone, Zone::Library);

    pass_pair(&mut game);
    assert_eq!(game.actor(), Some(owner));
    let object = game.object_ref(target);
    for destination in [LibraryDestination::Top, LibraryDestination::Bottom] {
        let mut branch = game.clone();
        act(
            &mut branch,
            Action::ChooseLibraryDestination {
                object,
                destination,
            },
        );
        let library = &branch.state.players[owner.index()].library;
        let expected = if destination == LibraryDestination::Top {
            library.last()
        } else {
            library.first()
        };
        assert_eq!(expected.copied(), Some(target));
        assert_eq!(branch.actor(), Some(player));
        assert!(branch.observe(player).library_choice.is_some());
        assert!(branch.observe(owner).library_choice.is_none());
        act(
            &mut branch,
            Action::ArrangeLibrary {
                card: top,
                destination: LibraryDestination::Graveyard,
            },
        );
        settle(&mut branch);
        assert_eq!(branch.object_view(top).zone, Zone::Graveyard);
    }
}

#[test]
fn mechan_assembler_observes_simultaneous_artifacts_once_and_resets_next_turn() {
    let player = Player::P0;
    let mut game = scenario(player);
    put(&mut game, player, "Mechan Assembler", Zone::Battlefield);
    put(&mut game, player, "Exalted Sunborn", Zone::Battlefield);
    let eject = put(&mut game, player, "Emergency Eject", Zone::Hand);
    let victim = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let first_bow = put(&mut game, player, "Short Bow", Zone::Hand);
    let second_bow = put(&mut game, player, "Short Bow", Zone::Hand);
    cast(&mut game, eject, Some(Target::Card(victim)));
    settle(&mut game);
    assert_eq!(battlefield_tokens(&game, player, Card(5238)).len(), 2);
    assert_eq!(battlefield_tokens(&game, player, Card(5309)).len(), 2);
    cast(&mut game, first_bow, None);
    settle(&mut game);
    assert_eq!(battlefield_tokens(&game, player, Card(5309)).len(), 2);
    advance(&mut game, |game| {
        game.state.turn > 3 && game.state.active == player && game.state.step == Step::Main1
    });
    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 2];
    cast(&mut game, second_bow, None);
    settle(&mut game);
    assert_eq!(battlefield_tokens(&game, player, Card(5309)).len(), 4);
}

#[test]
fn mechan_navigator_draws_and_can_discard_that_card_before_its_station_ability_resolves() {
    let player = Player::P1;
    let mut game = scenario(player);
    let navigator = put(&mut game, player, "Mechan Navigator", Zone::Battlefield);
    let ship = put(&mut game, player, "Warmaker Gunship", Zone::Battlefield);
    let old = put(&mut game, player, "Forest", Zone::Hand);
    let drawn = put(&mut game, player, "Island", Zone::Library);
    station(&mut game, ship, navigator);
    pass_pair(&mut game);
    assert_eq!(game.object_view(drawn).zone, Zone::Hand);
    assert!(
        game.observe(player.other())
            .objects
            .iter()
            .all(|object| object.id != drawn)
    );
    assert_eq!(game.counter_count(ship, CounterKind::Charge), 0);
    for card in [old, drawn] {
        let mut branch = game.clone();
        act(&mut branch, Action::Discard { card });
        settle(&mut branch);
        assert_eq!(branch.object_view(card).zone, Zone::Graveyard);
        assert_eq!(branch.counter_count(ship, CounterKind::Charge), 2);
    }
}

#[test]
fn mechan_shieldmate_ignores_defender_only_after_an_artifact_entered_under_its_controller() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let shieldmate = put(&mut game, player, "Mechan Shieldmate", Zone::Battlefield);
    let foreign = put(&mut game, opponent, "Illvoi Light Jammer", Zone::Hand);
    let own = put(&mut game, player, "Short Bow", Zone::Hand);
    let remove = put(&mut game, player, "Disenchant", Zone::Hand);
    opponent_priority(&mut game, opponent);
    cast(&mut game, foreign, None);
    settle(&mut game);

    let attack = Action::Attack { card: shieldmate };
    let mut foreign_only = game.clone();
    advance(&mut foreign_only, |game| {
        game.state.pending == Pending::Attackers
    });
    reject(&mut foreign_only, attack.clone());

    cast(&mut game, own, None);
    settle(&mut game);
    cast(&mut game, remove, Some(Target::Card(own)));
    settle(&mut game);
    assert_eq!(game.object_view(own).zone, Zone::Graveyard);
    advance(&mut game, |game| game.state.pending == Pending::Attackers);
    assert!(
        game.object_view(shieldmate)
            .keywords
            .contains(Keywords::DEFENDER)
    );
    act(&mut game, attack);
    act(&mut game, Action::FinishAttackers);
    assert!(game.object_view(shieldmate).attacking);
}

#[test]
fn mechanozoa_stuns_either_opposing_type_and_recasts_from_its_warp_exile() {
    let player = Player::P0;
    let mut game = scenario(player);
    let zoa = put(&mut game, player, "Mechanozoa", Zone::Hand);
    let artifact = put(&mut game, player.other(), "Short Bow", Zone::Battlefield);
    let creature = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    let ally = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    game.state.objects[artifact as usize].tapped = true;
    warp(&mut game, zoa);
    pass_pair(&mut game);
    advance(&mut game, |game| {
        game.legal_actions().contains(&choose_target(artifact))
    });
    reject(&mut game, choose_target(ally));

    let mut creature_branch = game.clone();
    trigger_target(&mut creature_branch, creature);
    settle(&mut creature_branch);
    assert!(creature_branch.object_view(creature).tapped);
    assert_eq!(creature_branch.object_view(creature).counters.stun, 1);

    trigger_target(&mut game, artifact);
    settle(&mut game);
    assert!(game.object_view(artifact).tapped);
    assert_eq!(game.object_view(artifact).counters.stun, 1);
    end_step(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(zoa).zone, Zone::Exile);
    advance(&mut game, |game| {
        game.state.active == player && game.state.step == Step::Main1
    });
    game.state.players[player.index()].mana = [0, 2, 0, 0, 0, 4];
    act(&mut game, Action::AnnounceExile { card: zoa });
    pay(&mut game);
    pass_pair(&mut game);
    trigger_target(&mut game, creature);
    settle(&mut game);
    assert_eq!(game.object_view(zoa).zone, Zone::Battlefield);
    assert_eq!(game.object_view(creature).counters.stun, 1);
    assert!(game.observe(player).warped_cards.is_empty());
}
