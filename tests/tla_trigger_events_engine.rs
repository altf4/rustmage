mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::engine::characteristic_layers::AnimationTypeMode;
use rustmage_core::{cards::*, engine::*};
use support::*;

const EXILE_CAST: TriggerCondition = TriggerCondition::SpellCast {
    relation: CastRelation::You,
    filter: CastFilter::Any,
    ordinal: None,
};
const EXILE_OBSERVER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::CastFromExile(&EXILE_CAST),
            targets: TargetPlan::NONE,
            program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
        },
        TriggeredAbility {
            condition: TriggerCondition::ControlledPermanentEntersFrom(Zone::Exile),
            targets: TargetPlan::NONE,
            program: &[Instruction::GainLife(PlayerSet::Controller, 2)],
        },
    ],
    ..CardRules::EMPTY
};
const DEATH_OBSERVER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::PermanentToGraveyard {
            kinds: 1 | 2,
            relation: CastRelation::You,
            exclude_source_kinds: 1,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
    }],
    ..CardRules::EMPTY
};
const FLYING_ATTACKS: CardRules = CardRules {
    effect: Effect::Enchantment,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::ControlledCreatureWithKeywordAttackDeclaration(
            Keywords::FLYING,
        ),
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
    }],
    ..CardRules::EMPTY
};
const POWER_ATTACKS: CardRules = CardRules {
    effect: Effect::Enchantment,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AttackPlayerWithPowerAtLeast(4),
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 2)],
    }],
    ..CardRules::EMPTY
};
const COMPANION_ATTACKS: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfAttacksWithOthers,
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 4)],
    }],
    ..CardRules::EMPTY
};

const FRONT: &str = "Brigid, Clachan's Heart";
const BACK: &str = "Brigid, Doun's Mind";
const OPTIONAL: TriggeredAbility = TriggeredAbility {
    condition: TriggerCondition::AtStep {
        step: TriggerStep::Upkeep,
        relation: CastRelation::You,
        guard: StepGuard::Always,
    },
    targets: TargetPlan::NONE,
    program: &[
        Instruction::MayTransformSource { skip: 2 },
        Instruction::TransformSource,
        Instruction::GainLife(PlayerSet::Controller, 4),
    ],
};
const OPTIONAL_FRONT: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[OPTIONAL],
    extras: &CardExtras {
        transforming_double_face: true,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const DELAYED_FRONT: CardRules = CardRules {
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::BattlefieldDeparture(DepartureFilter {
                kinds: 1,
                relation: CastRelation::You,
                other: true,
                graveyard_only: false,
                counters: DepartureCounters::Unrestricted,
            }),
            targets: TargetPlan::NONE,
            program: &[Instruction::ScheduleStep {
                step: TriggerStep::Upkeep,
                relation: CastRelation::Any,
                ability: 1,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::Reflexive,
            targets: TargetPlan::NONE,
            program: &[Instruction::TransformSource],
        },
    ],
    ..OPTIONAL_FRONT
};
const BACK_RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    ..CardRules::EMPTY
};

fn named(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn drain(game: &mut Game) {
    for _ in 0..24 {
        game.prepare_triggers();
        if game.state.stack.is_empty()
            && game.state.resolving.is_none()
            && game.state.pending == Pending::Priority
        {
            return;
        }
        choose(game, |action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. })
        });
    }
    panic!("trigger resolution did not finish");
}

fn life(game: &Game, player: Player) -> i128 {
    game.state.players[player.index()].life
}

#[test]
fn direct_exile_entry_and_casting_from_exile_are_distinct_events() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &EXILE_OBSERVER)]);
    let player = Player::P1;
    let mut game = scenario(player);
    put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let land = put(&mut game, player, "Forest", Zone::Exile);
    let creature = put(&mut game, player, "Bear Cub", Zone::Exile);
    let foreign = put(&mut game, player.other(), "Forest", Zone::Exile);

    game.move_object(land, Zone::Battlefield);
    game.move_object(foreign, Zone::Battlefield);
    drain(&mut game);
    assert_eq!(life(&game, player), 22);

    game.grant_exile_play_permission(player, game.object_ref(creature), false);
    choose(
        &mut game,
        |action| matches!(action, Action::AnnounceExile { card } if *card == creature),
    );
    pay(&mut game);
    drain(&mut game);
    assert_eq!(game.object_view(creature).zone, Zone::Battlefield);
    assert_eq!(life(&game, player), 23);
}

