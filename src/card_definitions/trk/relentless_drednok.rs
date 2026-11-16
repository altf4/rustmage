// Complete Oracle behavior, checked against pinned XMage.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    keywords: Keywords::LIFELINK,
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            colored: [0, 0, 1, 0, 0, 0],
            generic: 2,
            mana_component: true,
            exile_self: true,
            ..ActivationCost::ZERO
        },
        origin: ActivationOrigin::Graveyard,
        guard: ActivationGuard::Always,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::Draw(PlayerSet::Controller, 1),
            Instruction::LoseLife(PlayerSet::Controller, 1),
        ],
        sorcery: false,
        once_per_turn: false,
        cycling: false,
        exhaust: false,
    }],
    ..CardRules::EMPTY
};
