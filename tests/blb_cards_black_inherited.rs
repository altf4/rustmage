//! Printed obligations for previously admitted black Bloomburrow cards.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn pick_target(game: &mut Game, target: Target) {
    act(game, Action::ChooseTarget { target });
    for finish in [Action::FinishTargets, Action::FinishStackTargets] {
        if game.legal_actions().contains(&finish) {
            act(game, finish);
        }
    }
}

fn activate(game: &mut Game, source: CardId) {
    choose(
        game,
        |action| matches!(action, Action::Activate { card, .. } | Action::AnnounceActivate { card, .. } if *card == source),
    );
}

fn life(game: &Game, player: Player) -> i128 {
    game.state.players[player.index()].life
}

#[test]
fn bonebind_orator_exiles_itself_to_return_another_creature_card() {
    let player = Player::P0;
    let mut game = scenario(player);
    let orator = put(&mut game, player, "Bonebind Orator", Zone::Graveyard);
    let creature = put(&mut game, player, "Grizzly Bears", Zone::Graveyard);
    let opponent_creature = put(&mut game, player.other(), "Grizzly Bears", Zone::Graveyard);
    act(
        &mut game,
        Action::AnnounceActivate {
            card: orator,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    for bad in [orator, opponent_creature] {
        reject(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(bad),
            },
        );
    }
    pick_target(&mut game, Target::Card(creature));
    pay(&mut game);
    assert_eq!(game.object_view(orator).zone, Zone::Exile);
    let mut response = game.clone();
    response.move_object(creature, Zone::Exile);
    settle(&mut response);
    assert_eq!(response.object_view(orator).zone, Zone::Exile);
    assert_eq!(response.object_view(creature).zone, Zone::Exile);
    settle(&mut game);
    assert_eq!(game.object_view(creature).zone, Zone::Hand);
}

#[test]
fn daggerfang_duo_can_decline_or_mill_and_has_lethal_deathtouch() {
    let player = Player::P1;
    for accept in [false, true] {
        let mut game = scenario(player);
        let duo = put(&mut game, player, "Daggerfang Duo", Zone::Hand);
        cast(&mut game, duo, None);
        pass_pair(&mut game);
        pass_pair(&mut game);
        act(&mut game, Action::ChooseOptional { accept });
        assert_eq!(
            game.state.players[player.index()].library.len(),
            if accept { 6 } else { 8 }
        );
        assert!(
            game.object_view(duo)
                .keywords
                .contains(Keywords::DEATHTOUCH)
        );
    }
}

#[test]
fn diresight_surveils_before_drawing_and_losing_life() {
    let player = Player::P0;
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Diresight", Zone::Hand);
    let under = put(&mut game, player, "Forest", Zone::Library);
    let keep = put(&mut game, player, "Swamp", Zone::Library);
    let mill = put(&mut game, player, "Grizzly Bears", Zone::Library);
    cast(&mut game, spell, None);
    pass_pair(&mut game);
    assert!(game.observe(player).library_choice.is_some());
    assert!(game.observe(player.other()).library_choice.is_none());
    act(
        &mut game,
        Action::ArrangeLibrary {
            card: mill,
            destination: LibraryDestination::Graveyard,
        },
    );
    act(
        &mut game,
        Action::ArrangeLibrary {
            card: keep,
            destination: LibraryDestination::Top,
        },
    );
    assert_eq!(game.object_view(mill).zone, Zone::Graveyard);
    for drawn in [keep, under] {
        assert_eq!(game.object_view(drawn).zone, Zone::Hand);
    }
    assert_eq!(life(&game, player), 18);
}

#[test]
fn downwind_ambusher_can_flash_in_and_both_modes_check_the_targets() {
    let player = Player::P0;
    for mode in [0, 1] {
        let mut game = scenario(player.other());
        game.state.priority = player;
        let ambusher = put(&mut game, player, "Downwind Ambusher", Zone::Hand);
        let clean = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
        let damaged = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
        let shock = put(&mut game, player, "Shock", Zone::Hand);
        cast(&mut game, shock, Some(Target::Card(damaged)));
        settle(&mut game);
        if game.actor() != Some(player) {
            act(&mut game, Action::Pass);
        }
        cast(&mut game, ambusher, None);
        pass_pair(&mut game);
        act(&mut game, Action::ChooseMode { mode });
        if mode == 1 {
            reject(
                &mut game,
                Action::ChooseTarget {
                    target: Target::Card(clean),
                },
            );
        }
        let target = if mode == 0 { clean } else { damaged };
        pick_target(&mut game, Target::Card(target));
        settle(&mut game);
        if mode == 0 {
            assert_eq!(stats(&game, clean), (2, 2));
        } else {
            assert_eq!(game.object_view(damaged).zone, Zone::Graveyard);
        }
    }
}

#[test]
fn fell_respects_indestructible_but_savor_reduces_toughness_and_creates_food() {
    let player = Player::P1;
    let mut game = scenario(player);
    let fell = put(&mut game, player, "Fell", Zone::Hand);
    let savor = put(&mut game, player, "Savor", Zone::Hand);
    let creature = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    game.state.objects[creature as usize].gained_keywords = Keywords::INDESTRUCTIBLE;
    cast(&mut game, fell, Some(Target::Card(creature)));
    settle(&mut game);
    assert_eq!(game.object_view(creature).zone, Zone::Battlefield);
    cast(&mut game, savor, Some(Target::Card(creature)));
    let mut stale = game.clone();
    stale.move_object(creature, Zone::Exile);
    settle(&mut stale);
    assert!(stale.controlled_token_ids(player).next().is_none());
    settle(&mut game);
    assert_eq!(game.object_view(creature).zone, Zone::Graveyard);
    let food = game.controlled_token_ids(player).next().unwrap();
    assert!(game.has_subtype(food, "Food"));
}

#[test]
fn hazels_nocturne_drains_with_no_targets_or_one_surviving_target() {
    let player = Player::P0;
    for chosen in [false, true] {
        let mut game = scenario(player);
        let spell = put(&mut game, player, "Hazel's Nocturne", Zone::Hand);
        let first = put(&mut game, player, "Grizzly Bears", Zone::Graveyard);
        let second = put(&mut game, player, "Hill Giant", Zone::Graveyard);
        if chosen {
            announce(
                &mut game,
                spell,
                &[Target::Card(first), Target::Card(second)],
            );
            game.move_object(first, Zone::Exile);
        } else {
            announce(&mut game, spell, &[]);
        }
        settle(&mut game);
        assert_eq!([life(&game, player), life(&game, player.other())], [22, 18]);
        assert_eq!(
            game.object_view(second).zone,
            if chosen { Zone::Hand } else { Zone::Graveyard }
        );
    }
}

#[test]
fn consumed_by_greed_gifts_a_draw_then_uses_current_greatest_power_and_recovers_a_card() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Consumed by Greed", Zone::Hand);
    let recovery = put(&mut game, player, "Grizzly Bears", Zone::Graveyard);
    let biggest = put(&mut game, opponent, "Hill Giant", Zone::Battlefield);
    let tied = put(&mut game, opponent, "Grizzly Bears", Zone::Battlefield);
    let small = put(&mut game, opponent, "Grizzly Bears", Zone::Battlefield);
    act(&mut game, Action::AnnounceCast { card: spell });
    act(
        &mut game,
        Action::ChooseGift {
            beneficiary: Some(opponent),
        },
    );
    for target in [Target::Player(opponent), Target::Card(recovery)] {
        pick_target(&mut game, target);
    }
    pay(&mut game);
    game.add_counter(tied, CounterKind::PlusOne, 1);
    pass_pair(&mut game);
    assert_eq!(
        game.state.players[opponent.index()].cards_drawn_this_turn,
        1
    );
    let object = game.object_ref(small);
    reject(&mut game, Action::Sacrifice { object });
    let object = game.object_ref(tied);
    act(&mut game, Action::Sacrifice { object });
    assert_eq!(game.object_view(tied).zone, Zone::Graveyard);
    assert_eq!(game.object_view(biggest).zone, Zone::Battlefield);
    assert_eq!(game.object_view(recovery).zone, Zone::Hand);
}

