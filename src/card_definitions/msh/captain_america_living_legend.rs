// Whole printed body and first-tap trigger from pinned Oracle/XMage, with exact object history.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 4),
    keywords: Keywords::VIGILANCE,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::ControlledCreatureFirstTapOwnTurn,
        targets: TargetPlan::NONE,
        program: &[Instruction::UntapEventObject],
    }],
    ..CardRules::EMPTY
};
