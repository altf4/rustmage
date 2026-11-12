//! Authored Murders at Karlov Manor gameplay for collectors 73 through 147.
mod support;

use rustmage as _;

#[path = "mkm_black_red_cards/cases_and_costs.rs"]
mod cases_and_costs;

#[path = "mkm_black_red_cards/exile_and_evidence.rs"]
mod exile_and_evidence;

use rustmage_core::{cards::*, engine::*};
use support::{act, choose, pay};

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn enter_step(game: &mut Game, step: Step) {
    game.begin_step(step);
    game.prepare_triggers();
}

fn target(game: &mut Game, card: CardId) {
    act(
        game,
        Action::ChooseTarget {
            target: Target::Card(card),
        },
    );
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(game, Action::FinishTargets);
    }
}

fn activation(card: CardId) -> Action {
    Action::AnnounceActivate {
        card,
        ability: 0,
        counter_payment: Counters::default(),
    }
}

fn additional(game: &mut Game, card: CardId) {
    let object = game.object_ref(card);
    act(game, Action::ChooseAdditionalCost { object });
    if matches!(game.state.pending, Pending::CastCosts { .. }) {
        choose(game, |action| {
            matches!(action, Action::ChooseCastCost { .. })
        });
    }
    pay(game);
}

fn tokens(game: &Game, player: Player, card: Card) -> Vec<CardId> {
    game.controlled_token_ids(player)
        .filter(|&id| game.object_view(id).card == card)
        .collect()
}
