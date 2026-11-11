const BEHIND_ON_LANDS: StepGuard = StepGuard::Expression {
    value: StatValue::OpponentControlsMoreLands,
    minimum: Some(1),
    maximum: None,
};
const FIND_PLAINS: Instruction = Instruction::SearchWithoutShuffle(SearchSpec {
    filter: SearchFilter::Subtype("Plains"),
    maximum: 1,
    destination: SearchDestination::Battlefield,
    tapped: true,
    reveal: false,
});

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    keywords: Keywords::VIGILANCE,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::InterveningIf {
            guard: BEHIND_ON_LANDS,
            event: &TriggerCondition::SelfEnters,
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::May { skip: 1 },
            FIND_PLAINS,
            Instruction::If {
                guard: BEHIND_ON_LANDS,
                skip: 2,
            },
            Instruction::May { skip: 1 },
            FIND_PLAINS,
            Instruction::ShuffleAfterSearches,
        ],
    }],
    ..CardRules::EMPTY
};
