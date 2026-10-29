// Draft from pinned Oracle and XMage; gated pending full rules validation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::NoSpellsOrNonmanaDuringCombat],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::ControlledCreatureAttacksPlayerAlone,
        targets: TargetPlan::NONE,
        program: &[Instruction::GrantKeywords {
            subject: Subject::Entering,
            keywords: Keywords::DOUBLE_STRIKE,
        }],
    }],
    ..CardRules::EMPTY
};
