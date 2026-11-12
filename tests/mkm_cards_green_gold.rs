//! Authored Murders at Karlov Manor card scenarios for collectors 148–219.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

#[path = "mkm_cards_green_gold/evidence.rs"]
mod evidence;

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

fn activation(card: CardId, ability: u16) -> Action {
    Action::AnnounceActivate {
        card,
        ability,
        counter_payment: Counters::default(),
    }
}

fn attack(game: &mut Game, cards: &[CardId]) {
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    for &card in cards {
        act(game, Action::Attack { card });
    }
    act(game, Action::FinishAttackers);
}

fn tokens(game: &Game, player: Player, factory: u16) -> Vec<CardId> {
    let card = Card::from_catalogue_id(factory);
    game.controlled_token_ids(player)
        .filter(|&id| game.object_view(id).card == card)
        .collect()
}

#[path = "mkm_cards_green_gold/face_down.rs"]
mod face_down;
