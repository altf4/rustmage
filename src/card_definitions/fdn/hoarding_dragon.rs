// Direct typed behavior from complete Oracle and pinned XMage implementations.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    keywords: Keywords::FLYING,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[
                Instruction::May { skip: 1 },
                Instruction::Search(SearchSpec {
                    filter: SearchFilter::Artifact,
                    maximum: 1,
                    destination: SearchDestination::LinkedExile,
                    tapped: false,
                    reveal: true,
                }),
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfDies,
            targets: TargetPlan::NONE,
            program: &[
                Instruction::May { skip: 1 },
                Instruction::ReturnLinkedExileToHand,
            ],
        },
    ],
    ..CardRules::EMPTY
};
