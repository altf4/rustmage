pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 1),
    keywords: Keywords::FLYING.union(Keywords::VIGILANCE),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::FaceDilemma(CastRelation::You),
        targets: TargetPlan::NONE,
        program: &[Instruction::PutCounters {
            subject: Subject::Source,
            kind: CounterKind::PlusOne,
            amount: 1,
        }],
    }],
    ..CardRules::EMPTY
};
