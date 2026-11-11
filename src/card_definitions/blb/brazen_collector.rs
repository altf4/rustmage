// Direct typed behavior from complete pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 1),
    keywords: Keywords::FIRST_STRIKE,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfAttacks,
        targets: TargetPlan::NONE,
        program: &[Instruction::AddRetainedManaScaled {
            color: 3,
            amount: AmountValue::Fixed(1),
            retention: ManaRetention::Turn,
        }],
    }],
    ..CardRules::EMPTY
};
