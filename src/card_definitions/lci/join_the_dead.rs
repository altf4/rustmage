const DESCEND_FOUR: StepGuard = StepGuard::Expression {
    value: StatValue::Count(CountFilter {
        zone: CountZone::Graveyard,
        relation: 0,
        kind: 371,
        exclude: 0,
        subtype: None,
        basic: false,
        other: false,
        cards_only: true,
    }),
    minimum: Some(4),
    maximum: None,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Creature),
        &[
            Instruction::Grow {
                subject: Subject::Targets,
                power: -5,
                toughness: -5,
            },
            Instruction::If {
                guard: DESCEND_FOUR,
                skip: 1,
            },
            Instruction::Grow {
                subject: Subject::Targets,
                power: -5,
                toughness: -5,
            },
        ],
    ),
    ..CardRules::EMPTY
};
