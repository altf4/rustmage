pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 3),
    keywords: Keywords::TRAMPLE,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::OpponentHighestPowerCreatureDies,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::Draw(PlayerSet::Controller, 1),
            Instruction::PutCounters {
                subject: Subject::Source,
                kind: CounterKind::PlusOne,
                amount: 1,
            },
        ],
    }],
    ..CardRules::EMPTY
};
