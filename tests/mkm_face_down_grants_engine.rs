//! Exact layer-six grants and the free cast from a failed turn-up attempt.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const GRANT: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Ability,
    fragments: &[StaticAbilityFragment::ControlledFaceDownCreatureActivation(
        0,
    )],
}];
const GRANTER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            mana: ManaCost {
                colored: [0, 1, 1, 0, 0, 0],
                generic: 2,
            },
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::TurnSourceFaceUpOrExile,
            Instruction::CastRememberedExileFree,
        ],
        sorcery: false,
        once_per_turn: false,
    }],
    static_groups: GRANT,
    extras: &CardExtras {
        static_ability_groups: GRANT,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

const CLOAK: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::CloakLibraryTop(PlayerSet::Opponent)],
    ),
    ..CardRules::EMPTY
};
const REMOVE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[Instruction::RemoveTargetAbilitiesUntilEndOfTurn],
    ),
    ..CardRules::EMPTY
};

fn grant(source: ObjectRef, grantor: ObjectRef) -> Action {
    Action::AnnounceCreatureGrant {
        source,
        grantor,
        ability: 0,
    }
}

#[test]
fn a_granted_activation_exiles_and_can_cast_only_its_exact_hidden_source() {
    let provider = Card::from_name("Grizzly Bears").unwrap();
    let cloak = Card::from_name("Opt").unwrap();
    let remove = Card::from_name("Unsummon").unwrap();
    let _rules =
        fixture_rules::activate(&[(provider, &GRANTER), (cloak, &CLOAK), (remove, &REMOVE)]);
    for player in [Player::P0, Player::P1] {
        let opponent = player.other();
        let mut game = scenario(player);
        let giver = spawn(&mut game, player, provider, Zone::Battlefield);
        let hidden = put(&mut game, opponent, "Shock", Zone::Library);
        let cloak_spell = spawn(&mut game, player, cloak, Zone::Hand);
        cast(&mut game, cloak_spell, None);
        settle(&mut game);
        let object = game.object_ref(hidden);
        let grantor = game.object_ref(giver);
        let action = grant(object, grantor);
        assert_eq!(
            game.observe(opponent).face_down_permanents[0].underlying,
            None
        );
        reject(&mut game, grant(grantor, grantor));

        let mut silenced = game.clone();
        let silence = spawn(&mut silenced, player, remove, Zone::Hand);
        cast(&mut silenced, silence, Some(Target::Card(hidden)));
        settle(&mut silenced);
        reject(&mut silenced, action.clone());

        let saved = game.snapshot();
        let mut indexed = game.clone();
        let index = game
            .legal_actions()
            .iter()
            .position(|candidate| *candidate == action)
            .unwrap();
        act(&mut game, action);
        indexed
            .apply_index(player, indexed.revision(), index)
            .unwrap();
        assert_eq!(game.snapshot(), indexed.snapshot());
        pay(&mut game);
        assert_eq!(
            game.state.stack.last().unwrap().source.unwrap().object,
            object
        );
        act(&mut game, Action::Pass);
        act(&mut game, Action::Pass);
        let exiled = game.object_ref(hidden);
        assert_ne!(exiled, object);
        assert_eq!(game.object_view(hidden).zone, Zone::Exile);
        assert_eq!(game.object_view(hidden).owner, opponent);
        reject(
            &mut game,
            Action::CastExiledDuringResolution {
                object,
                face: CastFace::Normal,
            },
        );

        let mut decline = game.clone();
        act(&mut decline, Action::FinishResolutionCasts);
        assert_eq!(decline.object_view(hidden).zone, Zone::Exile);
        act(
            &mut game,
            Action::CastExiledDuringResolution {
                object: exiled,
                face: CastFace::Normal,
            },
        );
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
            act(&mut game, Action::FinishResolutionCasts);
        }
        assert_eq!(
            game.state.stack.last().unwrap().cast.unwrap().method,
            CastMethod::Free
        );
        settle(&mut game);
        assert_eq!(game.state.players[opponent.index()].life, 18);
        assert_eq!(game.object_view(hidden).zone, Zone::Graveyard);

        indexed.restore(&saved).unwrap();
        assert_eq!(indexed.object_ref(hidden), object);
        assert!(indexed.is_face_down(object));
    }
}

const WATCHDOG_GROUP: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Rule,
    fragments: &[StaticAbilityFragment::OpponentFaceUpProhibitedOnOwnTurn],
}];
const WATCHDOG: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    static_groups: WATCHDOG_GROUP,
    extras: &CardExtras {
        static_ability_groups: WATCHDOG_GROUP,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

#[test]
fn turn_up_prohibition_checks_the_effect_player_across_either_controller() {
    let provider = Card::from_name("Grizzly Bears").unwrap();
    let _rules = fixture_rules::activate(&[(provider, &WATCHDOG)]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    spawn(&mut game, player, provider, Zone::Battlefield);
    let owned = put(&mut game, player, "Forest", Zone::Library);
    let foreign = put(&mut game, opponent, "Forest", Zone::Library);
    game.cloak_library_top(player, player);
    game.cloak_library_top(opponent, opponent);
    let your_face = game.object_ref(owned);
    let their_face = game.object_ref(foreign);
    assert!(!game.turn_manifest_face_up(your_face, opponent));
    assert!(!game.turn_manifest_face_up(their_face, opponent));
    assert!(game.turn_manifest_face_up(their_face, player));
    assert_eq!(game.object_view(foreign).kinds & 2, 2);
    assert!(game.is_face_down(your_face));
}
