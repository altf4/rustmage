pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[Instruction::Search(SearchSpec {
                filter: SearchFilter::BasicLand,
                maximum: 2,
                destination: SearchDestination::Battlefield,
                tapped: true,
                reveal: false,
            })],
        },
        TriggeredAbility {
            condition: TriggerCondition::SubtypeEntersYou("Dragon"),
            targets: TargetPlan::NONE,
            program: &[Instruction::ReturnSourceToHand(TargetZone::Battlefield)],
        },
    ],
    ..CardRules::EMPTY
};
