// Source-audited direct typed definition; see paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::SearchCreaturesUpToChosenX(SearchSpec {
            filter: SearchFilter::Creature,
            maximum: 1,
            destination: SearchDestination::Battlefield,
            tapped: false,
            reveal: false,
        })],
    ),
    extras: &CardExtras {
        x_symbols: 1,
        harmonize_x_symbols: 1,
        harmonize_cost: Some(ManaCost {
            generic: 0,
            colored: [0, 0, 0, 0, 4, 0],
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
