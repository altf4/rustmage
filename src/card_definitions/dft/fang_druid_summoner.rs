// Optional zone choices preserve reveal/shuffle rules and the authored abilityless fact.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 4),
    keywords: Keywords::REACH,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::SearchLibraryAndOrGraveyardToHand(
            SearchFilter::AllOf(&[SearchFilter::Creature, SearchFilter::NoAbilities]),
        )],
    }],
    ..CardRules::EMPTY
};
