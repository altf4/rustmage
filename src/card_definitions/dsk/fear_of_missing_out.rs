pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[
                Instruction::Discard(PlayerSet::Controller, 1),
                Instruction::Draw(PlayerSet::Controller, 1),
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::InterveningIf {
                guard: StepGuard::Expression {
                    value: StatValue::CONTROLLER_GRAVEYARD_CARD_TYPES,
                    minimum: Some(4),
                    maximum: None,
                },
                event: &TriggerCondition::SelfFirstAttack,
            },
            targets: one(TargetRule::Creature),
            program: &[Instruction::Untap, Instruction::AdditionalCombat],
        },
    ],
    ..CardRules::EMPTY
};
