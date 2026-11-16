// Pinned XMage: basic land or Planet search enters tapped; Beam me up uses its own cost.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::Search(SearchSpec {
            filter: SearchFilter::AnyOf(&[
                SearchFilter::BasicLand,
                SearchFilter::Subtype("Planet"),
            ]),
            maximum: 1,
            destination: SearchDestination::Battlefield,
            tapped: true,
            reveal: false,
        })],
    ),
    extras: &CardExtras {
        beam_me_up_cost: Some(ManaCost {
            colored: [0, 0, 0, 0, 1, 0],
            generic: 1,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
