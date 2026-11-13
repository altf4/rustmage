//! Avatar mono-black definitions exercised through the normal game API.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

#[path = "tla_cards_black/graveyard_and_waterbend.rs"]
mod graveyard_and_waterbend;

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn finish_ordered_triggers(game: &mut Game) {
    for _ in 0..32 {
        if game.state.pending == Pending::Priority
            && game.state.stack.is_empty()
            && game.state.pending_triggers.is_empty()
        {
            return;
        }
        choose(game, |action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. })
        });
    }
    panic!("trigger resolution exceeded its action budget");
}

fn start_end_step(game: &mut Game, player: Player) {
    game.state.active = player;
    game.begin_step(Step::End);
    game.give_priority();
    game.prepare_triggers();
}

fn token_count(game: &Game, card: Card) -> usize {
    game.state
        .objects
        .iter()
        .filter(|object| {
            object.zone == Zone::Battlefield && object.is_token() && object.card == card
        })
        .count()
}

#[path = "tla_cards_black/other_cards.rs"]
mod other_cards;

#[path = "tla_cards_black/linked_and_vehicles.rs"]
mod linked_and_vehicles;

fn activate(card: CardId, ability: u16) -> Action {
    Action::AnnounceActivate {
        card,
        ability,
        counter_payment: Counters::default(),
    }
}

fn reach(game: &mut Game, condition: impl Fn(&Game) -> bool) {
    for _ in 0..48 {
        if condition(game) {
            return;
        }
        choose(game, |action| {
            matches!(
                action,
                Action::Pass
                    | Action::PutTrigger { .. }
                    | Action::FinishAttackers
                    | Action::FinishBlockers
            )
        });
    }
    panic!("requested action checkpoint was not reached");
}

fn finish_targets(game: &mut Game) {
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(game, Action::FinishTargets);
    }
}

fn declare_attackers(game: &mut Game, attackers: &[CardId]) {
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    for &card in attackers {
        act(game, Action::Attack { card });
    }
    act(game, Action::FinishAttackers);
}

fn sacrifice(game: &mut Game, card: CardId) {
    let object = game.object_ref(card);
    act(game, Action::Sacrifice { object });
}

fn crew(game: &mut Game, vehicle: CardId, ability: u16, contributor: CardId) {
    act(game, activate(vehicle, ability));
    let object = game.object_ref(contributor);
    act(game, Action::ChooseCrewer { object });
    if game.legal_actions().contains(&Action::FinishCrew) {
        act(game, Action::FinishCrew);
    }
    if game
        .legal_actions()
        .iter()
        .any(|a| matches!(a, Action::PayCosts { .. }))
    {
        pay(game);
    }
    settle(game);
}

#[path = "tla_cards_black/inherited.rs"]
mod inherited;
