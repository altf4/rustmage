mod support;

use rustmage as _;
use rustmage_core::engine::*;
use support::*;

#[test]
fn ancient_cornucopia_tracks_each_controllers_choice() {
    let mut game = scenario(Player::P0);
    let cornucopia = put(
        &mut game,
        Player::P0,
        "Ancient Cornucopia",
        Zone::Battlefield,
    );
    let offering = put(&mut game, Player::P0, "Harmless Offering", Zone::Hand);
    let opt = put(&mut game, Player::P1, "Opt", Zone::Hand);
    put(
        &mut game,
        Player::P1,
        "High Fae Trickster",
        Zone::Battlefield,
    );

    announce(
        &mut game,
        offering,
        &[Target::Player(Player::P1), Target::Card(cornucopia)],
    );
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    assert!(matches!(
        game.state.pending,
        Pending::EffectOptional {
            player: Player::P0,
            ..
        }
    ));
    act(&mut game, Action::ChooseOptional { accept: true });
    settle(&mut game);
    assert_eq!(game.state.players[Player::P0.index()].life, 21);
    assert_eq!(
        game.state.objects[cornucopia as usize].controller,
        Player::P1
    );
    assert_eq!(game.state.turn, 3);

    act(&mut game, Action::Pass);
    assert_eq!(game.actor(), Some(Player::P1));
    cast(&mut game, opt, None);
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    assert!(
        matches!(
            game.state.pending,
            Pending::EffectOptional {
                player: Player::P1,
                ..
            }
        ),
        "expected a life-gain choice before Opt, got {:?}",
        game.state.pending
    );
    act(&mut game, Action::ChooseOptional { accept: true });
    assert_eq!(game.state.players[Player::P1.index()].life, 21);

    let shock = put(&mut game, Player::P1, "Shock", Zone::Hand);
    assert_eq!(game.actor(), Some(Player::P0));
    act(&mut game, Action::Pass);
    cast(&mut game, shock, Some(Target::Player(Player::P0)));
    assert_eq!(game.state.stack.last().unwrap().source.unwrap().id, shock);
    assert!(game.state.pending_triggers.is_empty());

    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    let back = put(&mut game, Player::P1, "Harmless Offering", Zone::Hand);
    announce(
        &mut game,
        back,
        &[Target::Player(Player::P0), Target::Card(cornucopia)],
    );
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    assert_eq!(game.object_view(cornucopia).controller, Player::P0);
    let bolt = put(&mut game, Player::P0, "Lightning Bolt", Zone::Hand);
    cast(&mut game, bolt, Some(Target::Player(Player::P1)));
    assert_eq!(game.state.stack.last().unwrap().source.unwrap().id, bolt);
    assert!(game.state.pending_triggers.is_empty());
}
