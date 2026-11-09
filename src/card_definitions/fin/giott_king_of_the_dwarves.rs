// The entry filter matches Giott itself or a controlled Dwarf or Equipment.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    keywords: Keywords::DOUBLE_STRIKE,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::FilteredEntersYou(EntryPredicate::SourceOrSubtypeAny(&[
            "Dwarf",
            "Equipment",
        ])),
        targets: TargetPlan::NONE,
        program: &[Instruction::DiscardUpToDraw { maximum: 1 }],
    }],
    ..CardRules::EMPTY
};
