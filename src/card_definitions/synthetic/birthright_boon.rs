// Adventure spell of Kellan, the Fae-Blooded, with its own pinned Oracle face.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::Search(SearchSpec {
            filter: SearchFilter::AnyOf(&[
                SearchFilter::Subtype("Aura"),
                SearchFilter::Subtype("Equipment"),
            ]),
            maximum: 1,
            destination: SearchDestination::Hand,
            tapped: false,
            reveal: true,
        })],
    ),
    extras: &CardExtras {
        adventure_parent: Some(Card(2342)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
