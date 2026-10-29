//! Conservative candidate prior-schema audit; integration assigns a schema and dispatch.
use super::*;

pub(super) fn project_before_type_slots(input: &str) -> Result<String, String> {
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

fn type_slot_new_card(card: Card) -> bool {
    card.identity() == "Uldaros Theorix"
}

fn type_slot_recipe_supported(recipe: &CopyRecipe) -> bool {
    if type_slot_new_card(recipe.definition) {
        return false;
    }
    match recipe.faces {
        Some(copies::CopyFaces::Double {
            physical_front,
            front,
            back,
            ..
        }) => ![physical_front, front, back]
            .into_iter()
            .any(type_slot_new_card),
        _ => true,
    }
}

fn type_slot_facts_supported(facts: &DamageFacts) -> bool {
    !type_slot_new_card(facts.card)
}

/// Inspect all packet fields carrying card definitions, recipes or LKI subtype
/// overlays. Reference-only and numeric fields cannot contain the new rules.
fn type_slot_packet_supported(item: &SpellView) -> bool {
    if !serde_json::to_string(item).is_ok_and(|json| project_before_type_slots(&json).is_ok())
        || type_slot_new_card(item.source_card)
        || !type_slot_facts_supported(&item.source_facts)
        || matches!(item.kind, StackKind::DelayedTokenCreation { token, .. } if type_slot_new_card(token))
        || item
            .source_copy_recipe
            .as_deref()
            .is_some_and(|r| !type_slot_recipe_supported(r))
        || item
            .captured_artifact_copy
            .as_deref()
            .is_some_and(|r| !type_slot_recipe_supported(r))
        || item
            .attachment_trigger
            .as_ref()
            .is_some_and(|v| type_slot_new_card(v.granting_card))
        || item
            .delayed_sacrifice
            .as_ref()
            .is_some_and(|v| type_slot_new_card(v.source_card))
        || item
            .delayed_warp_exile
            .as_ref()
            .is_some_and(|v| type_slot_new_card(v.source_card))
        || item
            .cast_event
            .as_ref()
            .is_some_and(|v| type_slot_new_card(v.card))
        || item
            .cast
            .and_then(|v| v.physical_card)
            .is_some_and(type_slot_new_card)
        || item.mana_copy_sources.as_ref().is_some_and(|sources| {
            sources.iter().any(|(v, _)| {
                type_slot_new_card(v.card)
                    || type_slot_new_card(v.definition)
                    || !type_slot_facts_supported(&v.facts)
            })
        })
        || item
            .copy_event
            .as_ref()
            .is_some_and(|event| !type_slot_packet_supported(&event.spell))
    {
        return false;
    }
    if let Some(event) = &item.event_object {
        if event
            .subtype_lki
            .as_ref()
            .is_some_and(|facts| !type_slot_facts_supported(facts))
        {
            return false;
        }
        match &event.facts {
            event_objects::EventObjectFacts::DamageActor(source)
                if !type_slot_facts_supported(&source.facts) =>
            {
                return false;
            }
            event_objects::EventObjectFacts::Copiable(recipe)
                if !type_slot_recipe_supported(recipe) =>
            {
                return false;
            }
            _ => {}
        }
    }
    true
}

pub(super) fn prior_type_slots_owned_safe(state: &State) -> bool {
    if state.outside_game.as_ref().is_some_and(|outside| outside.pools.iter().flatten().copied().any(type_slot_new_card) || outside.choice.is_some())
        || state.resolving.is_some() || state.announcing.is_some()
        || state.mana_announcement_stack.is_some() || state.replacements.is_some()
        || state.entry_batch.is_some() || state.zone_rules.as_ref().is_some_and(|z| z.has_checkpoint())
        || state.linked_choices.as_ref().is_some_and(|c| c.has_opaque_interaction())
        || state.objects.iter().any(|o| type_slot_new_card(o.card))
        || state.explore_data.as_ref().and_then(|data| data.last.as_ref()).is_some_and(|event|
            type_slot_new_card(event.explorer.card)
                || event.revealed.is_some_and(|card| type_slot_new_card(card.card)))
        || state.last_target_library_move.as_ref().is_some_and(|event| type_slot_new_card(event.card))
        || state.library_top.as_ref().and_then(|data| data.reveal.as_ref())
            .is_some_and(|event| type_slot_new_card(event.revealed.card))
        || state.temporary_abilities.as_ref().is_some_and(|rows| rows.iter().any(|row| type_slot_new_card(row.granting_card)))
        || state.self_granted_abilities.as_ref().is_some_and(|rows| rows.iter().any(|row| type_slot_new_card(row.granting_card)))
        || state.graveyard_borrowing.as_ref().is_some_and(|rows| rows.iter().any(|row|
            type_slot_new_card(row.provider) || type_slot_new_card(row.definition)))
        // Slots retain an optional immutable donor definition; all other slot
        // fields are object references or numeric acquisition/use identities.
        || state.borrowed_slots.as_ref().is_some_and(|rows| rows.iter().any(|row|
            row.definition.is_some_and(type_slot_new_card)))
        || state.warp.as_ref().is_some_and(|w| !w.delayed.is_empty())
        // Delayed sacrifice packets retain public source-card identity and
        // frozen damage facts, including hidden/copied subtype overlays.
        || state.delayed_sacrifices.as_ref().is_some_and(|rows| rows.iter().any(|row|
            type_slot_new_card(row.view.source_card) || !type_slot_facts_supported(&row.facts)))
        || state.mobilize.is_some() || state.miracle.is_some()
        || state.adventures.as_ref().is_some_and(|a| a.permissions.iter().any(|v| type_slot_new_card(v.card)))
        || state.plot.as_ref().and_then(|p| p.payment.as_ref()).is_some_and(|p| type_slot_new_card(p.source_card))
        || state.hits.iter().any(|hit| !type_slot_facts_supported(&hit.source.facts))
        || state.damage_choices.iter().any(|hit| !type_slot_facts_supported(&hit.source.facts))
        || state.manifest.as_ref().is_some_and(|manifest| {
            manifest.permanents.iter().chain(manifest.spells.iter())
                .any(|row| type_slot_new_card(row.underlying))
        })
        || state.copy_recipes.as_ref().is_some_and(|rows| rows.iter().any(|row| !type_slot_recipe_supported(&row.recipe)))
        || state.temporary_copies.as_ref().is_some_and(|rows| rows.iter().any(|row| {
            type_slot_new_card(row.physical_card)
                || row.original_recipe.as_ref().is_some_and(|r| !type_slot_recipe_supported(r))
                || row.layers.iter().any(|layer| !type_slot_recipe_supported(&layer.value.recipe))
        }))
    {
        return false;
    }
    if let Some(mana) = &state.mana_lots {
        let copy_safe = |source: &mana_lots::ManaCopySource| {
            !type_slot_new_card(source.card)
                && !type_slot_new_card(source.definition)
                && type_slot_facts_supported(&source.facts)
        };
        let quality_safe =
            |quality: &mana_lots::Quality| quality.copy_source.as_ref().is_none_or(copy_safe);
        if mana.trace.is_some()
            || mana.replay.is_some()
            || mana.choice.is_some()
            || mana
                .pools
                .iter()
                .flat_map(|rows| rows.iter())
                .any(|lot| !quality_safe(&lot.quality))
            || mana
                .effect_quality
                .as_ref()
                .is_some_and(|q| !quality_safe(q))
            || mana
                .announcement_quality
                .as_ref()
                .is_some_and(|(_, q)| !quality_safe(q))
        {
            return false;
        }
    }
    if let Some(rollback) = &state.cast_rollback
        && (!rollback.events.is_empty()
            || rollback.replay.is_some()
            || rollback.capture.is_some()
            || !prior_type_slots_owned_safe(&rollback.before))
    {
        return false;
    }
    state
        .stack
        .iter()
        .chain(&state.pending_triggers)
        .chain(&state.ordering_triggers)
        .chain(
            state
                .delayed_artifact_copies
                .as_deref()
                .into_iter()
                .flatten(),
        )
        .chain(
            state
                .delayed_attack_this_turn
                .as_deref()
                .into_iter()
                .flatten(),
        )
        .chain(
            state
                .delayed_opponent_attack_until_next_turn
                .as_deref()
                .into_iter()
                .flatten(),
        )
        .chain(state.delayed_main_mana.as_deref().into_iter().flatten())
        .chain(state.delayed_upkeep_tokens.as_deref().into_iter().flatten())
        .chain(
            state
                .stack_copy_data
                .as_ref()
                .into_iter()
                .flat_map(|s| s.packets()),
        )
        .chain(
            state
                .paradigm
                .as_ref()
                .into_iter()
                .flat_map(|s| s.packets()),
        )
        .chain(
            state
                .delayed_returns
                .as_deref()
                .into_iter()
                .flatten()
                .map(|r| &r.item),
        )
        .chain(
            state
                .earthbend_returns
                .as_deref()
                .into_iter()
                .flatten()
                .map(|r| &r.item),
        )
        .chain(
            state
                .manifest
                .as_ref()
                .into_iter()
                .flat_map(|s| s.deaths.iter().map(|d| &d.source)),
        )
        .all(type_slot_packet_supported)
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
        g.state.objects[source as usize].card = Card::test_index(4739);
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
        let card = Card::test_index(4739);
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
        let card = Card::test_index(4739);
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
        assert!(prior_type_slots_owned_safe(&g.state));
        let snapshot = g.snapshot();
        g.state.borrowed_slots = None;
        assert!(prior_type_slots_owned_safe(&g.state));
        g.restore(&snapshot).unwrap();
        assert!(prior_type_slots_owned_safe(&g.state));
    }
}

