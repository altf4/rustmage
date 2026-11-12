pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Creature),
        &[
            Instruction::Damage(6),
            Instruction::If {
                guard: StepGuard::Expression {
                    value: StatValue::CONTROLLER_GRAVEYARD_CARD_TYPES,
                    minimum: Some(4),
                    maximum: None,
                },
                skip: 1,
            },
            Instruction::ExileTopPlay {
                amount: 1,
                next_turn: true,
            },
        ],
    ),
    ..CardRules::EMPTY
};
