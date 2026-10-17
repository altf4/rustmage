// Directly authored from complete Oracle and pinned XMage implementations.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::DrawScaled(
            PlayerSet::Controller,
            AmountValue::Stat(StatValue::DistinctManaValues(CountFilter {
                zone: CountZone::Battlefield,
                relation: 0,
                kind: 0,
                exclude: 2,
                subtype: None,
                basic: false,
                other: false,
                cards_only: false,
            })),
        )],
    ),
    ..CardRules::EMPTY
};
