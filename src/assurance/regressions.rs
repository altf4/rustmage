//! Single-card cast/entry regression traces supplement independent rules cases.
//! These baselines record established behavior; they are not Oracle correctness
//! proofs for every ability, activation, attack trigger or future board state.
use super::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Trace {
    root: String,
    status: String,
    decisions: usize,
    zone: String,
    journal: String,
    observations: String,
}
fn digest(bytes: &[u8]) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash = (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}
fn fixture() -> Game {
    let mut g = scenario(Player::P0);
    for owner in [Player::P0, Player::P1] {
        for name in [
            "Bear Cub",
            "Healer's Hawk",
            "Gilded Lotus",
            "Phyrexian Arena",
            "Serra Angel",
            "Vampire Nighthawk",
            "Aegis Turtle",
            "Forest",
        ] {
            let id = put(&mut g, owner, name, Zone::Battlefield);
            if name == "Bear Cub" {
                g.state.objects[id as usize].tapped = true;
            }
        }
        for name in [
            "Bear Cub",
            "Shock",
            "Divination",
            "Forest",
            "Gilded Lotus",
            "Phyrexian Arena",
            "Agent Phil Coulson",
            "A.I.M. Scientists",
        ] {
            put(&mut g, owner, name, Zone::Graveyard);
        }
        for name in ["Bear Cub", "Shock", "Forest"] {
            put(&mut g, owner, name, Zone::Hand);
        }
        for name in ["Shock", "Island", "Bear Cub", "Forest"] {
            put(&mut g, owner, name, Zone::Library);
        }
    }
    g
}
fn project_battle_protectors(view: &mut Observation) {
    for object in &mut view.objects {
        object.battle_protector = None;
    }
}

#[test]
fn previous_schema_projection_removes_battle_protector() {
    let mut game = scenario(Player::P0);
    let id = put(&mut game, Player::P0, "Bear Cub", Zone::Battlefield);
    game.set_battle_protector(game.object_ref(id), Player::P1);
    let mut view = game.observe(Player::P0);
    let object = view.objects.iter().find(|object| object.id == id).unwrap();
    assert_eq!(object.battle_protector, Some(Player::P1));
    assert!(
        serde_json::to_value(object)
            .unwrap()
            .get("battle_protector")
            .is_some()
    );
    project_battle_protectors(&mut view);
    let object = view.objects.iter().find(|object| object.id == id).unwrap();
    assert!(
        serde_json::to_value(object)
            .unwrap()
            .get("battle_protector")
            .is_none()
    );
}

/// Split-second and combat cast restrictions have no prior-schema semantics.
/// False spell flags are skipped by serde, so compatible observations need no
/// byte rewriting. All unexpected explicit fields are rejected conservatively.
fn project_prior_split_second(input: &str) -> Result<String, String> {
    fn check(v: &serde_json::Value, keyword_choice: bool) -> Result<(), String> {
        match v {
            serde_json::Value::Array(rows) => {
                for row in rows {
                    check(row, keyword_choice)?;
                }
            }
            serde_json::Value::Object(map) => {
                if map.contains_key("split_second") {
                    return Err("split-second spell view has no prior-schema representation".into());
                }
                for (key, value) in map {
                    let keyword_field = matches!(
                        key.as_str(),
                        "keywords"
                            | "source_keywords"
                            | "granted_keywords"
                            | "gained_keywords"
                            | "keyword"
                            | "rules"
                            | "without"
                    );
                    if keyword_field && value.as_u64().is_some_and(|bits| bits & (1u64 << 31) != 0)
                    {
                        return Err(
                            "split-second keyword has no prior-schema representation".into()
                        );
                    }
                    if keyword_choice && key == "choices" {
                        let values = value.as_array().ok_or("invalid keyword choice")?;
                        if values
                            .iter()
                            .any(|v| v.as_u64().is_none_or(|bits| bits & (1u64 << 31) != 0))
                        {
                            return Err("split-second or invalid keyword choice has no prior-schema representation".into());
                        }
                    }
                    check(value, key == "keyword_choice")?;
                }
            }
            _ => (),
        }
        Ok(())
    }
    let parsed: serde_json::Value = serde_json::from_str(input).map_err(|e| e.to_string())?;
    check(&parsed, false)?;
    Ok(input.to_owned())
}

/// Providers must be rejected even when blanked or currently outside combat:
/// their future cast/activation restrictions cannot be represented in schema 177.
fn prior_split_second_owned_state_supported(game: &Game) -> bool {
    !game.state.objects.iter().any(|object| {
        object.zone == Zone::Battlefield
            && object.card.static_ability_groups().iter().any(|group| {
                group.fragments.iter().any(|fragment| {
                    matches!(
                        fragment,
                        crate::cards::StaticAbilityFragment::ControlledInstantSorcerySplitSecond
                            | crate::cards::StaticAbilityFragment::NoSpellsOrNonmanaDuringCombat
                    )
                })
            })
    }) && !game.nonmana_actions_prohibited()
}

#[test]
fn prior_split_second_projection_rejects_new_flags_and_keyword_bits() {
    let old = r#"[{"objects":[{"keywords":256}],"stack":[{"source_keywords":0}]}]"#;
    assert_eq!(project_prior_split_second(old).unwrap(), old);
    for input in [
        r#"[{"stack":[{"split_second":true}]}]"#,
        r#"[{"announcing":{"split_second":false}}]"#,
        r#"[{"objects":[{"keywords":2147483648}]}]"#,
        r#"[{"stack":[{"source_keywords":2147483649}]}]"#,
        r#"[{"keyword_effects":[{"rules":2147483648}]}]"#,
        r#"[{"keyword_choice":{"choices":[256,2147483648]}}]"#,
    ] {
        assert!(project_prior_split_second(input).is_err(), "{input}");
    }
    // Other u64 fields are not keyword bitsets.
    let large_id = r#"[{"id":2147483648,"keywords":0}]"#;
    assert_eq!(project_prior_split_second(large_id).unwrap(), large_id);
}

#[test]
fn prior_split_second_projection_handles_actual_views_for_both_players() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let id = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        let encoded =
            serde_json::to_string(&[g.observe(player), g.observe(player.other())]).unwrap();
        assert!(prior_split_second_owned_state_supported(&g));
        assert_eq!(project_prior_split_second(&encoded).unwrap(), encoded);
        g.state.objects[id as usize].gained_keywords = Keywords::SPLIT_SECOND;
        let encoded =
            serde_json::to_string(&[g.observe(player), g.observe(player.other())]).unwrap();
        assert!(project_prior_split_second(&encoded).is_err());
    }
}

#[test]
fn prior_split_second_projection_rejects_dormant_restriction_provider() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let id = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        // Install an actual provider definition in a fixture without requiring
        // an unsupported candidate card to be admitted for the unit test.
        g.state.objects[id as usize].card = Card::from_name("Samut, Tyrant of Naktamun").unwrap();
        assert!(!prior_split_second_owned_state_supported(&g));
    }
}

/// These schema-178 flags have no earlier representation. Only false values
/// may be removed; even hidden owned flags can affect later game behavior.
fn prior_schema178_history_owned_state_supported(game: &Game) -> bool {
    !game
        .state
        .objects
        .iter()
        .any(|o| o.dealt_combat_damage_this_incarnation)
        && !game
            .state
            .players
            .iter()
            .any(|p| p.legend_rule_ignored_this_turn)
}

fn project_prior_schema178_history(input: &str) -> Result<String, String> {
    fn check(value: &serde_json::Value) -> Result<(), String> {
        match value {
            serde_json::Value::Array(rows) => {
                for row in rows {
                    check(row)?;
                }
            }
            serde_json::Value::Object(map) => {
                for (key, value) in map {
                    if matches!(
                        key.as_str(),
                        "dealt_combat_damage_this_incarnation" | "legend_rule_ignored_this_turn"
                    ) && value != &serde_json::Value::Bool(false)
                    {
                        return Err(format!("{key} has no prior-schema representation"));
                    }
                    check(value)?;
                }
            }
            _ => (),
        }
        Ok(())
    }
    let parsed: serde_json::Value = serde_json::from_str(input).map_err(|e| e.to_string())?;
    check(&parsed)?;
    // Copy tokens without decoding strings, so escaped text and ordering stay
    // byte-for-byte identical to the trace. Serde's output is compact JSON.
    let bytes = input.as_bytes();
    let mut result = String::with_capacity(input.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] != b'"' {
            let ch = input[i..].chars().next().unwrap();
            result.push(ch);
            i += ch.len_utf8();
            continue;
        }
        let start = i;
        i += 1;
        while i < bytes.len() {
            if bytes[i] == b'\\' {
                i += 2;
            } else if bytes[i] == b'"' {
                i += 1;
                break;
            } else {
                i += 1;
            }
        }
        let key = &input[start..i];
        if matches!(
            key,
            "\"dealt_combat_damage_this_incarnation\"" | "\"legend_rule_ignored_this_turn\""
        ) && input[i..].starts_with(":false")
        {
            i += 6;
            if bytes.get(i) == Some(&b',') {
                i += 1;
            } else if bytes.get(i) == Some(&b'}') {
                if result.ends_with(',') {
                    result.pop();
                }
            } else {
                return Err("unexpected schema-178 field encoding".into());
            }
        } else {
            result.push_str(key);
        }
    }
    Ok(result)
}

#[test]
fn prior_schema178_history_projection_preserves_text_and_rejects_nonfalse() {
    let input = r#"[{"text":"escaped \\\"dealt_combat_damage_this_incarnation\\\":false,", "dealt_combat_damage_this_incarnation":false,"legend_rule_ignored_this_turn":false,"other":1}]"#;
    // Actual traces are compact serde output.
    let value: serde_json::Value = serde_json::from_str(input).unwrap();
    let compact = serde_json::to_string(&value).unwrap();
    let projected = project_prior_schema178_history(&compact).unwrap();
    let output: serde_json::Value = serde_json::from_str(&projected).unwrap();
    assert_eq!(output[0]["text"], value[0]["text"]);
    assert_eq!(output[0]["other"], value[0]["other"]);
    assert!(
        output[0]
            .get("dealt_combat_damage_this_incarnation")
            .is_none()
    );
    assert!(output[0].get("legend_rule_ignored_this_turn").is_none());
    for key in [
        "dealt_combat_damage_this_incarnation",
        "legend_rule_ignored_this_turn",
    ] {
        for v in ["true", "null", "1"] {
            assert!(project_prior_schema178_history(&format!("{{\"{key}\":{v}}}")).is_err());
        }
    }
}

#[test]
fn prior_schema178_history_projection_checks_both_views_and_hidden_owned_objects() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let visible = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        let hidden = put(&mut g, player, "Bear Cub", Zone::Hand);
        let views =
            || serde_json::to_string(&[g.observe(player), g.observe(player.other())]).unwrap();
        let encoded = views();
        assert!(prior_schema178_history_owned_state_supported(&g));
        let projected = project_prior_schema178_history(&encoded).unwrap();
        assert!(!projected.contains("\"dealt_combat_damage_this_incarnation\":"));
        assert!(!projected.contains("\"legend_rule_ignored_this_turn\":"));
        g.state.objects[visible as usize].dealt_combat_damage_this_incarnation = true;
        assert!(!prior_schema178_history_owned_state_supported(&g));
        assert!(
            project_prior_schema178_history(
                &serde_json::to_string(&[g.observe(player), g.observe(player.other())]).unwrap()
            )
            .is_err()
        );
        g.state.objects[visible as usize].dealt_combat_damage_this_incarnation = false;
        g.state.objects[hidden as usize].dealt_combat_damage_this_incarnation = true;
        assert!(!prior_schema178_history_owned_state_supported(&g));
        g.state.objects[hidden as usize].dealt_combat_damage_this_incarnation = false;
        g.state.players[player.index()].legend_rule_ignored_this_turn = true;
        assert!(!prior_schema178_history_owned_state_supported(&g));
        assert!(
            project_prior_schema178_history(
                &serde_json::to_string(&[g.observe(player), g.observe(player.other())]).unwrap()
            )
            .is_err()
        );
    }
}

/// Schema 181 adds frozen trigger players, discard-only ward payments, and
/// delayed exile links. Inactive fields are omitted by serde; preserve bytes.
fn project_prior_schema181(input: &str) -> Result<String, String> {
    fn check(value: &serde_json::Value) -> Result<(), String> {
        match value {
            serde_json::Value::Array(rows) => {
                for row in rows {
                    check(row)?;
                }
            }
            serde_json::Value::Object(map) => {
                for key in ["event_player", "discard_only", "exile_after"] {
                    if map.contains_key(key) {
                        return Err(format!("{key} has no schema-180 representation"));
                    }
                }
                for value in map.values() {
                    check(value)?;
                }
            }
            _ => (),
        }
        Ok(())
    }
    let value: serde_json::Value = serde_json::from_str(input).map_err(|e| e.to_string())?;
    check(&value)?;
    Ok(input.to_owned())
}

/// Schema 189 adds outside-game state and cumulative free-cast budgets.
/// Reject both kinds of new state when projecting to schema 188.
fn project_prior_outside(input: &str) -> Result<String, String> {
    fn check(value: &serde_json::Value) -> Result<(), String> {
        match value {
            serde_json::Value::Array(rows) => {
                for row in rows {
                    check(row)?;
                }
            }
            serde_json::Value::String(name)
                if matches!(
                    name.as_str(),
                    "Extrapolate the Impossible"
                        | "ChooseOutsideCard"
                        | "ChooseOutsideBasic"
                        | "CancelOutsideCard"
                        | "DeclineOutsideCards"
                        | "ChooseOutsideRevealed"
                        | "outside_game_choice"
                ) =>
            {
                return Err("outside-game rules have no earlier baseline".to_owned());
            }
            serde_json::Value::Object(map) => {
                if [
                    "outside_choice",
                    "owned_outside",
                    "known_outside",
                    "sideboard_counts",
                    "remaining_mana_value",
                ]
                .iter()
                .any(|key| map.contains_key(*key))
                {
                    return Err("outside-game observation has no earlier representation".to_owned());
                }
                for value in map.values() {
                    check(value)?;
                }
            }
            _ => (),
        }
        Ok(())
    }
    let value: serde_json::Value = serde_json::from_str(input).map_err(|e| e.to_string())?;
    check(&value)?;
    Ok(input.to_owned())
}

fn outside_new_card(card: Card) -> bool {
    matches!(card.identity(), "Extrapolate the Impossible")
}

fn outside_recipe_supported(recipe: &CopyRecipe) -> bool {
    if outside_new_card(recipe.definition) {
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
            .any(outside_new_card),
        _ => true,
    }
}

fn outside_facts_supported(facts: &DamageFacts) -> bool {
    // Schema 180 already represents additive Construct and other subtype facts.
    !outside_new_card(facts.card)
}

/// Inspect all packet fields carrying card definitions, recipes or LKI subtype
/// overlays. Reference-only and numeric fields cannot contain the new rules.
fn outside_packet_supported(item: &SpellView) -> bool {
    let Some(source) = item.source else {
        return false;
    };
    if !serde_json::to_string(item).is_ok_and(|json| project_prior_outside(&json).is_ok())
        || outside_new_card(source.card)
        || !outside_facts_supported(&source.facts)
        || matches!(item.kind, StackKind::DelayedTokenCreation { token, .. } if outside_new_card(token))
        || item
            .source_copy_recipe
            .as_deref()
            .is_some_and(|r| !outside_recipe_supported(r))
        || item
            .captured_artifact_copy
            .as_deref()
            .is_some_and(|r| !outside_recipe_supported(r))
        || item
            .attachment_trigger
            .as_ref()
            .is_some_and(|v| outside_new_card(v.granting_card))
        || item
            .delayed_sacrifice
            .as_ref()
            .is_some_and(|v| outside_new_card(v.source_card))
        || item
            .delayed_warp_exile
            .as_ref()
            .is_some_and(|v| outside_new_card(v.source_card))
        || item
            .cast_event
            .as_ref()
            .is_some_and(|v| outside_new_card(v.card))
        || item
            .cast
            .and_then(|v| v.physical_card)
            .is_some_and(outside_new_card)
        || item.mana_copy_sources.as_ref().is_some_and(|sources| {
            sources.iter().any(|(v, _)| {
                outside_new_card(v.card)
                    || outside_new_card(v.definition)
                    || !outside_facts_supported(&v.facts)
            })
        })
        || item
            .copy_event
            .as_ref()
            .is_some_and(|event| !outside_packet_supported(&event.spell))
    {
        return false;
    }
    if let Some(event) = &item.event_object {
        if event
            .subtype_lki
            .as_ref()
            .is_some_and(|facts| !outside_facts_supported(facts))
        {
            return false;
        }
        match &event.facts {
            event_objects::EventObjectFacts::DamageActor(source)
                if !outside_facts_supported(&source.facts) =>
            {
                return false;
            }
            event_objects::EventObjectFacts::Copiable(recipe)
                if !outside_recipe_supported(recipe) =>
            {
                return false;
            }
            _ => {}
        }
    }
    true
}

