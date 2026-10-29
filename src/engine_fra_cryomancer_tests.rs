//! Delayed copied-spell grant after a planeswalker's loyalty ability resolves.
use super::*;

fn arm_copy(game: &mut Game, player: Player) -> CardId {
    let way = put(game, player, "Way of the Cryomancer", Zone::Hand);
    cast(game, way, None);
    settle(game);
    let walker = game.state.objects.iter().enumerate().find_map(|(id, object)| {
        (object.zone == Zone::Battlefield && object.controller == player && object.is_token()
            && game.has_subtype(id as CardId, "Jace")).then_some(id as CardId)
    }).unwrap();
    assert_eq!(game.counter_count(walker, CounterKind::Loyalty), 5);
    let source = game.object_ref(walker);
    let grantor = game.object_ref(way);
    act(game, Action::AnnouncePlaneswalkerGrant { source, grantor, ability: 0 });
    pay(game);
    assert_eq!(game.counter_count(walker, CounterKind::Loyalty), 2);
    settle(game);
    way
}

#[test]
fn cryomancer_copies_only_the_next_instant_or_sorcery_and_can_retarget() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let way = arm_copy(&mut game, player);
        game.move_object(way, Zone::Graveyard);
        let bear = put(&mut game, player, "Bear Cub", Zone::Hand);
        cast(&mut game, bear, None);
        settle(&mut game);
        assert_eq!(game.observe(player).delayed_spell_copies.len(), 1);
        let shock = put(&mut game, player, "Shock", Zone::Hand);
        cast(&mut game, shock, Some(Target::Player(player.other())));
        for _ in 0..32 {
            if matches!(game.state.pending, Pending::Retarget { .. }) { break; }
            choose(&mut game, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
        }
        super::super::contracts::branch_contract(game.clone());
        act(&mut game, Action::SetStackTarget { index: 0, target: Target::Player(player) });
        act(&mut game, Action::FinishStackTargets);
        settle(&mut game);
        assert_eq!(game.state.players[player.index()].life, 18);
        assert_eq!(game.state.players[player.other().index()].life, 18);
        assert!(game.observe(player).delayed_spell_copies.is_empty());
        let second = put(&mut game, player, "Shock", Zone::Hand);
        cast(&mut game, second, Some(Target::Player(player.other())));
        settle(&mut game);
        assert_eq!(game.state.players[player.other().index()].life, 16);
    }
}

#[test]
fn cryomancer_delayed_copy_expires_at_cleanup() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        arm_copy(&mut game, player);
        assert_eq!(game.observe(player).delayed_spell_copies.len(), 1);
        game.finish_cleanup();
        assert!(game.observe(player).delayed_spell_copies.is_empty());
    }
}
