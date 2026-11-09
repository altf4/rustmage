// Directly authored from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(7, 7),
    keywords: Keywords::FLYING,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEntersOrAttacks,
        targets: TargetPlan::NONE,
        program: &[Instruction::CopyRandomGraveyardPermanentUntilNonland],
    }],
    ..CardRules::EMPTY
};
