//! Focused printed outcomes; shared engine invariants live in core tests.
//! Paid artifact upgrade and exact-source mass exile/conversion.
use super::big_helpers::{checked, settle};
use super::*;

fn activation(card: CardId) -> Action {
    Action::AnnounceActivate {
        card,
        ability: 0,
        counter_payment: Counters::default(),
    }
}
fn until_search(g: &mut Game) {
    for _ in 0..20 {
        if matches!(g.state.pending, Pending::EffectSearch { .. }) {
            return;
        }
        checked(g, Action::Pass);
    }
    panic!("expected search");
}

#[test]
fn bay_pays_another_artifact_and_searches_the_frozen_mana_value_plus_one() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let bay = put(&mut g, p, "Repurposing Bay", Zone::Battlefield);
        let material = put(&mut g, p, "Mazemind Tome", Zone::Battlefield);
        let desired = put(&mut g, p, "Repurposing Bay", Zone::Library);
        let wrong_value = put(&mut g, p, "Hedron Archive", Zone::Library);
        let wrong_kind = put(&mut g, p, "Reclamation Sage", Zone::Library);
        let mut off_turn = g.clone();
        off_turn.state.active = p.other();
        assert!(!off_turn.legal_actions().contains(&activation(bay)));
        checked(&mut g, activation(bay));
        let source = g.object_ref(bay);
        reject(&mut g, Action::ChooseAdditionalCost { object: source });
        let object = g.object_ref(material);
        checked(&mut g, Action::ChooseAdditionalCost { object });
        pay(&mut g);
        assert!(g.object_view(bay).tapped);
        assert_eq!(g.object_view(material).zone, Zone::Graveyard);
        assert_eq!(
            g.state
                .stack
                .last()
                .unwrap()
                .sacrificed_mana_value
                .as_deref(),
            Some(&2)
        );
        // Both the paid permanent and ability source can leave without changing the search.
        g.move_object(material, Zone::Hand);
        g.move_object(bay, Zone::Graveyard);
        until_search(&mut g);
        assert!(g.observe(p.other()).search_choice.is_none());
        for card in [wrong_value, wrong_kind] {
            reject(&mut g, Action::ChooseSearch { card });
        }
        checked(&mut g, Action::ChooseSearch { card: desired });
        checked(&mut g, Action::FinishSearch);
        settle(&mut g);
        assert_eq!(g.object_view(desired).zone, Zone::Battlefield);
        assert_eq!(g.object_view(desired).controller, p);
    }
}

#[test]
fn starcage_exiles_both_sides_and_converts_only_cards_that_reach_a_graveyard() {
    {
        let p = Player::P0;
        for replacement in [false, true] {
            let mut g = scenario(p);
            if replacement {
                put(&mut g, p, "Rest in Peace", Zone::Battlefield);
            }
            let own = put(&mut g, p, "Mazemind Tome", Zone::Battlefield);
            let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let token = put(
                &mut g,
                p.other(),
                "@token/c9fd6abc12b4083149908964",
                Zone::Battlefield,
            );
            let land = put(&mut g, p, "Forest", Zone::Battlefield);
            let big = put(&mut g, p.other(), "Hedron Archive", Zone::Battlefield);
            let cage = put(&mut g, p, "Pinnacle Starcage", Zone::Hand);
            checked(&mut g, Action::AnnounceCast { card: cage });
            pay(&mut g);
            settle(&mut g);
            for id in [own, enemy] {
                assert_eq!(g.object_view(id).zone, Zone::Exile);
            }
            for id in [land, big, cage] {
                assert_eq!(g.object_view(id).zone, Zone::Battlefield);
            }
            if !replacement {
                // Existing exact-source until-leaves returns both owners' cards immediately.
                let mut leaves = g.clone();
                leaves.move_object(cage, Zone::Graveyard);
                settle(&mut leaves);
                for id in [own, enemy] {
                    assert_eq!(leaves.object_view(id).zone, Zone::Battlefield);
                }
                assert_eq!(leaves.object_view(enemy).controller, p.other());
                assert_eq!(leaves.object_view(token).zone, Zone::Outside);
            }
            checked(&mut g, activation(cage));
            pay(&mut g);
            settle(&mut g);
            for id in [own, enemy, cage] {
                assert_eq!(
                    g.object_view(id).zone,
                    if replacement {
                        Zone::Exile
                    } else {
                        Zone::Graveyard
                    }
                );
            }
            let robots = g
                .state
                .objects
                .iter()
                .filter(|o| {
                    o.zone == Zone::Battlefield
                        && o.controller == p
                        && o.card == Card::from_name("@token/c9fd6abc12b4083149908964").unwrap()
                })
                .count();
            assert_eq!(robots, if replacement { 0 } else { 2 });
        }
    }
}
