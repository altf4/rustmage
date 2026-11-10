pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Granted(&TriggerCondition::SelfDamaged),
        targets: TargetPlan::NONE,
        program: &[
            Instruction::DrawScaled(PlayerSet::Controller, AmountValue::DamageReceivedAmount),
            Instruction::LoseLifeScaled(PlayerSet::Controller, AmountValue::DamageReceivedAmount),
        ],
    }],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            tap: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        program: &[
            Instruction::GrantUntilEndTurn { ability: 0 },
            Instruction::MayTargetController { skip: 1 },
            Instruction::Damage(1),
        ],
        sorcery: true,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
