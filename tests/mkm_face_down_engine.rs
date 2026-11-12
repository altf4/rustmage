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

    let mut ward = game.clone();
    act(&mut ward, Action::Pass);
    let opposing_spell = spawn(&mut ward, player.other(), remove, Zone::Hand);
    cast(&mut ward, opposing_spell, Some(Target::Card(hidden)));
    for _ in 0..8 {
        if ward
            .legal_actions()
            .contains(&Action::DeclineResolutionMana)
        {
            break;
        }
        act(&mut ward, Action::Pass);
    }
    assert_eq!(
        ward.state.pending,
        Pending::ResolutionMana {
            player: player.other()
        }
    );
    act(&mut ward, Action::DeclineResolutionMana);
    settle(&mut ward);
    assert_eq!(ward.object_view(opposing_spell).zone, Zone::Graveyard);
    assert!(ward.is_face_down(object));

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
    reject(
        &mut game,
        Action::PayFaceUp {
            payment: [0, 2, 0, 0, 0, 0],
        },
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

const TURN_UP_WATCH: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Rule,
    fragments: &[StaticAbilityFragment::OpponentFaceUpProhibitedOnOwnTurn],
}];
const WATCHDOG: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    static_groups: TURN_UP_WATCH,
    extras: &CardExtras {
        static_ability_groups: TURN_UP_WATCH,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const FORCE_TURN_UP: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[Instruction::TurnTargetFaceUp],
    ),
    ..CardRules::EMPTY
};

#[test]
fn live_own_turn_prohibition_applies_to_effects_and_paid_turn_up() {
    let bear = Card::from_name("Grizzly Bears").unwrap();
    let shock = Card::from_name("Shock").unwrap();
    let _scope = fixture_rules::activate(&[(bear, &WATCHDOG), (shock, &FORCE_TURN_UP)]);
    let smuggler = Card::from_name("Bubble Smuggler").unwrap();
    let player = Player::P0;
    let mut game = scenario(player);
    let provider = spawn(&mut game, player.other(), bear, Zone::Battlefield);
    let hidden = spawn(&mut game, player, smuggler, Zone::Hand);
    act(&mut game, Action::AnnounceDisguise { card: hidden });
    pay(&mut game);
    settle(&mut game);
    let object = game.object_ref(hidden);
    assert!(game.legal_actions().contains(&turn_up(object, DISGUISE)));

    game.state.active = player.other();
    game.state.priority = player;
    reject(&mut game, turn_up(object, DISGUISE));
    let spell = spawn(&mut game, player, shock, Zone::Hand);
    cast(&mut game, spell, Some(Target::Card(hidden)));
    settle(&mut game);
    assert!(game.is_face_down(object));
    assert_eq!(
        game.observe(player.other()).face_down_permanents[0].underlying,
        None
    );

    game.move_object(provider, Zone::Graveyard);
    game.state.priority = player;
    assert!(game.legal_actions().contains(&turn_up(object, DISGUISE)));
}

#[test]
fn paying_copied_disguise_on_a_cloaked_instant_reveals_it_without_turning_it_up() {
    let spell = Card::from_name("Opt").unwrap();
    let _scope = fixture_rules::activate(&[(spell, &CLOAK_OPPOSING_TOP)]);
    let shock = Card::from_name("Shock").unwrap();
    let smuggler = Card::from_name("Bubble Smuggler").unwrap();
    let player = Player::P0;
    let mut game = scenario(player);
    let hidden = spawn(&mut game, player.other(), shock, Zone::Library);
    let cloak = spawn(&mut game, player, spell, Zone::Hand);
    cast(&mut game, cloak, None);
    settle(&mut game);

    let donor = spawn(&mut game, player, smuggler, Zone::Battlefield);
    let copy = put(&mut game, player, "Fleeting Reflection", Zone::Hand);
    announce(
        &mut game,
        copy,
        &[Target::Card(hidden), Target::Card(donor)],
    );
    settle(&mut game);

    let object = game.object_ref(hidden);
    assert!(game.is_face_down(object));
    let before = game.state.players[player.index()]
        .mana
        .into_iter()
        .sum::<u32>();
    act(&mut game, turn_up(object, DISGUISE));
    choose(&mut game, |action| {
        matches!(action, Action::PayFaceUp { .. })
    });
    assert_eq!(game.object_ref(hidden), object);
    assert!(game.is_face_down(object));
    assert_eq!(
        game.observe(player.other()).face_down_permanents[0].underlying,
        Some(shock)
    );
    assert_eq!(game.counter_count(hidden, CounterKind::PlusOne), 0);
    assert_eq!(
        game.state.players[player.index()]
            .mana
            .into_iter()
            .sum::<u32>(),
        before - 6
    );
    assert!(game.state.stack.is_empty());
}