#[test]
fn excluding_the_source_creature_clause_preserves_its_land_clause() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &DEATH_OBSERVER)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let other = put(&mut game, player, "Forest", Zone::Battlefield);
    let plain = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    for creature in [source, other] {
        game.change_object_form(
            game.object_ref(creature),
            AnimationSpec {
                artifact: false,
                creature_types: AnimationCreatureTypes::Unchanged,
                base: Some((2, 2)),
                keywords: Keywords::NONE,
                duration: AnimationDuration::EndOfTurn,
            },
            AnimationTypeMode::AddCreatureAndLand,
        );
    }
    game.move_object(plain, Zone::Graveyard);
    game.move_object(other, Zone::Graveyard);
    game.move_object(source, Zone::Graveyard);
    drain(&mut game);
    assert_eq!(life(&game, player), 23);
}

#[test]
fn attack_predicates_capture_the_declared_cohort_and_player_defender() {
    let _rules = fixture_rules::activate(&[
        (Card::GrizzlyBears, &COMPANION_ATTACKS),
        (named("Leyline of the Void"), &FLYING_ATTACKS),
        (named("Impact Tremors"), &POWER_ATTACKS),
    ]);
    let player = Player::P0;
    for (others, power_attacks_player, expected) in
        [(true, false, 25), (true, true, 27), (false, false, 20)]
    {
        let mut game = scenario(player);
        let source = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
        put(&mut game, player, "Leyline of the Void", Zone::Battlefield);
        put(&mut game, player, "Impact Tremors", Zone::Battlefield);
        let flyer = put(&mut game, player, "Spyglass Siren", Zone::Battlefield);
        game.add_counter(flyer, CounterKind::PlusOne, 3);
        let other_flyer = put(&mut game, player, "Spyglass Siren", Zone::Battlefield);
        let walker = put(
            &mut game,
            player.other(),
            "Ajani, Caller of the Pride",
            Zone::Battlefield,
        );
        game.add_counter(walker, CounterKind::Loyalty, 4);
        game.state.step = Step::Attackers;
        game.state.pending = Pending::Attackers;
        act(&mut game, Action::Attack { card: source });
        if others {
            let defender = game.object_ref(walker);
            act(
                &mut game,
                if power_attacks_player {
                    Action::Attack { card: flyer }
                } else {
                    Action::AttackPlaneswalker {
                        card: flyer,
                        defender,
                    }
                },
            );
            act(
                &mut game,
                Action::AttackPlaneswalker {
                    card: other_flyer,
                    defender,
                },
            );
        }
        act(&mut game, Action::FinishAttackers);
        drain(&mut game);
        assert_eq!(life(&game, player), expected);
    }
}

#[test]
fn optional_transform_rewards_only_the_accepted_live_incarnation() {
    let _rules =
        fixture_rules::activate(&[(named(FRONT), &OPTIONAL_FRONT), (named(BACK), &BACK_RULES)]);
    let player = Player::P0;
    for (departed, accept) in [(false, false), (false, true), (true, true)] {
        let mut game = scenario(player);
        let source = put(&mut game, player, FRONT, Zone::Battlefield);
        game.begin_step(Step::Upkeep);
        game.prepare_triggers();
        if departed {
            game.move_object(source, Zone::Exile);
            game.move_object(source, Zone::Battlefield);
        }
        act(&mut game, Action::Pass);
        act(&mut game, Action::Pass);
        if !departed {
            act(&mut game, Action::ChooseOptional { accept });
        }
        drain(&mut game);
        let transformed = accept && !departed;
        assert_eq!(life(&game, player), if transformed { 24 } else { 20 });
        assert_eq!(
            game.object_view(source).card,
            named(if transformed { BACK } else { FRONT }),
        );
    }
}

#[test]
fn multiple_next_upkeep_callbacks_share_the_actual_stack_transform_serial() {
    let _rules =
        fixture_rules::activate(&[(named(FRONT), &DELAYED_FRONT), (named(BACK), &BACK_RULES)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, FRONT, Zone::Battlefield);
    let first = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let second = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    game.move_object(first, Zone::Hand);
    game.move_object(second, Zone::Exile);
    drain(&mut game);
    assert_eq!(game.object_view(source).card, named(FRONT));

    game.state.active = player.other();
    game.state.priority = player.other();
    game.begin_step(Step::Upkeep);
    drain(&mut game);
    assert_eq!(game.object_view(source).card, named(BACK));
    assert_eq!(game.transform_serial(game.object_ref(source)), 1);
}
