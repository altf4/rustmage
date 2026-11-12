//! Public face-down causes, private identities, and their exact payment routes.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const PRINTED: ManaCost = ManaCost {
    colored: [0, 1, 0, 0, 0, 0],
    generic: 1,
};
const DISGUISE: ManaCost = ManaCost {
    colored: [0, 1, 0, 0, 0, 0],
    generic: 5,
};
const CLOAK_OPPOSING_TOP: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::CloakLibraryTop(PlayerSet::Opponent)],
    ),
    ..CardRules::EMPTY
};
const REMOVE_ABILITIES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[Instruction::RemoveTargetAbilitiesUntilEndOfTurn],
    ),
    ..CardRules::EMPTY
};
const CLOAK_AND_ATTACH: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::CaptureEntryResult,
            Instruction::CloakLibraryTop(PlayerSet::Controller),
            Instruction::AttachSourceToEntryResult,
        ],
    }],
    ..CardRules::EMPTY
};

fn turn_up(object: ObjectRef, cost: ManaCost) -> Action {
    Action::BeginFaceUp { object, cost }
}

#[test]
fn disguise_only_allows_disguise_payment_and_keeps_the_incarnation() {
    for player in [Player::P0, Player::P1] {
        let smuggler = Card::from_name("Bubble Smuggler").unwrap();
        let mut game = scenario(player);
        let hidden = spawn(&mut game, player, smuggler, Zone::Hand);
        act(&mut game, Action::AnnounceDisguise { card: hidden });
        pay(&mut game);
        let spells = game.observe(player.other()).face_down_spells;
        assert_eq!(spells.len(), 1);
        assert_eq!(spells[0].cause, FaceDownCause::Disguise);
        assert_eq!(spells[0].underlying, None);
        settle(&mut game);

        let object = game.object_ref(hidden);
        let opponents_view = game.observe(player.other()).face_down_permanents;
        assert_eq!(opponents_view[0].cause, FaceDownCause::Disguise);
        assert_eq!(opponents_view[0].underlying, None);
        assert_eq!(stats(&game, hidden), (2, 2));
        reject(&mut game, turn_up(object, PRINTED));

        let snapshot = game.snapshot();
        let action = turn_up(object, DISGUISE);
        let index = game
            .legal_actions()
            .iter()
            .position(|candidate| *candidate == action)
            .expect("disguise cost");
        let mut indexed = game.clone();
        act(&mut game, action);
        indexed
            .apply_index(player, indexed.revision(), index)
            .unwrap();
        assert_eq!(indexed.snapshot(), game.snapshot());
        assert_eq!(
            game.observe(player.other()).face_down_permanents[0].underlying,
            Some(smuggler)
        );

        choose(&mut game, |action| {
            matches!(action, Action::PayFaceUp { .. })
        });
        assert_eq!(game.object_ref(hidden), object);
        assert_eq!(game.object_view(hidden).card, smuggler);
        assert_eq!(stats(&game, hidden), (6, 5));
        assert!(!game.is_face_down(object));

        indexed.restore(&snapshot).unwrap();
        assert!(indexed.is_face_down(object));
        assert_eq!(
            indexed.observe(player.other()).face_down_permanents[0].underlying,
            None
        );
    }
}

