// Directly authored from the complete pinned Oracle and XMage implementation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::Search(SearchSpec {
            filter: SearchFilter::BasicLand,
            maximum: 1,
            destination: SearchDestination::Battlefield,
            tapped: true,
            reveal: true,
        })],
    ),
    extras: &CardExtras {
        prepared_identity: Some("@prepare/studious_first_year_rampant_growth"),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
