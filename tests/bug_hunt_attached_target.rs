mod support;
use rustmage as _;
use rustmage_core::{cards::Keywords, engine::*};
use support::*;

#[test]
fn fiery_annihilation_cannot_exile_equipment_when_its_creature_target_gains_hexproof() {
    let p = Player::P0;
    let mut game = scenario(p);
    let creature = put(&mut game, p.other(), "Bear Cub", Zone::Battlefield);
    let equipment = put(&mut game, p.other(), "Basilisk Collar", Zone::Battlefield);
    let creature_ref = game.object_ref(creature);
    assert!(game.attach_equipment(game.object_ref(equipment), creature_ref));
    let fiery = put(&mut game, p, "Fiery Annihilation", Zone::Hand);
    let shore_up = put(&mut game, p.other(), "Shore Up", Zone::Hand);
    announce(
        &mut game,
        fiery,
        &[Target::Card(creature), Target::Card(equipment)],
    );
    assert_eq!(game.state.stack[0].targets.len(), 2);
    let mut both_legal = game.clone();
    settle(&mut both_legal);
    assert_eq!(both_legal.object_view(equipment).zone, Zone::Exile);
    assert_eq!(both_legal.object_view(creature).zone, Zone::Exile);

    act(&mut game, Action::Pass);
    assert_eq!(game.actor(), Some(p.other()));
    cast(&mut game, shore_up, Some(Target::Card(creature)));
    settle(&mut game);
    assert_eq!(game.object_view(fiery).zone, Zone::Graveyard);
    assert_eq!(game.object_view(creature).zone, Zone::Battlefield);
    assert_eq!(game.object_view(creature).damage, 0);
    assert!(
        game.object_view(creature)
            .keywords
            .contains(Keywords::HEXPROOF)
    );
    assert_eq!(
        game.object_view(equipment).zone,
        Zone::Battlefield,
        "the Equipment target depends on a legal first target"
    );
    assert_eq!(game.object_view(equipment).attached_to, Some(creature_ref));
}
