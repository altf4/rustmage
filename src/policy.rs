//! Smoke policies for tests and benchmarks; not a training implementation.
use crate::cards::{CounterKind, Effect, Instruction, PlayerSet};
use crate::{Action, Game, StackKind, Step, Target, Zone};

/// Deterministic simple policy: keep, play lands, cast, attack, block, tap, pass.
/// Uses only the acting player's observation and the offered action list.
pub fn smoke_index(game: &Game, actions: &[Action]) -> usize {
    let actor = game.actor().expect("nonterminal game");
    let observation = game.observe(actor);
    let tap_for_main =
        observation.active == actor && matches!(observation.step, Step::Main1 | Step::Main2);
    let target_score = |program: &[Instruction], target: Target| -> i32 {
        let beneficial = program
            .iter()
            .find_map(|instruction| match instruction {
                Instruction::PutCounters { kind, .. } => Some(*kind == CounterKind::PlusOne),
                Instruction::Grow(amount) => Some(*amount >= 0),
                Instruction::Untap | Instruction::ReturnToBattlefield => Some(true),
                Instruction::Draw(PlayerSet::Targeted, _)
                | Instruction::GainLife(PlayerSet::Targeted, _) => Some(true),
                Instruction::ReturnToHand => Some(match target {
                    Target::Card(id) => observation
                        .objects
                        .iter()
                        .any(|o| o.id == id && o.zone == Zone::Graveyard),
                    _ => false,
                }),
                Instruction::Counter
                | Instruction::Destroy
                | Instruction::Damage(_)
                | Instruction::Tap
                | Instruction::LoseLife(PlayerSet::Targeted, _)
                | Instruction::Discard(PlayerSet::Targeted, _) => Some(false),
                _ => None,
            })
            .unwrap_or(false);
        let own = match target {
            Target::Player(player) => player == actor,
            Target::Card(id) => observation
                .objects
                .iter()
                .any(|o| o.id == id && o.owner == actor),
            Target::Spell(id) => observation.stack.iter().any(|item| {
                item.kind == StackKind::Spell && item.card == id && item.controller == actor
            }),
        };
        if own == beneficial { 70 } else { -10 }
    };
    let score = |action: &Action| -> i32 {
        match action {
            Action::Concede => -1000,
            Action::AnnounceCast { .. } => 79,
            Action::AnnounceActivate { .. } => 34,
            Action::AnnounceMana { .. } => -2,
            Action::PayCosts { .. } => 100,
            Action::Keep => 100,
            Action::Mulligan => -100,
            Action::PlayLand { .. } => 90,
            Action::BeginCast { .. } => 80,
            Action::ChooseTarget { target } => {
                let item = observation.announcing.as_ref().unwrap();
                let program = match item.kind {
                    StackKind::TriggeredAbility { ability } => {
                        item.source_card.triggers()[ability as usize].program
                    }
                    StackKind::ActivatedAbility { ability } => {
                        item.source_card.activations()[ability as usize].program
                    }
                    StackKind::Spell => match item.source_card.effect() {
                        Effect::Spell(_, program) => program,
                        _ => &[],
                    },
                };
                target_score(program, *target)
            }
            Action::FinishTargets => 0,
            Action::ChooseMana { production } => mana_score(&observation, *production),
            Action::ActivateMana { production, .. } => {
                if tap_for_main {
                    20 + mana_score(&observation, *production)
                } else {
                    -5
                }
            }
            Action::BeginActivate { .. } => 35,
            Action::Activate {
                card,
                ability,
                target,
                ..
            } => {
                let object = observation
                    .objects
                    .iter()
                    .find(|object| object.id == *card)
                    .unwrap();
                let program = object.card.activations()[*ability as usize].program;
                target
                    .map(|target| target_score(program, target) - 25)
                    .unwrap_or(35)
            }
            Action::Cast { card, target, .. } => {
                let definition = observation
                    .objects
                    .iter()
                    .find(|o| o.id == *card)
                    .unwrap()
                    .card;
                if definition.is_creature() || matches!(definition.effect(), Effect::Permanent) {
                    80
                } else {
                    let program = match definition.effect() {
                        Effect::Spell(_, program) => program,
                        _ => &[],
                    };
                    target
                        .map(|target| target_score(program, target))
                        .unwrap_or(70)
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

fn mana_score(observation: &crate::Observation, production: crate::Mana) -> i32 {
    let pool = observation.players[observation.viewer.index()].mana;
    observation
        .objects
        .iter()
        .filter(|object| object.owner == observation.viewer && object.zone == Zone::Hand)
        .map(|object| {
            let (required, _) = object.card.cost();
            (0..6)
                .map(|index| {
                    required[index]
                        .saturating_sub(pool[index])
                        .min(production[index]) as i32
                })
                .sum()
        })
        .max()
        .unwrap_or(0)
}
