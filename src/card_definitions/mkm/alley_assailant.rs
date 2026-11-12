pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    entry: EntryTapped::Always,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfTurnedFaceUp,
        targets: one(TargetRule::Player {
            opponent_only: true,
        }),
        program: &[
            Instruction::LoseLife(PlayerSet::Targeted, 3),
            Instruction::GainLife(PlayerSet::Controller, 3),
        ],
    }],
    extras: &CardExtras {
        disguise_cost: Some(ManaCost {
            colored: [0, 0, 2, 0, 0, 0],
            generic: 4,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
