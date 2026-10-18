//! Candidate endgame consumers: real Standard-legal FDN decks and the shared smoke policy.
use super::run_matrix_actions_observed_with_stride;
use rustmage::{Card, Observation, Outcome, Player, StackKind, Zone};
use std::collections::BTreeSet;

#[test]
fn foundations_endgame_cards_natural_games_and_owned_continuations() {
    // Only one deck contains Herald. Pact supplies an actual lose-game
    // obligation; Day of Judgment can end prevention without a forced outcome.
    let mut white = vec![Card::Plains; 18];
    white.extend([Card::Swamp; 18]);
    for name in [
        "Herald of Eternal Dawn",
        "Demonic Pact",
        "Day of Judgment",
        "Gleaming Barrier",
        "Crystal Barricade",
        "Pacifism",
    ] {
        white.extend([Card::from_name(name).unwrap(); 4]);
    }
    assert_eq!(white.len(), 60);
    let mut gates = Vec::new();
    for name in [
        "Azorius Guildgate",
        "Boros Guildgate",
        "Dimir Guildgate",
        "Golgari Guildgate",
        "Gruul Guildgate",
        "Izzet Guildgate",
        "Orzhov Guildgate",
        "Rakdos Guildgate",
        "Selesnya Guildgate",
        "Simic Guildgate",
        "Maze's End",
        "Time Stop",
        "Circuitous Route",
        "Fog Bank",
        "Gleaming Barrier",
    ] {
        gates.extend([Card::from_name(name).unwrap(); 4]);
    }
    assert_eq!(gates.len(), 60);
    let names = |o: &Observation, p: Player| -> BTreeSet<&'static str> {
        o.objects
            .iter()
            .filter(|v| v.zone == Zone::Battlefield && v.controller == p && v.has_subtype("Gate"))
            .map(|v| v.name())
            .collect()
    };
    let mut maze_winners = BTreeSet::new();
    let mut protected = BTreeSet::new();
    let mut stopped = BTreeSet::new();
    let mut owned_herald = 0;
    let mut owned_maze = 0;
    let mut owned_stop = 0;
    let mut observe = |before: &Observation, after: &Observation, owned: bool| {
        let live_herald = |o: &Observation, p: Player| {
            o.objects.iter().any(|v| {
                v.zone == Zone::Battlefield
                    && v.controller == p
                    && v.name() == "Herald of Eternal Dawn"
            })
        };
        if owned
            && before
                .objects
                .iter()
                .any(|v| v.zone == Zone::Battlefield && v.name() == "Herald of Eternal Dawn")
        {
            owned_herald += 1;
        }
        let source = before.resolving.as_ref().or_else(|| before.stack.last());
        if let Some(source) = source {
            if owned && source.source_card.name() == "Maze's End" {
                owned_maze += 1;
            }
            if owned && source.source_card.name() == "Time Stop" {
                owned_stop += 1;
            }
            if source.source_card.name() == "Maze's End" && source.kind != StackKind::Spell {
                let p = source.controller;
                if names(after, p).len() >= 10
                    && after.resolving.is_none()
                    && after.search_choice.is_none()
                {
                    if after.outcome == Some(Outcome::Winner(p)) {
                        maze_winners.insert(p);
                    }
                    // This witnesses an actual completed ten-name win check,
                    // rather than merely observing a Herald somewhere in play.
                    if after.outcome.is_none()
                        && live_herald(after, p.other())
                        && before.search_choice.is_some()
                    {
                        protected.insert(p.other());
                    }
                }
            }
            if source.source_card.name() == "Demonic Pact"
                && source.mode == Some(3)
                && matches!(source.kind, StackKind::TriggeredAbility { .. })
                && after.resolving.is_none()
                && !after.stack.iter().any(|v| v.id == source.id)
                && live_herald(after, source.controller)
                && after.outcome.is_none()
            {
                protected.insert(source.controller);
            }
            if source.source_card.name() == "Time Stop"
                && source.kind == StackKind::Spell
                && before
                    .objects
                    .iter()
                    .any(|v| v.reference == source.object && v.zone == Zone::Stack)
                && after
                    .objects
                    .iter()
                    .any(|v| v.id == source.card && v.zone == Zone::Exile)
                && after.stack.is_empty()
                && after.resolving.is_none()
            {
                stopped.insert(source.controller);
            }
        }
        for p in [Player::P0, Player::P1] {
            if live_herald(after, p)
                && after.players[p.index()].life <= 0
                && after.outcome.is_none()
            {
                protected.insert(p);
            }
        }
    };
    run_matrix_actions_observed_with_stride(
        &[white, gates],
        &["Herald of Eternal Dawn", "Time Stop"],
        &["Maze's End"],
        &["Maze's End"],
        &[],
        &[0, 1, 2, 3, 7, 13, 41, 73],
        Some(&mut observe),
        13,
    );
    println!(
        "FDN endgame witnesses: Maze winners={maze_winners:?}; protected={protected:?}; Time Stop={stopped:?}; owned Herald/Maze/Stop={owned_herald}/{owned_maze}/{owned_stop}"
    );
    assert_eq!(
        maze_winners.len(),
        2,
        "no distinct ten-Gate win in both seats"
    );
    assert_eq!(
        protected.len(),
        2,
        "no actual Herald prevention in both seats"
    );
    assert_eq!(stopped.len(), 2, "no Time Stop resolution in both seats");
    assert!(owned_herald > 0 && owned_maze > 0 && owned_stop > 0);
}
