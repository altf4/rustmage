//! The ordinary hand-cast route, separate from the permanent's ability proofs.
use super::*;
use crate::cards::coverage_bindings::Cost;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    card: String,
    catalogue_id: u16,
    cost: Cost,
}

#[test]
fn each_ordinary_permanent_pays_its_printed_mana_and_enters() {
    let cases: Vec<Case> = fixture_json("normal_permanents.json");
    let mut seen = std::collections::BTreeSet::new();
    for case in cases {
        println!("normal permanent: {}", case.card);
        assert!(seen.insert(case.catalogue_id));
        assert_eq!(
            Card::from_name(&case.card).unwrap().index(),
            usize::from(case.catalogue_id)
        );
        let mut g = game();
        let card = Card::from_name(&case.card).unwrap();
        assert_eq!(card.cost(), (case.cost.colored, case.cost.generic));
        assert_eq!(card.info().mana_cost, case.cost.printed);
        g.state.players[0].mana = case.cost.pool();
        let source = put(&mut g, Player::P0, &case.card, Zone::Hand);
        cast(&mut g, source, &[]);
        assert_eq!(g.state.players[0].mana, [0; 6], "printed mana cost");
        until(&mut g, |g| g.on_battlefield(source));
        assert_eq!(g.object_view(source).controller, Player::P0);
        covered(&format!("card/{}/cast/normal", case.catalogue_id));
    }
}
