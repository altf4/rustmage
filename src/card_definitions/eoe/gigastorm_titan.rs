// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
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
    keywords: Keywords::NONE,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::CostReduction(
            0,
            SpellReduction {
                source_only: true,
                filter: CastFilter::Any,
                amount: SpellReductionAmount::Fixed(3),
                turn: CostTurn::Any,
                guard: StepGuard::Threshold {
                    value: StepValue::SpellsCast,
                    minimum: Some(1),
                    maximum: None,
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
                    source_only: true,
                    filter: CastFilter::Any,
                    amount: SpellReductionAmount::Fixed(3),
                    turn: CostTurn::Any,
                    guard: StepGuard::Threshold {
                        value: StepValue::SpellsCast,
                        minimum: Some(1),
                        maximum: None,
                    },
                },
            )],
        }],
        cost_reductions: &[SpellReduction {
            source_only: true,
            filter: CastFilter::Any,
            amount: SpellReductionAmount::Fixed(3),
            turn: CostTurn::Any,
            guard: StepGuard::Threshold {
                value: StepValue::SpellsCast,
                minimum: Some(1),
                maximum: None,
            },
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
