// Whole-card typed Oracle behavior; complete XMage source pinned beside this file.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::SearchRequired(SearchSpec {
            filter: SearchFilter::Any,
            maximum: 1,
            destination: SearchDestination::Hand,
            tapped: false,
            reveal: false,
        })],
    ),
    extras: &CardExtras {
        prepared_identity: Some("@prepare/emeritus_of_woe_demonic_tutor"),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
