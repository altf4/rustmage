// Each recipient mills its own frozen graveyard count; exhaust uses exact object history.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: ActivationCost {
                generic: 1,
                tap: true,
                mana_component: true,
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::Single(TargetSpec {
                rule: TargetRule::Player {
                    opponent_only: false,
                },
                min: 0,
                max: 2,
            }),
            program: &[Instruction::Mill(PlayerSet::Targeted, MillAmount::Fixed(2))],
            sorcery: false,
            once_per_turn: false,
        },
        ActivatedAbility {
            exhaust: true,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: ActivationCost {
                generic: 2,
                colored: [0, 2, 0, 0, 0, 0],
                tap: true,
                mana_component: true,
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::Single(TargetSpec {
                rule: TargetRule::Player {
                    opponent_only: false,
                },
                min: 0,
                max: 2,
            }),
            program: &[Instruction::Mill(
                PlayerSet::Targeted,
                MillAmount::RecipientGraveyard,
            )],
            sorcery: false,
            once_per_turn: false,
        },
    ],
    extras: &CardExtras {
        exhaust_mask: 2,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
