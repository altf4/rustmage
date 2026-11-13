// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 1 },
            targets: TargetPlan::NONE,
            program: &[Instruction::ExileOpposingPermanentsChosenByEachPlayer(3)],
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 2 },
            targets: one(TargetRule::Player {
                opponent_only: true,
            }),
            program: &[
                Instruction::May { skip: 2 },
                Instruction::Draw(PlayerSet::Targeted, 3),
                Instruction::Draw(PlayerSet::Controller, 3),
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 3 },
            targets: TargetPlan::NONE,
            program: &[Instruction::ExileBattlefieldSourceReturn {
                transformed: true,
                under_owner: false,
            }],
        },
    ],
    extras: &CardExtras {
        transforming_double_face: true,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
