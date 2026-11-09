pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    keywords: Keywords::FLYING,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::SearchRequiredLibraryAndOrGraveyardToHand(
            SearchFilter::KindsManaValue {
                kinds: 16,
                minimum: 0,
                maximum: 2,
            },
        )],
    }],
    ..CardRules::EMPTY
};
