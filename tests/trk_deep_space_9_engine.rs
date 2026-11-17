//! Preserve exact top-library exiles before putting only the permanent results in play.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const EXILE_THEN_ENTER: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::ExileTopLibraryRemember {
                players: PlayerSet::Controller,
                amount: AmountValue::Fixed(4),
            },
            Instruction::PutRememberedExiledPermanentsOntoBattlefield,
        ],
    ),
    ..CardRules::EMPTY
};

#[test]
fn exiled_permanents_enter_and_nonpermanents_stay_exiled_with_owned_replay() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Opt").unwrap(), &EXILE_THEN_ENTER)]);
    let mut game = scenario(Player::P0);
    game.state.players[0].library.clear();
    let spell_card = put(&mut game, Player::P0, "Shock", Zone::Library);
    let land = put(&mut game, Player::P0, "Forest", Zone::Library);
    let creature = put(&mut game, Player::P0, "Grizzly Bears", Zone::Library);
    let other_spell = put(&mut game, Player::P0, "Opt", Zone::Library);
    let source = put(&mut game, Player::P0, "Opt", Zone::Hand);

    let checkpoint = game.snapshot();
    let revision = game.revision();
    let action = Action::AnnounceCast { card: source };
    let index = game
        .legal_actions()
        .iter()
        .position(|a| *a == action)
        .unwrap();
    let mut indexed = game.clone();
    act(&mut game, action.clone());
    indexed.apply_index(Player::P0, revision, index).unwrap();
    assert_eq!(game.snapshot(), indexed.snapshot());
    indexed.restore(&checkpoint).unwrap();
    assert_eq!(
        indexed.apply_index(Player::P0, revision, index),
        Err(Error::StaleRevision)
    );
    assert_eq!(indexed.snapshot(), checkpoint);
    let mut replay = indexed.clone();
    replay
        .apply_index(Player::P0, replay.revision(), index)
        .unwrap();
    act(&mut indexed, action);
    pay(&mut replay);
    settle(&mut replay);
    pay(&mut game);
    pay(&mut indexed);
    settle(&mut game);
    settle(&mut indexed);
    assert_eq!(replay.snapshot(), indexed.snapshot());

    assert_eq!(game.object_view(land).zone, Zone::Battlefield);
    assert_eq!(game.object_view(creature).zone, Zone::Battlefield);
    assert_eq!(game.object_view(spell_card).zone, Zone::Exile);
    assert_eq!(game.object_view(other_spell).zone, Zone::Exile);
    assert!(game.state.players[0].library.is_empty());
}
