// Direct typed behavior from the paired Oracle and XMage sources.
const FLOOR: ReplacementSpec = ReplacementSpec::NoncombatDamageToOpponentAtLeastSourcePower {
    color: ChoiceColor::Red,
};
const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 1,
    first_layer: AbilityLayer::Rule,
    fragments: &[StaticAbilityFragment::Replacement(0, FLOOR)],
}];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    keywords: Keywords::TRAMPLE,
    static_groups: GROUPS,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfDies,
        targets: TargetPlan::NONE,
        program: &[Instruction::ReturnDepartedSourceTransformedTapped { counters: &[] }],
    }],
    extras: &CardExtras {
        transforming_double_face: true,
        static_ability_groups: GROUPS,
        replacement_effects: &[FLOOR],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
