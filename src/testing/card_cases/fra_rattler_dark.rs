//! Focused printed outcomes; shared engine invariants live in core tests.
use super::*;

// Gated card definitions are exercised directly, without opening them to decks.
fn draft(g: &mut Game, owner: Player, name: &str, zone: Zone) -> CardId {
    let id = put(g, owner, "Bear Cub", zone);
    g.state.objects[id as usize].card = Card::from_name(name).unwrap();
    id
}

#[test]
fn dark_matter_mills_only_remaining_cards_and_respects_exile_replacement() {
    {
        let p = Player::P0;
        for replacement in [false, true] {
            let mut g = scenario(p);
            while g.state.players[p.index()].library.len() > 2 {
                let id = *g.state.players[p.index()].library.last().unwrap();
                g.move_object(id, Zone::Exile);
            }
            if replacement {
                put(&mut g, p.other(), "Rest in Peace", Zone::Battlefield);
            }
            let creature = draft(&mut g, p, "Dark Matter Manipulator", Zone::Hand);
            cast(&mut g, creature, None);

            settle(&mut g);
            assert_eq!(g.state.players[p.index()].library.len(), 0);
            let in_graveyard = g
                .state
                .objects
                .iter()
                .filter(|o| o.owner == p && o.zone == Zone::Graveyard)
                .count();
            assert_eq!(in_graveyard, if replacement { 0 } else { 2 });
            assert_eq!(stats(&g, creature), (1, 2));
            assert!(g.outcome().is_none());
        }
    }
}
