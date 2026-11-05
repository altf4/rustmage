mod support;
use rustmage as _;
use rustmage_core::{cards::CounterKind, engine::*};
use support::*;

#[test]
fn the_phoenix_with_two_attack_requirements_gets_tomiks_one_slot() {
    let active = Player::P0;
    let opponent = active.other();
    let mut game = scenario(active);
    let ordinary = put(&mut game, active, "Flamewake Phoenix", Zone::Battlefield);
    let returned = put(&mut game, active, "Flamewake Phoenix", Zone::Graveyard);
    let carnage = put(&mut game, active, "Carnage, Crimson Chaos", Zone::Hand);
    put(&mut game, opponent, "Storm, Windrider", Zone::Battlefield);
    put(
        &mut game,
        opponent,
        "Tomik, Orzhov Lawmage",
        Zone::Battlefield,
    );
    let walker = put(
        &mut game,
        opponent,
        "Liliana, Dreadhorde General",
        Zone::Battlefield,
    );
    game.add_counter(walker, CounterKind::Loyalty, 6);
    cast(&mut game, carnage, None);
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(returned),
        },
    );
    settle(&mut game);
    assert_eq!(game.object_view(returned).zone, Zone::Battlefield);
    for _ in 0..12 {
        if game.state.pending == Pending::Attackers {
            break;
        }
        act(&mut game, Action::Pass);
    }
    assert_eq!(game.state.pending, Pending::Attackers);
    let defender = game.object_ref(walker);
    let ordinary_attack = Action::AttackPlaneswalker {
        card: ordinary,
        defender,
    };
    let returned_attack = Action::AttackPlaneswalker {
        card: returned,
        defender,
    };
    assert!(
        !game
            .legal_actions()
            .contains(&Action::Attack { card: ordinary })
    );
    assert!(
        !game
            .legal_actions()
            .contains(&Action::Attack { card: returned })
    );
    assert!(!game.legal_actions().contains(&Action::FinishAttackers));
    let mut control = game.clone();
    act(&mut control, returned_attack);
    assert!(control.legal_actions().contains(&Action::FinishAttackers));
    if game.legal_actions().contains(&ordinary_attack) {
        act(&mut game, ordinary_attack);
        reject(&mut game, Action::FinishAttackers);
    }
}

#[test]
fn weighted_matching_can_reassign_a_heavier_attacker_to_another_slot() {
    use rustmage_core::engine::combat_restrictions::maximum_attack_matching;
    let mut candidates = [(2, vec![0]), (1, vec![1]), (3, vec![0, 1])];
    assert_eq!(maximum_attack_matching(&mut candidates, 2), 5);
}
