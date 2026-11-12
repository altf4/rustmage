pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    keywords: Keywords::FLYING,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfDies,
        targets: TargetPlan::NONE,
        program: &[Instruction::ExileDepartedSourceCollectEvidenceReturnTapped(
            4,
        )],
    }],
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::FLYING)],
    }],
    ..CardRules::EMPTY
};
