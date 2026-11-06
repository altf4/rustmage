mod support;

use rustmage as _;
use rustmage_core::engine::*;
use support::*;

#[test]
fn progenitus_protection_prevents_pyroclasm_without_asking() {
    let mut game = scenario(Player::P0);
    let progenitus = put(&mut game, Player::P1, "Progenitus", Zone::Battlefield);
    let pyroclasm = put(&mut game, Player::P0, "Pyroclasm", Zone::Hand);
    cast(&mut game, pyroclasm, None);
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    assert_eq!(
        game.state.pending,
        Pending::Priority,
        "protection is the only applicable effect: {:?}",
        game.legal_actions()
    );
    assert_eq!(
        game.state.objects[progenitus as usize].zone,
        Zone::Battlefield
    );
    assert_eq!(game.state.objects[progenitus as usize].damage, 0);
    assert_eq!(game.state.objects[pyroclasm as usize].zone, Zone::Graveyard);
}
