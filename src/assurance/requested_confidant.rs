//! Enlightened Confidant's private surveil and exact successful graveyard movement.
use super::*;

#[test]
fn confidant_surveillee_respects_mana_threshold_and_replacement() {
    for p in [Player::P0, Player::P1] {
        for (name, to_grave, prevent, expected) in [
            ("Bear Cub", true, false, Zone::Hand),
            ("Hill Giant", true, false, Zone::Graveyard),
            ("Bear Cub", false, false, Zone::Library),
            ("Bear Cub", true, true, Zone::Exile),
        ] {
            let mut g = scenario(p);
            let source = put(&mut g, p, "Enlightened Confidant", Zone::Hand);
            big_helpers::cast(&mut g, source, None, &[]);
            big_helpers::settle(&mut g);
            assert!(g.keywords(source).contains(Keywords::LIFELINK));
            if prevent {
                put(&mut g, p, "Rest in Peace", Zone::Battlefield);
            }
            let card = put(&mut g, p, name, Zone::Library);
            g.gain_life(p, 2);
            g.begin_step(Step::End);
            g.give_priority();
            g.prepare_triggers();
            g.move_object(source, Zone::Graveyard);
            for _ in 0..40 {
                if matches!(g.state.pending, Pending::EffectLibrary { .. }) {
                    break;
                }
                big_helpers::checked(&mut g, Action::Pass);
            }
            assert!(matches!(g.state.pending, Pending::EffectLibrary { .. }));
            assert!(g.observe(p).library_choice.is_some());
            assert!(g.observe(p.other()).library_choice.is_none());
            let invalid = Action::ArrangeLibrary {
                card: source,
                destination: LibraryDestination::Graveyard,
            };
            reject(&mut g, invalid);
            let saved = g.snapshot();
            let mut branch = g.clone();
            let action = Action::ArrangeLibrary {
                card,
                destination: if to_grave {
                    LibraryDestination::Graveyard
                } else {
                    LibraryDestination::Top
                },
            };
            big_helpers::checked(&mut g, action.clone());
            big_helpers::settle(&mut g);
            assert_eq!(g.object_view(card).zone, expected);
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
            branch.restore(&saved).unwrap();
            big_helpers::checked(&mut branch, action);
            big_helpers::settle(&mut branch);
            assert_eq!(g.snapshot(), branch.snapshot());
        }
    }
}

#[test]
fn confidant_intervening_gain_is_not_retroactive() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p, "Enlightened Confidant", Zone::Battlefield);
        let top = put(&mut g, p, "Bear Cub", Zone::Library);
        g.begin_step(Step::End);
        g.give_priority();
        g.prepare_triggers();
        g.gain_life(p, 2);
        big_helpers::settle(&mut g);
        assert_eq!(g.object_view(top).zone, Zone::Library);
        assert_eq!(g.state.players[p.index()].library.last(), Some(&top));
    }
}

#[test]
fn confidant_surveil_competing_zone_replacements_pause_and_replay() {
    for p in [Player::P0, Player::P1] {
        for effect in [
            ZoneReplacementKind::Exile,
            ZoneReplacementKind::RevealShuffle,
        ] {
            let mut g = scenario(p);
            put(&mut g, p, "Enlightened Confidant", Zone::Battlefield);
            put(&mut g, p.other(), "Rest in Peace", Zone::Battlefield);
            let card = put(&mut g, p, "Darksteel Colossus", Zone::Library);
            g.gain_life(p, 11);
            g.begin_step(Step::End);
            g.give_priority();
            g.prepare_triggers();
            for _ in 0..40 {
                if matches!(g.state.pending, Pending::EffectLibrary { .. }) {
                    break;
                }
                big_helpers::checked(&mut g, Action::Pass);
            }
            let action = Action::ArrangeLibrary {
                card,
                destination: LibraryDestination::Graveyard,
            };
            big_helpers::checked(&mut g, action);
            assert!(matches!(g.state.pending, Pending::ZoneReplacement { .. }));
            let saved = g.snapshot();
            let mut branch = g.clone();
            let choose = Action::ChooseZoneReplacement { effect };
            big_helpers::checked(&mut g, choose.clone());
            big_helpers::settle(&mut g);
            assert_eq!(
                g.object_view(card).zone,
                if effect == ZoneReplacementKind::Exile {
                    Zone::Exile
                } else {
                    Zone::Library
                }
            );
            branch.restore(&saved).unwrap();
            big_helpers::checked(&mut branch, choose);
            big_helpers::settle(&mut branch);
            assert_eq!(g.snapshot(), branch.snapshot());
        }
    }
}

#[test]
fn confidant_uses_live_life_gain_threshold_at_graveyard_move() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p, "Enlightened Confidant", Zone::Battlefield);
        let card = put(&mut g, p, "Hill Giant", Zone::Library);
        g.gain_life(p, 2);
        g.begin_step(Step::End);
        g.give_priority();
        g.prepare_triggers();
        for _ in 0..40 {
            if matches!(g.state.pending, Pending::EffectLibrary { .. }) {
                break;
            }
            big_helpers::checked(&mut g, Action::Pass);
        }
        // Model additional life gain during the suspended surveil continuation.
        g.gain_life(p, 2);
        big_helpers::checked(
            &mut g,
            Action::ArrangeLibrary {
                card,
                destination: LibraryDestination::Graveyard,
            },
        );
        big_helpers::settle(&mut g);
        assert_eq!(g.object_view(card).zone, Zone::Hand);
    }
}

#[test]
fn confidant_uses_front_face_mana_value_for_double_faced_and_adventure_cards() {
    for p in [Player::P0, Player::P1] {
        for (name, gain, expected) in [
            ("Bruce Banner", 1, Zone::Hand),
            ("Beorn, Reluctant Host", 1, Zone::Graveyard),
            ("Beorn, Reluctant Host", 5, Zone::Hand),
        ] {
            let mut g = scenario(p);
            put(&mut g, p, "Enlightened Confidant", Zone::Battlefield);
            let card = put(&mut g, p, name, Zone::Library);
            g.gain_life(p, gain);
            g.begin_step(Step::End);
            g.give_priority();
            g.prepare_triggers();
            for _ in 0..40 {
                if matches!(g.state.pending, Pending::EffectLibrary { .. }) {
                    break;
                }
                big_helpers::checked(&mut g, Action::Pass);
            }
            assert!(matches!(g.state.pending, Pending::EffectLibrary { .. }));
            big_helpers::checked(
                &mut g,
                Action::ArrangeLibrary {
                    card,
                    destination: LibraryDestination::Graveyard,
                },
            );
            big_helpers::settle(&mut g);
            assert_eq!(g.object_view(card).zone, expected, "{name}, gained {gain}");
        }
    }
}
