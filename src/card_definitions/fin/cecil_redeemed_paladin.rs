pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    keywords: Keywords::LIFELINK,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfAttacks,
        targets: TargetPlan::NONE,
        program: &[Instruction::GrantCreatures {
            filter: CreatureFilter {
                relation: -1,
                other: true,
                subtype: None,
                keyword: Keywords::NONE,
                attacking: true,
                tokens: None,
                artifact: false,
            },
            keywords: Keywords::INDESTRUCTIBLE,
        }],
    }],
    ..CardRules::EMPTY
};
