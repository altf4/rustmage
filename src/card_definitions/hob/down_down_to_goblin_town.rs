// Complete printed obligations from pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 1 },
            targets: one(TargetRule::Player {
                opponent_only: true,
            }),
            program: &[Instruction::RevealHandDiscard { excluded_kinds: 2 }],
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 2 },
            targets: TargetPlan::NONE,
            program: &[Instruction::EnsureGoblinArmy, Instruction::AmassGoblins(1)],
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 3 },
            targets: one(TargetRule::Player {
                opponent_only: true,
            }),
            program: &[
                Instruction::LoseLife(PlayerSet::Targeted, 1),
                Instruction::GainLife(PlayerSet::Controller, 1),
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 4 },
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
