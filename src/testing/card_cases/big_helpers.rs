//! Focused printed outcomes; shared engine invariants live in core tests.
//! Shared BIG scenario helpers; paid mutations retain owned raw/index replay checks.
use super::*;

pub(in crate::engine::testing) fn setup(player: Player) -> Game {
    let mut game = super::scenario(player);
    for state in &mut game.state.players {
        state.mana = [30; 6];
    }
    // Preserve the existing BIG fixtures' twenty-card baseline while sharing the
    // bounded assurance scenario's opening-state setup.
    for owner in [Player::P0, Player::P1] {
        for _ in game.state.players[owner.index()].library.len()..20 {
            super::put(&mut game, owner, "Forest", Zone::Library);
        }
    }
    game
}

pub(in crate::engine::testing) fn spawn(
    game: &mut Game,
    player: Player,
    card: Card,
    zone: Zone,
) -> CardId {
    super::put(game, player, card.identity(), zone)
}

pub(in crate::engine::testing) fn named(
    game: &mut Game,
    player: Player,
    name: &str,
    zone: Zone,
) -> CardId {
    super::put(game, player, name, zone)
}

pub(in crate::engine::testing) fn checked(game: &mut Game, action: Action) {
    super::act(game, action);
}

pub(in crate::engine::testing) fn cast(
    game: &mut Game,
    card: CardId,
    mode: Option<u16>,
    targets: &[Target],
) {
    checked(game, Action::AnnounceCast { card });
    if let Some(mode) = mode {
        checked(game, Action::ChooseMode { mode });
    }
    for &target in targets {
        for _ in 0..3 {
            if game
                .legal_actions()
                .contains(&Action::ChooseTarget { target })
            {
                break;
            }
            checked(game, Action::FinishTargets);
        }
        checked(game, Action::ChooseTarget { target });
    }
    for _ in 0..8 {
        if game.state.announcing.is_none() {
            return;
        }
        let action = game
            .legal_actions()
            .into_iter()
            .find(|action| matches!(action, Action::FinishTargets))
            .or_else(|| {
                game.legal_actions()
                    .into_iter()
                    .find(|action| matches!(action, Action::PayCosts { .. }))
            })
            .unwrap();
        checked(game, action);
    }
    panic!("BIG cast remained unpaid");
}

pub(in crate::engine::testing) fn settle(game: &mut Game) {
    game.prepare_triggers();
    for _ in 0..100 {
        if game.outcome().is_some()
            || (game.state.pending == Pending::Priority
                && game.state.stack.is_empty()
                && game.state.announcing.is_none()
                && game.state.resolving.is_none()
                && game.state.pending_triggers.is_empty())
        {
            return;
        }
        let actions = game.legal_actions();
        let action = if game.state.pending == Pending::Priority {
            Action::Pass
        } else {
            actions
                .iter()
                .find(|action| matches!(action, Action::PayCosts { .. }))
                .or_else(|| {
                    actions
                        .iter()
                        .find(|action| matches!(action, Action::FinishTargets))
                })
                .or_else(|| {
                    actions
                        .iter()
                        .find(|action| !matches!(action, Action::Concede))
                })
                .unwrap()
                .clone()
        };
        checked(game, action);
    }
    panic!("unsettled BIG scenario {:?}", game.state.pending);
}

pub(in crate::engine::testing) fn next_turn(game: &mut Game) {
    let turn = game.state.turn;
    for _ in 0..100 {
        if game.state.turn != turn {
            return;
        }
        let actions = game.legal_actions();
        let action = actions
            .iter()
            .find(|action| {
                matches!(
                    action,
                    Action::Pass | Action::FinishAttackers | Action::FinishBlockers
                )
            })
            .or_else(|| {
                actions
                    .iter()
                    .find(|action| !matches!(action, Action::Concede))
            })
            .unwrap()
            .clone();
        checked(game, action);
    }
    panic!("BIG turn failed to advance");
}
