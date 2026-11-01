//! Focused printed outcomes; shared engine invariants live in core tests.
//! Mayhem's exact discard origin and the returned creature's independent abilities.
use super::big_helpers::{cast, checked, settle, setup};
use super::*;

fn discard_with_thrill(g: &mut Game, p: Player, card: CardId) {
    let spell = put(g, p, "Thrill of Possibility", Zone::Hand);
    checked(g, Action::AnnounceCast { card: spell });
    let discard = Action::ChooseAdditionalCost {
        object: g.object_ref(card),
    };
    for _ in 0..10 {
        if g.legal_actions().contains(&discard) {
            break;
        }
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. } | Action::ChooseCastCost { .. }))
            .unwrap();
        checked(g, action);
    }
    checked(g, discard);
    while g.state.announcing.is_some() {
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. } | Action::ChooseCastCost { .. }))
            .unwrap();
        checked(g, action);
    }
    settle(g);
}

#[test]
fn carnage_mayhem_pays_br_after_actual_discard_and_countering_returns_it_to_the_graveyard() {
    {
        let p = Player::P0;
        let mut g = setup(p);
        let source = put(&mut g, p, "Carnage, Crimson Chaos", Zone::Hand);
        discard_with_thrill(&mut g, p, source);
        assert_eq!(
            g.observe(p.other()).mayhem_discards_this_turn,
            vec![g.object_ref(source)]
        );
        g.state.players[p.index()].mana = [0, 0, 1, 1, 0, 0];
        g.state.active = p.other();
        g.state.priority = p;
        assert!(
            !g.legal_actions()
                .contains(&Action::AnnounceMayhem { card: source })
        );
        g.state.active = p;
        checked(&mut g, Action::AnnounceMayhem { card: source });
        let pay = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, pay);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(
            g.state.stack.last().unwrap().cast.unwrap().method,
            CastMethod::Mayhem
        );
        checked(&mut g, Action::Pass);
        let counter = put(&mut g, p.other(), "Cancel", Zone::Hand);
        cast(&mut g, counter, None, &[Target::Spell(source)]);
        settle(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Graveyard);
        assert!(!g.can_cast_mayhem(p, source));
        assert!(g.observe(p).mayhem_discards_this_turn.is_empty());
    }
}

#[test]
fn carnage_mayhem_does_not_remember_a_new_incarnation_or_a_later_turn() {
    {
        let p = Player::P0;
        let mut g = setup(p);
        let source = put(&mut g, p, "Carnage, Crimson Chaos", Zone::Graveyard);
        assert!(!g.can_cast_mayhem(p, source));
        g.move_object(source, Zone::Hand);
        g.discard_batch(&[source], None);
        assert!(g.can_cast_mayhem(p, source));
        assert!(!g.can_cast_mayhem(p.other(), source));
        g.move_object(source, Zone::Exile);
        g.move_object(source, Zone::Graveyard);
        assert!(!g.can_cast_mayhem(p, source));
        g.move_object(source, Zone::Hand);
        g.discard_batch(&[source], None);
        assert!(g.can_cast_mayhem(p, source));
        g.state.turn += 1;
        g.begin_turn();
        assert!(!g.can_cast_mayhem(p, source));
    }
}
