// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[
            Instruction::Grow {
                subject: Subject::Targets,
                power: 3,
                toughness: 3,
            },
            Instruction::If {
                guard: StepGuard::Expression {
                    value: StatValue::CreatureEntriesThisTurn { other: false },
                    minimum: Some(1),
                    maximum: None,
                },
                skip: 1,
            },
            Instruction::Draw(PlayerSet::Controller, 1),
        ],
    ),
    ..CardRules::EMPTY
};
