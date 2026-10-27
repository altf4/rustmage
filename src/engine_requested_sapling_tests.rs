use super::assurance::big_helpers::{cast, checked, named, settle, setup};
use super::*;

fn pay(g: &mut Game) {
    let a = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    checked(g, a);
}
fn definition_objects(g: &Game, p: Player, name: &str) -> Vec<CardId> {
    let c = Card::from_name(name).unwrap();
    g.state
        .objects
        .iter()
        .enumerate()
        .filter(|(_, o)| o.controller == p && o.zone == Zone::Battlefield && o.card == c)
        .map(|(id, _)| id as CardId)
        .collect()
}
#[test]
fn ecl_green_sapling_affinity_landfall_and_exile_cost_freeze_filtered_indestructible() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let mut forests = Vec::new();
        for _ in 0..6 {
            forests.push(named(&mut g, p, "Forest", Zone::Battlefield));
        }
        let nursery = named(&mut g, p, "Sapling Nursery", Zone::Hand);
        let mana = g.state.players[p.index()].mana.iter().sum::<u32>();
        cast(&mut g, nursery, None, &[]);
        settle(&mut g);
        assert_eq!(
            g.state.players[p.index()].mana.iter().sum::<u32>(),
            mana - 2
        );
        let enemy_forest = named(&mut g, p.other(), "Forest", Zone::Hand);
        g.move_object(enemy_forest, Zone::Battlefield);
        settle(&mut g);
        assert!(definition_objects(&g, p, "@token/treefolk_3_4_green_reach").is_empty());
        let own = named(&mut g, p, "Forest", Zone::Hand);
        checked(&mut g, Action::PlayLand { card: own });
        settle(&mut g);
        let trees = definition_objects(&g, p, "@token/treefolk_3_4_green_reach");
        assert_eq!(trees.len(), 1);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let mana = g.state.players[p.index()].mana.iter().sum::<u32>();
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: nursery,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut g);
        assert_eq!(g.object_view(nursery).zone, Zone::Exile);
        assert_eq!(
            g.state.players[p.index()].mana.iter().sum::<u32>(),
            mana - 2
        );
        settle(&mut g);
        for id in forests.into_iter().chain([own, trees[0]]) {
            assert!(g.keywords(id).contains(Keywords::INDESTRUCTIBLE));
        }
        assert!(!g.keywords(enemy_forest).contains(Keywords::INDESTRUCTIBLE));
        assert!(!g.keywords(bear).contains(Keywords::INDESTRUCTIBLE));
        let late = named(&mut g, p, "Magnigoth Sentry", Zone::Battlefield);
        assert!(!g.keywords(late).contains(Keywords::INDESTRUCTIBLE));
        g.finish_cleanup();
        assert!(!g.keywords(trees[0]).contains(Keywords::INDESTRUCTIBLE));
    }
}