/// Conservatively reject opaque continuations, while checking copy layers and
/// retained packets by type. Rollback snapshots retain independent hidden state.
fn prior_outside_state_supported(state: &State) -> bool {
    if state.resolution_free_casts.as_ref().is_some_and(|offer| offer.remaining_mana_value.is_some())
        || state.outside_game.is_some() || state.resolving.is_some() || state.announcing.is_some()
        || state.mana_announcement_stack.is_some() || state.replacements.is_some()
        || state.entry_batch.is_some() || state.zone_rules.as_ref().is_some_and(|z| z.has_checkpoint())
        || state.linked_choices.as_ref().is_some_and(|c| c.has_opaque_interaction())
        || state.objects.iter().any(|o| outside_new_card(o.card))
        || state.explore_data.as_ref().and_then(|data| data.last.as_ref()).is_some_and(|event|
            outside_new_card(event.explorer.card)
                || event.revealed.is_some_and(|card| outside_new_card(card.card)))
        || state.last_target_library_move.as_ref().is_some_and(|event| outside_new_card(event.card))
        || state.library_top.as_ref().and_then(|data| data.reveal.as_ref())
            .is_some_and(|event| outside_new_card(event.revealed.card))
        || state.temporary_abilities.as_ref().is_some_and(|rows| rows.iter().any(|row| outside_new_card(row.granting_card)))
        || state.self_granted_abilities.as_ref().is_some_and(|rows| rows.iter().any(|row| outside_new_card(row.granting_card)))
        || state.graveyard_borrowing.as_ref().is_some_and(|rows| rows.iter().any(|row|
            outside_new_card(row.provider) || outside_new_card(row.definition)))
        // Slots retain an optional immutable donor definition; all other slot
        // fields are object references or numeric acquisition/use identities.
        || state.borrowed_slots.as_ref().is_some_and(|rows| rows.iter().any(|row|
            row.definition.is_some_and(outside_new_card)))
        || state.warp.as_ref().is_some_and(|w| !w.delayed.is_empty())
        // Delayed sacrifice packets retain public source-card identity and
        // frozen damage facts, including hidden/copied subtype overlays.
        || state.delayed_sacrifices.as_ref().is_some_and(|rows| rows.iter().any(|row|
            outside_new_card(row.view.source_card) || !outside_facts_supported(&row.facts)))
        || state.mobilize.is_some() || state.miracle.is_some()
        || state.adventures.as_ref().is_some_and(|a| a.permissions.iter().any(|v| outside_new_card(v.card)))
        || state.plot.as_ref().and_then(|p| p.payment.as_ref()).is_some_and(|p| outside_new_card(p.source_card))
        || state.hits.iter().any(|hit| !outside_facts_supported(&hit.source.facts))
        || state.damage_choices.iter().any(|hit| !outside_facts_supported(&hit.source.facts))
        || state.manifest.as_ref().is_some_and(|manifest| {
            manifest.permanents.iter().chain(manifest.spells.iter())
                .any(|row| outside_new_card(row.underlying))
        })
        || state.copy_recipes.as_ref().is_some_and(|rows| rows.iter().any(|row| !outside_recipe_supported(&row.recipe)))
        || state.temporary_copies.as_ref().is_some_and(|rows| rows.iter().any(|row| {
            outside_new_card(row.physical_card)
                || row.original_recipe.as_ref().is_some_and(|r| !outside_recipe_supported(r))
                || row.layers.iter().any(|layer| !outside_recipe_supported(&layer.value.recipe))
        }))
    {
        return false;
    }
    if let Some(mana) = &state.mana_lots {
        let copy_safe = |source: &mana_lots::ManaCopySource| {
            !outside_new_card(source.card)
                && !outside_new_card(source.definition)
                && outside_facts_supported(&source.facts)
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
            || !prior_outside_state_supported(&rollback.before))
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
        .all(outside_packet_supported)
}

/// Project schema 190's owned optional stack source to schema 189 without
/// changing JSON member order: regression hashes cover the raw bytes.
fn project_prior_schema190(input: &str) -> Result<String, String> {
    fn string_end(b: &[u8], mut i: usize) -> Result<usize, String> {
        i += 1;
        while i < b.len() {
            match b[i] {
                b'\\' => i += 2,
                b'"' => return Ok(i + 1),
                _ => i += 1,
            }
        }
        Err("unterminated JSON string".into())
    }
    fn value(b: &[u8], i: &mut usize) -> Result<String, String> {
        let start = *i;
        match b.get(*i).copied() {
            Some(b'"') => {
                *i = string_end(b, *i)?;
                let raw = std::str::from_utf8(&b[start..*i]).map_err(|e| e.to_string())?;
                let s: String = serde_json::from_str(raw).map_err(|e| e.to_string())?;
                if matches!(
                    s.as_str(),
                    "Tam, the Possibility"
                        | "Radiation"
                        | "InherentRule"
                        | "ProliferatePermanent"
                        | "ProliferatePlayer"
                        | "FinishProliferate"
                        | "ChooseSelfCounterReplacement"
                ) {
                    return Err("schema 190 rules lack a schema 189 representation".into());
                }
                Ok(raw.to_owned())
            }
            Some(b'[') => {
                *i += 1;
                let mut parts = Vec::new();
                while b.get(*i) != Some(&b']') {
                    if !parts.is_empty() {
                        if b.get(*i) != Some(&b',') {
                            return Err("invalid array".into());
                        }
                        *i += 1;
                    }
                    parts.push(value(b, i)?);
                }
                *i += 1;
                Ok(format!("[{}]", parts.join(",")))
            }
            Some(b'{') => {
                *i += 1;
                let mut fields: Vec<(String, String)> = Vec::new();
                while b.get(*i) != Some(&b'}') {
                    if !fields.is_empty() {
                        if b.get(*i) != Some(&b',') {
                            return Err("invalid object".into());
                        }
                        *i += 1;
                    }
                    let end = string_end(b, *i)?;
                    let key: String =
                        serde_json::from_slice(&b[*i..end]).map_err(|e| e.to_string())?;
                    *i = end;
                    if b.get(*i) != Some(&b':') {
                        return Err("invalid member".into());
                    }
                    *i += 1;
                    let v = value(b, i)?;
                    fields.push((key, v));
                }
                *i += 1;
                if fields.iter().any(|(k, _)| {
                    matches!(
                        k.as_str(),
                        "source_less_rule" | "other_counters" | "proliferate_choice"
                    )
                }) {
                    return Err("schema 190 public state lacks an earlier representation".into());
                }
                let packet = [
                    "source",
                    "kind",
                    "targets",
                    "controller",
                    "related_stack",
                    "source_keywords",
                ]
                .iter()
                .all(|k| fields.iter().any(|(name, _)| name == k));
                if packet {
                    let at = fields.iter().position(|(k, _)| k == "source").unwrap();
                    let source = fields.remove(at).1;
                    let source_value: serde_json::Value =
                        serde_json::from_str(&source).map_err(|e| e.to_string())?;
                    let source_map = source_value
                        .as_object()
                        .ok_or("source-less packet has no schema 189 representation")?;
                    if source_map.len() != 4 {
                        return Err("unknown stack source shape".into());
                    }
                    // Extract raw members without reserializing facts (whose member order matters).
                    let mut p = 1;
                    let bytes = source.as_bytes();
                    let mut parts = Vec::new();
                    while bytes.get(p) != Some(&b'}') {
                        if !parts.is_empty() {
                            if bytes.get(p) != Some(&b',') {
                                return Err("invalid source".into());
                            }
                            p += 1;
                        }
                        let end = string_end(bytes, p)?;
                        let key: String =
                            serde_json::from_slice(&bytes[p..end]).map_err(|e| e.to_string())?;
                        p = end + 1;
                        let v = value(bytes, &mut p)?;
                        parts.push((key, v));
                    }
                    let get = |key: &str| {
                        parts
                            .iter()
                            .find(|(k, _)| k == key)
                            .map(|(_, v)| v.clone())
                            .ok_or_else(|| format!("missing source {key}"))
                    };
                    let insert_before = |fields: &mut Vec<(String, String)>,
                                         anchor: &str,
                                         key: &str,
                                         v: String|
                     -> Result<(), String> {
                        let at = fields
                            .iter()
                            .position(|(k, _)| k == anchor)
                            .ok_or_else(|| format!("missing stack {anchor}"))?;
                        fields.insert(at, (key.to_owned(), v));
                        Ok(())
                    };
                    let anchor = if fields.iter().any(|(k, _)| k == "split_second") {
                        "split_second"
                    } else {
                        "source_keywords"
                    };
                    insert_before(&mut fields, anchor, "source_card", get("card")?)?;
                    insert_before(&mut fields, "related_stack", "source_facts", get("facts")?)?;
                    insert_before(&mut fields, "targets", "card", get("id")?)?;
                    insert_before(&mut fields, "targets", "object", get("object")?)?;
                }
                Ok(format!(
                    "{{{}}}",
                    fields
                        .into_iter()
                        .map(|(k, v)| format!("{}:{}", serde_json::to_string(&k).unwrap(), v))
                        .collect::<Vec<_>>()
                        .join(",")
                ))
            }
            Some(_) => {
                while *i < b.len() && !matches!(b[*i], b',' | b']' | b'}') {
                    *i += 1;
                }
                Ok(std::str::from_utf8(&b[start..*i])
                    .map_err(|e| e.to_string())?
                    .to_owned())
            }
            None => Err("unexpected end of JSON".into()),
        }
    }
    // Validate before raw traversal, and reject unexpected whitespace rather than
    // silently changing its byte representation.
    let _: serde_json::Value = serde_json::from_str(input).map_err(|e| e.to_string())?;
    let mut pos = 0;
    let result = value(input.as_bytes(), &mut pos)?;
    if pos != input.len() {
        return Err("unexpected trailing JSON".into());
    }
    Ok(result)
}

fn schema190_new_card(card: Card) -> bool {
    matches!(card.identity(), "Tam, the Possibility")
}

fn schema190_recipe_supported(recipe: &CopyRecipe) -> bool {
    if schema190_new_card(recipe.definition) {
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
            .any(schema190_new_card),
        _ => true,
    }
}

fn schema190_facts_supported(facts: &DamageFacts) -> bool {
    // Schema 180 already represents additive Construct and other subtype facts.
    !schema190_new_card(facts.card)
}

/// Inspect all packet fields carrying card definitions, recipes or LKI subtype
/// overlays. Reference-only and numeric fields cannot contain the new rules.
fn schema190_packet_supported(item: &SpellView) -> bool {
    if !serde_json::to_string(item).is_ok_and(|json| project_prior_schema190(&json).is_ok())
        || item.source_less_rule.is_some()
        || item.source.is_none_or(|source| {
            schema190_new_card(source.card) || !schema190_facts_supported(&source.facts)
        })
        || matches!(item.kind, StackKind::DelayedTokenCreation { token, .. } if schema190_new_card(token))
        || item
            .source_copy_recipe
            .as_deref()
            .is_some_and(|r| !schema190_recipe_supported(r))
        || item
            .captured_artifact_copy
            .as_deref()
            .is_some_and(|r| !schema190_recipe_supported(r))
        || item
            .attachment_trigger
            .as_ref()
            .is_some_and(|v| schema190_new_card(v.granting_card))
        || item
            .delayed_sacrifice
            .as_ref()
            .is_some_and(|v| schema190_new_card(v.source_card))
        || item
            .delayed_warp_exile
            .as_ref()
            .is_some_and(|v| schema190_new_card(v.source_card))
        || item
            .cast_event
            .as_ref()
            .is_some_and(|v| schema190_new_card(v.card))
        || item
            .cast
            .and_then(|v| v.physical_card)
            .is_some_and(schema190_new_card)
        || item.mana_copy_sources.as_ref().is_some_and(|sources| {
            sources.iter().any(|(v, _)| {
                schema190_new_card(v.card)
                    || schema190_new_card(v.definition)
                    || !schema190_facts_supported(&v.facts)
            })
        })
        || item
            .copy_event
            .as_ref()
            .is_some_and(|event| !schema190_packet_supported(&event.spell))
    {
        return false;
    }
    if let Some(event) = &item.event_object {
        if event
            .subtype_lki
            .as_ref()
            .is_some_and(|facts| !schema190_facts_supported(facts))
        {
            return false;
        }
        match &event.facts {
            event_objects::EventObjectFacts::DamageActor(source)
                if !schema190_facts_supported(&source.facts) =>
            {
                return false;
            }
            event_objects::EventObjectFacts::Copiable(recipe)
                if !schema190_recipe_supported(recipe) =>
            {
                return false;
            }
            _ => {}
        }
    }
    true
}

/// Conservatively reject opaque continuations, while checking copy layers and
/// retained packets by type. Rollback snapshots retain independent hidden state.
fn prior_schema190_state_supported(state: &State) -> bool {
    if state.players.iter().any(|player| player.other_player_counters.is_some())
        || state.outside_game.as_ref().is_some_and(|outside| outside.pools.iter().flatten().copied().any(schema190_new_card) || outside.choice.is_some() || outside.pending_entry.is_some() || outside.next_choice_id != 0 || outside.pool_versions != [0; 2])
        || state.resolving.is_some() || state.announcing.is_some()
        || state.mana_announcement_stack.is_some() || state.replacements.is_some()
        || state.entry_batch.is_some() || state.zone_rules.as_ref().is_some_and(|z| z.has_checkpoint())
        || state.linked_choices.as_ref().is_some_and(|c| c.has_opaque_interaction())
        || state.objects.iter().any(|o| schema190_new_card(o.card))
        || state.explore_data.as_ref().and_then(|data| data.last.as_ref()).is_some_and(|event|
            schema190_new_card(event.explorer.card)
                || event.revealed.is_some_and(|card| schema190_new_card(card.card)))
        || state.last_target_library_move.as_ref().is_some_and(|event| schema190_new_card(event.card))
        || state.library_top.as_ref().and_then(|data| data.reveal.as_ref())
            .is_some_and(|event| schema190_new_card(event.revealed.card))
        || state.temporary_abilities.as_ref().is_some_and(|rows| rows.iter().any(|row| schema190_new_card(row.granting_card)))
        || state.self_granted_abilities.as_ref().is_some_and(|rows| rows.iter().any(|row| schema190_new_card(row.granting_card)))
        || state.graveyard_borrowing.as_ref().is_some_and(|rows| rows.iter().any(|row|
            schema190_new_card(row.provider) || schema190_new_card(row.definition)))
        // Slots retain an optional immutable donor definition; all other slot
        // fields are object references or numeric acquisition/use identities.
        || state.borrowed_slots.as_ref().is_some_and(|rows| rows.iter().any(|row|
            row.definition.is_some_and(schema190_new_card)))
        || state.warp.as_ref().is_some_and(|w| !w.delayed.is_empty())
        // Delayed sacrifice packets retain public source-card identity and
        // frozen damage facts, including hidden/copied subtype overlays.
        || state.delayed_sacrifices.as_ref().is_some_and(|rows| rows.iter().any(|row|
            schema190_new_card(row.view.source_card) || !schema190_facts_supported(&row.facts)))
        || state.mobilize.is_some() || state.miracle.is_some()
        || state.adventures.as_ref().is_some_and(|a| a.permissions.iter().any(|v| schema190_new_card(v.card)))
        || state.plot.as_ref().and_then(|p| p.payment.as_ref()).is_some_and(|p| schema190_new_card(p.source_card))
        || state.hits.iter().any(|hit| !schema190_facts_supported(&hit.source.facts))
        || state.damage_choices.iter().any(|hit| !schema190_facts_supported(&hit.source.facts))
        || state.manifest.as_ref().is_some_and(|manifest| {
            manifest.permanents.iter().chain(manifest.spells.iter())
                .any(|row| schema190_new_card(row.underlying))
        })
        || state.copy_recipes.as_ref().is_some_and(|rows| rows.iter().any(|row| !schema190_recipe_supported(&row.recipe)))
        || state.temporary_copies.as_ref().is_some_and(|rows| rows.iter().any(|row| {
            schema190_new_card(row.physical_card)
                || row.original_recipe.as_ref().is_some_and(|r| !schema190_recipe_supported(r))
                || row.layers.iter().any(|layer| !schema190_recipe_supported(&layer.value.recipe))
        }))
    {
        return false;
    }
    if let Some(mana) = &state.mana_lots {
        let copy_safe = |source: &mana_lots::ManaCopySource| {
            !schema190_new_card(source.card)
                && !schema190_new_card(source.definition)
                && schema190_facts_supported(&source.facts)
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
            || !prior_schema190_state_supported(&rollback.before))
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
        .all(schema190_packet_supported)
}

fn schema194_new_card(card: Card) -> bool {
    matches!(
        card.identity(),
        "Ugin, Eye of the Storms" | "Jace Reawakened" | "The Aetherspark"
    )
}

fn schema194_recipe_supported(recipe: &CopyRecipe) -> bool {
    if schema194_new_card(recipe.definition) {
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
            .any(schema194_new_card),
        _ => true,
    }
}

fn schema194_facts_supported(facts: &DamageFacts) -> bool {
    // Schema 180 already represents additive Construct and other subtype facts.
    !schema194_new_card(facts.card)
}

/// Inspect all packet fields carrying card definitions, recipes or LKI subtype
/// overlays. Reference-only and numeric fields cannot contain the new rules.
fn schema194_packet_supported(item: &SpellView) -> bool {
    if !serde_json::to_string(item).is_ok_and(|json| project_prior_schema194(&json).is_ok())
        || item.source_less_rule.is_some()
        || item.source.is_none_or(|source| {
            schema194_new_card(source.card) || !schema194_facts_supported(&source.facts)
        })
        || matches!(item.kind, StackKind::DelayedTokenCreation { token, .. } if schema194_new_card(token))
        || item
            .source_copy_recipe
            .as_deref()
            .is_some_and(|r| !schema194_recipe_supported(r))
        || item
            .captured_artifact_copy
            .as_deref()
            .is_some_and(|r| !schema194_recipe_supported(r))
        || item
            .attachment_trigger
            .as_ref()
            .is_some_and(|v| schema194_new_card(v.granting_card))
        || item
            .delayed_sacrifice
            .as_ref()
            .is_some_and(|v| schema194_new_card(v.source_card))
        || item
            .delayed_warp_exile
            .as_ref()
            .is_some_and(|v| schema194_new_card(v.source_card))
        || item
            .cast_event
            .as_ref()
            .is_some_and(|v| schema194_new_card(v.card))
        || item
            .cast
            .and_then(|v| v.physical_card)
            .is_some_and(schema194_new_card)
        || item.mana_copy_sources.as_ref().is_some_and(|sources| {
            sources.iter().any(|(v, _)| {
                schema194_new_card(v.card)
                    || schema194_new_card(v.definition)
                    || !schema194_facts_supported(&v.facts)
            })
        })
        || item
            .copy_event
            .as_ref()
            .is_some_and(|event| !schema194_packet_supported(&event.spell))
    {
        return false;
    }
    if let Some(event) = &item.event_object {
        if event
            .subtype_lki
            .as_ref()
            .is_some_and(|facts| !schema194_facts_supported(facts))
        {
            return false;
        }
        match &event.facts {
            event_objects::EventObjectFacts::DamageActor(source)
                if !schema194_facts_supported(&source.facts) =>
            {
                return false;
            }
            event_objects::EventObjectFacts::Copiable(recipe)
                if !schema194_recipe_supported(recipe) =>
            {
                return false;
            }
            _ => {}
        }
    }
    true
}

/// Outside-entry replacement choices and continuations have no schema-191
/// representation. Keep all outside-owned states conservative until their
/// complete future behavior has an equivalent old-contract proof.
fn prior_schema194_state_supported(state: &State) -> bool {
    !state
        .objects
        .iter()
        .any(|object| schema194_new_card(object.card))
        && state.outside_game.as_ref().is_none_or(|outside| {
            !outside
                .pools
                .iter()
                .flatten()
                .copied()
                .any(schema194_new_card)
                && outside.choice.is_none()
                && outside.pending_entry.is_none()
        })
        && state.announcing.is_none()
        && state.mana_announcement_stack.is_none()
        && state.replacements.is_none()
        && state.copy_recipes.as_ref().is_none_or(|rows| {
            rows.iter()
                .all(|row| schema194_recipe_supported(&row.recipe))
        })
        && state.temporary_copies.as_ref().is_none_or(|rows| {
            rows.iter().all(|row| {
                !schema194_new_card(row.physical_card)
                    && row
                        .original_recipe
                        .as_ref()
                        .is_none_or(schema194_recipe_supported)
                    && row
                        .layers
                        .iter()
                        .all(|layer| schema194_recipe_supported(&layer.value.recipe))
            })
        })
        && state.borrowed_slots.as_ref().is_none_or(|rows| {
            rows.iter()
                .all(|row| row.definition.is_none_or(|card| !schema194_new_card(card)))
        })
        && state.temporary_abilities.as_ref().is_none_or(|rows| {
            rows.iter()
                .all(|row| !schema194_new_card(row.granting_card))
        })
        && state.self_granted_abilities.as_ref().is_none_or(|rows| {
            rows.iter()
                .all(|row| !schema194_new_card(row.granting_card))
        })
        && state.resolving.is_none()
        && state.entry_batch.is_none()
        && state
            .zone_rules
            .as_ref()
            .is_none_or(|rules| !rules.has_checkpoint())
        && !state
            .cast_rollback
            .as_ref()
            .is_some_and(|rollback| !prior_schema194_state_supported(&rollback.before))
        && {
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
                .all(schema194_packet_supported)
        }
}

fn schema193_new_card(card: Card) -> bool {
    matches!(card.identity(), "Elspeth, Storm Slayer")
}

fn schema193_recipe_supported(recipe: &CopyRecipe) -> bool {
    if schema193_new_card(recipe.definition) {
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
            .any(schema193_new_card),
        _ => true,
    }
}

fn schema193_facts_supported(facts: &DamageFacts) -> bool {
    // Schema 180 already represents additive Construct and other subtype facts.
    !schema193_new_card(facts.card)
}

/// Inspect all packet fields carrying card definitions, recipes or LKI subtype
/// overlays. Reference-only and numeric fields cannot contain the new rules.
fn schema193_packet_supported(item: &SpellView) -> bool {
    if !serde_json::to_string(item).is_ok_and(|json| project_prior_schema193(&json).is_ok())
        || item.source_less_rule.is_some()
        || item.source.is_none_or(|source| {
            schema193_new_card(source.card) || !schema193_facts_supported(&source.facts)
        })
        || matches!(item.kind, StackKind::DelayedTokenCreation { token, .. } if schema193_new_card(token))
        || item
            .source_copy_recipe
            .as_deref()
            .is_some_and(|r| !schema193_recipe_supported(r))
        || item
            .captured_artifact_copy
            .as_deref()
            .is_some_and(|r| !schema193_recipe_supported(r))
        || item
            .attachment_trigger
            .as_ref()
            .is_some_and(|v| schema193_new_card(v.granting_card))
        || item
            .delayed_sacrifice
            .as_ref()
            .is_some_and(|v| schema193_new_card(v.source_card))
        || item
            .delayed_warp_exile
            .as_ref()
            .is_some_and(|v| schema193_new_card(v.source_card))
        || item
            .cast_event
            .as_ref()
            .is_some_and(|v| schema193_new_card(v.card))
        || item
            .cast
            .and_then(|v| v.physical_card)
            .is_some_and(schema193_new_card)
        || item.mana_copy_sources.as_ref().is_some_and(|sources| {
            sources.iter().any(|(v, _)| {
                schema193_new_card(v.card)
                    || schema193_new_card(v.definition)
                    || !schema193_facts_supported(&v.facts)
            })
        })
        || item
            .copy_event
            .as_ref()
            .is_some_and(|event| !schema193_packet_supported(&event.spell))
    {
        return false;
    }
    if let Some(event) = &item.event_object {
        if event
            .subtype_lki
            .as_ref()
            .is_some_and(|facts| !schema193_facts_supported(facts))
        {
            return false;
        }
        match &event.facts {
            event_objects::EventObjectFacts::DamageActor(source)
                if !schema193_facts_supported(&source.facts) =>
            {
                return false;
            }
            event_objects::EventObjectFacts::Copiable(recipe)
                if !schema193_recipe_supported(recipe) =>
            {
                return false;
            }
            _ => {}
        }
    }
    true
}

/// Outside-entry replacement choices and continuations have no schema-191
/// representation. Keep all outside-owned states conservative until their
/// complete future behavior has an equivalent old-contract proof.
fn prior_schema193_state_supported(state: &State) -> bool {
    !state
        .objects
        .iter()
        .any(|object| schema193_new_card(object.card))
        && state.outside_game.as_ref().is_none_or(|outside| {
            !outside
                .pools
                .iter()
                .flatten()
                .copied()
                .any(schema193_new_card)
                && outside.choice.is_none()
                && outside.pending_entry.is_none()
        })
        && state.announcing.is_none()
        && state.mana_announcement_stack.is_none()
        && state.replacements.is_none()
        && state.copy_recipes.as_ref().is_none_or(|rows| {
            rows.iter()
                .all(|row| schema193_recipe_supported(&row.recipe))
        })
        && state.temporary_copies.as_ref().is_none_or(|rows| {
            rows.iter().all(|row| {
                !schema193_new_card(row.physical_card)
                    && row
                        .original_recipe
                        .as_ref()
                        .is_none_or(schema193_recipe_supported)
                    && row
                        .layers
                        .iter()
                        .all(|layer| schema193_recipe_supported(&layer.value.recipe))
            })
        })
        && state.borrowed_slots.as_ref().is_none_or(|rows| {
            rows.iter()
                .all(|row| row.definition.is_none_or(|card| !schema193_new_card(card)))
        })
        && state.temporary_abilities.as_ref().is_none_or(|rows| {
            rows.iter()
                .all(|row| !schema193_new_card(row.granting_card))
        })
        && state.self_granted_abilities.as_ref().is_none_or(|rows| {
            rows.iter()
                .all(|row| !schema193_new_card(row.granting_card))
        })
        && state.roles.as_ref().is_none_or(|roles| {
            !roles.keyword_effects.iter().any(|effect| {
                matches!(
                    effect.origin,
                    KeywordEffectOrigin::ResolutionUntilNextTurn { .. }
                )
            })
        })
        && state.resolving.as_ref().is_none_or(|resolution| {
            resolution.remembered_creatures.is_empty()
                && schema193_packet_supported(&resolution.item)
                && !resolution.program.iter().any(|instruction| {
                    matches!(
                        instruction,
                        Instruction::PutCreatureCountersAndRemember { .. }
                            | Instruction::GrantRememberedCreaturesUntilNextTurn(_)
                    )
                })
        })
        && state.entry_batch.is_none()
        && state
            .zone_rules
            .as_ref()
            .is_none_or(|rules| !rules.has_checkpoint())
        && !state
            .cast_rollback
            .as_ref()
            .is_some_and(|rollback| !prior_schema193_state_supported(&rollback.before))
        && {
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
                .all(schema193_packet_supported)
        }
}

fn project_prior_schema194(input: &str) -> Result<String, String> {
    fn check(value: &serde_json::Value) -> Result<(), String> {
        match value {
            serde_json::Value::String(text)
                if matches!(text.as_str(), "HandPlot" | "ExileFreeThisTurn") =>
            {
                Err("new plotted-hand or free-exile choice has no schema 193 representation".into())
            }
            serde_json::Value::Array(rows) => rows.iter().try_for_each(check),
            serde_json::Value::Object(fields) => {
                if fields.contains_key("HandPlot") || fields.contains_key("ExileFreeThisTurn") {
                    return Err(
                        "new plotted-hand or free-exile choice has no schema 193 representation"
                            .into(),
                    );
                }
                if let Some(filter) = fields.get("filter") {
                    let encoded = filter.to_string();
                    if encoded.contains("\"Colorless\"") || encoded.contains("\"Nonland\"") {
                        return Err("new search filter has no schema 193 representation".into());
                    }
                }
                fields.values().try_for_each(check)
            }
            _ => Ok(()),
        }
    }
    check(&serde_json::from_str(input).map_err(|e| e.to_string())?)?;
    Ok(input.to_owned())
}

fn project_prior_schema193(input: &str) -> Result<String, String> {
    fn check(value: &serde_json::Value) -> Result<(), String> {
        match value {
            serde_json::Value::String(text) if text == "ResolutionUntilNextTurn" => {
                Err("next-turn keyword grant has no schema 192 representation".into())
            }
            serde_json::Value::Array(rows) => rows.iter().try_for_each(check),
            serde_json::Value::Object(fields) => {
                if fields.contains_key("ResolutionUntilNextTurn") {
                    return Err("next-turn keyword grant has no schema 192 representation".into());
                }
                fields.values().try_for_each(check)
            }
            _ => Ok(()),
        }
    }
    check(&serde_json::from_str(input).map_err(|e| e.to_string())?)?;
    Ok(input.to_owned())
}

fn prior_schema192_state_supported(state: &State) -> bool {
    state.outside_game.is_none()
        // Entry batches and active zone replacement decisions can retain an
        // older complete State. Passive zone rules have no checkpoint.
        && state.entry_batch.is_none()
        && state.zone_rules.as_ref().is_none_or(|rules| !rules.has_checkpoint())
        && !matches!(state.pending, Pending::OutsideEntryReplacement { .. })
        && !state
            .cast_rollback
            .as_ref()
            .is_some_and(|rollback| !prior_schema192_state_supported(&rollback.before))
}

fn project_prior_schema192(input: &str) -> Result<String, String> {
    fn check(value: &serde_json::Value) -> Result<(), String> {
        match value {
            serde_json::Value::String(text)
                if text == "outside_entry_replacement"
                    || text == "ChooseOutsideEntryReplacement"
                    || text == "ChooseOutsideSelfReplacement" =>
            {
                Err("outside-entry replacement has no schema 191 representation".into())
            }
            serde_json::Value::Array(rows) => {
                for row in rows {
                    check(row)?;
                }
                Ok(())
            }
            serde_json::Value::Object(fields) => {
                for (key, value) in fields {
                    if key == "outside_entry_replacement"
                        || key == "ChooseOutsideEntryReplacement"
                        || key == "ChooseOutsideSelfReplacement"
                    {
                        return Err(
                            "outside-entry replacement has no schema 191 representation".into()
                        );
                    }
                    check(value)?;
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }
    let value: serde_json::Value = serde_json::from_str(input).map_err(|e| e.to_string())?;
    check(&value)?;
    Ok(input.to_owned())
}

/// Reject wire data introduced after published schema 190 without altering old bytes.
fn project_prior_schema191(input: &str) -> Result<String, String> {
    fn check(v: &serde_json::Value) -> Result<(), String> {
        match v {
            serde_json::Value::String(s)
                if matches!(
                    s.as_str(),
                    "Sphinx's Approach"
                        | "Emrakul, the Exigent Doom"
                        | "SelectResolutionExile"
                        | "ExileSourceAndNamed"
                        | "Prevent"
                        | "GrantTwoColorlessUntilSourceCast"
                        | "CounterRelatedUnlessSacrifice"
                        | "GrantLandManaUntilSourceCast"
                ) =>
            {
                return Err("schema 191 behavior has no schema 190 representation".into());
            }
            serde_json::Value::Array(rows) => {
                for row in rows {
                    check(row)?;
                }
            }
            serde_json::Value::Object(map) => {
                for (key, value) in map {
                    if matches!(
                        key.as_str(),
                        "SelectResolutionExile"
                            | "ExileSourceAndNamed"
                            | "Prevent"
                            | "GrantTwoColorlessUntilSourceCast"
                            | "CounterRelatedUnlessSacrifice"
                            | "GrantLandManaUntilSourceCast"
                    ) {
                        return Err("schema 191 behavior has no schema 190 representation".into());
                    }
                    check(value)?;
                }
            }
            _ => (),
        }
        Ok(())
    }
    let value: serde_json::Value =
        serde_json::from_str(input).map_err(|error| error.to_string())?;
    check(&value)?;
    Ok(input.to_owned())
}

fn schema191_new_card(card: Card) -> bool {
    matches!(
        card.identity(),
        "Sphinx's Approach" | "Emrakul, the Exigent Doom"
    )
}

fn schema191_recipe_supported(recipe: &CopyRecipe) -> bool {
    if schema191_new_card(recipe.definition) {
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
            .any(schema191_new_card),
        _ => true,
    }
}

fn schema191_facts_supported(facts: &DamageFacts) -> bool {
    // Schema 180 already represents additive Construct and other subtype facts.
    !schema191_new_card(facts.card)
}

/// Inspect all packet fields carrying card definitions, recipes or LKI subtype
/// overlays. Reference-only and numeric fields cannot contain the new rules.
fn schema191_packet_supported(item: &SpellView) -> bool {
    if !serde_json::to_string(item).is_ok_and(|json| project_prior_schema191(&json).is_ok())
        || !matches!(
            (item.source, item.source_less_rule),
            (Some(_), None) | (None, Some(SourceLessRule::Radiation))
        )
        || item.source.is_some_and(|source| {
            schema191_new_card(source.card) || !schema191_facts_supported(&source.facts)
        })
        || matches!(item.kind, StackKind::DelayedTokenCreation { token, .. } if schema191_new_card(token))
        || item
            .source_copy_recipe
            .as_deref()
            .is_some_and(|r| !schema191_recipe_supported(r))
        || item
            .captured_artifact_copy
            .as_deref()
            .is_some_and(|r| !schema191_recipe_supported(r))
        || item
            .attachment_trigger
            .as_ref()
            .is_some_and(|v| schema191_new_card(v.granting_card))
        || item
            .delayed_sacrifice
            .as_ref()
            .is_some_and(|v| schema191_new_card(v.source_card))
        || item
            .delayed_warp_exile
            .as_ref()
            .is_some_and(|v| schema191_new_card(v.source_card))
        || item
            .cast_event
            .as_ref()
            .is_some_and(|v| schema191_new_card(v.card))
        || item
            .cast
            .and_then(|v| v.physical_card)
            .is_some_and(schema191_new_card)
        || item.mana_copy_sources.as_ref().is_some_and(|sources| {
            sources.iter().any(|(v, _)| {
                schema191_new_card(v.card)
                    || schema191_new_card(v.definition)
                    || !schema191_facts_supported(&v.facts)
            })
        })
        || item
            .copy_event
            .as_ref()
            .is_some_and(|event| !schema191_packet_supported(&event.spell))
    {
        return false;
    }
    if let Some(event) = &item.event_object {
        if event
            .subtype_lki
            .as_ref()
            .is_some_and(|facts| !schema191_facts_supported(facts))
        {
            return false;
        }
        match &event.facts {
            event_objects::EventObjectFacts::DamageActor(source)
                if !schema191_facts_supported(&source.facts) =>
            {
                return false;
            }
            event_objects::EventObjectFacts::Copiable(recipe)
                if !schema191_recipe_supported(recipe) =>
            {
                return false;
            }
            _ => {}
        }
    }
    true
}

/// Conservatively reject opaque continuations, while checking copy layers and
/// retained packets by type. Rollback snapshots retain independent hidden state.
fn prior_schema191_state_supported(state: &State) -> bool {
    if state.objects.iter().enumerate().any(|(id, object)| object.is_spell_copy()
        && !(object.zone == Zone::Outside && state.free_tokens.contains(&(id as CardId))))
        || state.roles.as_ref().is_some_and(|roles| !roles.exile_grant_cast_history.is_empty() || roles.source_linked_effects.iter().any(|row| row.kind == SourceLinkedEffectKind::GrantTwoColorlessUntilSourceCast))
        || state.zone_rules.as_ref().is_some_and(|rules| !rules.schema191_passive_safe(schema191_new_card))
        || state.outside_game.as_ref().is_some_and(|outside| outside.pools.iter().flatten().copied().any(schema191_new_card) || outside.choice.is_some() || outside.pending_entry.is_some() || outside.next_choice_id != 0 || outside.pool_versions != [0; 2])
        || state.resolving.is_some() || state.announcing.is_some()
        || state.mana_announcement_stack.is_some() || state.replacements.is_some()
        || state.entry_batch.is_some() || state.zone_rules.as_ref().is_some_and(|z| z.has_checkpoint())
        || state.linked_choices.as_ref().is_some_and(|c| c.has_opaque_interaction())
        || state.objects.iter().any(|o| schema191_new_card(o.card))
        || state.explore_data.as_ref().and_then(|data| data.last.as_ref()).is_some_and(|event|
            schema191_new_card(event.explorer.card)
                || event.revealed.is_some_and(|card| schema191_new_card(card.card)))
        || state.last_target_library_move.as_ref().is_some_and(|event| schema191_new_card(event.card))
        || state.library_top.as_ref().and_then(|data| data.reveal.as_ref())
            .is_some_and(|event| schema191_new_card(event.revealed.card))
        || state.temporary_abilities.as_ref().is_some_and(|rows| rows.iter().any(|row| schema191_new_card(row.granting_card)))
        || state.self_granted_abilities.as_ref().is_some_and(|rows| rows.iter().any(|row| schema191_new_card(row.granting_card)))
        || state.graveyard_borrowing.as_ref().is_some_and(|rows| rows.iter().any(|row|
            schema191_new_card(row.provider) || schema191_new_card(row.definition)))
        // Slots retain an optional immutable donor definition; all other slot
        // fields are object references or numeric acquisition/use identities.
        || state.borrowed_slots.as_ref().is_some_and(|rows| rows.iter().any(|row|
            row.definition.is_some_and(schema191_new_card)))
        || state.warp.as_ref().is_some_and(|w| !w.delayed.is_empty())
        // Delayed sacrifice packets retain public source-card identity and
        // frozen damage facts, including hidden/copied subtype overlays.
        || state.delayed_sacrifices.as_ref().is_some_and(|rows| rows.iter().any(|row|
            schema191_new_card(row.view.source_card) || !schema191_facts_supported(&row.facts)))
        || state.mobilize.is_some() || state.miracle.is_some()
        || state.adventures.as_ref().is_some_and(|a| a.permissions.iter().any(|v| schema191_new_card(v.card)))
        || state.plot.as_ref().and_then(|p| p.payment.as_ref()).is_some_and(|p| schema191_new_card(p.source_card))
        || state.hits.iter().any(|hit| !schema191_facts_supported(&hit.source.facts))
        || state.damage_choices.iter().any(|hit| !schema191_facts_supported(&hit.source.facts))
        || state.manifest.as_ref().is_some_and(|manifest| {
            manifest.permanents.iter().chain(manifest.spells.iter())
                .any(|row| schema191_new_card(row.underlying))
        })
        || state.copy_recipes.as_ref().is_some_and(|rows| rows.iter().any(|row| !schema191_recipe_supported(&row.recipe)))
        || state.temporary_copies.as_ref().is_some_and(|rows| rows.iter().any(|row| {
            schema191_new_card(row.physical_card)
                || row.original_recipe.as_ref().is_some_and(|r| !schema191_recipe_supported(r))
                || row.layers.iter().any(|layer| !schema191_recipe_supported(&layer.value.recipe))
        }))
    {
        return false;
    }
    if let Some(mana) = &state.mana_lots {
        let copy_safe = |source: &mana_lots::ManaCopySource| {
            !schema191_new_card(source.card)
                && !schema191_new_card(source.definition)
                && schema191_facts_supported(&source.facts)
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
            || !prior_schema191_state_supported(&rollback.before))
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
        .all(schema191_packet_supported)
}

/// Schema 188 adds Cryomancer's public target limit and new Null/Sanctum rules.
/// Reject new retained behavior rather than fabricating a schema-187 baseline.
fn project_prior_schema188(input: &str) -> Result<String, String> {
    fn check(value: &serde_json::Value) -> Result<(), String> {
        match value {
            serde_json::Value::Array(rows) => {
                for row in rows {
                    check(row)?;
                }
            }
            serde_json::Value::String(name)
                if matches!(
                    name.as_str(),
                    "Seasoned Cryomancer" | "Null Summoner" | "Sanctum Lurker"
                ) =>
            {
                return Err(
                    "Null, Sanctum, or Cryomancer rules have no schema-187 baseline".to_owned(),
                );
            }
            serde_json::Value::Object(map) => {
                if ["reflexive_target_limit"]
                    .iter()
                    .any(|key| map.contains_key(*key))
                {
                    return Err("Reflexive target limit has no earlier representation".to_owned());
                }
                for value in map.values() {
                    check(value)?;
                }
            }
            _ => (),
        }
        Ok(())
    }
    let value: serde_json::Value = serde_json::from_str(input).map_err(|e| e.to_string())?;
    check(&value)?;
    Ok(input.to_owned())
}

fn schema188_new_card(card: Card) -> bool {
    matches!(
        card.identity(),
        "Seasoned Cryomancer" | "Null Summoner" | "Sanctum Lurker"
    )
}

fn schema188_recipe_supported(recipe: &CopyRecipe) -> bool {
    if schema188_new_card(recipe.definition) {
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
            .any(schema188_new_card),
        _ => true,
    }
}

fn schema188_facts_supported(facts: &DamageFacts) -> bool {
    // Schema 180 already represents additive Construct and other subtype facts.
    !schema188_new_card(facts.card)
}

/// Inspect all packet fields carrying card definitions, recipes or LKI subtype
/// overlays. Reference-only and numeric fields cannot contain the new rules.
fn schema188_packet_supported(item: &SpellView) -> bool {
    let Some(source) = item.source else {
        return false;
    };
    if !serde_json::to_string(item).is_ok_and(|json| project_prior_schema188(&json).is_ok())
        || schema188_new_card(source.card)
        || !schema188_facts_supported(&source.facts)
        || matches!(item.kind, StackKind::DelayedTokenCreation { token, .. } if schema188_new_card(token))
        || item
            .source_copy_recipe
            .as_deref()
            .is_some_and(|r| !schema188_recipe_supported(r))
        || item
            .captured_artifact_copy
            .as_deref()
            .is_some_and(|r| !schema188_recipe_supported(r))
        || item
            .attachment_trigger
            .as_ref()
            .is_some_and(|v| schema188_new_card(v.granting_card))
        || item
            .delayed_sacrifice
            .as_ref()
            .is_some_and(|v| schema188_new_card(v.source_card))
        || item
            .delayed_warp_exile
            .as_ref()
            .is_some_and(|v| schema188_new_card(v.source_card))
        || item
            .cast_event
            .as_ref()
            .is_some_and(|v| schema188_new_card(v.card))
        || item
            .cast
            .and_then(|v| v.physical_card)
            .is_some_and(schema188_new_card)
        || item.mana_copy_sources.as_ref().is_some_and(|sources| {
            sources.iter().any(|(v, _)| {
                schema188_new_card(v.card)
                    || schema188_new_card(v.definition)
                    || !schema188_facts_supported(&v.facts)
            })
        })
        || item
            .copy_event
            .as_ref()
            .is_some_and(|event| !schema188_packet_supported(&event.spell))
    {
        return false;
    }
    if let Some(event) = &item.event_object {
        if event
            .subtype_lki
            .as_ref()
            .is_some_and(|facts| !schema188_facts_supported(facts))
        {
            return false;
        }
        match &event.facts {
            event_objects::EventObjectFacts::DamageActor(source)
                if !schema188_facts_supported(&source.facts) =>
            {
                return false;
            }
            event_objects::EventObjectFacts::Copiable(recipe)
                if !schema188_recipe_supported(recipe) =>
            {
                return false;
            }
            _ => {}
        }
    }
    true
}

/// Conservatively reject opaque continuations, while checking copy layers and
/// retained packets by type. Rollback snapshots retain independent hidden state.
fn prior_schema188_state_supported(state: &State) -> bool {
    if state.linked_choices.as_ref().is_some_and(|data| !data.null_cast_links.is_empty())
        || state.resolving.is_some() || state.announcing.is_some()
        || state.mana_announcement_stack.is_some() || state.replacements.is_some()
        || state.entry_batch.is_some() || state.zone_rules.as_ref().is_some_and(|z| z.has_checkpoint())
        || state.linked_choices.as_ref().is_some_and(|c| c.has_opaque_interaction())
        || state.objects.iter().any(|o| schema188_new_card(o.card))
        || state.explore_data.as_ref().and_then(|data| data.last.as_ref()).is_some_and(|event|
            schema188_new_card(event.explorer.card)
                || event.revealed.is_some_and(|card| schema188_new_card(card.card)))
        || state.last_target_library_move.as_ref().is_some_and(|event| schema188_new_card(event.card))
        || state.library_top.as_ref().and_then(|data| data.reveal.as_ref())
            .is_some_and(|event| schema188_new_card(event.revealed.card))
        || state.temporary_abilities.as_ref().is_some_and(|rows| rows.iter().any(|row| schema188_new_card(row.granting_card)))
        || state.self_granted_abilities.as_ref().is_some_and(|rows| rows.iter().any(|row| schema188_new_card(row.granting_card)))
        || state.graveyard_borrowing.as_ref().is_some_and(|rows| rows.iter().any(|row|
            schema188_new_card(row.provider) || schema188_new_card(row.definition)))
        // Slots retain an optional immutable donor definition; all other slot
        // fields are object references or numeric acquisition/use identities.
        || state.borrowed_slots.as_ref().is_some_and(|rows| rows.iter().any(|row|
            row.definition.is_some_and(schema188_new_card)))
        || state.warp.as_ref().is_some_and(|w| !w.delayed.is_empty())
        // Delayed sacrifice packets retain public source-card identity and
        // frozen damage facts, including hidden/copied subtype overlays.
        || state.delayed_sacrifices.as_ref().is_some_and(|rows| rows.iter().any(|row|
            schema188_new_card(row.view.source_card) || !schema188_facts_supported(&row.facts)))
        || state.mobilize.is_some() || state.miracle.is_some()
        || state.adventures.as_ref().is_some_and(|a| a.permissions.iter().any(|v| schema188_new_card(v.card)))
        || state.plot.as_ref().and_then(|p| p.payment.as_ref()).is_some_and(|p| schema188_new_card(p.source_card))
        || state.hits.iter().any(|hit| !schema188_facts_supported(&hit.source.facts))
        || state.damage_choices.iter().any(|hit| !schema188_facts_supported(&hit.source.facts))
        || state.manifest.as_ref().is_some_and(|manifest| {
            manifest.permanents.iter().chain(manifest.spells.iter())
                .any(|row| schema188_new_card(row.underlying))
        })
        || state.copy_recipes.as_ref().is_some_and(|rows| rows.iter().any(|row| !schema188_recipe_supported(&row.recipe)))
        || state.temporary_copies.as_ref().is_some_and(|rows| rows.iter().any(|row| {
            schema188_new_card(row.physical_card)
                || row.original_recipe.as_ref().is_some_and(|r| !schema188_recipe_supported(r))
                || row.layers.iter().any(|layer| !schema188_recipe_supported(&layer.value.recipe))
        }))
    {
        return false;
    }
    if let Some(mana) = &state.mana_lots {
        let copy_safe = |source: &mana_lots::ManaCopySource| {
            !schema188_new_card(source.card)
                && !schema188_new_card(source.definition)
                && schema188_facts_supported(&source.facts)
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
            || !prior_schema188_state_supported(&rollback.before))
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
        .all(schema188_packet_supported)
}

/// Schema 187 adds the Jace loyalty timing permission.
fn project_prior_schema187(input: &str) -> Result<String, String> {
    fn check(value: &serde_json::Value) -> Result<(), String> {
        match value {
            serde_json::Value::Array(rows) => {
                for row in rows {
                    check(row)?;
                }
            }
            serde_json::Value::String(name) if matches!(name.as_str(), "Jace's Machinations") => {
                return Err("Jace timing rules have no schema-186 baseline".to_owned());
            }
            serde_json::Value::Object(map) => {
                if ["jace_loyalty_anytime"]
                    .iter()
                    .any(|key| map.contains_key(*key))
                {
                    return Err("Jace loyalty timing has no schema-186 representation".to_owned());
                }
                for value in map.values() {
                    check(value)?;
                }
            }
            _ => (),
        }
        Ok(())
    }
    let value: serde_json::Value = serde_json::from_str(input).map_err(|e| e.to_string())?;
    check(&value)?;
    Ok(input.to_owned())
}

fn schema187_new_card(card: Card) -> bool {
    matches!(card.identity(), "Jace's Machinations")
}

fn schema187_recipe_supported(recipe: &CopyRecipe) -> bool {
    if schema187_new_card(recipe.definition) {
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
            .any(schema187_new_card),
        _ => true,
    }
}

fn schema187_facts_supported(facts: &DamageFacts) -> bool {
    // Schema 180 already represents additive Construct and other subtype facts.
    !schema187_new_card(facts.card)
}

/// Inspect all packet fields carrying card definitions, recipes or LKI subtype
/// overlays. Reference-only and numeric fields cannot contain the new rules.
fn schema187_packet_supported(item: &SpellView) -> bool {
    let Some(source) = item.source else {
        return false;
    };
    if !serde_json::to_string(item).is_ok_and(|json| project_prior_schema187(&json).is_ok())
        || schema187_new_card(source.card)
        || !schema187_facts_supported(&source.facts)
        || matches!(item.kind, StackKind::DelayedTokenCreation { token, .. } if schema187_new_card(token))
        || item
            .source_copy_recipe
            .as_deref()
            .is_some_and(|r| !schema187_recipe_supported(r))
        || item
            .captured_artifact_copy
            .as_deref()
            .is_some_and(|r| !schema187_recipe_supported(r))
        || item
            .attachment_trigger
            .as_ref()
            .is_some_and(|v| schema187_new_card(v.granting_card))
        || item
            .delayed_sacrifice
            .as_ref()
            .is_some_and(|v| schema187_new_card(v.source_card))
        || item
            .delayed_warp_exile
            .as_ref()
            .is_some_and(|v| schema187_new_card(v.source_card))
        || item
            .cast_event
            .as_ref()
            .is_some_and(|v| schema187_new_card(v.card))
        || item
            .cast
            .and_then(|v| v.physical_card)
            .is_some_and(schema187_new_card)
        || item.mana_copy_sources.as_ref().is_some_and(|sources| {
            sources.iter().any(|(v, _)| {
                schema187_new_card(v.card)
                    || schema187_new_card(v.definition)
                    || !schema187_facts_supported(&v.facts)
            })
        })
        || item
            .copy_event
            .as_ref()
            .is_some_and(|event| !schema187_packet_supported(&event.spell))
    {
        return false;
    }
    if let Some(event) = &item.event_object {
        if event
            .subtype_lki
            .as_ref()
            .is_some_and(|facts| !schema187_facts_supported(facts))
        {
            return false;
        }
        match &event.facts {
            event_objects::EventObjectFacts::DamageActor(source)
                if !schema187_facts_supported(&source.facts) =>
            {
                return false;
            }
            event_objects::EventObjectFacts::Copiable(recipe)
                if !schema187_recipe_supported(recipe) =>
            {
                return false;
            }
            _ => {}
        }
    }
    true
}

/// Conservatively reject opaque continuations, while checking copy layers and
/// retained packets by type. Rollback snapshots retain independent hidden state.
fn prior_schema187_state_supported(state: &State) -> bool {
    if state.players.iter().any(|player| player.jace_loyalty_anytime)
        || state.resolving.is_some() || state.announcing.is_some()
        || state.mana_announcement_stack.is_some() || state.replacements.is_some()
        || state.entry_batch.is_some() || state.zone_rules.as_ref().is_some_and(|z| z.has_checkpoint())
        || state.linked_choices.as_ref().is_some_and(|c| c.has_opaque_interaction())
        || state.objects.iter().any(|o| schema187_new_card(o.card))
        || state.explore_data.as_ref().and_then(|data| data.last.as_ref()).is_some_and(|event|
            schema187_new_card(event.explorer.card)
                || event.revealed.is_some_and(|card| schema187_new_card(card.card)))
        || state.last_target_library_move.as_ref().is_some_and(|event| schema187_new_card(event.card))
        || state.library_top.as_ref().and_then(|data| data.reveal.as_ref())
            .is_some_and(|event| schema187_new_card(event.revealed.card))
        || state.temporary_abilities.as_ref().is_some_and(|rows| rows.iter().any(|row| schema187_new_card(row.granting_card)))
        || state.self_granted_abilities.as_ref().is_some_and(|rows| rows.iter().any(|row| schema187_new_card(row.granting_card)))
        || state.graveyard_borrowing.as_ref().is_some_and(|rows| rows.iter().any(|row|
            schema187_new_card(row.provider) || schema187_new_card(row.definition)))
        // Slots retain an optional immutable donor definition; all other slot
        // fields are object references or numeric acquisition/use identities.
        || state.borrowed_slots.as_ref().is_some_and(|rows| rows.iter().any(|row|
            row.definition.is_some_and(schema187_new_card)))
        || state.warp.as_ref().is_some_and(|w| !w.delayed.is_empty())
        // Delayed sacrifice packets retain public source-card identity and
        // frozen damage facts, including hidden/copied subtype overlays.
        || state.delayed_sacrifices.as_ref().is_some_and(|rows| rows.iter().any(|row|
            schema187_new_card(row.view.source_card) || !schema187_facts_supported(&row.facts)))
        || state.mobilize.is_some() || state.miracle.is_some()
        || state.adventures.as_ref().is_some_and(|a| a.permissions.iter().any(|v| schema187_new_card(v.card)))
        || state.plot.as_ref().and_then(|p| p.payment.as_ref()).is_some_and(|p| schema187_new_card(p.source_card))
        || state.hits.iter().any(|hit| !schema187_facts_supported(&hit.source.facts))
        || state.damage_choices.iter().any(|hit| !schema187_facts_supported(&hit.source.facts))
        || state.manifest.as_ref().is_some_and(|manifest| {
            manifest.permanents.iter().chain(manifest.spells.iter())
                .any(|row| schema187_new_card(row.underlying))
        })
        || state.copy_recipes.as_ref().is_some_and(|rows| rows.iter().any(|row| !schema187_recipe_supported(&row.recipe)))
        || state.temporary_copies.as_ref().is_some_and(|rows| rows.iter().any(|row| {
            schema187_new_card(row.physical_card)
                || row.original_recipe.as_ref().is_some_and(|r| !schema187_recipe_supported(r))
                || row.layers.iter().any(|layer| !schema187_recipe_supported(&layer.value.recipe))
        }))
    {
        return false;
    }
    if let Some(mana) = &state.mana_lots {
        let copy_safe = |source: &mana_lots::ManaCopySource| {
            !schema187_new_card(source.card)
                && !schema187_new_card(source.definition)
                && schema187_facts_supported(&source.facts)
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
            || !prior_schema187_state_supported(&rollback.before))
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
        .all(schema187_packet_supported)
}

fn project_prior_schema186(input: &str) -> Result<String, String> {
    fn check(value: &serde_json::Value) -> Result<(), String> {
        match value {
            serde_json::Value::Array(rows) => {
                for row in rows {
                    check(row)?;
                }
            }
            serde_json::Value::String(name)
                if matches!(
                    name.as_str(),
                    "Chandra, Chill of Compliance"
                        | "@emblem/fra-chandra-chill"
                        | "Garruk, Veiled Butcher"
                        | "NoncreatureSpell"
                ) =>
            {
                return Err("Chandra or Garruk rules have no schema-185 baseline".to_owned());
            }
            serde_json::Value::Object(map) => {
                if ["loyalty_x", "until_next_turn_stats"]
                    .iter()
                    .any(|key| map.contains_key(*key))
                {
                    return Err(
                        "variable loyalty or next-turn stat state has no schema-185 representation"
                            .to_owned(),
                    );
                }
                for value in map.values() {
                    check(value)?;
                }
            }
            _ => (),
        }
        Ok(())
    }
    let value: serde_json::Value = serde_json::from_str(input).map_err(|e| e.to_string())?;
    check(&value)?;
    Ok(input.to_owned())
}

fn schema186_new_card(card: Card) -> bool {
    matches!(
        card.identity(),
        "Chandra, Chill of Compliance" | "@emblem/fra-chandra-chill" | "Garruk, Veiled Butcher"
    )
}

fn schema186_recipe_supported(recipe: &CopyRecipe) -> bool {
    if schema186_new_card(recipe.definition) {
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
            .any(schema186_new_card),
        _ => true,
    }
}

fn schema186_facts_supported(facts: &DamageFacts) -> bool {
    // Schema 180 already represents additive Construct and other subtype facts.
    !schema186_new_card(facts.card)
}

/// Inspect all packet fields carrying card definitions, recipes or LKI subtype
/// overlays. Reference-only and numeric fields cannot contain the new rules.
fn schema186_packet_supported(item: &SpellView) -> bool {
    let Some(source) = item.source else {
        return false;
    };
    if !serde_json::to_string(item).is_ok_and(|json| project_prior_schema186(&json).is_ok())
        || item.loyalty_x.is_some()
        || schema186_new_card(source.card)
        || !schema186_facts_supported(&source.facts)
        || matches!(item.kind, StackKind::DelayedTokenCreation { token, .. } if schema186_new_card(token))
        || item
            .source_copy_recipe
            .as_deref()
            .is_some_and(|r| !schema186_recipe_supported(r))
        || item
            .captured_artifact_copy
            .as_deref()
            .is_some_and(|r| !schema186_recipe_supported(r))
        || item
            .attachment_trigger
            .as_ref()
            .is_some_and(|v| schema186_new_card(v.granting_card))
        || item
            .delayed_sacrifice
            .as_ref()
            .is_some_and(|v| schema186_new_card(v.source_card))
        || item
            .delayed_warp_exile
            .as_ref()
            .is_some_and(|v| schema186_new_card(v.source_card))
        || item
            .cast_event
            .as_ref()
            .is_some_and(|v| schema186_new_card(v.card))
        || item
            .cast
            .and_then(|v| v.physical_card)
            .is_some_and(schema186_new_card)
        || item.mana_copy_sources.as_ref().is_some_and(|sources| {
            sources.iter().any(|(v, _)| {
                schema186_new_card(v.card)
                    || schema186_new_card(v.definition)
                    || !schema186_facts_supported(&v.facts)
            })
        })
        || item
            .copy_event
            .as_ref()
            .is_some_and(|event| !schema186_packet_supported(&event.spell))
    {
        return false;
    }
    if let Some(event) = &item.event_object {
        if event
            .subtype_lki
            .as_ref()
            .is_some_and(|facts| !schema186_facts_supported(facts))
        {
            return false;
        }
        match &event.facts {
            event_objects::EventObjectFacts::DamageActor(source)
                if !schema186_facts_supported(&source.facts) =>
            {
                return false;
            }
            event_objects::EventObjectFacts::Copiable(recipe)
                if !schema186_recipe_supported(recipe) =>
            {
                return false;
            }
            _ => {}
        }
    }
    true
}

/// Conservatively reject opaque continuations, while checking copy layers and
/// retained packets by type. Rollback snapshots retain independent hidden state.
fn prior_schema186_state_supported(state: &State) -> bool {
    if state.roles.as_ref().is_some_and(|roles| !roles.until_next_turn_stats.is_empty())
        || state.resolving.is_some() || state.announcing.is_some()
        || state.mana_announcement_stack.is_some() || state.replacements.is_some()
        || state.entry_batch.is_some() || state.zone_rules.as_ref().is_some_and(|z| z.has_checkpoint())
        || state.linked_choices.as_ref().is_some_and(|c| c.has_opaque_interaction())
        || state.objects.iter().any(|o| schema186_new_card(o.card))
        || state.explore_data.as_ref().and_then(|data| data.last.as_ref()).is_some_and(|event|
            schema186_new_card(event.explorer.card)
                || event.revealed.is_some_and(|card| schema186_new_card(card.card)))
        || state.last_target_library_move.as_ref().is_some_and(|event| schema186_new_card(event.card))
        || state.library_top.as_ref().and_then(|data| data.reveal.as_ref())
            .is_some_and(|event| schema186_new_card(event.revealed.card))
        || state.temporary_abilities.as_ref().is_some_and(|rows| rows.iter().any(|row| schema186_new_card(row.granting_card)))
        || state.self_granted_abilities.as_ref().is_some_and(|rows| rows.iter().any(|row| schema186_new_card(row.granting_card)))
        || state.graveyard_borrowing.as_ref().is_some_and(|rows| rows.iter().any(|row|
            schema186_new_card(row.provider) || schema186_new_card(row.definition)))
        // Slots retain an optional immutable donor definition; all other slot
        // fields are object references or numeric acquisition/use identities.
        || state.borrowed_slots.as_ref().is_some_and(|rows| rows.iter().any(|row|
            row.definition.is_some_and(schema186_new_card)))
        || state.warp.as_ref().is_some_and(|w| !w.delayed.is_empty())
        // Delayed sacrifice packets retain public source-card identity and
        // frozen damage facts, including hidden/copied subtype overlays.
        || state.delayed_sacrifices.as_ref().is_some_and(|rows| rows.iter().any(|row|
            schema186_new_card(row.view.source_card) || !schema186_facts_supported(&row.facts)))
        || state.mobilize.is_some() || state.miracle.is_some()
        || state.adventures.as_ref().is_some_and(|a| a.permissions.iter().any(|v| schema186_new_card(v.card)))
        || state.plot.as_ref().and_then(|p| p.payment.as_ref()).is_some_and(|p| schema186_new_card(p.source_card))
        || state.hits.iter().any(|hit| !schema186_facts_supported(&hit.source.facts))
        || state.damage_choices.iter().any(|hit| !schema186_facts_supported(&hit.source.facts))
        || state.manifest.as_ref().is_some_and(|manifest| {
            manifest.permanents.iter().chain(manifest.spells.iter())
                .any(|row| schema186_new_card(row.underlying))
        })
        || state.copy_recipes.as_ref().is_some_and(|rows| rows.iter().any(|row| !schema186_recipe_supported(&row.recipe)))
        || state.temporary_copies.as_ref().is_some_and(|rows| rows.iter().any(|row| {
            schema186_new_card(row.physical_card)
                || row.original_recipe.as_ref().is_some_and(|r| !schema186_recipe_supported(r))
                || row.layers.iter().any(|layer| !schema186_recipe_supported(&layer.value.recipe))
        }))
    {
        return false;
    }
    if let Some(mana) = &state.mana_lots {
        let copy_safe = |source: &mana_lots::ManaCopySource| {
            !schema186_new_card(source.card)
                && !schema186_new_card(source.definition)
                && schema186_facts_supported(&source.facts)
        };
        let quality_safe = |quality: &mana_lots::Quality| {
            quality.restriction != ManaRestrictionView::NoncreatureSpell
                && quality.copy_source.as_ref().is_none_or(copy_safe)
        };
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
            || !prior_schema186_state_supported(&rollback.before))
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
        .all(schema186_packet_supported)
}

/// Schema 185 exposes Mountain-triggered mana and its retained payment facts.
/// Only states without the new rules can use byte-identical schema 184 traces.
fn project_prior_schema185(input: &str) -> Result<String, String> {
    fn check(value: &serde_json::Value) -> Result<(), String> {
        match value {
            serde_json::Value::Array(rows) => {
                for row in rows {
                    check(row)?;
                }
            }
            serde_json::Value::String(name)
                if matches!(
                    name.as_str(),
                    "Pyre Rhymer // Molten Tide"
                        | "Pyre Rhymer"
                        | "Molten Tide"
                        | "@prepare/pyre_rhymer_molten_tide"
                ) =>
            {
                return Err("Pyre rules have no schema-184 baseline".to_owned());
            }
            serde_json::Value::Object(map) => {
                if ["mountain_mana_bonus", "mana_tap_bonus", "mountain_bonus"]
                    .iter()
                    .any(|key| map.contains_key(*key))
                {
                    return Err(
                        "Mountain mana trigger state has no schema-184 representation".to_owned(),
                    );
                }
                for value in map.values() {
                    check(value)?;
                }
            }
            _ => (),
        }
        Ok(())
    }
    let value: serde_json::Value = serde_json::from_str(input).map_err(|e| e.to_string())?;
    check(&value)?;
    Ok(input.to_owned())
}

#[test]
fn schema185_preserves_old_bytes_and_rejects_new_fields_and_cards() {
    let old =
        r#"[{"resolution_free_casts":{"paid":{"kinds":255}},"card":"Chandra, Torch of Defiance"}]"#;
    assert_eq!(project_prior_schema185(old).unwrap(), old);
    for name in [
        "Pyre Rhymer // Molten Tide",
        "Pyre Rhymer",
        "Molten Tide",
        "@prepare/pyre_rhymer_molten_tide",
    ] {
        assert!(project_prior_schema185(&serde_json::json!({"card": name}).to_string()).is_err());
    }
    for field in ["mountain_mana_bonus", "mana_tap_bonus", "mountain_bonus"] {
        let mut value = serde_json::json!({"nested": {}});
        value["nested"][field] = serde_json::json!(1);
        assert!(project_prior_schema185(&value.to_string()).is_err());
    }
}

#[test]
fn schema185_rejects_owned_bonus_nested_packets_receipts_and_rollback() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let source = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        assert!(prior_schema185_state_supported(&g.state));
        g.state.players[player.index()].mountain_mana_bonus = 1;
        for viewer in [player, player.other()] {
            let view = serde_json::to_string(&g.observe(viewer)).unwrap();
            assert!(project_prior_schema185(&view).is_err());
        }
        g.begin_attack_rollback(player);
        g.state.players[player.index()].mountain_mana_bonus = 0;
        assert!(!prior_schema185_state_supported(&g.state));
        g.state.cast_rollback = None;
        assert!(prior_schema185_state_supported(&g.state));
        let hidden_id = put(&mut g, player, "Bear Cub", Zone::Hand);
        let hidden_object = g.object_ref(hidden_id);
        let bear = Card::from_name("Bear Cub").unwrap();
        let pyre = Card::from_name("Pyre Rhymer // Molten Tide").unwrap();
        let ordinary = CopyRecipe::new(bear);
        for card in [pyre, Card::test_index(5461)] {
            let hidden = ordinary.with_faces(copies::CopyFaces::Double {
                physical_front: bear,
                front: bear,
                back: card,
                back_up: false,
            });
            g.set_copy_recipe(hidden_object, hidden);
            assert!(!prior_schema185_state_supported(&g.state));
        }
        g.set_copy_recipe(hidden_object, ordinary);
        assert!(prior_schema185_state_supported(&g.state));
        let mut inner = g.activation_item(player, source, 0);
        inner.mana_tap_bonus = Some(1);
        let mut outer = g.activation_item(player, source, 0);
        outer.copy_event = Some(Box::new(stack_copies::CopyEvent {
            spell: inner,
            amount: 1,
        }));
        g.state.pending_triggers.push(outer);
        assert!(!prior_schema185_state_supported(&g.state));
        g.state.pending_triggers.clear();
        assert!(prior_schema185_state_supported(&g.state));
        g.begin_attack_rollback(player);
        let source = g.object_ref(source);
        g.state
            .cast_rollback
            .as_mut()
            .unwrap()
            .events
            .push(cast_rollback_mana::ManaEvent {
                id: 0,
                actor: player,
                kind: cast_rollback_mana::ManaEventKind::Basic { source },
                production: [0, 0, 0, 1, 0, 0],
                mountain_bonus: 1,
                quality: None,
                spent_lots: Vec::new(),
                counter_costs: Vec::new(),
                entries: Vec::new(),
                entry_life: [0; 2],
                moves: Vec::new(),
                allocations: Vec::new(),
                triggers: Vec::new(),
                irreversible: false,
            });
        assert!(!prior_schema185_state_supported(&g.state));
        let saved = g.snapshot();
        g.state.cast_rollback = None;
        assert!(prior_schema185_state_supported(&g.state));
        g.restore(&saved).unwrap();
        assert!(!prior_schema185_state_supported(&g.state));
    }
}

fn schema185_new_card(card: Card) -> bool {
    matches!(
        card.identity(),
        "Pyre Rhymer // Molten Tide" | "@prepare/pyre_rhymer_molten_tide"
    )
}

fn schema185_recipe_supported(recipe: &CopyRecipe) -> bool {
    if schema185_new_card(recipe.definition) {
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
            .any(schema185_new_card),
        _ => true,
    }
}

fn schema185_facts_supported(facts: &DamageFacts) -> bool {
    // Schema 180 already represents additive Construct and other subtype facts.
    !schema185_new_card(facts.card)
}

/// Inspect all packet fields carrying card definitions, recipes or LKI subtype
/// overlays. Reference-only and numeric fields cannot contain the new rules.
fn schema185_packet_supported(item: &SpellView) -> bool {
    let Some(source) = item.source else {
        return false;
    };
    if !serde_json::to_string(item).is_ok_and(|json| project_prior_schema185(&json).is_ok())
        || item.mana_tap_bonus.is_some()
        || schema185_new_card(source.card)
        || !schema185_facts_supported(&source.facts)
        || matches!(item.kind, StackKind::DelayedTokenCreation { token, .. } if schema185_new_card(token))
        || item
            .source_copy_recipe
            .as_deref()
            .is_some_and(|r| !schema185_recipe_supported(r))
        || item
            .captured_artifact_copy
            .as_deref()
            .is_some_and(|r| !schema185_recipe_supported(r))
        || item
            .attachment_trigger
            .as_ref()
            .is_some_and(|v| schema185_new_card(v.granting_card))
        || item
            .delayed_sacrifice
            .as_ref()
            .is_some_and(|v| schema185_new_card(v.source_card))
        || item
            .delayed_warp_exile
            .as_ref()
            .is_some_and(|v| schema185_new_card(v.source_card))
        || item
            .cast_event
            .as_ref()
            .is_some_and(|v| schema185_new_card(v.card))
        || item
            .cast
            .and_then(|v| v.physical_card)
            .is_some_and(schema185_new_card)
        || item.mana_copy_sources.as_ref().is_some_and(|sources| {
            sources.iter().any(|(v, _)| {
                schema185_new_card(v.card)
                    || schema185_new_card(v.definition)
                    || !schema185_facts_supported(&v.facts)
            })
        })
        || item
            .copy_event
            .as_ref()
            .is_some_and(|event| !schema185_packet_supported(&event.spell))
    {
        return false;
    }
    if let Some(event) = &item.event_object {
        if event
            .subtype_lki
            .as_ref()
            .is_some_and(|facts| !schema185_facts_supported(facts))
        {
            return false;
        }
        match &event.facts {
            event_objects::EventObjectFacts::DamageActor(source)
                if !schema185_facts_supported(&source.facts) =>
            {
                return false;
            }
            event_objects::EventObjectFacts::Copiable(recipe)
                if !schema185_recipe_supported(recipe) =>
            {
                return false;
            }
            _ => {}
        }
    }
    true
}

/// Conservatively reject opaque continuations, while checking copy layers and
/// retained packets by type. Rollback snapshots retain independent hidden state.
fn prior_schema185_state_supported(state: &State) -> bool {
    if state.players.iter().any(|player| player.mountain_mana_bonus != 0)
        || state.resolving.is_some() || state.announcing.is_some()
        || state.mana_announcement_stack.is_some() || state.replacements.is_some()
        || state.entry_batch.is_some() || state.zone_rules.as_ref().is_some_and(|z| z.has_checkpoint())
        || state.linked_choices.as_ref().is_some_and(|c| c.has_opaque_interaction())
        || state.objects.iter().any(|o| schema185_new_card(o.card))
        || state.explore_data.as_ref().and_then(|data| data.last.as_ref()).is_some_and(|event|
            schema185_new_card(event.explorer.card)
                || event.revealed.is_some_and(|card| schema185_new_card(card.card)))
        || state.last_target_library_move.as_ref().is_some_and(|event| schema185_new_card(event.card))
        || state.library_top.as_ref().and_then(|data| data.reveal.as_ref())
            .is_some_and(|event| schema185_new_card(event.revealed.card))
        || state.temporary_abilities.as_ref().is_some_and(|rows| rows.iter().any(|row| schema185_new_card(row.granting_card)))
        || state.self_granted_abilities.as_ref().is_some_and(|rows| rows.iter().any(|row| schema185_new_card(row.granting_card)))
        || state.graveyard_borrowing.as_ref().is_some_and(|rows| rows.iter().any(|row|
            schema185_new_card(row.provider) || schema185_new_card(row.definition)))
        // Slots retain an optional immutable donor definition; all other slot
        // fields are object references or numeric acquisition/use identities.
        || state.borrowed_slots.as_ref().is_some_and(|rows| rows.iter().any(|row|
            row.definition.is_some_and(schema185_new_card)))
        || state.warp.as_ref().is_some_and(|w| !w.delayed.is_empty())
        // Delayed sacrifice packets retain public source-card identity and
        // frozen damage facts, including hidden/copied subtype overlays.
        || state.delayed_sacrifices.as_ref().is_some_and(|rows| rows.iter().any(|row|
            schema185_new_card(row.view.source_card) || !schema185_facts_supported(&row.facts)))
        || state.mobilize.is_some() || state.miracle.is_some()
        || state.adventures.as_ref().is_some_and(|a| a.permissions.iter().any(|v| schema185_new_card(v.card)))
        || state.plot.as_ref().and_then(|p| p.payment.as_ref()).is_some_and(|p| schema185_new_card(p.source_card))
        || state.hits.iter().any(|hit| !schema185_facts_supported(&hit.source.facts))
        || state.damage_choices.iter().any(|hit| !schema185_facts_supported(&hit.source.facts))
        || state.manifest.as_ref().is_some_and(|manifest| {
            manifest.permanents.iter().chain(manifest.spells.iter())
                .any(|row| schema185_new_card(row.underlying))
        })
        || state.copy_recipes.as_ref().is_some_and(|rows| rows.iter().any(|row| !schema185_recipe_supported(&row.recipe)))
        || state.temporary_copies.as_ref().is_some_and(|rows| rows.iter().any(|row| {
            schema185_new_card(row.physical_card)
                || row.original_recipe.as_ref().is_some_and(|r| !schema185_recipe_supported(r))
                || row.layers.iter().any(|layer| !schema185_recipe_supported(&layer.value.recipe))
        }))
    {
        return false;
    }
    if let Some(mana) = &state.mana_lots {
        let copy_safe = |source: &mana_lots::ManaCopySource| {
            !schema185_new_card(source.card)
                && !schema185_new_card(source.definition)
                && schema185_facts_supported(&source.facts)
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
            || !prior_schema185_state_supported(&rollback.before))
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
        .all(schema185_packet_supported)
}

/// Schema 184 adds Chandra and changes actions during paid immediate casts.
fn project_prior_schema184(input: &str) -> Result<String, String> {
    fn check(value: &serde_json::Value) -> Result<(), String> {
        match value {
            serde_json::Value::Array(rows) => {
                for row in rows {
                    check(row)?;
                }
            }
            serde_json::Value::String(name)
                if matches!(
                    name.as_str(),
                    "Chandra, Torch of Defiance" | "@emblem/fra-chandra-torch"
                ) =>
            {
                return Err("Chandra rules or emblem have no schema-183 baseline".to_owned());
            }
            serde_json::Value::Object(map) => {
                if map
                    .get("resolution_free_casts")
                    .and_then(|v| v.as_object())
                    .and_then(|v| v.get("paid"))
                    .is_some_and(|v| !v.is_null())
                {
                    return Err("paid resolution-cast actions differ from schema 183".to_owned());
                }
                for value in map.values() {
                    check(value)?;
                }
            }
            _ => (),
        }
        Ok(())
    }
    let value: serde_json::Value = serde_json::from_str(input).map_err(|e| e.to_string())?;
    check(&value)?;
    Ok(input.to_owned())
}

fn schema184_new_card(card: Card) -> bool {
    matches!(
        card.identity(),
        "Chandra, Torch of Defiance" | "@emblem/fra-chandra-torch"
    )
}

fn schema184_recipe_supported(recipe: &CopyRecipe) -> bool {
    if schema184_new_card(recipe.definition) {
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
            .any(schema184_new_card),
        _ => true,
    }
}

fn schema184_facts_supported(facts: &DamageFacts) -> bool {
    // Schema 180 already represents additive Construct and other subtype facts.
    !schema184_new_card(facts.card)
}

/// Inspect all packet fields carrying card definitions, recipes or LKI subtype
/// overlays. Reference-only and numeric fields cannot contain the new rules.
fn schema184_packet_supported(item: &SpellView) -> bool {
    let Some(source) = item.source else {
        return false;
    };
    if !serde_json::to_string(item).is_ok_and(|json| project_prior_schema184(&json).is_ok())
        || schema184_new_card(source.card)
        || !schema184_facts_supported(&source.facts)
        || matches!(item.kind, StackKind::DelayedTokenCreation { token, .. } if schema184_new_card(token))
        || item
            .source_copy_recipe
            .as_deref()
            .is_some_and(|r| !schema184_recipe_supported(r))
        || item
            .captured_artifact_copy
            .as_deref()
            .is_some_and(|r| !schema184_recipe_supported(r))
        || item
            .attachment_trigger
            .as_ref()
            .is_some_and(|v| schema184_new_card(v.granting_card))
        || item
            .delayed_sacrifice
            .as_ref()
            .is_some_and(|v| schema184_new_card(v.source_card))
        || item
            .delayed_warp_exile
            .as_ref()
            .is_some_and(|v| schema184_new_card(v.source_card))
        || item
            .cast_event
            .as_ref()
            .is_some_and(|v| schema184_new_card(v.card))
        || item
            .cast
            .and_then(|v| v.physical_card)
            .is_some_and(schema184_new_card)
        || item.mana_copy_sources.as_ref().is_some_and(|sources| {
            sources.iter().any(|(v, _)| {
                schema184_new_card(v.card)
                    || schema184_new_card(v.definition)
                    || !schema184_facts_supported(&v.facts)
            })
        })
        || item
            .copy_event
            .as_ref()
            .is_some_and(|event| !schema184_packet_supported(&event.spell))
    {
        return false;
    }
    if let Some(event) = &item.event_object {
        if event
            .subtype_lki
            .as_ref()
            .is_some_and(|facts| !schema184_facts_supported(facts))
        {
            return false;
        }
        match &event.facts {
            event_objects::EventObjectFacts::DamageActor(source)
                if !schema184_facts_supported(&source.facts) =>
            {
                return false;
            }
            event_objects::EventObjectFacts::Copiable(recipe)
                if !schema184_recipe_supported(recipe) =>
            {
                return false;
            }
            _ => {}
        }
    }
    true
}

/// Conservatively reject opaque continuations, while checking copy layers and
/// retained packets by type. Rollback snapshots retain independent hidden state.
fn prior_schema184_state_supported(state: &State) -> bool {
    if state.resolution_free_casts.as_ref().is_some_and(|offer| offer.paid.is_some())
        || state.resolving.is_some() || state.announcing.is_some()
        || state.mana_announcement_stack.is_some() || state.replacements.is_some()
        || state.entry_batch.is_some() || state.zone_rules.as_ref().is_some_and(|z| z.has_checkpoint())
        || state.linked_choices.as_ref().is_some_and(|c| c.has_opaque_interaction())
        || state.objects.iter().any(|o| schema184_new_card(o.card))
        || state.explore_data.as_ref().and_then(|data| data.last.as_ref()).is_some_and(|event|
            schema184_new_card(event.explorer.card)
                || event.revealed.is_some_and(|card| schema184_new_card(card.card)))
        || state.last_target_library_move.as_ref().is_some_and(|event| schema184_new_card(event.card))
        || state.library_top.as_ref().and_then(|data| data.reveal.as_ref())
            .is_some_and(|event| schema184_new_card(event.revealed.card))
        || state.temporary_abilities.as_ref().is_some_and(|rows| rows.iter().any(|row| schema184_new_card(row.granting_card)))
        || state.self_granted_abilities.as_ref().is_some_and(|rows| rows.iter().any(|row| schema184_new_card(row.granting_card)))
        || state.graveyard_borrowing.as_ref().is_some_and(|rows| rows.iter().any(|row|
            schema184_new_card(row.provider) || schema184_new_card(row.definition)))
        // Slots retain an optional immutable donor definition; all other slot
        // fields are object references or numeric acquisition/use identities.
        || state.borrowed_slots.as_ref().is_some_and(|rows| rows.iter().any(|row|
            row.definition.is_some_and(schema184_new_card)))
        || state.warp.as_ref().is_some_and(|w| !w.delayed.is_empty())
        // Delayed sacrifice packets retain public source-card identity and
        // frozen damage facts, including hidden/copied subtype overlays.
        || state.delayed_sacrifices.as_ref().is_some_and(|rows| rows.iter().any(|row|
            schema184_new_card(row.view.source_card) || !schema184_facts_supported(&row.facts)))
        || state.mobilize.is_some() || state.miracle.is_some()
        || state.adventures.as_ref().is_some_and(|a| a.permissions.iter().any(|v| schema184_new_card(v.card)))
        || state.plot.as_ref().and_then(|p| p.payment.as_ref()).is_some_and(|p| schema184_new_card(p.source_card))
        || state.hits.iter().any(|hit| !schema184_facts_supported(&hit.source.facts))
        || state.damage_choices.iter().any(|hit| !schema184_facts_supported(&hit.source.facts))
        || state.manifest.as_ref().is_some_and(|manifest| {
            manifest.permanents.iter().chain(manifest.spells.iter())
                .any(|row| schema184_new_card(row.underlying))
        })
        || state.copy_recipes.as_ref().is_some_and(|rows| rows.iter().any(|row| !schema184_recipe_supported(&row.recipe)))
        || state.temporary_copies.as_ref().is_some_and(|rows| rows.iter().any(|row| {
            schema184_new_card(row.physical_card)
                || row.original_recipe.as_ref().is_some_and(|r| !schema184_recipe_supported(r))
                || row.layers.iter().any(|layer| !schema184_recipe_supported(&layer.value.recipe))
        }))
    {
        return false;
    }
    if let Some(mana) = &state.mana_lots {
        let copy_safe = |source: &mana_lots::ManaCopySource| {
            !schema184_new_card(source.card)
                && !schema184_new_card(source.definition)
                && schema184_facts_supported(&source.facts)
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
            || !prior_schema184_state_supported(&rollback.before))
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
        .all(schema184_packet_supported)
}

#[test]
fn schema184_preserves_old_bytes_and_rejects_changed_card_and_paid_offer() {
    assert_eq!(OBSERVATION_SCHEMA, 194);
    let old = r#"[{"resolution_free_casts":{"paid":null},"other":true}]"#;
    assert_eq!(project_prior_schema184(old).unwrap(), old);
    for card in ["Chandra, Torch of Defiance", "@emblem/fra-chandra-torch"] {
        assert!(project_prior_schema184(&serde_json::json!({"card":card}).to_string()).is_err());
    }
    let paid = r#"[{"resolution_free_casts":{"paid":{"kinds":255}}}]"#;
    assert!(project_prior_schema184(paid).is_err());
}

#[test]
fn schema184_rejects_public_and_hidden_paid_offers() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        assert!(prior_schema184_state_supported(&g.state));
        for origin in [Zone::Exile, Zone::Library] {
            g.state.resolution_free_casts = Some(Box::new(ResolutionFreeCasts {
                siege_transformed: false,
                paid: Some(ResolutionPaidCast {
                    kinds: u8::MAX,
                    exile_instants_sorceries: false,
                }),
                casts_remaining: Some(1),
                pending_cast: None,
                remaining_mana_value: None,
                alternative_cost: None,
                maximum_mana_value: None,
                origin,
                exile_on_graveyard: false,
                allow_lands: false,
                player,
                remaining: Vec::new(),
            }));
            assert!(!prior_schema184_state_supported(&g.state));
            for viewer in [player, player.other()] {
                let view = serde_json::to_string(&g.observe(viewer)).unwrap();
                if origin == Zone::Exile || viewer == player {
                    assert!(project_prior_schema184(&view).is_err());
                }
            }
            let saved = g.snapshot();
            g.state.resolution_free_casts = None;
            assert!(prior_schema184_state_supported(&g.state));
            g.restore(&saved).unwrap();
            assert!(!prior_schema184_state_supported(&g.state));
        }
    }
}

#[test]
fn schema184_rejects_hidden_card_recipe_and_rollback_checkpoint() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let id = put(&mut g, player, "Bear Cub", Zone::Hand);
        let reference = g.object_ref(id);
        let bear = Card::from_name("Bear Cub").unwrap();
        let chandra = Card::from_name("Chandra, Torch of Defiance").unwrap();
        let normal = CopyRecipe::new(bear);
        let hidden = normal.with_faces(copies::CopyFaces::Double {
            physical_front: bear,
            front: bear,
            back: chandra,
            back_up: false,
        });
        assert!(prior_schema184_state_supported(&g.state));
        g.set_copy_recipe(reference, hidden);
        assert!(!prior_schema184_state_supported(&g.state));
        g.begin_attack_rollback(player);
        g.set_copy_recipe(reference, normal);
        assert!(!prior_schema184_state_supported(&g.state));
        g.state.cast_rollback = None;
        assert!(prior_schema184_state_supported(&g.state));
        g.state.objects[id as usize].card = chandra;
        assert!(!prior_schema184_state_supported(&g.state));
        for viewer in [player, player.other()] {
            let view = serde_json::to_string(&g.observe(viewer)).unwrap();
            if viewer == player {
                assert!(project_prior_schema184(&view).is_err());
            }
        }
    }
}

/// Schema 183 exposes the persistent, exact-source delayed attack continuation.
/// Preserve prior JSON byte-for-byte only when no new field or card identity occurs.
fn project_prior_schema183(input: &str) -> Result<String, String> {
    fn check(value: &serde_json::Value) -> Result<(), String> {
        match value {
            serde_json::Value::Array(rows) => {
                for row in rows {
                    check(row)?;
                }
            }
            serde_json::Value::String(name)
                if Card::from_name(name).is_some_and(schema183_new_card) =>
            {
                return Err("Garruk delayed attack rules have no schema-182 baseline".to_owned());
            }
            serde_json::Value::Object(map) => {
                if map.contains_key("delayed_opponent_attack_until_next_turn") {
                    return Err(
                        "delayed attack continuation has no schema-182 representation".to_owned(),
                    );
                }
                for value in map.values() {
                    check(value)?;
                }
            }
            _ => (),
        }
        Ok(())
    }
    let value: serde_json::Value = serde_json::from_str(input).map_err(|e| e.to_string())?;
    check(&value)?;
    Ok(input.to_owned())
}

#[test]
fn schema183_projection_preserves_old_bytes_and_rejects_new_data() {
    let old = r#"[{"static_rule":"PlaneswalkerAttackLimit","discard_only":true}]"#;
    assert_eq!(project_prior_schema183(old).unwrap(), old);
    assert!(
        project_prior_schema183(r#"[{"nested":{"delayed_opponent_attack_until_next_turn":[{}]}}]"#)
            .is_err()
    );
    assert!(project_prior_schema183(r#"[{"nested":{"card":"Garruk, Curse Breaker"}}]"#).is_err());
}

#[test]
fn schema183_inspects_hidden_faces_delayed_packets_and_rollback() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let id = put(&mut g, player, "Bear Cub", Zone::Hand);
        let object = g.object_ref(id);
        let bear = Card::from_name("Bear Cub").unwrap();
        let garruk = Card::from_name("Garruk, Curse Breaker").unwrap();
        let ordinary = CopyRecipe::new(bear);
        let hidden = ordinary.with_faces(copies::CopyFaces::Double {
            physical_front: bear,
            front: bear,
            back: garruk,
            back_up: false,
        });
        g.set_copy_recipe(object, hidden);
        assert!(!prior_schema183_state_supported(&g.state));
        g.begin_attack_rollback(player);
        g.set_copy_recipe(object, ordinary);
        assert!(!prior_schema183_state_supported(&g.state));
        g.state.cast_rollback = None;
        assert!(prior_schema183_state_supported(&g.state));

        let source = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        let packet = g.activation_item(player, source, 0);
        let mut inner = packet.clone();
        inner.source_copy_recipe = Some(Box::new(hidden));
        let mut outer = packet.clone();
        outer.copy_event = Some(Box::new(stack_copies::CopyEvent {
            spell: inner,
            amount: 1,
        }));
        g.state.pending_triggers.push(outer);
        assert!(!prior_schema183_state_supported(&g.state));
        g.state.pending_triggers.clear();
        assert!(prior_schema183_state_supported(&g.state));

        g.state.delayed_opponent_attack_until_next_turn = Some(Box::new(vec![packet]));
        assert!(!prior_schema183_state_supported(&g.state));
        for viewer in [player, player.other()] {
            let view = serde_json::to_string(&g.observe(viewer)).unwrap();
            assert!(project_prior_schema183(&view).is_err());
        }
        let saved = g.snapshot();
        g.state.delayed_opponent_attack_until_next_turn = None;
        assert!(prior_schema183_state_supported(&g.state));
        g.restore(&saved).unwrap();
        assert!(!prior_schema183_state_supported(&g.state));
    }
}

fn schema183_new_card(card: Card) -> bool {
    card.identity() == "Garruk, Curse Breaker"
}

fn schema183_recipe_supported(recipe: &CopyRecipe) -> bool {
    if schema183_new_card(recipe.definition) {
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
            .any(schema183_new_card),
        _ => true,
    }
}

fn schema183_facts_supported(facts: &DamageFacts) -> bool {
    // Schema 180 already represents additive Construct and other subtype facts.
    !schema183_new_card(facts.card)
}

/// Inspect all packet fields carrying card definitions, recipes or LKI subtype
/// overlays. Reference-only and numeric fields cannot contain the new rules.
fn schema183_packet_supported(item: &SpellView) -> bool {
    let Some(source) = item.source else {
        return false;
    };
    if !serde_json::to_string(item).is_ok_and(|json| project_prior_schema183(&json).is_ok())
        || schema183_new_card(source.card)
        || !schema183_facts_supported(&source.facts)
        || matches!(item.kind, StackKind::DelayedTokenCreation { token, .. } if schema183_new_card(token))
        || item
            .source_copy_recipe
            .as_deref()
            .is_some_and(|r| !schema183_recipe_supported(r))
        || item
            .captured_artifact_copy
            .as_deref()
            .is_some_and(|r| !schema183_recipe_supported(r))
        || item
            .attachment_trigger
            .as_ref()
            .is_some_and(|v| schema183_new_card(v.granting_card))
        || item
            .delayed_sacrifice
            .as_ref()
            .is_some_and(|v| schema183_new_card(v.source_card))
        || item
            .delayed_warp_exile
            .as_ref()
            .is_some_and(|v| schema183_new_card(v.source_card))
        || item
            .cast_event
            .as_ref()
            .is_some_and(|v| schema183_new_card(v.card))
        || item
            .cast
            .and_then(|v| v.physical_card)
            .is_some_and(schema183_new_card)
        || item.mana_copy_sources.as_ref().is_some_and(|sources| {
            sources.iter().any(|(v, _)| {
                schema183_new_card(v.card)
                    || schema183_new_card(v.definition)
                    || !schema183_facts_supported(&v.facts)
            })
        })
        || item
            .copy_event
            .as_ref()
            .is_some_and(|event| !schema183_packet_supported(&event.spell))
    {
        return false;
    }
    if let Some(event) = &item.event_object {
        if event
            .subtype_lki
            .as_ref()
            .is_some_and(|facts| !schema183_facts_supported(facts))
        {
            return false;
        }
        match &event.facts {
            event_objects::EventObjectFacts::DamageActor(source)
                if !schema183_facts_supported(&source.facts) =>
            {
                return false;
            }
            event_objects::EventObjectFacts::Copiable(recipe)
                if !schema183_recipe_supported(recipe) =>
            {
                return false;
            }
            _ => {}
        }
    }
    true
}

/// Conservatively reject opaque continuations, while checking copy layers and
/// retained packets by type. Rollback snapshots retain independent hidden state.
fn prior_schema183_state_supported(state: &State) -> bool {
    if state.delayed_opponent_attack_until_next_turn.as_ref().is_some_and(|rows| !rows.is_empty())
        || state.resolving.is_some() || state.announcing.is_some()
        || state.mana_announcement_stack.is_some() || state.replacements.is_some()
        || state.entry_batch.is_some() || state.zone_rules.as_ref().is_some_and(|z| z.has_checkpoint())
        || state.linked_choices.as_ref().is_some_and(|c| c.has_opaque_interaction())
        || state.objects.iter().any(|o| schema183_new_card(o.card))
        || state.explore_data.as_ref().and_then(|data| data.last.as_ref()).is_some_and(|event|
            schema183_new_card(event.explorer.card)
                || event.revealed.is_some_and(|card| schema183_new_card(card.card)))
        || state.last_target_library_move.as_ref().is_some_and(|event| schema183_new_card(event.card))
        || state.library_top.as_ref().and_then(|data| data.reveal.as_ref())
            .is_some_and(|event| schema183_new_card(event.revealed.card))
        || state.temporary_abilities.as_ref().is_some_and(|rows| rows.iter().any(|row| schema183_new_card(row.granting_card)))
        || state.self_granted_abilities.as_ref().is_some_and(|rows| rows.iter().any(|row| schema183_new_card(row.granting_card)))
        || state.graveyard_borrowing.as_ref().is_some_and(|rows| rows.iter().any(|row|
            schema183_new_card(row.provider) || schema183_new_card(row.definition)))
        // Slots retain an optional immutable donor definition; all other slot
        // fields are object references or numeric acquisition/use identities.
        || state.borrowed_slots.as_ref().is_some_and(|rows| rows.iter().any(|row|
            row.definition.is_some_and(schema183_new_card)))
        || state.warp.as_ref().is_some_and(|w| !w.delayed.is_empty())
        // Delayed sacrifice packets retain public source-card identity and
        // frozen damage facts, including hidden/copied subtype overlays.
        || state.delayed_sacrifices.as_ref().is_some_and(|rows| rows.iter().any(|row|
            schema183_new_card(row.view.source_card) || !schema183_facts_supported(&row.facts)))
        || state.mobilize.is_some() || state.miracle.is_some()
        || state.adventures.as_ref().is_some_and(|a| a.permissions.iter().any(|v| schema183_new_card(v.card)))
        || state.plot.as_ref().and_then(|p| p.payment.as_ref()).is_some_and(|p| schema183_new_card(p.source_card))
        || state.hits.iter().any(|hit| !schema183_facts_supported(&hit.source.facts))
        || state.damage_choices.iter().any(|hit| !schema183_facts_supported(&hit.source.facts))
        || state.manifest.as_ref().is_some_and(|manifest| {
            manifest.permanents.iter().chain(manifest.spells.iter())
                .any(|row| schema183_new_card(row.underlying))
        })
        || state.copy_recipes.as_ref().is_some_and(|rows| rows.iter().any(|row| !schema183_recipe_supported(&row.recipe)))
        || state.temporary_copies.as_ref().is_some_and(|rows| rows.iter().any(|row| {
            schema183_new_card(row.physical_card)
                || row.original_recipe.as_ref().is_some_and(|r| !schema183_recipe_supported(r))
                || row.layers.iter().any(|layer| !schema183_recipe_supported(&layer.value.recipe))
        }))
    {
        return false;
    }
    if let Some(mana) = &state.mana_lots {
        let copy_safe = |source: &mana_lots::ManaCopySource| {
            !schema183_new_card(source.card)
                && !schema183_new_card(source.definition)
                && schema183_facts_supported(&source.facts)
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
            || !prior_schema183_state_supported(&rollback.before))
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
        .all(schema183_packet_supported)
}

/// Schema 182 gives layer-six static grants their own public identity.
/// Existing activation templates retain their schema-181 wire bytes.
fn project_prior_schema182(input: &str) -> Result<String, String> {
    fn check(value: &serde_json::Value) -> Result<(), String> {
        match value {
            serde_json::Value::Array(rows) => {
                for row in rows {
                    check(row)?;
                }
            }
            serde_json::Value::String(name)
                if Card::from_name(name).is_some_and(schema182_new_card) =>
            {
                return Err(
                    "card identity or command emblem behavior differs from the schema-181 baseline"
                        .to_owned(),
                );
            }
            serde_json::Value::Object(map) => {
                if map.contains_key("static_rule") {
                    return Err("static_rule has no schema-181 representation".to_owned());
                }
                for value in map.values() {
                    check(value)?;
                }
            }
            _ => (),
        }
        Ok(())
    }
    let value: serde_json::Value = serde_json::from_str(input).map_err(|e| e.to_string())?;
    check(&value)?;
    Ok(input.to_owned())
}

/// These authored rules first became representable in schema 182.
fn schema182_new_card(card: Card) -> bool {
    matches!(card.identity(), "Tomik, Orzhov Lawmage" | "Ajani Resolute" | "Vivien Reid"
        | "@token/fra-ajani-pridemate" | "@emblem/fra-ajani-resolute")
        // The shared command-zone change also affects previously existing
        // emblems with printed continuous groups, including Vivien Reid's.
        || (card.is_emblem() && !card.static_ability_groups().is_empty())
}

fn schema182_recipe_supported(recipe: &CopyRecipe) -> bool {
    if schema182_new_card(recipe.definition) {
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
            .any(schema182_new_card),
        _ => true,
    }
}

fn schema182_facts_supported(facts: &DamageFacts) -> bool {
    // Schema 180 already represents additive Construct and other subtype facts.
    !schema182_new_card(facts.card)
}

/// Inspect all packet fields carrying card definitions, recipes or LKI subtype
/// overlays. Reference-only and numeric fields cannot contain the new rules.
fn schema182_packet_supported(item: &SpellView) -> bool {
    let Some(source) = item.source else {
        return false;
    };
    if !serde_json::to_string(item).is_ok_and(|json| project_prior_schema182(&json).is_ok())
        || schema182_new_card(source.card)
        || !schema182_facts_supported(&source.facts)
        || matches!(item.kind, StackKind::DelayedTokenCreation { token, .. } if schema182_new_card(token))
        || item
            .source_copy_recipe
            .as_deref()
            .is_some_and(|r| !schema182_recipe_supported(r))
        || item
            .captured_artifact_copy
            .as_deref()
            .is_some_and(|r| !schema182_recipe_supported(r))
        || item
            .attachment_trigger
            .as_ref()
            .is_some_and(|v| schema182_new_card(v.granting_card))
        || item
            .delayed_sacrifice
            .as_ref()
            .is_some_and(|v| schema182_new_card(v.source_card))
        || item
            .delayed_warp_exile
            .as_ref()
            .is_some_and(|v| schema182_new_card(v.source_card))
        || item
            .cast_event
            .as_ref()
            .is_some_and(|v| schema182_new_card(v.card))
        || item
            .cast
            .and_then(|v| v.physical_card)
            .is_some_and(schema182_new_card)
        || item.mana_copy_sources.as_ref().is_some_and(|sources| {
            sources.iter().any(|(v, _)| {
                schema182_new_card(v.card)
                    || schema182_new_card(v.definition)
                    || !schema182_facts_supported(&v.facts)
            })
        })
        || item
            .copy_event
            .as_ref()
            .is_some_and(|event| !schema182_packet_supported(&event.spell))
    {
        return false;
    }
    if let Some(event) = &item.event_object {
        if event
            .subtype_lki
            .as_ref()
            .is_some_and(|facts| !schema182_facts_supported(facts))
        {
            return false;
        }
        match &event.facts {
            event_objects::EventObjectFacts::DamageActor(source)
                if !schema182_facts_supported(&source.facts) =>
            {
                return false;
            }
            event_objects::EventObjectFacts::Copiable(recipe)
                if !schema182_recipe_supported(recipe) =>
            {
                return false;
            }
            _ => {}
        }
    }
    true
}

/// Conservatively reject opaque continuations, while checking copy layers and
/// retained packets by type. Rollback snapshots retain independent hidden state.
fn prior_schema182_state_supported(state: &State) -> bool {
    if state.resolving.is_some() || state.announcing.is_some()
        || state.mana_announcement_stack.is_some() || state.replacements.is_some()
        || state.entry_batch.is_some() || state.zone_rules.as_ref().is_some_and(|z| z.has_checkpoint())
        || state.linked_choices.as_ref().is_some_and(|c| c.has_opaque_interaction())
        || state.objects.iter().any(|o| schema182_new_card(o.card))
        || state.explore_data.as_ref().and_then(|data| data.last.as_ref()).is_some_and(|event|
            schema182_new_card(event.explorer.card)
                || event.revealed.is_some_and(|card| schema182_new_card(card.card)))
        || state.last_target_library_move.as_ref().is_some_and(|event| schema182_new_card(event.card))
        || state.library_top.as_ref().and_then(|data| data.reveal.as_ref())
            .is_some_and(|event| schema182_new_card(event.revealed.card))
        || state.temporary_abilities.as_ref().is_some_and(|rows| rows.iter().any(|row| schema182_new_card(row.granting_card)))
        || state.self_granted_abilities.as_ref().is_some_and(|rows| rows.iter().any(|row| schema182_new_card(row.granting_card)))
        || state.graveyard_borrowing.as_ref().is_some_and(|rows| rows.iter().any(|row|
            schema182_new_card(row.provider) || schema182_new_card(row.definition)))
        // Slots retain an optional immutable donor definition; all other slot
        // fields are object references or numeric acquisition/use identities.
        || state.borrowed_slots.as_ref().is_some_and(|rows| rows.iter().any(|row|
            row.definition.is_some_and(schema182_new_card)))
        || state.warp.as_ref().is_some_and(|w| !w.delayed.is_empty())
        // Delayed sacrifice packets retain public source-card identity and
        // frozen damage facts, including hidden/copied subtype overlays.
        || state.delayed_sacrifices.as_ref().is_some_and(|rows| rows.iter().any(|row|
            schema182_new_card(row.view.source_card) || !schema182_facts_supported(&row.facts)))
        || state.mobilize.is_some() || state.miracle.is_some()
        || state.adventures.as_ref().is_some_and(|a| a.permissions.iter().any(|v| schema182_new_card(v.card)))
        || state.plot.as_ref().and_then(|p| p.payment.as_ref()).is_some_and(|p| schema182_new_card(p.source_card))
        || state.hits.iter().any(|hit| !schema182_facts_supported(&hit.source.facts))
        || state.damage_choices.iter().any(|hit| !schema182_facts_supported(&hit.source.facts))
        || state.manifest.as_ref().is_some_and(|manifest| {
            manifest.permanents.iter().chain(manifest.spells.iter())
                .any(|row| schema182_new_card(row.underlying))
        })
        || state.copy_recipes.as_ref().is_some_and(|rows| rows.iter().any(|row| !schema182_recipe_supported(&row.recipe)))
        || state.temporary_copies.as_ref().is_some_and(|rows| rows.iter().any(|row| {
            schema182_new_card(row.physical_card)
                || row.original_recipe.as_ref().is_some_and(|r| !schema182_recipe_supported(r))
                || row.layers.iter().any(|layer| !schema182_recipe_supported(&layer.value.recipe))
        }))
    {
        return false;
    }
    if let Some(mana) = &state.mana_lots {
        let copy_safe = |source: &mana_lots::ManaCopySource| {
            !schema182_new_card(source.card)
                && !schema182_new_card(source.definition)
                && schema182_facts_supported(&source.facts)
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
            || !prior_schema182_state_supported(&rollback.before))
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
        .all(schema182_packet_supported)
}

#[test]
fn schema182_rejects_live_static_grant_without_rejecting_schema181_fields() {
    let safe = r#"[{"event_player":"P0","discard_only":true,"exile_after":true}]"#;
    assert_eq!(project_prior_schema182(safe).unwrap(), safe);
    assert!(
        project_prior_schema182(
            r#"[{"abilities":{"grants":[{"static_rule":"PlaneswalkerAttackLimit"}]}}]"#
        )
        .is_err()
    );
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let source = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        g.state.objects[source as usize].card = Card::from_name("Tomik, Orzhov Lawmage").unwrap();
        g.register_battlefield_sources(source);
        let walker = put(
            &mut g,
            player,
            "Ajani, Caller of the Pride",
            Zone::Battlefield,
        );
        g.state.objects[walker as usize].counters.loyalty = 5;
        for viewer in [Player::P0, Player::P1] {
            let encoded = serde_json::to_string(&g.observe(viewer)).unwrap();
            assert!(project_prior_schema182(&encoded).is_err());
        }
        assert!(!prior_schema182_state_supported(&g.state));
        let saved = g.snapshot();
        g.move_object(source, Zone::Graveyard);
        g.restore(&saved).unwrap();
        assert!(!prior_schema182_state_supported(&g.state));
    }
}

#[test]
fn schema182_checks_hidden_cards_retained_packets_and_rollback() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let hidden = put(&mut g, player, "Bear Cub", Zone::Hand);
        let bear = Card::from_name("Bear Cub").unwrap();
        let tomik = Card::from_name("Tomik, Orzhov Lawmage").unwrap();
        assert!(prior_schema182_state_supported(&g.state));
        g.state.objects[hidden as usize].card = tomik;
        assert!(!prior_schema182_state_supported(&g.state));
        g.begin_attack_rollback(player);
        g.state.objects[hidden as usize].card = bear;
        assert!(!prior_schema182_state_supported(&g.state));
        g.state.cast_rollback = None;
        assert!(prior_schema182_state_supported(&g.state));
        let source = put(&mut g, player, "Gideon's Memorial", Zone::Battlefield);
        let mut packet = g.activation_item(player, source, 0);
        packet.source.as_mut().expect("object-backed packet").card = tomik;
        g.state.pending_triggers.push(packet);
        assert!(!prior_schema182_state_supported(&g.state));
        g.state.pending_triggers.clear();
        assert!(prior_schema182_state_supported(&g.state));
        let object = g.object_ref(source);
        g.state.library_top.get_or_insert_default().reveal =
            Some(library_top::LibraryRevealEvent {
                sequence: 1,
                source: object,
                player,
                revealed: explore::RevealedLibraryCard {
                    object,
                    card: tomik,
                },
                drawn: None,
            });
        assert!(!prior_schema182_state_supported(&g.state));
        g.state.library_top.as_mut().unwrap().reveal = None;
        assert!(prior_schema182_state_supported(&g.state));
        for name in ["Gideon the Oathless", "Vindictive Triumph"] {
            g.state.objects[hidden as usize].card = Card::from_name(name).unwrap();
            assert!(prior_schema182_state_supported(&g.state), "{name}");
            assert!(!prior_schema181_state_supported(&g.state), "{name}");
        }
    }
}

/// These authored rules first became representable in schema 181.
fn schema181_new_card(card: Card) -> bool {
    matches!(
        card.identity(),
        "Gideon the Oathless" | "Vindictive Triumph"
    )
}

fn schema181_recipe_supported(recipe: &CopyRecipe) -> bool {
    if schema181_new_card(recipe.definition) {
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
            .any(schema181_new_card),
        _ => true,
    }
}

fn schema181_facts_supported(facts: &DamageFacts) -> bool {
    // Schema 180 already represents additive Construct and other subtype facts.
    !schema181_new_card(facts.card)
}

/// Inspect all packet fields carrying card definitions, recipes or LKI subtype
/// overlays. Reference-only and numeric fields cannot contain the new rules.
fn schema181_packet_supported(item: &SpellView) -> bool {
    let Some(source) = item.source else {
        return false;
    };
    if !serde_json::to_string(item).is_ok_and(|json| project_prior_schema181(&json).is_ok())
        || schema181_new_card(source.card)
        || !schema181_facts_supported(&source.facts)
        || matches!(item.kind, StackKind::DelayedTokenCreation { token, .. } if schema181_new_card(token))
        || item
            .source_copy_recipe
            .as_deref()
            .is_some_and(|r| !schema181_recipe_supported(r))
        || item
            .captured_artifact_copy
            .as_deref()
            .is_some_and(|r| !schema181_recipe_supported(r))
        || item
            .attachment_trigger
            .as_ref()
            .is_some_and(|v| schema181_new_card(v.granting_card))
        || item
            .delayed_sacrifice
            .as_ref()
            .is_some_and(|v| schema181_new_card(v.source_card))
        || item
            .delayed_warp_exile
            .as_ref()
            .is_some_and(|v| schema181_new_card(v.source_card))
        || item
            .cast_event
            .as_ref()
            .is_some_and(|v| schema181_new_card(v.card))
        || item
            .cast
            .and_then(|v| v.physical_card)
            .is_some_and(schema181_new_card)
        || item.mana_copy_sources.as_ref().is_some_and(|sources| {
            sources.iter().any(|(v, _)| {
                schema181_new_card(v.card)
                    || schema181_new_card(v.definition)
                    || !schema181_facts_supported(&v.facts)
            })
        })
        || item
            .copy_event
            .as_ref()
            .is_some_and(|event| !schema181_packet_supported(&event.spell))
    {
        return false;
    }
    if let Some(event) = &item.event_object {
        if event
            .subtype_lki
            .as_ref()
            .is_some_and(|facts| !schema181_facts_supported(facts))
        {
            return false;
        }
        match &event.facts {
            event_objects::EventObjectFacts::DamageActor(source)
                if !schema181_facts_supported(&source.facts) =>
            {
                return false;
            }
            event_objects::EventObjectFacts::Copiable(recipe)
                if !schema181_recipe_supported(recipe) =>
            {
                return false;
            }
            _ => {}
        }
    }
    true
}

/// Conservatively reject opaque continuations, while checking copy layers and
/// retained packets by type. Rollback snapshots retain independent hidden state.
fn prior_schema181_state_supported(state: &State) -> bool {
    if state.resolving.is_some() || state.announcing.is_some()
        || state.mana_announcement_stack.is_some() || state.replacements.is_some()
        || state.entry_batch.is_some() || state.zone_rules.as_ref().is_some_and(|z| z.has_checkpoint())
        || state.linked_choices.as_ref().is_some_and(|c| c.has_opaque_interaction())
        || state.objects.iter().any(|o| schema181_new_card(o.card))
        || state.explore_data.as_ref().and_then(|data| data.last.as_ref()).is_some_and(|event|
            schema181_new_card(event.explorer.card)
                || event.revealed.is_some_and(|card| schema181_new_card(card.card)))
        || state.last_target_library_move.as_ref().is_some_and(|event| schema181_new_card(event.card))
        || state.library_top.as_ref().and_then(|data| data.reveal.as_ref())
            .is_some_and(|event| schema181_new_card(event.revealed.card))
        || state.temporary_abilities.as_ref().is_some_and(|rows| rows.iter().any(|row| schema181_new_card(row.granting_card)))
        || state.self_granted_abilities.as_ref().is_some_and(|rows| rows.iter().any(|row| schema181_new_card(row.granting_card)))
        || state.graveyard_borrowing.as_ref().is_some_and(|rows| rows.iter().any(|row|
            schema181_new_card(row.provider) || schema181_new_card(row.definition)))
        // Slots retain an optional immutable donor definition; all other slot
        // fields are object references or numeric acquisition/use identities.
        || state.borrowed_slots.as_ref().is_some_and(|rows| rows.iter().any(|row|
            row.definition.is_some_and(schema181_new_card)))
        || state.delayed_returns.as_ref().is_some_and(|rows| rows.iter().any(|row| row.view.returning.exile_after))
        || state.warp.as_ref().is_some_and(|w| !w.delayed.is_empty())
        // Delayed sacrifice packets retain public source-card identity and
        // frozen damage facts, including hidden/copied subtype overlays.
        || state.delayed_sacrifices.as_ref().is_some_and(|rows| rows.iter().any(|row|
            schema181_new_card(row.view.source_card) || !schema181_facts_supported(&row.facts)))
        || state.mobilize.is_some() || state.miracle.is_some()
        || state.adventures.as_ref().is_some_and(|a| a.permissions.iter().any(|v| schema181_new_card(v.card)))
        || state.plot.as_ref().and_then(|p| p.payment.as_ref()).is_some_and(|p| schema181_new_card(p.source_card))
        || state.hits.iter().any(|hit| !schema181_facts_supported(&hit.source.facts))
        || state.damage_choices.iter().any(|hit| !schema181_facts_supported(&hit.source.facts))
        || state.manifest.as_ref().is_some_and(|manifest| {
            manifest.permanents.iter().chain(manifest.spells.iter())
                .any(|row| schema181_new_card(row.underlying))
        })
        || state.copy_recipes.as_ref().is_some_and(|rows| rows.iter().any(|row| !schema181_recipe_supported(&row.recipe)))
        || state.temporary_copies.as_ref().is_some_and(|rows| rows.iter().any(|row| {
            schema181_new_card(row.physical_card)
                || row.original_recipe.as_ref().is_some_and(|r| !schema181_recipe_supported(r))
                || row.layers.iter().any(|layer| !schema181_recipe_supported(&layer.value.recipe))
        }))
    {
        return false;
    }
    if let Some(mana) = &state.mana_lots {
        let copy_safe = |source: &mana_lots::ManaCopySource| {
            !schema181_new_card(source.card)
                && !schema181_new_card(source.definition)
                && schema181_facts_supported(&source.facts)
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
            || !prior_schema181_state_supported(&rollback.before))
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
        .all(schema181_packet_supported)
}

#[test]
fn schema181_accepts_schema180_cards_and_additive_subtypes() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let hidden = put(&mut g, player, "Bear Cub", Zone::Hand);
        for name in ["Puppet Crafting", "Variable Chaser // Arc of Fortune"] {
            g.state.objects[hidden as usize].card = Card::from_name(name).unwrap();
            assert!(prior_schema181_state_supported(&g.state), "{name}");
            assert!(!prior_schema180_state_supported(&g.state), "{name}");
        }
        g.state.objects[hidden as usize].card = Card::from_name("Bear Cub").unwrap();
        let target = put(&mut g, player, "Gideon's Memorial", Zone::Battlefield);
        let aura = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        g.state.objects[aura as usize].card = Card::from_name("Puppet Crafting").unwrap();
        g.register_battlefield_sources(aura);
        g.state.objects[aura as usize].attached_to = Some(g.object_ref(target));
        for viewer in [Player::P0, Player::P1] {
            let encoded = serde_json::to_string(&g.observe(viewer)).unwrap();
            assert_eq!(project_prior_schema181(&encoded).unwrap(), encoded);
            assert!(project_prior_schema180(&encoded).is_err());
        }
        assert!(prior_schema181_state_supported(&g.state));
        let mut packet = g.activation_item(player, target, 0);
        packet.source.as_mut().expect("object-backed packet").facts = g.damage_source(target).facts;
        g.state.pending_triggers.push(packet);
        assert!(prior_schema181_state_supported(&g.state));
    }
}

#[test]
fn schema180_and_181_check_retained_library_reveal_card() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let id = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        let object = g.object_ref(id);
        let bear = Card::from_name("Bear Cub").unwrap();
        g.state.library_top.get_or_insert_default().reveal =
            Some(library_top::LibraryRevealEvent {
                sequence: 1,
                source: object,
                player,
                revealed: explore::RevealedLibraryCard { object, card: bear },
                drawn: None,
            });
        assert!(prior_schema180_state_supported(&g.state));
        assert!(prior_schema181_state_supported(&g.state));
        g.state
            .library_top
            .as_mut()
            .unwrap()
            .reveal
            .as_mut()
            .unwrap()
            .revealed
            .card = Card::from_name("Puppet Crafting").unwrap();
        assert!(!prior_schema180_state_supported(&g.state));
        assert!(prior_schema181_state_supported(&g.state));
        g.state
            .library_top
            .as_mut()
            .unwrap()
            .reveal
            .as_mut()
            .unwrap()
            .revealed
            .card = Card::from_name("Gideon the Oathless").unwrap();
        assert!(!prior_schema181_state_supported(&g.state));
        g.begin_attack_rollback(player);
        g.state
            .library_top
            .as_mut()
            .unwrap()
            .reveal
            .as_mut()
            .unwrap()
            .revealed
            .card = bear;
        assert!(!prior_schema181_state_supported(&g.state));
        g.state.cast_rollback = None;
        assert!(prior_schema181_state_supported(&g.state));
    }
}

#[test]
fn schema181_rejects_nested_new_fields_without_reserializing_safe_bytes() {
    let safe = r#"[{"text":"event_player", "nested":{"counter":0}}]"#;
    assert_eq!(project_prior_schema181(safe).unwrap(), safe);
    for input in [
        r#"[{"stack":[{"copy_event":{"spell":{"event_player":"P0"}}}]}]"#,
        r#"[{"payment":{"discard_only":true}}]"#,
        r#"[{"delayed":{"returning":{"exile_after":true}}}]"#,
    ] {
        assert!(project_prior_schema181(input).is_err(), "{input}");
    }
}

#[test]
fn schema181_rejects_owned_packets_and_rollback_for_both_viewers() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let source = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        for viewer in [Player::P0, Player::P1] {
            let encoded = serde_json::to_string(&g.observe(viewer)).unwrap();
            assert_eq!(project_prior_schema181(&encoded).unwrap(), encoded);
        }
        assert!(prior_schema181_state_supported(&g.state));
        let mut packet = g.activation_item(player, source, 0);
        g.state.pending_triggers.push(packet.clone());
        assert!(prior_schema181_state_supported(&g.state));
        g.state.pending_triggers.clear();
        let original = CopyRecipe::new(Card::from_name("Bear Cub").unwrap());
        g.set_copy_recipe(g.object_ref(source), original);
        assert!(prior_schema181_state_supported(&g.state));
        let hidden = original.with_faces(copies::CopyFaces::Double {
            physical_front: Card::from_name("Gideon the Oathless").unwrap(),
            front: original.definition,
            back: original.definition,
            back_up: false,
        });
        g.set_copy_recipe(g.object_ref(source), hidden);
        assert!(!prior_schema181_state_supported(&g.state));
        g.set_copy_recipe(g.object_ref(source), original);
        packet.event_player = Some(player);
        g.state.pending_triggers.push(packet);
        // Pending triggers may not appear in either public view.
        assert!(!prior_schema181_state_supported(&g.state));
        let saved = g.snapshot();
        g.begin_attack_rollback(player);
        g.state.pending_triggers.clear();
        assert!(!prior_schema181_state_supported(&g.state));
        g.state.cast_rollback = None;
        assert!(prior_schema181_state_supported(&g.state));
        g.restore(&saved).unwrap();
        assert!(!prior_schema181_state_supported(&g.state));
        g.state.pending_triggers.clear();
        let mut delayed = g.activation_item(player, source, 0);
        delayed.return_link = Some(Box::new(ReturnLinkView {
            exile_after: true,
            additional_returning: None,
            object: g.object_ref(source),
            zone: Zone::Battlefield,
            controller: player,
            counter: None,
            tapped: false,
        }));
        g.state.stack.push(delayed);
        for viewer in [Player::P0, Player::P1] {
            let encoded = serde_json::to_string(&g.observe(viewer)).unwrap();
            assert!(project_prior_schema181(&encoded).is_err());
            assert!(!prior_schema181_state_supported(&g.state));
        }
    }
}

