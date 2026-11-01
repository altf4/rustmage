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
    for p in [Player::P0, Player::P1] {
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

#[test]
fn battlefront_privately_filters_seven_and_puts_up_to_two_eligible_permanents_into_play() {
    for p in [Player::P0, Player::P1] {
        for choose in [false, true] {
            let mut g = scenario(p);
            let below = put(&mut g, p, "Mazemind Tome", Zone::Library);
            let eligible_a = put(&mut g, p, "Mazemind Tome", Zone::Library);
            let eligible_b = put(&mut g, p, "Phyrexian Arena", Zone::Library);
            let mut invalid = Vec::new();
            for name in [
                "Forest",
                "Bear Cub",
                "Quick Study",
                "Hedron Archive",
                "Day of Judgment",
            ] {
                invalid.push(put(&mut g, p, name, Zone::Library));
            }
            let spell = put(&mut g, p, "United Battlefront", Zone::Hand);
            checked(&mut g, Action::AnnounceCast { card: spell });
            pay(&mut g);
            resolve_to_selection(&mut g);
            let view = g.observe(p).library_selection.unwrap();
            assert_eq!(view.remaining.len(), 7);
            assert_eq!(view.minimum, 0);
            assert_eq!(view.maximum, 2);
            assert_eq!(
                view.candidates.iter().map(|r| r.card).collect::<Vec<_>>(),
                vec![eligible_b, eligible_a]
            );
            assert!(g.observe(p.other()).library_selection.is_none());
            for card in invalid.iter().copied().chain([below]) {
                reject(&mut g, Action::SelectLibrary { card });
            }
            if choose {
                checked(&mut g, Action::SelectLibrary { card: eligible_a });
                checked(&mut g, Action::SelectLibrary { card: eligible_b });
            }
            checked(&mut g, Action::FinishLibrarySelection);
            settle(&mut g);
            for card in [eligible_a, eligible_b] {
                assert_eq!(
                    g.object_view(card).zone,
                    if choose {
                        Zone::Battlefield
                    } else {
                        Zone::Library
                    }
                );
                if choose {
                    assert_eq!(g.object_view(card).controller, p);
                }
            }
            let bottom_count = if choose { 5 } else { 7 };
            let bottom = &g.state.players[p.index()].library[..bottom_count];
            assert!(invalid.iter().all(|id| bottom.contains(id)));
            assert!(!bottom.contains(&below));
        }
    }
}
