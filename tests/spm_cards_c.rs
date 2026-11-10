//! Paid rules scenarios for Marvel's Spider-Man members 097–144.
#[path = "spm_cards_c/cast_costs.rs"]
mod cast_costs;
#[path = "spm_cards_c/events.rs"]
mod events;
#[path = "spm_cards_c/layers.rs"]
mod layers;
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn target(game: &mut Game, target: Target) {
    act(game, Action::ChooseTarget { target });
}

fn activate(game: &mut Game, card: CardId, ability: u16) {
    act(
        game,
        Action::AnnounceActivate {
            card,
            ability,
            counter_payment: Counters::default(),
        },
    );
}

fn cleanup(game: &mut Game) {
    game.begin_step(Step::End);
    game.give_priority();
    pass_pair(game);
}

fn start_step(game: &mut Game, step: Step) {
    game.begin_step(step);
    game.give_priority();
    game.prepare_triggers();
}

fn declare(game: &mut Game, attackers: &[CardId]) {
    start_step(game, Step::BeginCombat);
    pass_pair(game);
    for &card in attackers {
        act(game, Action::Attack { card });
    }
    act(game, Action::FinishAttackers);
}
