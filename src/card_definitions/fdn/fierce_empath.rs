pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Single(TargetSpec::NONE),
        program: &[
            Instruction::May { skip: 1 },
            Instruction::Search(SearchSpec {
                filter: SearchFilter::AllOf(&[
                    SearchFilter::Creature,
                    SearchFilter::ManaValue {
                        minimum: 6,
                        maximum: u64::MAX,
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
