// Pinned Oracle and XMage: colorless mana or paid sacrifice for life swing and Planet.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Land,
    activated: &[
        ActivatedAbility {
            cost: ActivationCost {
                tap: true,
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::AddMana(ManaProduction::Fixed([
                0, 0, 0, 0, 0, 1,
            ]))],
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            sorcery: false,
            once_per_turn: false,
            cycling: false,
            exhaust: false,
        },
        ActivatedAbility {
            cost: ActivationCost {
                colored: [0, 0, 1, 0, 0, 0],
                generic: 3,
                mana_component: true,
                tap: true,
                sacrifice: true,
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::LoseLife(PlayerSet::Opponent, 2),
                Instruction::GainLife(PlayerSet::Controller, 2),
                Instruction::CreateTokens {
                    token: Card(6009),
                    amount: 1,
                    tapped: true,
                },
            ],
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            sorcery: false,
            once_per_turn: false,
            cycling: false,
            exhaust: false,
        },
    ],
    ..CardRules::EMPTY
};
