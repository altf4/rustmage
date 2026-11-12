pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    keywords: Keywords::HASTE,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfDamaged,
        targets: one(TargetRule::Filtered {
            base: &TargetRule::Any,
            predicate: &TargetPredicate::OtherThanSource,
        }),
        program: &[
            Instruction::DamageRememberPlayersScaled(AmountValue::DamageReceivedAmount),
            Instruction::DamagedPlayersCannotGainLifeForGame,
        ],
    }],
    ..CardRules::EMPTY
};
