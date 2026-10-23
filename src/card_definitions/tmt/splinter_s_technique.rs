// Direct typed behavior from complete pinned Oracle/XMage sources.
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
        sneak_costs: &[ManaCost {
            colored: [0, 0, 1, 0, 0, 0],
            generic: 1,
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
