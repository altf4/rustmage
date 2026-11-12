pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 5),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Eerie,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::Grow {
                subject: Subject::Source,
                power: 2,
                toughness: 0,
            },
            Instruction::GrantKeywords {
                subject: Subject::Source,
                keywords: Keywords::LIFELINK,
            },
        ],
    }],
    ..CardRules::EMPTY
};
