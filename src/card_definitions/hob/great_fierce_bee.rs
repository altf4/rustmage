pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    keywords: Keywords::FLYING,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::FLYING)],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::CreatureDiesBatch {
            filter: CreatureFilter {
                relation: -1,
                other: true,
                subtype: None,
                keyword: Keywords::NONE,
                attacking: false,
                tokens: None,
                artifact: false,
            },
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::Scry(1)],
    }],
    ..CardRules::EMPTY
};