#[test]
fn feed_the_cycle_accepts_food_graveyard_exile_or_the_additional_black_mana() {
    let player = Player::P0;
    for route in [
        ForageRoute::Black,
        ForageRoute::Food,
        ForageRoute::ExileThree,
    ] {
        let mut game = scenario(player);
        let feed = put(&mut game, player, "Feed the Cycle", Zone::Hand);
        let victim = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
        let selected = match route {
            ForageRoute::Food => vec![put(&mut game, player, "Gingerbrute", Zone::Battlefield)],
            ForageRoute::ExileThree => (0..3)
                .map(|_| put(&mut game, player, "Forest", Zone::Graveyard))
                .collect(),
            ForageRoute::Black => vec![],
        };
        act(&mut game, Action::AnnounceCast { card: feed });
        if matches!(game.state.pending, Pending::Targets { .. }) {
            pick_target(&mut game, Target::Card(victim));
        }
        act(&mut game, Action::ChooseForageRoute { route });
        if matches!(game.state.pending, Pending::Targets { .. }) {
            pick_target(&mut game, Target::Card(victim));
        }
        for card in selected {
            let object = game.object_ref(card);
            act(&mut game, Action::ChooseForage { object });
        }
        pay(&mut game);
        settle(&mut game);
        assert_eq!(game.object_view(victim).zone, Zone::Graveyard);
    }
}

