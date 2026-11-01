//! Focused printed outcomes; shared engine invariants live in core tests.
//! Hartford's private hand picks and filtered battlefield selection.
use super::big_helpers::{checked, settle};
use super::*;

fn resolve_to_selection(g: &mut Game) {
    for _ in 0..20 {
        if matches!(g.state.pending, Pending::EffectLibrarySelection { .. }) {
            return;
        }
        checked(g, Action::Pass);
    }
    panic!("expected library selection");
}

#[test]
fn consult_counts_lands_at_resolution_and_requires_the_kicked_pick_count() {
    {
        let p = Player::P0;
        for kicked in [false, true] {
            let mut g = scenario(p);
            for _ in 0..2 {
                put(&mut g, p, "Island", Zone::Battlefield);
            }
            let below = put(&mut g, p, "Forest", Zone::Library);
            let a = put(&mut g, p, "Bear Cub", Zone::Library);
            let b = put(&mut g, p, "Quick Study", Zone::Library);
            let c = put(&mut g, p, "Island", Zone::Library);
            let spell = put(&mut g, p, "Consult the Star Charts", Zone::Hand);
            checked(&mut g, Action::AnnounceCast { card: spell });
            checked(&mut g, Action::ChooseKicker { pay: kicked });
            pay(&mut g);
            // The land count is a resolution value, not an announcement value.
            put(&mut g, p, "Island", Zone::Battlefield);
            resolve_to_selection(&mut g);
            let view = g.observe(p).library_selection.unwrap();
            assert_eq!(
                view.remaining.iter().map(|o| o.id).collect::<Vec<_>>(),
                vec![c, b, a]
            );
            assert_eq!(view.minimum, if kicked { 2 } else { 1 });
            assert_eq!(view.maximum, view.minimum);
            assert!(g.observe(p.other()).library_selection.is_none());
            reject(&mut g, Action::SelectLibrary { card: below });
            reject(&mut g, Action::FinishLibrarySelection);
            checked(&mut g, Action::SelectLibrary { card: a });
            if kicked {
                reject(&mut g, Action::FinishLibrarySelection);
                checked(&mut g, Action::SelectLibrary { card: c });
            }
            checked(&mut g, Action::FinishLibrarySelection);
            settle(&mut g);
            assert_eq!(g.object_view(a).zone, Zone::Hand);
            assert_eq!(
                g.object_view(c).zone,
                if kicked { Zone::Hand } else { Zone::Library }
            );
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
            let bottom = &g.state.players[p.index()].library[..if kicked { 1 } else { 2 }];
            assert!(bottom.contains(&b));
            if !kicked {
                assert!(bottom.contains(&c));
            }
        }
    }
}
