// Direct typed behavior from complete pinned Oracle and XMage implementations.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec {
            rule: TargetRule::Creature,
            min: 1,
            max: 3,
        }),
        &[
            Instruction::DistributeCounters {
                total: 3,
                kind: CounterKind::PlusOne,
            },
            Instruction::DoubleCounterKind {
                subject: Subject::Targets,
                kind: CounterKind::PlusOne,
            },
        ],
    ),
    ..CardRules::EMPTY
};
