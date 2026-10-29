//! Conservative candidate prior-schema audit; integration assigns a schema and dispatch.
use super::*;

fn project_before_type_slots(input: &str) -> Result<String, String> {
    fn visit(value: &serde_json::Value) -> Result<(), String> {
        match value {
            serde_json::Value::Array(values) => {
                for value in values {
                    visit(value)?;
                }
            }
            serde_json::Value::Object(map) => {
                if map.contains_key("target_type_slots") {
                    return Err("target type slots have no prior representation".into());
                }
                for value in map.values() {
                    visit(value)?;
                }
            }
            serde_json::Value::String(value)
                if matches!(value.as_str(), "ChooseTargetCardType" | "Uldaros Theorix") =>
            {
                return Err("type-slot choice or Uldaros has no prior baseline".into());
            }
            _ => (),
        }
        Ok(())
    }
    let value: serde_json::Value = serde_json::from_str(input).map_err(|e| e.to_string())?;
    visit(&value)?;
    Ok(input.to_owned())
}

fn prior_type_slots_owned_safe(state: &State) -> bool {
    // A complete inspection must include hidden copied recipes and all retained
    // nested packets. Reject opaque containers rather than claim equivalence.
    if state
        .objects
        .iter()
        .any(|o| o.card.identity() == "Uldaros Theorix")
        || state
            .explore_data
            .as_ref()
            .and_then(|data| data.last.as_ref())
            .is_some_and(|event| {
                event.explorer.card.identity() == "Uldaros Theorix"
                    || event
                        .revealed
                        .is_some_and(|card| card.card.identity() == "Uldaros Theorix")
            })
        || state
            .last_target_library_move
            .as_ref()
            .is_some_and(|event| event.card.identity() == "Uldaros Theorix")
        || state
            .library_top
            .as_ref()
            .and_then(|data| data.reveal.as_ref())
            .is_some_and(|event| event.revealed.card.identity() == "Uldaros Theorix")
        || state.announcing.is_some()
        || state.resolving.is_some()
        || state.stack_copy_data.is_some()
        || state.paradigm.is_some()
        || state.replacements.is_some()
        || state
            .zone_rules
            .as_ref()
            .is_some_and(|z| z.has_checkpoint())
        || state.entry_batch.is_some()
        || state.mana_announcement_stack.is_some()
        || state.linked_choices.is_some()
        || state.copy_recipes.is_some()
        || state.temporary_copies.is_some()
        || state
            .stack
            .iter()
            .chain(&state.pending_triggers)
            .chain(&state.ordering_triggers)
            .any(|item| {
                item.target_type_slots.is_some() || item.source_card.identity() == "Uldaros Theorix"
            })
    {
        return false;
    }
    if let Some(rollback) = &state.cast_rollback {
        if !rollback.events.is_empty()
            || rollback.replay.is_some()
            || rollback.capture.is_some()
            || !prior_type_slots_owned_safe(&rollback.before)
        {
            return false;
        }
    }
    // Other delayed stores are deliberately rejected pending typed inspection.
    state.delayed_artifact_copies.is_none()
        && state.delayed_returns.is_none()
        && state.delayed_attack_this_turn.is_none()
        && state.delayed_opponent_attack_until_next_turn.is_none()
        && state.delayed_main_mana.is_none()
        && state.delayed_upkeep_tokens.is_none()
        && state.earthbend_returns.is_none()
        && state.manifest.is_none()
        && state.mana_lots.is_none()
        && state.warp.is_none()
        && state.miracle.is_none()
        && state.mobilize.is_none()
        && state.delayed_sacrifices.is_none()
}

#[test]
fn type_slot_projection_rejects_new_action_and_nested_view() {
    let old = r#"[{"card":"Bear Cub"}]"#;
    assert_eq!(project_before_type_slots(old).unwrap(), old);
    assert!(
        project_before_type_slots(r#"[{"kind":"ChooseTargetCardType","kind_mask":1}]"#).is_err()
    );
    assert!(project_before_type_slots(r#"[{"item":{"target_type_slots":[1]}}]"#).is_err());
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        assert!(prior_type_slots_owned_safe(&g.state));
        let source = put(&mut g, player, "Bear Cub", Zone::Hand);
        g.state.objects[source as usize].card = Card::from_name("Uldaros Theorix").unwrap();
        g.begin_attack_rollback(player);
        g.state.objects[source as usize].card = Card::from_name("Bear Cub").unwrap();
        assert!(!prior_type_slots_owned_safe(&g.state));
        let checkpoint = g.snapshot();
        g.state.cast_rollback = None;
        assert!(prior_type_slots_owned_safe(&g.state));
        g.restore(&checkpoint).unwrap();
        assert!(!prior_type_slots_owned_safe(&g.state));
    }
}

#[test]
fn historical_uldaros_identity_is_rejected_even_after_the_object_changes() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let id = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        let object = g.object_ref(id);
        let card = Card::from_name("Uldaros Theorix").unwrap();
        g.state.explore_data.get_or_insert_default().last = Some(explore::ExploreEventView {
            sequence: 1,
            explorer: explore::ExplorerView {
                kinds: 1,
                object,
                card,
                controller: player,
            },
            revealed: None,
            destination: None,
            result: None,
            counter_added: false,
        });
        assert!(!prior_type_slots_owned_safe(&g.state));
        g.state.explore_data.as_mut().unwrap().last = None;
        g.state.last_target_library_move = Some(Box::new(TargetLibraryMovement {
            source: object,
            card,
            target_before: object,
            target_after: object,
            owner: player,
            destination: LibraryDestination::Top,
        }));
        assert!(!prior_type_slots_owned_safe(&g.state));
        g.state.last_target_library_move = None;
        g.state.library_top.get_or_insert_default().reveal =
            Some(library_top::LibraryRevealEvent {
                sequence: 1,
                source: object,
                player,
                revealed: explore::RevealedLibraryCard { object, card },
                drawn: None,
            });
        assert!(!prior_type_slots_owned_safe(&g.state));
        let snapshot = g.snapshot();
        g.state.library_top.as_mut().unwrap().reveal = None;
        assert!(prior_type_slots_owned_safe(&g.state));
        g.restore(&snapshot).unwrap();
        assert!(!prior_type_slots_owned_safe(&g.state));
    }
}
