//! Authored Duskmourn death, sacrifice, and damage-to-attachment interactions.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn sacrifice(game: &mut Game, card: CardId) {
    let object = game.object_ref(card);
    act(game, Action::Sacrifice { object });
}

fn target_player(game: &mut Game, player: Player) {
    act(
        game,
        Action::ChooseTarget {
            target: Target::Player(player),
        },
    );
}

fn murder(game: &mut Game, player: Player, victim: CardId) {
    let spell = put(game, player, "Murder", Zone::Hand);
    cast(game, spell, Some(Target::Card(victim)));
    pass_pair(game);
}

fn cast_meathook(game: &mut Game, player: Player, x: u32) {
    let spell = put(game, player, "Meathook Massacre II", Zone::Hand);
    act(game, Action::AnnounceCast { card: spell });
    act(game, Action::ChooseX { value: x });
    pay(game);
    pass_pair(game);
}

#[test]
fn come_back_wrong_returns_a_destroyed_card_then_sacrifices_its_new_incarnation() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let victim = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    let spell = put(&mut game, player, "Come Back Wrong", Zone::Hand);
    game.state.players[player.index()].mana = [0, 0, 1, 0, 0, 2];
    let old = game.object_ref(victim);

    let mut exiled = game.clone();
    exiled.add_counter(victim, CounterKind::Finality, 1);
    cast(&mut exiled, spell, Some(Target::Card(victim)));
    settle(&mut exiled);
    assert_eq!(exiled.object_view(victim).zone, Zone::Exile);
    assert!(exiled.observe(player).delayed_sacrifices.is_empty());

    cast(&mut game, spell, Some(Target::Card(victim)));
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    assert_eq!(game.object_view(victim).zone, Zone::Battlefield);
    assert_eq!(game.object_view(victim).owner, opponent);
    assert_eq!(game.object_view(victim).controller, player);
    let returned = game.object_ref(victim);
    assert_ne!(returned, old);
    assert_eq!(
        game.observe(player).delayed_sacrifices[0].tokens,
        [returned]
    );

    let mut blinked = game.clone();
    blinked.move_object(victim, Zone::Exile);
    blinked.move_object(victim, Zone::Battlefield);
    assert_ne!(blinked.object_ref(victim), returned);
    for (branch, zone) in [
        (&mut game, Zone::Graveyard),
        (&mut blinked, Zone::Battlefield),
    ] {
        branch.begin_step(Step::End);
        branch.prepare_triggers();
        settle(branch);
        assert_eq!(branch.object_view(victim).zone, zone);
    }
}

#[test]
fn meathook_massacre_ii_pays_double_x_and_sacrifices_in_one_batch() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let first = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let second = put(&mut game, player, "Savannah Lions", Zone::Battlefield);
    let opposing = put(&mut game, opponent, "Hill Giant", Zone::Battlefield);
    let land = put(&mut game, player, "Forest", Zone::Battlefield);
    game.state.objects[land as usize].tapped = true;
    game.state.players[player.index()].mana = [0, 0, 4, 0, 0, 4];

    cast_meathook(&mut game, player, 2);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    pass_pair(&mut game);
    let land_payment = Action::Sacrifice {
        object: game.object_ref(land),
    };
    reject(&mut game, land_payment);
    sacrifice(&mut game, first);
    sacrifice(&mut game, second);
    assert_eq!(game.actor(), Some(opponent));
    for id in [first, second, opposing] {
        assert_eq!(game.object_view(id).zone, Zone::Battlefield);
    }
    sacrifice(&mut game, opposing);
    for id in [first, second, opposing] {
        assert_eq!(game.object_view(id).zone, Zone::Graveyard);
    }
    assert_eq!(game.object_view(land).zone, Zone::Battlefield);
    assert!(
        matches!(game.state.pending, Pending::OrderTriggers { player: owner } if owner == player)
    );
    assert_eq!(game.state.ordering_triggers.len(), 3);
}

