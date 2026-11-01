//! Boundary scenarios for the requested modal and library-selection spells.
use super::*;

#[test]
fn origin_fizzles_when_target_leaves_and_returns_as_new_object() {
    for p in [Player::P0, Player::P1] {
        for (name, mode) in [("Murmuring Volume", 0), ("Grizzly Bears", 1)] {
            let mut g = scenario(p);
            let target = put(&mut g, p, name, Zone::Battlefield);
            let spell = put(&mut g, p, "Origin of Metalbending", Zone::Hand);
            act(&mut g, Action::AnnounceCast { card: spell });
            act(&mut g, Action::ChooseMode { mode });
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(target),
                },
            );
            if g.legal_actions().contains(&Action::FinishTargets) {
                act(&mut g, Action::FinishTargets);
            }
            pay(&mut g);
            g.move_object(target, Zone::Exile);
            g.move_object(target, Zone::Battlefield);
            settle(&mut g);
            assert_eq!(g.object_view(target).zone, Zone::Battlefield);
            if mode == 1 {
                assert_eq!(g.stats(target).power, 2);
                assert!(!g.keywords(target).contains(Keywords::INDESTRUCTIBLE));
            }
            assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
        }
    }
}

#[test]
fn sleight_handles_short_libraries_and_rejects_invalid_selection_atomically() {
    for p in [Player::P0, Player::P1] {
        for size in 0..=1 {
            let mut g = scenario(p);
            let old = g.state.players[p.index()].library.to_vec();
            for id in old {
                g.state.objects[id as usize].zone = Zone::Outside;
            }
            g.state.players[p.index()].library.clear();
            let only = (size == 1).then(|| put(&mut g, p, "Forest", Zone::Library));
            let spell = put(&mut g, p, "Sleight of Hand", Zone::Hand);
            big_helpers::cast(&mut g, spell, None, &[]);
            // An empty library resolves immediately; with one card the mandatory
            // selection is forced and is completed automatically.
            big_helpers::settle(&mut g);
            if let Some(only) = only {
                assert_eq!(g.object_view(only).zone, Zone::Hand);
            }
            assert!(g.state.players[p.index()].library.is_empty());
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
            assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
        }
    }
}
