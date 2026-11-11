// The transform condition reads actual declared attackers this turn.
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
        1, 0, 0, 0, 0, 0,
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
                value: StatValue::DeclaredAttackersThisTurn,
                minimum: 3,
            },
            cost: ActivationCost {
                colored: [1, 0, 0, 0, 0, 0],
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
