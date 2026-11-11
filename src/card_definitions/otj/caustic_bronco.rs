pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfAttacks,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::If {
                guard: StepGuard::SourceSaddled,
                skip: 2,
            },
            Instruction::RevealTopToHandLoseManaValueFor(PlayerSet::Opponent),
            Instruction::Skip(1),
            Instruction::RevealTopToHandLoseManaValue,
        ],
    }],
    activated: &[ActivatedAbility {
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        exhaust: false,
        guard: ActivationGuard::Always,
        cost: ActivationCost::ZERO,
        targets: TargetPlan::NONE,
        program: &[Instruction::BecomeSaddled { minimum: 3 }],
        sorcery: true,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
