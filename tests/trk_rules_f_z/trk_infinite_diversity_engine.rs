//! Repeated copies of one exiled graveyard card are separate optional free casts.
use crate::support::*;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};

const REPEATED: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Object {
            zone: TargetZone::Graveyard,
            kinds: 4 | 8,
            relation: 0,
            other: false,
            nonland: true,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        })),
        &[
            Instruction::ExileTargetCopyCastFreeRepeated(
                StatValue::DistinctControlledCreatureTypesExcept("Borg"),
            ),
            Instruction::ExileResolvingSpell,
        ],
    ),
    ..CardRules::EMPTY
};

fn offer(game: &mut Game, player: Player) -> Vec<ObjectRef> {
    act(game, Action::Pass);
    act(game, Action::Pass);
    let view = game.observe(player).resolution_free_casts.unwrap();
    assert_eq!(view.origin, Zone::Exile);
    view.remaining
}

#[test]
fn every_federation_type_makes_a_separate_free_cast_and_source_exiles() {
    let source_card = Card::from_name("Opt").unwrap();
    let _rules = fixture_rules::activate(&[(source_card, &REPEATED)]);
    let player = Player::P0;
    let mut game = scenario(player);
    put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let target = put(&mut game, player, "Divination", Zone::Graveyard);
    let source = spawn(&mut game, player, source_card, Zone::Hand);
    cast(&mut game, source, Some(Target::Card(target)));
    let copies = offer(&mut game, player);
    assert_eq!(copies.len(), 2);
    assert_ne!(copies[0], copies[1]);
    assert_eq!(game.object_view(target).zone, Zone::Exile);
    let physical = game.object_ref(target);
    reject(
        &mut game,
        Action::CastExiledDuringResolution {
            object: physical,
            face: CastFace::Normal,
        },
    );

    let first = Action::CastExiledDuringResolution {
        object: copies[0],
        face: CastFace::Normal,
    };
    let checkpoint = game.snapshot();
    let revision = game.revision();
    let index = game
        .legal_actions()
        .iter()
        .position(|action| *action == first)
        .unwrap();
    let mut indexed = game.clone();
    act(&mut game, first);
    indexed.apply_index(player, revision, index).unwrap();
    assert_eq!(game.snapshot(), indexed.snapshot());
    indexed.restore(&checkpoint).unwrap();
    assert_eq!(
        indexed.apply_index(player, revision, index),
        Err(Error::StaleRevision)
    );
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(&mut game);
    }
    let second = Action::CastExiledDuringResolution {
        object: copies[1],
        face: CastFace::Normal,
    };
    assert!(game.legal_actions().contains(&second));
    act(&mut game, second);
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(&mut game);
    }
    act(&mut game, Action::FinishResolutionCasts);
    assert_eq!(game.object_view(source).zone, Zone::Exile);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 4);
}

#[test]
fn zero_types_still_exile_target_and_spell_without_offering_a_copy() {
    let source_card = Card::from_name("Opt").unwrap();
    let _rules = fixture_rules::activate(&[(source_card, &REPEATED)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let target = put(&mut game, player, "Divination", Zone::Graveyard);
    let source = spawn(&mut game, player, source_card, Zone::Hand);
    cast(&mut game, source, Some(Target::Card(target)));
    settle(&mut game);
    assert_eq!(game.object_view(target).zone, Zone::Exile);
    assert_eq!(game.object_view(source).zone, Zone::Exile);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 0);
}
