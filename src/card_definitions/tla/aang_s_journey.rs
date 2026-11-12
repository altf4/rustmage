// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::IfKicked { skip: 2 },
            Instruction::SearchTwoQualitiesToHand(&[
                SearchFilter::BasicLand,
                SearchFilter::Subtype("Shrine"),
            ]),
            Instruction::Skip(1),
            Instruction::Search(SearchSpec {
                filter: SearchFilter::BasicLand,
                maximum: 1,
                destination: SearchDestination::Hand,
                tapped: false,
                reveal: true,
            }),
            Instruction::GainLife(PlayerSet::Controller, 2),
        ],
    ),
    extras: &CardExtras {
        kicker_cost: Some(generic(2)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
