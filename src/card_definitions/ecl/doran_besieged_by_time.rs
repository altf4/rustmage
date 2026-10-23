// Complete pinned Oracle and XMage source authored directly in typed Rust.
const REDUCTION: SpellReduction = SpellReduction {
    source_only: false,
    filter: CastFilter::CreatureToughnessGreaterThanPower,
    amount: SpellReductionAmount::Fixed(1),
    turn: CostTurn::Any,
    guard: StepGuard::Always,
};
const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Rule,
    fragments: &[StaticAbilityFragment::CostReduction(0, REDUCTION)],
}];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(0, 5),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::CreatureAttacksOrBlocksYou,
        targets: TargetPlan::NONE,
        program: &[Instruction::GrowCreatureActionPowerToughnessDifference],
    }],
    static_groups: GROUPS,
    extras: &CardExtras {
        cost_reductions: &[REDUCTION],
        static_ability_groups: GROUPS,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
