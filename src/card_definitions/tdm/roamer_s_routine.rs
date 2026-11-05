// Source-audited direct typed definition; see paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::Search(SearchSpec {
            filter: SearchFilter::BasicLand,
            maximum: 1,
            destination: SearchDestination::Battlefield,
            tapped: true,
            reveal: false,
        })],
    ),
    extras: &CardExtras {
        harmonize_cost: Some(ManaCost {
            generic: 4,
            colored: [0, 0, 0, 0, 1, 0],
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
