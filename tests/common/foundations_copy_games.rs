//! Four complete copy cards in natural FDN-only games; core regressions cover edges.
use super::{deck, run_matrix_actions_observed};
use rustmage::{Card, Observation, StackKind, Step, Zone};
use std::collections::BTreeSet;

fn copy_count(observation: &Observation, name: Option<&str>) -> usize {
    observation
        .objects
        .iter()
        .filter(|object| {
            object.zone == Zone::Battlefield
                && object
                    .copy_recipe
                    .is_some_and(|recipe| name.is_none_or(|name| recipe.definition.name() == name))
        })
        .count()
}

#[test]
fn foundations_four_copy_cards_natural_games_and_owned_copy_state() {
    let mut blue = deck(
        "Island",
        &[
            "High Fae Trickster",
            "Tolarian Terror",
            "Hedron Archive",
            "Think Twice",
            "Unsummon",
            "Self-Reflection",
            "Rite of Replication",
            "Extravagant Replication",
            "Homunculus Horde",
        ],
    );
    // Two Hordes and finite draw effects exercise real token growth without
    // manufacturing a prolonged mirror of multiplying engines.
    for name in ["Homunculus Horde"] {
        let definition = Card::from_name(name).unwrap();
        for card in blue.iter_mut().rev().filter(|c| **c == definition).take(2) {
            *card = Card::Island;
        }
    }
    let white = deck(
        "Plains",
        &[
            "Healer's Hawk",
            "Inspiring Overseer",
            "Serra Angel",
            "Leonin Vanguard",
            "Angel of Vitality",
            "Resolute Reinforcements",
            "Pacifism",
            "Day of Judgment",
            "Banishing Light",
        ],
    );
    let mut copied_names = BTreeSet::new();
    let mut horde_created_from_second_draw = false;
    let mut replication_upkeep_created_copy = false;
    let mut owned_copy_replays = 0;
    let mut observe = |before: &Observation, after: &Observation, owned_replay: bool| {
        let before_copies = copy_count(before, None);
        let after_copies = copy_count(after, None);
        if owned_replay && before_copies > 0 {
            owned_copy_replays += 1;
        }
        for object in &after.objects {
            if object.zone == Zone::Battlefield
                && let Some(recipe) = object.copy_recipe
            {
                assert!(object.is_token());
                copied_names.insert(recipe.definition.name());
            }
        }
        let resolving = before.resolving.as_ref().or_else(|| before.stack.last());
        if let Some(item) = resolving.filter(|item| item.kind != StackKind::Spell) {
            if item.source_card.name() == "Homunculus Horde"
                && item.draw_event.is_some_and(|event| event.ordinal == 2)
                && copy_count(after, Some("Homunculus Horde"))
                    > copy_count(before, Some("Homunculus Horde"))
            {
                horde_created_from_second_draw = true;
            }
            if item.source_card.name() == "Extravagant Replication"
                && before.step == Step::Upkeep
                && after_copies > before_copies
            {
                replication_upkeep_created_copy = true;
            }
        }
    };
    run_matrix_actions_observed(
        &[blue, white],
        &[
            "Self-Reflection",
            "Rite of Replication",
            "Extravagant Replication",
            "Homunculus Horde",
        ],
        &[],
        &[],
        &[],
        &[7, 41],
        Some(&mut observe),
    );
    assert!(horde_created_from_second_draw, "no second-draw Horde copy");
    assert!(
        replication_upkeep_created_copy,
        "no upkeep Replication copy"
    );
    assert!(copied_names.contains("Homunculus Horde"));
    assert!(
        owned_copy_replays > 0,
        "no owned replay with a runtime copy"
    );
    println!(
        "FDN four-copy acceptance: second-draw Horde and upkeep Replication created copies; {owned_copy_replays} owned copy-state replays; copied definitions={copied_names:?}"
    );
}
