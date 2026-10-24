//! Funding worlds must order keyword counters against virtual ability removal.
use super::*;
use crate::engine::assurance::big_helpers::{cast, named, settle, setup};
fn goal(p: Player) -> FundingGoal {
    FundingGoal {
        actor: p,
        required: [0; 6],
        generic: 0,
        reserve: None,
        source: None,
    }
}
fn flying(w: &FundingState<'_>, id: CardId) -> bool {
    characteristic_query::CharacteristicQuery::in_world(w)
        .keywords(id)
        .contains(Keywords::FLYING)
}
#[test]
fn keyword_counters_funding_refresh_removal_control_and_timestamp_memo_do_not_mutate_parent() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.effect_counter(
            bear,
            CounterKind::Flying,
            1,
            replacements::ResumeMode::Pending,
        );
        let aura = named(&mut g, p, "Witness Protection", Zone::Hand);
        cast(&mut g, aura, None, &[Target::Card(bear)]);
        settle(&mut g);
        let snapshot = g.snapshot();
        let goal = goal(p);
        let mut w = FundingState::from_game(&g, &goal);
        assert!(!flying(&w, bear));
        w.prepare_entry_projection();
        let object = w.reference(bear);
        let original = w.key(&goal);
        // Same counts with a later placement history can have different abilities.
        let roles = &mut w.entry_projection.as_mut().unwrap().roles;
        let time = roles.next_timestamp();
        for row in &mut roles.keyword_effects {
            if row.recipient == object
                && row.origin
                    == (KeywordEffectOrigin::Counter {
                        kind: CounterKind::Flying,
                    })
            {
                row.timestamp = time;
            }
        }
        assert!(flying(&w, bear));
        assert!(w.key(&goal) != original);
        let index = w.resources.iter().position(|r| r.id == bear).unwrap();
        assert!(w.keywords(&w.resources[index]).contains(Keywords::FLYING));
        w.resources[index].object.controller = p.other();
        assert!(flying(&w, bear));
        let mut counters = w.counter_amounts(bear).to_owned();
        counters.set(CounterKind::Flying, 2);
        live_counters::set(&mut w.named_counters, object, counters.named);
        let before = w.key(&goal);
        let mut counters = w.counter_amounts(bear).to_owned();
        counters.set(CounterKind::Flying, 1);
        live_counters::set(&mut w.named_counters, object, counters.named);
        assert!(flying(&w, bear));
        assert!(w.key(&goal) != before);
        live_counters::set(&mut w.named_counters, object, None);
        assert!(!flying(&w, bear));
        assert!(!w.keywords(&w.resources[index]).contains(Keywords::FLYING));
        assert_eq!(g.snapshot(), snapshot);
        assert_eq!(g.counter_count(bear, CounterKind::Flying), 1);
        assert!(!g.object_view(bear).keywords.contains(Keywords::FLYING));
    }
}
#[test]
fn keyword_counters_funding_entry_replacements_and_timestamp_permutations_match_actual_rules() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        named(&mut g, p, "Doubling Season", Zone::Battlefield);
        let ids = [
            named(&mut g, p, "Bear Cub", Zone::Hand),
            named(&mut g, p, "Bear Cub", Zone::Hand),
        ];
        let moves = ids
            .into_iter()
            .map(|id| {
                let mut m = g.entry_move(id, Zone::Battlefield, false);
                m.effect_counter = Some((CounterKind::Flying, 1));
                m
            })
            .collect();
        let snapshot = g.snapshot();
        let w = FundingState::from_game(&g, &goal(p));
        let mut times = Vec::new();
        w.visit_entry_moves(moves, true, None, &mut |next| {
            let mut ordered = Vec::new();
            for id in ids {
                assert_eq!(next.counter_count(id, CounterKind::Flying), 2);
                assert!(flying(next, id));
                let row = next
                    .roles()
                    .unwrap()
                    .keyword_effects
                    .iter()
                    .find(|r| {
                        r.recipient == next.reference(id)
                            && matches!(
                                r.origin,
                                KeywordEffectOrigin::Counter {
                                    kind: CounterKind::Flying
                                }
                            )
                    })
                    .unwrap();
                assert!(row.timestamp > 0);
                ordered.push(row.timestamp);
            }
            times.push(ordered[0] < ordered[1]);
            false
        });
        times.sort();
        times.dedup();
        assert_eq!(times, vec![false, true]);
        assert_eq!(g.snapshot(), snapshot);
        assert!(g.state.named_counters.is_none());
    }
}
