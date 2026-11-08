// Direct typed behavior from complete pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::May { skip: 4 },
            Instruction::CaptureSearchResult,
            Instruction::Search(SearchSpec {
                filter: SearchFilter::Subtype("Food"),
                maximum: 1,
                destination: SearchDestination::Hand,
                tapped: false,
                reveal: true,
            }),
            Instruction::IfSearchPutInHand { skip: 1 },
            Instruction::Skip(1),
            Instruction::CreateTokens {
                token: Card(5246),
                amount: 1,
                tapped: false,
            },
        ],
    }],
    ..CardRules::EMPTY
};
