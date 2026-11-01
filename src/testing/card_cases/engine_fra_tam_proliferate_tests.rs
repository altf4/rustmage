//! Focused printed outcomes; shared engine invariants live in core tests.
use super::*;
use crate::cards::CastFilter;

#[test]
fn tam_reduces_only_own_planeswalker_spells() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let walker = put(&mut g, p, "Ajani, Caller of the Pride", Zone::Hand);
        let enemy = put(&mut g, p.other(), "Ajani, Caller of the Pride", Zone::Hand);
        let creature = put(&mut g, p, "Bear Cub", Zone::Hand);
        assert_eq!(g.spell_reduction(p, walker), 0);
        let tam = put(&mut g, p, "Tam, the Possibility", Zone::Battlefield);
        assert_eq!(g.spell_reduction(p, walker), 1);
        assert_eq!(g.spell_reduction(p, creature), 0);
        assert_eq!(g.spell_reduction(p.other(), enemy), 0);
        g.move_object(tam, Zone::Graveyard);
        assert_eq!(g.spell_reduction(p, walker), 0);
    }
}

#[test]
fn planeswalker_cast_filter_distinguishes_card_kinds() {
    let walker = Card::from_name("Ajani, Caller of the Pride").unwrap();
    let creature = Card::from_name("Bear Cub").unwrap();
    assert!(CastFilter::Planeswalker.matches(walker));
    assert!(!CastFilter::Planeswalker.matches(creature));
    assert!(CastFilter::Planeswalker.matches_mana_value(walker, 0));
    assert!(!CastFilter::Planeswalker.matches_mana_value(creature, 99));
}
