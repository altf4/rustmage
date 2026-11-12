pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 5),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::InterveningIf {
            guard: StepGuard::Expression {
                value: StatValue::SourceSuspected,
                minimum: None,
                maximum: Some(0),
            },
            event: &TriggerCondition::SelfAttacks,
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::May { skip: 1 },
            Instruction::Suspect {
                subject: Subject::Source,
            },
        ],
    }],
    ..CardRules::EMPTY
};
