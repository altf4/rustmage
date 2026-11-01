//! Focused printed outcomes; shared engine invariants live in core tests.
use super::big_helpers::{checked, settle};
use super::*;

#[test]
fn suppression_batch_black_sun_pays_bb_and_removes_indestructible_and_death_triggers_before_destroying()
 {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.add_counter(bear, CounterKind::Indestructible, 1);
        let freebooter = put(&mut g, p.other(), "Greedy Freebooter", Zone::Battlefield);
        let survivor = put(&mut g, p, "Sunspine Lynx", Zone::Battlefield);
        let sun = put(&mut g, p, "Day of Black Sun", Zone::Hand);
        let mut no_black = g.clone();
        no_black.state.players[p.index()].mana = [8, 8, 0, 8, 8, 8];
        assert!(
            !no_black
                .legal_actions()
                .contains(&Action::AnnounceCast { card: sun })
        );
        checked(&mut g, Action::AnnounceCast { card: sun });
        checked(&mut g, Action::ChooseX { value: 2 });
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(bear).zone, Zone::Graveyard);
        assert_eq!(g.object_view(freebooter).zone, Zone::Graveyard);
        assert_eq!(g.object_view(survivor).zone, Zone::Battlefield);
        assert!(g.printed_abilities_available(survivor));
        assert!(
            !g.state
                .objects
                .iter()
                .any(|o| o.zone == Zone::Battlefield && o.card.name() == "Treasure")
        );
    }
}
