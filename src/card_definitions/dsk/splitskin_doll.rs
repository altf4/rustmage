// Directly authored from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 1),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::Draw(PlayerSet::Controller, 1),
            Instruction::If {
                guard: StepGuard::Threshold {
                    value: StepValue::ControlledCreaturePowerAtMost {
                        maximum: 2,
                        other: true,
                    },
                    minimum: None,
                    maximum: Some(0),
                },
                skip: 1,
            },
            Instruction::Discard(PlayerSet::Controller, 1),
        ],
    }],
    ..CardRules::EMPTY
};
