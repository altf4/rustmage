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
