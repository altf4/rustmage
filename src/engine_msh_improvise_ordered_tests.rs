//! Private ordered-cost matching tested at its owning module boundary.
use super::*;
use crate::engine::assurance::big_helpers::{named, setup};
fn goal(actor: Player, required: Mana, generic: u32) -> FundingGoal {
    FundingGoal {
        actor,
        required,
        generic,
        source: None,
        reserve: None,
    }
}
#[test]
fn msh_improvise_ordered_hall_small_oracle_and_forty_artifact_fail_first() {
    for p in [Player::P0, Player::P1] {
        for count in (0u32..=3).chain(std::iter::once(40)) {
            for blue in 0..=2 {
                for generic in [0, 1, 2, 3, 4, 5, 40, u32::MAX] {
                    let mut g = setup(p);
                    for _ in 0..count {
                        named(&mut g, p, "Hawkeye's Bow", Zone::Battlefield);
                    }
                    let arc = named(&mut g, p, "Arc Reactor", Zone::Hand);
                    g.state.players[p.index()].mana = [0, blue, 0, 0, 0, 0];
                    let before = g.snapshot();
                    let purpose = g.spell_mana_purpose_for_card(
                        p,
                        arc,
                        Card::from_name("Arc Reactor").unwrap(),
                    );
                    let need = goal(p, [0, 1, 0, 0, 0, 0], generic);
                    let mut w = FundingState::from_game(&g, &need);
                    w.set_purpose(purpose);
                    w.install_cost_context(Some(Box::new(CostContext {
                        actor: p,
                        source: g.object_ref(arc),
                        clauses: vec![CastCostRow {
                            clause: CastCostClause::Mana,
                            paid: false,
                        }],
                        active: None,
                        casualty: vec![],
                        additional: None,
                        forage: None,
                        selected: vec![],
                        exact: false,
                        counters: vec![],
                        mana: Some(ManaCost {
                            colored: need.required,
                            generic,
                        }),
                        teammates: vec![],
                        teamwork_exact: false,
                    })));
                    // Independent generic-only arithmetic: an artifact contributes at most
                    // once and cannot cover the missing U. No subsets or mana units expand.
                    assert_eq!(
                        w.cost_completion(),
                        blue >= 1
                            && u64::from(generic)
                                <= u64::from(blue.saturating_sub(1)) + u64::from(count)
                    );
                    assert_eq!(g.snapshot(), before);
                }
            }
        }
    }
}
