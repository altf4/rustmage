pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        cycling: false,
        origin: ActivationOrigin::Battlefield,
        cost: ActivationCost {
            generic: 2,
            mana_component: true,
            sacrifice: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::Scry(1),
            Instruction::Draw(PlayerSet::Controller, 1),
        ],
        sorcery: false,
        once_per_turn: false,
    }],
    deck_limit: None,
    ..CardRules::EMPTY
};
