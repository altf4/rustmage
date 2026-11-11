// Direct typed back face from the paired Oracle and XMage sources.
const MANA: ActivatedAbility = ActivatedAbility {
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
        0, 0, 0, 1, 0, 0,
    ]))],
    sorcery: false,
    once_per_turn: false,
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Land,
    activated: &[
        MANA,
        ActivatedAbility {
            guard: ActivationGuard::StatAtLeast {
                value: StatValue::NoncombatDamageDealtByColorThisTurn(ChoiceColor::Red),
                minimum: 4,
            },
            cost: ActivationCost {
                colored: [0, 0, 0, 1, 0, 0],
                generic: 2,
                mana_component: true,
                ..MANA.cost
            },
            program: &[Instruction::TransformSource],
            sorcery: true,
            ..MANA
        },
    ],
    ..CardRules::EMPTY
};
