//! Exact physical-source returns and projected entry-face contracts.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const FRONT: &str = "Brigid, Clachan's Heart";
const BACK: &str = "Brigid, Doun's Mind";

const TRANSFORM_ENTRY: ActivatedAbility = ActivatedAbility {
    exhaust: false,
    guard: ActivationGuard::Always,
    cycling: false,
    origin: ActivationOrigin::Battlefield,
    cost: ActivationCost {
        generic: 1,
        mana_component: true,
        tap: true,
        ..ActivationCost::ZERO
    },
    targets: TargetPlan::NONE,
    program: &[Instruction::ExileBattlefieldSourceReturn {
        transformed: true,
        under_owner: true,
    }],
    sorcery: true,
    once_per_turn: false,
};

const FRONT_RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 2)],
    }],
    activated: &[TRANSFORM_ENTRY],
    extras: &CardExtras {
        transforming_double_face: true,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const BACK_RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 6),
    entry: EntryTapped::Always,
    keywords: Keywords::FLYING,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 3)],
    }],
    activated: &[ActivatedAbility {
        cost: ActivationCost::ZERO,
        program: &[Instruction::ExileBattlefieldSourceReturn {
            transformed: false,
            under_owner: false,
        }],
        sorcery: false,
        ..TRANSFORM_ENTRY
    }],
    extras: &CardExtras {
        entry_choice: Some(EntryChoiceKind::Color),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const GRAVEYARD_RULES: CardRules = CardRules {
    effect: Effect::Spell(TargetPlan::NONE, &[]),
    activated: &[ActivatedAbility {
        origin: ActivationOrigin::Graveyard,
        cost: ActivationCost {
            generic: 1,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        program: &[Instruction::ReturnGraveyardSourceTransformed],
        ..TRANSFORM_ENTRY
    }],
    extras: &CardExtras {
        transforming_double_face: true,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

fn face_fixture() -> fixture_rules::Guard {
    fixture_rules::activate(&[
        (Card::from_name(FRONT).unwrap(), &FRONT_RULES),
        (Card::from_name(BACK).unwrap(), &BACK_RULES),
    ])
}

fn activate(g: &mut Game, card: CardId) {
    act(
        g,
        Action::AnnounceActivate {
            card,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    if matches!(g.state.pending, Pending::PayCosts { .. }) {
        pay(g);
    }
}

fn color(value: ChoiceColor) -> Action {
    Action::ChooseEntryValue {
        value: EntryChoiceValue::Color(value),
    }
}

#[test]
fn source_blinks_back_for_owner_and_front_again_with_owned_entry_choice() {
    let _rules = face_fixture();
    let mut parent = scenario(Player::P0);
    let card = put(&mut parent, Player::P1, FRONT, Zone::Battlefield);
    parent.state.objects[card as usize].controller = Player::P0;
    let before = parent.object_ref(card);
    parent.set_counters(card, Counters::of(CounterKind::PlusOne, 2));
    activate(&mut parent, card);
    assert!(parent.object_view(card).tapped);
    assert_eq!(parent.object_ref(card), before);
    for _ in 0..8 {
        if matches!(parent.state.pending, Pending::EntryValue { .. }) {
            break;
        }
        act(&mut parent, Action::Pass);
    }
    assert!(matches!(parent.state.pending, Pending::EntryValue { .. }));
    assert_eq!(parent.actor(), Some(Player::P1));
    let exiled = parent.object_view(card);
    assert_eq!(exiled.zone, Zone::Exile);
    assert_eq!(exiled.card, Card::from_name(FRONT).unwrap());
    assert_ne!(exiled.reference, before);
    let pending = &parent.state.entry_batch.as_ref().unwrap().moves[0];
    assert_eq!(pending.before.card, Card::from_name(BACK).unwrap());
    assert_eq!(pending.before.power, 5);
    assert!(pending.before.keywords.contains(Keywords::FLYING));

    let green = color(ChoiceColor::Green);
    let saved = parent.snapshot();
    let revision = parent.revision();
    let index = parent
        .legal_actions()
        .iter()
        .position(|action| action == &green)
        .unwrap();
    assert_eq!(
        parent.apply(Player::P0, green.clone()),
        Err(Error::WrongActor)
    );
    assert_eq!(parent.snapshot(), saved);
    let mut raw = parent.clone();
    let mut indexed = parent.clone();
    drop(parent);
    act(&mut raw, green);
    indexed.apply_index(Player::P1, revision, index).unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    settle(&mut raw);
    settle(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
    let back = raw.object_view(card);
    assert_eq!(back.zone, Zone::Battlefield);
    assert_ne!(back.reference, exiled.reference);
    assert_eq!(back.card, Card::from_name(BACK).unwrap());
    assert_eq!(back.controller, Player::P1);
    assert!(back.tapped && back.summoning_sick);
    assert_eq!(back.counters.get(CounterKind::PlusOne), 0);
    assert_eq!(raw.state.players[1].life, 23);
    assert_eq!(raw.state.players[0].life, 20);

    raw.restore(&saved).unwrap();
    act(&mut raw, color(ChoiceColor::White));
    settle(&mut raw);
    assert_eq!(
        raw.object_view(card).entry_choice,
        Some(EntryChoiceValue::Color(ChoiceColor::White))
    );
    assert_eq!(
        indexed.object_view(card).entry_choice,
        Some(EntryChoiceValue::Color(ChoiceColor::Green))
    );
    raw.state.objects[card as usize].controller = Player::P0;
    if raw.actor() != Some(Player::P0) {
        act(&mut raw, Action::Pass);
    }
    activate(&mut raw, card);
    raw.state.objects[card as usize].controller = Player::P1;
    settle(&mut raw);
    let front = raw.object_view(card);
    assert_eq!(front.zone, Zone::Battlefield);
    assert_eq!(front.card, Card::from_name(FRONT).unwrap());
    assert_eq!(front.owner, Player::P1);
    assert_eq!(front.controller, Player::P0);
    assert_ne!(front.reference, back.reference);
    assert!(!front.tapped);
    assert!(front.entry_choice.is_none());
    assert_eq!(raw.state.players[0].life, 22);
    assert_eq!(raw.state.players[1].life, 23);
}

#[test]
fn copied_source_uses_its_physical_face_and_tokens_cannot_return() {
    let _rules = face_fixture();
    for (physical, returns) in [
        ("Grizzly Bears", false),
        ("Tony Stark", true),
        ("Sygg, Wanderwine Wisdom", true),
    ] {
        let mut g = scenario(Player::P0);
        let donor = put(&mut g, Player::P1, FRONT, Zone::Battlefield);
        let card = put(&mut g, Player::P0, physical, Zone::Battlefield);
        let reference = g.object_ref(card);
        let recipe = g.copiable_recipe(g.object_ref(donor));
        g.apply_temporary_copy(reference, recipe, CopyDeadline::EndOfTurn);
        assert_eq!(g.object_view(card).card, Card::from_name(FRONT).unwrap());
        activate(&mut g, card);
        settle(&mut g);
        let returned = g.object_view(card);
        let original = Card::from_name(physical).unwrap();
        assert_ne!(returned.reference, reference);
        assert_eq!(
            (returned.zone, returned.card),
            if returns {
                (Zone::Battlefield, original.back_face().unwrap())
            } else {
                (Zone::Exile, original)
            },
        );
        assert!(returned.temporary_copy.is_none());
        assert_eq!(g.object_view(donor).zone, Zone::Battlefield);
    }

    let mut g = scenario(Player::P0);
    let donor = put(&mut g, Player::P1, FRONT, Zone::Battlefield);
    g.stage_copy_creation(Player::P0, g.copiable_recipe(g.object_ref(donor)), 1);
    g.prepare_triggers();
    settle(&mut g);
    let token = g.state.copy_recipes.as_ref().unwrap()[0].object.card;
    g.state.objects[token as usize].summoning_sick = false;
    activate(&mut g, token);
    settle(&mut g);
    assert_eq!(g.state.objects[token as usize].zone, Zone::Outside);
}

#[test]
fn battlefield_return_requires_current_source_and_successful_exile() {
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|v| v.set(None));
        }
    }
    let _rules = face_fixture();
    let _reset = Reset;
    for stale in [false, true] {
        let mut g = scenario(Player::P0);
        let card = put(&mut g, Player::P0, FRONT, Zone::Battlefield);
        activate(&mut g, card);
        if stale {
            g.move_object(card, Zone::Exile);
            g.move_object(card, Zone::Battlefield);
        } else {
            zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|v| v.set(Some(g.object_ref(card))));
        }
        let reference = g.object_ref(card);
        settle(&mut g);
        let source = g.object_view(card);
        assert_eq!(source.reference, reference);
        assert_eq!(source.zone, Zone::Battlefield);
        assert_eq!(source.card, Card::from_name(FRONT).unwrap());
        assert_eq!(source.tapped, !stale);
        zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|v| v.set(None));
    }
}

#[test]
fn graveyard_activation_enters_as_a_saga_and_ignores_stale_incarnations() {
    let front = Card::from_name("Esper Origins").unwrap();
    let _rules = fixture_rules::activate(&[(front, &GRAVEYARD_RULES)]);
    for stale in [false, true] {
        let mut g = scenario(Player::P1);
        let card = put(&mut g, Player::P1, "Esper Origins", Zone::Graveyard);
        let top = g.top_library_card(Player::P1).unwrap();
        activate(&mut g, card);
        if stale {
            g.move_object(card, Zone::Exile);
            g.move_object(card, Zone::Graveyard);
        }
        let reference = g.object_ref(card);
        settle(&mut g);
        let source = g.object_view(card);
        if stale {
            assert_eq!(source.reference, reference);
            assert_eq!(source.zone, Zone::Graveyard);
            assert_eq!(source.card, front);
            assert_eq!(g.object_view(top).zone, Zone::Library);
        } else {
            assert_ne!(source.reference, reference);
            assert_eq!(source.zone, Zone::Battlefield);
            assert_eq!(source.card, front.back_face().unwrap());
            assert_eq!(source.controller, Player::P1);
            assert!(source.has_subtype("Saga"));
            assert_eq!(source.counters.get(CounterKind::Lore), 1);
            assert_eq!(source.counters.get(CounterKind::Finality), 0);
            assert_eq!(g.object_view(top).zone, Zone::Hand, "chapter I resolved");
        }
    }
}

#[test]
fn spell_return_adds_counter_only_after_actual_exile() {
    const RULES: CardRules = CardRules {
        effect: Effect::Spell(
            TargetPlan::NONE,
            &[Instruction::ExileSourceReturnTransformed {
                counter: CounterKind::Finality,
            }],
        ),
        extras: &CardExtras {
            transforming_double_face: true,
            ..CardExtras::EMPTY
        },
        ..CardRules::EMPTY
    };
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            zone_replacements::TEST_REDIRECT_EXILE_TO_BATTLEFIELD.with(|v| v.set(None));
        }
    }
    let front = Card::from_name("The Enigma Jewel").unwrap();
    let _rules = fixture_rules::activate(&[(front, &RULES)]);
    let _reset = Reset;
    for redirected in [false, true] {
        let mut g = scenario(Player::P0);
        let card = put(&mut g, Player::P0, "The Enigma Jewel", Zone::Hand);
        cast(&mut g, card, None);
        let stack = g.object_ref(card);
        if redirected {
            zone_replacements::TEST_REDIRECT_EXILE_TO_BATTLEFIELD.with(|v| v.set(Some(stack)));
        }
        settle(&mut g);
        let source = g.object_view(card);
        assert_eq!(source.zone, Zone::Battlefield);
        assert_eq!(
            source.card,
            if redirected {
                front
            } else {
                front.back_face().unwrap()
            }
        );
        assert_eq!(
            source.counters.get(CounterKind::Finality),
            u64::from(!redirected)
        );
        assert_eq!(
            source.reference.generation,
            stack.generation + if redirected { 1 } else { 2 }
        );
        zone_replacements::TEST_REDIRECT_EXILE_TO_BATTLEFIELD.with(|v| v.set(None));
    }
}

