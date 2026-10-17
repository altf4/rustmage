//! Fixed printed entry counters use existing replacement and owned-state paths.
use super::ability_timeline_tests::{checked, named, setup};
use super::tests::spawn;
use super::*;

#[test]
fn fixed_named_entry_counters_are_replaced_before_entry_and_reset_per_incarnation() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let card = Card::from_name("Wishclaw Talisman").unwrap();
        assert_eq!(card.enters_counters(), Counters::of(CounterKind::Wish, 3));
        named(&mut g, p, "Doubling Season", Zone::Battlefield);
        let id = spawn(&mut g, p, card, Zone::Exile);
        let movement = g.entry_move(id, Zone::Battlefield, false);
        g.stage_entries(vec![movement], None);
        g.advance_entries();
        for _ in 0..20 {
            if g.state.objects[id as usize].zone == Zone::Battlefield {
                break;
            }
            let a = g
                .legal_actions()
                .into_iter()
                .find(|a| !matches!(a, Action::Concede))
                .unwrap();
            checked(&mut g, a);
        }
        assert_eq!(g.counter_count(id, CounterKind::Wish), 6);
        let old = g.object_ref(id);
        let snapshot = g.snapshot();
        let mut branch = {
            let parent = g.clone();
            parent.clone()
        };
        branch.move_object(id, Zone::Exile);
        assert_eq!(branch.counter_count(id, CounterKind::Wish), 0);
        assert_eq!(g.counter_count(id, CounterKind::Wish), 6);
        let movement = branch.entry_move(id, Zone::Battlefield, false);
        branch.stage_entries(vec![movement], None);
        branch.advance_entries();
        for _ in 0..20 {
            if branch.state.objects[id as usize].zone == Zone::Battlefield {
                break;
            }
            let a = branch
                .legal_actions()
                .into_iter()
                .find(|a| !matches!(a, Action::Concede))
                .unwrap();
            checked(&mut branch, a);
        }
        assert_ne!(branch.object_ref(id), old);
        assert_eq!(branch.counter_count(id, CounterKind::Wish), 6);
        branch.restore(&snapshot).unwrap();
        assert_eq!(branch.snapshot(), g.snapshot());
    }
}
