use super::*;

#[test]
fn essence_burn_exiles_deaths_only_during_its_turn() {
    for player in [Player::P0, Player::P1] {
        // The first target dies from the five damage during resolution.
        let mut game = scenario(player);
        let victim = put(
            &mut game,
            player.other(),
            "Llanowar Elves",
            Zone::Battlefield,
        );
        let burn = put(&mut game, player, "Essence Burn", Zone::Hand);
        cast(&mut game, burn, Some(Target::Card(victim)));
        settle(&mut game);
        assert_eq!(game.object_view(victim).zone, Zone::Exile);

        for expires in [false, true] {
            let mut game = scenario(player);
            let victim = put(
                &mut game,
                player.other(),
                "Ghalta the Unstoppable",
                Zone::Battlefield,
            );
            let burn = put(&mut game, player, "Essence Burn", Zone::Hand);
            cast(&mut game, burn, Some(Target::Card(victim)));
            settle(&mut game);
            assert_eq!(game.object_view(victim).zone, Zone::Battlefield);
            assert_eq!(game.state.objects[victim as usize].damage, 5);
            if expires {
                game.finish_cleanup();
                // Cleanup advances to the next upkeep and empties mana.
                // Pass priority, then replenish the fixture for the public cast.
                assert_eq!(game.actor(), Some(player.other()));
                act(&mut game, Action::Pass);
                game.state.players[player.index()].mana = [8; 6];
            }
            let murder = put(&mut game, player, "Murder", Zone::Hand);
            cast(&mut game, murder, Some(Target::Card(victim)));
            settle(&mut game);
            assert_eq!(
                game.object_view(victim).zone,
                if expires {
                    Zone::Graveyard
                } else {
                    Zone::Exile
                }
            );
        }
    }
}

#[test]
fn lyra_counts_controlled_kindred_angel_permanents() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let lyra = put(
            &mut game,
            player,
            "Lyra, Archangel of Dawn",
            Zone::Battlefield,
        );
        // Firdoch Core is gated; use its printed noncreature kindred and
        // changeling identity solely as a selector fixture.
        let kindred = put(&mut game, player, "Forest", Zone::Battlefield);
        game.state.objects[kindred as usize].card = Card::from_name("Firdoch Core").unwrap();
        assert!(!game.is_creature(kindred));
        assert!(game.has_subtype(kindred, "Angel"));
        game.gain_life(player, 1);
        game.prepare_triggers();
        settle(&mut game);
        assert_eq!(game.counter_count(lyra, CounterKind::PlusOne), 1);
        assert_eq!(game.counter_count(kindred, CounterKind::PlusOne), 1);
    }
}
