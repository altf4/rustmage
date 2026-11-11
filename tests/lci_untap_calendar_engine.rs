//! Untap step receipts and counter state triggers use completed rules events.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const CALENDAR: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::OwnUntapStepUntaps,
            targets: TargetPlan::NONE,
            program: &[Instruction::PutCountersScaled {
                subject: Subject::Source,
                kind: CounterKind::Time,
                amount: AmountValue::UntapStepCount,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::StateCounterAtLeast {
                kind: CounterKind::Time,
                minimum: 1000,
            },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::SacrificeAbilitySource,
                Instruction::LoseLife(PlayerSet::Opponent, 1000),
            ],
        },
    ],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        cycling: false,
        origin: ActivationOrigin::Battlefield,
        cost: ActivationCost {
            generic: 2,
            tap: true,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::DoubleCounterKind {
            subject: Subject::Source,
            kind: CounterKind::Time,
        }],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
const OTHER_STEP: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::UntapDuringOtherUntapSteps],
    }],
    ..CardRules::EMPTY
};
const LOCKED: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::SelfUntap(UntapRestriction::All)],
    }],
    ..CardRules::EMPTY
};
const UPKEEP: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::Upkeep,
            relation: CastRelation::Any,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
    }],
    ..CardRules::EMPTY
};
const COPY: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Ability)),
        &[Instruction::CopyTargetAbility { group: 0 }],
    ),
    ..CardRules::EMPTY
};
const PROLIFERATE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::Proliferate(AmountValue::Fixed(1))],
    ),
    ..CardRules::EMPTY
};

