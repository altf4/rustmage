// The Adventure face of The Arkenstone; full Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::Search(SearchSpec {
            filter: SearchFilter::AllOf(&[SearchFilter::Creature, SearchFilter::Legendary]),
            maximum: 1,
            destination: SearchDestination::Hand,
            tapped: false,
            reveal: true,
        })],
    ),
    extras: &CardExtras {
        adventure_parent: Some(Card(4420)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
