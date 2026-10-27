// Complete printed obligations from pinned Oracle and XMage sources.
const FIRST: StepGuard = StepGuard::Expression {
    value: StatValue::CreatureSpellsCastThisTurn,
    minimum: None,
    maximum: Some(0),
};
const REDUCTION: SpellReduction = SpellReduction {
    source_only: false,
    filter: CastFilter::Creature,
    amount: SpellReductionAmount::Fixed(2),
    turn: CostTurn::Any,
    guard: FIRST,
};
const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Rule,
    fragments: &[
        StaticAbilityFragment::CastTiming(0, CastFilter::Creature),
        StaticAbilityFragment::CostReduction(0, REDUCTION),
    ],
}];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 5),
    static_groups: GROUPS,
    extras: &CardExtras {
        static_ability_groups: GROUPS,
        cast_timing_grants: &[CastFilter::Creature],
        cast_timing_guard: FIRST,
        cost_reductions: &[REDUCTION],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