fn named(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn pass_priority(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn tap(game: &mut Game, ids: &[CardId]) {
    for &id in ids {
        game.state.objects[id as usize].tapped = true;
    }
}

fn time(game: &Game, id: CardId) -> u64 {
    game.counter_count(id, CounterKind::Time)
}

fn state_trigger(game: &Game, source: CardId) -> u64 {
    let mut items = game.state.stack.iter().filter(|item| {
        !item.is_copy
            && item.kind == (StackKind::TriggeredAbility { ability: 1 })
            && item.source.is_some_and(|s| s.id == source)
    });
    let id = items.next().expect("one state trigger").id;
    assert!(items.next().is_none());
    id
}

#[test]
fn actual_untaps_are_one_batch_and_wait_to_order_with_upkeep() {
    let _rules = fixture_rules::activate(&[
        (named("Mazemind Tome"), &CALENDAR),
        (named("Grizzly Bears"), &OTHER_STEP),
        (named("Short Bow"), &LOCKED),
        (named("Impact Tremors"), &UPKEEP),
    ]);
    for player in [Player::P0, Player::P1] {
        let other = player.other();
        let mut root = scenario(other);
        let calendar = put(&mut root, player, "Mazemind Tome", Zone::Battlefield);
        let inactive_calendar = put(&mut root, other, "Mazemind Tome", Zone::Battlefield);
        let land = put(&mut root, player, "Forest", Zone::Battlefield);
        let foreign_land = put(&mut root, other, "Forest", Zone::Battlefield);
        let stunned = put(&mut root, player, "Healer's Hawk", Zone::Battlefield);
        let locked = put(&mut root, player, "Short Bow", Zone::Battlefield);
        let infantry = put(&mut root, other, "Grizzly Bears", Zone::Battlefield);
        let upkeep = put(&mut root, player, "Impact Tremors", Zone::Battlefield);
        let foreign_upkeep = put(&mut root, other, "Impact Tremors", Zone::Battlefield);
        tap(
            &mut root,
            &[
                calendar,
                inactive_calendar,
                land,
                foreign_land,
                stunned,
                locked,
                infantry,
            ],
        );
        for id in [stunned, locked] {
            root.add_counter(id, CounterKind::Stun, 1);
        }
        root.state.step = Step::End;
        act(&mut root, Action::Pass);
        let revision = root.revision();
        let index = root
            .legal_actions()
            .iter()
            .position(|a| *a == Action::Pass)
            .unwrap();
        let saved = root.snapshot();
        let mut raw = root.clone();
        let mut indexed = root.clone();
        drop(root);

        act(&mut raw, Action::Pass);
        indexed.apply_index(player, revision, index).unwrap();
        assert_eq!(raw.snapshot(), indexed.snapshot());
        assert_eq!(raw.state.step, Step::Upkeep);
        assert_eq!(raw.state.pending, Pending::OrderTriggers { player });
        assert!(raw.state.stack.is_empty());
        for id in [calendar, land, infantry] {
            assert!(!raw.object_view(id).tapped);
        }
        for id in [inactive_calendar, foreign_land, stunned, locked] {
            assert!(raw.object_view(id).tapped);
        }
        assert_eq!(raw.counter_count(stunned, CounterKind::Stun), 0);
        assert_eq!(raw.counter_count(locked, CounterKind::Stun), 1);
        assert_eq!(time(&raw, calendar), 0);
        let pending = &raw.state.ordering_triggers;
        assert_eq!(pending.len(), 3);
        let batch = pending
            .iter()
            .find(|item| item.source.is_some_and(|s| s.id == calendar))
            .unwrap();
        assert_eq!(
            batch.event_object.as_ref().unwrap().facts,
            EventObjectFacts::UntapStepCount(2)
        );
        let calendar_trigger = batch.id;

        indexed.restore(&saved).unwrap();
        assert_eq!(
            indexed.apply_index(player, revision, index),
            Err(Error::StaleRevision)
        );
        act(&mut indexed, Action::Pass);
        assert_eq!(raw.snapshot(), indexed.snapshot());

        act(
            &mut raw,
            Action::PutTrigger {
                id: calendar_trigger,
            },
        );
        let order: Vec<_> = raw
            .state
            .stack
            .iter()
            .map(|item| item.source.unwrap().id)
            .collect();
        assert_eq!(order, [calendar, upkeep, foreign_upkeep]);
        settle(&mut raw);
        assert_eq!(time(&raw, calendar), 2);
        assert_eq!(time(&raw, inactive_calendar), 0);
    }
}

#[test]
fn effect_untaps_and_other_players_untaps_do_not_start_a_calendar_batch() {
    let _rules = fixture_rules::activate(&[
        (named("Mazemind Tome"), &CALENDAR),
        (named("Grizzly Bears"), &OTHER_STEP),
    ]);
    for player in [Player::P0, Player::P1] {
        let other = player.other();
        let mut game = scenario(other);
        let calendar = put(&mut game, player, "Mazemind Tome", Zone::Battlefield);
        let land = put(&mut game, player, "Forest", Zone::Battlefield);
        let eligible = put(&mut game, other, "Grizzly Bears", Zone::Battlefield);
        let stunned = put(&mut game, other, "Grizzly Bears", Zone::Battlefield);
        let suppressed = put(&mut game, other, "Grizzly Bears", Zone::Battlefield);
        let blanked = game.object_ref(suppressed);
        tap(&mut game, &[land, eligible, stunned, suppressed]);
        game.add_counter(stunned, CounterKind::Stun, 1);
        game.remove_abilities_exact(&[blanked], blanked, other, AnimationDuration::Permanent);

        game.untap_objects(&[land], untap_rules::UntapCause::Effect);
        assert!(game.state.pending_triggers.is_empty());
        game.state.step = Step::End;
        pass_priority(&mut game);

        assert_eq!(game.state.step, Step::Upkeep);
        assert!(game.state.stack.is_empty());
        assert!(!game.object_view(eligible).tapped);
        assert!(game.object_view(stunned).tapped);
        assert_eq!(game.counter_count(stunned, CounterKind::Stun), 0);
        assert!(game.object_view(suppressed).tapped);
        assert_eq!(time(&game, calendar), 0);
    }
}

#[test]
fn doubling_time_latches_one_trigger_that_survives_lowering_the_count_and_source_departure() {
    let _rules = fixture_rules::activate(&[(named("Mazemind Tome"), &CALENDAR)]);
    for player in [Player::P0, Player::P1] {
        let other = player.other();
        let mut game = scenario(player);
        game.state.players[other.index()].life = 2000;
        let calendar = put(&mut game, player, "Mazemind Tome", Zone::Battlefield);
        let old = game.object_ref(calendar);
        game.add_counter(calendar, CounterKind::Time, 500);
        game.add_counter(calendar, CounterKind::Charge, 7);
        act(
            &mut game,
            Action::AnnounceActivate {
                card: calendar,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut game);
        pass_priority(&mut game);
        let trigger = state_trigger(&game, calendar);
        assert_eq!(time(&game, calendar), 1000);
        assert_eq!(game.counter_count(calendar, CounterKind::Charge), 7);
        assert!(game.object_view(calendar).tapped);
        game.prepare_triggers();
        assert_eq!(state_trigger(&game, calendar), trigger);

        game.pay_counters(calendar, &Counters::of(CounterKind::Time, 1000));
        let mut reentered = game.clone();
        reentered.move_object(calendar, Zone::Graveyard);
        reentered.move_object(calendar, Zone::Battlefield);
        assert_ne!(reentered.object_ref(calendar), old);
        settle(&mut game);
        settle(&mut reentered);
        assert_eq!(game.object_view(calendar).zone, Zone::Graveyard);
        assert_eq!(reentered.object_view(calendar).zone, Zone::Battlefield);
        for branch in [&game, &reentered] {
            assert_eq!(branch.state.players[other.index()].life, 1000);
        }
    }
}

#[test]
fn countering_the_original_retriggers_before_tidebinder_blanks_the_source() {
    let _rules =
        fixture_rules::activate(&[(named("Mazemind Tome"), &CALENDAR), (named("Opt"), &COPY)]);
    for player in [Player::P0, Player::P1] {
        let other = player.other();
        let mut game = scenario(player);
        game.state.players[other.index()].life = 3000;
        let calendar = put(&mut game, player, "Mazemind Tome", Zone::Battlefield);
        game.add_counter(calendar, CounterKind::Time, 1000);
        game.prepare_triggers();
        let original = state_trigger(&game, calendar);
        let copy_spell = put(&mut game, player, "Opt", Zone::Hand);
        cast(&mut game, copy_spell, Some(Target::Stack(original)));
        pass_priority(&mut game);
        assert_eq!(game.state.stack.len(), 2);
        let copied = game.state.stack.last().unwrap().id;
        assert!(game.state.stack.last().unwrap().is_copy);

        assert_eq!(game.actor(), Some(player));
        act(&mut game, Action::Pass);
        let binder = put(&mut game, other, "Tishana's Tidebinder", Zone::Hand);
        cast(&mut game, binder, None);
        pass_priority(&mut game);
        act(
            &mut game,
            Action::ChooseTarget {
                target: Target::Stack(original),
            },
        );
        if game.legal_actions().contains(&Action::FinishTargets) {
            act(&mut game, Action::FinishTargets);
        }
        pass_priority(&mut game);

        assert!(!game.printed_abilities_available(calendar));
        assert_eq!(game.state.stack.len(), 2);
        assert_eq!(game.state.stack[0].id, copied);
        assert_ne!(state_trigger(&game, calendar), original);
        settle(&mut game);
        assert_eq!(game.object_view(calendar).zone, Zone::Graveyard);
        assert_eq!(game.state.players[other.index()].life, 1000);
    }
}

#[test]
fn proliferating_to_the_threshold_captures_the_actual_counter_event() {
    let _rules = fixture_rules::activate(&[
        (named("Mazemind Tome"), &CALENDAR),
        (named("Opt"), &PROLIFERATE),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);
    game.state.players[player.other().index()].life = 2000;
    let calendar = put(&mut game, player, "Mazemind Tome", Zone::Battlefield);
    game.add_counter(calendar, CounterKind::Time, 999);
    let spell = put(&mut game, player, "Opt", Zone::Hand);
    cast(&mut game, spell, None);
    pass_priority(&mut game);
    let object = game.object_ref(calendar);
    act(&mut game, Action::ProliferatePermanent { object });
    act(&mut game, Action::FinishProliferate);
    assert_eq!(time(&game, calendar), 1000);
    state_trigger(&game, calendar);
    settle(&mut game);
    assert_eq!(game.object_view(calendar).zone, Zone::Graveyard);
    assert_eq!(game.state.players[player.other().index()].life, 1000);
}

#[test]
fn a_state_ability_is_captured_before_a_later_instruction_destroys_its_source() {
    const ARTIFACT: TargetSpec = TargetSpec::one(TargetRule::Object {
        zone: TargetZone::Battlefield,
        kinds: 16,
        relation: -1,
        other: false,
        nonland: false,
        tapped: false,
        min_mana: 0,
        max_mana: u32::MAX,
    });
    const COPY_AND_DESTROY: CardRules = CardRules {
        effect: Effect::Spell(
            TargetPlan::Groups(&[ARTIFACT, ARTIFACT]),
            &[
                Instruction::BecomeCopy(InPlaceCopySpec {
                    recipient_group: Some(0),
                    blueprint_group: 1,
                    overrides: CopyOverrides {
                        name: None,
                        legendary: None,
                        kinds: None,
                        types: None,
                        keywords: Keywords::NONE,
                    },
                    add_kinds: 0,
                    base: None,
                    next_turn: false,
                }),
                Instruction::SelectGroup(0),
                Instruction::Destroy,
            ],
        ),
        ..CardRules::EMPTY
    };
    let _rules = fixture_rules::activate(&[
        (named("Mazemind Tome"), &CALENDAR),
        (named("Opt"), &COPY_AND_DESTROY),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);
    game.state.players[player.other().index()].life = 2000;
    let model = put(&mut game, player, "Mazemind Tome", Zone::Battlefield);
    let recipient = put(&mut game, player, "Short Bow", Zone::Battlefield);
    game.add_counter(recipient, CounterKind::Time, 1000);
    let spell = put(&mut game, player, "Opt", Zone::Hand);
    announce(
        &mut game,
        spell,
        &[Target::Card(recipient), Target::Card(model)],
    );
    pass_priority(&mut game);
    assert_eq!(game.object_view(recipient).zone, Zone::Graveyard);
    state_trigger(&game, recipient);
    assert!(game.state.stack[0].source_ability_origin.is_some());
    settle(&mut game);
    assert_eq!(game.state.players[player.other().index()].life, 1000);
}
