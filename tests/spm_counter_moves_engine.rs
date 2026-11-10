//! Counter transfers and active-player resolution payments use the replacement scheduler.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const TRANSFER: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            tap: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 1,
            relation: 0,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        })),
        program: &[Instruction::MoveCountersFromSource {
            kind: CounterKind::PlusOne,
            amount: AmountValue::Fixed(1),
        }],
        sorcery: true,
        once_per_turn: false,
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::When {
            event: &TriggerCondition::OtherCreatureCounterPlacedByYou {
                kind: CounterKind::PlusOne,
            },
            guard: StepGuard::Expression {
                value: StatValue::SourceCounters(CounterKind::PlusOne),
                minimum: Some(1),
                maximum: Some(1),
            },
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 2)],
    }],
    ..CardRules::EMPTY
};
const CROWD: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::Main1,
            relation: CastRelation::Any,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::IfSourceCanReceiveCounters { skip: 3 },
            Instruction::MayForPlayer {
                player: PlayerSet::Active,
                skip: 2,
            },
            Instruction::PaySourceCounterPlacement {
                player: PlayerSet::Active,
                kind: CounterKind::PlusOne,
                amount: 1,
            },
            Instruction::AddColorlessScaledFor(PlayerSet::Active, AmountValue::SourceTotalCounters),
        ],
    }],
    ..CardRules::EMPTY
};

const PROHIBIT: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::CounterProhibition(1)],
    }],
    ..CardRules::EMPTY
};

const fn replacement(factor: u32, add: u32) -> ReplacementSpec {
    ReplacementSpec::Counters {
        filter: ReplacementFilter::Creature,
        kind: Some(CounterKind::PlusOne),
        factor,
        add,
        effects_only: false,
    }
}

