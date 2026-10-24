//! Paid exact exile permissions and revealed search-destination transactions.
use super::big_helpers::{checked, next_turn, settle as finish};
use super::*;
fn pause(g: &mut Game) {
    for _ in 0..20 {
        if g.state.pending != Pending::Priority {
            return;
        }
        checked(g, Action::Pass);
    }
    panic!("expected choice");
}
#[test]
fn sos_effects_scrollsmith_hybrid_targets_normal_paid_cast_and_deadline() {
    for p in [Player::P0, Player::P1] {
        for white in [false, true] {
            let mut g = scenario(p);
            let shock = put(&mut g, p, "Shock", Zone::Graveyard);
            let land = put(&mut g, p, "Island", Zone::Graveyard);
            let creature = put(&mut g, p, "Bear Cub", Zone::Graveyard);
            let foreign = put(&mut g, p.other(), "Shock", Zone::Graveyard);
            g.state.players[p.index()].mana = if white {
                [2, 0, 0, 1, 0, 0]
            } else {
                [1, 0, 0, 2, 0, 0]
            };
            let source = put(&mut g, p, "Practiced Scrollsmith", Zone::Hand);
            cast(&mut g, source, None);
            pause(&mut g);
            for bad in [land, creature, foreign] {
                reject(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(bad),
                    },
                );
            }
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(shock),
                },
            );
            finish(&mut g);
            assert_eq!(g.object_view(shock).zone, Zone::Exile);
            assert!(g.can_play_exiled(p, shock));
            assert!(g.keywords(source).contains(Keywords::FIRST_STRIKE));
            g.move_object(source, Zone::Graveyard);
            assert!(g.can_play_exiled(p, shock));
            if white {
                next_turn(&mut g);
                assert!(g.can_play_exiled(p, shock));
                next_turn(&mut g);
                assert!(g.can_play_exiled(p, shock));
                next_turn(&mut g);
                assert!(!g.can_play_exiled(p, shock));
            } else {
                let target = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
                g.state.players[p.index()].mana[3] = 1;
                checked(&mut g, Action::AnnounceExile { card: shock });
                checked(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(target),
                    },
                );
                pay(&mut g);
                finish(&mut g);
                assert_eq!(g.object_view(shock).zone, Zone::Graveyard);
                assert!(!g.can_play_exiled(p, shock));
            }
        }
    }
}
#[test]
fn sos_effects_suspend_owner_permissions_token_and_full_target_fizzle() {
    for p in [Player::P0, Player::P1] {
        for kind in 0..3 {
            let mut g = scenario(p);
            let target = put(
                &mut g,
                p.other(),
                if kind == 2 {
                    "@token/sos-spirit"
                } else {
                    "Bear Cub"
                },
                Zone::Battlefield,
            );
            let reference = g.object_ref(target);
            g.change_controller(reference, p);
            let top = put(&mut g, p, "Island", Zone::Library);
            let spell = put(&mut g, p, "Suspend Aggression", Zone::Hand);
            cast(&mut g, spell, Some(Target::Card(target)));
            if kind == 1 {
                g.move_object(target, Zone::Hand);
            }
            finish(&mut g);
            if kind == 1 {
                assert_eq!(g.object_view(top).zone, Zone::Library);
                assert!(!g.can_play_exiled(p, top));
                continue;
            }
            assert_eq!(g.object_view(top).zone, Zone::Exile);
            assert!(g.can_play_exiled(p, top));
            assert!(!g.can_play_exiled(p.other(), top));
            if kind == 0 {
                assert!(g.can_play_exiled(p.other(), target));
                assert!(!g.can_play_exiled(p, target));
            } else {
                assert_eq!(g.object_view(target).zone, Zone::Outside);
                assert!(
                    !g.state
                        .exile_permissions
                        .iter()
                        .any(|r| r.object.card == target)
                );
            }
            checked(&mut g, Action::PlayLand { card: top });
            finish(&mut g);
            assert_eq!(g.object_view(top).zone, Zone::Battlefield);
        }
    }
}
#[test]
fn sos_effects_dina_private_search_public_reveal_then_destination_and_shuffle() {
    for p in [Player::P0, Player::P1] {
        for choice in 0..4 {
            let mut g = scenario(p);
            if choice == 3 {
                put(&mut g, p, "Rest in Peace", Zone::Battlefield);
            }
            let creature = put(&mut g, p, "Bear Cub", Zone::Library);
            let bad = put(&mut g, p, "Island", Zone::Library);
            let spell = put(&mut g, p, "Dina's Guidance", Zone::Hand);
            cast(&mut g, spell, None);
            pause(&mut g);
            assert!(g.observe(p.other()).search_choice.is_none());
            assert!(g.observe(p.other()).search_destination_choice.is_none());
            reject(&mut g, Action::ChooseSearch { card: bad });
            if choice != 0 {
                checked(&mut g, Action::ChooseSearch { card: creature });
            }
            checked(&mut g, Action::FinishSearch);
            if choice == 0 {
                finish(&mut g);
                assert_eq!(g.object_view(creature).zone, Zone::Library);
                assert!(g.observe(p.other()).search_destination_choice.is_none());
                continue;
            }
            assert_eq!(g.object_view(creature).zone, Zone::Library);
            let view = g.observe(p.other()).search_destination_choice.unwrap();
            assert_eq!(view.cards.len(), 1);
            assert_eq!(view.cards[0].id, creature);
            assert!(g.observe(p.other()).search_choice.is_none());
            assert!(g.observe(p).search_choice.is_none());
            reject(
                &mut g,
                Action::ChooseSearchDestination {
                    destination: crate::SearchDestination::Battlefield,
                },
            );
            checked(
                &mut g,
                Action::ChooseSearchDestination {
                    destination: if choice == 1 {
                        crate::SearchDestination::Hand
                    } else {
                        crate::SearchDestination::Graveyard
                    },
                },
            );
            finish(&mut g);
            assert_eq!(
                g.object_view(creature).zone,
                if choice == 1 {
                    Zone::Hand
                } else if choice == 3 {
                    Zone::Exile
                } else {
                    Zone::Graveyard
                }
            );
            if choice == 1 {
                assert!(
                    g.observe(p.other())
                        .objects
                        .iter()
                        .any(|o| o.id == creature && o.zone == Zone::Hand)
                );
            }
        }
    }
}
