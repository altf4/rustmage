//! Focused printed outcomes; shared engine invariants live in core tests.
//! Enlightened Confidant's private surveil and exact successful graveyard movement.
use super::*;

#[test]
fn confidant_intervening_gain_is_not_retroactive() {
    {
        let p = Player::P0;
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
fn confidant_uses_live_life_gain_threshold_at_graveyard_move() {
    {
        let p = Player::P0;
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
    {
        let p = Player::P0;
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
