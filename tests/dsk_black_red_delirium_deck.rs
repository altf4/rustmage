//! Authored haste, blocking, combat-phase, and private delirium choices.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn attack(game: &mut Game, card: CardId) {
    act(game, Action::Attack { card });
    act(game, Action::FinishAttackers);
}

fn begin_combat(game: &mut Game) {
    game.begin_step(Step::BeginCombat);
    game.prepare_triggers();
    pass_pair(game);
}

fn graveyard_types(game: &mut Game, player: Player, delirium: bool) -> CardId {
    for name in ["Forest", "Jade-Cast Sentinel"] {
        put(game, player, name, Zone::Graveyard);
    }
    put(
        game,
        player,
        "Pacifism",
        if delirium {
            Zone::Graveyard
        } else {
            Zone::Exile
        },
    )
}

#[test]
fn fear_of_being_hunted_attacks_on_entry_and_must_be_blocked_if_possible() {
    let player = Player::P0;
    let mut game = scenario(player);
    let attacker = put(&mut game, player, "Fear of Being Hunted", Zone::Hand);
    let blocker = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    cast(&mut game, attacker, None);
    settle(&mut game);
    assert!(
        game.object_view(attacker)
            .keywords
            .contains(Keywords::HASTE)
    );
    assert!(game.state.objects[attacker as usize].summoning_sick);
    begin_combat(&mut game);
    attack(&mut game, attacker);
    let mut no_blocker = game.clone();
    no_blocker.state.objects[blocker as usize].tapped = true;
    pass_pair(&mut no_blocker);
    act(&mut no_blocker, Action::FinishBlockers);

    pass_pair(&mut game);
    reject(&mut game, Action::FinishBlockers);
    act(&mut game, Action::Block { blocker, attacker });
    act(&mut game, Action::FinishBlockers);
}

#[test]
fn fear_of_missing_out_discards_before_drawing_and_draws_from_an_empty_hand() {
    let player = Player::P1;
    for with_hand in [false, true] {
        let mut game = scenario(player);
        let source = put(&mut game, player, "Fear of Missing Out", Zone::Hand);
        let next = put(&mut game, player, "Mountain", Zone::Library);
        let discard = with_hand.then(|| put(&mut game, player, "Forest", Zone::Hand));
        cast(&mut game, source, None);
        pass_pair(&mut game);
        if let Some(card) = discard {
            assert_eq!(game.object_view(next).zone, Zone::Library);
            act(&mut game, Action::Discard { card });
            assert_eq!(game.object_view(card).zone, Zone::Graveyard);
        }
        settle(&mut game);
        assert_eq!(game.object_view(next).zone, Zone::Hand);
        assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
    }
}

#[test]
fn fear_of_missing_out_adds_one_combat_if_its_target_and_delirium_stay_legal() {
    let player = Player::P0;
    let mut game = scenario(player);
    let enchantment = graveyard_types(&mut game, player, true);
    let attacker = put(&mut game, player, "Fear of Missing Out", Zone::Battlefield);
    begin_combat(&mut game);
    attack(&mut game, attacker);
    assert!(game.object_view(attacker).tapped);
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(attacker),
        },
    );
    let mut lost_types = game.clone();
    lost_types.move_object(enchantment, Zone::Exile);
    settle(&mut lost_types);
    assert!(lost_types.object_view(attacker).tapped);
    assert_eq!(lost_types.observe(player).additional_combats, 0);
    let mut lost_target = game.clone();
    lost_target.move_object(attacker, Zone::Exile);
    settle(&mut lost_target);
    assert_eq!(lost_target.observe(player).additional_combats, 0);

    settle(&mut game);
    assert!(!game.object_view(attacker).tapped);
    assert_eq!(game.observe(player).additional_combats, 1);
    pass_pair(&mut game);
    act(&mut game, Action::FinishBlockers);
    for _ in 0..3 {
        pass_pair(&mut game);
    }
    assert_eq!(game.state.step, Step::BeginCombat);
    pass_pair(&mut game);
    attack(&mut game, attacker);
    assert_eq!(game.state.pending, Pending::Priority);
    assert!(game.state.stack.is_empty());
    assert_eq!(game.observe(player).additional_combats, 0);
}

#[test]
fn hand_that_feeds_checks_delirium_only_when_it_attacks() {
    let player = Player::P1;
    for delirium in [false, true] {
        let mut game = scenario(player);
        let enchantment = graveyard_types(&mut game, player, delirium);
        let attacker = put(&mut game, player, "Hand That Feeds", Zone::Battlefield);
        begin_combat(&mut game);
        attack(&mut game, attacker);
        assert_eq!(game.state.stack.len(), usize::from(delirium));
        game.move_object(
            enchantment,
            if delirium {
                Zone::Exile
            } else {
                Zone::Graveyard
            },
        );
        settle(&mut game);
        assert_eq!(stats(&game, attacker), (if delirium { 4 } else { 2 }, 2));
        assert_eq!(
            game.object_view(attacker)
                .keywords
                .contains(Keywords::MENACE),
            delirium
        );
    }
}

#[test]
fn osseous_sticktwister_takes_either_payment_or_deals_lifelink_damage() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let enchantment = graveyard_types(&mut game, player, true);
    let source = put(&mut game, player, "Osseous Sticktwister", Zone::Battlefield);
    game.add_counter(source, CounterKind::PlusOne, 1);
    let permanent = put(&mut game, opponent, "Candy Trail", Zone::Battlefield);
    let land = put(&mut game, opponent, "Forest", Zone::Battlefield);
    let discard = put(&mut game, opponent, "Island", Zone::Hand);
    game.begin_step(Step::End);
    game.prepare_triggers();
    let mut fell_below = game.clone();
    fell_below.move_object(enchantment, Zone::Exile);
    settle(&mut fell_below);
    assert_eq!(fell_below.state.players[opponent.index()].life, 20);

    pass_pair(&mut game);
    assert_eq!(game.actor(), Some(opponent));
    let invalid = Action::Sacrifice {
        object: game.object_ref(land),
    };
    reject(&mut game, invalid);
    let sacrifice = Action::Sacrifice {
        object: game.object_ref(permanent),
    };
    for (action, paid) in [
        (sacrifice, permanent),
        (Action::Discard { card: discard }, discard),
    ] {
        let mut branch = game.clone();
        act(&mut branch, action);
        settle(&mut branch);
        assert_eq!(branch.object_view(paid).zone, Zone::Graveyard);
        assert_eq!(branch.state.players[player.index()].life, 20);
        assert_eq!(branch.state.players[opponent.index()].life, 20);
    }
    act(&mut game, Action::DeclineEffectPayment);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].life, 23);
    assert_eq!(game.state.players[opponent.index()].life, 17);
}
