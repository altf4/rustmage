const CONTROLLED_BIRDS: StatValue = StatValue::Count(CountFilter {
    zone: CountZone::Battlefield,
    relation: 0,
    kind: 0,
    exclude: 0,
    subtype: Some("Bird"),
    basic: false,
    other: false,
    cards_only: false,
});

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Spell)),
        &[
            Instruction::If {
                guard: StepGuard::Expression {
                    value: CONTROLLED_BIRDS,
                    minimum: Some(1),
                    maximum: None,
                },
                skip: 2,
            },
            Instruction::CounterUnlessPay {
                cost: ManaCost {
                    colored: [0; 6],
                    generic: 4,
                },
                subject: CounterSubject::Targets,
                exile: false,
                unpaid_skip: 0,
            },
            Instruction::Skip(1),
            Instruction::CounterUnlessPay {
                cost: ManaCost {
                    colored: [0; 6],
                    generic: 2,
                },
                subject: CounterSubject::Targets,
                exile: false,
                unpaid_skip: 0,
            },
        ],
    ),
    ..CardRules::EMPTY
};
