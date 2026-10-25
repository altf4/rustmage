//! Coupled paid Teamwork/Mana search regressions. The scoped node budget is
//! disabled by default and reset by RAII, including panic unwinding; it prevents
//! the intentionally red development check from becoming a runaway workload.
use super::*;
use crate::engine::assurance::big_helpers::{cast, checked, named, settle, setup};
use std::cell::Cell;
#[derive(Clone, Copy)]
struct Limit {
    nodes: u64,
    maximum: u64,
}
thread_local! { static LIMIT: Cell<Option<Limit>> = const { Cell::new(None) }; }
struct Budget;
impl Budget {
    fn new(maximum: u64) -> Self {
        LIMIT.with(|c| {
            assert!(c.get().is_none());
            c.set(Some(Limit { nodes: 0, maximum }));
        });
        Self
    }
    fn nodes(&self) -> u64 {
        LIMIT.with(|c| c.get().unwrap().nodes)
    }
}
impl Drop for Budget {
    fn drop(&mut self) {
        LIMIT.with(|c| c.set(None));
    }
}
pub(super) fn visit_teamwork_node() {
    LIMIT.with(|c| {
        if let Some(mut limit) = c.get() {
            limit.nodes += 1;
            c.set(Some(limit));
            assert!(
                limit.nodes <= limit.maximum,
                "coupled Teamwork/Mana search exceeded scoped {}-node budget before mana readiness",
                limit.maximum
            );
        }
    });
}
fn reject(g: &mut Game, action: Action) {
    let snapshot = g.snapshot();
    let revision = g.revision();
    let views = [Player::P0, Player::P1].map(|p| serde_json::to_value(g.observe(p)).unwrap());
    assert_eq!(
        g.apply(g.actor().unwrap(), action),
        Err(Error::IllegalAction)
    );
    assert_eq!(g.snapshot(), snapshot);
    assert_eq!(g.revision(), revision);
    assert_eq!(
        [Player::P0, Player::P1].map(|p| serde_json::to_value(g.observe(p)).unwrap()),
        views
    );
}
fn clause(g: &mut Game, kind: CastCostClause) {
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
        .position(|r| r.clause == kind)
        .unwrap() as u16;
    checked(g, Action::ChooseCastCost { clause });
}
fn paid_iron(g: &mut Game, p: Player) -> CardId {
    let id = named(g, p, "Ironheart, Clever Champion", Zone::Hand);
    g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 4];
    cast(g, id, None, &[]);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    settle(g);
    id
}
#[test]
fn msh_teamwork_funding_scalability_paid_atlantis_zero_floated_blue_then_real_islands_both_orders()
{
    for p in [Player::P0, Player::P1] {
        for mana_first in [false, true] {
            let mut g = setup(p);
            let iron = paid_iron(&mut g, p);
            // Initial board of forty ordinary existing Doombot tokens: reachable by
            // repeated token effects, not forty physical copies in a legal deck.
            g.create_tokens(p, Card::MshDoombotToken, 40, false);
            settle(&mut g);
            let robots: Vec<_> = g
                .state
                .objects
                .iter()
                .enumerate()
                .filter_map(|(i, o)| {
                    (o.zone == Zone::Battlefield
                        && o.controller == p
                        && o.card == Card::MshDoombotToken)
                        .then_some(i as CardId)
                })
                .collect();
            assert_eq!(robots.len(), 40);
            let islands = [
                named(&mut g, p, "Island", Zone::Battlefield),
                named(&mut g, p, "Island", Zone::Battlefield),
            ];
            let victim = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let spell = named(&mut g, p, "Atlantis Attacks", Zone::Hand);
            g.state.players[p.index()].mana = [0; 6];
            let budget = Budget::new(256);
            checked(&mut g, Action::AnnounceCast { card: spell });
            assert!(matches!(g.state.pending, Pending::TeamworkIntent { .. }));
            let actions = g.legal_actions();
            assert!(actions.contains(&Action::ChooseTeamwork { pay: false }));
            assert!(actions.contains(&Action::ChooseTeamwork { pay: true }));
            println!(
                "coupled paid Atlantis actor={p:?} mana_first={mana_first} nodes={}",
                budget.nodes()
            );
            drop(budget);
            checked(&mut g, Action::ChooseTeamwork { pay: true });
            checked(&mut g, Action::ChooseMode { mode: 2 });
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Player(p),
                },
            );
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(victim),
                },
            );
            if g.legal_actions().contains(&Action::FinishTargets) {
                checked(&mut g, Action::FinishTargets);
            }
            for card in islands {
                checked(&mut g, Action::TapMana { card });
            }
            assert_eq!(g.state.players[p.index()].mana, [0, 2, 0, 0, 0, 0]);
            checked(&mut g, Action::BeginImprovise);
            let (teammates, helpers) = if mana_first {
                (&robots[5..7], &robots[..5])
            } else {
                (&robots[..2], &robots[2..7])
            };
            if !mana_first {
                clause(&mut g, CastCostClause::Teamwork(4));
                for &id in teammates {
                    let object = g.object_ref(id);
                    checked(&mut g, Action::ChooseTeammate { object });
                }
                checked(&mut g, Action::FinishTeamwork);
            }
            clause(&mut g, CastCostClause::Mana);
            if !mana_first {
                let object = g.object_ref(teammates[0]);
                reject(&mut g, Action::ChooseImproviser { object });
            }
            for &id in helpers {
                let object = g.object_ref(id);
                checked(&mut g, Action::ChooseImproviser { object });
            }
            let pay = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::PayCosts { .. }))
                .unwrap();
            checked(&mut g, pay);
            if mana_first {
                clause(&mut g, CastCostClause::Teamwork(4));
                let object = g.object_ref(helpers[0]);
                reject(&mut g, Action::ChooseTeammate { object });
                for &id in teammates {
                    let object = g.object_ref(id);
                    checked(&mut g, Action::ChooseTeammate { object });
                }
                checked(&mut g, Action::FinishTeamwork);
            }
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert!(
                teammates
                    .iter()
                    .chain(helpers)
                    .all(|&id| g.object_view(id).tapped)
            );
            assert!(!g.object_view(iron).tapped);
            assert_eq!(
                g.state.stack.last().unwrap().cast.unwrap().teamwork,
                Some(true)
            );
            settle(&mut g);
            assert_eq!(g.object_view(victim).zone, Zone::Hand);
            assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
            assert!(g.state.objects.iter().any(|o| o.zone == Zone::Battlefield
                && o.controller == p
                && o.card == Card::MshLeviathanToken));
        }
    }
}
