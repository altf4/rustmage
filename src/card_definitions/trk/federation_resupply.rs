// Pinned XMage: revealed Aura, Equipment, or Spacecraft tutor with Beam me up.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::Search(SearchSpec {
            filter: SearchFilter::AnyOf(&[
                SearchFilter::Subtype("Aura"),
                SearchFilter::Subtype("Equipment"),
                SearchFilter::Subtype("Spacecraft"),
            ]),
            maximum: 1,
            destination: SearchDestination::Hand,
            tapped: false,
            reveal: true,
        })],
    ),
    extras: &CardExtras {
        beam_me_up_cost: Some(ManaCost {
            colored: [1, 0, 0, 0, 0, 0],
            generic: 3,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
