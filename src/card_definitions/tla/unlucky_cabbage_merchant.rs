// Direct typed behavior from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateTokens {
                token: Card(5246),
                amount: 1,
                tapped: false,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::FoodsSacrificed,
            targets: TargetPlan::NONE,
            program: &[
                Instruction::May { skip: 3 },
                Instruction::SearchWithoutShuffle(SearchSpec {
                    filter: SearchFilter::BasicLand,
                    maximum: 1,
                    destination: SearchDestination::Battlefield,
                    tapped: true,
                    reveal: false,
                }),
                Instruction::PutSourceOnLibraryBottom,
                Instruction::ShuffleAfterSearches,
            ],
        },
    ],
    ..CardRules::EMPTY
};
