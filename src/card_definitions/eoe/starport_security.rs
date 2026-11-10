pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    extras: &CardExtras {
        activation_generic_reductions: &[(
            0,
            ActivationGenericReduction::Stat {
                value: StatValue::AtLeast {
                    value: &StatValue::ControlledCreaturesWithCounter(CounterKind::PlusOne),
                    minimum: 1,
                },
                multiplier: 2,
            },
        )],
        ..CardExtras::EMPTY
    },
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        cycling: false,
        origin: ActivationOrigin::Battlefield,
        cost: ActivationCost {
            colored: [1, 0, 0, 0, 0, 0],
            generic: 3,
            mana_component: true,
            tap: true,
            ..ActivationCost::ZERO
        },
        targets: one(TargetRule::Filtered {
            base: &TargetRule::Creature,
            predicate: &TargetPredicate::OtherThanSource,
        }),
        program: &[Instruction::Tap],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
