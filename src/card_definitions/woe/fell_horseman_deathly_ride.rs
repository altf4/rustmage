// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfDies,
        targets: TargetPlan::NONE,
        program: &[Instruction::PutDepartedSourceOnLibraryBottom],
    }],
    extras: &CardExtras {
        adventure_spell: Some(Card(5552)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
