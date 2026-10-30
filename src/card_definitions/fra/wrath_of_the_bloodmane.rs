// Directly authored from pinned Oracle and XMage sources; admission awaits rules tests.
const REDUCTION: SpellReduction = SpellReduction {
    source_only: true,
    filter: CastFilter::Any,
    amount: SpellReductionAmount::Fixed(1),
    turn: CostTurn::Any,
    guard: StepGuard::Expression {
        value: StatValue::LegendaryCreaturesYouControl,
        minimum: Some(1),
        maximum: None,
    },
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 65,
            relation: -1,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        }),
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
