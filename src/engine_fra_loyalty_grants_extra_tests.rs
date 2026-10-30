//! Landfall and two-target effects carried by controlled planeswalkers.
use super::*;

#[test]
fn avatar_landfall_empowers_and_granted_ability_counts_lands_on_resolution() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let avatar = put(
            &mut game,
            player,
            "Avatar of Burgeoning Echoes",
            Zone::Battlefield,
        );
        let land = put(&mut game, player, "Forest", Zone::Hand);
        act(&mut game, Action::PlayLand { card: land });
        settle(&mut game);
        let jace = game
            .state
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
            .unwrap();
        assert_eq!(game.counter_count(jace, CounterKind::Loyalty), 2);
        let second = put(&mut game, player, "Island", Zone::Battlefield);
        let walker = put(
            &mut game,
            player,
            "Ajani, Caller of the Pride",
            Zone::Battlefield,
        );
        game.state.objects[walker as usize].counters.loyalty = 10;
        let creature = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
        let source = game.object_ref(walker);
        let grantor = game.object_ref(avatar);
        act(
            &mut game,
            Action::AnnouncePlaneswalkerGrant {
                source,
                grantor,
                ability: 0,
            },
        );
        act(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(creature),
            },
        );
        pay(&mut game);
        game.move_object(second, Zone::Graveyard);
        super::super::contracts::branch_contract(game.clone());
        settle(&mut game);
        assert_eq!(game.counter_count(creature, CounterKind::PlusOne), 1);
    }
}

#[test]
fn warlord_grant_can_skip_optional_target_or_resolve_after_its_target_leaves() {
    for player in [Player::P0, Player::P1] {
        for case in 0..3 {
            let mut game = scenario(player);
            let way = put(&mut game, player, "Way of the Warlord", Zone::Hand);
            cast(&mut game, way, None);
            settle(&mut game);
            let walker = game
                .state
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
                .unwrap();
            let target = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
            let source = game.object_ref(walker);
            let grantor = game.object_ref(way);
            act(
                &mut game,
                Action::AnnouncePlaneswalkerGrant {
                    source,
                    grantor,
                    ability: 0,
                },
            );
            if case == 0 {
                act(&mut game, Action::FinishTargets);
            } else {
                act(
                    &mut game,
                    Action::ChooseTarget {
                        target: Target::Card(target),
                    },
                );
            }
            act(
                &mut game,
                Action::ChooseTarget {
                    target: Target::Player(player.other()),
                },
            );
            pay(&mut game);
            if case == 2 {
                game.move_object(target, Zone::Hand);
            }
            game.move_object(way, Zone::Graveyard);
            super::super::contracts::branch_contract(game.clone());
            settle(&mut game);
            assert_eq!(game.state.players[player.other().index()].life, 18);
            assert_eq!(
                game.object_view(target).zone,
                match case {
                    0 => Zone::Battlefield,
                    1 => Zone::Graveyard,
                    _ => Zone::Hand,
                }
            );
        }
    }
}
