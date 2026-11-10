// Direct typed behavior from pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::ControlledEquippedCreatureKeywords(
            Keywords::HASTE,
        )],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::FilteredEntersYou(EntryPredicate::NontokenEquipment),
        targets: TargetPlan::NONE,
        program: &[Instruction::CopyEnteringWithEquipDiscount(2)],
    }],
    ..CardRules::EMPTY
};
