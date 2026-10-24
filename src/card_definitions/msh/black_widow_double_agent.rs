// Directly authored from the complete pinned Oracle and XMage implementation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    keywords: Keywords::DEATHTOUCH,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::ControlledCreatureAttacksAlone,
        targets: TargetPlan::NONE,
        program: &[Instruction::GrantKeywords {
            subject: Subject::Entering,
            keywords: Keywords::FIRST_STRIKE.union(Keywords::MENACE),
        }],
    }],
    ..CardRules::EMPTY
};
