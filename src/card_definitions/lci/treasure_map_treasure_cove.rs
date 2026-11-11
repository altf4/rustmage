pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            generic: 1,
            mana_component: true,
            tap: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::Scry(1),
            Instruction::PutCounters {
                subject: Subject::Source,
                kind: CounterKind::Landmark,
                amount: 1,
            },
            Instruction::IfAmountAtLeast {
                value: AmountValue::SourceCounterQuantity(CounterKind::Landmark),
                minimum: 3,
                skip: 3,
            },
            Instruction::RemoveAllSourceCounters(CounterKind::Landmark),
            Instruction::TransformSource,
            Instruction::CreateTokens {
                token: Card(5302),
                amount: 3,
                tapped: false,
            },
        ],
        sorcery: false,
        once_per_turn: false,
    }],
    extras: &CardExtras {
        transforming_double_face: true,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
