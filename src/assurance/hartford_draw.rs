//! Hartford draw spells use the existing private payment and harmonize paths.
use super::big_helpers::{cast, checked, settle, setup};
use super::*;

fn reach_payment(g: &mut Game) {
    for _ in 0..20 {
        if matches!(g.state.pending, Pending::EffectChoice { .. }) {
            return;
        }
        checked(g, Action::Pass);
    }
    panic!("missing private payment");
}

#[test]
fn winternight_draws_three_and_privately_chooses_one_creature_or_two_cards() {
    for p in [Player::P0, Player::P1] {
        for creature_route in [false, true] {
            let mut g = setup(p);
            let spell = put(&mut g, p, "Winternight Stories", Zone::Hand);
            let creature = put(&mut g, p, "Bear Cub", Zone::Hand);
            let before = g.hand_count(p);
            cast(&mut g, spell, None, &[]);
            reach_payment(&mut g);
            assert_eq!(g.hand_count(p), before + 2);
            assert!(g.observe(p.other()).effect_choice.is_none());
            if creature_route {
                checked(&mut g, Action::Discard { card: creature });
            } else {
                checked(&mut g, Action::DeclineEffectPayment);
                assert!(matches!(
                    g.state.pending,
                    Pending::EffectDiscard { remaining: 2, .. }
                ));
                checked(&mut g, Action::Discard { card: creature });
                let second = g
                    .legal_actions()
                    .into_iter()
                    .find(|a| matches!(a, Action::Discard { .. }))
                    .unwrap();
                checked(&mut g, second);
            }
            settle(&mut g);
            assert_eq!(g.hand_count(p), before + if creature_route { 1 } else { 0 });
            assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
            assert_eq!(g.object_view(creature).zone, Zone::Graveyard);
        }
    }
}

#[test]
fn winternight_harmonize_taps_a_creature_reduces_cost_and_exiles() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let helper = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let spell = put(&mut g, p, "Winternight Stories", Zone::Graveyard);
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 2];
        let object = g.object_ref(helper);
        checked(&mut g, Action::AnnounceHarmonize { card: spell });
        checked(
            &mut g,
            Action::ChooseHarmonizer {
                object: Some(object),
            },
        );
        let pay = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, pay);
        let clause = g
            .state
            .announcing
            .as_ref()
            .unwrap()
            .cast_costs
            .as_ref()
            .unwrap()
            .clauses
            .iter()
            .position(|r| r.clause == CastCostClause::Harmonize(object) && !r.paid)
            .unwrap();
        checked(
            &mut g,
            Action::ChooseCastCost {
                clause: clause as u16,
            },
        );
        assert!(g.object_view(helper).tapped);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        reach_payment(&mut g);
        checked(&mut g, Action::DeclineEffectPayment);
        settle(&mut g);
        assert_eq!(g.object_view(spell).zone, Zone::Exile);
        assert_eq!(g.hand_count(p), 1);
    }
}
