// Recovery chooses from the whole graveyard after milling; it does not target.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::NONE),
        &[
            Instruction::Mill(PlayerSet::Controller, MillAmount::Fixed(3)),
            Instruction::CollectGraveyard,
            Instruction::SelectMilled(MilledSelectionSpec {
                filter: MilledFilter::Kinds(4 | 8),
                count: MilledCount::Range {
                    minimum: 1,
                    maximum: 1,
                },
                destination: MilledDestination::Hand,
            }),
        ],
    ),
    extras: &CardExtras {
        flashback_cost: Some(ManaCost {
            colored: [0, 2, 0, 0, 0, 0],
            generic: 5,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
