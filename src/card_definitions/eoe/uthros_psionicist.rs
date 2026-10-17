// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 4),
    triggers: &[],
    activated: &[],
    enters: Counters {
        plus_one: 0,
        minus_one: 0,
        stun: 0,
        loyalty: 0,
        lore: 0,
    },
    entry: EntryTapped::Never,
    keywords: Keywords(0),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::CostReduction(
            0,
            SpellReduction {
                source_only: false,
                filter: CastFilter::Any,
                amount: SpellReductionAmount::Fixed(2),
                turn: CostTurn::Any,
                guard: StepGuard::Threshold {
                    value: StepValue::SpellsCast,
                    minimum: Some(1),
                    maximum: Some(1),
                },
            },
        )],
    }],
    extras: &CardExtras {
        static_ability_groups: &[StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::CostReduction(
                0,
                SpellReduction {
                    source_only: false,
                    filter: CastFilter::Any,
                    amount: SpellReductionAmount::Fixed(2),
                    turn: CostTurn::Any,
                    guard: StepGuard::Threshold {
                        value: StepValue::SpellsCast,
                        minimum: Some(1),
                        maximum: Some(1),
                    },
                },
            )],
        }],
        cost_reductions: &[SpellReduction {
            source_only: false,
            filter: CastFilter::Any,
            amount: SpellReductionAmount::Fixed(2),
            turn: CostTurn::Any,
            guard: StepGuard::Threshold {
                value: StepValue::SpellsCast,
                minimum: Some(1),
                maximum: Some(1),
            },
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
