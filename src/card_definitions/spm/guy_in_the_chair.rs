const TAP: ActivationCost = ActivationCost {
    tap: true,
    ..ActivationCost::ZERO
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    activated: &[
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: TAP,
            targets: TargetPlan::NONE,
            program: &[Instruction::AddMana(ManaProduction::any_color(1))],
            sorcery: false,
            once_per_turn: false,
        },
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: ActivationCost {
                colored: [0, 0, 0, 0, 1, 0],
                generic: 2,
                mana_component: true,
                ..TAP
            },
            targets: one(TargetRule::Filtered {
                base: &object(0),
                predicate: &TargetPredicate::Subtype("Spider"),
            }),
            program: &[Instruction::PutCounters {
                subject: Subject::Targets,
                kind: CounterKind::PlusOne,
                amount: 1,
            }],
            sorcery: true,
            once_per_turn: false,
        },
    ],
    ..CardRules::EMPTY
};
