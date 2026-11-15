// Complete Oracle federation behavior, checked against pinned XMage.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(0, 5),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::BeginCombat,
            relation: CastRelation::You,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::DrawRemembered(AmountValue::Divided {
                value: &AmountValue::Stat(StatValue::DistinctControlledCreatureTypesExcept("Borg")),
                divisor: std::num::NonZeroU16::new(5).unwrap(),
            }),
            Instruction::IfRememberedDrawsBelow {
                minimum: 3,
                skip: 1,
            },
            Instruction::PutPlayerCreatureCounters {
                players: PlayerSet::Controller,
                kind: CounterKind::PlusOne,
                amount: 2,
            },
        ],
    }],
    ..CardRules::EMPTY
};
