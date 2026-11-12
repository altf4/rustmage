pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    keywords: Keywords::VIGILANCE,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::OtherCreatureEntersYouIfGreaterStats,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::PutCounters {
                subject: Subject::Source,
                kind: CounterKind::PlusOne,
                amount: 1,
            },
            Instruction::CreateTokens {
                token: Card(5296),
                amount: 1,
                tapped: false,
            },
        ],
    }],
    ..CardRules::EMPTY
};
