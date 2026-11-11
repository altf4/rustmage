pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[ActivatedAbility {
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        exhaust: false,
        guard: ActivationGuard::Always,
        cost: ActivationCost {
            generic: 6,
            mana_component: true,
            tap: true,
            sacrifice: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::Groups(&[
            TargetSpec {
                rule: object(16),
                min: 0,
                max: 1,
            },
            TargetSpec {
                rule: TargetRule::Creature,
                min: 0,
                max: 1,
            },
            TargetSpec {
                rule: object(2),
                min: 0,
                max: 1,
            },
        ]),
        program: &[Instruction::DestroyAllTargets],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
