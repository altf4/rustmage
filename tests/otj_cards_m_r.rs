//! Actual Outlaws definitions and their interactions through owned game actions.
mod support;

use rustmage::Card;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

#[path = "otj_cards_m_r/combat.rs"]
mod combat;
#[path = "otj_cards_m_r/crimes.rs"]
mod crimes;
#[path = "otj_cards_m_r/entries.rs"]
mod entries;
#[path = "otj_cards_m_r/inherited.rs"]
mod inherited;
#[path = "otj_cards_m_r/libraries.rs"]
mod libraries;
#[path = "otj_cards_m_r/modes.rs"]
mod modes;

const TREASURE: u16 = 5302;
const ZOMBIE_ROGUE: u16 = 5305;
const MERCENARY: u16 = 5612;
const SCORPION_DRAGON: u16 = 5615;
const BLUE_BIRD: u16 = 5616;
const METEORITE: u16 = 5617;
const SPIRIT: u16 = 5618;

fn reach(game: &mut Game, checkpoint: impl Fn(&Game) -> bool) {
    for _ in 0..96 {
        if checkpoint(game) {
            return;
        }
        match game.state.pending {
            Pending::Priority => act(game, Action::Pass),
            Pending::OrderTriggers { .. } => {
                choose(game, |action| matches!(action, Action::PutTrigger { .. }))
            }
            ref other => panic!("unexpected checkpoint state {other:?}"),
        }
    }
    panic!("scenario did not reach its checkpoint");
}

fn quiet(game: &mut Game) {
    reach(game, |g| {
        g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.resolving.is_none()
    });
}

fn has(game: &Game, action: &Action) -> bool {
    game.legal_actions().contains(action)
}

fn until_action(game: &mut Game, action: &Action) {
    reach(game, |g| has(g, action));
}

fn board(game: &mut Game, player: Player, name: &str) -> CardId {
    put(game, player, name, Zone::Battlefield)
}

fn hand(game: &mut Game, player: Player, name: &str) -> CardId {
    put(game, player, name, Zone::Hand)
}

fn graveyard(game: &mut Game, player: Player, name: &str) -> CardId {
    put(game, player, name, Zone::Graveyard)
}

fn top(game: &mut Game, player: Player, name: &str) -> CardId {
    put(game, player, name, Zone::Library)
}

fn tokens(game: &Game, player: Player, catalogue_id: u16) -> Vec<CardId> {
    let card = Card::from_catalogue_id(catalogue_id);
    game.state
        .objects
        .iter()
        .enumerate()
        .filter(|(_, object)| {
            object.card == card && object.controller == player && object.zone == Zone::Battlefield
        })
        .map(|(index, _)| index as CardId)
        .collect()
}

fn zone(game: &Game, card: CardId) -> Zone {
    game.object_view(card).zone
}

fn keywords(game: &Game, card: CardId) -> Keywords {
    game.object_view(card).keywords
}

fn counters(game: &Game, card: CardId) -> u64 {
    game.object_view(card).counters.plus_one
}

fn life(game: &Game, player: Player) -> i128 {
    game.state.players[player.index()].life
}

fn announce_activation(game: &mut Game, source: CardId, ability: u16) {
    act(
        game,
        Action::AnnounceActivate {
            card: source,
            ability,
            counter_payment: Counters::default(),
        },
    );
}

fn bind_targets(game: &mut Game, targets: &[Target]) {
    for &target in targets {
        act(game, Action::ChooseTarget { target });
    }
    if has(game, &Action::FinishTargets) {
        act(game, Action::FinishTargets);
    }
}

fn modal(game: &mut Game, spell: CardId, mode: u16, targets: &[Target]) {
    act(game, Action::AnnounceCast { card: spell });
    act(game, Action::ChooseMode { mode });
    bind_targets(game, targets);
    pay(game);
}

fn activate(game: &mut Game, source: CardId, ability: u16, targets: &[Target]) {
    announce_activation(game, source, ability);
    bind_targets(game, targets);
    pay(game);
}

fn finish_additional_cost(game: &mut Game, cards: &[CardId]) {
    for &card in cards {
        let object = game.object_ref(card);
        act(game, Action::ChooseAdditionalCost { object });
    }
    if has(game, &Action::FinishAdditionalCostGroup) {
        act(game, Action::FinishAdditionalCostGroup);
    }
    pay(game);
}

fn choose_library(game: &mut Game, card: CardId) {
    act(game, Action::SelectLibrary { card });
    if has(game, &Action::FinishLibrarySelection) {
        act(game, Action::FinishLibrarySelection);
    }
}

fn arrange(game: &mut Game, card: CardId, destination: LibraryDestination) {
    act(game, Action::ArrangeLibrary { card, destination });
}

fn restock_mana(game: &mut Game, player: Player) {
    game.state.players[player.index()].mana = [8; 6];
}
