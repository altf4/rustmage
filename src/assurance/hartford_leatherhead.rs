//! Leatherhead's paid combat reflexive trigger can remove any counter or decline.
use super::big_helpers::{cast, checked, settle, setup};
use super::*;

#[test]
fn leatherhead_enters_hexproof_and_combat_removal_creates_a_separate_qualified_trigger() {
    for player in [Player::P0, Player::P1] {
        for removal in [
            None,
            Some(CounterKind::Hexproof),
            Some(CounterKind::PlusOne),
        ] {
            let mut g = setup(player);
            let source = put(&mut g, player, "Leatherhead, Swamp Stalker", Zone::Hand);
            let mine = put(&mut g, player, "Mazemind Tome", Zone::Battlefield);
            let theirs = put(&mut g, player.other(), "Mazemind Tome", Zone::Battlefield);
            cast(&mut g, source, None, &[]);
            settle(&mut g);
            assert_eq!(g.counter_count(source, CounterKind::Hexproof), 1);
            assert!(g.keywords(source).contains(Keywords::HEXPROOF));
            assert!(g.keywords(source).contains(Keywords::TRAMPLE));
            if removal == Some(CounterKind::PlusOne) {
                g.add_counter(source, CounterKind::PlusOne, 1);
            }
            g.state.objects[source as usize].summoning_sick = false;
            g.state.step = Step::Attackers;
            g.state.pending = Pending::Attackers;
            checked(&mut g, Action::Attack { card: source });
            checked(&mut g, Action::FinishAttackers);
            for _ in 0..60 {
                if matches!(g.state.pending, Pending::TargetCounterRemoval { .. }) {
                    break;
                }
                let action = g
                    .legal_actions()
                    .into_iter()
                    .find(|a| {
                        matches!(
                            a,
                            Action::Pass | Action::FinishBlockers | Action::PutTrigger { .. }
                        )
                    })
                    .expect("combat continuation");
                checked(&mut g, action);
            }
            assert!(matches!(
                g.state.pending,
                Pending::TargetCounterRemoval { .. }
            ));
            if let Some(counter) = removal {
                checked(&mut g, Action::SelectTargetCounter { counter });
            }
            checked(&mut g, Action::FinishTargetCounterRemoval);
            if removal.is_some() {
                for _ in 0..20 {
                    if matches!(g.state.pending, Pending::Targets { .. }) {
                        break;
                    }
                    let action = g
                        .legal_actions()
                        .into_iter()
                        .find(|a| matches!(a, Action::Pass | Action::PutTrigger { .. }))
                        .expect("reflexive continuation");
                    checked(&mut g, action);
                }
                assert!(!g.legal_actions().contains(&Action::ChooseTarget {
                    target: Target::Card(mine)
                }));
                checked(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(theirs),
                    },
                );
            }
            settle(&mut g);
            assert_eq!(g.object_view(mine).zone, Zone::Battlefield);
            assert_eq!(
                g.object_view(theirs).zone,
                if removal.is_some() {
                    Zone::Graveyard
                } else {
                    Zone::Battlefield
                }
            );
            assert_eq!(
                g.counter_count(source, CounterKind::Hexproof),
                u64::from(removal != Some(CounterKind::Hexproof))
            );
        }
    }
}