#[test]
fn nested_packets_and_opaque_permissions_are_rejected_across_rollback() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let id = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        let mut nested = g.activation_item(player, id, 0);
        nested.source_card = Card::test_index(4739);
        let mut outer = g.activation_item(player, id, 0);
        outer.copy_event = Some(Box::new(stack_copies::CopyEvent {
            spell: nested,
            amount: 1,
        }));
        g.state.pending_triggers.push(outer);
        assert!(!prior_type_slots_owned_safe(&g.state));
        g.begin_attack_rollback(player);
        g.state.pending_triggers.clear();
        assert!(!prior_type_slots_owned_safe(&g.state));
        let saved = g.snapshot();
        g.state.cast_rollback = None;
        assert!(prior_type_slots_owned_safe(&g.state));
        g.restore(&saved).unwrap();
        assert!(!prior_type_slots_owned_safe(&g.state));
        g.state.cast_rollback = None;
        g.state.adventures = Some(Default::default());
        assert!(prior_type_slots_owned_safe(&g.state));
        g.state.adventures = None;
        g.state.plot = Some(Default::default());
        assert!(prior_type_slots_owned_safe(&g.state));
    }
}

#[test]
fn normal_stack_packets_remain_representable_but_slots_and_hidden_nested_identity_do_not() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let id = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        let normal = g.activation_item(player, id, 0);
        assert!(type_slot_packet_supported(&normal));
        g.state.pending_triggers.push(normal.clone());
        assert!(prior_type_slots_owned_safe(&g.state));
        let mut typed = normal.clone();
        typed.target_type_slots = Some(Box::new(vec![1]));
        assert!(!type_slot_packet_supported(&typed));
        g.state.pending_triggers[0] = typed;
        assert!(!prior_type_slots_owned_safe(&g.state));
        let mut inner = normal.clone();
        inner.source_facts.card = Card::test_index(4739);
        let mut outer = normal;
        outer.copy_event = Some(Box::new(stack_copies::CopyEvent {
            spell: inner,
            amount: 1,
        }));
        g.state.pending_triggers[0] = outer;
        assert!(!prior_type_slots_owned_safe(&g.state));
        g.begin_attack_rollback(player);
        g.state.pending_triggers.clear();
        assert!(!prior_type_slots_owned_safe(&g.state));
        let checkpoint = g.snapshot();
        g.state.cast_rollback = None;
        assert!(prior_type_slots_owned_safe(&g.state));
        g.restore(&checkpoint).unwrap();
        assert!(!prior_type_slots_owned_safe(&g.state));
    }
}
