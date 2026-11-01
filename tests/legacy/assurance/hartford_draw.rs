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

#[test]
fn quantum_warp_entry_draws_two_then_exiles_with_next_turn_permission() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = put(&mut g, p, "Quantum Riddler", Zone::Hand);
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 1];
        checked(&mut g, Action::AnnounceWarp { card: source });
        let pay = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, pay);
        settle(&mut g);
        assert_eq!(g.hand_count(p), 2);
        assert_eq!((g.stats(source).power, g.stats(source).toughness), (4, 6));
        assert!(g.keywords(source).contains(Keywords::FLYING));
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        big_helpers::next_turn(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Exile);
        assert!(g.can_cast_warped(p, source));
    }
}

#[test]
fn quantum_replaces_the_whole_draw_instruction_once_per_source() {
    for p in [Player::P0, Player::P1] {
        for (hand, riddlers, expected) in [(0, 1, 4), (1, 2, 5), (2, 1, 3)] {
            let mut g = setup(p);
            for _ in 0..hand {
                put(&mut g, p, "Island", Zone::Hand);
            }
            for _ in 0..riddlers {
                put(&mut g, p, "Quantum Riddler", Zone::Battlefield);
            }
            g.draw_many_interruptible(p, 3);
            assert_eq!(g.hand_count(p), hand + expected);
        }
    }
}

#[test]
fn quantum_and_bard_single_draw_order_is_owned_and_multi_draw_order_is_outer_first() {
    for p in [Player::P0, Player::P1] {
        for quantum_first in [false, true] {
            let mut g = setup(p);
            let quantum = put(&mut g, p, "Quantum Riddler", Zone::Battlefield);
            let bard = put(&mut g, p, "Bard, King of Dale", Zone::Battlefield);
            g.draw_many_interruptible(p, 1);
            assert_eq!(g.observe(p.other()).decision, "draw_replacement");
            assert_eq!(
                g.observe(p)
                    .draw_replacement
                    .as_ref()
                    .unwrap()
                    .additional_sources,
                vec![g.object_ref(quantum)]
            );
            let source = g.object_ref(if quantum_first { quantum } else { bard });
            checked(&mut g, Action::ChooseDrawReplacement { source });
            assert_eq!(g.hand_count(p), if quantum_first { 4 } else { 3 });
        }
        let mut g = setup(p);
        put(&mut g, p, "Quantum Riddler", Zone::Battlefield);
        put(&mut g, p, "Bard, King of Dale", Zone::Battlefield);
        g.draw_many_interruptible(p, 3);
        assert_eq!(g.hand_count(p), 8);
    }
}
