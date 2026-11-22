//! Star Trek equipment and ward scenarios from direct typed card definitions.
use crate::support::*;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};

fn owned(game: &mut Game, player: Player, name: &str, zone: Zone) -> CardId {
    spawn(game, player, Card::from_name(name).unwrap(), zone)
}

fn drain(game: &mut Game, target: Option<CardId>) {
    for _ in 0..48 {
        if game.state.pending == Pending::Priority
            && game.state.stack.is_empty()
            && game.state.pending_triggers.is_empty()
            && game.state.resolving.is_none()
        {
            return;
        }
        let legal = game.legal_actions();
        if let Some(id) = target {
            let choice = Action::ChooseTarget {
                target: Target::Card(id),
            };
            if legal.contains(&choice) {
                act(game, choice);
                continue;
            }
        }
        if legal.contains(&Action::FinishTargets) {
            act(game, Action::FinishTargets);
            continue;
        }
        choose(game, |a| {
            matches!(a, Action::Pass | Action::PutTrigger { .. })
        });
    }
    panic!("resolution budget");
}

#[test]
fn batleth_shrinks_opposing_creature_then_equips_for_power() {
    let mut game = scenario(Player::P0);
    let opponent = put(&mut game, Player::P1, "Hill Giant", Zone::Battlefield);
    let bearer = put(&mut game, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let equipment = owned(&mut game, Player::P0, "Bat'leth", Zone::Hand);
    cast(&mut game, equipment, None);
    drain(&mut game, Some(opponent));
    assert_eq!(stats(&game, opponent), (1, 1));
    assert_eq!(stats(&game, bearer), (2, 2));

    act(
        &mut game,
        Action::AnnounceActivate {
            card: equipment,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(bearer),
        },
    );
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(&mut game, Action::FinishTargets);
    }
    pay(&mut game);
    drain(&mut game, None);
    assert_eq!(stats(&game, bearer), (4, 2));
}

#[test]
fn cetacean_ops_grows_only_from_own_noncreature_cast_and_ward_charges_opponent() {
    let mut game = scenario(Player::P0);
    let whale = owned(&mut game, Player::P0, "Cetacean Ops", Zone::Battlefield);
    let shock = put(&mut game, Player::P0, "Shock", Zone::Hand);
    cast(&mut game, shock, Some(Target::Player(Player::P1)));
    drain(&mut game, None);
    assert_eq!(game.counter_count(whale, CounterKind::PlusOne), 1);
    assert_eq!(stats(&game, whale), (4, 4));

    let bear = put(&mut game, Player::P0, "Grizzly Bears", Zone::Hand);
    cast(&mut game, bear, None);
    drain(&mut game, None);
    assert_eq!(game.counter_count(whale, CounterKind::PlusOne), 1);

    act(&mut game, Action::Pass);
    let unsummon = put(&mut game, Player::P1, "Unsummon", Zone::Hand);
    cast(&mut game, unsummon, Some(Target::Card(whale)));
    for _ in 0..32 {
        if matches!(game.state.pending, Pending::ResolutionMana { .. }) {
            break;
        }
        choose(&mut game, |a| {
            matches!(a, Action::Pass | Action::PutTrigger { .. })
        });
    }
    assert!(matches!(game.state.pending, Pending::ResolutionMana { .. }));
    act(&mut game, Action::DeclineResolutionMana);
    drain(&mut game, None);
    assert_eq!(game.object_view(whale).zone, Zone::Battlefield);
    assert_eq!(game.object_view(unsummon).zone, Zone::Graveyard);
}
