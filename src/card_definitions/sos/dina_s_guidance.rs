// Direct typed behavior from the complete pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::Search(SearchSpec {
            filter: SearchFilter::Kinds(1),
            maximum: 1,
            destination: SearchDestination::HandOrGraveyard,
            tapped: false,
            reveal: true,
        })],
    ),
    ..CardRules::EMPTY
};
