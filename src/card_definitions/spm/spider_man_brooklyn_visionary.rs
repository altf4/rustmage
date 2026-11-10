// Oracle and pinned XMage evidence are recorded in the paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::Search(SearchSpec {
            filter: SearchFilter::BasicLand,
            maximum: 1,
            destination: SearchDestination::Battlefield,
            tapped: true,
            reveal: false,
        })],
    }],
    extras: &CardExtras {
        web_slinging_cost: Some(ManaCost {
            colored: [0, 0, 0, 0, 1, 0],
            generic: 2,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
