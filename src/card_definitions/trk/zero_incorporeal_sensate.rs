// Complete pinned XMage ZeroIncorporealSensate.java and Oracle reviewed.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    keywords: Keywords::LIFELINK,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: one(TargetRule::Player { opponent_only: true }),
        program: &[Instruction::LookTargetHandMayExileNonlandWithPlaySurcharge(
            std::num::NonZeroU32::new(2).unwrap(),
        )],
    }],
    ..CardRules::EMPTY
};
