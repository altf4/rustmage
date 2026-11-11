pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[Instruction::SelectLibraryTo(LibrarySelectionEffect {
                spec: LibrarySelectionSpec {
                    look: 4,
                    count: LibrarySelectionCount::Fixed(1),
                    filter: LibrarySelectionFilter::Kinds(1),
                    optional: true,
                    reveal: true,
                    remainder: LibraryRemainder::BottomAny,
                },
                look: LibraryLook::Fixed(4),
                destination: LibraryPickDestination::Hand,
                public: false,
                maximum_mana_value: None,
            })],
        },
        TriggeredAbility {
            condition: TriggerCondition::AtStep {
                step: TriggerStep::End,
                relation: CastRelation::You,
                guard: StepGuard::Expression {
                    value: StatValue::Count(CountFilter {
                        zone: CountZone::Battlefield,
                        relation: 0,
                        kind: 1,
                        exclude: 0,
                        subtype: None,
                        basic: false,
                        other: false,
                        cards_only: false,
                    }),
                    minimum: Some(4),
                    maximum: None,
                },
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::TransformSource],
        },
    ],
    extras: &CardExtras {
        transforming_double_face: true,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
