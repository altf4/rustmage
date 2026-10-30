// Direct Oracle and pinned XMage behavior; gated pending integration validation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(0, 7),
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::CostReduction(
                0,
                SpellReduction {
                    source_only: true,
                    filter: CastFilter::Any,
                    amount: SpellReductionAmount::GreatestCreatureToughness,
                    turn: CostTurn::Any,
                    guard: StepGuard::Always,
                },
            )],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::ControlledAttackWithoutDefender],
        },
        StaticAbilityGroup {
            line: 2,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::ControlledCombatDamageByGreaterToughness],
        },
    ],
    extras: &CardExtras {
        cost_reductions: &[SpellReduction {
            source_only: true,
            filter: CastFilter::Any,
            amount: SpellReductionAmount::GreatestCreatureToughness,
            turn: CostTurn::Any,
            guard: StepGuard::Always,
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
