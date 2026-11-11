// Direct typed behavior from the paired Oracle and XMage sources.
const ACTIVATION: ActivatedAbility = ActivatedAbility {
    exhaust: false,
    guard: ActivationGuard::Always,
    origin: ActivationOrigin::Battlefield,
    cycling: false,
    cost: ActivationCost {
        tap: true,
        ..ActivationCost::ZERO
    },
    targets: TargetPlan::NONE,
    program: &[
        Instruction::Draw(PlayerSet::Controller, 1),
        Instruction::Discard(PlayerSet::Controller, 1),
    ],
    sorcery: false,
    once_per_turn: false,
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[
        ACTIVATION,
        ActivatedAbility {
            guard: ActivationGuard::StatAtLeast {
                value: StatValue::GraveyardPermanentTypes,
                minimum: 4,
            },
            cost: ActivationCost {
                generic: 4,
                mana_component: true,
                ..ACTIVATION.cost
            },
            program: &[Instruction::TransformSource],
            ..ACTIVATION
        },
    ],
    extras: &CardExtras {
        transforming_double_face: true,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
