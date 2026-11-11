// Direct typed behavior from the paired Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::Search(SearchSpec {
            filter: SearchFilter::AnyOf(&[SearchFilter::BasicLand, SearchFilter::Subtype("Cave")]),
            maximum: 1,
            destination: SearchDestination::Battlefield,
            tapped: true,
            reveal: false,
        })],
    }],
    ..CardRules::EMPTY
};
