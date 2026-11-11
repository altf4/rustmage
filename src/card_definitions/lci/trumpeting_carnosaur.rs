pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(7, 6),
    keywords: Keywords::TRAMPLE,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::Discover(5)],
    }],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        cycling: false,
        origin: ActivationOrigin::Hand,
        cost: ActivationCost {
            generic: 2,
            colored: [0, 0, 0, 1, 0, 0],
            mana_component: true,
            discard: true,
            ..ActivationCost::ZERO
        },
        targets: one(object(1 | 64)),
        program: &[Instruction::Damage(3)],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
