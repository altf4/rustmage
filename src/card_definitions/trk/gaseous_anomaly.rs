// Pinned XMage: the second activation pays mana, taps and sacrifices before
// drawing and creating the tapped New Planet token.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Land,
    activated: &[
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: ActivationCost {
                tap: true,
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::AddMana(ManaProduction::Fixed([
                0, 0, 0, 0, 0, 1,
            ]))],
            sorcery: false,
            once_per_turn: false,
        },
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: ActivationCost {
                generic: 5,
                colored: [0, 1, 0, 0, 0, 0],
                tap: true,
                sacrifice: true,
                mana_component: true,
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::Draw(PlayerSet::Controller, 2),
                Instruction::CreateTokens {
                    token: Card(6009),
                    amount: 1,
                    tapped: true,
                },
            ],
            sorcery: false,
            once_per_turn: false,
        },
    ],
    ..CardRules::EMPTY
};
