pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Single(TargetSpec::NONE),
        program: &[
            Instruction::May { skip: 1 },
            Instruction::Search(SearchSpec {
                filter: SearchFilter::AllOf(&[
                    SearchFilter::Kinds(4 | 8),
                    SearchFilter::ManaValue {
                        minimum: 1,
                        maximum: 1,
                    },
                ]),
                maximum: 1,
                destination: SearchDestination::Hand,
                tapped: false,
                reveal: true,
            }),
        ],
    }],
    ..CardRules::EMPTY
};
