// Complete Oracle behavior, checked against pinned XMage or official release notes.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 1),
    activated: &[
        ActivatedAbility {
            cost: ActivationCost {
                tap: true,
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::AddMana(ManaProduction::Fixed([
                0, 0, 0, 1, 0, 0,
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
                generic: 3,
                mana_component: true,
                tap: true,
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateTokens {
                token: Card(5302),
                amount: 1,
                tapped: false,
            }],
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
