//! First-public-destination milling recovery through paid complete Rapid Rescue.
use super::ability_world::CharacteristicWorld;
use super::assurance::big_helpers::{cast, checked, named, settle, setup};
use super::*;

fn reject(g: &mut Game, action: Action) {
    let before = g.snapshot();
    let revision = g.revision();
    assert_eq!(
        g.apply(g.actor().unwrap(), action),
        Err(Error::IllegalAction)
    );
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), revision);
}
fn paid(g: &mut Game, p: Player, name: &str) -> CardId {
    let id = named(g, p, name, Zone::Hand);
    checked(g, Action::AnnounceCast { card: id });
    for _ in 0..12 {
        if g.state.announcing.is_none() {
            break;
        }
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::ChooseSpellCost { .. }
                        | Action::PayCosts { .. }
                        | Action::FinishTargets
                )
            })
            .unwrap();
        checked(g, action);
    }
    assert!(g.state.announcing.is_none());
    settle(g);
    id
}
fn rescue(g: &mut Game, p: Player) -> CardId {
    let id = named(g, p, "Rapid Rescue", Zone::Hand);
    let green = g.state.players[p.index()].mana[4];
    cast(g, id, None, &[]);
    assert_eq!(g.state.players[p.index()].mana[4], green - 1);
    for _ in 0..20 {
        if matches!(g.state.pending, Pending::RecoverMilled { .. }) {
            return id;
        }
        checked(g, Action::Pass);
    }
    panic!("Rapid Rescue did not offer recovery");
}
fn recover(g: &mut Game, object: Option<ObjectRef>) {
    checked(g, Action::RecoverMilled { object });
    settle(g);
}
fn visible(g: &Game, viewer: Player, id: CardId) -> bool {
    g.observe(viewer)
        .objects
        .iter()
        .any(|o| o.reference.card == id)
}
fn clear_library(g: &mut Game, p: Player) {
    let cards = g.state.players[p.index()].library.clone();
    for id in cards {
        g.move_object(id, Zone::Outside);
    }
}

#[test]
fn milled_recovery_paid_rapid_rescue_graveyard_or_rip_exile_and_optional_decline() {
    for p in [Player::P0, Player::P1] {
        for exile in [false, true] {
            for decline in [false, true] {
                let mut g = setup(p);
                if exile {
                    paid(&mut g, p, "Rest in Peace");
                }
                let secret_hand = named(&mut g, p, "Hill Giant", Zone::Hand);
                let untouched = named(&mut g, p, "Aegis Turtle", Zone::Library);
                let land = named(&mut g, p, "Forest", Zone::Library);
                let before_mill = g.object_ref(land);
                let instant = named(&mut g, p, "Giant Growth", Zone::Library);
                let life = g.life(p);
                let count = g.state.players[p.index()].library.len();
                let spell = rescue(&mut g, p);
                assert_eq!(g.life(p), life);
                assert_eq!(g.state.players[p.index()].library.len(), count - 2);
                let zone = if exile { Zone::Exile } else { Zone::Graveyard };
                assert_eq!(g.object_view(land).zone, zone);
                assert_eq!(g.object_view(instant).zone, zone);
                let object = g.object_ref(land);
                assert_ne!(object, before_mill);
                reject(
                    &mut g,
                    Action::RecoverMilled {
                        object: Some(before_mill),
                    },
                );
                let bad = g.object_ref(instant);
                reject(&mut g, Action::RecoverMilled { object: Some(bad) });
                for viewer in [p, p.other()] {
                    let choice = g.observe(viewer).recovery_choice.unwrap();
                    assert_eq!(choice.candidates, vec![object]);
                    assert_eq!(choice.milled.len(), 2);
                    assert!(visible(&g, viewer, land) && visible(&g, viewer, instant));
                    assert!(!visible(&g, viewer, untouched));
                }
                assert!(!visible(&g, p.other(), secret_hand));
                recover(&mut g, if decline { None } else { Some(object) });
                assert_eq!(g.life(p), life + 2);
                assert_eq!(
                    g.object_view(land).zone,
                    if decline { zone } else { Zone::Hand }
                );
                assert_eq!(g.object_view(instant).zone, zone);
                assert_eq!(g.object_view(spell).zone, zone);
                assert!(visible(&g, p.other(), land));
                assert!(!visible(&g, p.other(), secret_hand));
                assert!(!visible(&g, p.other(), untouched));
                reject(
                    &mut g,
                    Action::RecoverMilled {
                        object: Some(object),
                    },
                );
            }
        }
    }
}