#[test]
fn meathook_massacre_ii_uses_each_death_payer_and_applies_finality() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let yours = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    let theirs = put(&mut game, player, "Savannah Lions", Zone::Battlefield);
    game.change_controller(game.object_ref(yours), player);
    game.change_controller(game.object_ref(theirs), opponent);

    cast_meathook(&mut game, player, 0);
    settle(&mut game);
    assert_eq!(game.object_view(yours).zone, Zone::Battlefield);
    assert_eq!(game.object_view(theirs).zone, Zone::Battlefield);
    game.state.players[player.index()].mana = [8; 6];

    murder(&mut game, player, yours);
    pass_pair(&mut game);
    assert_eq!(game.actor(), Some(player));
    let mut decline = game.clone();
    act(&mut decline, Action::ChooseOptional { accept: false });
    assert_eq!(decline.object_view(yours).zone, Zone::Graveyard);
    assert_eq!(decline.state.players[player.index()].life, 20);
    act(&mut game, Action::ChooseOptional { accept: true });
    assert_eq!(game.state.players[player.index()].life, 17);
    assert_eq!(game.object_view(yours).controller, player);
    assert_eq!(game.counter_count(yours, CounterKind::Finality), 1);

    murder(&mut game, player, theirs);
    pass_pair(&mut game);
    assert_eq!(game.actor(), Some(opponent));
    let mut paid = game.clone();
    act(&mut paid, Action::PayResolutionLife);
    assert_eq!(paid.state.players[opponent.index()].life, 17);
    assert_eq!(paid.object_view(theirs).zone, Zone::Graveyard);
    act(&mut game, Action::DeclineResolutionLife);
    assert_eq!(game.state.players[opponent.index()].life, 20);
    assert_eq!(game.object_view(theirs).controller, player);
    assert_eq!(game.counter_count(theirs, CounterKind::Finality), 1);

    murder(&mut game, player, theirs);
    settle(&mut game);
    assert_eq!(game.object_view(theirs).zone, Zone::Exile);
}

#[test]
fn unstoppable_slasher_halves_after_combat_and_returns_only_without_counters() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let slasher = put(&mut game, player, "Unstoppable Slasher", Zone::Hand);
    cast(&mut game, slasher, None);
    settle(&mut game);
    assert_eq!(stats(&game, slasher), (2, 3));
    assert!(
        game.object_view(slasher)
            .keywords
            .contains(Keywords::DEATHTOUCH)
    );

    game.state.players[opponent.index()].life = 19;
    game.state.objects[slasher as usize].summoning_sick = false;
    game.begin_step(Step::BeginCombat);
    pass_pair(&mut game);
    act(&mut game, Action::Attack { card: slasher });
    act(&mut game, Action::FinishAttackers);
    pass_pair(&mut game);
    act(&mut game, Action::FinishBlockers);
    pass_pair(&mut game);
    assert_eq!(game.state.players[opponent.index()].life, 17);
    settle(&mut game);
    assert_eq!(game.state.players[opponent.index()].life, 8);

    game.state.players[player.index()].mana = [8; 6];
    game.change_controller(game.object_ref(slasher), opponent);
    let original = game.object_ref(slasher);
    let mut counted = game.clone();
    counted.add_counter(slasher, CounterKind::Time, 1);
    for branch in [&mut game, &mut counted] {
        murder(branch, player, slasher);
        settle(branch);
    }
    assert_eq!(counted.object_view(slasher).zone, Zone::Graveyard);
    assert_eq!(game.object_view(slasher).zone, Zone::Battlefield);
    assert_ne!(game.object_ref(slasher), original);
    assert_eq!(game.object_view(slasher).controller, player);
    assert!(game.object_view(slasher).tapped);
    assert_eq!(game.counter_count(slasher, CounterKind::Stun), 2);
}

#[test]
fn sporogenic_infection_excludes_the_enchanted_creature_then_destroys_it_on_damage() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let enchanted = put(&mut game, opponent, "Hill Giant", Zone::Battlefield);
    let fodder = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    let aura = put(&mut game, player, "Sporogenic Infection", Zone::Hand);
    game.state.objects[fodder as usize].gained_keywords = Keywords::HEXPROOF;

    cast(&mut game, aura, Some(Target::Card(enchanted)));
    pass_pair(&mut game);
    assert!(game.legal_actions().contains(&Action::ChooseTarget {
        target: Target::Player(player),
    }));
    target_player(&mut game, opponent);
    pass_pair(&mut game);
    assert_eq!(game.actor(), Some(opponent));
    let excluded = Action::Sacrifice {
        object: game.object_ref(enchanted),
    };
    reject(&mut game, excluded);
    sacrifice(&mut game, fodder);
    assert_eq!(game.object_view(fodder).zone, Zone::Graveyard);
    assert_eq!(
        game.object_view(aura).attached_to,
        Some(game.object_ref(enchanted))
    );

    let damage = put(&mut game, player, "Shock", Zone::Hand);
    cast(&mut game, damage, Some(Target::Card(enchanted)));
    pass_pair(&mut game);
    assert_eq!(game.object_view(enchanted).zone, Zone::Battlefield);
    assert_eq!(game.object_view(enchanted).damage, 2);
    assert!(game.state.stack.last().unwrap().targets.is_empty());

    let removal = put(&mut game, player, "Disenchant", Zone::Hand);
    cast(&mut game, removal, Some(Target::Card(aura)));
    settle(&mut game);
    assert_eq!(game.object_view(aura).zone, Zone::Graveyard);
    assert_eq!(game.object_view(enchanted).zone, Zone::Graveyard);
}