#[test]
fn schema181_live_discard_ward_payment_is_not_projectable() {
    for payer in [Player::P0, Player::P1] {
        let mut g = scenario(payer);
        let gideon = put(&mut g, payer.other(), "Bear Cub", Zone::Hand);
        g.state.objects[gideon as usize].card = Card::from_name("Gideon the Oathless").unwrap();
        g.move_object(gideon, Zone::Battlefield);
        let spell = put(&mut g, payer, "Shock", Zone::Hand);
        let secret = put(&mut g, payer, "Forest", Zone::Hand);
        cast(&mut g, spell, Some(Target::Card(gideon)));
        let id = g
            .state
            .stack
            .iter()
            .find(|s| s.source.is_some_and(|source| source.id == spell))
            .unwrap()
            .id;
        assert!(g.counter_stack_item(id, false));
        g.resolve_top();
        assert!(matches!(g.state.pending, Pending::ResolutionMana { player } if player == payer));
        assert!(g.observe(payer).resolution_payment.unwrap().discard_only);
        for viewer in [Player::P0, Player::P1] {
            assert!(!prior_schema181_state_supported(&g.state));
            let view = serde_json::to_string(&g.observe(viewer)).unwrap();
            if viewer == payer {
                assert!(project_prior_schema181(&view).is_err());
            }
        }
        let saved = g.snapshot();
        act(&mut g, Action::Discard { card: secret });
        g.restore(&saved).unwrap();
        assert!(!prior_schema181_state_supported(&g.state));
    }
}

