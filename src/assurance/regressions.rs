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

/// Recording-only projection to the schema preceding behold and universal
/// basic-land mana. Reject cases whose old wire representation cannot preserve
/// the observed ability identity or land-subtype semantics.
fn project_prior_behold_json(input: &str) -> Result<String, String> {
    fn check(value: &serde_json::Value) -> Result<(), String> {
        match value {
            serde_json::Value::Array(items) => {
                for item in items {
                    check(item)?;
                }
            }
            serde_json::Value::Object(map) => {
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
                        matches!(
                            kind,
                            "TapMana"
                                | "TapGrantedMana"
                                | "AnnounceMana"
                                | "ActivateMana"
                                | "AnnounceGraveyardBorrowedAbility"
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
                    check(value)?;
                }
            }
            _ => {}
        }
        Ok(())
    }
    let parsed: serde_json::Value = serde_json::from_str(input).map_err(|e| e.to_string())?;
    check(&parsed)?;
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
    !journal.iter().any(|action| {
        matches!(
            action,
            Action::ChooseEntryBehold { .. }
                | Action::TapMana { .. }
                | Action::TapGrantedMana { .. }
                | Action::AnnounceMana { .. }
                | Action::ActivateMana { .. }
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

fn trace(card: Card, schema_projection: Option<u32>) -> Trace {
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
    if schema_projection.is_some_and(|schema| schema < 177) {
        if !prior_behold_owned_state_supported(&g) {
            projection_error = Some(
                "owned borrowing acquisition state cannot be reconstructed in the prior schema"
                    .to_owned(),
            );
        } else if !prior_behold_journal_supported(&journal) {
            projection_error =
                Some("journal contains behold or mana-ability actions whose old identities cannot be proved".to_owned());
        } else {
            match project_prior_behold_json(&observation_bytes) {
                Ok(projected) => observation_bytes = projected,
                Err(reason) => projection_error = Some(reason),
            }
        }
    }
    if schema_projection.is_some_and(|schema| schema < 177) {
        for value in ["true", "false"] {
            observation_bytes = observation_bytes.replace(
                &format!("\"loyalty_ability_activated_this_turn\":{value},"),
                "",
            );
        }
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
