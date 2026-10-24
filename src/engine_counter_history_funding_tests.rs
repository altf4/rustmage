//! Speculative characteristics must use their own exact placement history.
use super::*;
use crate::engine::assurance::big_helpers::{named, setup};

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
fn counter_history_speculative_rows_upgrade_without_mutating_parent_and_keep_removed_counter_history()
 {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let beast = named(&mut g, p, "Beast, Erudite Aerialist", Zone::Battlefield);
        g.effect_counter(
            beast,
            CounterKind::Stun,
            1,
            replacements::ResumeMode::Pending,
        );
        let object = g.object_ref(beast);
        let snapshot = g.snapshot();
        let goal = goal(p);
        let mut world = FundingState::from_game(&g, &goal);
        assert!(!flying(&world, beast));
        let key = world.key(&goal);
        counter_placement_history::record(&mut world.counter_placement_history, object, p, true);
        assert!(flying(&world, beast));
        assert!(world.key(&goal) != key);
        let index = world.resources.iter().position(|r| r.id == beast).unwrap();
        assert!(
            world
                .keywords(&world.resources[index])
                .contains(Keywords::FLYING)
        );
        world.resources[index].object.counters = InitialCounters::default();
        assert!(flying(&world, beast));
        world.resources[index].object.controller = p.other();
        assert!(!flying(&world, beast));
        counter_placement_history::record(
            &mut world.counter_placement_history,
            object,
            p.other(),
            true,
        );
        assert!(flying(&world, beast));
        world.resources[index].object.generation += 1;
        assert!(!flying(&world, beast));
        assert_eq!(g.snapshot(), snapshot);
        assert!(!g.object_view(beast).keywords.contains(Keywords::FLYING));
        assert!(!g.state.counter_placement_history.as_ref().unwrap()[0].plus_one);
    }
}
#[test]
fn counter_history_speculative_entry_records_the_final_replaced_counters_on_new_incarnation() {
    for p in [Player::P0, Player::P1] {
        for kind in [CounterKind::PlusOne, CounterKind::Stun] {
            let mut g = setup(p);
            let beast = named(&mut g, p, "Beast, Erudite Aerialist", Zone::Hand);
            let old = g.object_ref(beast);
            let mut movement = g.entry_move(beast, Zone::Battlefield, false);
            // commit_projected_entries consumes counters after their replacement
            // enumeration, as on the actual entry path.
            movement.counters = Counters::of(kind, 4);
            let world = FundingState::from_game(&g, &goal(p));
            let mut observed = false;
            assert!(
                world.commit_projected_entries(vec![movement], &[0], &mut |next| {
                    observed = true;
                    let object = next.reference(beast);
                    assert_ne!(object, old);
                    assert!(counter_placement_history::any_placed(
                        &next.counter_placement_history,
                        object,
                        p
                    ));
                    assert_eq!(flying(next, beast), kind == CounterKind::PlusOne);
                    assert_eq!(next.counter_count(beast, kind), 4);
                    true
                })
            );
            assert!(observed);
            assert_eq!(g.object_view(beast).zone, Zone::Hand);
            assert!(g.state.counter_placement_history.is_none());
        }
    }
}
#[test]
fn counter_history_funding_key_keeps_exact_identity_after_counters_removed() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let a = named(&mut g, p, "Beast, Erudite Aerialist", Zone::Battlefield);
        let b = named(&mut g, p, "Beast, Erudite Aerialist", Zone::Battlefield);
        g.record_counter_placement(g.object_ref(a), p, true);
        let goal = goal(p);
        let mut first = FundingState::from_game(&g, &goal);
        let mut second = first.clone();
        let ai = first.resources.iter().position(|r| r.id == a).unwrap();
        let bi = second.resources.iter().position(|r| r.id == b).unwrap();
        first.resources[ai].object.tapped = true;
        second.resources[bi].object.tapped = true;
        assert!(first.key(&goal) != second.key(&goal));
        assert!(flying(&first, a));
        assert!(!flying(&first, b));
    }
}
