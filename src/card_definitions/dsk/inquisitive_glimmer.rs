// Directly authored from the paired Oracle and pinned XMage sources.
const ENCHANTMENT_REDUCTION: SpellReduction = SpellReduction {
    source_only: false,
    filter: CastFilter::Enchantment,
    amount: SpellReductionAmount::Fixed(1),
    turn: CostTurn::Any,
    guard: StepGuard::Always,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::CostReduction(
                0,
                ENCHANTMENT_REDUCTION,
            )],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::RoomUnlockReduction(1)],
        },
    ],
    extras: &CardExtras {
        cost_reductions: &[ENCHANTMENT_REDUCTION],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
