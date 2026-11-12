pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    keywords: Keywords::REACH,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Single(TargetSpec {
            rule: TargetRule::Creature,
            min: 1,
            max: 3,
        }),
        program: &[
            Instruction::DistributeCounters {
                total: 3,
                kind: CounterKind::PlusOne,
            },
            Instruction::GainLifeScaled(
                PlayerSet::Controller,
                AmountValue::GreatestCreatureToughness,
            ),
        ],
    }],
    ..CardRules::EMPTY
};
