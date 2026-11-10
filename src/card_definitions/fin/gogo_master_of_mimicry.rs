pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 4),
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        cycling: false,
        origin: ActivationOrigin::Battlefield,
        cost: ActivationCost {
            mana_component: true,
            tap: true,
            ..ActivationCost::ZERO
        },
        targets: one(TargetRule::YourAbility),
        program: &[
            Instruction::ActivationXWithMinimum {
                symbols: 2,
                minimum: 1,
            },
            Instruction::ActivationCannotBeCopied,
            Instruction::CopyTargetAbilityX { group: 0 },
        ],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
