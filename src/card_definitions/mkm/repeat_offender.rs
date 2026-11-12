pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 1),
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            colored: [0, 0, 1, 0, 0, 0],
            generic: 2,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::IfSubjectSuspected {
                subject: Subject::Source,
                skip: 2,
            },
            Instruction::PutCounters {
                subject: Subject::Source,
                kind: CounterKind::PlusOne,
                amount: 1,
            },
            Instruction::Skip(1),
            Instruction::Suspect {
                subject: Subject::Source,
            },
        ],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