const OTHER_CREATURES: StatValue = StatValue::Count(CountFilter {
    zone: CountZone::Battlefield,
    relation: 0,
    kind: 1,
    exclude: 0,
    subtype: None,
    basic: false,
    other: true,
    cards_only: false,
});
const WARDEN: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    extras: &CardExtras {
        disguise_cost: Some(DISGUISE),
        entry_counter_rules: &[EntryCounterRule {
            guard: StepGuard::Always,
            amount: EntryCounterAmount::Stat(OTHER_CREATURES),
        }],
        as_turned_face_up_counters: &[(
            CounterKind::PlusOne,
            TurnUpCounterAmount::Stat(OTHER_CREATURES),
        )],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const PERFORMER: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfOrControlledCreatureTurnedFaceUp,
        targets: TargetPlan::NONE,
        program: &[Instruction::EnteringPowerDamagePlayers(PlayerSet::Opponent)],
    }],
    ..CardRules::EMPTY
};

#[test]
fn turn_up_replacement_precedes_the_trigger_and_departure_keeps_the_damage_actor() {
    let warden = Card::from_name("Grizzly Bears").unwrap();
    let performer = Card::from_name("Bear Cub").unwrap();
    let _scope = fixture_rules::activate(&[(warden, &WARDEN), (performer, &PERFORMER)]);
    let player = Player::P0;
    let mut game = scenario(player);
    spawn(&mut game, player, performer, Zone::Battlefield);
    let hidden = spawn(&mut game, player, warden, Zone::Hand);
    act(&mut game, Action::AnnounceDisguise { card: hidden });
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.counter_count(hidden, CounterKind::PlusOne), 0);

    put(&mut game, player, "Healer's Hawk", Zone::Battlefield);
    put(
        &mut game,
        player.other(),
        "Healer's Hawk",
        Zone::Battlefield,
    );
    let object = game.object_ref(hidden);
    act(&mut game, turn_up(object, DISGUISE));
    choose(&mut game, |action| {
        matches!(action, Action::PayFaceUp { .. })
    });
    assert_eq!(game.counter_count(hidden, CounterKind::PlusOne), 2);
    assert_eq!(game.state.stack.len(), 1);

    let damage = |game: &Game| 20 - game.state.players[player.other().index()].life;
    let mut present = game.clone();
    settle(&mut present);
    assert_eq!(damage(&present), 6);

    let grow = put(&mut game, player, "Giant Growth", Zone::Hand);
    cast(&mut game, grow, Some(Target::Card(hidden)));
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    assert_eq!(stats(&game, hidden), (9, 9));
    let removal = put(&mut game, player, "Murder", Zone::Hand);
    cast(&mut game, removal, Some(Target::Card(hidden)));
    settle(&mut game);
    assert_eq!(game.object_view(hidden).zone, Zone::Graveyard);
    assert_eq!(damage(&game), 9);
}

const ENTRY_TIPSTER: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::InterveningIf {
            guard: StepGuard::FaceDownCreatureEnteredThisTurn,
            event: &TriggerCondition::AtStep {
                step: TriggerStep::End,
                relation: CastRelation::You,
                guard: StepGuard::Always,
            },
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::PutCountersScaled {
            subject: Subject::Source,
            kind: CounterKind::PlusOne,
            amount: AmountValue::Fixed(1),
        }],
    }],
    ..CardRules::EMPTY
};

