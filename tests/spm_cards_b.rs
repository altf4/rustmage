//! Paid Spider-Man card scenarios for the second authored batch.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

#[path = "spm_cards_b/faces_and_combat.rs"]
mod faces_and_combat;
#[path = "spm_cards_b/mana_and_permanents.rs"]
mod mana_and_permanents;

fn announce_ability(game: &mut Game, card: CardId, ability: u16) {
    act(
        game,
        Action::AnnounceActivate {
            card,
            ability,
            counter_payment: Counters::default(),
        },
    );
}

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn choose_target(game: &mut Game, target: Target) {
    act(game, Action::ChooseTarget { target });
}

fn discard_with_meower(game: &mut Game, player: Player, card: CardId) {
    let source = put(game, player, "Masked Meower", Zone::Battlefield);
    announce_ability(game, source, 0);
    let object = game.object_ref(card);
    act(game, Action::ChooseAdditionalCost { object });
    pay(game);
    pass_pair(game);
    assert_eq!(game.object_view(card).zone, Zone::Graveyard);
}

fn advance_until(game: &mut Game, arrived: impl Fn(&Game) -> bool) {
    for _ in 0..96 {
        if arrived(game) {
            return;
        }
        let action = match game.state.pending {
            Pending::Priority => Action::Pass,
            Pending::Attackers => Action::FinishAttackers,
            Pending::Blockers => Action::FinishBlockers,
            _ => panic!("unexpected decision {:?}", game.state.pending),
        };
        act(game, action);
    }
    panic!("timed scenario did not reach its next step");
}

fn token_refs(game: &Game, card: Card, player: Player) -> Vec<ObjectRef> {
    game.state
        .objects
        .iter()
        .enumerate()
        .filter(|(_, object)| {
            object.card == card
                && object.controller == player
                && object.zone == Zone::Battlefield
                && object.is_token()
        })
        .map(|(id, _)| game.object_ref(id as CardId))
        .collect()
}
