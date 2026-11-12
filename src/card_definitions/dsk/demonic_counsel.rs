const GRAVEYARD_TYPES: StatValue = StatValue::CardTypes(CountFilter {
    zone: CountZone::Graveyard,
    relation: 0,
    kind: 0,
    exclude: 0,
    subtype: None,
    basic: false,
    other: false,
    cards_only: true,
});

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::If {
                guard: StepGuard::Expression {
                    value: GRAVEYARD_TYPES,
                    minimum: Some(4),
                    maximum: None,
                },
                skip: 2,
            },
            Instruction::SearchRequired(SearchSpec {
                filter: SearchFilter::Any,
                maximum: 1,
                destination: SearchDestination::Hand,
                tapped: false,
                reveal: false,
            }),
            Instruction::Skip(1),
            Instruction::Search(SearchSpec {
                filter: SearchFilter::Subtype("Demon"),
                maximum: 1,
                destination: SearchDestination::Hand,
                tapped: false,
                reveal: true,
            }),
        ],
    ),
    ..CardRules::EMPTY
};
