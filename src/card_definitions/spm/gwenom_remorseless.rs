pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    keywords: Keywords::DEATHTOUCH.union(Keywords::LIFELINK),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfAttacks,
        targets: TargetPlan::NONE,
        program: &[Instruction::GrantLibraryTopLifePlayUntilEndTurn],
    }],
    ..CardRules::EMPTY
};
