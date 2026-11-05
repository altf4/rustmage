mod support;

use rustmage as _;
use rustmage_core::engine::*;
use support::*;

#[test]
fn grim_javelineer_tracks_the_watched_permanent_after_it_stops_being_a_creature() {
    let p = Player::P0;
    let mut game = scenario(p);
    put(&mut game, p, "Grim Javelineer", Zone::Battlefield);
    let cage = put(&mut game, p, "Collector's Cage", Zone::Battlefield);
    let animate = put(&mut game, p, "Puppet Crafting", Zone::Hand);
    let remove_aura = put(&mut game, p, "Disenchant", Zone::Hand);
    let remove_cage = put(&mut game, p, "Disenchant", Zone::Hand);
    cast(&mut game, animate, Some(Target::Card(cage)));
    settle(&mut game);
    assert_eq!(game.object_view(cage).kinds, 17);
    for _ in 0..16 {
        if game.state.pending == Pending::Attackers {
            break;
        }
        act(&mut game, Action::Pass);
    }
    act(&mut game, Action::Attack { card: cage });
    act(&mut game, Action::FinishAttackers);
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(cage),
        },
    );
    settle(&mut game);

    for _ in 0..18 {
        if game.state.step == Step::Main2 {
            break;
        }
        choose(&mut game, |a| {
            matches!(a, Action::Pass | Action::FinishBlockers)
        });
    }
    assert_eq!(game.state.step, Step::Main2);
    game.state.players[p.index()].mana = [4, 0, 0, 0, 0, 4];
    let mut still_creature = game.clone();
    cast(&mut still_creature, remove_cage, Some(Target::Card(cage)));
    act(&mut still_creature, Action::Pass);
    act(&mut still_creature, Action::Pass);
    assert_eq!(still_creature.object_view(cage).zone, Zone::Graveyard);
    assert_eq!(still_creature.state.stack.len(), 1);

    cast(&mut game, remove_aura, Some(Target::Card(animate)));
    settle(&mut game);
    assert_eq!(game.object_view(cage).kinds, 16);
    cast(&mut game, remove_cage, Some(Target::Card(cage)));
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    assert_eq!(game.object_view(cage).zone, Zone::Graveyard);
    assert_eq!(
        game.state.stack.len(),
        1,
        "Grim Javelineer's delayed surveil should trigger on the exact permanent's death"
    );
}
