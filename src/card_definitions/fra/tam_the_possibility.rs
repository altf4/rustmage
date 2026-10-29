// Pinned Oracle and XMage; gated pending proliferate replacement and interaction validation.
const REDUCTION: SpellReduction = SpellReduction {
    source_only: false, filter: CastFilter::Planeswalker,
    amount: SpellReductionAmount::Fixed(1), turn: CostTurn::Any, guard: StepGuard::Always,
};
const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0, first_layer: AbilityLayer::Rule,
    fragments: &[StaticAbilityFragment::CostReduction(0, REDUCTION)],
}];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 4),
    static_groups: GROUPS,
    activated: &[ActivatedAbility {
        exhaust: false, guard: ActivationGuard::Always, origin: ActivationOrigin::Battlefield,
        cycling: false, once_per_turn: false, sorcery: false, targets: TargetPlan::NONE,
        cost: ActivationCost {
            colored: [1, 1, 1, 1, 1, 0], generic: 0, alternatives: &[], additional: None,
            mana_component: true, tap: true, sacrifice: false, exile_self: false,
            discard: false, life: 0, remove: None, loyalty: None,
        },
        program: &[Instruction::Proliferate(AmountValue::Stat(StatValue::DistinctControlledPlaneswalkerTypes))],
    }],
    extras: &CardExtras { cost_reductions: &[REDUCTION], static_ability_groups: GROUPS, ..CardExtras::EMPTY },
    ..CardRules::EMPTY
};