/// Schema 180 exposes optional wheel choices and adds an additive Construct
/// subtype grant. Neither active feature has a schema-179 continuation.
fn project_prior_schema180(input: &str) -> Result<String, String> {
    fn check(value: &serde_json::Value) -> Result<(), String> {
        match value {
            serde_json::Value::Array(rows) => {
                for row in rows {
                    check(row)?;
                }
            }
            serde_json::Value::Object(map) => {
                if map.contains_key("optional_wheel_choices") {
                    return Err("optional wheel has no schema-179 representation".into());
                }
                if map
                    .get("added")
                    .and_then(|v| v.as_array())
                    .is_some_and(|rows| rows.iter().any(|v| v.as_str() == Some("Construct")))
                {
                    return Err("added Construct subtype has no schema-179 representation".into());
                }
                for value in map.values() {
                    check(value)?;
                }
            }
            _ => (),
        }
        Ok(())
    }
    let parsed: serde_json::Value = serde_json::from_str(input).map_err(|e| e.to_string())?;
    check(&parsed)?;
    // Inactive fields are omitted by serde, so preserve the exact old bytes.
    Ok(input.to_owned())
}

/// Schema-180 rules are carried by these authored card definitions.
fn schema180_new_card(card: Card) -> bool {
    card.aura_artifact_or_non_aura_enchantment()
        || card.identity() == "Variable Chaser // Arc of Fortune"
        || card.name() == "Arc of Fortune"
}

