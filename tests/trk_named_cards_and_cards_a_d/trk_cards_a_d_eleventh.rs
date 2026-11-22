//! Andromedan Automatons' optional named-card search.
use crate::support::*;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};

fn drain(game: &mut Game) {
    for _ in 0..48 {
        if game.state.pending == Pending::Priority
            && game.state.stack.is_empty()
            && game.state.pending_triggers.is_empty()
            && game.state.resolving.is_none()
        {
            return;
        }
        choose(game, |action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. })
        });
    }
    panic!("resolution budget");
}

#[test]
fn automatons_may_take_any_number_of_matching_cards_and_can_decline() {
    let mut game = scenario(Player::P0);
    let card = Card::from_name("Andromedan Automatons").unwrap();
    let names: Vec<_> = (0..3)
        .map(|_| spawn(&mut game, Player::P0, card, Zone::Library))
        .collect();
    let decoy = put(&mut game, Player::P0, "Forest", Zone::Library);
    let source = spawn(&mut game, Player::P0, card, Zone::Hand);
    cast(&mut game, source, None);
    for _ in 0..32 {
        if game
            .legal_actions()
            .contains(&Action::ChooseOptional { accept: true })
        {
            break;
        }
        choose(&mut game, |action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. })
        });
    }
    let checkpoint = game.snapshot();
    let mut declined = game.clone();
    act(&mut declined, Action::ChooseOptional { accept: false });
    drain(&mut declined);
    assert!(
        names
            .iter()
            .all(|&id| declined.object_view(id).zone == Zone::Library)
    );
    game.restore(&checkpoint).unwrap();
    act(&mut game, Action::ChooseOptional { accept: true });
    for &id in &names {
        act(&mut game, Action::ChooseSearch { card: id });
    }
    act(&mut game, Action::FinishSearch);
    drain(&mut game);
    assert!(
        names
            .iter()
            .all(|&id| game.object_view(id).zone == Zone::Hand)
    );
    assert_eq!(game.object_view(decoy).zone, Zone::Library);
}
