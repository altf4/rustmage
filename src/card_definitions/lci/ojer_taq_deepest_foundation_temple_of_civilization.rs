// The creation event multiplies only the recipes that have creature characteristics.
const TRIPLE_CREATURES: ReplacementSpec = ReplacementSpec::MultiplyTokens {
    kinds: 1,
    factor: 3,
};
const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 1,
    first_layer: AbilityLayer::Rule,
    fragments: &[StaticAbilityFragment::Replacement(0, TRIPLE_CREATURES)],
}];

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(6, 6),
    keywords: Keywords::VIGILANCE,
    static_groups: GROUPS,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfDies,
        targets: TargetPlan::NONE,
        program: &[Instruction::ReturnDepartedSourceTransformedTapped { counters: &[] }],
    }],
    extras: &CardExtras {
        transforming_double_face: true,
        static_ability_groups: GROUPS,
        replacement_effects: &[TRIPLE_CREATURES],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
