//! Scenario helpers for the Edge of Eternities 001–066 card targets.
#![allow(dead_code)]
use super::*;

pub(super) fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

pub(super) fn advance(game: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..80 {
        if ready(game) {
            return;
        }
        choose(game, |action| {
            matches!(
                action,
                Action::Pass
                    | Action::FinishAttackers
                    | Action::FinishBlockers
                    | Action::PutTrigger { .. }
            )
        });
    }
    panic!(
        "scenario did not reach its next choice at {:?}",
        game.state.pending
    );
}

pub(super) fn choose_target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

pub(super) fn trigger_target(game: &mut Game, card: CardId) {
    let action = choose_target(card);
    advance(game, |game| game.legal_actions().contains(&action));
    act(game, action);
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(game, Action::FinishTargets);
    }
}

pub(super) fn activate(game: &mut Game, card: CardId, ability: u16, target: Option<CardId>) {
    act(
        game,
        Action::AnnounceActivate {
            card,
            ability,
            counter_payment: Counters::default(),
        },
    );
    if let Some(recipient) = target {
        act(game, choose_target(recipient));
    }
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(game);
    }
}

pub(super) fn warp(game: &mut Game, card: CardId) {
    act(game, Action::AnnounceWarp { card });
    pay(game);
}

pub(super) fn end_step(game: &mut Game) {
    game.state.step = Step::Main2;
    game.state.priority = game.state.active;
    pass_pair(game);
    assert_eq!(game.state.step, Step::End);
}

pub(super) fn battlefield_tokens(game: &Game, player: Player, card: Card) -> Vec<ObjectRef> {
    game.observe(player)
        .objects
        .into_iter()
        .filter(|object| {
            object.zone == Zone::Battlefield
                && object.controller == player
                && object.card == card
                && object.token.is_some()
        })
        .map(|object| object.reference)
        .collect()
}

pub(super) fn station(game: &mut Game, ship: CardId, donor: CardId) {
    activate(game, ship, 0, None);
    let object = game.object_ref(donor);
    act(game, Action::ChooseAdditionalCost { object });
    pay(game);
}

pub(super) fn opponent_priority(game: &mut Game, opponent: Player) {
    if game.actor() != Some(opponent) {
        act(game, Action::Pass);
    }
    assert_eq!(game.actor(), Some(opponent));
}
