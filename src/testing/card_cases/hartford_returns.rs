//! Focused printed outcomes; shared engine invariants live in core tests.
//! Graveyard landfall, life-conditioned haste and return as an enchantment.
use super::big_helpers::{checked, settle};
use super::*;

#[test]
fn curiosity_exile_replacement_prevents_return() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        put(&mut g, p.other(), "Rest in Peace", Zone::Battlefield);
        let source = put(&mut g, p, "Enduring Curiosity", Zone::Battlefield);
        let kill = put(&mut g, p, "Murder", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: kill });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(source),
            },
        );
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Exile);
    }
}

#[test]
fn curiosity_draws_per_controlled_creature_combat_hit_even_after_returning() {
    {
        let p = Player::P0;
        for returned in [false, true] {
            let mut g = scenario(p);
            let source = put(&mut g, p, "Enduring Curiosity", Zone::Battlefield);
            if returned {
                let kill = put(&mut g, p, "Murder", Zone::Hand);
                checked(&mut g, Action::AnnounceCast { card: kill });
                checked(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(source),
                    },
                );
                pay(&mut g);
                settle(&mut g);
                assert!(!g.is_creature(source));
            }
            let first = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let second = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let draws = g.state.players[p.index()].cards_drawn_this_turn;
            g.state.step = Step::Attackers;
            g.state.pending = Pending::Attackers;
            checked(&mut g, Action::Attack { card: first });
            checked(&mut g, Action::Attack { card: second });
            checked(&mut g, Action::FinishAttackers);
            for _ in 0..60 {
                if g.state.players[p.index()].cards_drawn_this_turn == draws + 2 {
                    break;
                }
                let action = g
                    .legal_actions()
                    .into_iter()
                    .find(|a| {
                        matches!(
                            a,
                            Action::Pass | Action::FinishBlockers | Action::PutTrigger { .. }
                        )
                    })
                    .expect("combat continuation");
                checked(&mut g, action);
            }
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, draws + 2);
            assert_eq!(g.object_view(source).zone, Zone::Battlefield);
        }
    }
}
