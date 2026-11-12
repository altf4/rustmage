// Directly authored from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 4),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::When {
            guard: StepGuard::Expression {
                value: StatValue::CONTROLLER_GRAVEYARD_CARD_TYPES,
                minimum: Some(4),
                maximum: None,
            },
            event: &TriggerCondition::SelfEntersOrAttacks,
        },
        targets: TargetPlan::Single(TargetSpec {
            rule: TargetRule::Creature,
            min: 1,
            max: 2,
        }),
        program: &[
            Instruction::DistributeCounters {
                total: 2,
                kind: CounterKind::PlusOne,
            },
            Instruction::If {
                guard: StepGuard::Expression {
                    value: StatValue::CONTROLLER_GRAVEYARD_CARD_TYPES,
                    minimum: Some(6),
                    maximum: None,
                },
                skip: 1,
            },
            Instruction::DoubleCounterKind {
                subject: Subject::Targets,
                kind: CounterKind::PlusOne,
            },
        ],
    }],
    ..CardRules::EMPTY
};
