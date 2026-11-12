// Directly authored from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[Instruction::ManifestDread],
        },
        TriggeredAbility {
            condition: TriggerCondition::FilteredEntersYou(EntryPredicate::FaceDown),
            targets: TargetPlan::NONE,
            program: &[Instruction::Search(SearchSpec {
                filter: SearchFilter::BasicLand,
                maximum: 1,
                destination: SearchDestination::Battlefield,
                tapped: true,
                reveal: false,
            })],
        },
    ],
    ..CardRules::EMPTY
};
