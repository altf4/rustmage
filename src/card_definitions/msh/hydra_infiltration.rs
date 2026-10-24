// Directly authored from the complete pinned Oracle and XMage implementation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: one(TargetRule::Player {
                opponent_only: true,
            }),
            program: &[Instruction::Discard(PlayerSet::Targeted, 2)],
        },
        TriggeredAbility {
            condition: TriggerCondition::ControlledCreatureAttacksAlone,
            targets: one(TargetRule::Player {
                opponent_only: true,
            }),
            program: &[
                Instruction::LoseLife(PlayerSet::Targeted, 1),
                Instruction::GainLife(PlayerSet::Controller, 1),
            ],
        },
    ],
    ..CardRules::EMPTY
};