#[test]
fn maha_sets_only_enemy_base_toughness_and_its_ward_can_be_paid_or_declined() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(opponent);
    let maha = put(
        &mut game,
        player,
        "Maha, Its Feathers Night",
        Zone::Battlefield,
    );
    let enemy = put(&mut game, opponent, "Hill Giant", Zone::Battlefield);
    let friendly = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    game.add_counter(enemy, CounterKind::PlusOne, 1);
    assert_eq!(stats(&game, enemy), (4, 2));
    assert_eq!(stats(&game, friendly), (3, 3));
    assert!(
        game.object_view(maha)
            .keywords
            .contains(Keywords::FLYING.union(Keywords::TRAMPLE))
    );
    let fell = put(&mut game, opponent, "Fell", Zone::Hand);
    let discard = put(&mut game, opponent, "Forest", Zone::Hand);
    cast(&mut game, fell, Some(Target::Card(maha)));
    pass_pair(&mut game);
    let mut decline = game.clone();
    act(&mut decline, Action::DeclineEffectPayment);
    settle(&mut decline);
    assert_eq!(decline.object_view(maha).zone, Zone::Battlefield);
    act(&mut game, Action::Discard { card: discard });
    settle(&mut game);
    assert_eq!(game.object_view(discard).zone, Zone::Graveyard);
    assert_eq!(game.object_view(maha).zone, Zone::Graveyard);
    assert_eq!(stats(&game, enemy), (4, 4));
}

#[test]
fn ravine_raider_pumps_twice_and_one_creature_cannot_block_it() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let raider = put(&mut game, player, "Ravine Raider", Zone::Battlefield);
    let lone = put(&mut game, opponent, "Hill Giant", Zone::Battlefield);
    for _ in 0..2 {
        activate(&mut game, raider);
        if matches!(game.state.pending, Pending::PayCosts { .. }) {
            pay(&mut game);
        }
        settle(&mut game);
    }
    assert_eq!(stats(&game, raider), (3, 3));
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    act(&mut game, Action::Attack { card: raider });
    act(&mut game, Action::FinishAttackers);
    pass_pair(&mut game);
    assert!(matches!(game.state.pending, Pending::Blockers));
    let block = Action::Block {
        blocker: lone,
        attacker: raider,
    };
    if game.legal_actions().contains(&block) {
        act(&mut game, block);
        reject(&mut game, Action::FinishBlockers);
    } else {
        reject(&mut game, block);
    }
}

