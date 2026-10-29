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
    for player in [Player::P0, Player::P1] {
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
            super::super::contracts::branch_contract(game.clone());
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
fn relic_payment_precedes_reflexive_target_and_survives_source_departure() {
    for player in [Player::P0, Player::P1] {
        for paid in [false, true] {
            let mut game = scenario(player);
            let target = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
            let own = put(&mut game, player, "Bear Cub", Zone::Battlefield);
            let relic = put(&mut game, player, "Lich's Relic", Zone::Hand);
            cast(&mut game, relic, None);
            until(&mut game, |g| {
                matches!(g.state.pending, Pending::ResolutionMana { .. })
            });
            assert!(game.state.stack.iter().all(|item| !matches!(
                item.kind,
                StackKind::TriggeredAbility { ability: 1 }
            ) || item.source.is_none_or(|source| source.id != relic)));
            game.move_object(relic, Zone::Graveyard);
            if paid {
                choose(&mut game, |a| matches!(a, Action::PayResolutionMana { .. }));
                until(&mut game, |g| {
                    matches!(g.state.pending, Pending::Targets { .. })
                });
                reject(
                    &mut game,
                    Action::ChooseTarget {
                        target: Target::Card(own),
                    },
                );
                act(
                    &mut game,
                    Action::ChooseTarget {
                        target: Target::Card(target),
                    },
                );
                if game.legal_actions().contains(&Action::FinishTargets) {
                    act(&mut game, Action::FinishTargets);
                }
            } else {
                act(&mut game, Action::DeclineResolutionMana);
            }
            settle(&mut game);
            assert_eq!(
                game.object_view(target).zone,
                if paid {
                    Zone::Graveyard
                } else {
                    Zone::Battlefield
                }
            );
            assert_eq!(game.object_view(own).zone, Zone::Battlefield);
        }
    }
}

#[test]
fn relic_equip_bonus_is_live_and_goes_away_with_equipment() {
    for player in [Player::P0, Player::P1] {
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
