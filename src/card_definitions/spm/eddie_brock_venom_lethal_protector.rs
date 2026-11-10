pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: one(TargetRule::Object {
            zone: TargetZone::Graveyard,
            kinds: 1,
            relation: 0,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: 1,
        }),
        program: &[Instruction::ReturnToBattlefield],
    }],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            colored: [0, 0, 1, 1, 1, 0],
            generic: 3,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::TransformSource],
        sorcery: true,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