#[test]
fn scales_of_shale_counts_lizards_and_grants_lifelink_and_indestructible_to_another_type() {
    let player = Player::P0;
    let mut game = scenario(player);
    let scales = put(&mut game, player, "Scales of Shale", Zone::Hand);
    let bear = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    for _ in 0..2 {
        put(&mut game, player, "Ravine Raider", Zone::Battlefield);
    }
    game.state.players[player.index()].mana = [0, 0, 1, 0, 0, 0];
    cast(&mut game, scales, Some(Target::Card(bear)));
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    assert_eq!(stats(&game, bear), (4, 2));
    assert!(
        game.object_view(bear)
            .keywords
            .contains(Keywords::LIFELINK.union(Keywords::INDESTRUCTIBLE))
    );
    game.finish_cleanup();
    assert_eq!(stats(&game, bear), (2, 2));
}

#[test]
fn sinister_monolith_drains_before_attackers_and_can_be_sacrificed_to_draw_two() {
    let player = Player::P1;
    let mut game = scenario(player);
    let monolith = put(&mut game, player, "Sinister Monolith", Zone::Battlefield);
    game.begin_step(Step::BeginCombat);
    game.prepare_triggers();
    settle(&mut game);
    assert_eq!([life(&game, player), life(&game, player.other())], [21, 19]);
    assert!(!game.legal_actions().iter().any(|action| matches!(action, Action::Activate { card, .. } | Action::AnnounceActivate { card, .. } if *card == monolith)));
    game.state.step = Step::Main2;
    activate(&mut game, monolith);
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(&mut game);
    }
    assert_eq!(life(&game, player), 19);
    assert_eq!(game.object_view(monolith).zone, Zone::Graveyard);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 2);
}

#[test]
fn nocturnal_hunger_loses_two_life_only_without_a_promised_food() {
    let player = Player::P1;
    let opponent = player.other();
    for gift in [false, true] {
        let mut game = scenario(player);
        let hunger = put(&mut game, player, "Nocturnal Hunger", Zone::Hand);
        let victim = put(&mut game, opponent, "Hill Giant", Zone::Battlefield);
        act(&mut game, Action::AnnounceCast { card: hunger });
        act(
            &mut game,
            Action::ChooseGift {
                beneficiary: gift.then_some(opponent),
            },
        );
        pick_target(&mut game, Target::Card(victim));
        pay(&mut game);
        settle(&mut game);
        assert_eq!(life(&game, player), if gift { 20 } else { 18 });
        assert_eq!(game.object_view(victim).zone, Zone::Graveyard);
        assert_eq!(
            game.controlled_token_ids(opponent)
                .filter(|&object| game.has_subtype(object, "Food"))
                .count(),
            usize::from(gift)
        );
    }
}

#[test]
fn huskburster_swarm_reduces_generic_cost_for_its_owners_creature_cards_only() {
    let player = Player::P0;
    let mut game = scenario(player);
    let swarm = put(&mut game, player, "Huskburster Swarm", Zone::Hand);
    for _ in 0..3 {
        put(&mut game, player, "Grizzly Bears", Zone::Graveyard);
        put(&mut game, player, "Hill Giant", Zone::Exile);
    }
    put(&mut game, player.other(), "Grizzly Bears", Zone::Graveyard);
    put(&mut game, player, "Forest", Zone::Exile);
    game.state.players[player.index()].mana = [0, 0, 1, 0, 0, 0];
    reject(&mut game, Action::AnnounceCast { card: swarm });
    put(&mut game, player, "Grizzly Bears", Zone::Graveyard);
    cast(&mut game, swarm, None);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    assert!(
        game.object_view(swarm)
            .keywords
            .contains(Keywords::MENACE.union(Keywords::DEATHTOUCH))
    );
}
