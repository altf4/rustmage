pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(8, 8),
    keywords: Keywords::FLYING,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::ChosenPlayerLosesGame,
        targets: TargetPlan::NONE,
        program: &[Instruction::WinGame],
    }],
    extras: &CardExtras {
        as_transform: Some(AsTransformEffect::ChooseOpponent),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