#[test]
fn milled_recovery_mixed_first_public_destinations_under_dryad_militant() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, "Dryad Militant");
        let land = named(&mut g, p, "Forest", Zone::Library);
        let instant = named(&mut g, p, "Giant Growth", Zone::Library);
        rescue(&mut g, p);
        assert_eq!(g.object_view(land).zone, Zone::Graveyard);
        assert_eq!(g.object_view(instant).zone, Zone::Exile);
        let object = g.object_ref(land);
        for viewer in [p, p.other()] {
            let choice = g.observe(viewer).recovery_choice.unwrap();
            assert!(choice.milled.contains(&g.object_ref(instant)));
            assert!(choice.milled.contains(&object));
            assert_eq!(choice.candidates, vec![object]);
        }
        recover(&mut g, Some(object));
        assert_eq!(g.object_view(land).zone, Zone::Hand);
        assert_eq!(g.object_view(instant).zone, Zone::Exile);
        assert_eq!(g.life(p), 22);
    }
}

#[test]
fn milled_recovery_short_empty_and_no_permanent_libraries_still_gain_life() {
    for p in [Player::P0, Player::P1] {
        for (cards, permanent) in [(0, false), (1, false), (1, true), (2, false)] {
            let mut g = setup(p);
            clear_library(&mut g, p);
            for _ in 0..cards {
                named(
                    &mut g,
                    p,
                    if permanent { "Forest" } else { "Giant Growth" },
                    Zone::Library,
                );
            }
            let id = named(&mut g, p, "Rapid Rescue", Zone::Hand);
            cast(&mut g, id, None, &[]);
            if permanent {
                for _ in 0..4 {
                    if matches!(g.state.pending, Pending::RecoverMilled { .. }) {
                        break;
                    }
                    checked(&mut g, Action::Pass);
                }
                recover(&mut g, None);
            } else {
                settle(&mut g);
            }
            assert!(g.state.players[p.index()].library.is_empty());
            assert_eq!(g.life(p), 22);
            assert_eq!(g.outcome(), None);
            assert_eq!(g.object_view(id).zone, Zone::Graveyard);
        }
    }
}

#[test]
fn milled_recovery_rejects_foreign_not_milled_and_out_of_range_objects_atomically() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let ordinary = named(&mut g, p, "Forest", Zone::Graveyard);
        let foreign = named(&mut g, p.other(), "Forest", Zone::Graveyard);
        let land = named(&mut g, p, "Forest", Zone::Library);
        named(&mut g, p, "Giant Growth", Zone::Library);
        rescue(&mut g, p);
        for object in [
            g.object_ref(ordinary),
            g.object_ref(foreign),
            ObjectRef {
                card: CardId::MAX,
                generation: 0,
            },
        ] {
            reject(
                &mut g,
                Action::RecoverMilled {
                    object: Some(object),
                },
            );
        }
        // An adversarial mixed-player mill packet isolates the ownership guard,
        // independently of batch-membership checks in ordinary Rapid Rescue.
        let foreign = g.object_ref(foreign);
        g.state.resolving.as_mut().unwrap().milled.push(foreign);
        assert!(!g.can_recover_milled(p, RecoveryFilter::Permanent, foreign));
        reject(
            &mut g,
            Action::RecoverMilled {
                object: Some(foreign),
            },
        );
        let object = g.object_ref(land);
        recover(&mut g, Some(object));
        assert_eq!(g.life(p), 22);
    }
}

#[test]
fn milled_recovery_rejects_moved_hidden_and_reentered_incarnations_without_observation_leaks() {
    for p in [Player::P0, Player::P1] {
        for destination in [Zone::Hand, Zone::Library, Zone::Exile, Zone::Graveyard] {
            let mut g = setup(p);
            let keeper = named(&mut g, p, "Forest", Zone::Library);
            let moved = named(&mut g, p, "Forest", Zone::Library);
            rescue(&mut g, p);
            let original = g.object_ref(moved);
            // Moving out and back also invalidates the exact first destination.
            if destination == Zone::Graveyard {
                g.move_object(moved, Zone::Exile);
            }
            g.move_object(moved, destination);
            let current = g.object_ref(moved);
            assert_ne!(original, current);
            for object in [original, current] {
                reject(
                    &mut g,
                    Action::RecoverMilled {
                        object: Some(object),
                    },
                );
                assert!(!g.can_recover_milled(p, RecoveryFilter::Permanent, object));
            }
            for viewer in [p, p.other()] {
                let choice = g.observe(viewer).recovery_choice.unwrap();
                assert_eq!(choice.milled, vec![g.object_ref(keeper)]);
                assert_eq!(choice.candidates, vec![g.object_ref(keeper)]);
            }
            // A previously public card may remain known after entering hand.
            // The recovery continuation still excludes both hidden-zone moves.
            if destination == Zone::Library {
                assert!(!visible(&g, p.other(), moved));
            }
            let object = g.object_ref(keeper);
            recover(&mut g, Some(object));
            assert_eq!(g.life(p), 22);
        }
    }
}
