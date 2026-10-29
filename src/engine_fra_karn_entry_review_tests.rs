use super::*;

fn resolve_entries(game: &mut Game) {
    for _ in 0..64 {
        if game.state.pending == Pending::Priority
            && game.state.stack.is_empty()
            && game.state.resolving.is_none()
            && game.state.pending_triggers.is_empty()
        {
            return;
        }
        choose(game, |action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. })
        });
    }
    panic!("entry effects did not finish");
}

#[test]
fn karn_suppresses_artifact_and_creature_entry_triggers_including_its_own_entry() {
    for player in [Player::P0, Player::P1] {
        for with_karn in [false, true] {
            let mut game = scenario(player);
            let arni = put(&mut game, player, "Arni, Humble Scribe", Zone::Battlefield);
            game.state.objects[arni as usize].tapped = true;
            if with_karn {
                let karn = put(&mut game, player, "Karn, Argent Defender", Zone::Hand);
                cast(&mut game, karn, None);
                resolve_entries(&mut game);
                assert!(
                    game.object_view(arni).tapped,
                    "Karn's own entry must not trigger Arni"
                );
            }
            let creature = put(&mut game, player, "Llanowar Elves", Zone::Hand);
            cast(&mut game, creature, None);
            resolve_entries(&mut game);
            assert_eq!(game.object_view(arni).tapped, with_karn);

            let before = game.state.players[player.index()].library.len();
            let artifact = put(&mut game, player, "Futurist Forge", Zone::Hand);
            cast(&mut game, artifact, None);
            resolve_entries(&mut game);
            assert_eq!(game.object_view(artifact).zone, Zone::Battlefield);
            assert_eq!(
                game.state.players[player.index()].library.len(),
                before - usize::from(!with_karn)
            );
        }
    }
}
