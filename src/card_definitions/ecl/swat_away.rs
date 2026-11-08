// Direct typed behavior from complete pinned Oracle and XMage sources; Oracle governs the attack condition.
const REDUCTION: SpellReduction = SpellReduction {
    source_only: true,
    filter: CastFilter::Any,
    amount: SpellReductionAmount::Fixed(2),
    turn: CostTurn::Any,
    guard: StepGuard::Expression {
        value: StatValue::CreaturesAttackingYou,
        minimum: Some(1),
        maximum: None,
    },
};
const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Rule,
    fragments: &[StaticAbilityFragment::CostReduction(0, REDUCTION)],
}];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Union(&[
            TargetRule::Spell,
            TargetRule::Creature,
        ])),
        &[Instruction::OwnerLibrary],
    ),
    static_groups: GROUPS,
    extras: &CardExtras {
        cost_reductions: &[REDUCTION],
        static_ability_groups: GROUPS,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
