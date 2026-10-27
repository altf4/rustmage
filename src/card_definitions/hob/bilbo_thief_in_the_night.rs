// Complete printed obligations from pinned Oracle and XMage sources.
const REDUCTION: SpellReduction = SpellReduction {
    source_only: false,
    filter: CastFilter::Any,
    amount: SpellReductionAmount::Fixed(1),
    turn: CostTurn::NonHand,
    guard: StepGuard::Always,
};
const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Rule,
    fragments: &[StaticAbilityFragment::CostReduction(0, REDUCTION)],
}];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfAttacks,
        targets: TargetPlan::NONE,
        program: &[Instruction::CastGraveyard {
            kinds: 16 | 4 | 8,
            exile_instants_sorceries: true,
        }],
    }],
    static_groups: GROUPS,
    extras: &CardExtras {
        static_ability_groups: GROUPS,
        cost_reductions: &[REDUCTION],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
