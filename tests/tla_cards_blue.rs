//! Avatar mono-blue definitions exercised through the normal game API.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

#[path = "tla_cards_blue/waterbend.rs"]
mod waterbend;

fn activate(card: CardId, ability: u16) -> Action {
    Action::AnnounceActivate {
        card,
        ability,
        counter_payment: Counters::default(),
    }
}

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn finish_targets(game: &mut Game) {
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(game, Action::FinishTargets);
    }
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

fn choose_waterbender(game: &mut Game, card: CardId) {
    let object = game.object_ref(card);
    act(game, Action::ChooseWaterbender { object });
}

fn finish_scry(game: &mut Game, player: Player, count: usize) {
    assert!(game.observe(player.other()).library_choice.is_none());
    let choices = game.observe(player).library_choice.unwrap().remaining;
    assert_eq!(choices.len(), count);
    for card in choices {
        act(
            game,
            Action::ArrangeLibrary {
                card: card.id,
                destination: LibraryDestination::Bottom,
            },
        );
    }
}

fn start_end_step(game: &mut Game, player: Player) {
    game.state.active = player;
    game.begin_step(Step::End);
    game.give_priority();
    game.prepare_triggers();
}
