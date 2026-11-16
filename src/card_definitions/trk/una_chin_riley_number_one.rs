// Oracle and pinned XMage reviewed. Formidable is checked when attackers are declared.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 5),
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
        targets: one(TargetRule::Filtered {
            base: &TargetRule::Creature,
            predicate: &TargetPredicate::Attacking,
        }),
        program: &[
            Instruction::Grow {
                subject: Subject::Targets,
                power: 2,
                toughness: 0,
            },
            Instruction::GrantKeywords {
                subject: Subject::Targets,
                keywords: Keywords::FIRST_STRIKE,
            },
        ],
    }],
    ..CardRules::EMPTY
};
