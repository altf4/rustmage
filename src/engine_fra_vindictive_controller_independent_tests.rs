use super::*;

#[test]
fn vindictive_copy_uses_copy_controller_for_return_and_entry_replacement() {
    for caster in [Player::P0, Player::P1] {
        let mut g = scenario(caster);
        let copy_controller = caster.other();
        put(&mut g, copy_controller, "Doubling Season", Zone::Battlefield);
        let target = put(&mut g, caster, "Ajani, Caller of the Pride", Zone::Battlefield);
        // The bare battlefield fixture does not apply printed entry loyalty.
        g.state.objects[target as usize].counters.loyalty = 4;
        let spell = put(&mut g, caster, "Vindictive Triumph", Zone::Hand);
        cast(&mut g, spell, Some(Target::Card(target)));
        let original = g.state.stack.last().unwrap().clone();
        let copy = g.allocate_spell_copy(&original, copy_controller);
        g.state.stack.push(copy);
        assert_eq!(g.object_view(target).zone, Zone::Battlefield);
        let before = g.object_ref(target);
        let saved = g.snapshot();
        for _ in 0..2 {
            for _ in 0..96 {
                g.prepare_triggers();
                if g.state.pending == Pending::Priority && g.state.stack.is_empty()
                    && g.state.pending_triggers.is_empty() && g.state.resolving.is_none() { break; }
                choose(&mut g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }
                    | Action::ChooseReplacement { .. } | Action::ChooseEntryReplacement { .. }));
            }
            assert!(g.state.stack.is_empty());
            assert_ne!(g.object_ref(target), before);
            assert_eq!(g.object_view(target).zone, Zone::Battlefield);
            assert_eq!(g.object_view(target).controller, copy_controller);
            assert!(g.object_view(target).tapped);
            assert_eq!(g.counter_count(target, CounterKind::Loyalty), 8);
            assert_eq!(g.state.delayed_returns.as_ref().unwrap().len(), 1);
            g.begin_step(Step::End);
            for _ in 0..96 {
                g.prepare_triggers();
                if g.state.pending == Pending::Priority && g.state.stack.is_empty()
                    && g.state.pending_triggers.is_empty() && g.state.resolving.is_none() { break; }
                choose(&mut g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
            }
            assert_eq!(g.object_view(target).zone, Zone::Exile);
            g.restore(&saved).unwrap();
        }
    }
}
