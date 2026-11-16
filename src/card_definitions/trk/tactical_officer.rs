// Oracle and pinned XMage reviewed. Formidable is checked when attackers are declared.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 1),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AttackDeclared {
            minimum: 1,
            power_min: None,
            guard: StepGuard::Threshold {
                value: StepValue::TotalCreaturePower,
                minimum: Some(8),
                maximum: None,
            },
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::PutCounters {
                subject: Subject::Source,
                kind: CounterKind::PlusOne,
                amount: 1,
            },
            Instruction::UntapSource,
        ],
    }],
    ..CardRules::EMPTY
};