#[test]
fn the_dying_back_face_moves_its_exact_graveyard_front_to_its_owner_bottom() {
    const DEATH: CardRules = CardRules {
        effect: Effect::Creature(5, 5),
        keywords: Keywords::FLYING,
        triggers: &[TriggeredAbility {
            condition: TriggerCondition::SelfDies,
            targets: TargetPlan::NONE,
            program: &[Instruction::PutDepartedSourceOnLibraryBottom],
        }],
        ..CardRules::EMPTY
    };
    let front = Card::from_name(FRONT).unwrap();
    let back = front.back_face().unwrap();
    let _rules = fixture_rules::activate(&[(front, &GRAVEYARD_RULES), (back, &DEATH)]);
    let owner = Player::P0;
    let controller = owner.other();
    let mut game = scenario(owner);
    let card = spawn(&mut game, owner, front, Zone::Graveyard);
    activate(&mut game, card);
    settle(&mut game);
    let permanent = game.object_ref(card);
    assert_eq!(game.object_view(card).card, back);
    game.record_control(permanent, controller, None);
    game.move_object(card, Zone::Graveyard);
    game.prepare_triggers();
    let departed = game.object_ref(card);
    assert_eq!(game.object_view(card).card, front);
    let trigger = game.state.stack.last().unwrap();
    assert_eq!(trigger.controller, controller);
    assert!(trigger.targets.is_empty());
    assert_eq!(trigger.source_move.as_ref().unwrap().after, departed);

    let top = game.object_ref(game.top_library_card(owner).unwrap());
    game.remember_private_library_top(owner, owner, top);
    let other_library = game.state.players[controller.index()].library.clone();
    let mut stale = game.clone();
    stale.move_object(card, Zone::Exile);
    stale.move_object(card, Zone::Graveyard);
    assert_ne!(stale.object_ref(card), departed);
    settle(&mut stale);
    assert_eq!(stale.object_view(card).zone, Zone::Graveyard);

    act(&mut game, Action::Pass);
    let saved = game.snapshot();
    let payer = game.actor().unwrap();
    let revision = game.revision();
    let index = game
        .legal_actions()
        .iter()
        .position(|a| *a == Action::Pass)
        .unwrap();
    let mut indexed = game.clone();
    indexed.apply_index(payer, revision, index).unwrap();
    act(&mut game, Action::Pass);
    assert_eq!(game.snapshot(), indexed.snapshot());
    assert_eq!(game.state.players[owner.index()].library[0], card);
    assert_eq!(game.object_view(card).card, front);
    assert_eq!(game.private_library_top(owner, owner), Some(top));
    assert_eq!(game.private_library_top(controller, owner), None);
    assert_eq!(
        game.state.players[controller.index()].library,
        other_library
    );
    assert!(game.observe(controller).last_target_library_move.is_none());
    indexed.restore(&saved).unwrap();
    assert_eq!(
        indexed.apply_index(payer, revision, index),
        Err(Error::StaleRevision)
    );
}