#[test]
fn cloak_keeps_opponent_owned_identity_private_and_allows_either_payment() {
    let spell = Card::from_name("Opt").unwrap();
    let remove = Card::from_name("Shock").unwrap();
    let _scope =
        fixture_rules::activate(&[(spell, &CLOAK_OPPOSING_TOP), (remove, &REMOVE_ABILITIES)]);
    let smuggler = Card::from_name("Bubble Smuggler").unwrap();
    let player = Player::P1;
    let mut game = scenario(player);
    let hidden = spawn(&mut game, player.other(), smuggler, Zone::Library);
    let opt = spawn(&mut game, player, spell, Zone::Hand);
    cast(&mut game, opt, None);
    settle(&mut game);

    let object = game.object_ref(hidden);
    let view = game.object_view(hidden);
    assert_eq!(view.controller, player);
    assert_eq!(view.owner, player.other());
    assert_eq!(view.kinds, 1);
    assert_eq!(view.colors, 0);
    assert_eq!(view.name, "");
    assert_eq!(stats(&game, hidden), (2, 2));
    let public = game.observe(player.other());
    assert_eq!(public.face_down_permanents[0].cause, FaceDownCause::Cloak);
    assert_eq!(public.face_down_permanents[0].underlying, None);
    assert!(public.last_manifest_dread.is_none());
    assert_eq!(
        game.observe(player).face_down_permanents[0].underlying,
        Some(smuggler)
    );
    assert!(game.legal_actions().contains(&turn_up(object, DISGUISE)));
    assert!(game.legal_actions().contains(&turn_up(object, PRINTED)));

    let remove = spawn(&mut game, player, remove, Zone::Hand);
    cast(&mut game, remove, Some(Target::Card(hidden)));
    settle(&mut game);
    reject(&mut game, turn_up(object, DISGUISE));
    act(&mut game, turn_up(object, PRINTED));
    choose(&mut game, |action| {
        matches!(action, Action::PayFaceUp { .. })
    });
    assert_eq!(game.object_ref(hidden), object);
    assert_eq!(game.object_view(hidden).card, smuggler);
    assert_eq!(game.counter_count(hidden, CounterKind::PlusOne), 0);
}

