// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::Search(SearchSpec {
                filter: SearchFilter::BasicLand,
                maximum: 1,
                destination: SearchDestination::Battlefield,
                tapped: true,
                reveal: false,
            }),
            Instruction::If {
                guard: StepGuard::Expression {
                    value: StatValue::CountUnion(&[
                        CountFilter {
                            zone: CountZone::Battlefield,
                            relation: 0,
                            kind: 2,
                            exclude: 0,
                            subtype: None,
                            basic: false,
                            other: false,
                            cards_only: false,
                        },
                        CountFilter {
                            zone: CountZone::Battlefield,
                            relation: 0,
                            kind: 0,
                            exclude: 0,
                            subtype: Some("Treefolk"),
                            basic: false,
                            other: false,
                            cards_only: false,
                        },
                    ]),
                    minimum: Some(7),
                    maximum: None,
                },
                skip: 1,
            },
            Instruction::CreateTokens {
                token: Card(5345),
                amount: 1,
                tapped: false,
            },
        ],
    ),
    ..CardRules::EMPTY
};
