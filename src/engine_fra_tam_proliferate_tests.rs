use super::*;
use crate::cards::CastFilter;

#[test]
fn tam_reduces_only_own_planeswalker_spells() {
    for p in [Player::P0, Player::P1] {
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
fn tam_pays_five_colors_and_repeats_for_distinct_live_planeswalker_types() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let tam = put(&mut g, p, "Tam, the Possibility", Zone::Battlefield);
        let ajani = put(&mut g, p, "Ajani, Caller of the Pride", Zone::Battlefield);
        let kaito = put(&mut g, p, "Kaito, Cunning Infiltrator", Zone::Battlefield);
        let creature = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.add_counter(creature, CounterKind::PlusOne, 1);
        let activate = Action::AnnounceActivate { card: tam, ability: 0, counter_payment: Counters::default() };
        g.state.players[p.index()].mana = [1, 1, 1, 1, 0, 1];
        let before = g.snapshot();
        reject(&mut g, activate.clone());
        assert_eq!(g.snapshot(), before);
        g.state.players[p.index()].mana = [1, 1, 1, 1, 1, 0];
        act(&mut g, activate);
        pay(&mut g);
        assert!(g.state.objects[tam as usize].tapped);
        // The number of types is measured at resolution, not activation.
        g.move_object(kaito, Zone::Graveyard);
        for _ in 0..16 {
            if matches!(g.state.pending, Pending::Proliferate { .. }) { break; }
            act(&mut g, Action::Pass);
        }
        let object = g.object_ref(creature);
        act(&mut g, Action::ProliferatePermanent { object });
        act(&mut g, Action::FinishProliferate);
        assert_eq!(g.counter_count(creature, CounterKind::PlusOne), 2);
        assert!(!matches!(g.state.pending, Pending::Proliferate { .. }));
        assert!(g.on_battlefield(ajani));
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
