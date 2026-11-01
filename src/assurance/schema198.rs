//! Schema 198 adds Hartford rules and history absent from published schema 197.
//! Compatible observations keep their bytes; new states are rejected explicitly.
use super::*;
use serde::Serialize;

const NEW_CARDS: &[&str] = &[
    "Carnage, Crimson Chaos",
    "Cavern of Souls",
    "Consult the Star Charts",
    "Cool but Rude",
    "Day of Black Sun",
    "Floodpits Drowner",
    "Gene Pollinator",
    "Hydro-Man, Fluid Felon",
    "Inti, Seneschal of the Sun",
    "Jeskai Revelation",
    "Leatherhead, Swamp Stalker",
    "Lively Dirge",
    "Monument to Endurance",
    "Obsessive Pursuit",
    "Perilous Snare",
    "Pinnacle Starcage",
    "Quantum Riddler",
    "Repurposing Bay",
    "Riverchurn Monument",
    "The Fire Crystal",
    "Tishana's Tidebinder",
    "Torch the Tower",
    "United Battlefront",
    "Wan Shi Tong, Librarian",
    "Warmaker Gunship",
    "Webstrike Elite",
    "Winternight Stories",
    "@token/tdm-monk-prowess",
];
fn new_card(card: Card) -> bool {
    NEW_CARDS.contains(&card.identity())
}
fn check(value: &serde_json::Value) -> Result<(), String> {
    match value {
        serde_json::Value::String(text) => {
            if NEW_CARDS.contains(&text.as_str())
                || matches!(
                    text.as_str(),
                    "GraveyardCreatures"
                        | "SelectGraveyardReturn"
                        | "FinishGraveyardReturn"
                        | "KindsManaValue"
                        | "AnnounceSourceGrant"
                        | "Speed"
                        | "ChooseDrawReplacement"
                        | "draw_replacement"
                        | "AnnounceMayhem"
                        | "Mayhem"
                        | "StartOfNextEndStep"
                        | "AttacksEachCombat"
                        | "CreatureSpell"
                        | "Uncounterable"
                        | "LandUntilControllerNextTurn"
                )
            {
                return Err("new Hartford rule has no schema 197 representation".into());
            }
        }
        serde_json::Value::Array(rows) => {
            rows.iter().try_for_each(check)?;
        }
        serde_json::Value::Object(fields) => {
            for (key, value) in fields {
                if matches!(
                    key.as_str(),
                    "speed"
                        | "speed_triggered_this_turn"
                        | "sacrificed_mana_value"
                        | "permanents_sacrificed_this_turn"
                        | "mayhem_discards_this_turn"
                        | "draw_replacement"
                        | "batch_checked"
                        | "excluded_doublers"
                        | "draw_order"
                        | "attack_trigger_after"
                        | "GraveyardCreatures"
                        | "SelectGraveyardReturn"
                        | "FinishGraveyardReturn"
                        | "KindsManaValue"
                        | "AnnounceSourceGrant"
                        | "ChooseDrawReplacement"
                        | "AnnounceMayhem"
                        | "StartOfNextEndStep"
                        | "CreatureSpell"
                        | "LandUntilControllerNextTurn"
                ) {
                    return Err("new Hartford field has no schema 197 representation".into());
                }
                check(value)?;
            }
        }
        _ => {}
    }
    Ok(())
}
fn wire_supported(value: &impl Serialize) -> bool {
    serde_json::to_value(value).is_ok_and(|value| check(&value).is_ok())
}
pub(super) fn project_prior_schema198(input: &str) -> Result<String, String> {
    check(&serde_json::from_str(input).map_err(|e| e.to_string())?)?;
    Ok(input.to_owned())
}

