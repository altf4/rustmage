//! Bounded owned rollouts from a naturally reached FDN private continuation.
use super::deck;
use rustmage::{Action, Config, DeckRules, Error, Game, Player};
use std::time::Instant;

fn public_views(game: &Game) -> [serde_json::Value; 2] {
    [Player::P0, Player::P1].map(|viewer| {
        let mut observation = game.observe(viewer);
        // Restore deliberately invalidates action revisions, not game semantics.
        observation.revision = 0;
        serde_json::to_value(observation).unwrap()
    })
}

#[test]
fn foundations_private_choice_owned_sibling_rollouts() {
    let blue = deck(
        "Island",
        &[
            "Opt",
            "Think Twice",
            "Lightshell Duo",
            "Cephalid Inkmage",
            "Skyship Buccaneer",
            "Tolarian Terror",
            "Spectral Sailor",
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
    let mut branches = 0usize;
    let mut decisions = 0usize;
    let mut seconds = 0.0;
    let mut maximum_state_bytes = 0;
    for starter in [Player::P0, Player::P1] {
        let mut parent = Game::new(Config {
            decks: if starter == Player::P0 {
                [blue.clone(), white.clone()]
            } else {
                [white.clone(), blue.clone()]
            },
            seed: 7,
            starting_player: starter,
            deck_rules: DeckRules::Standard,
        })
        .unwrap();
        let mut found = false;
        for _ in 0..2_000 {
            let actor = parent.actor().expect("game ended before a private choice");
            let observation = parent.observe(actor);
            if observation.library_choice.is_some() {
                assert!(observation.resolving.is_some());
                assert!(parent.observe(actor.other()).library_choice.is_none());
                found = true;
                break;
            }
            let actions = parent.legal_actions();
            let index = rustmage::policy::smoke_index(&parent, &actions);
            assert!(!matches!(actions[index], Action::Concede));
            parent.apply(actor, actions[index].clone()).unwrap();
        }
        assert!(found, "no natural private library choice for {starter:?}");
        let actor = parent.actor().unwrap();
        let root_actions = parent.legal_actions();
        let choices: Vec<_> = root_actions
            .iter()
            .enumerate()
            .filter(|(_, action)| !matches!(action, Action::Concede))
            .map(|(index, _)| index)
            .take(2)
            .collect();
        assert_eq!(choices.len(), 2, "root must offer distinct sibling choices");
        let saved = parent.snapshot();
        let root_views = public_views(&parent);
        let root_revision = parent.revision();
        maximum_state_bytes = maximum_state_bytes.max(parent.state_bytes());
        let start = Instant::now();
        let mut siblings = Vec::new();
        for _ in 0..4 {
            for &index in &choices {
                siblings.push((index, parent.clone(), parent.clone()));
            }
        }
        drop(parent);
        let mut first_states = Vec::new();
        for (root_index, mut raw, mut indexed) in siblings {
            assert_eq!(
                raw.apply(actor.other(), root_actions[root_index].clone()),
                Err(Error::WrongActor)
            );
            assert_eq!(raw.snapshot(), saved);
            for step in 0..96 {
                if raw.outcome().is_some() {
                    break;
                }
                let actor = raw.actor().unwrap();
                assert_eq!(indexed.actor(), Some(actor));
                let actions = raw.legal_actions();
                assert_eq!(actions, indexed.legal_actions());
                let index = if step == 0 {
                    root_index
                } else {
                    rustmage::policy::smoke_index(&raw, &actions)
                };
                assert!(!matches!(actions[index], Action::Concede));
                let revision = indexed.revision();
                raw.apply(actor, actions[index].clone()).unwrap();
                indexed.apply_index(actor, revision, index).unwrap();
                assert_eq!(raw.snapshot(), indexed.snapshot());
                assert_eq!(public_views(&raw), public_views(&indexed));
                if step == 0 && first_states.len() < 2 {
                    first_states.push(raw.snapshot());
                }
                decisions += 2;
            }
            assert_ne!(raw.snapshot(), saved, "rollout never advanced");
            raw.restore(&saved).unwrap();
            indexed.restore(&saved).unwrap();
            assert_eq!(raw.snapshot(), saved);
            assert_eq!(indexed.snapshot(), saved);
            assert_eq!(public_views(&raw), root_views);
            assert_eq!(public_views(&indexed), root_views);
            assert_eq!(
                indexed.apply_index(actor, root_revision, root_index),
                Err(Error::StaleRevision)
            );
            assert_eq!(indexed.snapshot(), saved);
            raw.apply(actor, root_actions[root_index].clone()).unwrap();
            let revision = indexed.revision();
            indexed.apply_index(actor, revision, root_index).unwrap();
            assert_eq!(raw.snapshot(), indexed.snapshot());
            decisions += 2;
            branches += 2;
        }
        assert_ne!(
            first_states[0], first_states[1],
            "sibling choices collapsed"
        );
        seconds += start.elapsed().as_secs_f64();
    }
    assert_eq!(branches, 32);
    assert!(decisions > branches * 2, "no multi-action continuation");
    println!(
        "FDN owned rollout verification: branches={branches}, decisions={decisions}, seconds={seconds:.6}, branches_per_second={:.3}, decisions_per_second={:.3}, maximum_root_state_bytes_excluding_capacity_and_allocator={maximum_state_bytes}; two natural private-choice roots, 96-step budget; cloning, snapshots and both-player JSON observation checks included; trusted simulator, not hidden-information MCTS",
        branches as f64 / seconds,
        decisions as f64 / seconds,
    );
}
