//! Granted loyalty effects and their shared 4/4 trampling Beast token.
use super::*;

fn own_jace(game: &Game, player: Player) -> CardId {
    game.state
        .objects
        .iter()
        .enumerate()
        .find_map(|(id, object)| {
            (object.zone == Zone::Battlefield
                && object.controller == player
                && object.is_token()
                && game.has_subtype(id as CardId, "Jace"))
            .then_some(id as CardId)
        })
        .unwrap()
}
fn beasts(game: &Game, player: Player) -> Vec<CardId> {
    game.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(id, object)| {
            (object.zone == Zone::Battlefield
                && object.controller == player
                && object.card == Card::test_index(5239))
            .then_some(id as CardId)
        })
        .collect()
}
fn activate_grant(game: &mut Game, walker: CardId, provider: CardId) {
    let source = game.object_ref(walker);
    let grantor = game.object_ref(provider);
    act(
        game,
        Action::AnnouncePlaneswalkerGrant {
            source,
            grantor,
            ability: 0,
        },
    );
    pay(game);
}

#[test]
fn wildspeaker_empowers_then_grants_a_real_trampling_beast() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let way = put(&mut game, player, "Way of the Wildspeaker", Zone::Hand);
        cast(&mut game, way, None);
        settle(&mut game);
        let walker = own_jace(&game, player);
        assert_eq!(game.counter_count(walker, CounterKind::Loyalty), 7);
        activate_grant(&mut game, walker, way);
        assert_eq!(game.counter_count(walker, CounterKind::Loyalty), 3);
        game.move_object(way, Zone::Graveyard);
        settle(&mut game);
        let tokens = beasts(&game, player);
        assert_eq!(tokens.len(), 1);
        assert_eq!(stats(&game, tokens[0]), (4, 4));
        assert!(game.keywords(tokens[0]).contains(Keywords::TRAMPLE));
        super::super::contracts::branch_contract(game);
    }
}

#[test]
fn deathbringer_grant_may_sacrifice_a_creature_or_be_declined() {
    for player in [Player::P0, Player::P1] {
        for option in 0..3 {
            let mut game = scenario(player);
            let way = put(&mut game, player, "Way of the Deathbringer", Zone::Hand);
            cast(&mut game, way, None);
            settle(&mut game);
            let walker = own_jace(&game, player);
            assert_eq!(game.counter_count(walker, CounterKind::Loyalty), 5);
            let fodder =
                (option != 0).then(|| put(&mut game, player, "Bear Cub", Zone::Battlefield));
            let enemy = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
            activate_grant(&mut game, walker, way);
            assert_eq!(game.counter_count(walker, CounterKind::Loyalty), 3);
            game.move_object(way, Zone::Graveyard);
            if let Some(fodder) = fodder {
                for _ in 0..32 {
                    if matches!(game.state.pending, Pending::EffectChoice { .. }) {
                        break;
                    }
                    act(&mut game, Action::Pass);
                }
                let bad = game.object_ref(enemy);
                reject(&mut game, Action::Sacrifice { object: bad });
                super::super::contracts::branch_contract(game.clone());
                if option == 1 {
                    act(&mut game, Action::DeclineEffectPayment);
                } else {
                    let object = game.object_ref(fodder);
                    act(&mut game, Action::Sacrifice { object });
                }
            }
            settle(&mut game);
            assert_eq!(beasts(&game, player).len(), usize::from(option == 2));
            if let Some(fodder) = fodder {
                assert_eq!(
                    game.object_view(fodder).zone,
                    if option == 2 {
                        Zone::Graveyard
                    } else {
                        Zone::Battlefield
                    }
                );
            }
            assert_eq!(game.object_view(enemy).zone, Zone::Battlefield);
        }
    }
}
