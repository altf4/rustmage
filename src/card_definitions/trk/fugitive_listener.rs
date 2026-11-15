// Pinned XMage: entry makes each opponent choose and discard one card.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 1),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::Discard(PlayerSet::Opponent, 1)],
    }],
    ..CardRules::EMPTY
};
