// Pinned Oracle and complete XMage class: distinct land names are counted at resolution.
const LAND_NAMES: StatValue = StatValue::DistinctNames(CountFilter {
    zone: CountZone::Battlefield,
    relation: 0,
    kind: 2,
    exclude: 0,
    subtype: None,
    basic: false,
    other: false,
    cards_only: false,
});
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::DrawScaled(PlayerSet::Controller, AmountValue::Stat(LAND_NAMES)),
            Instruction::NoMaximumHandSizeUntilNextTurn,
        ],
    ),
    ..CardRules::EMPTY
};
