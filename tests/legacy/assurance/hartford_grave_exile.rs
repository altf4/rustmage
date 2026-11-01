use super::big_helpers::{checked, settle};
use super::*;

#[test]
fn betrayal_opponent_chooses_creature_and_exiles_their_graveyard() {
    for p in [Player::P0, Player::P1] {
        for creature in [false, true] {
            let mut g = scenario(p);
            let grave = put(&mut g, p.other(), "Bear Cub", Zone::Graveyard);
            let own_grave = put(&mut g, p, "Bear Cub", Zone::Graveyard);
            let selected = creature.then(|| put(&mut g, p.other(), "Bear Cub", Zone::Battlefield));
            let spell = put(&mut g, p, "Strategic Betrayal", Zone::Hand);
            checked(&mut g, Action::AnnounceCast { card: spell });
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Player(p.other()),
                },
            );
            pay(&mut g);
            if let Some(selected) = selected {
                for _ in 0..10 {
                    if g.legal_actions().contains(&Action::SelectResolutionExile {
                        object: g.object_ref(selected),
                    }) {
                        break;
                    }
                    let action = g
                        .legal_actions()
                        .into_iter()
                        .find(|a| matches!(a, Action::Pass | Action::PutTrigger { .. }))
                        .unwrap();
                    checked(&mut g, action);
                }
                assert_eq!(g.actor(), Some(p.other()));
                let object = g.object_ref(selected);
                checked(&mut g, Action::SelectResolutionExile { object });
            }
            settle(&mut g);
            assert_eq!(g.object_view(grave).zone, Zone::Exile);
            assert_eq!(g.object_view(own_grave).zone, Zone::Graveyard);
            if let Some(selected) = selected {
                assert_eq!(g.object_view(selected).zone, Zone::Exile);
            }
        }
    }
}

#[test]
fn cover_up_paid_searches_opponents_zones_privately_and_unpaid_only_destroys() {
    for p in [Player::P0, Player::P1] {
        for collect in [false, true] {
            let mut g = scenario(p);
            let evidence = put(&mut g, p, "Colossus of the Blood Age", Zone::Graveyard);
            let creature = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let grave = put(&mut g, p.other(), "Bear Cub", Zone::Graveyard);
            let hand = put(&mut g, p.other(), "Bear Cub", Zone::Hand);
            let library = put(&mut g, p.other(), "Bear Cub", Zone::Library);
            let spell = put(&mut g, p, "Deadly Cover-Up", Zone::Hand);
            checked(&mut g, Action::AnnounceCast { card: spell });
            checked(&mut g, Action::ChooseCollectEvidence { collect });
            if collect {
                let object = g.object_ref(evidence);
                checked(&mut g, Action::ChooseEvidence { object });
                checked(&mut g, Action::FinishEvidence);
            }
            pay(&mut g);
            if collect {
                for _ in 0..10 {
                    if g.legal_actions().contains(&Action::SelectResolutionExile {
                        object: g.object_ref(grave),
                    }) {
                        break;
                    }
                    let action = g
                        .legal_actions()
                        .into_iter()
                        .find(|a| matches!(a, Action::Pass | Action::PutTrigger { .. }))
                        .unwrap();
                    checked(&mut g, action);
                }
                let object = g.object_ref(grave);
                checked(&mut g, Action::SelectResolutionExile { object });
                let own_view = g.observe(p).effect_choice.unwrap();
                assert!(
                    own_view
                        .inspected_cards
                        .unwrap()
                        .iter()
                        .any(|card| card.id == hand)
                );
                assert!(g.observe(p.other()).effect_choice.is_none());
                for card in [hand, library] {
                    let object = g.object_ref(card);
                    checked(&mut g, Action::SelectResolutionExile { object });
                }
                checked(&mut g, Action::FinishSearch);
            }
            settle(&mut g);
            assert_eq!(g.object_view(creature).zone, Zone::Graveyard);
            assert_eq!(
                g.object_view(grave).zone,
                if collect {
                    Zone::Exile
                } else {
                    Zone::Graveyard
                }
            );
            assert_eq!(
                g.object_view(hand).zone,
                if collect { Zone::Exile } else { Zone::Hand }
            );
            assert_eq!(
                g.object_view(library).zone,
                if collect { Zone::Exile } else { Zone::Library }
            );
            if collect {
                assert_eq!(g.state.players[p.other().index()].cards_drawn_this_turn, 1);
            }
        }
    }
}
