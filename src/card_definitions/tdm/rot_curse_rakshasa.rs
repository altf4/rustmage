// Source-audited direct typed definition; see paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 5),
    keywords: Keywords::TRAMPLE.union(Keywords::DECAYED),
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Graveyard,
        cycling: false,
        cost: ActivationCost {
            colored: [0, 0, 2, 0, 0, 0],
            mana_component: true,
            exile_self: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::Single(TargetSpec {
            rule: TargetRule::Creature,
            min: 0,
            max: usize::MAX,
        }),
        program: &[
            Instruction::ActivationX,
            Instruction::TargetsExactlyX,
            Instruction::PutCounters {
                subject: Subject::Targets,
                kind: CounterKind::Decayed,
                amount: 1,
            },
        ],
        sorcery: true,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