#[test]
fn a_death_return_tapped_uses_the_resolving_controller_and_the_exact_graveyard_front() {
    const DEATH: CardRules = CardRules {
        effect: Effect::Creature(5, 5),
        triggers: &[TriggeredAbility {
            condition: TriggerCondition::SelfDies,
            targets: TargetPlan::NONE,
            program: &[Instruction::ReturnEventCardToBattlefieldTappedUnderControl],
        }],
        ..CardRules::EMPTY
    };
    const FRONT_RETURN: CardRules = CardRules {
        effect: Effect::Creature(3, 2),
        ..GRAVEYARD_RULES
    };
    const COPY: CardRules = CardRules {
        effect: Effect::Spell(
            TargetPlan::Single(TargetSpec::one(TargetRule::Ability)),
            &[Instruction::CopyTargetAbility { group: 0 }],
        ),
        ..CardRules::EMPTY
    };
    let front = Card::from_name(FRONT).unwrap();
    let back = front.back_face().unwrap();
    let copier = Card::from_name("Negate").unwrap();
    let _rules =
        fixture_rules::activate(&[(front, &FRONT_RETURN), (back, &DEATH), (copier, &COPY)]);
    let owner = Player::P0;
    let controller = owner.other();
    let mut game = scenario(owner);
    let card = spawn(&mut game, owner, front, Zone::Graveyard);
    activate(&mut game, card);
    settle(&mut game);
    assert_eq!(game.object_view(card).card, back);
    game.record_control(game.object_ref(card), controller, None);
    game.move_object(card, Zone::Graveyard);
    game.prepare_triggers();
    let departed = game.object_ref(card);
    let trigger = game.state.stack.last().unwrap();
    assert_eq!(trigger.controller, controller);
    let trigger_id = trigger.id;

    let mut stale = game.clone();
    stale.move_object(card, Zone::Exile);
    stale.move_object(card, Zone::Graveyard);
    settle(&mut stale);
    assert_eq!(stale.object_view(card).zone, Zone::Graveyard);

    let mut copied = game.clone();
    if copied.actor() != Some(owner) {
        act(&mut copied, Action::Pass);
    }
    let copy = spawn(&mut copied, owner, copier, Zone::Hand);
    cast(&mut copied, copy, Some(Target::Stack(trigger_id)));
    settle(&mut copied);
    settle(&mut game);
    for (branch, expected) in [(&game, controller), (&copied, owner)] {
        let returned = branch.object_view(card);
        assert_ne!(returned.reference, departed);
        assert_eq!(returned.zone, Zone::Battlefield);
        assert_eq!(returned.card, front);
        assert_eq!(returned.owner, owner);
        assert_eq!(returned.controller, expected);
        assert!(returned.tapped);
    }
}
