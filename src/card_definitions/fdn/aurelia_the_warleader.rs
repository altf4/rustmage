// Direct typed behavior from complete pinned Oracle and XMage implementations.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 4),
    keywords: Keywords::FLYING
        .union(Keywords::VIGILANCE)
        .union(Keywords::HASTE),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfFirstAttack,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::UntapPlayerCreatures(PlayerSet::Controller),
            Instruction::AdditionalCombat,
        ],
    }],
    ..CardRules::EMPTY
};