#[test]
fn cloak_returns_the_exact_successful_entry_to_its_equipment_continuation() {
    let equipment = Card::from_name("Short Bow").unwrap();
    let _scope = fixture_rules::activate(&[(equipment, &CLOAK_AND_ATTACH)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let hidden = put(&mut game, player, "Shock", Zone::Library);
    let bow = spawn(&mut game, player, equipment, Zone::Hand);
    cast(&mut game, bow, None);
    settle(&mut game);

    let object = game.object_ref(hidden);
    assert!(game.is_face_down(object));
    assert_eq!(game.object_view(bow).attached_to, Some(object));
    assert!(
        game.observe(player.other()).face_down_permanents[0]
            .underlying
            .is_none()
    );
    assert!(!game.legal_actions().iter().any(
        |action| matches!(action, Action::BeginFaceUp { object: candidate, .. } if *candidate == object)
    ));

    let mut empty = scenario(player);
    for id in empty.state.players[player.index()].library.clone() {
        empty.move_object(id, Zone::Exile);
    }
    let bow = spawn(&mut empty, player, equipment, Zone::Hand);
    cast(&mut empty, bow, None);
    settle(&mut empty);
    assert_eq!(empty.object_view(bow).attached_to, None);
    assert!(empty.observe(player).face_down_permanents.is_empty());
}

const VARIABLE_DISGUISE: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfTurnedFaceUp,
        targets: TargetPlan::NONE,
        program: &[Instruction::PutCountersScaled {
            subject: Subject::Source,
            kind: CounterKind::PlusOne,
            amount: AmountValue::ChosenX,
        }],
    }],
    extras: &CardExtras {
        disguise: Some(DisguiseSpec {
            mana: PRINTED,
            alternatives: &[],
            modifier: DisguiseCostModifier::X(std::num::NonZeroU8::MIN),
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const MIXED_DISGUISE: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    extras: &CardExtras {
        disguise: Some(DisguiseSpec {
            mana: ManaCost {
                colored: [0, 0, 2, 0, 0, 0],
                generic: 4,
            },
            alternatives: &[
                ManaCost {
                    colored: [0, 0, 1, 0, 1, 0],
                    generic: 4,
                },
                ManaCost {
                    colored: [0, 0, 0, 0, 2, 0],
                    generic: 4,
                },
            ],
            modifier: DisguiseCostModifier::GenericReduction(StatValue::Count(CountFilter {
                zone: CountZone::Graveyard,
                relation: 0,
                kind: 12,
                exclude: 0,
                subtype: None,
                basic: false,
                other: false,
                cards_only: true,
            })),
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

#[test]
fn variable_disguise_locks_the_chosen_value_and_binds_it_to_the_turn_up_trigger() {
    let bear = Card::from_name("Grizzly Bears").unwrap();
    let _scope = fixture_rules::activate(&[(bear, &VARIABLE_DISGUISE)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let hidden = spawn(&mut game, player, bear, Zone::Hand);
    act(&mut game, Action::AnnounceDisguise { card: hidden });
    pay(&mut game);
    settle(&mut game);

    let object = game.object_ref(hidden);
    act(
        &mut game,
        Action::BeginFaceUpX {
            object,
            cost: PRINTED,
            x_symbols: std::num::NonZeroU8::MIN,
        },
    );
    let domain = game.x_choice().expect("disguise X choice");
    assert_eq!(domain.card, object);
    assert!(domain.contains(3));
    assert!(
        !game
            .legal_actions()
            .iter()
            .any(|action| matches!(action, Action::PayFaceUp { .. }))
    );
    reject(&mut game, Action::ChooseX { value: u32::MAX });
    let before = game.snapshot();
    let mut branch = game.clone();
    act(&mut branch, Action::AbortFaceUpPayment);
    assert!(branch.is_face_down(object));
    assert_eq!(
        branch.observe(player.other()).face_down_permanents[0].underlying,
        Some(bear)
    );

    act(&mut game, Action::ChooseX { value: 3 });
    let payment = game.face_up_payment().unwrap();
    assert_eq!(payment.cost.generic, 4);
    assert_eq!(payment.x, Some(3));
    choose(&mut game, |action| {
        matches!(action, Action::PayFaceUp { .. })
    });
    settle(&mut game);
    assert_eq!(game.counter_count(hidden, CounterKind::PlusOne), 3);

    branch.restore(&before).unwrap();
    act(&mut branch, Action::ChooseX { value: 0 });
    choose(&mut branch, |action| {
        matches!(action, Action::PayFaceUp { .. })
    });
    settle(&mut branch);
    assert_eq!(branch.counter_count(hidden, CounterKind::PlusOne), 0);
    assert_eq!(game.counter_count(hidden, CounterKind::PlusOne), 3);
}

#[test]
fn disguise_hybrid_payment_uses_the_live_count_of_qualifying_graveyard_cards() {
    let bear = Card::from_name("Grizzly Bears").unwrap();
    let _scope = fixture_rules::activate(&[(bear, &MIXED_DISGUISE)]);
    let player = Player::P1;
    let mut game = scenario(player);
    let hidden = spawn(&mut game, player, bear, Zone::Hand);
    act(&mut game, Action::AnnounceDisguise { card: hidden });
    pay(&mut game);
    settle(&mut game);
    let object = game.object_ref(hidden);

    put(&mut game, player, "Shock", Zone::Graveyard);
    let second = put(&mut game, player, "Opt", Zone::Graveyard);
    put(&mut game, player, "Forest", Zone::Graveyard);
    put(&mut game, player.other(), "Shock", Zone::Graveyard);
    game.state.players[player.index()].mana = [0, 0, 1, 0, 1, 2];
    let mixed = ManaCost {
        colored: [0, 0, 1, 0, 1, 0],
        generic: 2,
    };
    let options: Vec<_> = game
        .legal_actions()
        .into_iter()
        .filter(|action| matches!(action, Action::BeginFaceUp { .. }))
        .collect();
    assert_eq!(options, vec![turn_up(object, mixed)]);

    let mut no_discount = game.clone();
    no_discount.move_object(second, Zone::Exile);
    reject(&mut no_discount, turn_up(object, mixed));
    act(&mut game, turn_up(object, mixed));
    choose(&mut game, |action| {
        matches!(action, Action::PayFaceUp { .. })
    });
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    assert!(!game.is_face_down(object));
}
