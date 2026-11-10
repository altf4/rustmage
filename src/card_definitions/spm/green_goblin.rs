const GRAVEYARD_REDUCTION: SpellReduction = SpellReduction {
    source_only: false,
    filter: CastFilter::Any,
    amount: SpellReductionAmount::Fixed(2),
    turn: CostTurn::Graveyard,
    guard: StepGuard::Always,
};

const GOBLIN_FORMULA: &[StaticAbilityGroup] = &[
    StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::CostReduction(0, GRAVEYARD_REDUCTION)],
    },
    StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::GrantManaCostMayhem],
    },
];

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    keywords: Keywords::FLYING.union(Keywords::MENACE),
    static_groups: GOBLIN_FORMULA,
    extras: &CardExtras {
        static_ability_groups: GOBLIN_FORMULA,
        cost_reductions: &[GRAVEYARD_REDUCTION],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
