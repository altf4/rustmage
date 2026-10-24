// Complete typed Oracle behavior; see paired source provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::creature_you_control())),
        &[
            Instruction::PutCounters {
                subject: Subject::Targets,
                kind: CounterKind::PlusOne,
                amount: 1,
            },
            Instruction::DoubleCounterKind {
                subject: Subject::Targets,
                kind: CounterKind::PlusOne,
            },
        ],
    ),
    ..CardRules::EMPTY
};
