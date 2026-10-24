// Complete typed Oracle behavior; see paired source provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(one(TargetRule::Creature), &[Instruction::Damage(5), Instruction::ExileIfDiesThisTurn]),
    static_groups: &[StaticAbilityGroup { line: 0, first_layer: AbilityLayer::Rule, fragments: &[StaticAbilityFragment::CostReduction(0, SpellReduction { source_only: true, filter: CastFilter::Any, amount: SpellReductionAmount::Fixed(2), turn: CostTurn::Any, guard: StepGuard::Expression { value: StatValue::CardLeftGraveyardThisTurn, minimum: Some(1), maximum: None } })] }],
    extras: &CardExtras { cost_reductions: &[SpellReduction { source_only: true, filter: CastFilter::Any, amount: SpellReductionAmount::Fixed(2), turn: CostTurn::Any, guard: StepGuard::Expression { value: StatValue::CardLeftGraveyardThisTurn, minimum: Some(1), maximum: None } }], static_ability_groups: &[StaticAbilityGroup { line: 0, first_layer: AbilityLayer::Rule, fragments: &[StaticAbilityFragment::CostReduction(0, SpellReduction { source_only: true, filter: CastFilter::Any, amount: SpellReductionAmount::Fixed(2), turn: CostTurn::Any, guard: StepGuard::Expression { value: StatValue::CardLeftGraveyardThisTurn, minimum: Some(1), maximum: None } })] }], ..CardExtras::EMPTY },
    ..CardRules::EMPTY
};
