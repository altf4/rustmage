//! Explorer's Cache moves counters and observes the dying creature's exact counter kind.
mod support;
use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

#[test]
fn explorers_cache_moves_one_counter_at_sorcery_speed_and_replaces_it_after_the_creature_dies() {
    let p = Player::P0;
    let mut game = scenario(p);
    let source = spawn(
        &mut game,
        p,
        Card::from_name("Explorer's Cache").unwrap(),
        Zone::Hand,
    );
    let recipient = put(&mut game, p, "Grizzly Bears", Zone::Battlefield);
    let other_counter = put(&mut game, p, "Bear Cub", Zone::Battlefield);
    let opposing = put(&mut game, p.other(), "Grizzly Bears", Zone::Battlefield);
    let land = put(&mut game, p, "Forest", Zone::Battlefield);
    let sweep = put(&mut game, p, "Day of Judgment", Zone::Hand);
    game.add_counter(other_counter, CounterKind::Stun, 1);
    game.add_counter(opposing, CounterKind::PlusOne, 1);
    cast(&mut game, source, None);
    settle(&mut game);
    assert_eq!(game.counter_count(source, CounterKind::PlusOne), 2);

    let activation = Action::AnnounceActivate {
        card: source,
        ability: 0,
        counter_payment: Counters::default(),
    };
    game.state.step = Step::Upkeep;
    reject(&mut game, activation.clone());
    game.state.step = Step::Main1;
    act(&mut game, activation);
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(land),
        },
    );
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(recipient),
        },
    );
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(&mut game);
    }
    assert!(game.object_view(source).tapped);
    let mut stale = game.clone();
    stale.move_object(recipient, Zone::Hand);
    stale.move_object(recipient, Zone::Battlefield);
    settle(&mut stale);
    assert_eq!(stale.counter_count(source, CounterKind::PlusOne), 2);
    assert_eq!(stale.counter_count(recipient, CounterKind::PlusOne), 0);
    settle(&mut game);
    assert_eq!(game.counter_count(source, CounterKind::PlusOne), 1);
    assert_eq!(game.counter_count(recipient, CounterKind::PlusOne), 1);

    game.state.players[p.index()].mana = [2, 0, 0, 0, 0, 2];
    cast(&mut game, sweep, None);
    settle(&mut game);
    for card in [recipient, other_counter, opposing] {
        assert_eq!(game.object_view(card).zone, Zone::Graveyard);
    }
    assert_eq!(game.counter_count(source, CounterKind::PlusOne), 2);
}
