// Directly authored from the complete pinned Oracle and XMage implementation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::Search(SearchSpec {
            filter: SearchFilter::Kinds(64),
            maximum: 1,
            destination: SearchDestination::Top,
            tapped: false,
            reveal: true,
        })],
    ),
    ..CardRules::EMPTY
};
