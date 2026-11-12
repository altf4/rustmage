pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::When {
            guard: StepGuard::Expression {
                value: StatValue::CONTROLLER_GRAVEYARD_CARD_TYPES,
                minimum: Some(4),
                maximum: None,
            },
            event: &TriggerCondition::SelfAttacks,
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::Grow {
                subject: Subject::Source,
                power: 2,
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