/// Draw requests, pending entries and mana rollback own continuation data that
/// is not completely exposed by observations. Reject these opaque continuations
/// until an old-schema future-behavior equivalence is established. Ordinary
/// announcements/resolutions and schema-197 cards retain their existing shape.
pub(super) fn prior_schema198_state_supported(state: &State) -> bool {
    if state.players.iter().any(|p| {
        p.speed != 0 || p.speed_triggered_this_turn || p.permanents_sacrificed_this_turn != 0
    }) || state
        .mayhem_discards
        .as_ref()
        .is_some_and(|rows| !rows.is_empty())
        || state.miracle.is_some()
        || state.cast_rollback.is_some()
        || state.mana_announcement_stack.is_some()
        || state.entry_batch.is_some()
        || state
            .zone_rules
            .as_ref()
            .is_some_and(|rules| rules.has_checkpoint())
        || state.objects.iter().any(|o| new_card(o.card))
        || state
            .outside_game
            .as_ref()
            .is_some_and(|outside| outside.pools.iter().flatten().copied().any(new_card))
        || state
            .copy_recipes
            .as_ref()
            .is_some_and(|rows| rows.iter().any(|r| !wire_supported(&r.recipe)))
        || state.temporary_copies.as_ref().is_some_and(|rows| {
            rows.iter().any(|r| {
                new_card(r.physical_card)
                    || r.original_recipe
                        .as_ref()
                        .is_some_and(|recipe| !wire_supported(recipe))
                    || r.layers
                        .iter()
                        .any(|layer| !wire_supported(&layer.value.recipe))
            })
        })
        || state
            .borrowed_slots
            .as_ref()
            .is_some_and(|rows| rows.iter().any(|r| r.definition.is_some_and(new_card)))
        || state
            .temporary_abilities
            .as_ref()
            .is_some_and(|rows| rows.iter().any(|r| new_card(r.granting_card)))
        || state
            .self_granted_abilities
            .as_ref()
            .is_some_and(|rows| rows.iter().any(|r| !wire_supported(r)))
        || state.graveyard_borrowing.as_ref().is_some_and(|rows| {
            rows.iter()
                .any(|r| new_card(r.provider) || new_card(r.definition))
        })
        || state.roles.as_ref().is_some_and(|roles| {
            roles
                .source_linked_effects
                .iter()
                .any(|row| !wire_supported(row))
        })
        || state
            .exile_permissions
            .iter()
            .any(|permission| !wire_supported(permission))
        || state.delayed_sacrifices.as_ref().is_some_and(|rows| {
            rows.iter()
                .any(|r| !wire_supported(&r.view) || !wire_supported(&r.facts))
        })
        || state
            .adventures
            .as_ref()
            .is_some_and(|a| a.permissions.iter().any(|v| new_card(v.card)))
        || state
            .plot
            .as_ref()
            .and_then(|p| p.payment.as_ref())
            .is_some_and(|p| new_card(p.source_card))
        || state
            .hits
            .iter()
            .any(|hit| !wire_supported(&hit.source.facts))
        || state
            .damage_choices
            .iter()
            .any(|hit| !wire_supported(&hit.source.facts))
        || state
            .last_target_library_move
            .as_ref()
            .is_some_and(|event| new_card(event.card))
        || state
            .explore_data
            .as_ref()
            .and_then(|data| data.last.as_ref())
            .is_some_and(|event| {
                new_card(event.explorer.card)
                    || event.revealed.is_some_and(|card| new_card(card.card))
            })
        || state
            .library_top
            .as_ref()
            .and_then(|data| data.reveal.as_ref())
            .is_some_and(|event| new_card(event.revealed.card))
        || state.manifest.as_ref().is_some_and(|m| {
            m.permanents
                .iter()
                .chain(m.spells.iter())
                .any(|r| new_card(r.underlying))
        })
    {
        return false;
    }
    if let Some(mana) = &state.mana_lots {
        if mana.trace.is_some()
            || mana.replay.is_some()
            || mana.choice.is_some()
            || mana.prepared.is_some()
            || mana
                .pools
                .iter()
                .flatten()
                .any(|lot| !wire_supported(&lot.quality))
            || mana
                .effect_quality
                .as_ref()
                .is_some_and(|q| !wire_supported(q))
            || mana
                .announcement_quality
                .as_ref()
                .is_some_and(|(_, q)| !wire_supported(q))
        {
            return false;
        }
    }
    if let Some(resolving) = &state.resolving {
        if resolving
            .search
            .as_ref()
            .is_some_and(|s| !wire_supported(&s.spec))
            || resolving
                .choice
                .as_ref()
                .is_some_and(|c| !wire_supported(&c.kind))
        {
            return false;
        }
    }
    state
        .stack
        .iter()
        .chain(&state.pending_triggers)
        .chain(&state.ordering_triggers)
        .chain(state.announcing.iter().map(|a| &a.item))
        .chain(state.activation_payment.iter().map(|a| &a.item))
        .chain(state.resolving.iter().map(|r| &r.item))
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
                .flat_map(|m| m.deaths.iter().map(|d| &d.source)),
        )
        .all(wire_supported)
}

