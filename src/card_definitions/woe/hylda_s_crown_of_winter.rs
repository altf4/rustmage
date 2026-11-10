// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    extras: &CardExtras {
        activation_generic_reductions: &[(
            0,
            ActivationGenericReduction::Stat {
                value: StatValue::ControllerTurn,
                multiplier: 1,
            },
        )],
        ..CardExtras::EMPTY
    },
    activated: &[
        ActivatedAbility {
            cost: ActivationCost {
                generic: 1,
                mana_component: true,
                tap: true,
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
            program: &[Instruction::Tap],
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            exhaust: false,
            sorcery: false,
            once_per_turn: false,
        },
        ActivatedAbility {
            cost: ActivationCost {
                generic: 3,
                mana_component: true,
                sacrifice: true,
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::DrawScaled(
                PlayerSet::Controller,
                AmountValue::Stat(StatValue::TappedCount(CountFilter {
                    zone: CountZone::Battlefield,
                    relation: 1,
                    kind: 1,
                    exclude: 0,
                    subtype: None,
                    basic: false,
                    other: false,
                    cards_only: false,
                })),
            )],
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            exhaust: false,
            sorcery: false,
            once_per_turn: false,
        },
    ],
    ..CardRules::EMPTY
};
