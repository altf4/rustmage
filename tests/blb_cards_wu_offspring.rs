mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

fn play_offspring(game: &mut Game, card: CardId, paid: bool) {
    act(game, Action::AnnounceCast { card });
    act(game, Action::ChooseOffspring { pay: paid });
    pay(game);
}

fn progress(game: &mut Game, recipient: Option<CardId>) {
    if let Some(card) = recipient {
        let action = target(card);
        if game.legal_actions().contains(&action) {
            act(game, action);
            return;
        }
    }
    choose(game, |action| {
        matches!(
            action,
            Action::PutTrigger { .. }
                | Action::Pass
                | Action::FinishTargets
                | Action::FinishStackTargets
        )
    });
}

fn settle_triggers(game: &mut Game, recipient: Option<CardId>) {
    for _ in 0..32 {
        if game.state.stack.is_empty()
            && game.state.pending_triggers.is_empty()
            && game.state.ordering_triggers.is_empty()
            && game.state.resolving.is_none()
            && game.state.pending == Pending::Priority
        {
            return;
        }
        progress(game, recipient);
    }
    panic!("offspring triggers did not settle");
}

fn copy_of(game: &Game, definition: Card) -> Option<CardId> {
    game.state
        .objects
        .iter()
        .enumerate()
        .find_map(|(id, object)| {
            (object.zone == Zone::Battlefield
                && object.is_copy_token()
                && object.card == definition)
                .then_some(id as CardId)
        })
}

#[test]
fn offspring_definitions_each_make_an_exact_copy_with_their_own_ability() {
    let player = Player::P0;
    for name in ["Intrepid Rabbit", "Finch Formation", "Splash Lasher"] {
        let mut game = scenario(player);
        let recipient = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
        let parent = put(&mut game, player, name, Zone::Hand);
        let definition = game.object_view(parent).card;
        let mut declined = game.clone();
        play_offspring(&mut declined, parent, false);
        settle_triggers(&mut declined, Some(recipient));
        assert_eq!(copy_of(&declined, definition), None);
        play_offspring(&mut game, parent, true);
        settle_triggers(&mut game, Some(recipient));
        let child = copy_of(&game, definition).unwrap_or_else(|| panic!("missing copy of {name}"));
        assert_eq!(stats(&game, child), (1, 1), "{name}");
        match name {
            "Intrepid Rabbit" => {
                assert_eq!(stats(&game, recipient), (4, 4));
                assert_eq!(stats(&declined, recipient), (3, 3));
            }
            "Finch Formation" => {
                assert!(game.object_view(child).keywords.contains(Keywords::FLYING));
                assert!(
                    game.object_view(recipient)
                        .keywords
                        .contains(Keywords::FLYING)
                );
            }
            "Splash Lasher" => {
                assert!(game.object_view(recipient).tapped);
                assert_eq!(game.counter_count(recipient, CounterKind::Stun), 2);
                assert_eq!(declined.counter_count(recipient, CounterKind::Stun), 1);
            }
            _ => unreachable!(),
        }
    }
}

#[test]
fn thundertrap_trainer_offspring_repeats_the_private_revealed_pick() {
    let player = Player::P1;
    let mut game = scenario(player);
    let mut opt_ids = Vec::new();
    for _ in 0..2 {
        opt_ids.push(put(&mut game, player, "Opt", Zone::Library));
        for _ in 0..3 {
            put(&mut game, player, "Forest", Zone::Library);
        }
    }
    let parent = put(&mut game, player, "Thundertrap Trainer", Zone::Hand);
    let definition = game.object_view(parent).card;
    play_offspring(&mut game, parent, true);
    for opt in opt_ids.into_iter().rev() {
        for _ in 0..24 {
            if game.observe(player).library_selection.is_some() {
                break;
            }
            progress(&mut game, None);
        }
        let look = game.observe(player).library_selection.unwrap();
        assert_eq!(look.remaining.len(), 4);
        assert_eq!(
            look.candidates
                .iter()
                .map(|candidate| candidate.card)
                .collect::<Vec<_>>(),
            [opt]
        );
        assert!(game.observe(player.other()).library_selection.is_none());
        act(&mut game, Action::SelectLibrary { card: opt });
        act(&mut game, Action::FinishLibrarySelection);
        assert_eq!(game.object_view(opt).zone, Zone::Hand);
    }
    settle_triggers(&mut game, None);
    let child = copy_of(&game, definition).unwrap();
    assert_eq!(stats(&game, child), (1, 1));
}

fn shore_up(game: &mut Game, player: Player, card: CardId) {
    if game.actor() != Some(player) {
        act(game, Action::Pass);
    }
    let spell = put(game, player, "Shore Up", Zone::Hand);
    cast(game, spell, Some(Target::Card(card)));
}

#[test]
fn both_flowerfoot_incarnations_trigger_only_on_their_first_friendly_target() {
    let player = Player::P0;
    let mut game = scenario(player);
    let parent = put(&mut game, player, "Flowerfoot Swordmaster", Zone::Hand);
    let definition = game.object_view(parent).card;
    play_offspring(&mut game, parent, true);
    settle_triggers(&mut game, None);
    let child = copy_of(&game, definition).unwrap();
    assert_eq!(stats(&game, child), (1, 1));
    shore_up(&mut game, player, parent);
    settle_triggers(&mut game, None);
    assert_eq!(stats(&game, child), (2, 1));
    shore_up(&mut game, player, parent);
    settle_triggers(&mut game, None);
    assert_eq!(stats(&game, child), (2, 1));
    shore_up(&mut game, player, child);
    settle_triggers(&mut game, None);
    assert_eq!(stats(&game, parent), (5, 4));
    assert_eq!(stats(&game, child), (4, 2));
}

#[test]
fn mouse_trapper_flash_and_nettle_guards_independent_sacrifice_are_live() {
    let player = Player::P1;
    let other = player.other();
    let mut game = scenario(other);
    let recipient = put(&mut game, other, "Grizzly Bears", Zone::Battlefield);
    let trapper = put(&mut game, player, "Mouse Trapper", Zone::Hand);
    act(&mut game, Action::Pass);
    cast(&mut game, trapper, None);
    settle_triggers(&mut game, None);
    assert_eq!(game.object_view(trapper).zone, Zone::Battlefield);
    shore_up(&mut game, player, trapper);
    settle_triggers(&mut game, Some(recipient));
    assert!(game.object_view(recipient).tapped);
    game.state.objects[recipient as usize].tapped = false;
    shore_up(&mut game, player, trapper);
    settle_triggers(&mut game, None);
    assert!(!game.object_view(recipient).tapped);

    let guard = put(&mut game, player, "Nettle Guard", Zone::Battlefield);
    let artifact = put(&mut game, other, "Candy Trail", Zone::Battlefield);
    shore_up(&mut game, player, guard);
    settle_triggers(&mut game, None);
    assert_eq!(stats(&game, guard), (4, 4));
    if game.actor() != Some(player) {
        act(&mut game, Action::Pass);
    }
    act(
        &mut game,
        Action::AnnounceActivate {
            card: guard,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    act(&mut game, target(artifact));
    pay(&mut game);
    assert_eq!(game.object_view(guard).zone, Zone::Graveyard);
    settle_triggers(&mut game, None);
    assert_eq!(game.object_view(artifact).zone, Zone::Graveyard);
}
