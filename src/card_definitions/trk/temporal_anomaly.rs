// Oracle and pinned XMage reviewed. The hand choice is limited to this exact four-card mill.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Land,
    activated: &[
        ActivatedAbility {
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            exhaust: false,
            sorcery: false,
            once_per_turn: false,
            cost: ActivationCost {
                tap: true,
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::AddMana(ManaProduction::Fixed([
                0, 0, 0, 0, 0, 1,
            ]))],
        },
        ActivatedAbility {
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            exhaust: false,
            sorcery: false,
            once_per_turn: false,
            cost: ActivationCost {
                generic: 3,
                colored: [0, 0, 0, 0, 1, 0],
                mana_component: true,
                tap: true,
                sacrifice: true,
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::Mill(PlayerSet::Controller, MillAmount::Fixed(4)),
                Instruction::SelectMilled(MilledSelectionSpec {
                    filter: MilledFilter::Permanent,
                    count: MilledCount::Range {
                        minimum: 0,
                        maximum: 1,
                    },
                    destination: MilledDestination::Hand,
                }),
                Instruction::CreateTokens {
                    token: Card(6009),
                    amount: 1,
                    tapped: true,
                },
            ],
        },
    ],
    ..CardRules::EMPTY
};