#[test]
fn prior_197_retains_published_cards_and_rejects_198_wire_markers() {
    assert_eq!(NEW_CARDS.len(), 28);
    for card in [
        "Greedy Freebooter",
        "Sunderflock",
        "Vren, the Relentless",
        "Hearth Elemental // Stoke Genius",
        "The Enigma Jewel // Locus of Enlightenment",
    ] {
        let value = format!(
            r#"{{"schema":197,"card":"{card}","restriction":"CastFromExile","source_less_rule":"Radiation"}}"#
        );
        assert_eq!(project_prior_schema198(&value).unwrap(), value);
    }
    for card in NEW_CARDS {
        assert!(project_prior_schema198(&format!(r#"{{"card":"{card}"}}"#)).is_err());
    }
    for value in [
        r#"{"speed":1}"#,
        r#"{"sacrificed_mana_value":2}"#,
        r#"{"permanents_sacrificed_this_turn":1}"#,
        r#"{"mayhem_discards_this_turn":[1]}"#,
        r#"{"draw_replacement":{"player":"P0"}}"#,
        r#"{"action":"ChooseDrawReplacement"}"#,
        r#"{"action":"AnnounceSourceGrant"}"#,
        r#"{"action":"SelectGraveyardReturn"}"#,
        r#"{"kind":{"GraveyardCreatures":{"maximum":2}}}"#,
        r#"{"method":"Mayhem"}"#,
        r#"{"expiry":{"StartOfNextEndStep":"P1"}}"#,
        r#"{"kind":"AttacksEachCombat"}"#,
        r#"{"source_less_rule":"Speed"}"#,
        r#"{"kind":"LandUntilControllerNextTurn"}"#,
        r#"{"restriction":{"CreatureSpell":"Bird"}}"#,
        r#"{"spell_bonus":"Uncounterable"}"#,
    ] {
        assert!(project_prior_schema198(value).is_err(), "{value}");
    }
}

#[test]
fn prior_197_rejects_new_owned_history_and_restores_an_independent_branch() {
    let mut game = scenario(Player::P0);
    assert!(prior_schema198_state_supported(&game.state));
    let saved = game.snapshot();
    game.state.players[0].speed = 1;
    assert!(!prior_schema198_state_supported(&game.state));
    let mut branch = game.clone();
    drop(game);
    assert!(!prior_schema198_state_supported(&branch.state));
    branch.restore(&saved).unwrap();
    assert!(prior_schema198_state_supported(&branch.state));
    branch.state.players[1].permanents_sacrificed_this_turn = 1;
    assert!(!prior_schema198_state_supported(&branch.state));
    branch.restore(&saved).unwrap();
    let card = put(&mut branch, Player::P0, "Forest", Zone::Hand);
    let object = branch.object_ref(card);
    branch
        .state
        .mayhem_discards
        .get_or_insert_default()
        .push(object);
    assert!(!prior_schema198_state_supported(&branch.state));
}
