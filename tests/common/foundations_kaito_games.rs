//! Natural set-only games supplement the exhaustive paid Kaito rules scenarios.
use super::{deck, run_matrix_actions_observed};
use rustmage::{Observation, StackKind, Zone};
use std::collections::BTreeSet;

#[test]
fn foundations_kaito_full_games_and_owned_loyalty_state() {
    let blue = deck(
        "Island",
        &[
            "Kaito, Cunning Infiltrator",
            "Spectral Sailor",
            "Lightshell Duo",
            "Cephalid Inkmage",
            "Skyship Buccaneer",
            "Opt",
            "Think Twice",
            "Unsummon",
            "Essence Scatter",
        ],
    );
    let white = deck(
        "Plains",
        &[
            "Healer's Hawk",
            "Leonin Vanguard",
            "Resolute Reinforcements",
            "Angel of Vitality",
            "Inspiring Overseer",
            "Serra Angel",
            "Pacifism",
            "Banishing Light",
            "Helpful Hunter",
        ],
    );
    let mut abilities = BTreeSet::new();
    let mut combat_trigger = false;
    let mut owned_loyalty_replays = 0usize;
    let mut observe = |before: &Observation, after: &Observation, replay: bool| {
        for item in &after.stack {
            if item.source_card.name() == "Kaito, Cunning Infiltrator" {
                match item.kind {
                    StackKind::ActivatedAbility { ability } => {
                        abilities.insert(ability);
                    }
                    StackKind::TriggeredAbility { .. } => combat_trigger = true,
                    _ => {}
                }
            }
        }
        if replay
            && before.objects.iter().any(|o| {
                o.zone == Zone::Battlefield && o.card.name() == "Kaito, Cunning Infiltrator"
            })
        {
            owned_loyalty_replays += 1;
        }
    };
    run_matrix_actions_observed(
        &[blue, white],
        &["Kaito, Cunning Infiltrator"],
        &[],
        &[],
        &[],
        &[7, 13, 41, 73],
        Some(&mut observe),
    );
    assert!(!abilities.is_empty(), "no natural loyalty activation");
    assert!(combat_trigger, "no natural combat-damage loyalty trigger");
    assert!(
        owned_loyalty_replays > 0,
        "no owned replay with Kaito on battlefield"
    );
    println!(
        "FDN Kaito acceptance: loyalty abilities={abilities:?}, combat trigger={combat_trigger}, owned loyalty replays={owned_loyalty_replays}"
    );
}
