pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfTurnedFaceUp,
        targets: TargetPlan::NONE,
        program: &[Instruction::Search(SearchSpec {
            filter: SearchFilter::AllOf(&[
                SearchFilter::Land,
                SearchFilter::AnyOf(&[
                    SearchFilter::Subtype("Plains"),
                    SearchFilter::Subtype("Island"),
                    SearchFilter::Subtype("Swamp"),
                    SearchFilter::Subtype("Mountain"),
                    SearchFilter::Subtype("Forest"),
                ]),
            ]),
            maximum: 1,
            destination: SearchDestination::Hand,
            tapped: false,
            reveal: true,
        })],
    }],
    extras: &CardExtras {
        disguise: Some(DisguiseSpec::fixed(ManaCost {
            colored: [0, 0, 0, 0, 1, 0],
            generic: 0,
        })),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
