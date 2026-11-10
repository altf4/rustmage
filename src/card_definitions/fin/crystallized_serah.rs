// Directly authored from the paired Oracle and pinned XMage sources.
const FIRST_LEGEND: SpellReduction = SpellReduction {
    source_only: false,
    filter: CastFilter::FirstQualifiedCreature(CreatureCastQualifier::Legendary),
    amount: SpellReductionAmount::Fixed(2),
    turn: CostTurn::Any,
    guard: StepGuard::Always,
};
const GROUPS: &[StaticAbilityGroup] = &[
    StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::CostReduction(0, FIRST_LEGEND)],
    },
    StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Modify,
        fragments: &[StaticAbilityFragment::ControlledLegendaryCreatureBonus(
            Stats {
                power: 2,
                toughness: 2,
            },
        )],
    },
];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: GROUPS,
    extras: &CardExtras {
        static_ability_groups: GROUPS,
        cost_reductions: &[FIRST_LEGEND],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
