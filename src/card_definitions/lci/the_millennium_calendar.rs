pub(in crate::cards) const RULES: CardRules = CardRules {
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
