// Direct typed rules from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 1 },
            targets: one(TargetRule::Creature),
            program: &[Instruction::Destroy],
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 2 },
            targets: TargetPlan::NONE,
            program: &[Instruction::MayDiscardOrLoseLife {
                players: PlayerSet::Each,
                amount: 3,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 3 },
            targets: TargetPlan::Single(TargetSpec {
                rule: TargetRule::Player {
                    opponent_only: false,
                },
                min: 0,
                max: 2,
            }),
            program: &[Instruction::ExileGraveyard(PlayerSet::Targeted)],
        },
    ],
    ..CardRules::EMPTY
};
