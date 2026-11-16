// Pinned Oracle and complete XMage class.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    keywords: Keywords::FLYING,
    activated: &[ActivatedAbility {
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        exhaust: false,
        once_per_turn: true,
        sorcery: true,
        cost: ActivationCost {
            colored: [1, 0, 0, 0, 0, 0],
            generic: 1,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        targets: one(TargetRule::Creature),
        program: &[Instruction::Untap],
    }],
    ..CardRules::EMPTY
};
