const COPY_RIDER: Instruction =
    Instruction::ChooseSaddleRiders(SaddleRiderEffect::CopyNonlegendaryAttacking);

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 6),
    keywords: Keywords::HASTE,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfAttacksWhileSaddled,
        targets: TargetPlan::NONE,
        program: &[COPY_RIDER, COPY_RIDER],
    }],
    activated: &[ActivatedAbility {
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        exhaust: false,
        guard: ActivationGuard::Always,
        cost: ActivationCost::ZERO,
        targets: TargetPlan::NONE,
        program: &[Instruction::BecomeSaddled { minimum: 1 }],
        sorcery: true,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