#[test]
fn end_step_observes_earlier_face_down_creature_entry_and_ignores_flip_only() {
    let tipster = Card::from_name("Grizzly Bears").unwrap();
    let _scope = fixture_rules::activate(&[(tipster, &ENTRY_TIPSTER)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let smuggler = Card::from_name("Bubble Smuggler").unwrap();
    let hidden = spawn(&mut game, player, smuggler, Zone::Hand);
    act(&mut game, Action::AnnounceDisguise { card: hidden });
    pay(&mut game);
    settle(&mut game);

    let mut flip_only = game.clone();
    flip_only.begin_turn();
    flip_only.state.step = Step::Main2;
    flip_only.state.priority = player;
    let hidden_object = game.object_ref(hidden);
    for (branch, expected) in [(&mut game, 1), (&mut flip_only, 0)] {
        // Enter the observer after the face-down creature to test turn history.
        let source = spawn(branch, player, tipster, Zone::Battlefield);
        branch.state.step = Step::Main2;
        branch.state.players[player.index()].mana = [8; 6];
        act(branch, turn_up(hidden_object, DISGUISE));
        choose(branch, |action| matches!(action, Action::PayFaceUp { .. }));
        act(branch, Action::Pass);
        act(branch, Action::Pass);
        assert_eq!(branch.state.step, Step::End);
        settle(branch);
        assert_eq!(branch.counter_count(source, CounterKind::PlusOne), expected);
    }
}

const TEMPORARY_LOOK: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::LookAtOpposingFaceDownCreaturesUntilEndOfTurn],
    ),
    ..CardRules::EMPTY
};
const CLOAK_OWN_TOP: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::CloakLibraryTop(PlayerSet::Controller)],
    ),
    ..CardRules::EMPTY
};

#[test]
fn temporary_look_remembers_known_creatures_and_expires_before_new_entries() {
    let player = Player::P0;
    let opponent = player.other();
    let look = Card::from_name("Opt").unwrap();
    let cloak = Card::from_name("Shock").unwrap();
    let _scope = fixture_rules::activate(&[(look, &TEMPORARY_LOOK), (cloak, &CLOAK_OWN_TOP)]);
    let mut game = scenario(player);
    let first = put(&mut game, opponent, "Healer's Hawk", Zone::Library);
    game.state.priority = opponent;
    let first_cloak = spawn(&mut game, opponent, cloak, Zone::Hand);
    cast(&mut game, first_cloak, None);
    settle(&mut game);

    let known = |game: &Game, id| {
        game.observe(player)
            .face_down_permanents
            .iter()
            .find(|view| view.object.card == id)
            .unwrap()
            .underlying
    };
    assert_eq!(known(&game, first), None);
    let peek = spawn(&mut game, player, look, Zone::Hand);
    cast(&mut game, peek, None);
    settle(&mut game);
    assert_eq!(known(&game, first), Card::from_name("Healer's Hawk"));

    let smuggler = Card::from_name("Bubble Smuggler").unwrap();
    let disguised = spawn(&mut game, opponent, smuggler, Zone::Hand);
    game.state.active = opponent;
    game.state.priority = opponent;
    act(&mut game, Action::AnnounceDisguise { card: disguised });
    pay(&mut game);
    assert_eq!(game.observe(player).face_down_spells[0].underlying, None);
    settle(&mut game);
    assert_eq!(known(&game, disguised), Some(smuggler));

    game.state.step = Step::End;
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    assert_eq!(game.state.active, player);
    assert_eq!(known(&game, first), Card::from_name("Healer's Hawk"));
    assert_eq!(known(&game, disguised), Some(smuggler));

    game.state.priority = opponent;
    game.state.players[opponent.index()].mana = [8; 6];
    let second = put(&mut game, opponent, "Grizzly Bears", Zone::Library);
    let second_cloak = spawn(&mut game, opponent, cloak, Zone::Hand);
    cast(&mut game, second_cloak, None);
    settle(&mut game);
    assert_eq!(known(&game, second), None);
}
