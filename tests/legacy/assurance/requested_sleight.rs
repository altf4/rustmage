//! Sleight of Hand's private, mandatory selection and library remainder.
use super::*;

#[test]
fn sleight_selects_one_privately_and_bottoms_the_other() {
    for p in [Player::P0, Player::P1] {
        for pick_top in [false, true] {
            let mut g = scenario(p);
            let below = put(&mut g, p, "Bear Cub", Zone::Library);
            let first = put(&mut g, p, "Forest", Zone::Library);
            let top = put(&mut g, p, "Quick Study", Zone::Library);
            let spell = put(&mut g, p, "Sleight of Hand", Zone::Hand);
            big_helpers::cast(&mut g, spell, None, &[]);
            for _ in 0..40 {
                if g.state.pending != Pending::Priority {
                    break;
                }
                big_helpers::checked(&mut g, Action::Pass);
            }
            let view = g.observe(p).library_selection.unwrap();
            assert_eq!(
                view.remaining
                    .iter()
                    .map(|o| o.reference.card)
                    .collect::<Vec<_>>(),
                vec![top, first]
            );
            assert!(g.observe(p.other()).library_selection.is_none());
            assert!(
                !g.legal_actions()
                    .contains(&Action::SelectLibrary { card: below })
            );
            assert!(!g.legal_actions().contains(&Action::FinishLibrarySelection));
            let before = g.snapshot();
            let revision = g.revision();
            assert_eq!(
                g.apply(p, Action::SelectLibrary { card: below }),
                Err(Error::IllegalAction)
            );
            assert_eq!(g.snapshot(), before);
            assert_eq!(g.revision(), revision);
            let (chosen, other) = if pick_top { (top, first) } else { (first, top) };
            big_helpers::checked(&mut g, Action::SelectLibrary { card: chosen });
            big_helpers::checked(&mut g, Action::FinishLibrarySelection);
            big_helpers::checked(&mut g, Action::OrderLibraryBottom { card: other });
            big_helpers::settle(&mut g);
            assert_eq!(g.object_view(chosen).zone, Zone::Hand);
            assert_eq!(g.object_view(other).zone, Zone::Library);
            assert_eq!(g.state.players[p.index()].library[0], other);
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
        }
    }
}
