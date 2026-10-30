use super::*;

#[test]
fn copied_gideon_source_uses_trigger_time_opponent_and_obeys_prevention() {
    for actor in [Player::P0, Player::P1] {
        for prevent in [false, true] {
            let mut g = scenario(actor);
            let source = put(&mut g, actor.other(), "Bear Cub", Zone::Battlefield);
            g.apply_temporary_copy(
                g.object_ref(source),
                CopyRecipe::new(Card::from_name("Gideon the Oathless").unwrap()),
                CopyDeadline::EndOfTurn,
            );
            let preventer = put(&mut g, actor, "Forest", Zone::Battlefield);
            if prevent {
                let timestamp = g.next_role_timestamp();
                let source_ref = g.object_ref(preventer);
                let recipient = g.object_ref(source);
                g.state
                    .roles
                    .get_or_insert_default()
                    .source_linked_effects
                    .push(SourceLinkedEffectView {
                        source: source_ref,
                        recipient,
                        controller: actor,
                        kind: SourceLinkedEffectKind::PreventDamageBy,
                        timestamp,
                    });
            }
            let life = g.state.players[actor.index()].life;
            let other_life = g.state.players[actor.other().index()].life;
            g.create_tokens(actor, Card::test_index(5298), 1, false);
            g.prepare_triggers();
            assert!(g.state.stack.iter().any(|item| {
                item.source.is_some_and(|identity| identity.id == source)
                    && item.event_player == Some(actor)
            }));
            g.change_controller(g.object_ref(source), actor);
            let saved = g.snapshot();
            for _ in 0..2 {
                for _ in 0..96 {
                    g.prepare_triggers();
                    if g.state.pending == Pending::Priority
                        && g.state.stack.is_empty()
                        && g.state.pending_triggers.is_empty()
                        && g.state.resolving.is_none()
                        && g.state.replacements.is_none()
                    {
                        break;
                    }
                    choose(&mut g, |a| {
                        matches!(
                            a,
                            Action::Pass
                                | Action::PutTrigger { .. }
                                | Action::ChooseReplacement { .. }
                                | Action::ChooseDamagePrevention { .. }
                        )
                    });
                }
                assert!(g.state.stack.is_empty());
                assert_eq!(
                    g.state.players[actor.index()].life,
                    life - if prevent { 0 } else { 1 }
                );
                assert_eq!(g.state.players[actor.other().index()].life, other_life);
                g.restore(&saved).unwrap();
            }
        }
    }
}
