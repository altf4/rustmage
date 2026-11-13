mod support;

use rustmage as _;
use rustmage_core::engine::characteristic_layers::AnimationTypeMode;
use rustmage_core::{cards::*, engine::*};
use support::*;

#[path = "tla_cards_multicolor/earth.rs"]
mod earth;

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

fn drain(game: &mut Game) {
    for _ in 0..64 {
        game.prepare_triggers();
        if game.state.stack.is_empty()
            && game.state.resolving.is_none()
            && game.state.pending == Pending::Priority
        {
            return;
        }
        choose(game, |action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. })
        });
    }
    panic!("multicolor card packet did not resolve");
}

fn life(game: &Game, player: Player) -> i128 {
    game.state.players[player.index()].life
}

fn draws(game: &Game, player: Player) -> u64 {
    game.state.players[player.index()].cards_drawn_this_turn
}

fn animate(game: &mut Game, id: CardId, base: Option<(i128, i128)>) {
    game.change_object_form(
        game.object_ref(id),
        AnimationSpec {
            artifact: false,
            creature_types: AnimationCreatureTypes::Unchanged,
            base,
            keywords: Keywords::NONE,
            duration: AnimationDuration::Permanent,
        },
        AnimationTypeMode::AddCreature,
    );
}

fn tokens(game: &Game, player: Player, factory: u16) -> Vec<CardId> {
    game.controlled_token_ids(player)
        .filter(|&id| game.object_view(id).card == Card(factory))
        .collect()
}

#[path = "tla_cards_multicolor/links.rs"]
mod links;

#[path = "tla_cards_multicolor/combat.rs"]
mod combat;

#[path = "tla_cards_multicolor/utility.rs"]
mod utility;

#[path = "tla_cards_multicolor/ally.rs"]
mod ally;

#[path = "tla_cards_multicolor/inherited.rs"]
mod inherited;
