// Direct typed behavior; full Oracle and pinned XMage dependencies reviewed.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::When {
            guard: StepGuard::Threshold {
                value: StepValue::GreatestCreaturePower,
                minimum: Some(4),
                maximum: None,
            },
            event: &TriggerCondition::SelfAttacks,
        },
        targets: TargetPlan::Single(TargetSpec::NONE),
        program: &[
            Instruction::Grow {
                subject: Subject::Source,
                power: 1,
                toughness: 0,
            },
            Instruction::GrantKeywords {
                subject: Subject::Source,
                keywords: Keywords::MENACE,
            },
        ],
    }],
    ..CardRules::EMPTY
};
