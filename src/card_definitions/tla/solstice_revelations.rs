// Direct typed behavior from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::ExileUntilNonlandCastBelowOrHand(
            AmountValue::Stat(StatValue::Count(CountFilter {
                zone: CountZone::Battlefield,
                relation: 0,
                kind: 2,
                exclude: 0,
                subtype: Some("Mountain"),
                basic: false,
                other: false,
                cards_only: false,
            })),
        )],
    ),
    extras: &CardExtras {
        flashback_cost: Some(ManaCost {
            colored: [0, 0, 0, 1, 0, 0],
            generic: 6,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