fn schema180_recipe_supported(recipe: &CopyRecipe) -> bool {
    if schema180_new_card(recipe.definition) {
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
            .any(schema180_new_card),
        _ => true,
    }
}

fn schema180_facts_supported(facts: &DamageFacts) -> bool {
    !schema180_new_card(facts.card)
        && serde_json::to_string(&facts.granted_subtypes)
            .is_ok_and(|json| project_prior_schema180(&json).is_ok())
}

/// Inspect all packet fields carrying card definitions, recipes or LKI subtype
/// overlays. Reference-only and numeric fields cannot contain the new rules.
fn schema180_packet_supported(item: &SpellView) -> bool {
    let Some(source) = item.source else {
        return false;
    };
    if schema180_new_card(source.card)
        || !schema180_facts_supported(&source.facts)
        || matches!(item.kind, StackKind::DelayedTokenCreation { token, .. } if schema180_new_card(token))
        || item
            .source_copy_recipe
            .as_deref()
            .is_some_and(|r| !schema180_recipe_supported(r))
        || item
            .captured_artifact_copy
            .as_deref()
            .is_some_and(|r| !schema180_recipe_supported(r))
        || item
            .attachment_trigger
            .as_ref()
            .is_some_and(|v| schema180_new_card(v.granting_card))
        || item
            .delayed_sacrifice
            .as_ref()
            .is_some_and(|v| schema180_new_card(v.source_card))
        || item
            .delayed_warp_exile
            .as_ref()
            .is_some_and(|v| schema180_new_card(v.source_card))
        || item
            .cast_event
            .as_ref()
            .is_some_and(|v| schema180_new_card(v.card))
        || item
            .cast
            .and_then(|v| v.physical_card)
            .is_some_and(schema180_new_card)
        || item.mana_copy_sources.as_ref().is_some_and(|sources| {
            sources.iter().any(|(v, _)| {
                schema180_new_card(v.card)
                    || schema180_new_card(v.definition)
                    || !schema180_facts_supported(&v.facts)
            })
        })
        || item
            .copy_event
            .as_ref()
            .is_some_and(|event| !schema180_packet_supported(&event.spell))
    {
        return false;
    }
    if let Some(event) = &item.event_object {
        if event
            .subtype_lki
            .as_ref()
            .is_some_and(|facts| !schema180_facts_supported(facts))
        {
            return false;
        }
        match &event.facts {
            event_objects::EventObjectFacts::DamageActor(source)
                if !schema180_facts_supported(&source.facts) =>
            {
                return false;
            }
            event_objects::EventObjectFacts::Copiable(recipe)
                if !schema180_recipe_supported(recipe) =>
            {
                return false;
            }
            _ => {}
        }
    }
    true
}

/// Conservatively reject opaque continuations, while checking copy layers and
/// retained packets by type. Rollback snapshots retain independent hidden state.
fn prior_schema180_state_supported(state: &State) -> bool {
    if state.resolving.is_some() || state.announcing.is_some()
        || state.mana_announcement_stack.is_some() || state.replacements.is_some()
        || state.entry_batch.is_some() || state.zone_rules.as_ref().is_some_and(|z| z.has_checkpoint())
        || state.linked_choices.as_ref().is_some_and(|c| c.has_opaque_interaction())
        || state.objects.iter().any(|o| schema180_new_card(o.card))
        || state.explore_data.as_ref().and_then(|data| data.last.as_ref()).is_some_and(|event|
            schema180_new_card(event.explorer.card)
                || event.revealed.is_some_and(|card| schema180_new_card(card.card)))
        || state.last_target_library_move.as_ref().is_some_and(|event| schema180_new_card(event.card))
        || state.library_top.as_ref().and_then(|data| data.reveal.as_ref())
            .is_some_and(|event| schema180_new_card(event.revealed.card))
        || state.temporary_abilities.as_ref().is_some_and(|rows| rows.iter().any(|row| schema180_new_card(row.granting_card)))
        || state.self_granted_abilities.as_ref().is_some_and(|rows| rows.iter().any(|row| schema180_new_card(row.granting_card)))
        || state.graveyard_borrowing.as_ref().is_some_and(|rows| rows.iter().any(|row|
            schema180_new_card(row.provider) || schema180_new_card(row.definition)))
        // Slots retain an optional immutable donor definition; all other slot
        // fields are object references or numeric acquisition/use identities.
        || state.borrowed_slots.as_ref().is_some_and(|rows| rows.iter().any(|row|
            row.definition.is_some_and(schema180_new_card)))
        || state.warp.as_ref().is_some_and(|w| !w.delayed.is_empty())
        // Delayed sacrifice packets retain public source-card identity and
        // frozen damage facts, including hidden/copied subtype overlays.
        || state.delayed_sacrifices.as_ref().is_some_and(|rows| rows.iter().any(|row|
            schema180_new_card(row.view.source_card) || !schema180_facts_supported(&row.facts)))
        || state.mobilize.is_some() || state.miracle.is_some()
        || state.adventures.as_ref().is_some_and(|a| a.permissions.iter().any(|v| schema180_new_card(v.card)))
        || state.plot.as_ref().and_then(|p| p.payment.as_ref()).is_some_and(|p| schema180_new_card(p.source_card))
        || state.hits.iter().any(|hit| !schema180_facts_supported(&hit.source.facts))
        || state.damage_choices.iter().any(|hit| !schema180_facts_supported(&hit.source.facts))
        || state.manifest.as_ref().is_some_and(|manifest| {
            manifest.permanents.iter().chain(manifest.spells.iter())
                .any(|row| schema180_new_card(row.underlying))
        })
        || state.copy_recipes.as_ref().is_some_and(|rows| rows.iter().any(|row| !schema180_recipe_supported(&row.recipe)))
        || state.temporary_copies.as_ref().is_some_and(|rows| rows.iter().any(|row| {
            schema180_new_card(row.physical_card)
                || row.original_recipe.as_ref().is_some_and(|r| !schema180_recipe_supported(r))
                || row.layers.iter().any(|layer| !schema180_recipe_supported(&layer.value.recipe))
        }))
    {
        return false;
    }
    if let Some(mana) = &state.mana_lots {
        let copy_safe = |source: &mana_lots::ManaCopySource| {
            !schema180_new_card(source.card)
                && !schema180_new_card(source.definition)
                && schema180_facts_supported(&source.facts)
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
            || !prior_schema180_state_supported(&rollback.before))
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
        .all(schema180_packet_supported)
}

#[test]
fn schema180_checks_retained_historical_card_identities_and_checkpoints() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let id = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        let object = g.object_ref(id);
        let bear = Card::from_name("Bear Cub").unwrap();
        let puppet = Card::from_name("Puppet Crafting").unwrap();
        let data = g.state.explore_data.get_or_insert_default();
        data.last = Some(explore::ExploreEventView {
            sequence: 1,
            explorer: explore::ExplorerView {
                kinds: 1,
                object,
                card: bear,
                controller: player,
            },
            revealed: Some(explore::RevealedLibraryCard { object, card: bear }),
            destination: None,
            result: None,
            counter_added: false,
        });
        assert!(prior_schema180_state_supported(&g.state));
        g.state
            .explore_data
            .as_mut()
            .unwrap()
            .last
            .as_mut()
            .unwrap()
            .explorer
            .card = puppet;
        assert!(!prior_schema180_state_supported(&g.state));
        g.begin_attack_rollback(player);
        g.state
            .explore_data
            .as_mut()
            .unwrap()
            .last
            .as_mut()
            .unwrap()
            .explorer
            .card = bear;
        assert!(!prior_schema180_state_supported(&g.state));
        g.state.cast_rollback = None;
        assert!(prior_schema180_state_supported(&g.state));
        g.state
            .explore_data
            .as_mut()
            .unwrap()
            .last
            .as_mut()
            .unwrap()
            .revealed
            .as_mut()
            .unwrap()
            .card = puppet;
        assert!(!prior_schema180_state_supported(&g.state));
        g.state.explore_data.as_mut().unwrap().last = None;
        g.state.last_target_library_move = Some(Box::new(TargetLibraryMovement {
            source: object,
            card: bear,
            target_before: object,
            target_after: object,
            owner: player,
            destination: LibraryDestination::Top,
        }));
        assert!(prior_schema180_state_supported(&g.state));
        g.state.last_target_library_move.as_mut().unwrap().card = puppet;
        assert!(!prior_schema180_state_supported(&g.state));
    }
}

#[test]
fn schema180_inspects_hidden_copy_faces_and_nested_packets() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let id = put(&mut g, player, "Bear Cub", Zone::Hand);
        let object = g.object_ref(id);
        let bear = Card::from_name("Bear Cub").unwrap();
        let puppet = Card::from_name("Puppet Crafting").unwrap();
        let ordinary = CopyRecipe::new(bear);
        g.set_copy_recipe(object, ordinary);
        assert!(prior_schema180_state_supported(&g.state));
        let hidden = ordinary.with_faces(copies::CopyFaces::Double {
            physical_front: puppet,
            front: bear,
            back: bear,
            back_up: false,
        });
        g.set_copy_recipe(object, hidden);
        assert!(!prior_schema180_state_supported(&g.state));
        g.begin_attack_rollback(player);
        g.set_copy_recipe(object, ordinary);
        assert!(!prior_schema180_state_supported(&g.state));
        g.state.cast_rollback = None;
        assert!(prior_schema180_state_supported(&g.state));

        let source = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        let packet = g.activation_item(player, source, 0);
        g.state.stack.push(packet.clone());
        assert!(prior_schema180_state_supported(&g.state));
        g.state.stack.clear();
        let mut hidden_fact = packet.clone();
        hidden_fact
            .source
            .as_mut()
            .expect("object-backed packet")
            .facts
            .card = puppet;
        g.state.stack.push(hidden_fact);
        assert!(!prior_schema180_state_supported(&g.state));
        g.state.stack.clear();
        let mut hidden_packet = packet.clone();
        hidden_packet.source_copy_recipe = Some(Box::new(hidden));
        let mut wrapper = packet.clone();
        wrapper.copy_event = Some(Box::new(stack_copies::CopyEvent {
            spell: hidden_packet.clone(),
            amount: 1,
        }));
        g.state.pending_triggers.push(wrapper);
        assert!(!prior_schema180_state_supported(&g.state));
        g.state.pending_triggers.clear();
        g.state.stack_copy_data = Some(Box::new(stack_copies::CopyState::test_retarget(
            packet.clone(),
            packet.clone(),
        )));
        assert!(prior_schema180_state_supported(&g.state));
        g.state.stack_copy_data = Some(Box::new(stack_copies::CopyState::test_retarget(
            packet.clone(),
            hidden_packet.clone(),
        )));
        assert!(!prior_schema180_state_supported(&g.state));
        g.state.stack_copy_data = None;
        g.state
            .manifest
            .get_or_insert_default()
            .deaths
            .push(manifest::ManifestDeath {
                object,
                source: hidden_packet,
            });
        assert!(!prior_schema180_state_supported(&g.state));
    }
}

#[test]
fn schema180_rejects_hidden_manifest_underlying_in_live_and_rollback_state() {
    for player in [Player::P0, Player::P1] {
        for name in ["Puppet Crafting", "Variable Chaser // Arc of Fortune"] {
            let mut g = scenario(player);
            let id = put(&mut g, player, "Bear Cub", Zone::Library);
            let physical = Card::from_name(name).unwrap();
            g.state.objects[id as usize].card = physical;
            let movement = g.manifest_entry(g.object_ref(id), player).unwrap();
            g.stage_entries(vec![movement], None);
            assert!(!g.advance_entries());
            assert!(g.is_face_down(g.object_ref(id)));
            assert_ne!(g.state.objects[id as usize].card, physical);
            for viewer in [player, player.other()] {
                let view = serde_json::to_string(&g.observe(viewer)).unwrap();
                assert!(project_prior_schema180(&view).is_ok());
            }
            assert!(!prior_schema180_state_supported(&g.state));
            let saved = g.snapshot();
            g.begin_attack_rollback(player);
            // Change the live underlying face while retaining its old value in
            // the checkpoint; prior-schema compatibility must inspect both.
            g.state.manifest.as_mut().unwrap().permanents[0].underlying =
                Card::from_name("Bear Cub").unwrap();
            assert!(!prior_schema180_state_supported(&g.state));
            g.state.cast_rollback = None;
            assert!(prior_schema180_state_supported(&g.state));
            g.restore(&saved).unwrap();
            assert!(!prior_schema180_state_supported(&g.state));
            // Exercise the separate face-down spell storage contract too.
            let manifest = g.state.manifest.as_mut().unwrap();
            let face = manifest.permanents.pop().unwrap();
            manifest.spells.push(face);
            assert!(!prior_schema180_state_supported(&g.state));
        }
    }
}

#[test]
fn schema180_audits_delayed_sacrifice_cards_and_frozen_construct_facts() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let bear = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        let molten = put(&mut g, player, "Molten Duplication", Zone::Hand);
        cast(&mut g, molten, Some(Target::Card(bear)));
        settle(&mut g);
        assert_eq!(g.state.delayed_sacrifices.as_ref().unwrap().len(), 1);
        assert!(prior_schema180_state_supported(&g.state));
        let safe = g.snapshot();
        let newer = Card::from_name("Puppet Crafting").unwrap();
        g.state.delayed_sacrifices.as_mut().unwrap()[0]
            .view
            .source_card = newer;
        assert!(!prior_schema180_state_supported(&g.state));
        g.restore(&safe).unwrap();
        g.state.delayed_sacrifices.as_mut().unwrap()[0].facts.card = newer;
        assert!(!prior_schema180_state_supported(&g.state));
        g.restore(&safe).unwrap();
        // Obtain an actual additive Construct overlay, then remove the source;
        // the old packet's frozen facts remain independently meaningful.
        let reference = g.object_ref(bear);
        let timestamp = g.next_role_timestamp();
        g.state.roles.get_or_insert_default().creature_types.push(
            subtypes::CreatureTypesEffectView {
                recipient: reference,
                source: reference,
                types: &["Construct"],
                additional: true,
                timestamp,
            },
        );
        let facts = g.damage_facts(bear);
        assert!(facts.granted_subtypes.contains("Construct"));
        g.state.roles.as_mut().unwrap().creature_types.clear();
        g.state.delayed_sacrifices.as_mut().unwrap()[0].facts = facts;
        assert!(!prior_schema180_state_supported(&g.state));
        g.begin_attack_rollback(player);
        let safe_facts = g.damage_facts(bear);
        g.state.delayed_sacrifices.as_mut().unwrap()[0].facts = safe_facts;
        assert!(!prior_schema180_state_supported(&g.state));
        g.state.cast_rollback = None;
        assert!(prior_schema180_state_supported(&g.state));
    }
}

#[test]
fn schema180_audits_hidden_borrowed_slot_definitions_and_rollbacks() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let forge = put(&mut g, player, "Territory Forge", Zone::Battlefield);
        let donor = put(&mut g, player, "Hedron Archive", Zone::Exile);
        let source = g.object_ref(forge);
        let exiled = g.object_ref(donor);
        g.state
            .borrowed_links
            .get_or_insert_default()
            .push(borrowed_activations::Link {
                source,
                origin: None,
                exiled,
                first_slot: 0,
                width: 1,
            });
        g.state.borrowed_slots = borrowed_activations::refreshed_slots(&g);
        let slots = g.state.borrowed_slots.as_ref().unwrap();
        assert!(slots.iter().any(|row| row.definition.is_some()));
        assert!(prior_schema180_state_supported(&g.state));
        let saved = g.snapshot();
        let safe_slots = g.state.borrowed_slots.clone();
        let slot = g
            .state
            .borrowed_slots
            .as_mut()
            .unwrap()
            .iter_mut()
            .find(|row| row.definition.is_some())
            .unwrap();
        slot.definition = Some(Card::from_name("Puppet Crafting").unwrap());
        assert!(!prior_schema180_state_supported(&g.state));
        g.begin_attack_rollback(player);
        g.state.borrowed_slots = safe_slots;
        assert!(!prior_schema180_state_supported(&g.state));
        g.state.cast_rollback = None;
        assert!(prior_schema180_state_supported(&g.state));
        g.restore(&saved).unwrap();
        assert!(prior_schema180_state_supported(&g.state));
    }
}

#[test]
fn schema180_projection_rejects_new_semantics_and_preserves_bytes() {
    let safe = r#"[{"added":["Spirit"],"text":"Construct"}]"#;
    assert_eq!(project_prior_schema180(safe).unwrap(), safe);
    assert!(project_prior_schema180(r#"[{"added":["Construct"]}]"#).is_err());
    assert!(project_prior_schema180(r#"[{"optional_wheel_choices":[null,true]}]"#).is_err());
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        for viewer in [Player::P0, Player::P1] {
            let view = serde_json::to_string(&g.observe(viewer)).unwrap();
            assert_eq!(project_prior_schema180(&view).unwrap(), view);
        }
        assert!(prior_schema180_state_supported(&g.state));
        let hidden = put(&mut g, player, "Bear Cub", Zone::Hand);
        g.state.objects[hidden as usize].card = Card::from_name("Puppet Crafting").unwrap();
        assert!(!prior_schema180_state_supported(&g.state));
        for viewer in [Player::P0, Player::P1] {
            let view = serde_json::to_string(&g.observe(viewer)).unwrap();
            assert!(project_prior_schema180(&view).is_ok());
            assert!(!prior_schema180_state_supported(&g.state));
        }
    }
}

#[test]
fn schema180_rejects_live_wheel_and_construct_for_both_viewers_and_restore() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let target = put(&mut g, player, "Gideon's Memorial", Zone::Battlefield);
        let aura = put(&mut g, player, "Bear Cub", Zone::Hand);
        g.state.objects[aura as usize].card = Card::from_name("Puppet Crafting").unwrap();
        g.move_object(aura, Zone::Battlefield);
        g.state.objects[aura as usize].attached_to = Some(g.object_ref(target));
        assert!(g.has_subtype(target, "Construct"));
        for viewer in [Player::P0, Player::P1] {
            let view = serde_json::to_string(&g.observe(viewer)).unwrap();
            assert!(project_prior_schema180(&view).is_err());
            assert!(!prior_schema180_state_supported(&g.state));
        }
        let saved = g.snapshot();
        g.move_object(aura, Zone::Graveyard);
        g.restore(&saved).unwrap();
        assert!(!prior_schema180_state_supported(&g.state));

        let mut g = scenario(player);
        let source = put(&mut g, player, "Bear Cub", Zone::Hand);
        g.state.objects[source as usize].card =
            Card::from_name("Variable Chaser // Arc of Fortune").unwrap();
        g.state.players[player.index()].mana = [0, 1, 0, 0, 0, 2];
        act(&mut g, Action::AnnounceCast { card: source });
        pay(&mut g);
        for _ in 0..96 {
            g.prepare_triggers();
            if g.state.pending == Pending::Priority
                && g.state.stack.is_empty()
                && g.state.pending_triggers.is_empty()
                && g.state.resolving.is_none()
            {
                break;
            }
            choose(&mut g, |a| {
                matches!(a, Action::Pass | Action::PutTrigger { .. })
            });
        }
        let prepared = g
            .state
            .preparations
            .as_ref()
            .unwrap()
            .iter()
            .find(|row| row.permanent == g.object_ref(source))
            .unwrap()
            .spell_copy;
        g.state.players[player.index()].mana = [0, 1, 0, 0, 0, 2];
        act(
            &mut g,
            Action::AnnounceExile {
                card: prepared.card,
            },
        );
        pay(&mut g);
        for _ in 0..64 {
            if matches!(g.state.pending, Pending::EffectOptional { .. }) {
                break;
            }
            choose(&mut g, |a| {
                matches!(a, Action::Pass | Action::PutTrigger { .. })
            });
        }
        assert!(matches!(g.state.pending, Pending::EffectOptional { .. }));
        let saved = g.snapshot();
        for viewer in [Player::P0, Player::P1] {
            let view = serde_json::to_string(&g.observe(viewer)).unwrap();
            assert!(project_prior_schema180(&view).is_err());
            assert!(!prior_schema180_state_supported(&g.state));
        }
        act(&mut g, Action::ChooseOptional { accept: true });
        g.restore(&saved).unwrap();
        assert!(!prior_schema180_state_supported(&g.state));
    }
}

/// Schema 179 adds a copiable conditional sacrifice ability. A zero field is
/// byte-projectable; a live ability or its stack packet has no old meaning.
fn project_prior_schema179_copy_abilities(input: &str) -> Result<String, String> {
    fn check(value: &serde_json::Value) -> Result<(), String> {
        match value {
            serde_json::Value::Array(rows) => {
                for row in rows {
                    check(row)?;
                }
            }
            serde_json::Value::Object(map) => {
                for (key, value) in map {
                    if key == "conditional_sacrifice_abilities" && value.as_u64() != Some(0) {
                        return Err(
                            "conditional copy ability has no prior-schema representation".into(),
                        );
                    }
                    if key == "CopyEndStepConditionalSacrifice" {
                        return Err(
                            "conditional copy trigger has no prior-schema representation".into(),
                        );
                    }
                    check(value)?;
                }
            }
            serde_json::Value::String(value) if value == "CopyEndStepConditionalSacrifice" => {
                return Err("conditional copy trigger has no prior-schema representation".into());
            }
            _ => (),
        }
        Ok(())
    }
    let parsed: serde_json::Value = serde_json::from_str(input).map_err(|e| e.to_string())?;
    check(&parsed)?;
    // Preserve the exact compact serde bytes, including key order and escaped text.
    let bytes = input.as_bytes();
    let mut result = String::with_capacity(input.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] != b'"' {
            let ch = input[i..].chars().next().unwrap();
            result.push(ch);
            i += ch.len_utf8();
            continue;
        }
        let start = i;
        i += 1;
        while i < bytes.len() {
            if bytes[i] == b'\\' {
                i += 2;
            } else if bytes[i] == b'"' {
                i += 1;
                break;
            } else {
                i += 1;
            }
        }
        let key = &input[start..i];
        if key == "\"conditional_sacrifice_abilities\"" && input[i..].starts_with(":0") {
            i += 2;
            if bytes.get(i) == Some(&b',') {
                i += 1;
            } else if bytes.get(i) == Some(&b'}') {
                if result.ends_with(',') {
                    result.pop();
                }
            } else {
                return Err("unexpected schema-179 field encoding".into());
            }
        } else {
            result.push_str(key);
        }
    }
    Ok(result)
}

