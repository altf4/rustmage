mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn progress(game: &mut Game) {
    choose(game, |a| {
        matches!(a, Action::Pass | Action::PutTrigger { .. })
    });
}

fn until(game: &mut Game, action: Action) {
    for _ in 0..20 {
        if game.legal_actions().contains(&action) {
            return;
        }
        progress(game);
    }
    panic!("missing {action:?} in {:?}", game.state.pending);
}

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

fn activate(game: &mut Game, source: CardId, recipient: Option<CardId>) {
    act(
        game,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    if let Some(recipient) = recipient {
        act(game, target(recipient));
    }
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(game);
    }
    settle(game);
}

#[test]
fn brave_kin_targets_any_creature_at_sorcery_speed_and_warren_elder_pumps_its_team() {
    let player = Player::P0;
    let mut game = scenario(player);
    let duo = put(&mut game, player, "Brave-Kin Duo", Zone::Battlefield);
    let elder = put(&mut game, player, "Warren Elder", Zone::Battlefield);
    let other = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    let mut off_turn = game.clone();
    off_turn.finish_cleanup();
    off_turn.state.players[player.index()].mana = [8; 6];
    if off_turn.actor() != Some(player) {
        act(&mut off_turn, Action::Pass);
    }
    reject(
        &mut off_turn,
        Action::AnnounceActivate {
            card: duo,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    activate(&mut game, duo, Some(other));
    assert!(game.object_view(duo).tapped);
    assert_eq!(stats(&game, other), (3, 3));
    activate(&mut game, elder, None);
    assert_eq!(stats(&game, duo), (2, 2));
    assert_eq!(stats(&game, elder), (3, 3));
    assert_eq!(stats(&game, other), (3, 3));
    game.finish_cleanup();
    assert_eq!(stats(&game, elder), (2, 2));
    assert_eq!(stats(&game, other), (2, 2));
}

#[test]
fn stoat_lifelink_uses_first_strike_and_shrike_uses_both_damage_steps() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let stoat = put(&mut game, player, "Brightblade Stoat", Zone::Battlefield);
    let shrike = put(&mut game, player, "Shrike Force", Zone::Battlefield);
    let ground = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    let first = Action::Attack { card: stoat };
    until(&mut game, first.clone());
    act(&mut game, first);
    act(&mut game, Action::Attack { card: shrike });
    act(&mut game, Action::FinishAttackers);
    assert!(!game.object_view(shrike).tapped);
    let block = Action::Block {
        blocker: ground,
        attacker: stoat,
    };
    until(&mut game, block.clone());
    reject(
        &mut game,
        Action::Block {
            blocker: ground,
            attacker: shrike,
        },
    );
    act(&mut game, block);
    act(&mut game, Action::FinishBlockers);
    for _ in 0..8 {
        if game.state.step == Step::FirstStrikeDamage {
            break;
        }
        act(&mut game, Action::Pass);
    }
    assert_eq!(game.state.step, Step::FirstStrikeDamage);
    assert_eq!(game.object_view(ground).zone, Zone::Graveyard);
    assert_eq!(game.object_view(stoat).damage, 0);
    assert_eq!(game.state.players[player.index()].life, 22);
    assert_eq!(game.state.players[opponent.index()].life, 19);
    for _ in 0..8 {
        if game.state.step == Step::CombatDamage {
            break;
        }
        act(&mut game, Action::Pass);
    }
    assert_eq!(game.state.step, Step::CombatDamage);
    assert_eq!(game.state.players[opponent.index()].life, 18);
}

#[test]
fn feather_can_flash_in_draw_and_temporarily_modify_an_enchanted_creature() {
    let player = Player::P1;
    let mut game = scenario(player.other());
    let bear = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    let feather = put(&mut game, player, "Feather of Flight", Zone::Hand);
    act(&mut game, Action::Pass);
    cast(&mut game, feather, Some(Target::Card(bear)));
    settle(&mut game);
    assert_eq!(stats(&game, bear), (3, 2));
    assert!(game.object_view(bear).keywords.contains(Keywords::FLYING));
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
    game.move_object(feather, Zone::Graveyard);
    assert_eq!(stats(&game, bear), (2, 2));
    assert!(!game.object_view(bear).keywords.contains(Keywords::FLYING));
}

#[test]
fn rabbit_response_pumps_all_controlled_creatures_and_scries_only_with_a_rabbit() {
    let player = Player::P0;
    let mut game = scenario(player);
    let rabbit = put(&mut game, player, "Brave-Kin Duo", Zone::Battlefield);
    let own = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let enemy = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    let response = put(&mut game, player, "Rabbit Response", Zone::Hand);
    cast(&mut game, response, None);
    let mut no_rabbit = game.clone();
    no_rabbit.move_object(rabbit, Zone::Exile);
    settle(&mut no_rabbit);
    assert_eq!(stats(&no_rabbit, own), (4, 3));
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    for _ in 0..2 {
        choose(&mut game, |a| {
            matches!(
                a,
                Action::ArrangeLibrary {
                    destination: LibraryDestination::Top,
                    ..
                }
            )
        });
    }
    settle(&mut game);
    assert_eq!(stats(&game, rabbit), (3, 2));
    assert_eq!(stats(&game, own), (4, 3));
    assert_eq!(stats(&game, enemy), (2, 2));
}

#[test]
fn repel_calamity_accepts_either_large_stat_and_rechecks_it_at_resolution() {
    let player = Player::P0;
    for (power, toughness) in [(2, 0), (0, 2)] {
        let mut game = scenario(player);
        let enemy = put(
            &mut game,
            player.other(),
            "Grizzly Bears",
            Zone::Battlefield,
        );
        let spell = put(&mut game, player, "Repel Calamity", Zone::Hand);
        reject(&mut game, Action::AnnounceCast { card: spell });
        game.state.objects[enemy as usize].growth = Stats { power, toughness };
        cast(&mut game, spell, Some(Target::Card(enemy)));
        let mut below = game.clone();
        below.state.objects[enemy as usize].growth = Stats::default();
        settle(&mut below);
        assert_eq!(below.object_view(enemy).zone, Zone::Battlefield);
        settle(&mut game);
        assert_eq!(game.object_view(enemy).zone, Zone::Graveyard);
    }
}

#[test]
fn serra_redeemer_counts_small_entries_and_channeler_transfers_all_its_counters() {
    let player = Player::P0;
    let mut game = scenario(player);
    let redeemer = put(&mut game, player, "Serra Redeemer", Zone::Hand);
    let channeler = put(&mut game, player, "Essence Channeler", Zone::Hand);
    let giant = put(&mut game, player, "Hill Giant", Zone::Hand);
    cast(&mut game, redeemer, None);
    settle(&mut game);
    assert_eq!(game.counter_count(redeemer, CounterKind::PlusOne), 0);
    assert!(
        game.object_view(redeemer)
            .keywords
            .contains(Keywords::FLYING)
    );
    cast(&mut game, giant, None);
    settle(&mut game);
    assert_eq!(game.counter_count(giant, CounterKind::PlusOne), 0);
    cast(&mut game, channeler, None);
    settle(&mut game);
    assert_eq!(game.counter_count(channeler, CounterKind::PlusOne), 2);
    game.lose_life(player, 1);
    game.gain_life(player, 1);
    game.prepare_triggers();
    settle(&mut game);
    assert_eq!(game.counter_count(channeler, CounterKind::PlusOne), 3);
    assert!(
        game.object_view(channeler)
            .keywords
            .contains(Keywords::FLYING.union(Keywords::VIGILANCE))
    );
    game.add_counter(channeler, CounterKind::Stun, 1);
    let spell = put(&mut game, player, "Murder", Zone::Hand);
    cast(&mut game, spell, Some(Target::Card(channeler)));
    until(&mut game, target(giant));
    act(&mut game, target(giant));
    settle(&mut game);
    assert_eq!(game.counter_count(giant, CounterKind::PlusOne), 3);
    assert_eq!(game.counter_count(giant, CounterKind::Stun), 1);
}

#[test]
fn thistledown_targets_any_nonland_and_stun_replaces_its_untap() {
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Thistledown Players", Zone::Battlefield);
    let enemy = put(
        &mut game,
        player.other(),
        "Swiftfoot Boots",
        Zone::Battlefield,
    );
    let land = put(&mut game, player, "Forest", Zone::Battlefield);
    game.state.objects[enemy as usize].tapped = true;
    game.add_counter(enemy, CounterKind::Stun, 1);
    let attack = Action::Attack { card: source };
    until(&mut game, attack.clone());
    act(&mut game, attack);
    act(&mut game, Action::FinishAttackers);
    reject(&mut game, target(land));
    act(&mut game, target(enemy));
    settle(&mut game);
    assert!(game.object_view(enemy).tapped);
    assert_eq!(game.counter_count(enemy, CounterKind::Stun), 0);
}

#[test]
fn lifecreed_duo_only_gains_life_for_each_other_controlled_creature() {
    let player = Player::P0;
    let other = player.other();
    let mut game = scenario(player);
    let duo = put(&mut game, player, "Lifecreed Duo", Zone::Hand);
    cast(&mut game, duo, None);
    settle(&mut game);
    assert!(game.object_view(duo).keywords.contains(Keywords::FLYING));
    assert_eq!(game.state.players[player.index()].life, 20);
    let rabbits = put(&mut game, player, "Hop to It", Zone::Hand);
    cast(&mut game, rabbits, None);
    for _ in 0..24 {
        if game.state.stack.is_empty()
            && game.state.pending == Pending::Priority
            && game.state.ordering_triggers.is_empty()
            && game.state.pending_triggers.is_empty()
        {
            break;
        }
        progress(&mut game);
    }
    assert_eq!(game.state.players[player.index()].life, 23);
    game.finish_cleanup();
    game.state.step = Step::Main1;
    game.state.priority = other;
    game.state.players[other.index()].mana = [8; 6];
    let enemy = put(&mut game, other, "Bear Cub", Zone::Hand);
    cast(&mut game, enemy, None);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].life, 23);
}
