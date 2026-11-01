//! Focused printed outcomes; shared engine invariants live in core tests.
//! Artifact continuations after public choices and source departure.
use super::*;

fn until(game: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..60 {
        if ready(game) {
            return;
        }
        choose(game, |a| {
            matches!(a, Action::Pass | Action::PutTrigger { .. })
        });
    }
    panic!(
        "missing conditional-artifact decision: {:?}",
        game.state.pending
    );
}

#[test]
fn eye_checks_graveyard_after_surveil_and_uses_its_queued_source() {
    {
        let player = Player::P0;
        for (initial, into_graveyard, depart, fires) in [
            (6, false, false, false),
            (6, true, false, true),
            (5, true, true, true),
        ] {
            let mut game = scenario(player);
            let eye = put(&mut game, player, "Eye of Jace", Zone::Battlefield);
            for _ in 0..initial {
                put(&mut game, player, "Forest", Zone::Graveyard);
            }
            let top = put(&mut game, player, "Island", Zone::Library);
            game.begin_step(Step::Upkeep);
            game.prepare_triggers();
            if depart {
                game.move_object(eye, Zone::Graveyard);
            }
            until(&mut game, |g| {
                matches!(g.state.pending, Pending::EffectLibrary { .. })
            });

            act(
                &mut game,
                Action::ArrangeLibrary {
                    card: top,
                    destination: if into_graveyard {
                        LibraryDestination::Graveyard
                    } else {
                        LibraryDestination::Top
                    },
                },
            );
            settle(&mut game);
            assert_eq!(
                game.state.players[player.index()].life,
                if fires { 22 } else { 20 }
            );
            assert_eq!(
                game.state.players[player.other().index()].life,
                if fires { 18 } else { 20 }
            );
            assert_eq!(
                game.object_view(eye).zone,
                if fires || depart {
                    Zone::Graveyard
                } else {
                    Zone::Battlefield
                }
            );
        }
    }
}

#[test]
fn relic_equip_bonus_is_live_and_goes_away_with_equipment() {
    {
        let player = Player::P0;
        let mut game = scenario(player);
        let relic = put(&mut game, player, "Lich's Relic", Zone::Battlefield);
        let creature = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        act(
            &mut game,
            Action::AnnounceActivate {
                card: relic,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        act(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(creature),
            },
        );
        pay(&mut game);
        settle(&mut game);
        assert_eq!(stats(&game, creature), (4, 3));
        game.move_object(relic, Zone::Graveyard);
        assert_eq!(stats(&game, creature), (2, 2));
    }
}
