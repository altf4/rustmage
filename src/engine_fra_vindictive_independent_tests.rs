use super::*;

fn vindictive_independent_finish(g: &mut Game) {
    for _ in 0..128 {
        g.prepare_triggers();
        if g.state.pending == Pending::Priority && g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty() && g.state.resolving.is_none() { return; }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("Vindictive Triumph did not finish: {:?}", g.state.pending);
}

#[test]
fn vindictive_planeswalker_returns_under_caster_with_entry_counter_replacement() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p, "Doubling Season", Zone::Battlefield);
        let walker = put(&mut g, p.other(), "Ajani, Caller of the Pride", Zone::Battlefield);
        g.state.objects[walker as usize].counters.loyalty = 1;
        let spell = put(&mut g, p, "Vindictive Triumph", Zone::Hand);
        let original = g.object_ref(walker);
        cast(&mut g, spell, Some(Target::Card(walker)));
        let saved = g.snapshot();
        for _ in 0..2 {
            vindictive_independent_finish(&mut g);
            assert_ne!(g.object_ref(walker), original);
            assert_eq!(g.object_view(walker).zone, Zone::Battlefield);
            assert_eq!(g.object_view(walker).controller, p);
            assert!(g.object_view(walker).tapped);
            assert_eq!(g.counter_count(walker, CounterKind::Loyalty), 8);
            g.begin_step(Step::End);
            vindictive_independent_finish(&mut g);
            assert_eq!(g.object_view(walker).zone, Zone::Exile);
            g.restore(&saved).unwrap();
        }
    }
}

#[test]
fn vindictive_uses_current_copied_mana_value_before_exile() {
    for p in [Player::P0, Player::P1] {
        for (physical, copied, returns) in [("Bear Cub", "Hill Giant", false), ("Hill Giant", "Bear Cub", true)] {
            let mut g = scenario(p);
            let target = put(&mut g, p.other(), physical, Zone::Battlefield);
            let copied = Card::from_name(copied).unwrap();
            g.apply_temporary_copy(g.object_ref(target), CopyRecipe::new(copied), CopyDeadline::EndOfTurn);
            let spell = put(&mut g, p, "Vindictive Triumph", Zone::Hand);
            cast(&mut g, spell, Some(Target::Card(target)));
            let saved = g.snapshot();
            for _ in 0..2 {
                vindictive_independent_finish(&mut g);
                assert_eq!(g.object_view(target).zone, if returns { Zone::Battlefield } else { Zone::Exile });
                assert_eq!(g.state.delayed_returns.as_ref().map_or(0, |rows| rows.len()), usize::from(returns));
                if returns {
                    assert_eq!(g.object_view(target).card, Card::from_name(physical).unwrap());
                    assert_eq!(g.object_view(target).controller, p);
                    assert!(g.object_view(target).tapped);
                }
                g.restore(&saved).unwrap();
            }
        }
    }
}

#[test]
fn vindictive_target_losing_all_legal_types_before_resolution_fizzles() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let target = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let spell = put(&mut g, p, "Vindictive Triumph", Zone::Hand);
        cast(&mut g, spell, Some(Target::Card(target)));
        g.apply_temporary_copy(g.object_ref(target), CopyRecipe::new(Card::from_name("Forest").unwrap()), CopyDeadline::EndOfTurn);
        let saved = g.snapshot();
        for _ in 0..2 {
            vindictive_independent_finish(&mut g);
            assert_eq!(g.object_view(target).zone, Zone::Battlefield);
            assert!(g.state.delayed_returns.is_none());
            g.restore(&saved).unwrap();
        }
    }
}

#[test]
fn copied_vindictive_returns_once_and_invalidates_original_target() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let target = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let spell = put(&mut g, p, "Vindictive Triumph", Zone::Hand);
        cast(&mut g, spell, Some(Target::Card(target)));
        let original = g.state.stack.last().unwrap().clone();
        let copy = g.allocate_spell_copy(&original, p);
        g.state.stack.push(copy);
        let before = g.object_ref(target);
        let saved = g.snapshot();
        for _ in 0..2 {
            vindictive_independent_finish(&mut g);
            assert_ne!(g.object_ref(target), before);
            assert_eq!(g.object_view(target).zone, Zone::Battlefield);
            assert_eq!(g.object_view(target).controller, p);
            assert!(g.object_view(target).tapped);
            assert_eq!(g.state.delayed_returns.as_ref().unwrap().len(), 1);
            g.restore(&saved).unwrap();
        }
    }
}
