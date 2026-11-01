//! Focused printed outcomes; shared engine invariants live in core tests.
//! Greedy Freebooter's death trigger, hidden scry and Treasure continuation.
use super::*;

#[test]
fn freebooter_empty_library_still_creates_treasure() {
    {
        let p = Player::P0;
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
    {
        let p = Player::P0;
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
