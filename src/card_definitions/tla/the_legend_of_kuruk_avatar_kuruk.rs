// Direct typed behavior from the pinned Oracle and XMage sources.
const SCRY_DRAW: TriggeredAbility = TriggeredAbility {
    condition: TriggerCondition::Chapter { number: 1 },
    targets: TargetPlan::NONE,
    program: &[
        Instruction::Scry(2),
        Instruction::Draw(PlayerSet::Controller, 1),
    ],
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        SCRY_DRAW,
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 2 },
            ..SCRY_DRAW
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
