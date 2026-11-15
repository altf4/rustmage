// Oracle and pinned XMage reviewed. The entrant explores, including Janeway herself.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::CreatureEntersYou,
        targets: TargetPlan::NONE,
        program: &[Instruction::Explore(Subject::Entering)],
    }],
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::AdditionalLandPlays(1)],
    }],
    ..CardRules::EMPTY
};
