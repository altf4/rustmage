// Complete Oracle behavior, checked against pinned XMage.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    keywords: Keywords::LIFELINK,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: one(TargetRule::Player {
            opponent_only: true,
        }),
        program: &[Instruction::LookTargetHandMayExileNonlandThenVictimDraw],
    }],
    ..CardRules::EMPTY
};
