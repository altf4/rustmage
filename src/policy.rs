//! Smoke policies for tests and benchmarks; not a training implementation.
use crate::{Action, Game, Step, Target};

/// Deterministic simple policy: keep, play lands, cast, attack, block, tap, pass.
/// Uses only the acting player's observation and the offered action list.
pub fn smoke_index(game: &Game, actions: &[Action]) -> usize {
    let actor = game.actor().expect("nonterminal game");
    let observation = game.observe(actor);
    let tap_for_main =
        observation.active == actor && matches!(observation.step, Step::Main1 | Step::Main2);
    let score = |action: &Action| -> i32 {
        match action {
            Action::Concede => -1000,
            Action::Keep => 100,
            Action::Mulligan => -100,
            Action::PlayLand { .. } => 90,
            Action::Cast { card, target, .. } => {
                let definition = observation
                    .objects
                    .iter()
                    .find(|o| o.id == *card)
                    .unwrap()
                    .card;
                if definition.is_creature() {
                    80
                } else {
                    match target {
                        Some(Target::Player(p)) if *p == actor.other() => 85,
                        Some(Target::Card(id)) => {
                            let own = observation
                                .objects
                                .iter()
                                .find(|o| o.id == *id)
                                .is_some_and(|o| o.owner == actor);
                            if (definition == crate::Card::GiantGrowth) == own {
                                70
                            } else {
                                -10
                            }
                        }
                        Some(Target::Spell(id))
                            if observation
                                .stack
                                .iter()
                                .any(|s| s.card == *id && s.controller != actor) =>
                        {
                            75
                        }
                        None => 70,
                        _ => -10,
                    }
                }
            }
            Action::Attack { .. } | Action::Block { .. } => 60,
            Action::AssignDamage { amount, .. } => 50 + *amount,
            Action::Bottom { .. } | Action::Discard { .. } => 50,
            Action::PutTrigger { .. } => 50,
            Action::ChooseOptional { accept } => {
                if *accept {
                    50
                } else {
                    0
                }
            }
            Action::TapMana { .. } => {
                if tap_for_main {
                    10
                } else {
                    -20
                }
            }
            Action::Pass | Action::FinishAttackers | Action::FinishBlockers => 0,
        }
    };
    actions
        .iter()
        .enumerate()
        .max_by_key(|(_, a)| score(a))
        .map(|(i, _)| i)
        .expect("legal action")
}
