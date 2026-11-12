pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 1),
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::StatAtLeast {
            value: StatValue::CONTROLLER_GRAVEYARD_CARD_TYPES,
            minimum: 4,
        },
        origin: ActivationOrigin::Graveyard,
        cycling: false,
        cost: ActivationCost {
            colored: [0, 0, 2, 0, 0, 0],
            generic: 2,
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
