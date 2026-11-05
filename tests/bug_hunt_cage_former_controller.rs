mod support;

use rustmage as _;
use rustmage_core::engine::*;
use support::*;

#[test]
fn collector_cage_hides_exiled_card_from_player_who_only_controlled_it_before_exile() {
    let p0 = Player::P0;
    let p1 = Player::P1;
    let mut game = scenario(p0);
    for p in [p0, p1] {
        put(&mut game, p, "High Fae Trickster", Zone::Battlefield);
    }
    let cage = put(&mut game, p0, "Collector's Cage", Zone::Hand);
    let offer = put(&mut game, p0, "Harmless Offering", Zone::Hand);
    let return_offer = put(&mut game, p1, "Harmless Offering", Zone::Hand);
    let hidden = put(&mut game, p0, "Bear Cub", Zone::Library);

    let passes = |game: &mut Game| {
        act(game, Action::Pass);
        act(game, Action::Pass);
    };
    cast(&mut game, cage, None);
    passes(&mut game);
    assert_eq!(game.state.stack.len(), 1); // Hideaway is still on the stack.
    announce(&mut game, offer, &[Target::Player(p1), Target::Card(cage)]);
    passes(&mut game);
    assert_eq!(game.object_view(cage).controller, p1);
    assert_eq!(game.object_view(hidden).zone, Zone::Library);

    act(&mut game, Action::Pass);
    assert_eq!(game.actor(), Some(p1));
    announce(
        &mut game,
        return_offer,
        &[Target::Player(p0), Target::Card(cage)],
    );
    passes(&mut game);
    assert_eq!(game.object_view(cage).controller, p0);
    assert_eq!(game.object_view(hidden).zone, Zone::Library);

    passes(&mut game);
    act(&mut game, Action::SelectLibrary { card: hidden });
    if game
        .legal_actions()
        .contains(&Action::FinishLibrarySelection)
    {
        act(&mut game, Action::FinishLibrarySelection);
    }
    settle(&mut game);

    let can_see = |game: &Game, p| {
        game.observe(p)
            .objects
            .iter()
            .any(|o| o.id == hidden && o.zone == Zone::Exile)
    };
    assert_eq!(game.object_view(hidden).zone, Zone::Exile);
    assert!(can_see(&game, p0));
    assert!(
        !can_see(&game, p1),
        "the former controller sees Bear Cub in face-down exile"
    );

    let later_offer = put(&mut game, p0, "Harmless Offering", Zone::Hand);
    announce(
        &mut game,
        later_offer,
        &[Target::Player(p1), Target::Card(cage)],
    );
    passes(&mut game);
    assert!(can_see(&game, p1));
    let later_return = put(&mut game, p1, "Harmless Offering", Zone::Hand);
    act(&mut game, Action::Pass);
    announce(
        &mut game,
        later_return,
        &[Target::Player(p0), Target::Card(cage)],
    );
    passes(&mut game);
    assert_eq!(game.object_view(cage).controller, p0);
    assert!(
        can_see(&game, p1),
        "look permission persists after control ends"
    );
}