const ZERO: ReplacementSpec = replacement(0, 0);
const INCREASE: ReplacementSpec = replacement(1, 1);
const BLOCK_PLACEMENT: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::Replacement(0, ZERO)],
    }],
    extras: &CardExtras {
        replacement_effects: &[ZERO],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const ADD_COUNTER: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::Replacement(0, INCREASE)],
    }],
    extras: &CardExtras {
        replacement_effects: &[INCREASE],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

fn definition(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn activate_transfer(game: &mut Game, source: CardId, recipient: CardId) {
    let target = Some(Target::Card(recipient));
    let action = Action::Activate {
        card: source,
        ability: 0,
        target,
        payment: [0; 6],
        counter_payment: Counters::default(),
    };
    if game.legal_actions().contains(&action) {
        act(game, action);
    } else {
        choose(
            game,
            |a| matches!(a, Action::AnnounceActivate { card, ability: 0, .. } if *card == source),
        );
        act(
            game,
            Action::ChooseTarget {
                target: target.unwrap(),
            },
        );
        if game.legal_actions().contains(&Action::FinishTargets) {
            act(game, Action::FinishTargets);
        }
        pay(game);
    }
}

fn transfer_scenario() -> (Game, CardId, CardId) {
    let mut game = scenario(Player::P0);
    let source = put(&mut game, Player::P0, "Short Bow", Zone::Battlefield);
    let recipient = put(&mut game, Player::P0, "Savannah Lions", Zone::Battlefield);
    game.add_counter(source, CounterKind::PlusOne, 2);
    (game, source, recipient)
}

fn counter_count(game: &Game, id: CardId) -> u64 {
    game.counter_count(id, CounterKind::PlusOne)
}

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

#[test]
fn transfer_keeps_the_source_untouched_through_replacement_ordering() {
    let add = definition("Phyrexian Arena");
    let _rules =
        fixture_rules::activate(&[(definition("Short Bow"), &TRANSFER), (add, &ADD_COUNTER)]);
    let (mut game, source, recipient) = transfer_scenario();
    let add = spawn(&mut game, Player::P0, add, Zone::Battlefield);
    let double = put(&mut game, Player::P0, "Doubling Season", Zone::Battlefield);
    activate_transfer(&mut game, source, recipient);
    pass_pair(&mut game);
    assert_eq!(
        game.state.pending,
        Pending::Replacement { player: Player::P0 }
    );
    assert_eq!(counter_count(&game, source), 2);
    assert_eq!(counter_count(&game, recipient), 0);

    let saved = game.snapshot();
    for (replacement, ability, expected) in [(add, 0, 4), (double, 1, 3)] {
        game.restore(&saved).unwrap();
        let action = Action::ChooseReplacement {
            source: game.object_ref(replacement),
            ability,
        };
        let mut indexed = game.clone();
        let index = indexed
            .legal_actions()
            .iter()
            .position(|a| *a == action)
            .unwrap();
        indexed
            .apply_index(Player::P0, indexed.revision(), index)
            .unwrap();
        act(&mut game, action);
        assert_eq!(game.snapshot(), indexed.snapshot());
        assert_eq!(counter_count(&game, source), 1);
        assert_eq!(counter_count(&game, recipient), expected);
        settle(&mut game);
        assert_eq!(game.state.players[0].life, 22);
    }
}

#[test]
fn prohibited_or_zero_placement_and_stale_incarnations_leave_the_source_unpaid() {
    for prevented in [&PROHIBIT, &BLOCK_PLACEMENT] {
        let _rules = fixture_rules::activate(&[
            (definition("Short Bow"), &TRANSFER),
            (definition("Phyrexian Arena"), prevented),
        ]);
        let (mut game, source, recipient) = transfer_scenario();
        put(&mut game, Player::P0, "Phyrexian Arena", Zone::Battlefield);
        activate_transfer(&mut game, source, recipient);
        settle(&mut game);
        assert_eq!(counter_count(&game, source), 2);
        assert_eq!(counter_count(&game, recipient), 0);
        assert_eq!(game.state.players[0].life, 20);
    }

    let _rules = fixture_rules::activate(&[(definition("Short Bow"), &TRANSFER)]);
    let (mut game, source, recipient) = transfer_scenario();
    activate_transfer(&mut game, source, recipient);
    let saved = game.snapshot();
    for moved in [source, recipient] {
        game.restore(&saved).unwrap();
        let original = game.object_ref(moved);
        game.move_object(moved, Zone::Exile);
        game.move_object(moved, Zone::Battlefield);
        assert_ne!(game.object_ref(moved), original);
        if moved == source {
            game.add_counter(source, CounterKind::PlusOne, 2);
        }
        settle(&mut game);
        assert_eq!(counter_count(&game, source), 2);
        assert_eq!(counter_count(&game, recipient), 0);
    }
}

fn crowd_scenario(active: Player) -> (Game, CardId) {
    let mut game = scenario(active);
    let source = put(&mut game, Player::P0, "Grizzly Bears", Zone::Battlefield);
    game.add_counter(source, CounterKind::PlusOne, 1);
    game.add_counter(source, CounterKind::Stun, 2);
    game.add_counter(source, CounterKind::Charge, 3);
    for player in &mut game.state.players {
        player.mana = [0; 6];
    }
    game.state.step = Step::Draw;
    (game, source)
}

#[test]
fn the_active_player_owns_the_optional_cost_and_gets_mana_for_every_counter() {
    let _rules = fixture_rules::activate(&[
        (Card::GrizzlyBears, &CROWD),
        (definition("Phyrexian Arena"), &ADD_COUNTER),
    ]);
    for active in [Player::P0, Player::P1] {
        let (mut game, source) = crowd_scenario(active);
        put(&mut game, Player::P0, "Doubling Season", Zone::Battlefield);
        if active == Player::P1 {
            put(&mut game, Player::P0, "Phyrexian Arena", Zone::Battlefield);
        }
        pass_pair(&mut game);
        assert_eq!(game.state.step, Step::Main1);
        assert_eq!(game.state.stack.last().unwrap().controller, Player::P0);
        pass_pair(&mut game);
        assert!(
            matches!(game.state.pending, Pending::EffectOptional { player, .. } if player == active)
        );
        let saved = game.snapshot();
        let accept = Action::ChooseOptional { accept: true };
        assert_eq!(
            game.apply(active.other(), accept.clone()),
            Err(Error::WrongActor)
        );
        assert_eq!(game.snapshot(), saved);
        act(&mut game, Action::ChooseOptional { accept: false });
        assert_eq!(counter_count(&game, source), 1);
        assert_eq!(game.state.players[active.index()].mana, [0; 6]);

        game.restore(&saved).unwrap();
        act(&mut game, accept);
        settle(&mut game);
        // Doubling Season cannot affect the original cost. If another effect
        // changes it first, that replacement enables Doubling Season (CR 614.16).
        let expected_plus_one = if active == Player::P0 { 2 } else { 5 };
        assert_eq!(counter_count(&game, source), expected_plus_one);
        assert_eq!(
            game.state.players[active.index()].mana,
            [0, 0, 0, 0, 0, (expected_plus_one + 5).try_into().unwrap()]
        );
        assert_eq!(game.state.players[active.other().index()].mana, [0; 6]);
        assert!(
            game.observe(active)
                .counter_placement_history
                .iter()
                .any(|event| {
                    event.object == game.object_ref(source)
                        && event.placed_by == active
                        && event.plus_one
                })
        );
    }
}

#[test]
fn the_optional_cost_rejects_impossible_payment_but_survives_a_zero_replacement() {
    for (provider, payable) in [
        (None, false),
        (Some(&PROHIBIT), false),
        (Some(&BLOCK_PLACEMENT), true),
    ] {
        let mut definitions = vec![(Card::GrizzlyBears, &CROWD)];
        if let Some(provider) = provider {
            definitions.push((definition("Phyrexian Arena"), provider));
        }
        let _rules = fixture_rules::activate(&definitions);
        let (mut game, source) = crowd_scenario(Player::P1);
        if provider.is_some() {
            put(&mut game, Player::P0, "Phyrexian Arena", Zone::Battlefield);
        }
        pass_pair(&mut game);
        if provider.is_none() {
            game.move_object(source, Zone::Exile);
            game.move_object(source, Zone::Battlefield);
            game.add_counter(source, CounterKind::PlusOne, 1);
        }
        pass_pair(&mut game);
        assert_eq!(
            matches!(game.state.pending, Pending::EffectOptional { .. }),
            payable
        );
        if payable {
            act(&mut game, Action::ChooseOptional { accept: true });
            assert_eq!(game.state.players[1].mana, [0, 0, 0, 0, 0, 6]);
            assert!(
                game.observe(Player::P1)
                    .counter_placement_history
                    .is_empty()
            );
        } else {
            reject(&mut game, Action::ChooseOptional { accept: true });
            assert_eq!(game.state.players[1].mana, [0; 6]);
        }
        assert_eq!(counter_count(&game, source), 1);
    }
}
