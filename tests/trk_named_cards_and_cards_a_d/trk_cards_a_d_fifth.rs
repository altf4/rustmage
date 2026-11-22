//! Star Trek A–D modal, graveyard and hand activation scenarios.
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
    panic!("modal choice not offered");
}

#[test]
fn dominion_assault_force_exiles_either_graveyard_or_surveils_privately() {
    for mode in [0, 1] {
        let mut game = scenario(Player::P0);
        let card = owned(&mut game, Player::P0, "Dominion Assault Force", Zone::Hand);
        let grave = put(&mut game, Player::P1, "Grizzly Bears", Zone::Graveyard);
        let top = put(&mut game, Player::P0, "Forest", Zone::Library);
        cast(&mut game, card, None);
        mode_after_passes(&mut game, mode);
        if mode == 0 {
            act(
                &mut game,
                Action::ChooseTarget {
                    target: Target::Card(grave),
                },
            );
            if game.legal_actions().contains(&Action::FinishTargets) {
                act(&mut game, Action::FinishTargets);
            }
            drain(&mut game);
            assert_eq!(game.object_view(grave).zone, Zone::Exile);
            assert_eq!(game.object_view(top).zone, Zone::Library);
        } else {
            for _ in 0..32 {
                if matches!(game.state.pending, Pending::EffectLibrary { .. }) {
                    break;
                }
                choose(&mut game, |a| {
                    matches!(a, Action::Pass | Action::PutTrigger { .. })
                });
            }
            assert!(game.observe(Player::P0).library_choice.is_some());
            assert!(game.observe(Player::P1).library_choice.is_none());
            act(
                &mut game,
                Action::ArrangeLibrary {
                    card: top,
                    destination: LibraryDestination::Graveyard,
                },
            );
            drain(&mut game);
            assert_eq!(game.object_view(top).zone, Zone::Graveyard);
        }
        assert!(
            game.object_view(card)
                .keywords
                .contains(Keywords::VIGILANCE)
        );
    }
}

#[test]
fn cybernetic_specialist_surveils_then_returns_own_grave_card_to_library_bottom() {
    let mut game = scenario(Player::P0);
    let top = put(&mut game, Player::P0, "Forest", Zone::Library);
    let specialist = owned(&mut game, Player::P0, "Cybernetic Specialist", Zone::Hand);
    cast(&mut game, specialist, None);
    for _ in 0..32 {
        if matches!(game.state.pending, Pending::EffectLibrary { .. }) {
            break;
        }
        choose(&mut game, |a| {
            matches!(a, Action::Pass | Action::PutTrigger { .. })
        });
    }
    act(
        &mut game,
        Action::ArrangeLibrary {
            card: top,
            destination: LibraryDestination::Graveyard,
        },
    );
    drain(&mut game);
    assert_eq!(game.object_view(top).zone, Zone::Graveyard);
    game.state.objects[specialist as usize].summoning_sick = false;
    let other = put(&mut game, Player::P0, "Mountain", Zone::Library);
    act(
        &mut game,
        Action::AnnounceActivate {
            card: specialist,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(top),
        },
    );
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(&mut game, Action::FinishTargets);
    }
    pay(&mut game);
    drain(&mut game);
    assert!(game.object_view(specialist).tapped);
    assert_eq!(game.state.players[0].library.first(), Some(&top));
    assert_eq!(game.state.players[0].library.last(), Some(&other));
}

#[test]
fn armus_hand_ability_discards_as_cost_and_shrinks_target() {
    let mut game = scenario(Player::P0);
    let victim = put(&mut game, Player::P1, "Hill Giant", Zone::Battlefield);
    let armus = owned(&mut game, Player::P0, "Armus, Skin of Evil", Zone::Hand);
    act(
        &mut game,
        Action::AnnounceActivate {
            card: armus,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(victim),
        },
    );
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(&mut game, Action::FinishTargets);
    }
    pay(&mut game);
    assert_eq!(game.object_view(armus).zone, Zone::Graveyard);
    drain(&mut game);
    assert_eq!(game.object_view(victim).zone, Zone::Graveyard);
}

#[test]
fn armus_own_end_step_makes_target_opponent_discard_two() {
    let mut game = scenario(Player::P0);
    let armus = owned(
        &mut game,
        Player::P0,
        "Armus, Skin of Evil",
        Zone::Battlefield,
    );
    let one = put(&mut game, Player::P1, "Forest", Zone::Hand);
    let two = put(&mut game, Player::P1, "Mountain", Zone::Hand);
    game.begin_step(Step::End);
    game.give_priority();
    for _ in 0..40 {
        if game.state.pending == Pending::Priority
            && game.state.stack.is_empty()
            && game.state.pending_triggers.is_empty()
            && game.state.resolving.is_none()
        {
            break;
        }
        let legal = game.legal_actions();
        let target = Action::ChooseTarget {
            target: Target::Player(Player::P1),
        };
        if legal.contains(&target) {
            act(&mut game, target);
            continue;
        }
        if legal.contains(&Action::FinishTargets) {
            act(&mut game, Action::FinishTargets);
            continue;
        }
        if matches!(game.state.pending, Pending::EffectDiscard { .. }) {
            choose(&mut game, |a| matches!(a, Action::Discard { .. }));
            continue;
        }
        choose(&mut game, |a| {
            matches!(a, Action::Pass | Action::PutTrigger { .. })
        });
    }
    assert_eq!(game.object_view(one).zone, Zone::Graveyard);
    assert_eq!(game.object_view(two).zone, Zone::Graveyard);
    assert!(game.object_view(armus).keywords.contains(Keywords::MENACE));
}
