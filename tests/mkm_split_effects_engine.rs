//! Exact resolution effects of split spells.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const FREE_FROM_OPPONENT: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::CastOpponentGraveyardFreeThenExile],
    ),
    ..CardRules::EMPTY
};

#[test]
fn free_cast_uses_the_opposing_graveyard_and_exiles_the_resolved_spell() {
    let opt = Card::from_name("Opt").unwrap();
    let _rules = fixture_rules::activate(&[(opt, &FREE_FROM_OPPONENT)]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let chosen = put(&mut game, opponent, "Shock", Zone::Graveyard);
    let other = put(&mut game, opponent, "Grizzly Bears", Zone::Graveyard);
    let own = put(&mut game, player, "Shock", Zone::Graveyard);
    let land = put(&mut game, opponent, "Forest", Zone::Graveyard);
    let offer = spawn(&mut game, player, opt, Zone::Hand);
    cast(&mut game, offer, None);
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    let choice = |game: &Game, id| Action::CastExiledDuringResolution {
        object: game.object_ref(id),
        face: CastFace::Normal,
    };
    for illegal in [land, own] {
        let action = choice(&game, illegal);
        reject(&mut game, action);
    }
    let action = choice(&game, chosen);
    act(&mut game, action);
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(opponent),
        },
    );
    if game
        .legal_actions()
        .iter()
        .any(|a| matches!(a, Action::PayCosts { .. }))
    {
        pay(&mut game);
    }
    if game
        .legal_actions()
        .contains(&Action::FinishResolutionCasts)
    {
        let action = choice(&game, other);
        reject(&mut game, action);
        act(&mut game, Action::FinishResolutionCasts);
    }
    let cast = game.state.stack.last().unwrap().cast.unwrap();
    assert_eq!(cast.origin, Zone::Graveyard);
    assert_eq!(cast.method, CastMethod::Free);
    assert_eq!(cast.payment, [0; 6]);
    settle(&mut game);
    assert_eq!(game.object_view(chosen).zone, Zone::Exile);
    assert_eq!(game.object_view(other).zone, Zone::Graveyard);
}

const OPTIONAL_TURN_UP: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::MayTurnControlledCreatureFaceUp],
    ),
    ..CardRules::EMPTY
};

#[test]
fn optional_untargeted_turn_up_can_reveal_an_underlying_instant() {
    let opt = Card::from_name("Opt").unwrap();
    let _rules = fixture_rules::activate(&[(opt, &OPTIONAL_TURN_UP)]);
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let hidden = put(&mut game, opponent, "Shock", Zone::Library);
    game.cloak_library_top(player, opponent);
    assert!(!game.advance_entries());
    let other = put(&mut game, opponent, "Grizzly Bears", Zone::Library);
    game.cloak_library_top(opponent, opponent);
    assert!(!game.advance_entries());
    let offer = spawn(&mut game, player, opt, Zone::Hand);
    cast(&mut game, offer, None);
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    let hostile = game.object_ref(other);
    reject(&mut game, Action::ChooseEffectObject { object: hostile });
    let object = game.object_ref(hidden);
    let seen = |game: &Game| {
        game.observe(opponent)
            .face_down_permanents
            .iter()
            .find(|row| row.object == object)
            .unwrap()
            .underlying
    };
    assert_eq!(seen(&game), None);
    let mut declined = game.clone();
    act(&mut declined, Action::DeclineEffectPayment);
    act(&mut game, Action::ChooseEffectObject { object });
    assert!(game.is_face_down(object));
    assert_eq!(seen(&game), Card::from_name("Shock"));
    assert_eq!(seen(&declined), None);
}

const CREATURE_CARD: TargetRule = TargetRule::Object {
    zone: TargetZone::Graveyard,
    kinds: 1,
    relation: -1,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
};
const RETURN_WITH_DELAY: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec {
            rule: TargetRule::Grouped {
                base: &CREATURE_CARD,
                constraint: TargetGroupConstraint::SameGraveyard,
            },
            min: 0,
            max: 2,
        }),
        &[Instruction::ReturnGraveyardTargetsWithHasteAndDelayedSacrifice],
    ),
    ..CardRules::EMPTY
};

#[test]
fn delayed_sacrifice_tracks_the_exact_creatures_that_returned() {
    let opt = Card::from_name("Opt").unwrap();
    let _rules = fixture_rules::activate(&[(opt, &RETURN_WITH_DELAY)]);
    let player = Player::P0;
    let owner = player.other();
    let mut game = scenario(player);
    let first = put(&mut game, owner, "Grizzly Bears", Zone::Graveyard);
    let second = put(&mut game, owner, "Bear Cub", Zone::Graveyard);
    let own = put(&mut game, player, "Healer's Hawk", Zone::Graveyard);
    let spell = spawn(&mut game, player, opt, Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: spell });
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(first),
        },
    );
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(own),
        },
    );
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(second),
        },
    );
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(&mut game, Action::FinishTargets);
    }
    pay(&mut game);
    settle(&mut game);
    for id in [first, second] {
        assert_eq!(game.object_view(id).controller, player);
        assert_eq!(game.object_view(id).owner, owner);
        assert!(game.keywords(id).contains(Keywords::HASTE));
    }

    game.move_object(first, Zone::Exile);
    game.move_object(first, Zone::Battlefield);
    game.state.step = Step::Main2;
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    settle(&mut game);
    assert_eq!(game.object_view(first).zone, Zone::Battlefield);
    assert!(!game.keywords(first).contains(Keywords::HASTE));
    assert_eq!(game.object_view(second).zone, Zone::Graveyard);
}

const RETURN_EQUIPMENT: CardRules = CardRules {
    effect: Effect::Creature(3, 4),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Filtered {
            base: &TargetRule::Object {
                zone: TargetZone::Graveyard,
                kinds: 16,
                relation: 0,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            },
            predicate: &TargetPredicate::Subtype("Equipment"),
        })),
        program: &[Instruction::ReturnGraveyardEquipmentAttachedToSource],
    }],
    ..CardRules::EMPTY
};

#[test]
fn equipment_returns_attached_only_to_the_original_creature() {
    let creature = Card::from_name("Grizzly Bears").unwrap();
    let _rules = fixture_rules::activate(&[(creature, &RETURN_EQUIPMENT)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let source = spawn(&mut game, player, creature, Zone::Hand);
    let equipment = put(&mut game, player, "Short Bow", Zone::Graveyard);
    cast(&mut game, source, None);
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(equipment),
        },
    );
    let source_ref = game.object_ref(source);
    let mut moved = game.clone();
    moved.move_object(source, Zone::Exile);
    settle(&mut moved);
    assert_eq!(moved.object_view(equipment).zone, Zone::Battlefield);
    assert_eq!(moved.object_view(equipment).attached_to, None);
    settle(&mut game);
    assert_eq!(game.object_view(equipment).attached_to, Some(source_ref));
}
