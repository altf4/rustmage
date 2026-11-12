//! Printed Avatar rules exercised through the registered card definitions.
#[path = "tla_cards_white_colorless/choices.rs"]
mod choices;
#[path = "tla_cards_white_colorless/combat.rs"]
mod combat;
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn drain(game: &mut Game) {
    for _ in 0..40 {
        game.prepare_triggers();
        if game.state.pending == Pending::Priority
            && game.state.stack.is_empty()
            && game.state.resolving.is_none()
        {
            return;
        }
        choose(game, |action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. })
        });
    }
    panic!("card scenario exceeded its resolution budget");
}

fn target(game: &mut Game, target: Target) {
    act(game, Action::ChooseTarget { target });
}

fn optional(game: &mut Game, accept: bool) {
    act(game, Action::ChooseOptional { accept });
}

fn choose_effect(game: &mut Game, card: CardId) {
    let object = game.object_ref(card);
    act(game, Action::ChooseEffectObject { object });
}

fn finish_targets(game: &mut Game) {
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(game, Action::FinishTargets);
    }
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

fn life(game: &Game, player: Player) -> i128 {
    game.state.players[player.index()].life
}

fn draw_count(game: &Game, player: Player) -> u64 {
    game.state.players[player.index()].cards_drawn_this_turn
}

fn tokens_of(game: &Game, player: Player, subtype: &str) -> Vec<ObjectView> {
    game.state
        .objects
        .iter()
        .enumerate()
        .filter(|(_, object)| {
            object.zone == Zone::Battlefield && object.controller == player && object.is_token()
        })
        .map(|(id, _)| game.object_view(id as CardId))
        .filter(|view| view.has_subtype(subtype))
        .collect()
}
