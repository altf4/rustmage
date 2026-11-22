//! Dr. Crusher life-gain triggers and Death Is Irrelevant assimilation.
use crate::support::*;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};

fn owned(game: &mut Game, name: &str, zone: Zone) -> CardId {
    spawn(game, Player::P0, Card::from_name(name).unwrap(), zone)
}
fn drain(game: &mut Game) {
    for _ in 0..60 {
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
fn dr_beverly_crusher_doubles_only_life_gain_triggers_from_owned_permanents() {
    let mut game = scenario(Player::P0);
    let crusher = owned(&mut game, "Dr. Beverly Crusher", Zone::Battlefield);
    let neelix = owned(&mut game, "Neelix, Moral Support", Zone::Battlefield);
    let land = owned(&mut game, "Bajor", Zone::Hand);
    act(&mut game, Action::PlayLand { card: land });
    drain(&mut game);
    assert_eq!(game.state.players[0].life, 21);
    assert_eq!(game.counter_count(crusher, CounterKind::PlusOne), 2);
    assert_eq!(game.counter_count(neelix, CounterKind::PlusOne), 2);
}

#[test]
fn death_is_irrelevant_grants_only_artifact_creatures_and_assimilates_grave_card() {
    let mut game = scenario(Player::P0);
    let _source = owned(&mut game, "Death Is Irrelevant", Zone::Battlefield);
    let borg = spawn(&mut game, Player::P0, Card(6007), Zone::Battlefield);
    let plain = put(&mut game, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let enemy = spawn(&mut game, Player::P1, Card(6007), Zone::Battlefield);
    assert!(game.keywords(borg).contains(Keywords::INDESTRUCTIBLE));
    assert!(!game.keywords(plain).contains(Keywords::INDESTRUCTIBLE));
    assert!(!game.keywords(enemy).contains(Keywords::INDESTRUCTIBLE));
    let grave = put(&mut game, Player::P1, "Bear Cub", Zone::Graveyard);
    game.state.step = Step::Main2;
    game.give_priority();
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    for _ in 0..32 {
        if game.legal_actions().contains(&Action::ChooseTarget {
            target: Target::Card(grave),
        }) {
            break;
        }
        choose(&mut game, |action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. })
        });
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
    drain(&mut game);
    assert_eq!(game.object_view(grave).zone, Zone::Battlefield);
    assert_eq!(game.object_view(grave).controller, Player::P0);
    assert_eq!(game.counter_count(grave, CounterKind::PlusOne), 1);
    assert!(game.has_subtype(grave, "Borg"));
    assert!(game.keywords(grave).contains(Keywords::INDESTRUCTIBLE));
}
