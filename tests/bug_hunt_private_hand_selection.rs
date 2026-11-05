mod support;
use rustmage as _;
use rustmage_core::engine::*;
use support::*;

#[test]
fn brainstorm_does_not_reveal_whether_a_known_card_went_back() {
    let p = Player::P0;
    let mut game = scenario(p);
    let trio = put(&mut game, p, "Harmonized Trio", Zone::Battlefield);
    let known = put(&mut game, p, "Bear Cub", Zone::Battlefield);
    let bounce = put(&mut game, p, "Unsummon", Zone::Hand);
    let island = put(&mut game, p, "Island", Zone::Library);
    let mountain = put(&mut game, p, "Mountain", Zone::Library);
    put(&mut game, p, "Swamp", Zone::Library);
    cast(&mut game, bounce, Some(Target::Card(known)));
    settle(&mut game);

    let visible_hand = |game: &Game| {
        game.observe(p.other())
            .objects
            .into_iter()
            .filter(|o| o.zone == Zone::Hand && o.owner == p)
            .map(|o| o.id)
            .collect::<Vec<_>>()
    };
    assert_eq!(visible_hand(&game), vec![known]);
    game.prepare_permanent(game.object_ref(trio));
    let brainstorm = game.observe(p).preparations[0].spell_copy.card;
    act(&mut game, Action::AnnounceExile { card: brainstorm });
    pay(&mut game);
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    assert!(matches!(game.state.pending, Pending::EffectChoice { .. }));

    let select = |game: &mut Game, card| {
        let object = game.object_ref(card);
        act(game, Action::Sacrifice { object });
    };
    select(&mut game, island);
    let checkpoint = game.snapshot();
    let action = Action::Sacrifice {
        object: game.object_ref(known),
    };
    let index = game
        .legal_actions()
        .iter()
        .position(|candidate| *candidate == action)
        .unwrap();
    let mut returned_known = game.clone();
    act(&mut returned_known, action);
    let mut indexed = game.clone();
    indexed.apply_index(p, game.revision(), index).unwrap();
    assert_eq!(indexed.snapshot(), returned_known.snapshot());
    indexed.restore(&checkpoint).unwrap();
    assert_eq!(indexed.snapshot(), game.snapshot());

    select(&mut game, mountain);
    assert_eq!(returned_known.object_view(known).zone, Zone::Library);
    assert_eq!(game.object_view(known).zone, Zone::Hand);
    assert_eq!(game.hand_count(p), returned_known.hand_count(p));
    assert_eq!(
        visible_hand(&game),
        visible_hand(&returned_known),
        "the opponent cannot distinguish the identities of privately returned cards",
    );
    let kept = serde_json::to_value(game.observe(p.other())).unwrap();
    let returned = serde_json::to_value(returned_known.observe(p.other())).unwrap();
    assert_eq!(kept, returned);
}
