//! Focused printed outcomes; shared engine invariants live in core tests.
//! Additional SOS multicolor interactions with real cast and turn histories.
use super::big_helpers::{checked, settle as finish};
use super::*;
fn until_choice(g: &mut Game) {
    for _ in 0..24 {
        if g.state.pending != Pending::Priority {
            return;
        }
        checked(g, Action::Pass);
    }
    panic!("expected choice");
}
#[test]
fn sos_more_essenceknit_hybrid_pest_controller_deaths_and_turn_reset() {
    {
        let p = Player::P0;
        for kind in 0..5 {
            let mut g = scenario(p);
            if kind == 4 {
                put(&mut g, p, "Rest in Peace", Zone::Battlefield);
            }
            let owner = if kind == 1 { p.other() } else { p };
            let victim = put(
                &mut g,
                owner,
                if kind == 3 {
                    "@token/sos-pest"
                } else {
                    "Bear Cub"
                },
                Zone::Battlefield,
            );
            let reference = g.object_ref(victim);
            if kind == 1 {
                g.change_controller(reference, p)
            } else if kind == 2 {
                g.change_controller(reference, p.other())
            }
            g.move_object(victim, Zone::Graveyard);
            finish(&mut g);
            let qualifies = kind != 2 && kind != 4;
            assert_eq!(
                g.observe(p).players[p.index()].controlled_creature_died_this_turn,
                qualifies
            );
            g.state.players[p.index()].mana = if kind % 2 == 0 {
                [0, 0, 1, 0, 2, 0]
            } else {
                [0, 0, 2, 0, 1, 0]
            };
            let source = put(&mut g, p, "Essenceknit Scholar", Zone::Hand);
            cast(&mut g, source, None);
            finish(&mut g);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert_eq!(
                g.state
                    .objects
                    .iter()
                    .filter(|o| o.zone == Zone::Battlefield && o.card == Card::SosPestToken)
                    .count(),
                1
            );
            let hand = g.hand_count(p);
            g.state.active = p.other();
            g.begin_step(Step::End);
            finish(&mut g);
            assert_eq!(g.hand_count(p), hand);
            g.state.active = p;
            g.begin_step(Step::End);
            // A queued draw does not require the original Scholar to remain on the battlefield.
            if qualifies {
                g.move_object(source, Zone::Exile);
            }
            finish(&mut g);
            assert_eq!(g.hand_count(p), hand + usize::from(qualifies));
            g.begin_turn();
            assert!(!g.observe(p).players[p.index()].controlled_creature_died_this_turn);
        }
    }
}
#[test]
fn sos_more_social_snub_optional_copy_qualifies_when_cast_and_sacrifices_each() {
    {
        let p = Player::P0;
        for (started, accept, remove) in [
            (false, false, false),
            (true, false, false),
            (true, true, false),
            (true, true, true),
        ] {
            let mut g = scenario(p);
            let mut creatures = Vec::new();
            if started {
                for owner in [p, p.other()] {
                    for _ in 0..2 {
                        creatures.push(put(&mut g, owner, "Bear Cub", Zone::Battlefield));
                    }
                }
            }
            let spell = put(&mut g, p, "Social Snub", Zone::Hand);
            cast(&mut g, spell, None);
            if remove {
                for &id in &creatures {
                    g.move_object(id, Zone::Exile);
                }
            }
            if started {
                until_choice(&mut g);
                checked(&mut g, Action::ChooseOptional { accept });
            }
            finish(&mut g);
            let resolutions = 1 + i128::from(started && accept);
            assert_eq!(g.state.players[p.index()].life, 20 + resolutions);
            assert_eq!(g.state.players[p.other().index()].life, 20 - resolutions);
            let survivors = g
                .state
                .objects
                .iter()
                .filter(|o| {
                    o.zone == Zone::Battlefield && o.card == Card::from_name("Bear Cub").unwrap()
                })
                .count();
            assert_eq!(survivors, if started && !remove && !accept { 2 } else { 0 });
        }
    }
}

#[test]
fn sos_more_paradox_surveyor_land_or_x_optional_reveal_and_hybrid() {
    {
        let p = Player::P0;
        for select in [0, 1, 2] {
            let mut g = scenario(p);
            let mut top = Vec::new();
            for name in [
                "Island",
                "Shock",
                "Slumbering Trudge",
                "Bear Cub",
                "Mountain",
            ] {
                top.push(put(&mut g, p, name, Zone::Library));
            }
            g.state.players[p.index()].mana = if select % 2 == 0 {
                [0, 1, 0, 0, 2, 0]
            } else {
                [0, 2, 0, 0, 1, 0]
            };
            let source = put(&mut g, p, "Paradox Surveyor", Zone::Hand);
            cast(&mut g, source, None);
            until_choice(&mut g);
            let view = g.observe(p).library_selection.unwrap();
            assert_eq!(view.remaining.len(), 5);
            assert_eq!(view.candidates.len(), 3);
            assert!(g.observe(p.other()).library_selection.is_none());
            reject(&mut g, Action::SelectLibrary { card: top[1] });
            reject(&mut g, Action::SelectLibrary { card: top[3] });
            if select == 0 {
                checked(&mut g, Action::FinishLibrarySelection);
            } else {
                checked(
                    &mut g,
                    Action::SelectLibrary {
                        card: top[if select == 1 { 0 } else { 2 }],
                    },
                );
            }
            finish(&mut g);
            assert_eq!(g.hand_count(p), usize::from(select != 0));

            if select != 0 {
                let chosen = top[if select == 1 { 0 } else { 2 }];
                assert!(
                    g.observe(p.other())
                        .objects
                        .iter()
                        .any(|o| o.id == chosen && o.zone == Zone::Hand)
                );
            }
        }
    }
}
