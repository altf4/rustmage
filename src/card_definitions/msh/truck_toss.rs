// Direct typed behavior from the complete pinned Oracle and XMage sources.
const REDUCTION: SpellReduction = SpellReduction {
    source_only: true,
    filter: CastFilter::Any,
    amount: SpellReductionAmount::Fixed(2),
    turn: CostTurn::Any,
    guard: StepGuard::Expression {
        value: StatValue::Count(CountFilter {
            zone: CountZone::Battlefield,
            relation: 0,
            kind: 0,
            exclude: 0,
            subtype: Some("Vehicle"),
            basic: false,
            other: false,
            cards_only: false,
        }),
        minimum: Some(1),
        maximum: None,
    },
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Any)),
        &[Instruction::Damage(4)],
    ),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::CostReduction(0, REDUCTION)],
    }],
    extras: &CardExtras {
        cost_reductions: &[REDUCTION],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