/// Audit accessible copy recipes and spell packets explicitly. Opaque suspended
/// continuations can retain private recipes; reject them rather than guessing.
fn prior_schema179_copy_owned_state_supported(game: &Game) -> bool {
    prior_schema179_copy_state_supported(&game.state)
}

fn prior_schema179_copy_state_supported(state: &State) -> bool {
    if state.replacements.is_some()
        || state.entry_batch.is_some()
        || state.resolving.is_some()
        || state.announcing.is_some()
        || state.mana_announcement_stack.is_some()
        || state
            .zone_rules
            .as_ref()
            .is_some_and(|z| z.has_checkpoint())
        || state
            .linked_choices
            .as_ref()
            .is_some_and(|c| c.has_opaque_interaction())
    {
        return false;
    }
    if let Some(rollback) = &state.cast_rollback {
        // Mana recordings are opaque; an empty checkpoint can be checked recursively.
        if !rollback.events.is_empty()
            || rollback.replay.is_some()
            || rollback.capture.is_some()
            || !prior_schema179_copy_state_supported(&rollback.before)
        {
            return false;
        }
    }
    let safe = |recipe: &CopyRecipe| recipe.exceptions.conditional_sacrifice_abilities == 0;
    if state
        .copy_recipes
        .as_ref()
        .is_some_and(|rows| rows.iter().any(|row| !safe(&row.recipe)))
        || state.temporary_copies.as_ref().is_some_and(|rows| {
            rows.iter().any(|row| {
                row.original_recipe
                    .as_ref()
                    .is_some_and(|recipe| !safe(recipe))
                    || row.layers.iter().any(|layer| !safe(&layer.value.recipe))
            })
        })
    {
        return false;
    }
    // These owned containers can hold private SpellViews that observations omit.
    // SpellView serialization retains nested copy events, LKI recipes and kind;
    // skipped fields are unrelated trigger-count/target witnesses.
    let packet_safe = |item: &SpellView| {
        serde_json::to_string(item)
            .is_ok_and(|json| project_prior_schema179_copy_abilities(&json).is_ok())
    };
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
        .all(packet_safe)
}

#[test]
fn prior_schema179_inspects_passive_containers_and_hidden_packets() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        g.state.zone_rules = Some(Box::default());
        g.state.linked_choices = Some(Box::default());
        g.state.stack_copy_data = Some(Box::default());
        g.state.manifest = Some(Box::default());
        assert!(prior_schema179_copy_owned_state_supported(&g));
        let source = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        let mut packet = g.activation_item(player, source, 0);
        let active =
            CopyRecipe::new(Card::from_name("Bear Cub").unwrap()).with_exceptions(CopyExceptions {
                conditional_sacrifice_abilities: 1,
                ..CopyExceptions::NONE
            });
        packet.source_copy_recipe = Some(Box::new(active));
        let reference = g.object_ref(source);
        g.state
            .manifest
            .as_mut()
            .unwrap()
            .deaths
            .push(manifest::ManifestDeath {
                object: reference,
                source: packet,
            });
        assert!(!prior_schema179_copy_owned_state_supported(&g));
        g.state.manifest.as_mut().unwrap().deaths.clear();
        g.begin_attack_rollback(player);
        assert!(prior_schema179_copy_owned_state_supported(&g));
        g.set_copy_recipe(reference, active);
        assert!(!prior_schema179_copy_owned_state_supported(&g));
    }
}

#[test]
fn prior_schema179_rejects_private_paradigm_copy_packets() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        assert!(prior_schema179_copy_owned_state_supported(&g));
        let source = put(&mut g, player, "Germination Practicum", Zone::Hand);
        let reference = g.object_ref(source);
        let mut item = g.cast_preview(player, source, Zone::Hand, reference, 0, [0; 6]);
        let recipe =
            CopyRecipe::new(Card::from_name("Bear Cub").unwrap()).with_exceptions(CopyExceptions {
                conditional_sacrifice_abilities: 1,
                ..CopyExceptions::NONE
            });
        g.resolve_paradigm(&item, 0);
        assert!(prior_schema179_copy_owned_state_supported(&g));
        g.state.paradigm = None;
        item.source_copy_recipe = Some(Box::new(recipe));
        // The delayed packet survives after the source packet and live recipe
        // are gone; the public paradigm view does not expose its recipe.
        g.resolve_paradigm(&item, 0);
        assert!(g.state.paradigm.is_some());
        assert!(g.state.stack.is_empty());
        assert!(g.state.copy_recipes.is_none());
        assert!(!prior_schema179_copy_owned_state_supported(&g));
        g.queue_paradigm_triggers();
        assert!(
            g.state
                .pending_triggers
                .iter()
                .any(|trigger| trigger.source_copy_recipe.as_deref() == Some(&recipe))
        );
    }
}

#[test]
fn prior_schema179_projection_preserves_bytes_and_rejects_new_semantics() {
    let input = r#"[{"text":"escaped \"conditional_sacrifice_abilities\":0,","recipe":{"conditional_sacrifice_abilities":0,"other":1}},{"conditional_sacrifice_abilities":0}]"#;
    let out = project_prior_schema179_copy_abilities(input).unwrap();
    assert_eq!(
        out,
        r#"[{"text":"escaped \"conditional_sacrifice_abilities\":0,","recipe":{"other":1}},{}]"#
    );
    for input in [
        r#"[{"conditional_sacrifice_abilities":1}]"#,
        r#"[{"conditional_sacrifice_abilities":null}]"#,
        r#"[{"conditional_sacrifice_abilities":false}]"#,
        r#"[{"kind":"CopyEndStepConditionalSacrifice"}]"#,
        r#"[{"kind":{"CopyEndStepConditionalSacrifice":{}}}]"#,
    ] {
        assert!(
            project_prior_schema179_copy_abilities(input).is_err(),
            "{input}"
        );
    }
}

#[test]
fn prior_schema179_projection_detects_hidden_recipes_rollback_and_trigger_packets() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let hidden = put(&mut g, player, "Bear Cub", Zone::Hand);
        let reference = g.object_ref(hidden);
        let zero = CopyRecipe::new(g.state.objects[hidden as usize].card);
        g.set_copy_recipe(reference, zero);
        let views = serde_json::to_string(&[g.observe(player), g.observe(player.other())]).unwrap();
        assert!(prior_schema179_copy_owned_state_supported(&g));
        assert!(project_prior_schema179_copy_abilities(&views).is_ok());
        let active = zero.with_exceptions(CopyExceptions {
            conditional_sacrifice_abilities: 1,
            ..CopyExceptions::NONE
        });
        g.set_copy_recipe(reference, active);
        assert!(!prior_schema179_copy_owned_state_supported(&g));
        // Checkpoint retains the hidden recipe even after the live row is removed.
        g.begin_attack_rollback(player);
        g.clear_copy_recipe(reference);
        assert!(!prior_schema179_copy_owned_state_supported(&g));
        g.state.cast_rollback = None;
        assert!(prior_schema179_copy_owned_state_supported(&g));
        let source = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        let source_ref = g.object_ref(source);
        g.set_copy_recipe(source_ref, zero);
        let actual =
            serde_json::to_string(&[g.observe(player), g.observe(player.other())]).unwrap();
        assert!(actual.contains("\"conditional_sacrifice_abilities\":0"));
        let old = project_prior_schema179_copy_abilities(&actual).unwrap();
        assert!(!old.contains("\"conditional_sacrifice_abilities\":"));
        assert!(prior_schema179_copy_owned_state_supported(&g));
        let mut packet = g.activation_item(player, source, 0);
        packet.source_copy_recipe = Some(Box::new(active));
        g.state.pending_triggers.push(packet.clone());
        assert!(!prior_schema179_copy_owned_state_supported(&g));
        g.state.pending_triggers.clear();
        packet.source_copy_recipe = None;
        packet.kind = StackKind::CopyEndStepConditionalSacrifice;
        assert!(
            project_prior_schema179_copy_abilities(&serde_json::to_string(&packet).unwrap())
                .is_err()
        );
        g.state.pending_triggers.push(packet);
        assert!(!prior_schema179_copy_owned_state_supported(&g));
        g.state.pending_triggers.clear();
        // A newer ordinary copy can hide an older incompatible copy layer.
        g.apply_temporary_copy(source_ref, active, CopyDeadline::EndOfTurn);
        g.apply_temporary_copy(source_ref, zero, CopyDeadline::EndOfTurn);
        assert!(!prior_schema179_copy_owned_state_supported(&g));
    }
}

/// Recording-only projection to the schema preceding behold and universal
/// basic-land mana. Reject cases whose old wire representation cannot preserve
/// the observed ability identity or land-subtype semantics.
fn project_prior_behold_json(input: &str) -> Result<String, String> {
    project_prior_behold_candidate(input, false)
}

fn project_prior_behold_candidate(input: &str, compare_mana: bool) -> Result<String, String> {
    fn check(value: &serde_json::Value, compare_mana: bool) -> Result<(), String> {
        match value {
            serde_json::Value::Array(items) => {
                for item in items {
                    check(item, compare_mana)?;
                }
            }
            serde_json::Value::Object(map) => {
                if map.get("kind").and_then(|v| v.as_str()) == Some("RemoveAbilitiesUntilEndOfTurn")
                {
                    return Err(
                        "temporary ability removal has no prior-schema representation".into(),
                    );
                }
                if map
                    .get("loyalty_ability_activated_this_turn")
                    .is_some_and(|v| v.as_bool() != Some(false))
                {
                    return Err(
                        "loyalty activation history has no prior-schema representation".into(),
                    );
                }
                if map
                    .get("next_spell_uncounterable")
                    .is_some_and(|v| v.as_bool() != Some(false))
                    || map
                        .get("uncounterable_spell_ids")
                        .is_some_and(|v| v.as_array().is_none_or(|a| !a.is_empty()))
                {
                    return Err("next-spell protection has no prior-schema representation".into());
                }
                for key in ["borrowed_abilities", "graveyard_borrowed_abilities"] {
                    if map
                        .get(key)
                        .is_some_and(|v| !v.is_null() && v.as_array().is_none_or(|a| !a.is_empty()))
                    {
                        return Err(format!(
                            "{key} uses new acquisition slots and ability indices"
                        ));
                    }
                }
                if map.get("borrowed_slot").is_some_and(|v| !v.is_null()) {
                    return Err("a borrowed ability on the stack has a new acquisition slot".into());
                }
                // Action uses serde's internally tagged `kind` representation.
                // Offered actions matter even when no such action was selected.
                if map
                    .get("kind")
                    .and_then(|v| v.as_str())
                    .is_some_and(|kind| {
                        (!compare_mana
                            && matches!(
                                kind,
                                "TapMana" | "TapGrantedMana" | "AnnounceMana" | "ActivateMana"
                            ))
                            || matches!(
                                kind,
                                "AnnounceGraveyardBorrowedAbility"
                                    | "AnnounceGraveyardBorrowedMana"
                                    | "ActivateGraveyardBorrowedMana"
                                    | "AnnounceBorrowedAbility"
                                    | "AnnounceBorrowedMana"
                                    | "ActivateBorrowedMana"
                                    | "ChooseEntryBehold"
                            )
                    })
                {
                    return Err(
                        "an offered behold or mana-ability action has no proven prior identity"
                            .into(),
                    );
                }
                if map.get("decision").and_then(|v| v.as_str()) == Some("entry_behold")
                    || map.contains_key("ChooseEntryBehold")
                    || map.contains_key("Behold")
                    || map.get("kind").and_then(|v| v.as_str()) == Some("Behold")
                {
                    return Err("behold decision or cost has no prior-schema representation".into());
                }
                for key in ["added_basic_lands", "effective_basic_lands"] {
                    if let Some(value) = map.get(key) {
                        let types = value
                            .as_array()
                            .ok_or_else(|| format!("invalid {key} array"))?;
                        if types.iter().any(|kind| {
                            !matches!(
                                kind.as_str(),
                                Some("Plains" | "Island" | "Swamp" | "Mountain" | "Forest")
                            )
                        }) {
                            return Err(format!("invalid basic land type in {key}"));
                        }
                    }
                }
                if map
                    .get("suppress_printed_land")
                    .is_some_and(|v| v.as_bool() != Some(true))
                {
                    return Err("invalid printed land suppression flag".into());
                }
                let added = map.get("added_basic_lands").and_then(|v| v.as_array());
                if added.is_some_and(|a| !a.is_empty()) {
                    return Err(
                        "an added basic land subtype is not expressible in the prior schema".into(),
                    );
                }
                let effective = map.get("effective_basic_lands").and_then(|v| v.as_array());
                let chosen = map.get("basic_land").and_then(|v| v.as_str());
                let suppressed = map
                    .get("suppress_printed_land")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                if suppressed
                    && !(chosen.is_some()
                        && effective.is_some_and(|a| a.len() == 1 && a[0].as_str() == chosen))
                {
                    return Err(
                        "lost printed land subtypes cannot be expressed in the prior schema".into(),
                    );
                }
                if let (Some(kind), Some(types)) = (chosen, effective)
                    && !(types.len() == 1 && types[0].as_str() == Some(kind))
                {
                    return Err(
                        "effective basic land subtypes disagree with the prior chosen type".into(),
                    );
                }
                for value in map.values() {
                    check(value, compare_mana)?;
                }
            }
            _ => {}
        }
        Ok(())
    }
    let parsed: serde_json::Value = serde_json::from_str(input).map_err(|e| e.to_string())?;
    check(&parsed, compare_mana)?;
    let mut output = input.to_owned();
    // These fields are always emitted after existing fields, so removing their
    // leading comma preserves every other byte and hence the historical hash.
    for key in ["effective_basic_lands", "added_basic_lands"] {
        let prefix = format!(",\"{key}\":[");
        while let Some(start) = output.find(&prefix) {
            let value_start = start + prefix.len();
            let end = output[value_start..]
                .find(']')
                .ok_or("malformed subtype array")?
                + value_start;
            output.replace_range(start..=end, "");
        }
    }
    output = output.replace(",\"suppress_printed_land\":true", "");
    // Parsing again detects accidental damage without changing field order.
    serde_json::from_str::<serde_json::Value>(&output).map_err(|e| e.to_string())?;
    Ok(output)
}

/// Hidden retained acquisitions affect future identities even when current
/// public views and action sets happen to be empty.
fn prior_behold_owned_state_supported(game: &Game) -> bool {
    if game.state.roles.as_ref().is_some_and(|roles| {
        roles
            .source_linked_effects
            .iter()
            .any(|row| row.kind == SourceLinkedEffectKind::RemoveAbilitiesUntilEndOfTurn)
    }) {
        return false;
    }
    game.state
        .borrowed_links
        .as_ref()
        .is_none_or(|rows| rows.is_empty())
        && game
            .state
            .borrowed_slots
            .as_ref()
            .is_none_or(|rows| rows.is_empty())
        && game
            .state
            .graveyard_borrowing
            .as_ref()
            .is_none_or(|rows| rows.is_empty())
        && game
            .state
            .borrowed_uses
            .as_ref()
            .is_none_or(|rows| rows.is_empty())
}

fn prior_behold_journal_supported(journal: &[Action]) -> bool {
    prior_behold_candidate_journal_supported(journal, false)
}

fn prior_behold_candidate_journal_supported(journal: &[Action], compare_mana: bool) -> bool {
    !journal.iter().any(|action| {
        (!compare_mana
            && matches!(
                action,
                Action::TapMana { .. }
                    | Action::TapGrantedMana { .. }
                    | Action::AnnounceMana { .. }
                    | Action::ActivateMana { .. }
            ))
            || matches!(
                action,
                Action::ChooseEntryBehold { .. }
                    | Action::AnnounceGraveyardBorrowedAbility { .. }
                    | Action::AnnounceGraveyardBorrowedMana { .. }
                    | Action::ActivateGraveyardBorrowedMana { .. }
                    | Action::AnnounceBorrowedAbility { .. }
                    | Action::AnnounceBorrowedMana { .. }
                    | Action::ActivateBorrowedMana { .. }
            )
    })
}

#[test]
fn prior_behold_projection_preserves_order_and_rejects_unrepresentable_state() {
    let current = r#"[{"granted_subtypes":{"added":[],"creature_types":null,"effective_basic_lands":["Island"]},"borrowed_abilities":[],"borrowed_slot":null}]"#;
    let previous = r#"[{"granted_subtypes":{"added":[],"creature_types":null},"borrowed_abilities":[],"borrowed_slot":null}]"#;
    assert_eq!(project_prior_behold_json(current).unwrap(), previous);
    let escaped = r#"{"label":"x,\"effective_basic_lands\":[\"Island\"]"}"#;
    assert_eq!(project_prior_behold_json(escaped).unwrap(), escaped);
    let chosen = r#"{"granted_subtypes":{"added":[],"creature_types":null,"effective_basic_lands":["Forest"],"suppress_printed_land":true,"basic_land":"Forest"}}"#;
    let expected =
        r#"{"granted_subtypes":{"added":[],"creature_types":null,"basic_land":"Forest"}}"#;
    assert_eq!(project_prior_behold_json(chosen).unwrap(), expected);
    for unrepresentable in [
        r#"{"granted_subtypes":{"added":[],"added_basic_lands":["Forest"]}}"#,
        r#"{"granted_subtypes":{"added":[],"effective_basic_lands":[],"suppress_printed_land":true}}"#,
        r#"{"borrowed_abilities":[{"ability":65533,"slot":0}]}"#,
        r#"{"graveyard_borrowed_abilities":[{"ability":0,"slot":0}]}"#,
        r#"{"borrowed_slot":0}"#,
        r#"{"loyalty_ability_activated_this_turn":true}"#,
        r#"{"kind":"RemoveAbilitiesUntilEndOfTurn"}"#,
        r#"{"next_spell_uncounterable":true}"#,
        r#"{"uncounterable_spell_ids":[1]}"#,
        r#"{"decision":"entry_behold"}"#,
        r#"{"entry_replacement":{"kind":{"Behold":"Jace"}}}"#,
        r#"{"additional_cost":{"kind":"Behold"}}"#,
        r#"{"granted_subtypes":{"effective_basic_lands":["unknown"]}}"#,
    ] {
        assert!(
            project_prior_behold_json(unrepresentable).is_err(),
            "{unrepresentable}"
        );
    }
    assert!(prior_behold_journal_supported(&[Action::Pass]));
    assert!(!prior_behold_journal_supported(&[Action::TapMana {
        card: 0
    }]));
    assert!(!prior_behold_journal_supported(&[
        Action::ChooseEntryBehold { object: None }
    ]));
}

#[test]
fn prior_projection_rejects_offered_actions_even_when_journal_is_empty() {
    let donor = ObjectRef {
        card: 7,
        generation: 3,
    };
    let actions = [
        Action::TapMana { card: 0 },
        Action::TapGrantedMana {
            card: 0,
            grantor: donor,
            production: [0, 1, 0, 0, 0, 0],
        },
        Action::AnnounceMana {
            card: 0,
            ability: 0,
            counter_payment: Counters::default(),
            production: [0, 1, 0, 0, 0, 0],
        },
        Action::ActivateMana {
            card: 0,
            ability: 0,
            payment: [0; 6],
            counter_payment: Counters::default(),
            production: [0, 1, 0, 0, 0, 0],
        },
        Action::AnnounceBorrowedAbility {
            card: 0,
            exiled: donor,
            ability: 0,
            counter_payment: Counters::default(),
        },
        Action::AnnounceBorrowedMana {
            card: 0,
            exiled: donor,
            ability: u16::MAX - 2,
            counter_payment: Counters::default(),
            production: [0, 1, 0, 0, 0, 0],
        },
        Action::ActivateBorrowedMana {
            card: 0,
            exiled: donor,
            ability: u16::MAX - 2,
            payment: [0; 6],
            counter_payment: Counters::default(),
            production: [0, 1, 0, 0, 0, 0],
        },
        Action::AnnounceGraveyardBorrowedAbility {
            card: 0,
            graveyard: donor,
            ability: 0,
            counter_payment: Counters::default(),
        },
        Action::AnnounceGraveyardBorrowedMana {
            card: 0,
            graveyard: donor,
            ability: u16::MAX,
            counter_payment: Counters::default(),
            production: [0, 0, 0, 0, 1, 0],
        },
        Action::ActivateGraveyardBorrowedMana {
            card: 0,
            graveyard: donor,
            ability: u16::MAX,
            payment: [0; 6],
            counter_payment: Counters::default(),
            production: [0, 0, 0, 0, 1, 0],
        },
        Action::ChooseEntryBehold { object: None },
    ];
    assert!(prior_behold_journal_supported(&[]));
    for action in actions {
        let offered = serde_json::json!({ "actions": [Action::Pass, action] }).to_string();
        assert!(project_prior_behold_json(&offered).is_err(), "{offered}");
    }
    let safe = serde_json::json!({ "actions": [Action::Pass] }).to_string();
    assert_eq!(project_prior_behold_json(&safe).unwrap(), safe);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let land = put(&mut game, player, "Forest", Zone::Battlefield);
        let view = game.observe(player);
        assert!(view.actions.contains(&Action::TapMana { card: land }));
        let observation = serde_json::to_string(&[view]).unwrap();
        assert!(project_prior_behold_json(&observation).is_err());
    }
}

#[test]
fn prior_projection_rejects_hidden_retained_borrowing_state() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let source = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let donor = put(&mut game, player, "Forest", Zone::Exile);
        assert!(prior_behold_owned_state_supported(&game));
        let source_ref = game.object_ref(source);
        let donor_ref = game.object_ref(donor);
        game.state
            .borrowed_links
            .get_or_insert_default()
            .push(borrowed_activations::Link {
                source: source_ref,
                origin: None,
                exiled: donor_ref,
                first_slot: 0,
                width: 1,
            });
        assert!(game.observe(player).borrowed_abilities.is_empty());
        assert!(!prior_behold_owned_state_supported(&game));
    }
}

/// Classify only live, exact prepared copies for the separate final-byte audit.
/// Check all other retained state with the unmodified owned guard. Masking the
/// copies is used only in this temporary clone and does not assert anything
/// about future moves, actions, rollback, or replay under the previous schema.
fn prior_schema191_prepared_observation_candidate(state: &State) -> bool {
    let mut copy = state.clone();
    let mut found = false;
    for (id, object) in state.objects.iter().enumerate() {
        if !object.is_spell_copy()
            || (object.zone == Zone::Outside && state.free_tokens.contains(&(id as CardId)))
        {
            continue;
        }
        let reference = ObjectRef {
            card: id as CardId,
            generation: object.generation,
        };
        if object.copy_kind != CopyKind::Card
            || object.zone != Zone::Exile
            || !state
                .preparations
                .as_ref()
                .is_some_and(|rows| rows.iter().any(|row| row.spell_copy == reference))
        {
            return false;
        }
        copy.objects[id].copy_kind = CopyKind::None;
        found = true;
    }
    found && prior_schema191_state_supported(&copy)
}

fn trace(card: Card, schema_projection: Option<u32>) -> Trace {
    trace_candidate(card, schema_projection, false)
}

fn trace_candidate(card: Card, schema_projection: Option<u32>, compare_mana: bool) -> Trace {
    trace_candidate_with_prepared_observation_audit(card, schema_projection, compare_mana, false)
}

