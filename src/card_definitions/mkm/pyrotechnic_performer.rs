pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfOrControlledCreatureTurnedFaceUp,
        targets: TargetPlan::NONE,
        program: &[Instruction::EnteringPowerDamagePlayers(PlayerSet::Opponent)],
    }],
    extras: &CardExtras {
        disguise_cost: Some(ManaCost {
            colored: [0, 0, 0, 1, 0, 0],
            generic: 0,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
