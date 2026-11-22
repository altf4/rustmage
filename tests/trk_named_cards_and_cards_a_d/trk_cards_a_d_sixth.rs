//! More Star Trek A–D typed card scenarios.
use crate::support::*;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};

fn owned(game: &mut Game, player: Player, name: &str, zone: Zone) -> CardId {
    spawn(game, player, Card::from_name(name).unwrap(), zone)
}
fn drain(game: &mut Game) {
    for _ in 0..50 {
        if game.state.pending == Pending::Priority
            && game.state.stack.is_empty()
            && game.state.pending_triggers.is_empty()
            && game.state.resolving.is_none()
        {
            return;
        }
        choose(game, |a| {
            matches!(a, Action::Pass | Action::PutTrigger { .. })
        });
    }
    panic!("resolution budget");
}
fn mode_after_passes(game: &mut Game, mode: u16) {
    for _ in 0..32 {
        if game.legal_actions().contains(&Action::ChooseMode { mode }) {
            act(game, Action::ChooseMode { mode });
            return;
        }
        choose(game, |a| {
            matches!(a, Action::Pass | Action::PutTrigger { .. })
        });
    }
    panic!("modal choice unavailable");
}

#[test]
fn supervisor_counter_and_live_trample_apply_only_to_own_countered_creatures() {
    let mut game = scenario(Player::P0);
    let ally = put(&mut game, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let plain = put(&mut game, Player::P0, "Bear Cub", Zone::Battlefield);
    let enemy = put(&mut game, Player::P1, "Hill Giant", Zone::Battlefield);
    let source = owned(&mut game, Player::P0, "Dominion Supervisor", Zone::Hand);
    cast(&mut game, source, None);
    for _ in 0..32 {
        if game.legal_actions().contains(&Action::ChooseTarget {
            target: Target::Card(ally),
        }) {
            break;
        }
        choose(&mut game, |a| {
            matches!(a, Action::Pass | Action::PutTrigger { .. })
        });
    }
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(ally),
        },
    );
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(&mut game, Action::FinishTargets);
    }
    drain(&mut game);
    assert_eq!(game.counter_count(ally, CounterKind::PlusOne), 1);
    assert!(game.keywords(ally).contains(Keywords::TRAMPLE));
    assert!(!game.keywords(plain).contains(Keywords::TRAMPLE));
    assert!(!game.keywords(enemy).contains(Keywords::TRAMPLE));
}

#[test]
fn culber_and_stamets_counter_or_two_same_color_mana() {
    for mode in [0, 1] {
        let mut game = scenario(Player::P0);
        let ally = put(&mut game, Player::P0, "Grizzly Bears", Zone::Battlefield);
        let source = owned(
            &mut game,
            Player::P0,
            "Culber and Stamets, Star-Crossed",
            Zone::Hand,
        );
        cast(&mut game, source, None);
        mode_after_passes(&mut game, mode);
        if mode == 0 {
            act(
                &mut game,
                Action::ChooseTarget {
                    target: Target::Card(ally),
                },
            );
            if game.legal_actions().contains(&Action::FinishTargets) {
                act(&mut game, Action::FinishTargets);
            }
            drain(&mut game);
            assert_eq!(game.counter_count(ally, CounterKind::PlusOne), 1);
        } else {
            for _ in 0..32 {
                if game.legal_actions().contains(&Action::ChooseMana {
                    production: [0, 0, 0, 0, 2, 0],
                }) {
                    break;
                }
                choose(&mut game, |a| {
                    matches!(a, Action::Pass | Action::PutTrigger { .. })
                });
            }
            let before = game.state.players[0].mana[4];
            act(
                &mut game,
                Action::ChooseMana {
                    production: [0, 0, 0, 0, 2, 0],
                },
            );
            drain(&mut game);
            assert_eq!(game.state.players[0].mana[4], before + 2);
        }
    }
}

#[test]
fn dal_landfall_chooses_red_or_green_and_sacrifices_land_to_draw() {
    let mut game = scenario(Player::P0);
    let dal = owned(
        &mut game,
        Player::P0,
        "Dal R'El, Augment Orphan",
        Zone::Battlefield,
    );
    let land = put(&mut game, Player::P0, "Forest", Zone::Hand);
    act(&mut game, Action::PlayLand { card: land });
    for _ in 0..32 {
        if game.legal_actions().contains(&Action::ChooseMana {
            production: [0, 0, 0, 1, 0, 0],
        }) {
            break;
        }
        choose(&mut game, |a| {
            matches!(a, Action::Pass | Action::PutTrigger { .. })
        });
    }
    let before_red = game.state.players[0].mana[3];
    act(
        &mut game,
        Action::ChooseMana {
            production: [0, 0, 0, 1, 0, 0],
        },
    );
    drain(&mut game);
    assert_eq!(game.state.players[0].mana[3], before_red + 1);
    let _draw = put(&mut game, Player::P0, "Mountain", Zone::Library);
    let before_hand = game.hand_count(Player::P0);
    act(
        &mut game,
        Action::AnnounceActivate {
            card: dal,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    let sacrifice = Action::ChooseAdditionalCost {
        object: game.object_ref(land),
    };
    act(&mut game, sacrifice);
    pay(&mut game);
    drain(&mut game);
    assert_eq!(game.object_view(land).zone, Zone::Graveyard);
    assert_eq!(game.hand_count(Player::P0), before_hand + 1);
}

#[test]
fn dghor_modes_shrink_other_creatures_or_make_each_player_sacrifice() {
    for mode in [0, 1] {
        let mut game = scenario(Player::P0);
        let ally = put(&mut game, Player::P0, "Grizzly Bears", Zone::Battlefield);
        let enemy = put(&mut game, Player::P1, "Grizzly Bears", Zone::Battlefield);
        let source = owned(&mut game, Player::P0, "D'Ghor Bloodletter", Zone::Hand);
        cast(&mut game, source, None);
        mode_after_passes(&mut game, mode);
        if mode == 0 {
            drain(&mut game);
            assert_eq!(stats(&game, ally), (1, 1));
            assert_eq!(stats(&game, enemy), (1, 1));
            assert_eq!(stats(&game, source), (2, 2));
        } else {
            for _ in 0..40 {
                if game.state.pending == Pending::Priority
                    && game.state.stack.is_empty()
                    && game.state.pending_triggers.is_empty()
                    && game.state.resolving.is_none()
                {
                    break;
                }
                if matches!(game.state.pending, Pending::EffectChoice { .. }) {
                    let pick = if game.actor() == Some(Player::P0) {
                        ally
                    } else {
                        enemy
                    };
                    let object = game.object_ref(pick);
                    act(&mut game, Action::Sacrifice { object });
                    continue;
                }
                choose(&mut game, |a| {
                    matches!(a, Action::Pass | Action::PutTrigger { .. })
                });
            }
            assert_eq!(game.object_view(ally).zone, Zone::Graveyard);
            assert_eq!(game.object_view(enemy).zone, Zone::Graveyard);
            assert_eq!(game.object_view(source).zone, Zone::Battlefield);
        }
    }
}
