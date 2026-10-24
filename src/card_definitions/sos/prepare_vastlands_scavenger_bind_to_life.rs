// Directly authored from the complete pinned Oracle and XMage implementation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::Mill(PlayerSet::Controller, MillAmount::Fixed(7)),
            Instruction::SelectMilled(MilledSelectionSpec {
                filter: MilledFilter::Kinds(1),
                count: MilledCount::Range {
                    minimum: 1,
                    maximum: 1,
                },
                destination: MilledDestination::Battlefield,
            }),
        ],
    ),
    extras: &CardExtras {
        prepared_identity: Some("@prepare/vastlands_scavenger_bind_to_life"),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
