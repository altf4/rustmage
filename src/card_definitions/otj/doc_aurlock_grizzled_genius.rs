// Directly authored from Oracle and the pinned XMage implementation.
const REDUCTION: SpellReduction = SpellReduction {
    source_only: false,
    filter: CastFilter::Any,
    amount: SpellReductionAmount::Fixed(2),
    turn: CostTurn::GraveyardOrExile,
    guard: StepGuard::Always,
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::CostReduction(0, REDUCTION)],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::PlotCostReduction(2)],
        },
    ],
    extras: &CardExtras {
        cost_reductions: &[REDUCTION],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
