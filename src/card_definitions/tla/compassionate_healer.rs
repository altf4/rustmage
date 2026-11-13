// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfBecomesTapped,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::GainLife(PlayerSet::Controller, 1),
            Instruction::Scry(1),
        ],
    }],
    ..CardRules::EMPTY
};
