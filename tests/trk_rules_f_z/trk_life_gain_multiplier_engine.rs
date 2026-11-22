//! Dr. Crusher doubles only triggers caused by her controller gaining life.
use crate::support::*;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};

const CRUSHER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::LifeGained {
            first_only: false,
            own_turn: false,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::PutCounters {
            subject: Subject::Source,
            kind: CounterKind::PlusOne,
            amount: 1,
        }],
    }],
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::AdditionalTriggers {
            scope: AdditionalTriggerScope::ControlledPermanentLifeGain,
            condition: StatValue::Fixed(1),
        }],
    }],
    ..CardRules::EMPTY
};
const OBSERVER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::LifeGained {
                first_only: false,
                own_turn: false,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::PutCounters {
                subject: Subject::Source,
                kind: CounterKind::PlusOne,
                amount: 1,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::LifeLost {
                relation: CastRelation::You,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::PutCounters {
                subject: Subject::Source,
                kind: CounterKind::PlusOne,
                amount: 1,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::SpellCast {
                relation: CastRelation::You,
                filter: CastFilter::Any,
                ordinal: None,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::PutCounters {
                subject: Subject::Source,
                kind: CounterKind::PlusOne,
                amount: 1,
            }],
        },
    ],
    ..CardRules::EMPTY
};
const LIMITED: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::OncePerTurn(&TriggerCondition::LifeGained {
            first_only: false,
            own_turn: false,
        }),
        targets: TargetPlan::NONE,
        program: &[Instruction::PutCounters {
            subject: Subject::Source,
            kind: CounterKind::PlusOne,
            amount: 1,
        }],
    }],
    ..CardRules::EMPTY
};
const GAIN: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::GainLife(PlayerSet::Controller, 2)],
    ),
    ..CardRules::EMPTY
};
const LOSS: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::LoseLife(PlayerSet::Controller, 1)],
    ),
    ..CardRules::EMPTY
};

fn settle_triggers(game: &mut Game) {
    for _ in 0..96 {
        if game.state.stack.is_empty()
            && game.state.resolving.is_none()
            && game.state.pending == Pending::Priority
            && game.state.pending_triggers.is_empty()
            && game.state.ordering_triggers.is_empty()
        {
            return;
        }
        choose(game, |action| {
            matches!(action, Action::PutTrigger { .. } | Action::Pass)
        });
    }
    panic!("trigger ordering did not finish");
}

#[test]
fn life_gain_cause_doubles_controlled_permanent_triggers_only() {
    let _rules = fixture_rules::activate(&[
        (Card::from_name("Bear Cub").unwrap(), &CRUSHER),
        (Card::from_name("Grizzly Bears").unwrap(), &OBSERVER),
        (Card::from_name("Hill Giant").unwrap(), &LIMITED),
        (Card::from_name("Opt").unwrap(), &GAIN),
        (Card::from_name("Duress").unwrap(), &LOSS),
    ]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let crusher = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let observer = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
        let enemy_observer = put(
            &mut game,
            player.other(),
            "Grizzly Bears",
            Zone::Battlefield,
        );
        let limited = put(&mut game, player, "Hill Giant", Zone::Battlefield);
        let spell = put(&mut game, player, "Opt", Zone::Hand);
        cast(&mut game, spell, None);
        settle_triggers(&mut game);
        assert_eq!(game.counter_count(crusher, CounterKind::PlusOne), 2);
        assert_eq!(game.counter_count(observer, CounterKind::PlusOne), 3);
        assert_eq!(game.counter_count(limited, CounterKind::PlusOne), 1);
        assert_eq!(game.counter_count(enemy_observer, CounterKind::PlusOne), 0);
        assert_eq!(game.state.players[player.index()].life, 22);

        let loss = put(&mut game, player, "Duress", Zone::Hand);
        cast(&mut game, loss, None);
        settle_triggers(&mut game);
        assert_eq!(game.counter_count(crusher, CounterKind::PlusOne), 2);
        assert_eq!(game.counter_count(observer, CounterKind::PlusOne), 5);
        assert_eq!(game.counter_count(limited, CounterKind::PlusOne), 1);

        game.commit_life_gain(player.other(), 2);
        settle_triggers(&mut game);
        assert_eq!(game.counter_count(enemy_observer, CounterKind::PlusOne), 1);
        assert_eq!(game.counter_count(observer, CounterKind::PlusOne), 5);

        let checkpoint = game.snapshot();
        let mut restored = game.clone();
        restored.restore(&checkpoint).unwrap();
        let later = put(&mut restored, player, "Opt", Zone::Hand);
        restored.remove_abilities_exact(
            &[restored.object_ref(crusher)],
            restored.object_ref(crusher),
            player,
            AnimationDuration::EndOfTurn,
        );
        cast(&mut restored, later, None);
        settle_triggers(&mut restored);
        assert_eq!(restored.counter_count(crusher, CounterKind::PlusOne), 2);
        assert_eq!(restored.counter_count(observer, CounterKind::PlusOne), 7);
        assert_eq!(restored.counter_count(limited, CounterKind::PlusOne), 1);
    }
}
