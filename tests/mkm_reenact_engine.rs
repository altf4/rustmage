//! Free casting of an exact card copy from the original's actual public zone.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const REPLAY: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Filtered {
            base: &TargetRule::Object {
                zone: TargetZone::Graveyard,
                kinds: 0,
                relation: -1,
                other: false,
                nonland: true,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            },
            predicate: &TargetPredicate::EnteredGraveyardThisTurn,
        })),
        &[Instruction::ExileTargetCopyCastFree],
    ),
    ..CardRules::EMPTY
};
const NO_EXILE: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::GraveyardCardsCannotBeExiled],
    }],
    ..CardRules::EMPTY
};

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

fn cast_copy(object: ObjectRef) -> Action {
    Action::CastExiledDuringResolution {
        object,
        face: CastFace::Normal,
    }
}

fn offer(game: &mut Game, player: Player, origin: Zone) -> ObjectRef {
    act(game, Action::Pass);
    act(game, Action::Pass);
    let offer = game.observe(player).resolution_free_casts.unwrap();
    assert_eq!(offer.origin, origin);
    assert_eq!(offer.remaining.len(), 1);
    offer.remaining[0]
}

#[test]
fn both_graveyards_are_targetable_and_the_copy_stays_in_the_actual_origin() {
    let spell = Card::from_name("Opt").unwrap();
    let lock = Card::from_name("Authority of the Consuls").unwrap();
    let _rules = fixture_rules::activate(&[(spell, &REPLAY), (lock, &NO_EXILE)]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let old = put(&mut game, opponent, "Grizzly Bears", Zone::Graveyard);
    let land = put(&mut game, player, "Forest", Zone::Hand);
    let recent = put(&mut game, opponent, "Bear Cub", Zone::Hand);
    game.move_object(land, Zone::Graveyard);
    game.move_object(recent, Zone::Graveyard);
    let spell = spawn(&mut game, player, spell, Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: spell });
    reject(&mut game, target(old));
    reject(&mut game, target(land));
    act(&mut game, target(recent));
    pay(&mut game);

    let mut prevented = game.clone();
    spawn(&mut prevented, opponent, lock, Zone::Battlefield);
    let old_ref = game.object_ref(recent);

    let copy = offer(&mut game, player, Zone::Exile);
    assert_eq!(game.object_view(recent).zone, Zone::Exile);
    assert_eq!(game.object_view(copy.card).zone, Zone::Exile);
    let physical = game.object_ref(recent);
    reject(&mut game, cast_copy(physical));
    act(&mut game, Action::FinishResolutionCasts);
    assert_eq!(game.object_view(copy.card).zone, Zone::Outside);
    assert_eq!(game.object_view(recent).zone, Zone::Exile);

    let copy = offer(&mut prevented, player, Zone::Graveyard);
    assert_eq!(prevented.object_ref(recent), old_ref);
    assert_eq!(prevented.object_view(copy.card).zone, Zone::Graveyard);
    reject(&mut prevented, cast_copy(old_ref));
    prevented.state.players[player.index()].mana = [0; 6];
    let action = cast_copy(copy);
    let root = prevented.snapshot();
    let index = prevented
        .legal_actions()
        .iter()
        .position(|a| *a == action)
        .unwrap();
    let mut indexed = prevented.clone();
    indexed
        .apply_index(player, prevented.revision(), index)
        .unwrap();
    act(&mut prevented, action);
    assert_eq!(prevented.snapshot(), indexed.snapshot());
    indexed.restore(&root).unwrap();
    assert_eq!(indexed.snapshot(), root);
    if matches!(prevented.state.pending, Pending::PayCosts { .. }) {
        pay(&mut prevented);
    }
    if prevented
        .legal_actions()
        .contains(&Action::FinishResolutionCasts)
    {
        act(&mut prevented, Action::FinishResolutionCasts);
    }
    let cast = prevented
        .state
        .stack
        .iter()
        .find(|item| item.source.is_some_and(|source| source.id == copy.card))
        .unwrap();
    assert_eq!(cast.cast.unwrap().origin, Zone::Graveyard);
    settle(&mut prevented);
    let token = prevented.object_view(copy.card);
    assert_eq!(token.zone, Zone::Battlefield);
    assert!(token.is_token());
    assert_eq!(token.controller, player);
    assert_eq!(prevented.object_ref(recent), old_ref);
    assert_eq!(prevented.object_view(recent).zone, Zone::Graveyard);
}
