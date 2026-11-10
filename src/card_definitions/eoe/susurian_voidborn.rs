pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfOrControlledPermanentDies(17),
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Player {
            opponent_only: true,
        })),
        program: &[
            Instruction::LoseLife(PlayerSet::Targeted, 1),
            Instruction::GainLife(PlayerSet::Controller, 1),
        ],
    }],
    extras: &CardExtras {
        warp: Some(&WarpSpec::fixed(ManaCost {
            colored: [0, 0, 1, 0, 0, 0],
            generic: 0,
        })),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
