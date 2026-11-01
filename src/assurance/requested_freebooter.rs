//! Greedy Freebooter's death trigger, hidden scry and Treasure continuation.
use super::*;

#[test]
fn freebooter_dies_scry_is_private_and_treasure_follows_choice() {
    for p in [Player::P0, Player::P1] {
        for bottom in [false, true] {
            let mut g = scenario(p);
            let below = put(&mut g, p, "Island", Zone::Library);
            let top = put(&mut g, p, "Bear Cub", Zone::Library);
            let source = put(&mut g, p, "Greedy Freebooter", Zone::Hand);
            big_helpers::cast(&mut g, source, None, &[]);
            big_helpers::settle(&mut g);
            assert_eq!(stats(&g, source), (1, 1));
            let murder = put(&mut g, p, "Murder", Zone::Hand);
            big_helpers::cast(&mut g, murder, None, &[Target::Card(source)]);
            for _ in 0..30 {
                if g.observe(p).library_choice.is_some() {
                    break;
                }
                big_helpers::checked(&mut g, Action::Pass);
            }
            assert_eq!(g.object_view(source).zone, Zone::Graveyard);
            assert_eq!(g.observe(p).library_choice.unwrap().remaining[0].id, top);
            assert!(g.observe(p.other()).library_choice.is_none());
            assert!(g.observe(p.other()).objects.iter().all(|o| o.id != top));
            reject(
                &mut g,
                Action::ArrangeLibrary {
                    card: below,
                    destination: LibraryDestination::Bottom,
                },
            );
            contracts::branch_contract(g.clone());
            big_helpers::checked(
                &mut g,
                Action::ArrangeLibrary {
                    card: top,
                    destination: if bottom {
                        LibraryDestination::Bottom
                    } else {
                        LibraryDestination::Top
                    },
                },
            );
            big_helpers::settle(&mut g);
            let library = &g.state.players[p.index()].library;
            assert_eq!(
                if bottom {
                    library.first()
                } else {
                    library.last()
                },
                Some(&top)
            );
            let treasures: Vec<_> = g
                .state
                .objects
                .iter()
                .filter(|o| {
                    o.zone == Zone::Battlefield
                        && o.controller == p
                        && o.card == Card::from_name("@token/b60f6cd3b44b22bd289fe4e2").unwrap()
                })
                .collect();
            assert_eq!(treasures.len(), 1);
            assert!(!treasures[0].tapped);
        }
    }
}

#[test]
fn freebooter_empty_library_still_creates_treasure() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        g.state.players[p.index()].library.clear();
        let source = put(&mut g, p, "Greedy Freebooter", Zone::Battlefield);
        let murder = put(&mut g, p, "Murder", Zone::Hand);
        big_helpers::cast(&mut g, murder, None, &[Target::Card(source)]);
        big_helpers::settle(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Graveyard);
        assert_eq!(
            g.state
                .objects
                .iter()
                .filter(|o| o.zone == Zone::Battlefield
                    && o.controller == p
                    && o.card == Card::from_name("@token/b60f6cd3b44b22bd289fe4e2").unwrap())
                .count(),
            1
        );
    }
}

#[test]
fn stolen_freebooter_scries_for_last_controller_and_treasure_produces_each_color() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let own_top = put(&mut g, p, "Island", Zone::Library);
        let owner_top = put(&mut g, p.other(), "Bear Cub", Zone::Library);
        let source = put(&mut g, p.other(), "Greedy Freebooter", Zone::Battlefield);
        g.state.objects[source as usize].controller = p;
        let murder = put(&mut g, p, "Murder", Zone::Hand);
        big_helpers::cast(&mut g, murder, None, &[Target::Card(source)]);
        for _ in 0..30 {
            if g.observe(p).library_choice.is_some() {
                break;
            }
            big_helpers::checked(&mut g, Action::Pass);
        }
        assert_eq!(g.object_view(source).zone, Zone::Graveyard);
        assert_eq!(
            g.observe(p).library_choice.unwrap().remaining[0].id,
            own_top
        );
        assert!(g.observe(p.other()).library_choice.is_none());
        big_helpers::checked(
            &mut g,
            Action::ArrangeLibrary {
                card: own_top,
                destination: LibraryDestination::Bottom,
            },
        );
        big_helpers::settle(&mut g);
        assert_eq!(
            g.state.players[p.other().index()].library.last(),
            Some(&owner_top)
        );
        let treasure = g
            .state
            .objects
            .iter()
            .enumerate()
            .find_map(|(id, o)| {
                (o.zone == Zone::Battlefield
                    && o.controller == p
                    && o.card == Card::from_name("@token/b60f6cd3b44b22bd289fe4e2").unwrap())
                .then_some(id as CardId)
            })
            .expect("last controller creates Treasure");
        assert_eq!(g.state.objects[treasure as usize].owner, p);
        for color in 0..5 {
            let mut branch = g.clone();
            branch.state.players[p.index()].mana = [0; 6];
            let mut production = [0; 6];
            production[color] = 1;
            let action = branch.legal_actions().into_iter().find(|a| matches!(a, Action::ActivateMana { card, production: actual, .. } if *card == treasure && *actual == production)).expect("Treasure offers each colored mana");
            big_helpers::checked(&mut branch, action);
            assert_eq!(branch.state.players[p.index()].mana, production);
            assert_ne!(branch.object_view(treasure).zone, Zone::Battlefield);
        }
    }
}

#[test]
fn freebooter_exile_replacement_prevents_death_trigger() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p.other(), "Rest in Peace", Zone::Battlefield);
        let source = put(&mut g, p, "Greedy Freebooter", Zone::Battlefield);
        let library = g.state.players[p.index()].library.clone();
        let murder = put(&mut g, p, "Murder", Zone::Hand);
        big_helpers::cast(&mut g, murder, None, &[Target::Card(source)]);
        big_helpers::settle(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Exile);
        assert_eq!(g.state.players[p.index()].library, library);
        assert!(g.state.objects.iter().all(|o| o.zone != Zone::Battlefield
            || o.card != Card::from_name("@token/b60f6cd3b44b22bd289fe4e2").unwrap()));
    }
}
