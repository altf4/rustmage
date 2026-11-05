mod support;
use rustmage as _;
use rustmage_core::engine::*;
use support::*;

#[test]
fn active_player_has_the_first_simultaneous_death_replacement_choice() {
    let active = Player::P1;
    let nonactive = active.other();
    let mut game = scenario(active);
    let nonactive_head = put(&mut game, nonactive, "Head of the Hunt", Zone::Battlefield);
    let active_head = put(&mut game, active, "Head of the Hunt", Zone::Battlefield);
    assert!(nonactive_head < active_head);
    put(&mut game, active, "Rest in Peace", Zone::Battlefield);
    let wrath = put(&mut game, active, "Day of Judgment", Zone::Hand);
    cast(&mut game, wrath, None);
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    assert!(matches!(
        game.state.pending,
        Pending::ZoneReplacement { .. }
    ));
    assert_eq!(
        game.actor(),
        Some(active),
        "simultaneous replacement choices are made in APNAP order"
    );

    let source = game.object_ref(nonactive_head);
    let choose_head = Action::ChooseZoneReplacement {
        effect: ZoneReplacementKind::ExileReflexive { source, ability: 0 },
    };
    let before_choices = game.snapshot();
    let index = game
        .legal_actions()
        .iter()
        .position(|a| *a == choose_head)
        .unwrap();
    let mut indexed = game.clone();
    indexed.apply_index(active, game.revision(), index).unwrap();
    act(&mut game, choose_head);
    assert_eq!(indexed.snapshot(), game.snapshot());
    assert_eq!(game.actor(), Some(nonactive));
    assert_eq!(game.object_view(active_head).zone, Zone::Battlefield);
    assert_eq!(game.object_view(nonactive_head).zone, Zone::Battlefield);

    act(
        &mut game,
        Action::ChooseZoneReplacement {
            effect: ZoneReplacementKind::Exile,
        },
    );
    settle(&mut game);
    assert_eq!(game.object_view(active_head).zone, Zone::Exile);
    assert_eq!(game.object_view(nonactive_head).zone, Zone::Exile);
    let wolves: Vec<_> = game
        .observe(nonactive)
        .objects
        .into_iter()
        .filter(|o| o.zone == Zone::Battlefield && o.is_token() && o.card.has_subtype("Wolf"))
        .collect();
    assert_eq!(wolves.len(), 1);
    assert_eq!(wolves[0].controller, nonactive);
    indexed.restore(&before_choices).unwrap();
    assert_eq!(indexed.actor(), Some(active));
}
