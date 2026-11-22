//! Blastoneuron's entry exile and one-or-more graveyard-departure trigger.
use crate::support::*;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};

fn settle_ordered(game: &mut Game) {
    for _ in 0..48 {
        if game.state.stack.is_empty()
            && game.state.resolving.is_none()
            && game.state.pending == Pending::Priority
        {
            return;
        }
        match game.state.pending {
            Pending::OrderTriggers { .. } => {
                choose(game, |action| matches!(action, Action::PutTrigger { .. }));
            }
            Pending::Priority => act(game, Action::Pass),
            ref pending => panic!("unexpected Blastoneuron choice {pending:?}"),
        }
    }
    panic!("Blastoneuron resolution budget exhausted");
}

fn copied_tokens(game: &Game) -> Vec<CardId> {
    game.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(id, object)| {
            (object.zone == Zone::Battlefield
                && object.controller == Player::P0
                && object.is_token())
            .then_some(id as CardId)
        })
        .collect()
}

#[test]
fn entry_copies_only_a_cheap_own_creature_and_batch_departure_drains_once() {
    let mut game = scenario(Player::P0);
    let grave = put(&mut game, Player::P0, "Bear Cub", Zone::Graveyard);
    let large = put(&mut game, Player::P0, "Hill Giant", Zone::Graveyard);
    let foreign = put(&mut game, Player::P1, "Grizzly Bears", Zone::Graveyard);
    let source = put(&mut game, Player::P0, "Blastoneuron", Zone::Hand);
    cast(&mut game, source, None);
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    assert_eq!(game.object_view(source).zone, Zone::Battlefield);
    assert!(game.keywords(source).contains(Keywords::FLYING));
    for illegal in [large, foreign] {
        reject(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(illegal),
            },
        );
    }
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(grave),
        },
    );
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(&mut game, Action::FinishTargets);
    }
    settle_ordered(&mut game);
    assert_eq!(game.object_view(grave).zone, Zone::Exile);
    assert_eq!(game.state.players[0].life, 21);
    assert_eq!(game.state.players[1].life, 19);
    let created = copied_tokens(&game);
    assert_eq!(created.len(), 1);
    let token = created[0];
    assert_eq!(
        game.object_view(token).card,
        Card::from_name("Bear Cub").unwrap()
    );
    assert!(game.keywords(token).contains(Keywords::HASTE));
    assert_eq!(game.observe(Player::P0).delayed_sacrifices.len(), 1);

    let first = put(&mut game, Player::P0, "Forest", Zone::Graveyard);
    let second = put(&mut game, Player::P0, "Opt", Zone::Graveyard);
    let events = game.move_objects_raw(&[first, second], Zone::Exile);
    game.finish_zone_events(&events, &[]);
    game.prepare_triggers();
    settle_ordered(&mut game);
    assert_eq!(game.state.players[0].life, 22);
    assert_eq!(game.state.players[1].life, 18);

    game.begin_step(Step::End);
    game.prepare_triggers();
    settle_ordered(&mut game);
    assert_eq!(game.object_view(token).zone, Zone::Outside);
    assert_eq!(game.object_view(source).zone, Zone::Battlefield);
}
