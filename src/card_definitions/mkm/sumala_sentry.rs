pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 3),
    keywords: Keywords::REACH,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::PermanentYouControlTurnedFaceUp,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::PutCountersOnEventPermanent {
                kind: CounterKind::PlusOne,
                amount: 1,
            },
            Instruction::PutCounters {
                subject: Subject::Source,
                kind: CounterKind::PlusOne,
                amount: 1,
            },
        ],
    }],
    ..CardRules::EMPTY
};
