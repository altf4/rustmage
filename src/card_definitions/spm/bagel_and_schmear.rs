const TAP_AND_SACRIFICE: ActivationCost = ActivationCost {
    mana_component: true,
    tap: true,
    sacrifice: true,
    ..ActivationCost::ZERO
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: ActivationCost {
                colored: [1, 0, 0, 0, 0, 0],
                ..TAP_AND_SACRIFICE
            },
            targets: TargetPlan::Single(TargetSpec {
                rule: TargetRule::Creature,
                min: 0,
                max: 1,
            }),
            program: &[
                Instruction::PutCounters {
                    subject: Subject::Targets,
                    kind: CounterKind::PlusOne,
                    amount: 1,
                },
                Instruction::Draw(PlayerSet::Controller, 1),
            ],
            sorcery: true,
            once_per_turn: false,
        },
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: ActivationCost {
                generic: 2,
                ..TAP_AND_SACRIFICE
            },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::GainLife(PlayerSet::Controller, 3),
                Instruction::Draw(PlayerSet::Controller, 1),
            ],
            sorcery: false,
            once_per_turn: false,
        },
    ],
    ..CardRules::EMPTY
};
