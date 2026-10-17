// Directly authored from the paired pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec {
            min: 0,
            max: 2,
            rule: TargetRule::Object {
                zone: TargetZone::Graveyard,
                kinds: 1,
                relation: 0,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            },
        }),
        &[Instruction::ReturnToHand],
    ),
    extras: &CardExtras {
        cost_reductions: &[SpellReduction {
            source_only: true,
            filter: CastFilter::Any,
            amount: SpellReductionAmount::Fixed(3),
            turn: CostTurn::Any,
            guard: StepGuard::Expression {
                value: StatValue::LifeGainedThisTurn,
                minimum: Some(3),
                maximum: None,
            },
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
