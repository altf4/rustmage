pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 4),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::CardsLeaveGraveyardKinds(1),
        targets: TargetPlan::NONE,
        program: &[
            Instruction::Grow {
                subject: Subject::Source,
                power: 1,
                toughness: 0,
            },
            Instruction::GrantKeywords {
                subject: Subject::Source,
                keywords: Keywords::MENACE.union(Keywords::LIFELINK),
            },
        ],
    }],
    ..CardRules::EMPTY
};
