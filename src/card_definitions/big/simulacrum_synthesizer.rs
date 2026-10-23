pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[Instruction::Scry(2)],
        },
        TriggeredAbility {
            condition: TriggerCondition::OtherFilteredEntersYou(
                EntryPredicate::ArtifactManaValueAtLeast(3),
            ),
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateTokens {
                token: Card(5345),
                amount: 1,
                tapped: false,
            }],
        },
    ],
    ..CardRules::EMPTY
};
