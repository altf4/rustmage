pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[ActivatedAbility {
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cost: ActivationCost {
            generic: 4,
            mana_component: true,
            tap: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::TapCreaturesForActivationMana,
            Instruction::Draw(PlayerSet::Controller, 1),
        ],
        cycling: false,
        exhaust: false,
        sorcery: true,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
