pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 5),
    keywords: Keywords::TRAMPLE,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::TRAMPLE)],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Expend(4),
        targets: TargetPlan::NONE,
        program: &[Instruction::Grow {
            subject: Subject::Source,
            power: 2,
            toughness: 1,
        }],
    }],
    ..CardRules::EMPTY
};
