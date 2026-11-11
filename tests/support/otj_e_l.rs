#![allow(dead_code)]

use super::support::{act, choose, pay};
use rustmage_core::{cards::*, engine::*};

pub const TREASURE: Card = Card::from_catalogue_id(5302);
pub const ZOMBIE_ROGUE: Card = Card::from_catalogue_id(5305);
pub const MERCENARY: Card = Card::from_catalogue_id(5612);

pub fn tokens(game: &Game, controller: Player, token: Card) -> Vec<CardId> {
    game.state
        .objects
        .iter()
        .enumerate()
        .filter(|(_, object)| {
            object.zone == Zone::Battlefield
                && object.controller == controller
                && object.card == token
        })
        .map(|(id, _)| id as CardId)
        .collect()
}

pub fn advance(game: &mut Game, reached: impl Fn(&Game) -> bool) {
    for _ in 0..96 {
        if reached(game) {
            return;
        }
        match game.state.pending {
            Pending::Priority => act(game, Action::Pass),
            Pending::Blockers => act(game, Action::FinishBlockers),
            Pending::OrderTriggers { .. } => {
                choose(game, |action| matches!(action, Action::PutTrigger { .. }))
            }
            ref pending => panic!("unexpected choice at {:?}: {pending:?}", game.state.step),
        }
    }
    panic!("scenario exceeded 96 actions at {:?}", game.state.step);
}

pub fn resolve(game: &mut Game) {
    game.prepare_triggers();
    advance(game, |game| {
        game.state.pending == Pending::Priority
            && game.state.stack.is_empty()
            && game.state.pending_triggers.is_empty()
            && game.state.ordering_triggers.is_empty()
            && game.state.resolving.is_none()
    });
}

pub fn checkpoint(game: &mut Game, action: Action) {
    advance(game, |game| game.legal_actions().contains(&action));
}

pub fn choose_when_ready(game: &mut Game, action: Action) {
    checkpoint(game, action.clone());
    act(game, action);
}

pub fn start_step(game: &mut Game, step: Step) {
    game.begin_step(step);
    game.give_priority();
    game.prepare_triggers();
}

pub fn choose_targets(game: &mut Game, targets: &[Target]) {
    for &target in targets {
        act(game, Action::ChooseTarget { target });
    }
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(game, Action::FinishTargets);
    }
}

pub fn spree(game: &mut Game, card: CardId, mode: u16, targets: &[Target]) {
    act(game, Action::AnnounceCast { card });
    act(game, Action::ChooseMode { mode });
    choose_targets(game, targets);
    pay(game);
}

pub fn cast_x(game: &mut Game, card: CardId, value: u32, targets: &[Target]) {
    act(game, Action::AnnounceCast { card });
    act(game, Action::ChooseX { value });
    choose_targets(game, targets);
    pay(game);
}

pub fn announce_activation(game: &mut Game, card: CardId, ability: u16) {
    act(
        game,
        Action::AnnounceActivate {
            card,
            ability,
            counter_payment: Counters::default(),
        },
    );
}

pub fn saddle(game: &mut Game, mount: CardId, riders: &[CardId]) {
    announce_activation(game, mount, 0);
    for &rider in riders {
        let object = game.object_ref(rider);
        act(game, Action::ChooseSaddler { object });
    }
    act(game, Action::FinishSaddle);
    resolve(game);
}

pub fn attack(game: &mut Game, card: CardId) {
    advance(game, |game| game.state.pending == Pending::Attackers);
    act(game, Action::Attack { card });
    act(game, Action::FinishAttackers);
}

pub fn apply_raw_and_index(game: &mut Game, action: Action) {
    let actor = game.actor().unwrap();
    let revision = game.revision();
    let index = game
        .legal_actions()
        .iter()
        .position(|legal| legal == &action)
        .unwrap();
    let snapshot = game.snapshot();
    let mut indexed = game.clone();
    indexed.apply_index(actor, revision, index).unwrap();
    act(game, action.clone());
    assert_eq!(game.snapshot(), indexed.snapshot());
    indexed.restore(&snapshot).unwrap();
    act(&mut indexed, action);
    assert_eq!(game.snapshot(), indexed.snapshot());
}
