pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    keywords: Keywords::REACH,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfAttacksDefendingPlayer,
        targets: TargetPlan::NONE,
        program: &[Instruction::DamagePlayersScaled(
            PlayerSet::EventPlayer,
            AmountValue::EquippedCreaturesYouControl,
        )],
    }],
    ..CardRules::EMPTY
};
