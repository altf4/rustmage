//! Focused printed outcomes; shared engine invariants live in core tests.
//! Ward and life-gain explore use the source's live or last-known characteristics.
use super::big_helpers::{checked, settle, setup};
use super::*;

#[test]
fn amalia_explore_checks_exact_power_after_counter_and_respects_indestructible() {
    {
        let p = Player::P0;
        for (growth, destroy) in [(16, false), (17, true), (18, false)] {
            let mut g = setup(p);
            let source = put(&mut g, p, "Amalia Benavides Aguirre", Zone::Battlefield);
            let victim = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let protected = put(&mut g, p.other(), "Darksteel Colossus", Zone::Battlefield);
            let top = put(&mut g, p, "Bear Cub", Zone::Library);
            g.state.objects[source as usize].growth.power = growth;
            g.gain_life(p, 1);
            settle(&mut g);
            assert_eq!(g.stats(source).power, 3 + growth);
            assert_eq!(g.object_view(victim).zone == Zone::Graveyard, destroy);
            assert_eq!(g.object_view(protected).zone, Zone::Battlefield);
            assert_eq!(g.object_view(top).zone, Zone::Library);
        }
    }
}

#[test]
fn amalia_ward_can_be_paid_or_counter_the_opposing_spell() {
    {
        let p = Player::P0;
        for pay_life in [false, true] {
            let mut g = setup(p.other());
            let source = put(&mut g, p, "Amalia Benavides Aguirre", Zone::Battlefield);
            let shock = put(&mut g, p.other(), "Shock", Zone::Hand);
            super::big_helpers::cast(&mut g, shock, None, &[Target::Card(source)]);
            for _ in 0..40 {
                if matches!(g.state.pending, Pending::ResolutionLife { .. }) {
                    break;
                }
                checked(&mut g, Action::Pass);
            }
            assert!(matches!(g.state.pending, Pending::ResolutionLife { .. }));
            let life = g.state.players[p.other().index()].life;
            checked(
                &mut g,
                if pay_life {
                    Action::PayResolutionLife
                } else {
                    Action::DeclineResolutionLife
                },
            );
            settle(&mut g);
            assert_eq!(
                g.state.players[p.other().index()].life,
                life - if pay_life { 3 } else { 0 }
            );
            assert_eq!(
                g.object_view(source).zone,
                if pay_life {
                    Zone::Graveyard
                } else {
                    Zone::Battlefield
                }
            );
        }
    }
}
