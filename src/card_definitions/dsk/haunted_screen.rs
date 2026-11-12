const TAP_MANA: ActivatedAbility = ActivatedAbility {
    exhaust: false,
    guard: ActivationGuard::Always,
    cycling: false,
    origin: ActivationOrigin::Battlefield,
    cost: ActivationCost {
        tap: true,
        ..ActivationCost::ZERO
    },
    targets: TargetPlan::NONE,
    program: &[Instruction::AddMana(ManaProduction::Choice {
        mask: 1 | 4,
        amount: 1,
    })],
    sorcery: false,
    once_per_turn: false,
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[
        TAP_MANA,
        ActivatedAbility {
            cost: ActivationCost {
                life: 1,
                ..TAP_MANA.cost
            },
            program: &[Instruction::AddMana(ManaProduction::Choice {
                mask: 2 | 8 | 16,
                amount: 1,
            })],
            ..TAP_MANA
        },
        ActivatedAbility {
            guard: ActivationGuard::OncePerIncarnation(&ActivationGuard::Always),
            cost: ActivationCost {
                generic: 7,
                mana_component: true,
                ..ActivationCost::ZERO
            },
            program: &[
                Instruction::PutCounters {
                    subject: Subject::Source,
                    kind: CounterKind::PlusOne,
                    amount: 7,
                },
                Instruction::AnimateSource(AnimationSpec {
                    artifact: false,
                    creature_types: AnimationCreatureTypes::Additional(&["Spirit"]),
                    base: Some((0, 0)),
                    keywords: Keywords::NONE,
                    duration: AnimationDuration::Permanent,
                }),
            ],
            ..TAP_MANA
        },
    ],
    ..CardRules::EMPTY
};
