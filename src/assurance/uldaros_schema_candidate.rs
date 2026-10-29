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
        || state.hits.iter().any(|event| event.source.facts.card.identity() == "Uldaros Theorix")
        || state.damage_choices.iter().any(|event| event.source.facts.card.identity() == "Uldaros Theorix")
        || state.temporary_abilities.as_ref().is_some_and(|rows| rows.iter().any(|row| row.granting_card.identity() == "Uldaros Theorix"))
        || state.self_granted_abilities.as_ref().is_some_and(|rows| rows.iter().any(|row| row.granting_card.identity() == "Uldaros Theorix"))
        || state.graveyard_borrowing.as_ref().is_some_and(|rows| rows.iter().any(|row|
            row.provider.identity() == "Uldaros Theorix" || row.definition.identity() == "Uldaros Theorix"))
        // Slot registry internals are intentionally opaque here.
        || state.borrowed_slots.is_some()
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

#[test]
fn retained_damage_grants_and_borrowed_registries_are_audited() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let id = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        let reference = g.object_ref(id);
        let card = Card::from_name("Uldaros Theorix").unwrap();
        let mut source = g.damage_source(id);
        source.facts.card = card;
        g.state.hits.push(DamageEvent {
            preventable: true,
            source,
            target: Target::Player(player.other()),
            amount: 1,
            combat: false,
        });
        assert!(!prior_type_slots_owned_safe(&g.state));
        g.state.hits.clear();
        g.state.damage_choices.push(DamageChoice {
            source,
            defender: None,
            blockers: vec![],
            remaining: 1,
            trample: false,
            prior_blockers_lethal: true,
        });
        assert!(!prior_type_slots_owned_safe(&g.state));
        g.state.damage_choices.clear();
        g.state.temporary_abilities =
            Some(Box::new(vec![temporary_abilities::TemporaryAbilityView {
                recipient: reference,
                grantor: reference,
                granting_card: card,
                ability: 0,
                grantor_controller: player,
                expires_turn: g.state.turn,
                permanent: false,
                timestamp: 1,
                origin: None,
            }]));
        assert!(!prior_type_slots_owned_safe(&g.state));
        g.state.temporary_abilities = None;
        g.state.self_granted_abilities = Some(Box::new(vec![self_granted::Row {
            recipient: reference,
            granting_card: card,
            kind: SelfGrantedAbilityKind::ControllerHandPower,
            timestamp: 1,
            effect_timestamp: None,
            expires_turn: None,
        }]));
        assert!(!prior_type_slots_owned_safe(&g.state));
        g.state.self_granted_abilities = None;
        g.state.graveyard_borrowing = Some(
            vec![graveyard_borrowing::Row {
                source: reference,
                provider: card,
                origin: None,
                controller: player,
                donor: reference,
                definition: Card::from_name("Bear Cub").unwrap(),
                first_slot: 0,
                width: 1,
                active: true,
            }]
            .into(),
        );
        assert!(!prior_type_slots_owned_safe(&g.state));
        let row = &mut g.state.graveyard_borrowing.as_mut().unwrap()[0];
        row.provider = Card::from_name("Bear Cub").unwrap();
        row.definition = card;
        assert!(!prior_type_slots_owned_safe(&g.state));
        g.state.graveyard_borrowing = None;
        g.state.borrowed_slots = Some(Default::default());
        assert!(!prior_type_slots_owned_safe(&g.state));
        let snapshot = g.snapshot();
        g.state.borrowed_slots = None;
        assert!(prior_type_slots_owned_safe(&g.state));
        g.restore(&snapshot).unwrap();
        assert!(!prior_type_slots_owned_safe(&g.state));
    }
}
