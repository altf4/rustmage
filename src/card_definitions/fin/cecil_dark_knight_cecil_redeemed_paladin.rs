pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    keywords: Keywords::DEATHTOUCH,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SourceDealtDamage,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::LoseLifeScaled(PlayerSet::Controller, AmountValue::DamageEventAmount),
            Instruction::If {
                guard: StepGuard::Expression {
                    value: StatValue::ControllerLifeAtMostHalfStarting,
                    minimum: Some(1),
                    maximum: None,
                },
                skip: 2,
            },
            Instruction::UntapSource,
            Instruction::TransformSource,
        ],
    }],
    extras: &CardExtras {
        transforming_double_face: true,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
