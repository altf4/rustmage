pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 4),
    keywords: Keywords::MENACE,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEntersOrAttacks,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::MayMill(2),
            Instruction::Mill(PlayerSet::Controller, MillAmount::Fixed(2)),
        ],
    }],
    ..CardRules::EMPTY
};
