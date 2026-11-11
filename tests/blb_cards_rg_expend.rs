//! Bloomburrow card-level Expend outcomes and the source's paid Stash receipt.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn settle_expend(game: &mut Game, recipient: CardId, discard: bool) {
    for _ in 0..48 {
        if game.state.pending == Pending::Priority
            && game.state.stack.is_empty()
            && game.state.pending_triggers.is_empty()
            && game.state.resolving.is_none()
        {
            return;
        }
        choose(game, |action| match *action {
            Action::ChooseTarget { target } => target == Target::Card(recipient),
            Action::ChooseOptional { accept } => accept == discard,
            Action::PutTrigger { .. } | Action::Pass => true,
            _ => false,
        });
    }
    panic!("Expend triggers did not settle");
}

#[test]
fn expending_four_resolves_each_printed_reward_and_only_crosses_once() {
    for player in [Player::P0, Player::P1] {
        let opponent = player.other();
        let mut game = scenario(player);
        let byway = put(&mut game, player, "Byway Barterer", Zone::Battlefield);
        let roughshod = put(&mut game, player, "Roughshod Duo", Zone::Battlefield);
        let slinger = put(&mut game, player, "Teapot Slinger", Zone::Battlefield);
        let baker = put(&mut game, player, "Bakersbane Duo", Zone::Battlefield);
        let boxer = put(&mut game, player, "Bark-Knuckle Boxer", Zone::Battlefield);
        let veteran = put(&mut game, player, "Brambleguard Veteran", Zone::Battlefield);
        let raccoon = put(&mut game, player, "Raccoon Rallier", Zone::Battlefield);
        let foreign = put(&mut game, opponent, "Raccoon Rallier", Zone::Battlefield);
        let recipient = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
        let overflow = put(&mut game, player, "Hoarder's Overflow", Zone::Hand);

        assert!(game.keywords(byway).contains(Keywords::MENACE));
        assert!(game.keywords(roughshod).contains(Keywords::TRAMPLE));
        assert!(game.keywords(slinger).contains(Keywords::MENACE));
        cast(&mut game, overflow, None);
        settle(&mut game);
        assert_eq!(game.counter_count(overflow, CounterKind::Stash), 1);

        let spell = put(&mut game, player, "Hill Giant", Zone::Hand);
        let extra = put(&mut game, player, "Forest", Zone::Hand);
        cast(&mut game, spell, None);
        let mut declined = game.clone();
        settle_expend(&mut declined, recipient, false);
        assert_eq!(declined.object_view(extra).zone, Zone::Hand);
        assert_eq!(
            declined.state.players[player.index()].cards_drawn_this_turn,
            0
        );

        settle_expend(&mut game, recipient, true);
        assert_eq!(game.object_view(extra).zone, Zone::Graveyard);
        assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 2);
        assert_eq!(game.state.players[opponent.index()].life, 18);
        assert_eq!(game.counter_count(overflow, CounterKind::Stash), 2);
        assert_eq!(stats(&game, recipient), (3, 3));
        assert!(game.keywords(recipient).contains(Keywords::TRAMPLE));
        assert_eq!(stats(&game, baker), (4, 4));
        assert!(game.keywords(baker).contains(Keywords::VIGILANCE));
        assert!(game.keywords(boxer).contains(Keywords::INDESTRUCTIBLE));
        assert_eq!(stats(&game, raccoon), (3, 3));
        assert!(game.keywords(raccoon).contains(Keywords::VIGILANCE));
        assert_eq!(stats(&game, veteran), (4, 5));
        assert_eq!(stats(&game, foreign), (2, 2));
        assert!(!game.keywords(foreign).contains(Keywords::VIGILANCE));

        game.state.players[player.index()].mana = [8; 6];
        let another = put(&mut game, player, "Hill Giant", Zone::Hand);
        cast(&mut game, another, None);
        settle(&mut game);
        assert_eq!(game.state.players[opponent.index()].life, 18);
        assert_eq!(game.counter_count(overflow, CounterKind::Stash), 2);

        choose(&mut game, |action| match *action {
            Action::Activate {
                card, ability: 0, ..
            }
            | Action::AnnounceActivate {
                card, ability: 0, ..
            } => card == overflow,
            _ => false,
        });
        if matches!(game.state.pending, Pending::PayCosts { .. }) {
            pay(&mut game);
        }
        assert_eq!(game.object_view(overflow).zone, Zone::Graveyard);
        settle(&mut game);
        assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 4);

        game.finish_cleanup();
        assert_eq!(stats(&game, recipient), (2, 2));
        assert!(!game.keywords(recipient).contains(Keywords::TRAMPLE));
        assert!(!game.keywords(boxer).contains(Keywords::INDESTRUCTIBLE));
        assert!(!game.keywords(raccoon).contains(Keywords::VIGILANCE));
    }
}

#[test]
fn bakersbane_duos_entry_creates_an_untapped_food() {
    let player = Player::P0;
    let mut game = scenario(player);
    let baker = put(&mut game, player, "Bakersbane Duo", Zone::Hand);
    cast(&mut game, baker, None);
    settle(&mut game);
    let foods: Vec<_> = game
        .observe(player)
        .objects
        .into_iter()
        .filter(|object| {
            object.zone == Zone::Battlefield
                && object.controller == player
                && object.is_token()
                && object.has_subtype("Food")
        })
        .collect();
    assert_eq!(foods.len(), 1);
    assert!(!foods[0].tapped);
}
