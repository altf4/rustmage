// Complete Oracle body from the paired pinned direct source.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    keywords: Keywords::VIGILANCE,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::OtherCreatureEntersYouIfGreaterStats,
        targets: TargetPlan::NONE,
        program: &[Instruction::PutCounters {
            subject: Subject::Source,
            kind: CounterKind::PlusOne,
            amount: 1,
        }],
    }],
    ..CardRules::EMPTY
};
