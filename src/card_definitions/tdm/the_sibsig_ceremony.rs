// Source-audited direct typed definition; see paired provenance.
const REDUCTION: SpellReduction = SpellReduction {
    source_only: false,
    filter: CastFilter::Creature,
    amount: SpellReductionAmount::Fixed(2),
    turn: CostTurn::Any,
    guard: StepGuard::Always,
};
const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Rule,
    fragments: &[StaticAbilityFragment::CostReduction(0, REDUCTION)],
}];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: GROUPS,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::CreatureEntersYouIfCast,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::DestroyEntering,
            Instruction::CreateTokens {
                token: Card(5290),
                amount: 1,
                tapped: false,
            },
        ],
    }],
    extras: &CardExtras {
        cost_reductions: &[REDUCTION],
        static_ability_groups: GROUPS,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
