//! Paid grants and life-bounded casts during an owned end-step resolution.
use super::ability_timeline_tests::{cast, checked, named, settle, setup};
use super::*;
fn reject(g: &mut Game, a: Action) {
    let before = g.snapshot();
    let r = g.revision();
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), r);
}
fn paid(g: &mut Game, p: Player) -> CardId {
    let id = named(g, p, "Bre of Clan Stoutarm", Zone::Hand);
    g.state.players[p.index()].mana = [1, 0, 0, 1, 0, 2];
    cast(g, id, None, &[]);
    settle(g);
    id
}
fn grant(g: &mut Game, p: Player, id: CardId, target: CardId) {
    g.state.players[p.index()].mana = [1, 0, 0, 0, 0, 1];
    let action=g.legal_actions().into_iter().find(|a|matches!(a,Action::Activate{card,target:Some(Target::Card(t)),..}if *card==id&&*t==target)).unwrap();
    checked(g, action);
}
fn window(g: &mut Game) {
    g.begin_step(Step::End);
    g.prepare_triggers();
    for _ in 0..20 {
        if matches!(g.state.pending, Pending::ResolutionCasts { .. }) {
            return;
        }
        let action = if g.state.pending == Pending::Priority {
            Action::Pass
        } else {
            g.legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::PutTrigger { .. }))
                .unwrap()
        };
        checked(g, action);
    }
    panic!("no Bre cast window");
}
fn finish_cast(g: &mut Game) {
    for _ in 0..10 {
        if g.state.announcing.is_none() {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. } | Action::FinishTargets))
            .unwrap();
        checked(g, a);
    }
    panic!("unpaid");
}
#[test]
fn ecl_bre_paid_grant_is_another_own_creature_tap_cost_and_real_flying_lifelink_combat() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = paid(&mut g, p);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.state.players[p.index()].mana = [1, 0, 0, 0, 0, 1];
        assert!(!g.legal_actions().iter().any(
            |a| matches!(a,Action::Activate{card,..}|Action::AnnounceActivate{card,..}if *card==id)
        ));
        g.state.objects[id as usize].summoning_sick = false;
        assert!(!g.legal_actions().iter().any(|a|matches!(a,Action::Activate{card,target:Some(Target::Card(t)),..}if *card==id&&(*t==id||*t==enemy))));
        grant(&mut g, p, id, bear);
        assert!(g.object_view(id).tapped);
        settle(&mut g);
        assert!(
            g.keywords(bear)
                .contains(Keywords::FLYING.union(Keywords::LIFELINK))
        );
        g.state.objects[bear as usize].summoning_sick = false;
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        checked(&mut g, Action::Attack { card: bear });
        checked(&mut g, Action::FinishAttackers);
        settle(&mut g);
        assert!(!g.can_block_pair(enemy, bear));
        for _ in 0..20 {
            if g.state.players[p.other().index()].life == 18 {
                break;
            }
            let a = if g.state.pending == Pending::Priority {
                Action::Pass
            } else {
                g.legal_actions()
                    .into_iter()
                    .find(|a| matches!(a, Action::FinishBlockers))
                    .unwrap()
            };
            checked(&mut g, a);
        }
        assert_eq!(g.state.players[p.other().index()].life, 18);
        assert_eq!(g.state.players[p.index()].life, 22);
        assert_eq!(g.state.players[p.index()].life_gained_this_turn, 2);
        g.finish_cleanup();
        assert!(
            !g.keywords(bear)
                .contains(Keywords::FLYING.union(Keywords::LIFELINK))
        );
    }
}
#[test]
fn ecl_bre_grant_survives_source_departure_but_not_recipient_reentry_and_works_opponent_turn() {
    for p in [Player::P0, Player::P1] {
        for blink in [false, true] {
            let mut g = setup(p);
            let id = paid(&mut g, p);
            let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            g.state.objects[id as usize].summoning_sick = false;
            g.state.active = p.other();
            g.state.priority = p;
            grant(&mut g, p, id, bear);
            g.move_object(id, Zone::Graveyard);
            if blink {
                g.move_object(bear, Zone::Hand);
                g.move_object(bear, Zone::Battlefield);
            }
            settle(&mut g);
            assert_eq!(
                g.keywords(bear)
                    .contains(Keywords::FLYING.union(Keywords::LIFELINK)),
                !blink
            );
        }
    }
}
#[test]
fn ecl_bre_end_trigger_requires_own_turn_life_gain_and_rechecks_at_resolution() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p);
        g.begin_step(Step::End);
        assert!(g.state.pending_triggers.is_empty());
        g.gain_life(p, 2);
        g.state.active = p.other();
        g.begin_step(Step::End);
        assert!(g.state.pending_triggers.is_empty());
        g.state.active = p;
        g.begin_step(Step::End);
        g.prepare_triggers();
        assert_eq!(
            g.state.stack.len(),
            1,
            "sole trigger is placed automatically"
        );
        g.state.players[p.index()].life_gained_this_turn = 0;
        let library = g.state.players[p.index()].library.clone();
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].library, library);
        assert!(g.state.resolution_free_casts.is_none());
    }
}
#[test]
fn ecl_bre_optional_decline_or_too_large_moves_only_nonland_to_publicly_known_hand() {
    for p in [Player::P0, Player::P1] {
        for gain in [1, 4] {
            let mut g = setup(p);
            g.state.players[p.index()].library.clear();
            let id = paid(&mut g, p);
            let bottom = named(&mut g, p, "Forest", Zone::Library);
            let nonland = named(&mut g, p, "Hill Giant", Zone::Library);
            let land = named(&mut g, p, "Mountain", Zone::Library);
            g.gain_life(p, gain);
            window(&mut g);
            assert_eq!(g.state.objects[land as usize].zone, Zone::Exile);
            assert_eq!(g.state.players[p.index()].library.as_slice(), &[bottom]);
            let object = g.object_ref(nonland);
            let action = Action::CastExiledDuringResolution {
                object,
                face: CastFace::Normal,
            };
            assert_eq!(g.legal_actions().contains(&action), gain == 4);
            if gain == 1 {
                reject(&mut g, action);
            }
            g.move_object(id, Zone::Graveyard);
            checked(&mut g, Action::FinishResolutionCasts);
            settle(&mut g);
            assert_eq!(g.state.objects[nonland as usize].zone, Zone::Hand);
            assert!(g.state.known_hand.contains(&g.object_ref(nonland)));
            assert!(
                g.observe(p.other())
                    .objects
                    .iter()
                    .any(|o| o.reference == g.object_ref(nonland))
            );
            assert_eq!(g.state.objects[land as usize].zone, Zone::Exile);
        }
    }
}
#[test]
fn ecl_bre_casts_during_end_resolution_with_free_x_zero_and_spell_face_mana_value() {
    for p in [Player::P0, Player::P1] {
        for adventure in [false, true] {
            let mut g = setup(p);
            g.state.players[p.index()].library.clear();
            paid(&mut g, p);
            let name = if adventure {
                "Bilbo Baggins, Burglar // Take a Glance"
            } else {
                "Exsanguinate"
            };
            let card = named(&mut g, p, name, Zone::Library);
            g.gain_life(p, if adventure { 1 } else { 2 });
            window(&mut g);
            let object = g.object_ref(card);
            if adventure {
                reject(
                    &mut g,
                    Action::CastExiledDuringResolution {
                        object,
                        face: CastFace::Normal,
                    },
                );
            }
            g.state.players[p.index()].mana = [0; 6];
            checked(
                &mut g,
                Action::CastExiledDuringResolution {
                    object,
                    face: if adventure {
                        CastFace::Adventure
                    } else {
                        CastFace::Normal
                    },
                },
            );
            finish_cast(&mut g);
            let item = g.state.stack.last().unwrap();
            assert_eq!(item.cast.unwrap().method, CastMethod::Free);
            assert_eq!(item.cast.unwrap().payment, [0; 6]);
            if !adventure {
                assert_eq!(item.x, Some(0));
            }
            checked(&mut g, Action::FinishResolutionCasts);
            settle(&mut g);
            assert_eq!(
                g.state.objects[card as usize].zone,
                if adventure {
                    Zone::Exile
                } else {
                    Zone::Graveyard
                }
            );
        }
    }
}
#[test]
fn ecl_bre_no_mandatory_discard_witness_falls_back_and_all_land_library_exiles_completely() {
    for p in [Player::P0, Player::P1] {
        for all_lands in [false, true] {
            let mut g = setup(p);
            g.state.players[p.index()].library.clear();
            paid(&mut g, p);
            let card = named(
                &mut g,
                p,
                if all_lands {
                    "Forest"
                } else {
                    "Thrill of Possibility"
                },
                Zone::Library,
            );
            let land = named(&mut g, p, "Mountain", Zone::Library);
            g.gain_life(p, 2);
            if all_lands {
                g.begin_step(Step::End);
                settle(&mut g);
                assert!(g.state.players[p.index()].library.is_empty());
                assert_eq!(g.state.objects[card as usize].zone, Zone::Exile);
            } else {
                window(&mut g);
                assert!(
                    !g.legal_actions()
                        .iter()
                        .any(|a| matches!(a, Action::CastExiledDuringResolution { .. }))
                );
                checked(&mut g, Action::FinishResolutionCasts);
                settle(&mut g);
                assert_eq!(g.state.objects[card as usize].zone, Zone::Hand);
            }
            assert_eq!(g.state.objects[land as usize].zone, Zone::Exile);
            assert!(g.outcome().is_none());
        }
    }
}
