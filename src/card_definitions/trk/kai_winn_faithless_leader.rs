// Oracle and official release notes: the name choice precedes entry and both named events count.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 1),
    extras: &CardExtras {
        entry_choice: Some(EntryChoiceKind::NonlandCardName),
        ..CardExtras::EMPTY
    },
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::OpponentCastsOrActivatesChosenName,
        targets: TargetPlan::NONE,
        program: &[Instruction::LoseLife(PlayerSet::EventPlayer, 3)],
    }],
    ..CardRules::EMPTY
};
