pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(6, 4),
    keywords: Keywords::VIGILANCE.union(Keywords::MENACE),
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::GraveyardCardsAtLeast {
            kind: 1 | 2 | 16 | 32 | 64 | 256,
            minimum: 8,
        },
        cycling: false,
        origin: ActivationOrigin::Graveyard,
        cost: ActivationCost {
            colored: [0, 1, 1, 0, 0, 0],
            generic: 4,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::ReturnSourceToBattlefieldWithCounter {
            tapped: false,
            counter: CounterKind::Finality,
        }],
        sorcery: true,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