// This separate, opt-in audit compares the final public bytes only. It is not
// an owned-state or future-continuation projection and must never select the
// ordinary compatibility result.
fn trace_candidate_with_prepared_observation_audit(
    card: Card,
    schema_projection: Option<u32>,
    compare_mana: bool,
    observation_only: bool,
) -> Trace {
    let mut g = fixture();
    let id = put(&mut g, Player::P0, card.identity(), Zone::Hand);
    let opening = if card.is_land() {
        Action::PlayLand { card: id }
    } else {
        Action::AnnounceCast { card: id }
    };
    let mut root = "main";
    if !g.legal_actions().contains(&opening) {
        match card.identity() {
            "Choreographed Sparks" => {
                root = "own_stack";
                // Its modes copy only controlled spells, unlike counterspell roots.
                let creature = put(&mut g, Player::P0, "Bear Cub", Zone::Hand);
                cast(&mut g, creature, None);
                let instant = put(&mut g, Player::P0, "Shock", Zone::Hand);
                cast(&mut g, instant, Some(Target::Player(Player::P1)));
            }
            "Dreadmaw's Ire"
            | "Cosmium Blast"
            | "Helicarrier Strike"
            | "Elspeth's Smite"
            | "Joust Through"
            | "Kellan's Lightblades"
            | "Not on My Watch"
            | "Protective Response"
            | "Sudden Strike" => {
                root = "combat";
                g.state.active = Player::P1;
                g.state.priority = Player::P1;
                let attacker = put(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
                g.state.step = Step::Attackers;
                g.state.pending = Pending::Attackers;
                act(&mut g, Action::Attack { card: attacker });
                act(&mut g, Action::FinishAttackers);
                act(&mut g, Action::Pass);
            }
            _ => {
                root = "stack";
                // Qualified counterspells get independently named target roots.
                let target_name = match card.identity() {
                    "Annul" => "Gilded Lotus",
                    "Precise Redaction" => "Serra Angel",
                    "Spell Snare" => "Bear Cub",
                    _ => "Galewind Moose",
                };
                let target = put(&mut g, Player::P1, target_name, Zone::Hand);
                g.state.active = Player::P1;
                g.state.priority = Player::P1;
                cast(&mut g, target, None);
                if target_name == "Galewind Moose" {
                    let shock = put(&mut g, Player::P1, "Shock", Zone::Hand);
                    cast(&mut g, shock, Some(Target::Player(Player::P0)));
                }
                g.state.active = Player::P0;
                g.state.priority = Player::P0;
            }
        }
    }

    let mut journal = Vec::new();
    let offered = g.legal_actions().contains(&opening);
    assert!(
        offered,
        "{} needs a reviewed castable fixture",
        card.identity()
    );
    let status = if offered {
        act(&mut g, opening.clone());
        journal.push(opening);
        for _ in 0..63 {
            if g.outcome().is_some()
                || (g.state.pending == Pending::Priority
                    && g.state.stack.is_empty()
                    && g.state.resolving.is_none())
            {
                break;
            }
            assert!(
                g.state.objects.len() < 192,
                "{} exceeded object budget",
                card.identity()
            );
            let actions = g.legal_actions();
            let action = if g.state.pending == Pending::Priority {
                Action::Pass
            } else {
                actions
                    .iter()
                    .find(|a| matches!(a, Action::ChooseX { value: 2 }))
                    .or_else(|| {
                        actions.iter().find(|a| {
                            matches!(
                                a,
                                Action::FinishTargets
                                    | Action::FinishSearch
                                    | Action::FinishLibrarySelection
                            )
                        })
                    })
                    .or_else(|| actions.iter().find(|a| !matches!(a, Action::Concede)))
                    .unwrap_or_else(|| {
                        panic!(
                            "{} has no continuation at {:?}",
                            card.identity(),
                            g.state.pending
                        )
                    })
                    .clone()
            };
            act(&mut g, action.clone());
            journal.push(action);
        }
        assert!(
            g.outcome().is_some()
                || (g.state.pending == Pending::Priority
                    && g.state.stack.is_empty()
                    && g.state.resolving.is_none()),
            "{} exceeded64 decisions at {:?}",
            card.identity(),
            g.state.pending
        );
        "completed"
    } else {
        "not_offered"
    };
    assert!(
        g.state.objects.len() < 192,
        "{} exceeded object budget",
        card.identity()
    );
    let mut views = [g.observe(Player::P0), g.observe(Player::P1)];
    for view in &mut views {
        view.catalogue = "";
        if let Some(schema) = schema_projection {
            view.schema = schema;
            if schema < 169 {
                view.solved_cases.clear();
                project_battle_protectors(view);
            }
            if schema < 139 {
                for row in &mut view.counter_placement_history {
                    row.plus_one = false;
                }
            }
        }
    }
    let mut observation_bytes = serde_json::to_string(&views).unwrap();
    let mut projection_error = None;
    if schema_projection.is_some_and(|schema| schema < 194) {
        if !prior_schema194_state_supported(&g.state) {
            projection_error =
                Some("owned Jace or Ugin effects cannot be projected to schema 193".to_owned());
        } else {
            match project_prior_schema194(&observation_bytes) {
                Ok(projected) => observation_bytes = projected,
                Err(reason) => projection_error = Some(reason),
            }
        }
    }
    if projection_error.is_none() && schema_projection.is_some_and(|schema| schema < 193) {
        if !prior_schema193_state_supported(&g.state) {
            projection_error = Some(
                "owned next-turn keyword effects cannot be projected to schema 192".to_owned(),
            );
        } else {
            match project_prior_schema193(&observation_bytes) {
                Ok(projected) => observation_bytes = projected,
                Err(reason) => projection_error = Some(reason),
            }
        }
    }
    if projection_error.is_none() && schema_projection.is_some_and(|schema| schema < 192) {
        if !prior_schema192_state_supported(&g.state) {
            projection_error =
                Some("owned outside-entry rules cannot be projected to schema 191".to_owned());
        } else {
            match project_prior_schema192(&observation_bytes) {
                Ok(projected) => observation_bytes = projected,
                Err(reason) => projection_error = Some(reason),
            }
        }
    }
    if projection_error.is_none() && schema_projection.is_some_and(|schema| schema < 191) {
        if !prior_schema191_state_supported(&g.state)
            && !(observation_only
                && schema_projection == Some(190)
                && prior_schema191_prepared_observation_candidate(&g.state))
        {
            projection_error = Some(
                "owned Sphinx or zone-cost state cannot be projected to schema 190".to_owned(),
            );
        } else {
            match project_prior_schema191(&observation_bytes) {
                Ok(projected) => observation_bytes = projected,
                Err(reason) => projection_error = Some(reason),
            }
        }
    }
    if projection_error.is_none() && schema_projection.is_some_and(|schema| schema < 190) {
        if !prior_schema190_state_supported(&g.state)
            || !uldaros_schema_candidate::prior_type_slots_owned_safe(&g.state)
        {
            projection_error =
                Some("owned schema 190 rules cannot be projected to schema 189".to_owned());
        } else {
            match uldaros_schema_candidate::project_before_type_slots(&observation_bytes)
                .and_then(|json| project_prior_schema190(&json))
            {
                Ok(projected) => observation_bytes = projected,
                Err(reason) => projection_error = Some(reason),
            }
        }
    }
    if projection_error.is_none() && schema_projection.is_some_and(|schema| schema < 189) {
        if !prior_outside_state_supported(&g.state) {
            projection_error =
                Some("owned outside-game rules cannot be projected to schema 188".to_owned());
        } else {
            match project_prior_outside(&observation_bytes) {
                Ok(projected) => observation_bytes = projected,
                Err(reason) => projection_error = Some(reason),
            }
        }
    }
    if projection_error.is_none() && schema_projection.is_some_and(|schema| schema < 188) {
        if !prior_schema188_state_supported(&g.state) {
            projection_error = Some(
                "owned Null, Sanctum, or Cryomancer rules cannot be projected to schema 187"
                    .to_owned(),
            );
        } else {
            match project_prior_schema188(&observation_bytes) {
                Ok(projected) => observation_bytes = projected,
                Err(reason) => projection_error = Some(reason),
            }
        }
    }
    if projection_error.is_none() && schema_projection.is_some_and(|schema| schema < 187) {
        if !prior_schema187_state_supported(&g.state) {
            projection_error =
                Some("owned Jace loyalty timing cannot be projected to schema 186".to_owned());
        } else {
            match project_prior_schema187(&observation_bytes) {
                Ok(projected) => observation_bytes = projected,
                Err(reason) => projection_error = Some(reason),
            }
        }
    }
    if projection_error.is_none() && schema_projection.is_some_and(|schema| schema < 186) {
        if !prior_schema186_state_supported(&g.state) {
            projection_error =
                Some("owned Chandra or Garruk rules cannot be projected to schema 185".to_owned());
        } else {
            match project_prior_schema186(&observation_bytes) {
                Ok(projected) => observation_bytes = projected,
                Err(reason) => projection_error = Some(reason),
            }
        }
    }
    if projection_error.is_none() && schema_projection.is_some_and(|schema| schema < 185) {
        if !prior_schema185_state_supported(&g.state) {
            projection_error =
                Some("owned Mountain mana rules cannot be projected to schema 184".to_owned());
        } else {
            match project_prior_schema185(&observation_bytes) {
                Ok(projected) => observation_bytes = projected,
                Err(reason) => projection_error = Some(reason),
            }
        }
    }
    if projection_error.is_none() && schema_projection.is_some_and(|schema| schema < 184) {
        if !prior_schema184_state_supported(&g.state) {
            projection_error = Some(
                "owned Chandra or paid resolution cast cannot be projected to schema 183"
                    .to_owned(),
            );
        } else {
            match project_prior_schema184(&observation_bytes) {
                Ok(projected) => observation_bytes = projected,
                Err(reason) => projection_error = Some(reason),
            }
        }
    }
    if projection_error.is_none() && schema_projection.is_some_and(|schema| schema < 183) {
        if !prior_schema183_state_supported(&g.state) {
            projection_error = Some(
                "owned Garruk delayed attack rules cannot be projected to schema 182".to_owned(),
            );
        } else {
            match project_prior_schema183(&observation_bytes) {
                Ok(projected) => observation_bytes = projected,
                Err(reason) => projection_error = Some(reason),
            }
        }
    }
    if projection_error.is_none() && schema_projection.is_some_and(|schema| schema < 182) {
        if !prior_schema182_state_supported(&g.state) {
            projection_error = Some(
                "owned static grant or Tomik rules cannot be projected to schema 181".to_owned(),
            );
        } else {
            match project_prior_schema182(&observation_bytes) {
                Ok(projected) => observation_bytes = projected,
                Err(reason) => projection_error = Some(reason),
            }
        }
    }
    if projection_error.is_none() && schema_projection.is_some_and(|schema| schema < 181) {
        if !prior_schema181_state_supported(&g.state) {
            projection_error = Some(
                "owned trigger, ward or delayed exile cannot be projected to schema 180".to_owned(),
            );
        } else {
            match project_prior_schema181(&observation_bytes) {
                Ok(projected) => observation_bytes = projected,
                Err(reason) => projection_error = Some(reason),
            }
        }
    }
    if projection_error.is_none() && schema_projection.is_some_and(|schema| schema < 180) {
        if !prior_schema180_state_supported(&g.state) {
            projection_error = Some(
                "owned wheel or additive subtype continuation cannot be projected to schema 179"
                    .to_owned(),
            );
        } else {
            match project_prior_schema180(&observation_bytes) {
                Ok(projected) => observation_bytes = projected,
                Err(reason) => projection_error = Some(reason),
            }
        }
    }
    if projection_error.is_none() && schema_projection.is_some_and(|schema| schema < 179) {
        if !prior_schema179_copy_owned_state_supported(&g) {
            projection_error = Some(
                "owned conditional copy ability has no prior-schema representation".to_owned(),
            );
        } else {
            match project_prior_schema179_copy_abilities(&observation_bytes) {
                Ok(projected) => observation_bytes = projected,
                Err(reason) => projection_error = Some(reason),
            }
        }
    }
    if projection_error.is_none() && schema_projection.is_some_and(|schema| schema < 178) {
        if !prior_split_second_owned_state_supported(&g) {
            projection_error = Some(
                "split-second or combat restriction provider has no prior-schema representation"
                    .to_owned(),
            );
        } else if let Err(reason) = project_prior_split_second(&observation_bytes) {
            projection_error = Some(reason);
        } else if !prior_schema178_history_owned_state_supported(&g) {
            projection_error = Some(
                "owned combat-damage or legend-rule history has no prior-schema representation"
                    .to_owned(),
            );
        } else {
            match project_prior_schema178_history(&observation_bytes) {
                Ok(projected) => observation_bytes = projected,
                Err(reason) => projection_error = Some(reason),
            }
        }
    }
    if projection_error.is_none() && schema_projection.is_some_and(|schema| schema < 177) {
        if !prior_behold_owned_state_supported(&g) {
            projection_error = Some(
                "owned borrowing acquisition state cannot be reconstructed in the prior schema"
                    .to_owned(),
            );
        } else if !prior_behold_candidate_journal_supported(&journal, compare_mana) {
            projection_error =
                Some("journal contains behold or mana-ability actions whose old identities cannot be proved".to_owned());
        } else {
            match project_prior_behold_candidate(&observation_bytes, compare_mana) {
                Ok(projected) => observation_bytes = projected,
                Err(reason) => projection_error = Some(reason),
            }
        }
    }
    if schema_projection.is_some_and(|schema| schema < 177) {
        observation_bytes =
            observation_bytes.replace("\"loyalty_ability_activated_this_turn\":false,", "");
    }
    if schema_projection.is_some_and(|schema| schema < 177) {
        observation_bytes = observation_bytes.replace(",\"next_spell_uncounterable\":false", "");
    }
    if schema_projection.is_some_and(|schema| schema < 175) {
        for field in [
            "\"noncombat_damage_this_turn\":true,",
            "\"noncombat_damage_this_turn\":false,",
            "\"noncombat_damage_last_turn\":true,",
            "\"noncombat_damage_last_turn\":false,",
        ] {
            observation_bytes = observation_bytes.replace(field, "");
        }
    }
    if schema_projection.is_some_and(|schema| schema < 174) {
        for field in [
            "\"scried_or_surveilled_this_turn\":true,",
            "\"scried_or_surveilled_this_turn\":false,",
        ] {
            observation_bytes = observation_bytes.replace(field, "");
        }
    }
    if schema_projection.is_some_and(|schema| schema < 173) {
        for view in &views {
            for player in &view.players {
                observation_bytes = observation_bytes.replace(
                    &format!(
                        "\"cards_library_to_graveyard_this_turn\":{},",
                        player.cards_library_to_graveyard_this_turn
                    ),
                    "",
                );
            }
        }
        observation_bytes = observation_bytes.replace(",\"loyalty_activated_this_turn\":true", "");
    }
    if schema_projection.is_some_and(|schema| schema < 138) {
        // Recording-only projection of the two new ObjectView wire fields.
        // Preserve every inherited field and byte order so old hashes still
        // detect gameplay drift. Normal validation never takes this branch.
        for field in [
            ",\"entered_this_turn\":true",
            ",\"entered_this_turn\":false",
            ",\"power_up_uses\":[]",
        ] {
            observation_bytes = observation_bytes.replace(field, "");
        }
    }
    if schema_projection.is_some_and(|schema| schema < 154) {
        // Recording-only projection of the new first PlayerView field. Keep
        // inherited byte order and every existing fact; normal validation never
        // removes the public enduring-story designation.
        for field in ["\"enduring_story\":true,", "\"enduring_story\":false,"] {
            observation_bytes = observation_bytes.replace(field, "");
        }
    }
    if schema_projection.is_some_and(|schema| schema < 169) {
        observation_bytes = observation_bytes.replace("\"earthbend_returns\":[],", "");
        for value in ["true", "false"] {
            observation_bytes =
                observation_bytes.replace(&format!("\"cast_for_warp\":{value},"), "");
        }
        for count in 0..=4 {
            observation_bytes =
                observation_bytes.replace(&format!("\"turns_started\":{count},"), "");
        }
    }
    Trace {
        root: root.into(),
        status: projection_error.as_ref().map_or_else(
            || status.to_owned(),
            |reason| format!("prior_schema_unrepresentable: {reason}"),
        ),
        decisions: journal.len(),
        zone: format!("{:?}", g.object_view(id).zone),
        journal: digest(&serde_json::to_vec(&journal).unwrap()),
        observations: projection_error.map_or_else(
            || digest(observation_bytes.as_bytes()),
            |reason| format!("UNREPRESENTABLE: {reason}"),
        ),
    }
}

// Compare every field, retaining the historical digest and never treating an
// unsupported state as equal, even if a baseline happened to contain it.
fn classify_mana_candidate(candidate: &Trace, baseline: Option<&Trace>) -> &'static str {
    if candidate
        .status
        .starts_with("prior_schema_unrepresentable:")
    {
        "unrepresentable"
    } else if baseline == Some(candidate) {
        "matches_baseline"
    } else {
        "different_or_missing_baseline"
    }
}

#[test]
fn prior_mana_audit_detects_offered_action_identity_and_payload_changes() {
    fn row(actions: &str) -> Trace {
        let bytes = project_prior_behold_candidate(actions, true).unwrap();
        Trace {
            root: "main".into(),
            status: "completed".into(),
            decisions: 0,
            zone: "Battlefield".into(),
            journal: digest(b"[]"),
            observations: digest(bytes.as_bytes()),
        }
    }
    let baseline = row(r#"[{"actions":[{"kind":"TapMana","card":1}]}]"#);
    assert_eq!(
        classify_mana_candidate(&baseline, Some(&baseline)),
        "matches_baseline"
    );
    for changed in [
        r#"[{"actions":[{"kind":"TapMana","card":2}]}]"#,
        r#"[{"actions":[{"kind":"TapGrantedMana","card":1,"grantor":{"card":3,"generation":0},"production":[0,0,0,0,1,0]}]}]"#,
    ] {
        assert_eq!(
            classify_mana_candidate(&row(changed), Some(&baseline)),
            "different_or_missing_baseline"
        );
    }
    assert!(
        project_prior_behold_candidate(
            r#"[{"actions":[{"kind":"ChooseEntryBehold","object":null}]}]"#,
            true
        )
        .is_err()
    );
    assert!(project_prior_behold_candidate(r#"[{"borrowed_abilities":[{}]}]"#, true).is_err());
}

#[test]
fn supported_cards_retain_bounded_cast_and_entry_traces() {
    let coverage: serde_json::Value =
        serde_json::from_str(include_str!("../../data/standard/coverage.json")).unwrap();
    let actual: BTreeMap<String, Trace> = coverage["cards"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| row["implemented"] == true)
        .map(|row| {
            let name = row["name"].as_str().unwrap();
            let card = Card::from_name(name).unwrap();
            (name.into(), trace(card, None))
        })
        .collect();
    // Recording is explicit, outside ordinary validation, and still compares
    // against the committed baseline. Write into ~/.cache, never source inputs.
    if let Ok(output) = std::env::var("RUSTMAGE_RECORD_CARD_TRACES") {
        let path = std::path::PathBuf::from(output);
        let cache = std::path::PathBuf::from(std::env::var("HOME").unwrap()).join(".cache");
        assert!(path.starts_with(cache), "record traces under ~/.cache");
        std::fs::write(&path, serde_json::to_string_pretty(&actual).unwrap() + "\n").unwrap();
        // Audit a wire-only version migration without accepting any baseline
        // drift: the ordinary assertions below still run and fail on changes.
        if let Ok(schema) = std::env::var("RUSTMAGE_RECORD_CARD_TRACES_PREVIOUS_SCHEMA") {
            let schema: u32 = schema.parse().expect("numeric prior observation schema");
            let previous: BTreeMap<String, Trace> = coverage["cards"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|row| row["implemented"] == true)
                .map(|row| {
                    let name = row["name"].as_str().unwrap();
                    (
                        name.into(),
                        trace(Card::from_name(name).unwrap(), Some(schema)),
                    )
                })
                .collect();
            std::fs::write(
                path.with_extension("previous-schema.json"),
                serde_json::to_string_pretty(&previous).unwrap() + "\n",
            )
            .unwrap();
            if schema == 190
                && std::env::var_os("RUSTMAGE_AUDIT_PREPARED_OBSERVATIONS_ONLY").is_some()
            {
                let rows: BTreeMap<String, Trace> = coverage["cards"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter(|row| row["implemented"] == true)
                    .map(|row| {
                        let name = row["name"].as_str().unwrap();
                        (
                            name.to_owned(),
                            trace_candidate_with_prepared_observation_audit(
                                Card::from_name(name).unwrap(),
                                Some(190),
                                false,
                                true,
                            ),
                        )
                    })
                    .collect();
                let report = serde_json::json!({
                    "scope": "Final observation bytes only; not owned-state, action, rollback, or continuation compatibility.",
                    "schema": 190,
                    "traces": rows,
                });
                std::fs::write(
                    path.with_extension("prior-prepared-observations-only.json"),
                    serde_json::to_string_pretty(&report).unwrap() + "\n",
                )
                .unwrap();
            }
            // Opt-in evidence only: exact whole-trace equality against the
            // existing baseline; no historical re-recording or acceptance.
            // Observations are final only, and the historical digest is FNV64.
            if std::env::var_os("RUSTMAGE_AUDIT_PRIOR_MANA_ACTIONS").is_some() {
                let baseline: BTreeMap<String, Trace> =
                    serde_json::from_str(include_str!("card_traces.json")).unwrap();
                // This report contains only trace summaries and fixed projector
                // error messages; it never emits raw observations or hidden cards.
                let mut exceptions: BTreeMap<String, (Trace, Option<&Trace>)> = BTreeMap::new();
                let results: BTreeMap<String, &str> = coverage["cards"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter(|row| row["implemented"] == true)
                    .map(|row| {
                        let name = row["name"].as_str().unwrap();
                        let candidate =
                            trace_candidate(Card::from_name(name).unwrap(), Some(schema), true);
                        let previous = baseline.get(name);
                        let result = classify_mana_candidate(&candidate, previous);
                        if result != "matches_baseline" {
                            exceptions.insert(name.to_owned(), (candidate, previous));
                        }
                        (name.to_owned(), result)
                    })
                    .collect();
                std::fs::write(
                    path.with_extension("prior-mana-audit.json"),
                    serde_json::to_string_pretty(&results).unwrap() + "\n",
                )
                .unwrap();
                std::fs::write(
                    path.with_extension("prior-mana-exceptions.json"),
                    serde_json::to_string_pretty(&exceptions).unwrap() + "\n",
                )
                .unwrap();
            }
        }
    }
    let expected: BTreeMap<String, Trace> =
        serde_json::from_str(include_str!("card_traces.json")).unwrap();
    assert_eq!(
        actual.len(),
        expected.len(),
        "review new or removed supported card traces"
    );
    for (name, row) in actual {
        assert_eq!(
            Some(&row),
            expected.get(&name),
            "{name}: review the changed cast/entry trace and add an independent rules case for intentional changes"
        );
    }
}

// Append inside src/assurance/regressions.rs so this test can call its private projector.
#[test]
fn prior_schema_removes_inactive_loyalty_and_proxy_fields_from_real_observations() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        // Keep a real nonempty legacy cast-grant object while Proxy's new flag is false.
        game.grant_artifact_spell_reduction(player, 1);
        let views = [game.observe(Player::P0), game.observe(Player::P1)];
        let original = serde_json::to_string(&views).unwrap();
        assert!(original.contains("\"loyalty_ability_activated_this_turn\":false"));
        assert!(original.contains("\"next_spell_uncounterable\":false"));
        let projected = project_prior_behold_json(&original)
            .unwrap()
            .replace("\"loyalty_ability_activated_this_turn\":false,", "")
            .replace(",\"next_spell_uncounterable\":false", "");
        let mut expected: serde_json::Value = serde_json::from_str(&original).unwrap();
        for view in expected.as_array_mut().unwrap() {
            for state in view["players"].as_array_mut().unwrap() {
                state
                    .as_object_mut()
                    .unwrap()
                    .remove("loyalty_ability_activated_this_turn");
                if let Some(grants) = state
                    .get_mut("turn_cast_grants")
                    .and_then(|v| v.as_object_mut())
                {
                    grants.remove("next_spell_uncounterable");
                }
            }
        }
        let actual: serde_json::Value = serde_json::from_str(&projected).unwrap();
        assert_eq!(actual, expected);
        assert!(!projected.contains("loyalty_ability_activated_this_turn"));
        assert!(!projected.contains("next_spell_uncounterable"));
    }
}

#[test]
fn prior_schema179_inspects_nested_copy_event_and_captured_recipe() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let source = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        let mut packet = g.activation_item(player, source, 0);
        let zero = CopyRecipe::new(Card::from_name("Bear Cub").unwrap());
        let active = zero.with_exceptions(CopyExceptions {
            conditional_sacrifice_abilities: 1,
            ..CopyExceptions::NONE
        });
        let inner = packet.clone();
        packet.copy_event = Some(Box::new(stack_copies::CopyEvent {
            spell: inner,
            amount: 1,
        }));
        g.state.pending_triggers.push(packet);
        assert!(prior_schema179_copy_owned_state_supported(&g));
        let saved = g.snapshot();
        for replay in 0..2 {
            if replay == 1 {
                g.restore(&saved).unwrap();
            }
            g.state
                .pending_triggers
                .last_mut()
                .unwrap()
                .copy_event
                .as_mut()
                .unwrap()
                .spell
                .captured_artifact_copy = Some(Box::new(active));
            assert!(!prior_schema179_copy_owned_state_supported(&g));
        }
    }
}

#[test]
fn prior_schema179_inspects_both_hidden_retarget_packets() {
    for player in [Player::P0, Player::P1] {
        for unsafe_item in [false, true] {
            let mut g = scenario(player);
            let source = put(&mut g, player, "Bear Cub", Zone::Battlefield);
            let zero = g.activation_item(player, source, 0);
            let mut active = zero.clone();
            active.source_copy_recipe = Some(Box::new(
                CopyRecipe::new(active.source.expect("object-backed packet").card).with_exceptions(
                    CopyExceptions {
                        conditional_sacrifice_abilities: 1,
                        ..CopyExceptions::NONE
                    },
                ),
            ));
            g.state.stack_copy_data = Some(Box::new(stack_copies::CopyState::test_retarget(
                zero.clone(),
                zero.clone(),
            )));
            assert!(prior_schema179_copy_owned_state_supported(&g));
            let saved = g.snapshot();
            let (item, template) = if unsafe_item {
                (active, zero)
            } else {
                (zero, active)
            };
            g.state.stack_copy_data = Some(Box::new(stack_copies::CopyState::test_retarget(
                item, template,
            )));
            assert!(!prior_schema179_copy_owned_state_supported(&g));
            g.restore(&saved).unwrap();
            assert!(prior_schema179_copy_owned_state_supported(&g));
        }
    }
}

#[test]
fn prior_schema179_inspects_hidden_delayed_return_and_earthbend_packets() {
    for player in [Player::P0, Player::P1] {
        for earthbend in [false, true] {
            let mut g = scenario(player);
            let source = put(&mut g, player, "Bear Cub", Zone::Battlefield);
            let reference = g.object_ref(source);
            let mut packet = g.activation_item(player, source, 0);
            packet.source_copy_recipe = Some(Box::new(
                CopyRecipe::new(packet.source.expect("object-backed packet").card).with_exceptions(
                    CopyExceptions {
                        conditional_sacrifice_abilities: 1,
                        ..CopyExceptions::NONE
                    },
                ),
            ));
            if earthbend {
                packet.targets.push(BoundTarget {
                    group: 0,
                    target: Target::Card(source),
                    object: Some(reference),
                });
                g.register_earthbend_return(&packet);
                assert!(g.state.earthbend_returns.is_some());
            } else {
                let link = return_links::ReturnLinkView {
                    exile_after: false,
                    additional_returning: None,
                    object: reference,
                    zone: Zone::Graveyard,
                    controller: player,
                    counter: None,
                    tapped: false,
                };
                let view = return_links::DelayedReturnView {
                    id: 1,
                    source: reference,
                    source_card: packet.source.expect("object-backed packet").card,
                    controller: player,
                    returning: link,
                };
                g.state.delayed_returns = Some(Box::new(vec![return_links::DelayedReturn {
                    view,
                    item: packet,
                }]));
            }
            assert!(g.observe(player).stack.is_empty());
            assert!(g.observe(player.other()).stack.is_empty());
            let saved = g.snapshot();
            assert!(!prior_schema179_copy_owned_state_supported(&g));
            if earthbend {
                g.state.earthbend_returns = None;
            } else {
                g.state.delayed_returns = None;
            }
            assert!(prior_schema179_copy_owned_state_supported(&g));
            g.restore(&saved).unwrap();
            assert!(!prior_schema179_copy_owned_state_supported(&g));
        }
    }
}

#[test]
fn schema182_preserves_existing_activated_grant_wire_bytes() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let giver = put(&mut g, player, "Way of the Healer", Zone::Battlefield);
        let walker = put(
            &mut g,
            player,
            "Ajani, Caller of the Pride",
            Zone::Battlefield,
        );
        g.state.objects[walker as usize].counters.loyalty = 5;
        for viewer in [Player::P0, Player::P1] {
            let view = g.observe(viewer);
            let walker_view = view
                .objects
                .iter()
                .find(|object| object.id == walker)
                .unwrap();
            let grant = walker_view
                .abilities
                .as_ref()
                .unwrap()
                .grants
                .iter()
                .find(|grant| grant.grantor == g.object_ref(giver))
                .unwrap();
            assert_eq!(grant.ability, Some(0));
            assert_eq!(grant.static_rule, None);
            let encoded = serde_json::to_string(grant).unwrap();
            #[derive(serde::Serialize)]
            struct PriorGrant {
                grantor: ObjectRef,
                granting_card: Card,
                ability: u16,
                grantor_controller: Player,
                timestamp: u64,
            }
            let prior = PriorGrant {
                grantor: grant.grantor,
                granting_card: grant.granting_card,
                ability: 0,
                grantor_controller: grant.grantor_controller,
                timestamp: grant.timestamp,
            };
            assert_eq!(encoded, serde_json::to_string(&prior).unwrap());
            let observation = serde_json::to_string(&view).unwrap();
            assert_eq!(project_prior_schema182(&observation).unwrap(), observation);
        }
    }
}

#[test]
fn schema182_rejects_hidden_face_and_nested_packet_with_rollback() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let id = put(&mut g, player, "Bear Cub", Zone::Hand);
        let object = g.object_ref(id);
        let bear = Card::from_name("Bear Cub").unwrap();
        let tomik = Card::from_name("Tomik, Orzhov Lawmage").unwrap();
        let ordinary = CopyRecipe::new(bear);
        let hidden = ordinary.with_faces(copies::CopyFaces::Double {
            physical_front: bear,
            front: bear,
            back: tomik,
            back_up: false,
        });
        g.set_copy_recipe(object, hidden);
        assert!(!prior_schema182_state_supported(&g.state));
        g.begin_attack_rollback(player);
        g.set_copy_recipe(object, ordinary);
        assert!(!prior_schema182_state_supported(&g.state));
        g.state.cast_rollback = None;
        assert!(prior_schema182_state_supported(&g.state));
        let source = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        let mut inner = g.activation_item(player, source, 0);
        inner.source_copy_recipe = Some(Box::new(hidden));
        let mut outer = g.activation_item(player, source, 0);
        outer.copy_event = Some(Box::new(stack_copies::CopyEvent {
            spell: inner,
            amount: 1,
        }));
        g.state.pending_triggers.push(outer);
        assert!(!prior_schema182_state_supported(&g.state));
        let saved = g.snapshot();
        for _ in 0..2 {
            g.state.pending_triggers.clear();
            assert!(prior_schema182_state_supported(&g.state));
            g.restore(&saved).unwrap();
            assert!(!prior_schema182_state_supported(&g.state));
        }
    }
}

#[test]
fn schema182_rejects_real_static_grant_nested_in_packet_json() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let source = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        g.state.objects[source as usize].card = Card::from_name("Tomik, Orzhov Lawmage").unwrap();
        g.register_battlefield_sources(source);
        let walker = put(
            &mut g,
            player,
            "Ajani, Caller of the Pride",
            Zone::Battlefield,
        );
        g.state.objects[walker as usize].counters.loyalty = 5;
        for viewer in [Player::P0, Player::P1] {
            let view = g.observe(viewer);
            let walker_view = view
                .objects
                .iter()
                .find(|object| object.id == walker)
                .unwrap();
            let nested = serde_json::json!({"copy_event": {"spell": {"source_view": walker_view}}});
            assert!(project_prior_schema182(&nested.to_string()).is_err());
        }
    }
}

#[test]
fn schema181_rejects_ajani_and_older_live_emblem_semantics_in_hidden_state() {
    for player in [Player::P0, Player::P1] {
        for changed in [
            Card::from_name("Ajani Resolute").unwrap(),
            Card::test_index(5462),
            Card::test_index(5463),
            Card::from_name("Vivien Reid").unwrap(),
        ] {
            let mut g = scenario(player);
            let id = put(&mut g, player, "Bear Cub", Zone::Hand);
            let object = g.object_ref(id);
            let bear = Card::from_name("Bear Cub").unwrap();
            assert!(prior_schema182_state_supported(&g.state));
            let hidden = CopyRecipe::new(bear).with_faces(copies::CopyFaces::Double {
                physical_front: bear,
                front: bear,
                back: changed,
                back_up: false,
            });
            g.set_copy_recipe(object, hidden);
            assert!(!prior_schema182_state_supported(&g.state));
            g.begin_attack_rollback(player);
            g.set_copy_recipe(object, CopyRecipe::new(bear));
            assert!(!prior_schema182_state_supported(&g.state));
            let saved = g.snapshot();
            g.state.cast_rollback = None;
            assert!(prior_schema182_state_supported(&g.state));
            g.restore(&saved).unwrap();
            assert!(!prior_schema182_state_supported(&g.state));
            for viewer in [player, player.other()] {
                let observation = serde_json::to_string(&g.observe(viewer)).unwrap();
                assert!(project_prior_schema182(&observation).is_ok());
                // The hidden recipe is absent from wire views; owned-state audit
                // must reject it even when projection of the view alone succeeds.
                assert!(!prior_schema182_state_supported(&g.state));
            }
        }
        for emblem in [Card::test_index(5463), Card::test_index(5338)] {
            let mut g = scenario(player);
            g.create_emblem(player, emblem);
            assert!(!prior_schema182_state_supported(&g.state));
            for viewer in [player, player.other()] {
                let observation = serde_json::to_string(&g.observe(viewer)).unwrap();
                assert!(project_prior_schema182(&observation).is_err());
            }
        }
    }
}

#[path = "schema183_garruk_independent.rs"]
mod schema183_garruk_independent;

#[test]
fn schema186_rejects_full_width_loyalty_x_in_nested_packets_and_rollback() {
    for player in [Player::P0, Player::P1] {
        for amount in [0, u64::from(u32::MAX) + 17] {
            let mut g = scenario(player);
            let source = put(&mut g, player, "Bear Cub", Zone::Battlefield);
            let mut inner = g.activation_item(player, source, 0);
            inner.loyalty_x = Some(amount);
            let mut outer = g.activation_item(player, source, 0);
            outer.copy_event = Some(Box::new(stack_copies::CopyEvent {
                spell: inner,
                amount: 1,
            }));
            let json = serde_json::to_string(&outer).unwrap();
            assert!(project_prior_schema186(&json).is_err());
            g.state.pending_triggers.push(outer);
            assert!(!prior_schema186_state_supported(&g.state));
            g.begin_attack_rollback(player);
            g.state.pending_triggers.clear();
            assert!(!prior_schema186_state_supported(&g.state));
            let saved = g.snapshot();
            g.state.cast_rollback = None;
            assert!(prior_schema186_state_supported(&g.state));
            g.restore(&saved).unwrap();
            assert!(!prior_schema186_state_supported(&g.state));
        }
    }
}

#[test]
fn schema186_rejects_chandra_restriction_in_hidden_mana_and_rollback() {
    for value in [
        "NoncreatureSpell",
        "Chandra, Chill of Compliance",
        "@emblem/fra-chandra-chill",
    ] {
        assert!(
            project_prior_schema186(&serde_json::json!({"nested": [value]}).to_string()).is_err()
        );
    }
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let quality = mana_lots::Quality {
            restriction: ManaRestrictionView::NoncreatureSpell,
            spell_bonus: ManaSpellBonusView::None,
            retention: crate::cards::ManaRetention::Step,
            copy_source: None,
            treasure_source: false,
        };
        assert!(prior_schema186_state_supported(&g.state));
        g.add_quality_mana(player, [0, 1, 0, 0, 0, 0], Some(quality));
        assert!(!prior_schema186_state_supported(&g.state));
        for viewer in [player, player.other()] {
            let view = serde_json::to_string(&g.observe(viewer)).unwrap();
            assert!(project_prior_schema186(&view).is_err());
        }
        g.begin_attack_rollback(player);
        g.state.mana_lots = None;
        assert!(!prior_schema186_state_supported(&g.state));
        let snapshot = g.snapshot();
        g.state.cast_rollback = None;
        assert!(prior_schema186_state_supported(&g.state));
        g.restore(&snapshot).unwrap();
        assert!(!prior_schema186_state_supported(&g.state));
        g.state.cast_rollback = None;
        let mana = g.state.mana_lots.get_or_insert_default();
        mana.effect_quality = Some(quality);
        assert!(!prior_schema186_state_supported(&g.state));
        g.state.mana_lots.as_mut().unwrap().effect_quality = None;
        let source = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        let source_ref = g.object_ref(source);
        g.state.mana_lots.as_mut().unwrap().announcement_quality = Some((source_ref, quality));
        assert!(!prior_schema186_state_supported(&g.state));
    }
}

#[test]
fn schema186_rejects_garruk_hidden_faces_delayed_stats_packets_and_rollback() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let source = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        let hidden = put(&mut g, player, "Bear Cub", Zone::Hand);
        let object = g.object_ref(hidden);
        let bear = Card::from_name("Bear Cub").unwrap();
        let garruk = Card::from_name("Garruk, Veiled Butcher").unwrap();
        let ordinary = CopyRecipe::new(bear);
        g.set_copy_recipe(
            object,
            ordinary.with_faces(copies::CopyFaces::Double {
                physical_front: bear,
                front: bear,
                back: garruk,
                back_up: false,
            }),
        );
        assert!(!prior_schema186_state_supported(&g.state));
        g.set_copy_recipe(object, ordinary);
        assert!(prior_schema186_state_supported(&g.state));
        let mut inner = g.activation_item(player, source, 0);
        inner.source.as_mut().expect("object-backed packet").card = garruk;
        let mut outer = g.activation_item(player, source, 0);
        outer.copy_event = Some(Box::new(stack_copies::CopyEvent {
            spell: inner,
            amount: 1,
        }));
        g.state.pending_triggers.push(outer);
        assert!(!prior_schema186_state_supported(&g.state));
        g.state.pending_triggers.clear();
        let recipient = g.object_ref(source);
        g.state
            .roles
            .get_or_insert_default()
            .until_next_turn_stats
            .push(UntilNextTurnStatsView {
                recipient,
                controller: player,
                power: -4,
                toughness: -1,
            });
        assert!(!prior_schema186_state_supported(&g.state));
        for viewer in [player, player.other()] {
            let view = serde_json::to_string(&g.observe(viewer)).unwrap();
            assert!(project_prior_schema186(&view).is_err());
        }
        g.begin_attack_rollback(player);
        g.state
            .roles
            .as_mut()
            .unwrap()
            .until_next_turn_stats
            .clear();
        assert!(!prior_schema186_state_supported(&g.state));
        let saved = g.snapshot();
        g.state.cast_rollback = None;
        assert!(prior_schema186_state_supported(&g.state));
        g.restore(&saved).unwrap();
        assert!(!prior_schema186_state_supported(&g.state));
    }
}

