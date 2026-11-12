//! Authored Murders at Karlov Manor gameplay for collectors 73 through 147.
mod support;

use rustmage as _;

#[path = "mkm_black_red_cards/cases_and_costs.rs"]
mod cases_and_costs;

#[path = "mkm_black_red_cards/exile_and_evidence.rs"]
mod exile_and_evidence;

#[path = "mkm_black_red_cards/disguise.rs"]
mod disguise;

#[path = "mkm_black_red_cards/black.rs"]
mod black;

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
    activation_at(card, 0)
}

fn activation_at(card: CardId, ability: u16) -> Action {
    Action::AnnounceActivate {
        card,
        ability,
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

fn begin_attack(game: &mut Game, attacker: CardId) {
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    act(game, Action::Attack { card: attacker });
    act(game, Action::FinishAttackers);
}

fn life(game: &Game, player: Player) -> i128 {
    game.state.players[player.index()].life
}
