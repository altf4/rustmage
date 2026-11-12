// Directly authored from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfAttacks,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::RememberCount(AmountValue::ControlledCreaturePowerAtMost {
                maximum: 2,
                other: false,
            }),
            Instruction::DamagePlayersScaled(PlayerSet::Opponent, AmountValue::RememberedCount),
            Instruction::GainLifeScaled(PlayerSet::Controller, AmountValue::RememberedCount),
        ],
    }],
    ..CardRules::EMPTY
};
