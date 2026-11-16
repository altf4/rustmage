// Complete Oracle behavior, checked against pinned XMage or official release notes.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 5),
    keywords: Keywords::REACH,
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            colored: [0, 0, 0, 0, 1, 0],
            mana_component: true,
            exile_self: true,
            ..ActivationCost::ZERO
        },
        origin: ActivationOrigin::Graveyard,
        guard: ActivationGuard::Always,
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokens {
            token: Card(6008),
            amount: 1,
            tapped: false,
        }],
        sorcery: true,
        once_per_turn: false,
        cycling: false,
        exhaust: false,
    }],
    ..CardRules::EMPTY
};
