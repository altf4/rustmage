// Directly authored from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::SearchDifferentNames(SearchSpec {
            filter: SearchFilter::AnyOf(&[SearchFilter::BasicLand, SearchFilter::Subtype("Town")]),
            maximum: 2,
            destination: SearchDestination::Battlefield,
            tapped: true,
            reveal: true,
        })],
    ),
    ..CardRules::EMPTY
};
