// Pinned Oracle and complete XMage class: any number of exact names can be found.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 1),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::May { skip: 1 },
            Instruction::Search(SearchSpec {
                filter: SearchFilter::Name("Andromedan Automatons"),
                maximum: u64::MAX,
                destination: SearchDestination::Hand,
                tapped: false,
                reveal: true,
            }),
        ],
    }],
    ..CardRules::EMPTY
};
