//! Evidence audit of previously admitted physical ECL cards.
use super::ability_timeline_tests::{checked, named, settle, setup};
use super::*;
#[test]
fn ecl_inherited_basic_lands_play_once_generate_exact_color_and_pay_real_spells() {
    for p in [Player::P0, Player::P1] {
        for (color, land, spell) in [
            (0, "Plains", "Savannah Lions"),
            (1, "Island", "Unsummon"),
            (2, "Swamp", "Duress"),
            (3, "Mountain", "Shock"),
            (4, "Forest", "Llanowar Elves"),
        ] {
            let mut g = setup(p);
            g.state.players[p.index()].mana = [0; 6];
            let first = named(&mut g, p, land, Zone::Hand);
            let second = named(&mut g, p, land, Zone::Hand);
            let spell = named(&mut g, p, spell, Zone::Hand);
            let target = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
            let visible = serde_json::to_value(g.observe(p.other())).unwrap();
            assert!(
                !g.observe(p.other())
                    .objects
                    .iter()
                    .any(|o| o.reference.card == first),
                "hidden hand land"
            );
            let before = g.snapshot();
            assert!(
                g.apply(p.other(), Action::PlayLand { card: first })
                    .is_err()
            );
            assert_eq!(g.snapshot(), before);
            checked(&mut g, Action::PlayLand { card: first });
            assert_eq!(g.object_view(first).zone, Zone::Battlefield);
            assert!(!g.object_view(first).tapped);
            assert!(
                !g.legal_actions()
                    .contains(&Action::PlayLand { card: second })
            );
            checked(&mut g, Action::TapMana { card: first });
            let mut mana = [0; 6];
            mana[color] = 1;
            assert_eq!(g.state.players[p.index()].mana, mana);
            assert!(g.state.stack.is_empty());
            let before = g.snapshot();
            assert!(g.apply(p, Action::TapMana { card: first }).is_err());
            assert_eq!(g.snapshot(), before);
            checked(&mut g, Action::AnnounceCast { card: spell });
            let actions = g.legal_actions();
            if actions.contains(&Action::ChooseTarget {
                target: Target::Card(target),
            }) {
                checked(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(target),
                    },
                );
            } else if actions.contains(&Action::ChooseTarget {
                target: Target::Player(p.other()),
            }) {
                checked(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Player(p.other()),
                    },
                );
            }
            for _ in 0..5 {
                if g.state.announcing.is_none() {
                    break;
                }
                let action = g
                    .legal_actions()
                    .into_iter()
                    .find(|a| matches!(a, Action::PayCosts { .. } | Action::FinishTargets))
                    .unwrap();
                checked(&mut g, action);
            }
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            settle(&mut g);
            assert_ne!(visible, serde_json::to_value(g.observe(p.other())).unwrap());
        }
    }
}