#[test]
fn schema186_preserves_published_schema185_pyremana_bytes() {
    let old = r#"[{"card":"Pyre Rhymer // Molten Tide","mountain_mana_bonus":1}]"#;
    assert_eq!(project_prior_schema186(old).unwrap(), old);
    assert!(project_prior_schema185(old).is_err());
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        g.state.players[player.index()].mountain_mana_bonus = 1;
        assert!(prior_schema186_state_supported(&g.state));
        assert!(!prior_schema185_state_supported(&g.state));
    }
}

#[path = "schema186_cross_feature.rs"]
mod schema186_cross_feature;

#[test]
fn schema187_preserves_old_bytes_and_rejects_jace_timing_and_nested_rollback() {
    let old = r#"[{"mountain_mana_bonus":1,"other":true}]"#;
    assert_eq!(project_prior_schema187(old).unwrap(), old);
    assert!(project_prior_schema187(r#"[{"jace_loyalty_anytime":true}]"#).is_err());
    assert!(project_prior_schema187(r#"[{"card":"Jace's Machinations"}]"#).is_err());
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        assert!(prior_schema187_state_supported(&g.state));
        g.state.players[player.index()].jace_loyalty_anytime = true;
        assert!(!prior_schema187_state_supported(&g.state));
        for viewer in [player, player.other()] {
            assert!(
                project_prior_schema187(&serde_json::to_string(&g.observe(viewer)).unwrap())
                    .is_err()
            );
        }
        g.begin_attack_rollback(player);
        g.state.players[player.index()].jace_loyalty_anytime = false;
        assert!(!prior_schema187_state_supported(&g.state));
        let saved = g.snapshot();
        g.state.cast_rollback = None;
        assert!(prior_schema187_state_supported(&g.state));
        g.restore(&saved).unwrap();
        assert!(!prior_schema187_state_supported(&g.state));
        g.state.cast_rollback = None;
        let source = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        let mut inner = g.activation_item(player, source, 0);
        inner.source.as_mut().expect("object-backed packet").card =
            Card::from_name("Jace's Machinations").unwrap();
        let mut outer = g.activation_item(player, source, 0);
        outer.copy_event = Some(Box::new(stack_copies::CopyEvent {
            spell: inner,
            amount: 1,
        }));
        g.state.pending_triggers.push(outer);
        assert!(!prior_schema187_state_supported(&g.state));
    }
}

#[test]
fn cryomancer_projection_rejects_quantity_nested_packets_and_rollback() {
    let old = r#"[{"jace_loyalty_anytime":true}]"#;
    assert_eq!(project_prior_schema188(old).unwrap(), old);
    assert!(project_prior_schema188(r#"[{"reflexive_target_limit":0}]"#).is_err());
    assert!(project_prior_schema188(r#"[{"card":"Seasoned Cryomancer"}]"#).is_err());
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        assert!(prior_schema188_state_supported(&g.state));
        let source = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        let mut inner = g.activation_item(player, source, 0);
        inner.reflexive_target_limit = Some(0);
        let mut outer = g.activation_item(player, source, 0);
        outer.copy_event = Some(Box::new(stack_copies::CopyEvent {
            spell: inner,
            amount: 1,
        }));
        g.state.pending_triggers.push(outer);
        assert!(!prior_schema188_state_supported(&g.state));
        g.begin_attack_rollback(player);
        g.state.pending_triggers.clear();
        assert!(!prior_schema188_state_supported(&g.state));
        let snapshot = g.snapshot();
        g.state.cast_rollback = None;
        assert!(prior_schema188_state_supported(&g.state));
        g.restore(&snapshot).unwrap();
        assert!(!prior_schema188_state_supported(&g.state));
    }
}

#[test]
fn schema188_preserves_schema187_jace_and_rejects_new_cards_and_hidden_null_links() {
    let old = r#"[{"card":"Jace's Machinations","jace_loyalty_anytime":true}]"#;
    assert_eq!(project_prior_schema188(old).unwrap(), old);
    for name in ["Null Summoner", "Sanctum Lurker", "Seasoned Cryomancer"] {
        assert!(project_prior_schema188(&serde_json::json!({"card":name}).to_string()).is_err());
    }
    assert!(project_prior_schema188(r#"[{"nested":{"reflexive_target_limit":0}}]"#).is_err());
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let source = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        let hidden = put(&mut g, player, "Opt", Zone::Hand);
        let source_ref = g.object_ref(source);
        let hidden_ref = g.object_ref(hidden);
        g.state.players[player.index()].jace_loyalty_anytime = true;
        assert!(prior_schema188_state_supported(&g.state));
        g.state
            .linked_choices
            .get_or_insert_default()
            .null_cast_links
            .push(linked_choices::NullCastLink {
                source: source_ref,
                exiled: hidden_ref,
                ability_origin: Some(41),
            });
        for viewer in [player, player.other()] {
            let view = serde_json::to_value(g.observe(viewer)).unwrap();
            assert_eq!(view["linked_exile"], serde_json::json!([]));
            assert!(!prior_schema188_state_supported(&g.state));
        }
        g.begin_attack_rollback(player);
        g.state
            .linked_choices
            .as_mut()
            .unwrap()
            .null_cast_links
            .clear();
        assert!(!prior_schema188_state_supported(&g.state));
        let snapshot = g.snapshot();
        g.state.cast_rollback = None;
        assert!(prior_schema188_state_supported(&g.state));
        g.restore(&snapshot).unwrap();
        assert!(!prior_schema188_state_supported(&g.state));
    }
}

#[test]
fn outside_projection_rejects_hidden_pools_nested_packets_and_rollback() {
    assert_eq!(
        project_prior_outside(r#"[{"jace_loyalty_anytime":true}]"#).unwrap(),
        r#"[{"jace_loyalty_anytime":true}]"#
    );
    assert!(project_prior_outside(r#"[{"known_outside":[[],[]]}]"#).is_err());
    assert!(project_prior_outside(r#"[{"card":"Extrapolate the Impossible"}]"#).is_err());
    assert!(project_prior_outside(r#"[{"kind":"ChooseOutsideCard","index":0}]"#).is_err());
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        assert!(prior_outside_state_supported(&g.state));
        g.state.outside_game = Some(Box::new(OutsideGameState {
            mode: OutsideGameMode::LimitedSideboard,
            pools: [Vec::new(), Vec::new()],
            choice: None,
            pending_entry: None,
            pool_versions: [0; 2],
            next_choice_id: 0,
            known: [Vec::new(), Vec::new()],
        }));
        assert!(!prior_outside_state_supported(&g.state));
        g.begin_attack_rollback(player);
        g.state.outside_game = None;
        assert!(!prior_outside_state_supported(&g.state));
        let snapshot = g.snapshot();
        g.state.cast_rollback = None;
        assert!(prior_outside_state_supported(&g.state));
        g.restore(&snapshot).unwrap();
        assert!(!prior_outside_state_supported(&g.state));
        g.state.cast_rollback = None;
        let source = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        let mut inner = g.activation_item(player, source, 0);
        inner.source.as_mut().expect("object-backed packet").card =
            Card::from_name("Extrapolate the Impossible").unwrap();
        let mut outer = g.activation_item(player, source, 0);
        outer.copy_event = Some(Box::new(stack_copies::CopyEvent {
            spell: inner,
            amount: 1,
        }));
        g.state.pending_triggers.push(outer);
        assert!(!prior_outside_state_supported(&g.state));
    }
}

#[test]
fn outside_projection_checks_actual_private_and_public_views_for_both_players() {
    for owner in [Player::P0, Player::P1] {
        let mut g = scenario(owner);
        let baseline = serde_json::to_string(&g.observe(owner)).unwrap();
        assert_eq!(project_prior_outside(&baseline).unwrap(), baseline);
        let mut pools = [vec![], vec![]];
        pools[owner.index()] = vec![Card::from_name("Shock").unwrap(), Card::Forest];
        g.state.outside_game = Some(Box::new(OutsideGameState {
            mode: OutsideGameMode::ConstructedSideboard,
            pools,
            choice: None,
            pending_entry: None,
            pool_versions: [0; 2],
            next_choice_id: 0,
            known: [Vec::new(), Vec::new()],
        }));
        assert!(!prior_outside_state_supported(&g.state));
        for viewer in [Player::P0, Player::P1] {
            let wire = serde_json::to_string(&g.observe(viewer)).unwrap();
            assert!(project_prior_outside(&wire).is_err());
        }
        let before = g.snapshot();
        g.begin_attack_rollback(owner);
        g.state.outside_game = None;
        assert!(!prior_outside_state_supported(&g.state));
        g.restore(&before).unwrap();
        assert!(!prior_outside_state_supported(&g.state));
    }
}

#[path = "schema189_independent_tests.rs"]
mod schema189_independent_tests;

#[test]
fn schema189_budget_projection_rejects_public_hidden_and_rollback_state() {
    let old = r#"[{"resolution_free_casts":{"paid":null}}]"#;
    assert_eq!(project_prior_outside(old).unwrap(), old);
    for amount in [0, 6] {
        assert!(
            project_prior_outside(
                &serde_json::json!({"nested":{"remaining_mana_value":amount}}).to_string()
            )
            .is_err()
        );
    }
    for player in [Player::P0, Player::P1] {
        for origin in [Zone::Exile, Zone::Library] {
            let mut g = scenario(player);
            g.state.resolution_free_casts = Some(Box::new(ResolutionFreeCasts {
                siege_transformed: false,
                paid: None,
                casts_remaining: None,
                pending_cast: None,
                remaining_mana_value: Some(0),
                alternative_cost: None,
                maximum_mana_value: None,
                origin,
                exile_on_graveyard: false,
                allow_lands: false,
                player,
                remaining: Vec::new(),
            }));
            assert!(!prior_outside_state_supported(&g.state));
            for viewer in [player, player.other()] {
                let view = serde_json::to_string(&g.observe(viewer)).unwrap();
                if origin == Zone::Exile || viewer == player {
                    assert!(project_prior_outside(&view).is_err());
                }
            }
            g.begin_attack_rollback(player);
            g.state.resolution_free_casts = None;
            assert!(!prior_outside_state_supported(&g.state));
            let saved = g.snapshot();
            g.state.cast_rollback = None;
            assert!(prior_outside_state_supported(&g.state));
            g.restore(&saved).unwrap();
            assert!(!prior_outside_state_supported(&g.state));
        }
    }
}

#[test]
fn schema190_restores_ordered_stack_fields_recursively() {
    let item = r#"{"source":{"id":7,"object":{"id":7,"generation":2},"card":"Bear Cub","facts":{"power":2,"card":"Bear Cub"}},"copy_event":null,"id":12,"kind":"Spell","source_transform":4,"source_keywords":0,"related_stack":null,"targets":[],"mode":null,"controller":"P0"}"#;
    let old = r#"{"copy_event":null,"id":12,"kind":"Spell","source_transform":4,"source_card":"Bear Cub","source_keywords":0,"source_facts":{"power":2,"card":"Bear Cub"},"related_stack":null,"card":7,"object":{"id":7,"generation":2},"targets":[],"mode":null,"controller":"P0"}"#;
    assert_eq!(project_prior_schema190(item).unwrap(), old);
    let nested = format!(r#"{{"copy_event":{{"spell":{item}}}}}"#);
    assert_eq!(
        project_prior_schema190(&nested).unwrap(),
        format!(r#"{{"copy_event":{{"spell":{old}}}}}"#)
    );
    let no_source = item.replace(r#"{"id":7,"object":{"id":7,"generation":2},"card":"Bear Cub","facts":{"power":2,"card":"Bear Cub"}}"#, "null");
    assert!(project_prior_schema190(&no_source).is_err());
    assert!(
        project_prior_schema190(r#"{"other_counters":[{"kind":"Radiation","count":1}]}"#).is_err()
    );
    assert!(project_prior_schema190(r#"{"proliferate_choice":{}}"#).is_err());
}

#[test]
fn schema190_preserves_older_revisions_and_audits_owned_state() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let source = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        let packet = g.activation_item(player, source, 0);
        g.state.stack.push(packet.clone());
        assert!(prior_schema190_state_supported(&g.state));
        for viewer in [player, player.other()] {
            let wire = serde_json::to_string(&g.observe(viewer)).unwrap();
            let old = project_prior_schema190(&wire).unwrap();
            assert!(old.contains("\"source_card\":"));
            assert!(!old.contains("\"source\":{\"id\":"));
            // A standard packet is accepted by older revision projectors too.
            assert!(project_prior_outside(&old).is_ok());
            assert!(project_prior_schema188(&old).is_ok());
        }
        let mut nested = packet.clone();
        let mut hidden = packet.clone();
        hidden.source.as_mut().unwrap().card = Card::test_index(4330);
        nested.copy_event = Some(Box::new(stack_copies::CopyEvent {
            spell: hidden,
            amount: 1,
        }));
        g.state.pending_triggers.push(nested);
        assert!(!prior_schema190_state_supported(&g.state));
        let snapshot = g.snapshot();
        g.state.pending_triggers.clear();
        assert!(prior_schema190_state_supported(&g.state));
        g.restore(&snapshot).unwrap();
        assert!(!prior_schema190_state_supported(&g.state));
        g.state.pending_triggers.clear();
        g.state.players[player.index()].other_player_counters = Some(Box::new([0, 0, 0, 1]));
        assert!(!prior_schema190_state_supported(&g.state));
    }
    for card in [
        "Jace, the Mind Sculptor",
        "Null Summoner",
        "Uldaros, the Architect",
        "Extrapolate the Impossible",
    ] {
        // Schema 190 itself does not erase fields or identities from earlier revisions.
        let json = serde_json::to_string(&card).unwrap();
        assert_eq!(project_prior_schema190(&json).unwrap(), json);
    }
}

#[path = "schema190_packet_independent.rs"]
mod schema190_packet_independent;
#[path = "uldaros_schema_candidate.rs"]
mod uldaros_schema_candidate;

#[test]
fn schema191_retains_old_bytes_and_rejects_new_zone_cost_state() {
    let old = r#"[{"card":"Bear Cub","revision":3}]"#;
    assert_eq!(project_prior_schema191(old).unwrap(), old);
    for value in [
        serde_json::json!({"card": "Sphinx's Approach"}),
        serde_json::json!({"card": "Emrakul, the Exigent Doom"}),
        serde_json::json!({"SelectResolutionExile": {"object": 1}}),
        serde_json::json!({"copy_event": {"spell": {"kind": "Prevent"}}}),
    ] {
        assert!(project_prior_schema191(&value.to_string()).is_err());
    }
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        assert!(prior_schema191_state_supported(&game.state));
        let object = put(&mut game, player, "Shock", Zone::Hand);
        game.state.objects[object as usize].card = Card::test_index(4016);
        assert!(!prior_schema191_state_supported(&game.state));
    }
}

#[test]
fn schema191_preserves_published_radiation_packets_in_nested_state_and_rollback() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        game.state.players[player.index()]
            .add_player_counter(crate::cards::PlayerCounterKind::Rad, 1);
        game.begin_step(Step::Main1);
        game.prepare_triggers();
        let radiation = game.state.stack.last().unwrap().clone();
        assert_eq!(radiation.source_less_rule, Some(SourceLessRule::Radiation));
        assert!(radiation.source.is_none());
        assert!(schema191_packet_supported(&radiation));
        assert!(prior_schema191_state_supported(&game.state));
        for viewer in [player, player.other()] {
            let wire = serde_json::to_string(&game.observe(viewer)).unwrap();
            assert_eq!(project_prior_schema191(&wire).unwrap(), wire);
        }
        let source = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let mut outer = game.activation_item(player, source, 0);
        outer.copy_event = Some(Box::new(stack_copies::CopyEvent {
            spell: radiation,
            amount: 1,
        }));
        game.state.stack.clear();
        game.state.pending_triggers.push(outer);
        assert!(prior_schema191_state_supported(&game.state));
        for viewer in [player, player.other()] {
            let wire = serde_json::to_string(&game.observe(viewer)).unwrap();
            assert_eq!(project_prior_schema191(&wire).unwrap(), wire);
        }
        game.begin_attack_rollback(player);
        game.state.pending_triggers.clear();
        assert!(prior_schema191_state_supported(&game.state));
        let checkpoint = game.snapshot();
        for _ in 0..2 {
            assert!(prior_schema191_state_supported(&game.state));
            game.restore(&checkpoint).unwrap();
        }
    }
}

#[test]
fn schema191_accepts_ceased_copy_slots_and_passive_zone_registry() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        game.state.zone_rules = Some(Box::default());
        assert!(prior_schema191_state_supported(&game.state));
        let spell = put(&mut game, player, "Shock", Zone::Hand);
        let target = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
        cast(&mut game, spell, Some(Target::Card(target)));
        let original = game.state.stack.last().unwrap().clone();
        let copy = game.allocate_spell_copy(&original, player);
        let reference = copy.source.unwrap().object;
        game.state.stack.push(copy);
        game.move_object(reference.card, Zone::Exile);
        assert!(!prior_schema191_state_supported(&game.state));
        game.state_based_actions();
        assert!(game.state.free_tokens.contains(&reference.card));
        assert!(prior_schema191_state_supported(&game.state));
        for viewer in [player, player.other()] {
            let wire = serde_json::to_string(&game.observe(viewer)).unwrap();
            assert_eq!(project_prior_schema191(&wire).unwrap(), wire);
        }
    }
}

#[test]
fn schema191_distinguishes_passive_real_zone_reveals_from_new_cards() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        // The real provider registers passive zone state on entry.
        let provider = put(&mut game, player, "Rest in Peace", Zone::Battlefield);
        assert!(game.state.zone_rules.is_some());
        assert!(prior_schema191_state_supported(&game.state));
        game.move_object(provider, Zone::Graveyard);
        // Progenitus's own replacement creates an actual reveal and shuffle.
        let progenitus = put(&mut game, player, "Progenitus", Zone::Battlefield);
        game.move_object(progenitus, Zone::Graveyard);
        assert!(
            game.zone_reveals()
                .iter()
                .any(|r| r.card.identity() == "Progenitus")
        );
        assert!(prior_schema191_state_supported(&game.state));
        let before = game.snapshot();
        for viewer in [player, player.other()] {
            let wire = serde_json::to_string(&game.observe(viewer)).unwrap();
            assert_eq!(project_prior_schema191(&wire).unwrap(), wire);
        }
        // The new identity exists only in an owned reveal registry, never in
        // any object, recipe, stack packet or outside-game pool.
        game.record_zone_reveal(Card::test_index(4016), player, Zone::Hand);
        assert!(!prior_schema191_state_supported(&game.state));
        game.restore(&before).unwrap();
        assert!(prior_schema191_state_supported(&game.state));
    }
}

#[test]
fn schema191_checks_card_copy_free_slot_and_rejects_unowned_outside_copy() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let copy = game.allocate_card_copy(Card::from_name("Bear Cub").unwrap(), player);
        // A live card copy remains conservatively rejected because its
        // pre-SBA zone behavior differs in schema 191.
        assert!(!prior_schema191_state_supported(&game.state));
        game.cease_card_copy(copy);
        assert!(game.state.free_tokens.contains(&copy.card));
        assert!(prior_schema191_state_supported(&game.state));
        let saved = game.snapshot();
        // Malformed or in-flight Outside copy: absence from the free list
        // means it cannot be treated as an inert recycled slot.
        game.state.free_tokens.retain(|&id| id != copy.card);
        assert!(!prior_schema191_state_supported(&game.state));
        game.restore(&saved).unwrap();
        assert!(prior_schema191_state_supported(&game.state));
        for viewer in [player, player.other()] {
            let wire = serde_json::to_string(&game.observe(viewer)).unwrap();
            assert_eq!(project_prior_schema191(&wire).unwrap(), wire);
        }
    }
}

#[test]
fn schema191_prepared_observation_audit_does_not_claim_owned_equivalence() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let id = put(
            &mut game,
            player,
            "Hallway Heckler // Vicious Verse",
            Zone::Battlefield,
        );
        game.prepare_permanent(game.object_ref(id));
        let copy = game.state.preparations.as_ref().unwrap()[0].spell_copy;
        assert_eq!(
            game.state.objects[copy.card as usize].copy_kind,
            CopyKind::Card
        );
        assert_eq!(game.state.objects[copy.card as usize].zone, Zone::Exile);
        assert!(!prior_schema191_state_supported(&game.state));
        assert!(prior_schema191_prepared_observation_candidate(&game.state));
        for viewer in [player, player.other()] {
            let mut view = game.observe(viewer);
            view.schema = 190;
            view.catalogue = "";
            let bytes = serde_json::to_string(&view).unwrap();
            assert_eq!(project_prior_schema191(&bytes).unwrap(), bytes);
        }
        // A live unassociated copy must not qualify for the restricted audit.
        let unrelated = game.allocate_card_copy(Card::from_name("Shock").unwrap(), player);
        assert!(!prior_schema191_prepared_observation_candidate(&game.state));
        game.cease_card_copy(unrelated);
        assert!(prior_schema191_prepared_observation_candidate(&game.state));
        let saved = game.snapshot();
        game.move_object(id, Zone::Graveyard);
        game.restore(&saved).unwrap();
        assert!(!prior_schema191_state_supported(&game.state));
        assert!(prior_schema191_prepared_observation_candidate(&game.state));
    }
}

#[test]
fn prepared_copy_can_produce_a_real_stack_to_graveyard_event_before_sba() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let id = put(
            &mut game,
            player,
            "Hallway Heckler // Vicious Verse",
            Zone::Battlefield,
        );
        game.prepare_permanent(game.object_ref(id));
        let copy = game.state.preparations.as_ref().unwrap()[0].spell_copy;
        act(&mut game, Action::AnnounceExile { card: copy.card });
        for _ in 0..20 {
            if game.state.announcing.is_none() {
                break;
            }
            let actions = game.legal_actions();
            let action = actions.iter().find(|a| matches!(a, Action::ChooseTarget { target: Target::Player(p) } if *p == player.other()))
                .or_else(|| actions.iter().find(|a| matches!(a, Action::ChooseSpellCost { .. } | Action::FinishTargets | Action::PayCosts { .. })))
                .expect("prepared cast continuation").clone();
            act(&mut game, action);
        }
        assert_eq!(game.state.objects[copy.card as usize].zone, Zone::Stack);
        let before = game.snapshot();
        for _ in 0..2 {
            let event = game.move_object_raw(copy.card, Zone::Graveyard);
            assert!(event.moved);
            assert_eq!(event.before.zone, Zone::Stack);
            assert_eq!(event.destination, Zone::Graveyard);
            assert!(!event.was_card);
            assert_eq!(game.state.objects[copy.card as usize].zone, Zone::Graveyard);
            game.finish_zone_events(&[event], &[]);
            game.state_based_actions();
            assert_eq!(game.state.objects[copy.card as usize].zone, Zone::Outside);
            game.restore(&before).unwrap();
        }
    }
}

#[test]
fn refute_counters_prepared_copy_before_its_own_discard_choice() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let permanent = put(
            &mut game,
            player,
            "Hallway Heckler // Vicious Verse",
            Zone::Battlefield,
        );
        game.prepare_permanent(game.object_ref(permanent));
        let copy = game.state.preparations.as_ref().unwrap()[0].spell_copy;
        let counterspell = put(&mut game, player.other(), "Refute", Zone::Hand);
        act(&mut game, Action::AnnounceExile { card: copy.card });
        for _ in 0..20 {
            if game.state.announcing.is_none() {
                break;
            }
            let actions = game.legal_actions();
            let next = actions.iter().find(|a| matches!(a, Action::ChooseTarget { target: Target::Player(p) } if *p == player.other()))
                .or_else(|| actions.iter().find(|a| matches!(a, Action::ChooseSpellCost { .. } | Action::FinishTargets | Action::PayCosts { .. })))
                .expect("prepared cast continuation").clone();
            act(&mut game, next);
        }
        assert_eq!(game.state.objects[copy.card as usize].zone, Zone::Stack);
        assert!(!game.is_prepared(game.object_ref(permanent)));
        act(&mut game, Action::Pass);
        cast(&mut game, counterspell, Some(Target::Spell(copy.card)));
        for _ in 0..20 {
            if matches!(game.state.pending, Pending::EffectDiscard { .. }) {
                break;
            }
            act(&mut game, Action::Pass);
        }
        assert!(matches!(game.state.pending, Pending::EffectDiscard { .. }));
        // Refute has countered the copy but has not finished resolving, so no
        // state-based action has yet removed the new graveyard incarnation.
        let current = game.object_ref(copy.card);
        assert_ne!(current, copy);
        assert_eq!(game.state.objects[copy.card as usize].zone, Zone::Graveyard);
        assert!(game.state.objects[copy.card as usize].is_spell_copy());
        assert!(!game.state.objects[copy.card as usize].is_card());
        for viewer in [player, player.other()] {
            let view = game.observe(viewer);
            assert!(
                view.objects
                    .iter()
                    .any(|object| object.reference == current && object.zone == Zone::Graveyard)
            );
        }
        let checkpoint = game.snapshot();
        let revision = game.revision();
        let discard = game
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::Discard { .. }))
            .unwrap();
        for _ in 0..2 {
            act(&mut game, discard.clone());
            settle(&mut game);
            assert_eq!(game.state.objects[copy.card as usize].zone, Zone::Outside);
            assert!(game.state.free_tokens.contains(&copy.card));
            assert!(!game.is_prepared(game.object_ref(permanent)));
            game.restore(&checkpoint).unwrap();
            assert!(game.revision() > revision);
            assert_eq!(
                game.apply_index(player.other(), revision, 0),
                Err(Error::StaleRevision)
            );
            assert_eq!(game.object_ref(copy.card), current);
            assert_eq!(game.state.objects[copy.card as usize].zone, Zone::Graveyard);
        }
    }
}

#[test]
fn prior_schema191_rejects_retained_outside_identity_counters() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        g.state.outside_game = Some(Box::new(OutsideGameState {
            mode: OutsideGameMode::CasualCollection,
            pools: [Vec::new(), Vec::new()],
            choice: None,
            pending_entry: None,
            pool_versions: [0; 2],
            next_choice_id: 0,
            known: [Vec::new(), Vec::new()],
        }));
        assert!(prior_schema191_state_supported(&g.state));
        let before = g.snapshot();
        g.state.outside_game.as_mut().unwrap().pool_versions[player.index()] = 1;
        assert!(!prior_schema191_state_supported(&g.state));
        g.restore(&before).unwrap();
        g.state.outside_game.as_mut().unwrap().next_choice_id = 1;
        assert!(!prior_schema191_state_supported(&g.state));
        g.restore(&before).unwrap();
        assert!(prior_schema191_state_supported(&g.state));
    }
}

#[test]
fn prior_schema192_rejects_outside_replacement_wire_and_owned_continuations() {
    assert_eq!(
        project_prior_schema192(r#"[{"decision":"priority"}]"#).unwrap(),
        r#"[{"decision":"priority"}]"#
    );
    for json in [
        r#"{"decision":"outside_entry_replacement"}"#,
        r#"{"actions":[{"kind":"ChooseOutsideEntryReplacement","ability":0}]}"#,
        r#"{"actions":[{"kind":"ChooseOutsideSelfReplacement","effect":0}]}"#,
        r#"{"outside_entry_replacement":{}}"#,
    ] {
        assert!(project_prior_schema192(json).is_err());
    }
    for owner in [Player::P0, Player::P1] {
        let mut game = scenario(owner);
        assert!(prior_schema192_state_supported(&game.state));
        game.state.outside_game = Some(Box::new(OutsideGameState {
            mode: OutsideGameMode::CasualCollection,
            pools: [Vec::new(), Vec::new()],
            choice: None,
            pending_entry: None,
            pool_versions: [0; 2],
            next_choice_id: 0,
            known: [Vec::new(), Vec::new()],
        }));
        assert!(!prior_schema192_state_supported(&game.state));
        // The saved checkpoint may contain an outside state even when the
        // current outer state no longer does.
        let saved = Box::new(game.state.clone());
        game.state.outside_game = None;
        game.state.entry_batch = Some(Box::new(entries::EntryBatch {
            atomic_cost: Some(Box::new(entries::AtomicCostCheckpoint {
                before: saved,
                bytes: 0,
                failure_pc: None,
                components: Vec::new(),
            })),
            temporary_copy: None,
            discard_context: None,
            moves: Vec::new(),
            choices: Vec::new(),
            next: 0,
            counter_next: 0,
            entry_bonuses_ready: false,
            counter_order: Vec::new(),
            timestamp_order: Vec::new(),
            timestamps_ready: false,
            resume: None,
            shuffle_after: None,
            replacement_resume: None,
            tap_events: Vec::new(),
            tap_next: 0,
        }));
        assert!(!prior_schema192_state_supported(&game.state));
        game.state.entry_batch = None;
        game.state.zone_rules = Some(Box::default());
        assert!(prior_schema192_state_supported(&game.state));
        let mut checkpoint = game.state.clone();
        checkpoint.outside_game = Some(Box::new(OutsideGameState {
            mode: OutsideGameMode::CasualCollection,
            pools: [Vec::new(), Vec::new()],
            choice: None,
            pending_entry: None,
            pool_versions: [0; 2],
            next_choice_id: 0,
            known: [Vec::new(), Vec::new()],
        }));
        let reference = game.object_ref(0);
        game.state.zone_rules = Some(Box::new(zone_replacements::test_checkpoint(
            checkpoint, reference, owner,
        )));
        assert!(!prior_schema192_state_supported(&game.state));
        game.state.zone_rules = None;
        let source = game.object_ref(0);
        let action = Action::ChooseOutsideEntryReplacement { source, ability: 0 };
        let wire = serde_json::to_string(&action).unwrap();
        assert!(wire.contains("\"kind\":\"ChooseOutsideEntryReplacement\""));
        assert!(project_prior_schema192(&wire).is_err());
        let view = OutsideEntryReplacementView {
            player: owner,
            card: Card::Forest,
            destination: OutsideEntryDestination::Hand,
            providers: vec![OutsideEntryProviderView {
                source,
                ability: 0,
                destination: OutsideEntryDestination::Exile,
            }],
            self_effects: Vec::new(),
        };
        let public = serde_json::json!({"outside_entry_replacement": view}).to_string();
        assert!(project_prior_schema192(&public).is_err());
    }
}

#[test]
fn prior_schema193_rejects_next_turn_keyword_grants() {
    let old = r#"[{"decision":"priority"}]"#;
    assert_eq!(project_prior_schema193(old).unwrap(), old);
    assert!(
        project_prior_schema193(r#"{"origin":{"ResolutionUntilNextTurn":{"controller":"P0"}}}"#)
            .is_err()
    );
    let mut game = scenario(Player::P0);
    assert!(prior_schema193_state_supported(&game.state));
    put(
        &mut game,
        Player::P0,
        "Elspeth, Storm Slayer",
        Zone::Battlefield,
    );
    assert!(!prior_schema193_state_supported(&game.state));
}
