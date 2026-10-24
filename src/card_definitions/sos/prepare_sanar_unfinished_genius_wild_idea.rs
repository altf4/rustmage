// Complete typed Oracle behavior; see paired source provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::Search(SearchSpec {
            filter: SearchFilter::Kinds(12),
            maximum: 1,
            destination: SearchDestination::Hand,
            tapped: false,
            reveal: true,
        })],
    ),
    extras: &CardExtras {
        prepared_identity: Some("@prepare/sanar_unfinished_genius_wild_idea"),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
