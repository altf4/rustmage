// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(0, 4),
    keywords: Keywords::DEFENDER,
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            colored: [1, 0, 0, 0, 0, 0],
            generic: 2,
            mana_component: true,
            tap: true,
            ..ActivationCost::ZERO
        },
        targets: one(TargetRule::Creature),
        program: &[
            Instruction::Tap,
            Instruction::IfSubjectCanReceiveCounters {
                subject: Subject::Targets,
                skip: 3,
            },
            Instruction::May { skip: 2 },
            Instruction::PaySubjectCounterPlacement {
                subject: Subject::Targets,
                player: PlayerSet::Controller,
                kind: CounterKind::Stun,
                amount: 1,
            },
            Instruction::PutCounters {
                subject: Subject::Source,
                kind: CounterKind::Stun,
                amount: 1,
            },
        ],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
