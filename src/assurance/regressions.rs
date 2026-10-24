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
            "Cosmium Blast"
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
        }
    }
    Trace {
        root: root.into(),
        status: status.into(),
        decisions: journal.len(),
        zone: format!("{:?}", g.object_view(id).zone),
        journal: digest(&serde_json::to_vec(&journal).unwrap()),
        observations: digest(&serde_json::to_vec(&views).unwrap()),
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
