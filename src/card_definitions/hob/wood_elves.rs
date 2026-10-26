// Direct typed behavior from the complete pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::Search(SearchSpec {
            filter: SearchFilter::AllOf(&[SearchFilter::Land, SearchFilter::Subtype("Forest")]),
            maximum: 1,
            destination: SearchDestination::Battlefield,
            tapped: false,
            reveal: false,
        })],
    }],
    ..CardRules::EMPTY
};
